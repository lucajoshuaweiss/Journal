//! Persistence for journal entries.
//!
//! Each journal entry is stored as an individual JSON file:
//!
//! ```text
//! ~/.journal/1.json
//! ~/.journal/2.json
//! ~/.journal/3.json
//! ```

use anyhow::{Context, Result};
use std::{
    fs::{self, File},
    path::{Path, PathBuf},
};

use crate::models::JournalEntry;

/// Returns the directory used to store journal entries.
///
/// The default location is:
///
/// ```text
/// ~/.journal
/// ```
pub fn journal_directory() -> Result<PathBuf> {
    let home = dirs::home_dir().context("Could not determine home directory")?;

    Ok(home.join(".journal"))
}

/// Creates the journal directory if it doesn't already exist.
pub fn initialize(dir: &Path) -> Result<()> {
    fs::create_dir_all(dir).context("Could not create journal directory")?;

    Ok(())
}

/// Returns the path for a journal entry with the given ID.
fn entry_path(dir: &Path, id: u64) -> PathBuf {
    dir.join(format!("{id}.json"))
}

/// Saves an entry as JSON.
pub fn save_entry(dir: &Path, entry: &JournalEntry) -> Result<()> {
    let path = entry_path(dir, entry.id);

    let file = File::create(&path).with_context(|| format!("Could not create {}", path.display()))?;

    serde_json::to_writer_pretty(file, entry).context("Could not serialize journal entry")?;

    Ok(())
}

/// Loads an entry by ID.
pub fn load_entry(dir: &Path, id: u64) -> Result<JournalEntry> {
    let path = entry_path(dir, id);

    let file = File::open(&path).with_context(|| format!("Journal entry {id} does not exist"))?;

    serde_json::from_reader(file).with_context(|| format!("Could not parse journal entry {id}"))
}

/// Deletes an entry by ID.
pub fn delete_entry(dir: &Path, id: u64) -> Result<()> {
    let path = entry_path(dir, id);

    fs::remove_file(&path).with_context(|| format!("Could not delete journal entry {id}"))?;

    Ok(())
}

/// Returns all journal entries sorted by ID.
pub fn list_entries(dir: &Path) -> Result<Vec<JournalEntry>> {
    let mut entries = Vec::new();

    for item in fs::read_dir(dir)? {
        let item = item?;
        let path = item.path();

        if path.extension().and_then(|x| x.to_str()) != Some("json") {
            continue;
        }

        let file = File::open(&path)?;

        let entry: JournalEntry = serde_json::from_reader(file)
            .with_context(|| format!("Could not parse {}", path.display()))?;

        entries.push(entry);
    }

    entries.sort_by_key(|entry| entry.id);

    Ok(entries)
}

/// Calculates the next available entry ID.
pub fn next_id(dir: &Path) -> Result<u64> {
    let entries = list_entries(dir)?;

    Ok(entries.iter().map(|entry| entry.id).max().unwrap_or(0) + 1)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn test_entry(id: u64) -> JournalEntry {
        JournalEntry::new(id, format!("Journal entry {id}"))
    }

    #[test]
    fn initialize_creates_directory() {
        let root = tempdir().unwrap();
        let journal_dir = root.path().join("journal");

        initialize(&journal_dir).unwrap();

        assert!(journal_dir.exists());
        assert!(journal_dir.is_dir());
    }

    #[test]
    fn save_and_load_entry() {
        let dir = tempdir().unwrap();
        let entry = test_entry(1);

        save_entry(dir.path(), &entry).unwrap();

        let loaded = load_entry(dir.path(), 1).unwrap();

        assert_eq!(entry, loaded);
    }

    #[test]
    fn list_entries_returns_entries_sorted_by_id() {
        let dir = tempdir().unwrap();

        save_entry(dir.path(), &test_entry(3)).unwrap();
        save_entry(dir.path(), &test_entry(1)).unwrap();
        save_entry(dir.path(), &test_entry(2)).unwrap();

        let entries = list_entries(dir.path()).unwrap();

        assert_eq!(entries.len(), 3);
        assert_eq!(entries[0].id, 1);
        assert_eq!(entries[1].id, 2);
        assert_eq!(entries[2].id, 3);
    }

    #[test]
    fn next_id_returns_one_for_empty_directory() {
        let dir = tempdir().unwrap();

        assert_eq!(next_id(dir.path()).unwrap(), 1);
    }

    #[test]
    fn next_id_returns_next_available_id() {
        let dir = tempdir().unwrap();

        save_entry(dir.path(), &test_entry(2)).unwrap();
        save_entry(dir.path(), &test_entry(5)).unwrap();

        assert_eq!(next_id(dir.path()).unwrap(), 6);
    }

    #[test]
    fn delete_entry_removes_file() {
        let dir = tempdir().unwrap();
        let entry = test_entry(1);

        save_entry(dir.path(), &entry).unwrap();

        assert!(entry_path(dir.path(), 1).exists());

        delete_entry(dir.path(), 1).unwrap();

        assert!(!entry_path(dir.path(), 1).exists());
    }

    #[test]
    fn loading_missing_entry_returns_error() {
        let dir = tempdir().unwrap();

        let result = load_entry(dir.path(), 999);

        assert!(result.is_err());
    }
}
