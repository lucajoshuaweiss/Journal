//! # Journal
//!
//! A small command-line journaling application written in Rust.
//!
//! Journal stores entries locally as JSON files in:
//!
//! ```text
//! ~/.journal/
//! ```
//!
//! ## Getting started
//!
//! Create a new journal entry:
//!
//! ```text
//! journal new
//! ```
//!
//! List all existing entries:
//!
//! ```text
//! journal list
//! ```
//!
//! Read an entry by its ID:
//!
//! ```text
//! journal read <id>
//! ```
//!
//! Search your journal:
//!
//! ```text
//! journal search <expression>
//! ```
//!
//! Edit an existing entry:
//!
//! ```text
//! journal edit <id>
//! ```
//!
//! Delete an entry:
//!
//! ```text
//! journal delete <id>
//! ```
//!
//! ## Commands
//!
//! The commands are organized around the typical journal workflow.
//!
//! ### 1. Create an entry
//!
//! Use [`commands::new`] to create a new journal entry:
//!
//! ```text
//! journal new
//! ```
//!
//! The built-in editor opens and allows you to enter the journal text.
//!
//! ### 2. List entries
//!
//! Use [`commands::list`] to see all existing journal entries:
//!
//! ```text
//! journal list
//! ```
//!
//! The list shows each entry's ID, creation time, and last modification
//! time.
//!
//! ### 3. Read an entry
//!
//! Use [`commands::read`] to display the complete contents of an entry:
//!
//! ```text
//! journal read <id>
//! ```
//!
//! For example:
//!
//! ```text
//! journal read 3
//! ```
//!
//! ### 4. Search entries
//!
//! Use [`commands::search`] to find entries containing an expression:
//!
//! ```text
//! journal search rust
//! ```
//!
//! Searches are case-insensitive and support partial expressions.
//!
//! For example, searching for:
//!
//! ```text
//! rust
//! ```
//!
//! also finds text containing `Rust` or `RUST`.
//!
//! ### 5. Edit an entry
//!
//! Use [`commands::edit`] to modify an existing entry:
//!
//! ```text
//! journal edit <id>
//! ```
//!
//! The entry is opened in the built-in editor.
//!
//! ### 6. Delete an entry
//!
//! Use [`commands::delete`] to permanently remove an entry:
//!
//! ```text
//! journal delete <id>
//! ```
//!
//! Deletion requires explicit confirmation.
//!
//! ## Built-in editor
//!
//! `journal new` and `journal edit` use a small line-oriented editor built
//! into the application.
//!
//! No external editor such as Vim or Nano is required.
//!
//! Normal input is added to the journal entry. Lines beginning with `:` are
//! interpreted as editor commands.
//!
//! ### Editor commands
//!
//! ```text
//! :help             Show this help
//! :show             Show the current entry
//! :edit N TEXT      Replace line N
//! :insert N TEXT    Insert a new line before line N
//! :delete N         Delete line N
//! :clear            Clear the entire entry
//! :w                Save and continue editing
//! :q                Quit without saving changes since the last save
//! :wq               Save and quit
//! ```
//!
//! Line numbers start at 1.
//!
//! ### Saving and quitting
//!
//! The editor distinguishes between saving and quitting.
//!
//! `:w` saves the current version and continues editing.
//!
//! `:q` quits and returns the most recently saved version. Changes made
//! after the last save are discarded.
//!
//! `:wq` saves the current version and quits immediately.
//!
//! ## Data model
//!
//! Each journal entry contains:
//!
//! - a unique numeric ID
//! - its creation timestamp
//! - its last modification timestamp
//! - its journal text
//!
//! See [`models::JournalEntry`] for the entry data structure.
//!
//! ## Storage
//!
//! Each journal entry is stored as an individual JSON file:
//!
//! ```text
//! ~/.journal/1.json
//! ~/.journal/2.json
//! ~/.journal/3.json
//! ```
//!
//! The storage implementation is contained in [`storage`], keeping
//! persistence details separate from the rest of the application.
//!
//! ## Testing
//!
//! Run all unit tests with:
//!
//! ```text
//! cargo test
//! ```
//!
//! The project contains tests for:
//!
//! - command-line argument parsing
//! - journal entry creation and modification
//! - journal entry storage
//! - journal searching
//! - editor commands and editing behavior
//!
//! ## Documentation
//!
//! Generate and open the project documentation with:
//!
//! ```text
//! cargo doc --no-deps --open
//! ```
//!
//! ## Building
//!
//! Build the application in debug mode:
//!
//! ```text
//! cargo build
//! ```
//!
//! Build an optimized release version:
//!
//! ```text
//! cargo build --release
//! ```
//!
//! ## Project structure
//!
//! The main modules are:
//!
//! - [`cli`] — command-line argument parsing
//! - [`commands`] — implementations of the journal commands
//! - [`editor`] — the built-in line-oriented editor
//! - [`models`] — journal data structures
//! - [`storage`] — persistence of journal entries

pub mod cli;
pub mod commands;
pub mod editor;
pub mod models;
pub mod storage;
