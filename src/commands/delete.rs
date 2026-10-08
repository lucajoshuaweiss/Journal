//! Implementation of the `journal delete` command.

use anyhow::Result;
use std::{
    io::{self, Write},
    path::Path,
};

use crate::storage;

/// Deletes a journal entry after confirmation.
pub fn run(dir: &Path, id: u64) -> Result<()> {
    let entry = storage::load_entry(dir, id)?;

    println!("Are you sure you want to delete journal entry #{id}?");
    println!("This cannot be undone.");
    print!("Type 'yes' to confirm: ");

    io::stdout().flush()?;

    let mut confirmation = String::new();

    io::stdin().read_line(&mut confirmation)?;

    if confirmation.trim() != "yes" {
        println!("Nothing deleted.");
        return Ok(());
    }

    storage::delete_entry(dir, entry.id)?;

    println!("Journal entry #{id} deleted.");

    Ok(())
}
