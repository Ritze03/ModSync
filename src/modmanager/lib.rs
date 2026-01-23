use anyhow::{Context, Result};
use crate::types::ModEntry;

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};

use sha2::{Digest, Sha256};
use reqwest::Client;
use tokio::sync::mpsc::UnboundedSender;
use futures::{stream, StreamExt};

/// Final report of a sync operation
#[derive(Debug, Clone)]
pub struct SyncReport {
    pub downloaded: Vec<ModEntry>,
    pub unchanged: Vec<ModEntry>,
    pub removed: Vec<ModEntry>,
    pub failed: Vec<(ModEntry, String)>,
}

/// Shared progress state (UI-readable at any time)
#[derive(Debug)]
pub struct SyncProgress {
    pub total: usize,
    pub processed: AtomicUsize,
    pub downloaded: AtomicUsize,
    pub unchanged: AtomicUsize,
    pub removed: AtomicUsize,
    pub failed: AtomicUsize,

    // Keep the last processed mod for UI
    last_mod: parking_lot::Mutex<Option<String>>,
}

impl SyncProgress {
    pub fn new(total: usize) -> Self {
        Self {
            total,
            processed: AtomicUsize::new(0),
            downloaded: AtomicUsize::new(0),
            unchanged: AtomicUsize::new(0),
            removed: AtomicUsize::new(0),
            failed: AtomicUsize::new(0),
            last_mod: parking_lot::Mutex::new(None),
        }
    }

    /// Total number of mods processed so far
    pub fn processed(&self) -> usize {
        self.downloaded.load(Ordering::Relaxed)
            + self.unchanged.load(Ordering::Relaxed)
            + self.removed.load(Ordering::Relaxed)
            + self.failed.load(Ordering::Relaxed)
    }

    /// Get current counts for UI display
    pub fn stats(&self) -> SyncStats {
        SyncStats {
            downloaded: self.downloaded.load(Ordering::Relaxed),
            unchanged: self.unchanged.load(Ordering::Relaxed),
            removed: self.removed.load(Ordering::Relaxed),
            failed: self.failed.load(Ordering::Relaxed),
        }
    }

    /// Update the last processed mod (called from sync_all_from_entries)
    pub fn set_last_mod(&self, filename: String) {
        let mut lock = self.last_mod.lock();
        *lock = Some(filename);
    }

    /// Retrieve the last processed mod for UI
    pub fn last_processed(&self) -> Option<String> {
        self.last_mod.lock().clone()
    }
}

/// Simple struct for UI to read current numbers
pub struct SyncStats {
    pub downloaded: usize,
    pub unchanged: usize,
    pub removed: usize,
    pub failed: usize,
}


/// Optional real-time events for UI
#[derive(Debug, Clone)]
pub enum SyncEvent {
    Downloaded { filename: String },
    Unchanged { filename: String },
    Removed { filename: String },
    Failed { filename: String, error: String },
    Finished(SyncReport),
}

pub struct ModManager;

impl ModManager {
    pub async fn load_mod_entries(file: &Option<PathBuf>, url: &Option<String>) -> anyhow::Result<Vec<ModEntry>> {
        let text = if let Some(f) = file {
            std::fs::read_to_string(f).context("Failed to read modsfile")?
        } else if let Some(u) = url {
            reqwest::get(u).await?.text().await.context("Failed to fetch mods list")?
        } else {
            anyhow::bail!("Missing --modsurl or --modsfile argument");
        };

        Ok(text.lines().filter_map(|l| crate::types::parse_line(l)).collect())
    }

    /// Main sync entry point (parallel, UI-ready)
    pub async fn sync_all_from_entries(
        mod_entries: Vec<ModEntry>,
        mods_dir: PathBuf,
        client: Client,
        progress: Arc<SyncProgress>,
        event_tx: Option<UnboundedSender<SyncEvent>>,
    ) -> Result<SyncReport> {
        let mods_folder = mods_dir.join("mods");
        if !mods_folder.exists() {
            fs::create_dir_all(&mods_folder)
                .context("Failed to create mods folder")?;
        }

        let results = stream::iter(mod_entries)
            .map(|entry| {
                let progress = progress.clone();
                let tx = event_tx.clone();
                let client = client.clone();
                let mods_folder = mods_folder.clone();

                async move {
                    Self::handle_entry(
                        entry,
                        &mods_folder,
                        &client,
                        progress,
                        tx,
                    ).await
                }
            })
            .buffer_unordered(8) // parallelism limit
            .collect::<Vec<_>>()
            .await;


        let mut downloaded = Vec::new();
        let mut unchanged = Vec::new();
        let mut removed = Vec::new();
        let mut failed = Vec::new();


        for result in results {
            match result {
                EntryResult::Downloaded(e) => downloaded.push(e),
                EntryResult::Unchanged(e) => unchanged.push(e),
                EntryResult::Removed(e) => removed.push(e),
                EntryResult::Failed(e, msg) => failed.push((e, msg)),
            }
        }

        println!("Downloaded: {:?}\n", downloaded);
        println!("Unchanged: {:?}\n", unchanged);
        println!("Removed: {:?}\n", removed);
        println!("Failed: {:?}\n", failed);

        let report = SyncReport {
            downloaded,
            unchanged,
            removed,
            failed,
        };

        if let Some(tx) = &event_tx {
            let _ = tx.send(SyncEvent::Finished(report.clone()))?;
        }

        Ok(report)
    }

