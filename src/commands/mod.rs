//! Implementations of the journal CLI commands.
//!
//! The commands are presented below in the order that best reflects the
//! typical workflow of the application:
//!
//! 1. [`new`] — Create a new journal entry.
//! 2. [`list`] — List existing journal entries.
//! 3. [`read`] — Read an existing journal entry.
//! 4. [`search`] — Search journal entries.
//! 5. [`edit`] — Edit an existing journal entry.
//! 6. [`delete`] — Delete a journal entry.
//!
//! Rustdoc displays the individual modules alphabetically in its module
//! index. The ordering above is therefore provided explicitly here so that
//! the intended command workflow remains clear in the documentation.

pub mod delete;
pub mod edit;
pub mod list;
pub mod new;
pub mod read;
pub mod search;
