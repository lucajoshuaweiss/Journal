//! Data structures used by the journal application.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// A single journal entry.
///
/// Each entry has a stable numeric ID and records both its creation
/// and last modification timestamps.
///
/// # Examples
///
/// ```
/// use journal::models::JournalEntry;
///
/// let entry = JournalEntry::new(
///     1,
///     "Today I learned Rust.".to_string(),
/// );
///
/// assert_eq!(entry.id, 1);
/// assert_eq!(entry.text, "Today I learned Rust.");
/// assert_eq!(entry.created_at, entry.updated_at);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JournalEntry {
    pub id: u64,

    pub created_at: DateTime<Utc>,

    pub updated_at: DateTime<Utc>,

    pub text: String,
}

impl JournalEntry {
    pub fn new(id: u64, text: String) -> Self {
        let now = Utc::now();

        Self {
            id,
            created_at: now,
            updated_at: now,
            text,
        }
    }

    pub fn update_text(&mut self, text: String) {
        self.text = text;
        self.updated_at = Utc::now();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn creates_entry_with_correct_values() {
        let entry = JournalEntry::new(5, "Hello".to_string());

        assert_eq!(entry.id, 5);
        assert_eq!(entry.text, "Hello");
        assert_eq!(entry.created_at, entry.updated_at);
    }

    #[test]
    fn updating_text_changes_text() {
        let mut entry = JournalEntry::new(1, "Old".to_string());

        entry.update_text("New".to_string());

        assert_eq!(entry.text, "New");
    }
}