    async fn handle_entry(
        entry: ModEntry,
        mods_folder: &Path,
        client: &Client,
        progress: Arc<SyncProgress>,
        event_tx: Option<UnboundedSender<SyncEvent>>,
    ) -> EntryResult {
        let filename = entry.filename.clone();
        let local_path = mods_folder.join(&filename);

        let result = if entry.category.eq_ignore_ascii_case("REMOVE") {
            // REMOVE category: delete if exists
            if local_path.exists() {
                match fs::remove_file(&local_path) {
                    Ok(_) => {
                        progress.removed.fetch_add(1, Ordering::Relaxed);
                        send_event(&event_tx, SyncEvent::Removed { filename: filename.clone() });
                        EntryResult::Removed(entry)
                    }
                    Err(e) => {
                        progress.failed.fetch_add(1, Ordering::Relaxed);
                        send_event(&event_tx, SyncEvent::Failed {
                            filename: filename.clone(),
                            error: e.to_string(),
                        });
                        EntryResult::Failed(entry, e.to_string())
                    }
                }
            } else {
                progress.unchanged.fetch_add(1, Ordering::Relaxed);
                send_event(&event_tx, SyncEvent::Unchanged { filename: filename.clone() });
                EntryResult::Unchanged(entry)
            }
        } else {
            // Required mod, or optional selected: always check
            match ModManager::check_and_download(&entry, mods_folder, client).await {
                Ok(true) => {
                    // Downloaded (new file or hash mismatch)
                    progress.downloaded.fetch_add(1, Ordering::Relaxed);
                    send_event(&event_tx, SyncEvent::Downloaded { filename: filename.clone() });
                    EntryResult::Downloaded(entry)
                }
                Ok(false) => {
                    // File exists and hash matches
                    progress.unchanged.fetch_add(1, Ordering::Relaxed);
                    send_event(&event_tx, SyncEvent::Unchanged { filename: filename.clone() });
                    EntryResult::Unchanged(entry)
                }
                Err(e) => {
                    progress.failed.fetch_add(1, Ordering::Relaxed);
                    send_event(&event_tx, SyncEvent::Failed { filename: filename.clone(), error: e.to_string() });
                    EntryResult::Failed(entry, e.to_string())
                }
            }
        };

        progress.set_last_mod(filename);
        progress.processed.fetch_add(1, Ordering::Relaxed);
        result
    }


    async fn check_and_download(
        entry: &ModEntry,
        mods_folder: &Path,
        client: &Client,
    ) -> Result<bool> {
        let local_path = mods_folder.join(&entry.filename);

        if local_path.exists() {
            if let Some(expected) = &entry.sha256 {
                let actual = Self::sha256_file(&local_path)?;
                if actual.eq_ignore_ascii_case(expected) {
                    return Ok(false);
                } else {
                    anyhow::bail!(
                        "SHA256 mismatch for {} (expected {}, got {})",
                        entry.filename,
                        expected,
                        actual
                    );
                }
            }
            return Ok(false);
        }

        Self::download_mod(entry, &local_path, client).await?;
        Ok(true)
    }

    async fn download_mod(
        entry: &ModEntry,
        local_path: &Path,
        client: &Client,
    ) -> Result<()> {
        let bytes = client
            .get(&entry.url)
            .send()
            .await
            .context(format!("Failed to download {}", entry.filename))?
            .bytes()
            .await
            .context(format!("Failed to read response for {}", entry.filename))?;

        fs::write(local_path, &bytes)
            .context(format!("Failed to write {}", entry.filename))?;

        if let Some(expected) = &entry.sha256 {
            let actual = Self::sha256_file(local_path)?;
            if !actual.eq_ignore_ascii_case(expected) {
                anyhow::bail!(
                    "SHA256 mismatch for {} (expected {}, got {})",
                    entry.filename,
                    expected,
                    actual
                );
            }
        }

        Ok(())
    }

    pub(crate) fn sha256_file(path: &Path) -> Result<String> {
        let data = fs::read(path).context("Failed to read file for hashing")?;
        let mut hasher = Sha256::new();
        hasher.update(&data);
        Ok(format!("{:x}", hasher.finalize()))
    }
}

/// Internal per-entry result
enum EntryResult {
    Downloaded(ModEntry),
    Unchanged(ModEntry),
    Removed(ModEntry),
    Failed(ModEntry, String),
}

fn send_event(tx: &Option<UnboundedSender<SyncEvent>>, event: SyncEvent) {
    if let Some(tx) = tx {
        let _ = tx.send(event);
    }
}
