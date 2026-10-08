//! Implementation of the `journal edit` command.

use anyhow::Result;
use std::path::Path;

use crate::{editor, storage};

/// Opens an existing journal entry in the built-in editor.
pub fn run(dir: &Path, id: u64) -> Result<()> {
    let mut entry = storage::load_entry(dir, id)?;

    println!("Editing journal entry #{id}");

    let result = editor::run(entry.text.clone())?;

    let Some(text) = result else {
        println!("Changes discarded.");
        return Ok(());
    };

    if text.trim().is_empty() {
        println!("Empty entry. Existing entry was not changed.");
        return Ok(());
    }

    entry.update_text(text);

    storage::save_entry(dir, &entry)?;

    println!("Journal entry #{id} updated.");

    Ok(())
}
