//! Command-line interface definitions.
//!
//! This module contains argument parsing but does not execute commands.

use clap::{Parser, Subcommand};

/// Command-line arguments for the journal application.
#[derive(Debug, Parser)]
#[command(
    name = "journal",
    version,
    about = "A simple command-line journaling application"
)]
pub struct Cli {
    /// The command to execute.
    #[command(subcommand)]
    pub command: Commands,
}

/// Commands supported by the journal application.
///
/// Commands are organized roughly by their normal workflow:
///
/// - create an entry
/// - inspect existing entries
/// - search entries
/// - read an individual entry
/// - edit an entry
/// - delete an entry
#[derive(Debug, Subcommand)]
pub enum Commands {
    New,

    List,

    Search {
        expression: String,
    },

    Read {
        id: u64,
    },

    Edit {
        id: u64,
    },

    Delete {
        id: u64,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_new_command() {
        let cli = Cli::try_parse_from(["journal", "new"]).expect("new command should parse");

        assert!(matches!(cli.command, Commands::New));
    }

    #[test]
    fn parses_list_command() {
        let cli = Cli::try_parse_from(["journal", "list"]).expect("list command should parse");

        assert!(matches!(cli.command, Commands::List));
    }

    #[test]
    fn parses_search_command() {
        let cli = Cli::try_parse_from(["journal", "search", "rust"])
            .expect("search command should parse");

        assert!(matches!(
            cli.command,
            Commands::Search { expression } if expression == "rust"
        ));
    }

    #[test]
    fn parses_read_command() {
        let cli =
            Cli::try_parse_from(["journal", "read", "42"]).expect("read command should parse");

        assert!(matches!(cli.command, Commands::Read { id: 42 }));
    }

    #[test]
    fn parses_edit_command() {
        let cli = Cli::try_parse_from(["journal", "edit", "7"]).expect("edit command should parse");

        assert!(matches!(cli.command, Commands::Edit { id: 7 }));
    }

    #[test]
    fn parses_delete_command() {
        let cli =
            Cli::try_parse_from(["journal", "delete", "10"]).expect("delete command should parse");

        assert!(matches!(cli.command, Commands::Delete { id: 10 }));
    }
}
