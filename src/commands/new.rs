//! Implementation of the `journal new` command.

use anyhow::Result;
use std::path::Path;

use crate::{editor, models::JournalEntry, storage};

/// Creates a new journal entry.
///
/// The built-in editor is opened and the entry is saved when the user
/// executes `:w` or `:wq`.
pub fn run(dir: &Path) -> Result<()> {
    let id = storage::next_id(dir)?;

    println!("Creating journal entry #{id}");

    let result = editor::run(String::new())?;

    let Some(text) = result else {
        println!("Changes discarded.");
        return Ok(());
    };

    if text.trim().is_empty() {
        println!("Empty entry. Nothing saved.");
        return Ok(());
    }

    let entry = JournalEntry::new(id, text);

    storage::save_entry(dir, &entry)?;

    println!("Journal entry #{id} saved.");

    Ok(())
}
