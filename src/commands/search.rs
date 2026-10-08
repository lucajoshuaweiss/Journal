//! Implementation of the `journal search` command.
//!
//! Searching is case-insensitive and matches partial expressions.
//!
//! The command searches the text of every stored journal entry and displays
//! all matching entries in ascending entry ID order.
//!
//! For example:
//!
//! ```text
//! journal search rust
//! ```
//!
//! finds entries containing `rust`, `Rust`, `RUST`, or another case
//! variation.
//!
//! ## Search behavior
//!
//! Search expressions:
//!
//! - are case-insensitive
//! - can match part of a word
//! - match anywhere in the journal entry text
//! - return every matching entry
//!
//! An empty expression matches every entry.

use anyhow::Result;
use std::path::Path;

use crate::{models::JournalEntry, storage};

/// Searches journal entries for an expression.
///
/// The search is case-insensitive and matches partial expressions anywhere
/// in the entry text.
///
/// The returned entries preserve the order of the supplied `entries` slice.
/// The storage layer supplies entries sorted by ID, so [`run`] displays
/// matching entries in ascending ID order.
///
/// # Examples
///
/// ```
/// use journal::commands::search::search_entries;
/// use journal::models::JournalEntry;
///
/// let entries = vec![
///     JournalEntry::new(1, "Today I learned Rust.".to_string()),
///     JournalEntry::new(2, "I went for a walk.".to_string()),
/// ];
///
/// let matches = search_entries(&entries, "rust");
///
/// assert_eq!(matches.len(), 1);
/// assert_eq!(matches[0].id, 1);
/// ```
///
/// # Case-insensitive matching
///
/// ```
/// use journal::commands::search::search_entries;
/// use journal::models::JournalEntry;
///
/// let entries = vec![
///     JournalEntry::new(1, "Today I learned Rust.".to_string()),
/// ];
///
/// assert_eq!(search_entries(&entries, "rust").len(), 1);
/// assert_eq!(search_entries(&entries, "RUST").len(), 1);
/// assert_eq!(search_entries(&entries, "RuSt").len(), 1);
/// ```
///
/// # Partial expressions
///
/// ```
/// use journal::commands::search::search_entries;
/// use journal::models::JournalEntry;
///
/// let entries = vec![
///     JournalEntry::new(1, "Rust programming is interesting.".to_string()),
/// ];
///
/// let matches = search_entries(&entries, "gram");
///
/// assert_eq!(matches.len(), 1);
/// assert_eq!(matches[0].id, 1);
/// ```
pub fn search_entries<'a>(entries: &'a [JournalEntry], expression: &str) -> Vec<&'a JournalEntry> {
    let expression_lower = expression.to_lowercase();

    entries
        .iter()
        .filter(|entry| entry.text.to_lowercase().contains(&expression_lower))
        .collect()
}

/// Searches all stored journal entries and displays matching entries.
///
/// The search is case-insensitive and matches partial expressions.
///
/// Results are displayed in ascending entry ID order because entries loaded
/// from [`storage::list_entries`] are sorted by ID.
pub fn run(dir: &Path, expression: &str) -> Result<()> {
    let entries = storage::list_entries(dir)?;
    let matches = search_entries(&entries, expression);

    if matches.is_empty() {
        println!("No journal entries found matching \"{expression}\".");
        return Ok(());
    }

    println!("Journal entries matching \"{expression}\":");
    println!();

    for entry in matches {
        println!("#{} — {}", entry.id, entry.text);
        println!();
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::search_entries;
    use crate::models::JournalEntry;

    fn test_entries() -> Vec<JournalEntry> {
        vec![
            JournalEntry::new(1, "Today I learned Rust.".to_string()),
            JournalEntry::new(2, "I worked on Rust ownership.".to_string()),
            JournalEntry::new(3, "I went for a walk.".to_string()),
        ]
    }

    #[test]
    fn search_finds_matching_entries() {
        let entries = test_entries();

        let matches = search_entries(&entries, "Rust");

        assert_eq!(matches.len(), 2);
        assert_eq!(matches[0].id, 1);
        assert_eq!(matches[1].id, 2);
    }

    #[test]
    fn search_is_case_insensitive() {
        let entries = vec![JournalEntry::new(1, "Today I learned Rust.".to_string())];

        assert_eq!(search_entries(&entries, "rust").len(), 1);
        assert_eq!(search_entries(&entries, "RUST").len(), 1);
        assert_eq!(search_entries(&entries, "RuSt").len(), 1);
    }

    #[test]
    fn search_does_not_match_unrelated_entries() {
        let entries = test_entries();

        let matches = search_entries(&entries, "coffee");

        assert!(matches.is_empty());
    }

    #[test]
    fn search_matches_partial_expressions() {
        let entries = vec![JournalEntry::new(
            1,
            "I learned something interesting today.".to_string(),
        )];

        let matches = search_entries(&entries, "interest");

        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].id, 1);
    }

    #[test]
    fn search_matches_expressions_inside_longer_words() {
        let entries = vec![
            JournalEntry::new(1, "Rust programming is interesting.".to_string()),
            JournalEntry::new(2, "I went outside.".to_string()),
        ];

        let matches = search_entries(&entries, "gram");

        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].id, 1);
    }

    #[test]
    fn search_returns_entries_in_existing_order() {
        let entries = vec![
            JournalEntry::new(5, "Rust is great.".to_string()),
            JournalEntry::new(2, "Rust is fast.".to_string()),
            JournalEntry::new(9, "No match.".to_string()),
        ];

        let matches = search_entries(&entries, "rust");

        assert_eq!(matches.len(), 2);
        assert_eq!(matches[0].id, 5);
        assert_eq!(matches[1].id, 2);
    }

    #[test]
    fn search_with_empty_expression_matches_all_entries() {
        let entries = test_entries();

        let matches = search_entries(&entries, "");

        assert_eq!(matches.len(), entries.len());
    }

    #[test]
    fn search_handles_empty_entries() {
        let entries: Vec<JournalEntry> = Vec::new();

        let matches = search_entries(&entries, "rust");

        assert!(matches.is_empty());
    }
}
