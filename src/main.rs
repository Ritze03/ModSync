mod types;
mod modmanager;
mod ui;

use clap::Parser;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use anyhow::Context;
use eframe::NativeOptions;
use reqwest::Client;
use tokio::sync::mpsc::unbounded_channel;
use tokio::time::sleep;

use crate::types::ModEntry;
use crate::modmanager::{ModManager, SyncProgress};
use crate::ui::ModSyncApp;

#[derive(Parser, Debug)]
#[command(
    author,
    version,
    about = "ModSync Pre-Launch Tool\n\
           Syncs Minecraft mods before launch.\n\n\
           Mod list format:\n\
           # Category | ModName | DownloadURL | SHA256\n\
           - Category: REQUIRED or REMOVE\n\
             * REQUIRED: Automatically downloaded; required for the game to run.\n\
             * REMOVE: Deletes the specified mod from the local mods folder.\n\
           \n\
           - ModName: filename of the mod jar\n\
           - DownloadURL: URL to download the mod (ignored for REMOVE entries)\n\
           - SHA256: optional SHA256 hash of the file (ignored for REMOVE entries)"
)]

struct Args {
    /// URL of the remote mod list
    #[arg(long, conflicts_with = "modsfile")]
    modsurl: Option<String>,

    /// Local file containing the mod list
    #[arg(long, conflicts_with = "modsurl")]
    modsfile: Option<PathBuf>,

    /// Path to the modpack root (default: current dir)
    #[arg(long)]
    path: Option<PathBuf>,

    /// CLI-Mode
    #[arg(long)]
    cli: bool,

    /// Generate SHA256 hash of a file and exit
    #[arg(long, value_name = "FILE")]
    hash: Option<PathBuf>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    // SHA256 mode
    if let Some(file) = args.hash {
        let hash = ModManager::sha256_file(&file)?;
        println!("{}", hash);
        return Ok(());
    }

    // Determine mods dir
    let mods_dir = args.path.unwrap_or_else(|| std::env::current_dir().unwrap());
    println!("Mods directory: {}", mods_dir.display());

    // Load mod list
    let mod_entries: Vec<ModEntry> = ModManager::load_mod_entries(&args.modsfile, &args.modsurl).await?;
    println!("Loaded {} mods from list", mod_entries.len());

    // Setup progress
    let total = mod_entries.len();
    let progress = Arc::new(SyncProgress::new(total));

    // Setup events channel for UI
    let (event_tx, event_rx) = unbounded_channel();

    // Spawn background sync
    let mods_dir_clone = mods_dir.clone();
    let progress_clone = progress.clone();
    let mod_entries_clone = mod_entries.clone();

    tokio::spawn(async move {
        let _ = ModManager::sync_all_from_entries(
            mod_entries_clone,
            mods_dir_clone,
            Client::new(),
            progress_clone,
            Some(event_tx),
        ).await;
    });

    // Decide if we launch UI or splash mode
    if !args.cli {
        // Full UI mode
        let native_options = NativeOptions {
            viewport: egui::ViewportBuilder::default()
                .with_inner_size([400.0, 260.0])
                .with_resizable(false)
                .with_decorations(false)
                .with_title("ModSync"),
            ..Default::default()
        };

        eframe::run_native(
            "ModSync",
            native_options,
            Box::new(|cc| {
                Ok(Box::new(ModSyncApp::new(cc, progress, event_rx)))
            }),
        ).expect("Failed to launch UI");
    } else {
        loop {
            let processed = progress.processed();
            let total = progress.total;

            // Print live progress
            println!("Progress: {}/{}", processed, total);

            sleep(Duration::from_millis(250)).await;
        }
    }

    println!("Exiting ModSync. Minecraft launcher should start now.");
    Ok(())
}