use anyhow::Result;
use clap::Parser;

use journal::{
    cli::{Cli, Commands},
    commands, storage,
};

fn main() -> Result<()> {
    let cli = Cli::parse();

    let journal_dir = storage::journal_directory()?;
    storage::initialize(&journal_dir)?;

    match cli.command {
        Commands::New => commands::new::run(&journal_dir)?,
        Commands::List => commands::list::run(&journal_dir)?,
        Commands::Search { expression } => commands::search::run(&journal_dir, &expression)?,
        Commands::Read { id } => commands::read::run(&journal_dir, id)?,
        Commands::Edit { id } => commands::edit::run(&journal_dir, id)?,
        Commands::Delete { id } => commands::delete::run(&journal_dir, id)?,
    }

    Ok(())
}
