//! Implementation of the `journal read` command.

use anyhow::Result;
use chrono::Local;
use std::path::Path;

use crate::storage;

/// Displays a journal entry.
pub fn run(dir: &Path, id: u64) -> Result<()> {
    let entry = storage::load_entry(dir, id)?;

    let created = entry
        .created_at
        .with_timezone(&Local)
        .format("%Y-%m-%d %H:%M");

    let updated = entry
        .updated_at
        .with_timezone(&Local)
        .format("%Y-%m-%d %H:%M");

    println!();
    println!("Journal Entry #{id}");
    println!("Created: {created}");
    println!("Updated: {updated}");
    println!("{}", "=".repeat(60));
    println!();
    println!("{}", entry.text);
    println!();

    Ok(())
}
