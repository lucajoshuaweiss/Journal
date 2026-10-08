//! Implementation of the `journal list` command.

use anyhow::Result;
use chrono::Local;
use std::path::Path;

use crate::storage;

/// Lists all journal entries.
pub fn run(dir: &Path) -> Result<()> {
    let entries = storage::list_entries(dir)?;

    if entries.is_empty() {
        println!("No journal entries yet.");
        return Ok(());
    }

    println!("{:<6} {:<22} {:<22}", "ID", "Created", "Updated");
    println!("{}", "-".repeat(52));

    for entry in entries {
        let created = entry
            .created_at
            .with_timezone(&Local)
            .format("%Y-%m-%d %H:%M");

        let updated = entry
            .updated_at
            .with_timezone(&Local)
            .format("%Y-%m-%d %H:%M");

        println!("{:<6} {:<22} {:<22}", entry.id, created, updated);
    }

    Ok(())
}
