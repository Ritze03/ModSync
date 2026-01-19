use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct ModEntry {
    pub filename: String,
    pub url: String,
    pub sha256: Option<String>,
    pub category: String, // "REQUIRED", "Optional", "Shaders", etc.
}

impl ModEntry {
    /// Returns true if this mod is in the reserved REQUIRED category
    pub fn is_required(&self) -> bool {
        self.category.eq_ignore_ascii_case("REQUIRED")
    }

    pub fn local_path(&self, mods_dir: &Path) -> PathBuf {
        mods_dir.join("mods").join(&self.filename)
    }
}

/// Parses a line from the modsync config file into a ModEntry
pub fn parse_line(line: &str) -> Option<ModEntry> {
    let line = line.trim();
    if line.is_empty() || line.starts_with('#') {
        return None;
    }

    let parts: Vec<&str> = line.split('|').collect();
    let category = parts.get(0)?.to_string();
    let filename = parts.get(1)?.to_string();
    let url = parts.get(2)?.to_string();
    let sha256 = parts.get(3).map(|s| s.to_string());

    Some(ModEntry {
        filename,
        url,
        sha256,
        category,
    })
}

