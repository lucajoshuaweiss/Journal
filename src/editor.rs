//! A small line-oriented command-line editor.
//!
//! The editor intentionally provides a much smaller feature set than
//! full-screen editors such as Vim or Emacs.
//!
//! Normal input is treated as journal text. Lines beginning with `:` are
//! interpreted as editor commands.
//!
//! Supported commands:
//!
//! ```text
//! :help             Show help
//! :show             Display the current text
//! :edit N TEXT      Replace line N
//! :insert N TEXT    Insert a new line before line N
//! :delete N         Delete line N
//! :clear            Remove all text
//! :w                Save and continue editing
//! :q                Quit without saving changes since the last save
//! :wq               Save and quit
//! ```
//!
//! Line numbers are 1-based.
//!
//! The editor's state and command processing are separated from terminal
//! input so that the core behavior can be tested without an interactive
//! terminal.
//!
//! # Examples
//!
//! ```
//! use journal::editor::{Editor, EditorAction};
//!
//! let mut editor = Editor::new("Hello".to_string());
//!
//! assert_eq!(editor.text(), "Hello");
//!
//! let action = editor.process_line("World");
//!
//! assert_eq!(action, EditorAction::Continue);
//! assert_eq!(editor.text(), "Hello\nWorld");
//! ```

use anyhow::{Context, Result};
use std::io::{self, Write};

/// Action requested by an editor command.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EditorAction {
    Continue,

    /// Save the current text and continue editing.
    ///
    /// The caller of [`run`] is responsible for actually persisting the text.
    Save,

    /// Quit the editor.
    ///
    /// If text was previously saved with [`EditorAction::Save`], the caller
    /// can return that saved version. Changes made after the last save are
    /// discarded.
    Quit,

    /// Save the current text and quit.
    ///
    /// The caller receives the final text from [`run`].
    SaveAndQuit,
}

/// Result of executing an editor command.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CommandResult {
    Action(EditorAction),

    Message(String),

    UnknownCommand(String),
}

/// The in-memory state of the journal editor.
///
/// Lines are stored individually so that existing lines can be replaced,
/// inserted, or deleted without having to manipulate the complete text as
/// one large string.
#[derive(Debug, Clone)]
pub struct Editor {
    lines: Vec<String>,
}

impl Editor {
    /// Creates an editor from existing text.
    ///
    /// The supplied text is split into lines.
    ///
    /// # Examples
    ///
    /// ```
    /// use journal::editor::Editor;
    ///
    /// let editor = Editor::new("one\ntwo\nthree".to_string());
    ///
    /// assert_eq!(editor.line_count(), 3);
    /// assert_eq!(editor.text(), "one\ntwo\nthree");
    /// ```
    pub fn new(text: String) -> Self {
        let lines = if text.is_empty() {
            Vec::new()
        } else {
            text.lines().map(String::from).collect()
        };

        Self { lines }
    }

    /// Returns the complete editor contents.
    ///
    /// # Examples
    ///
    /// ```
    /// use journal::editor::Editor;
    ///
    /// let editor = Editor::new("one\ntwo".to_string());
    ///
    /// assert_eq!(editor.text(), "one\ntwo");
    /// ```
    pub fn text(&self) -> String {
        self.lines.join("\n")
    }

    /// Returns the number of lines currently in the editor.
    pub fn line_count(&self) -> usize {
        self.lines.len()
    }

    /// Adds a new line to the end of the editor.
    ///
    /// # Examples
    ///
    /// ```
    /// use journal::editor::Editor;
    ///
    /// let mut editor = Editor::new("one".to_string());
    ///
    /// editor.add_line("two");
    ///
    /// assert_eq!(editor.text(), "one\ntwo");
    /// ```
    pub fn add_line(&mut self, line: impl Into<String>) {
        self.lines.push(line.into());
    }

    /// Inserts a new line before an existing line.
    ///
    /// Line numbers are 1-based. To insert at the end of the entry, use a
    /// line number equal to `line_count() + 1`.
    ///
    /// # Examples
    ///
    /// ```
    /// use journal::editor::Editor;
    ///
    /// let mut editor = Editor::new("one\nthree".to_string());
    ///
    /// editor.insert_line(2, "two").unwrap();
    ///
    /// assert_eq!(editor.text(), "one\ntwo\nthree");
    /// ```
    pub fn insert_line(&mut self, line_number: usize, text: impl Into<String>) -> Result<()> {
        if line_number == 0 || line_number > self.lines.len() + 1 {
            anyhow::bail!(
                "Line number {} is out of range (1-{})",
                line_number,
                self.lines.len() + 1
            );
        }

        self.lines.insert(line_number - 1, text.into());

        Ok(())
    }

    /// Replaces an existing line.
    ///
    /// Line numbers are 1-based.
    ///
    /// # Examples
    ///
    /// ```
    /// use journal::editor::Editor;
    ///
    /// let mut editor = Editor::new("one\ntwo\nthree".to_string());
    ///
    /// editor.edit_line(2, "changed").unwrap();
    ///
    /// assert_eq!(editor.text(), "one\nchanged\nthree");
    /// ```
    pub fn edit_line(&mut self, line_number: usize, text: impl Into<String>) -> Result<()> {
        if line_number == 0 || line_number > self.lines.len() {
            anyhow::bail!(
                "Line number {} is out of range (1-{})",
                line_number,
                self.lines.len()
            );
        }

        self.lines[line_number - 1] = text.into();

        Ok(())
    }

    /// Deletes an existing line.
    ///
    /// Line numbers are 1-based.
    ///
    /// # Examples
    ///
    /// ```
    /// use journal::editor::Editor;
    ///
    /// let mut editor = Editor::new("one\ntwo\nthree".to_string());
    ///
    /// editor.delete_line(2).unwrap();
    ///
    /// assert_eq!(editor.text(), "one\nthree");
    /// ```
    pub fn delete_line(&mut self, line_number: usize) -> Result<()> {
        if line_number == 0 || line_number > self.lines.len() {
            anyhow::bail!(
                "Line number {} is out of range (1-{})",
                line_number,
                self.lines.len()
            );
        }

        self.lines.remove(line_number - 1);

        Ok(())
    }

    /// Removes all text from the editor.
    ///
    /// This clears both lines loaded from the existing journal and lines
    /// added during the current editing session.
    ///
    /// # Examples
    ///
    /// ```
    /// use journal::editor::Editor;
    ///
    /// let mut editor = Editor::new("one\ntwo".to_string());
    ///
    /// editor.add_line("three");
    /// editor.clear();
    ///
    /// assert_eq!(editor.text(), "");
    /// assert_eq!(editor.line_count(), 0);
    /// ```
    pub fn clear(&mut self) {
        self.lines.clear();
    }

    /// Interprets a line without modifying editor state.
    ///
    /// This method determines what a line means without executing commands
    /// that modify the editor.
    ///
    /// Normal text produces [`EditorAction::Continue`].
    ///
    /// Commands that modify the editor, such as `:clear` or `:edit`, are
    /// intentionally not executed here.
    ///
    /// # Examples
    ///
    /// ```
    /// use journal::editor::{
    ///     CommandResult,
    ///     Editor,
    ///     EditorAction,
    /// };
    ///
    /// let editor = Editor::new(String::new());
    ///
    /// assert_eq!(
    ///     editor.command(":w"),
    ///     CommandResult::Action(EditorAction::Save)
    /// );
    ///
    /// assert_eq!(
    ///     editor.command(":q"),
    ///     CommandResult::Action(EditorAction::Quit)
    /// );
    ///
    /// assert_eq!(
    ///     editor.command(":wq"),
    ///     CommandResult::Action(EditorAction::SaveAndQuit)
    /// );
    /// ```
    pub fn command(&self, line: &str) -> CommandResult {
        let trimmed = line.trim();

        match trimmed {
            ":help" => CommandResult::Message(help_text()),

            ":show" => CommandResult::Message(self.display_text()),

            ":clear" => CommandResult::Action(EditorAction::Continue),

            ":w" => CommandResult::Action(EditorAction::Save),

            ":q" => CommandResult::Action(EditorAction::Quit),

            ":wq" => CommandResult::Action(EditorAction::SaveAndQuit),

            command if command.starts_with(':') => {
                CommandResult::UnknownCommand(command.to_string())
            }

            _ => CommandResult::Action(EditorAction::Continue),
        }
    }

    /// Executes one line of editor input.
    ///
    /// Normal lines are appended to the entry. Commands beginning with
    /// `:` are executed.
    ///
    /// # Examples
    ///
    /// ```
    /// use journal::editor::{
    ///     CommandResult,
    ///     Editor,
    ///     EditorAction,
    /// };
    ///
    /// let mut editor = Editor::new(String::new());
    ///
    /// let result = editor.execute_command("Hello");
    ///
    /// assert_eq!(
    ///     result,
    ///     CommandResult::Action(EditorAction::Continue)
    /// );
    ///
    /// assert_eq!(editor.text(), "Hello");
    /// ```
    pub fn execute_command(&mut self, line: &str) -> CommandResult {
        let trimmed = line.trim();

        match trimmed {
            ":help" => CommandResult::Message(help_text()),

            ":show" => CommandResult::Message(self.display_text()),

            ":edit" => CommandResult::Message("Usage: :edit <line> <text>".to_string()),

            command if command.starts_with(":edit ") => {
                let arguments = command.strip_prefix(":edit ").unwrap_or("");

                let mut parts = arguments.splitn(2, char::is_whitespace);

                let line_number = parts.next().unwrap_or("");
                let new_text = parts.next().unwrap_or("").trim_start();

                match line_number.parse::<usize>() {
                    Ok(line_number) => match self.edit_line(line_number, new_text) {
                        Ok(()) => CommandResult::Message(format!("Line {} updated.", line_number)),

                        Err(error) => CommandResult::Message(error.to_string()),
                    },

                    Err(_) => CommandResult::Message("Usage: :edit <line> <text>".to_string()),
                }
            }

            ":insert" => CommandResult::Message("Usage: :insert <line> <text>".to_string()),

            command if command.starts_with(":insert ") => {
                let arguments = command.strip_prefix(":insert ").unwrap_or("");

                let mut parts = arguments.splitn(2, char::is_whitespace);

                let line_number = parts.next().unwrap_or("");
                let new_text = parts.next().unwrap_or("").trim_start();

                match line_number.parse::<usize>() {
                    Ok(line_number) => match self.insert_line(line_number, new_text) {
                        Ok(()) => CommandResult::Message(format!("Line {} inserted.", line_number)),

                        Err(error) => CommandResult::Message(error.to_string()),
                    },

                    Err(_) => CommandResult::Message("Usage: :insert <line> <text>".to_string()),
                }
            }

            ":delete" => CommandResult::Message("Usage: :delete <line>".to_string()),

            command if command.starts_with(":delete ") => {
                let line_number = command.strip_prefix(":delete ").unwrap_or("").trim();

                match line_number.parse::<usize>() {
                    Ok(line_number) => match self.delete_line(line_number) {
                        Ok(()) => CommandResult::Message(format!("Line {} deleted.", line_number)),

                        Err(error) => CommandResult::Message(error.to_string()),
                    },

                    Err(_) => CommandResult::Message("Usage: :delete <line>".to_string()),
                }
            }

            ":clear" => {
                self.clear();

                CommandResult::Message("Entry cleared.".to_string())
            }

            ":w" => CommandResult::Action(EditorAction::Save),

            ":q" => CommandResult::Action(EditorAction::Quit),

            ":wq" => CommandResult::Action(EditorAction::SaveAndQuit),

            command if command.starts_with(':') => {
                CommandResult::UnknownCommand(command.to_string())
            }

            _ => {
                self.add_line(line);

                CommandResult::Action(EditorAction::Continue)
            }
        }
    }

    /// Processes one line of user input.
    ///
    /// This is a convenience method around [`Editor::execute_command`].
    ///
    /// # Examples
    ///
    /// ```
    /// use journal::editor::{Editor, EditorAction};
    ///
    /// let mut editor = Editor::new(String::new());
    ///
    /// assert_eq!(
    ///     editor.process_line("Hello"),
    ///     EditorAction::Continue
    /// );
    ///
    /// assert_eq!(
    ///     editor.process_line(":wq"),
    ///     EditorAction::SaveAndQuit
    /// );
    ///
    /// assert_eq!(editor.text(), "Hello");
    /// ```
    pub fn process_line(&mut self, line: &str) -> EditorAction {
        match self.execute_command(line) {
            CommandResult::Action(action) => action,

            CommandResult::Message(_) => EditorAction::Continue,

            CommandResult::UnknownCommand(_) => EditorAction::Continue,
        }
    }

    /// Formats the current text with line numbers for display.
    fn display_text(&self) -> String {
        if self.lines.is_empty() {
            return "(empty entry)".to_string();
        }

        self.lines
            .iter()
            .enumerate()
            .map(|(index, line)| format!("{:>4} | {}", index + 1, line))
            .collect::<Vec<_>>()
            .join("\n")
    }
}

/// Runs the interactive terminal editor.
///
/// The supplied text is displayed in the editor and can then be modified.
///
/// The returned string is:
///
/// - the most recently saved text if the user uses `:w` and then `:q`;
/// - the current text if the user uses `:wq`;
/// - `None` if the user uses `:q` without ever saving.
///
/// This means changes made after `:w` are discarded when the user later
/// chooses `:q`.
///
/// The actual persistence of the journal belongs to the caller of this
/// function.
///
/// # Examples
///
/// ```no_run
/// use journal::editor::run;
///
/// let result = run("Existing journal text".to_string())?;
///
/// if let Some(text) = result {
///     println!("Saved: {text}");
/// }
///
/// # Ok::<(), anyhow::Error>(())
/// ```
pub fn run(initial_text: String) -> Result<Option<String>> {
    let mut editor = Editor::new(initial_text);

    // Stores the most recently explicitly saved version.
    //
    // `None` means the user has not used :w or :wq yet.
    let mut saved_text: Option<String> = None;

    println!();
    println!("Journal editor");
    println!("Type :help for commands.");
    println!();

    if editor.line_count() > 0 {
        println!("{}", editor.display_text());
        println!();
    }

    loop {
        print!("> ");

        io::stdout()
            .flush()
            .context("Could not flush editor prompt")?;

        let mut input = String::new();

        let bytes_read = io::stdin()
            .read_line(&mut input)
            .context("Could not read editor input")?;

        if bytes_read == 0 {
            return Ok(saved_text);
        }

        let line = input.trim_end_matches(['\r', '\n']);

        match editor.execute_command(line) {
            CommandResult::Action(EditorAction::Continue) => {}

            CommandResult::Action(EditorAction::Save) => {
                // Remember the version that was explicitly saved.
                //
                // The caller owns the actual persistence layer. This
                // snapshot allows :q to return the version that was saved
                // by the most recent :w.
                saved_text = Some(editor.text());

                println!("Entry saved.");
            }

            CommandResult::Action(EditorAction::Quit) => {
                // Return the most recently saved version, if one exists.
                //
                // Any changes made after the last :w are discarded.
                return Ok(saved_text);
            }

            CommandResult::Action(EditorAction::SaveAndQuit) => {
                return Ok(Some(editor.text()));
            }

            CommandResult::Message(message) => {
                println!("{message}");
            }

            CommandResult::UnknownCommand(command) => {
                println!("Unknown command: {command}");
                println!("Type :help for available commands.");
            }
        }
    }
}

/// Returns the help text displayed by `:help`.
///
/// # Examples
///
/// ```
/// use journal::editor::help_text;
///
/// let help = help_text();
///
/// assert!(help.contains(":help"));
/// assert!(help.contains(":show"));
/// assert!(help.contains(":edit"));
/// assert!(help.contains(":insert"));
/// assert!(help.contains(":delete"));
/// assert!(help.contains(":clear"));
/// assert!(help.contains(":w"));
/// assert!(help.contains(":q"));
/// assert!(help.contains(":wq"));
/// ```
pub fn help_text() -> String {
    [
        "Editor commands:",
        "",
        "  :help             Show this help",
        "  :show             Show the current entry",
        "  :edit N TEXT      Replace line N",
        "  :insert N TEXT    Insert a new line before line N",
        "  :delete N         Delete line N",
        "  :clear            Clear the entire entry",
        "  :w                Save and continue editing",
        "  :q                Quit without saving changes since the last save",
        "  :wq               Save and quit",
        "",
        "Line numbers start at 1.",
        "",
        "Any other line is added to the journal.",
    ]
    .join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_editor_has_zero_lines() {
        let editor = Editor::new(String::new());

        assert_eq!(editor.line_count(), 0);
        assert_eq!(editor.text(), "");
    }

    #[test]
    fn existing_text_is_split_into_lines() {
        let editor = Editor::new("one\ntwo\nthree".to_string());

        assert_eq!(editor.line_count(), 3);
        assert_eq!(editor.text(), "one\ntwo\nthree");
    }

    #[test]
    fn adding_line_appends_text() {
        let mut editor = Editor::new("first".to_string());

        editor.add_line("second");

        assert_eq!(editor.text(), "first\nsecond");
    }

    #[test]
    fn insert_line_before_existing_line() {
        let mut editor = Editor::new("one\nthree".to_string());

        editor.insert_line(2, "two").unwrap();

        assert_eq!(editor.text(), "one\ntwo\nthree");
    }

    #[test]
    fn insert_line_at_beginning() {
        let mut editor = Editor::new("two\nthree".to_string());

        editor.insert_line(1, "one").unwrap();

        assert_eq!(editor.text(), "one\ntwo\nthree");
    }

    #[test]
    fn insert_line_at_end() {
        let mut editor = Editor::new("one\ntwo".to_string());

        editor.insert_line(3, "three").unwrap();

        assert_eq!(editor.text(), "one\ntwo\nthree");
    }

    #[test]
    fn insert_line_into_empty_editor() {
        let mut editor = Editor::new(String::new());

        editor.insert_line(1, "first").unwrap();

        assert_eq!(editor.text(), "first");
    }

    #[test]
    fn edit_existing_line() {
        let mut editor = Editor::new("one\ntwo\nthree".to_string());

        editor.edit_line(2, "changed").unwrap();

        assert_eq!(editor.text(), "one\nchanged\nthree");
    }

    #[test]
    fn edit_first_line() {
        let mut editor = Editor::new("one\ntwo\nthree".to_string());

        editor.edit_line(1, "changed").unwrap();

        assert_eq!(editor.text(), "changed\ntwo\nthree");
    }

    #[test]
    fn edit_last_line() {
        let mut editor = Editor::new("one\ntwo\nthree".to_string());

        editor.edit_line(3, "changed").unwrap();

        assert_eq!(editor.text(), "one\ntwo\nchanged");
    }

    #[test]
    fn edit_line_rejects_zero() {
        let mut editor = Editor::new("one\ntwo".to_string());

        assert!(editor.edit_line(0, "changed").is_err());
        assert_eq!(editor.text(), "one\ntwo");
    }

    #[test]
    fn edit_line_rejects_line_past_end() {
        let mut editor = Editor::new("one\ntwo".to_string());

        assert!(editor.edit_line(3, "changed").is_err());
        assert_eq!(editor.text(), "one\ntwo");
    }

    #[test]
    fn delete_existing_line() {
        let mut editor = Editor::new("one\ntwo\nthree".to_string());

        editor.delete_line(2).unwrap();

        assert_eq!(editor.text(), "one\nthree");
    }

    #[test]
    fn delete_first_line() {
        let mut editor = Editor::new("one\ntwo\nthree".to_string());

        editor.delete_line(1).unwrap();

        assert_eq!(editor.text(), "two\nthree");
    }

    #[test]
    fn delete_last_line() {
        let mut editor = Editor::new("one\ntwo\nthree".to_string());

        editor.delete_line(3).unwrap();

        assert_eq!(editor.text(), "one\ntwo");
    }

    #[test]
    fn delete_line_rejects_zero() {
        let mut editor = Editor::new("one\ntwo".to_string());

        assert!(editor.delete_line(0).is_err());
        assert_eq!(editor.text(), "one\ntwo");
    }

    #[test]
    fn delete_line_rejects_line_past_end() {
        let mut editor = Editor::new("one\ntwo".to_string());

        assert!(editor.delete_line(3).is_err());
        assert_eq!(editor.text(), "one\ntwo");
    }

    #[test]
    fn clear_removes_all_existing_and_new_text() {
        let mut editor = Editor::new("old one\nold two".to_string());

        editor.add_line("new one");
        editor.add_line("new two");

        editor.clear();

        assert_eq!(editor.text(), "");
        assert_eq!(editor.line_count(), 0);
    }

    #[test]
    fn normal_input_is_added() {
        let mut editor = Editor::new(String::new());

        let action = editor.process_line("Hello world");

        assert_eq!(action, EditorAction::Continue);
        assert_eq!(editor.text(), "Hello world");
    }

    #[test]
    fn save_command_is_recognized() {
        let editor = Editor::new(String::new());

        assert_eq!(
            editor.command(":w"),
            CommandResult::Action(EditorAction::Save)
        );
    }

    #[test]
    fn quit_command_is_recognized() {
        let editor = Editor::new(String::new());

        assert_eq!(
            editor.command(":q"),
            CommandResult::Action(EditorAction::Quit)
        );
    }

    #[test]
    fn save_and_quit_command_is_recognized() {
        let editor = Editor::new(String::new());

        assert_eq!(
            editor.command(":wq"),
            CommandResult::Action(EditorAction::SaveAndQuit)
        );
    }

    #[test]
    fn edit_command_changes_existing_line() {
        let mut editor = Editor::new("one\ntwo\nthree".to_string());

        let result = editor.execute_command(":edit 2 changed");

        assert_eq!(
            result,
            CommandResult::Message("Line 2 updated.".to_string())
        );

        assert_eq!(editor.text(), "one\nchanged\nthree");
    }

    #[test]
    fn edit_command_preserves_spaces_in_text() {
        let mut editor = Editor::new("one\ntwo".to_string());

        editor.execute_command(":edit 2 this is the new text");

        assert_eq!(editor.text(), "one\nthis is the new text");
    }

    #[test]
    fn edit_command_requires_line_number_and_text() {
        let mut editor = Editor::new("one\ntwo".to_string());

        let result = editor.execute_command(":edit");

        assert_eq!(
            result,
            CommandResult::Message("Usage: :edit <line> <text>".to_string())
        );

        assert_eq!(editor.text(), "one\ntwo");
    }

    #[test]
    fn edit_command_rejects_invalid_line_number() {
        let mut editor = Editor::new("one\ntwo".to_string());

        let result = editor.execute_command(":edit abc changed");

        assert_eq!(
            result,
            CommandResult::Message("Usage: :edit <line> <text>".to_string())
        );

        assert_eq!(editor.text(), "one\ntwo");
    }

    #[test]
    fn edit_command_rejects_out_of_range_line() {
        let mut editor = Editor::new("one\ntwo".to_string());

        let result = editor.execute_command(":edit 3 changed");

        assert_eq!(
            result,
            CommandResult::Message("Line number 3 is out of range (1-2)".to_string())
        );

        assert_eq!(editor.text(), "one\ntwo");
    }

    #[test]
    fn insert_command_adds_line() {
        let mut editor = Editor::new("one\nthree".to_string());

        let result = editor.execute_command(":insert 2 two");

        assert_eq!(
            result,
            CommandResult::Message("Line 2 inserted.".to_string())
        );

        assert_eq!(editor.text(), "one\ntwo\nthree");
    }

    #[test]
    fn delete_command_removes_line() {
        let mut editor = Editor::new("one\ntwo\nthree".to_string());

        let result = editor.execute_command(":delete 2");

        assert_eq!(
            result,
            CommandResult::Message("Line 2 deleted.".to_string())
        );

        assert_eq!(editor.text(), "one\nthree");
    }

    #[test]
    fn clear_command_removes_all_existing_and_new_text() {
        let mut editor = Editor::new("old one\nold two".to_string());

        editor.process_line("new one");
        editor.process_line("new two");
        editor.process_line(":clear");

        assert_eq!(editor.text(), "");
        assert_eq!(editor.line_count(), 0);
    }

    #[test]
    fn show_command_returns_current_text() {
        let editor = Editor::new("Hello\nWorld".to_string());

        let result = editor.command(":show");

        match result {
            CommandResult::Message(message) => {
                assert!(message.contains("Hello"));
                assert!(message.contains("World"));
            }

            _ => panic!("expected show message"),
        }
    }

    #[test]
    fn show_command_displays_line_numbers() {
        let editor = Editor::new("Hello\nWorld".to_string());

        let result = editor.command(":show");

        match result {
            CommandResult::Message(message) => {
                assert!(message.contains("1 | Hello"));
                assert!(message.contains("2 | World"));
            }

            _ => panic!("expected show message"),
        }
    }

    #[test]
    fn help_command_returns_help() {
        let editor = Editor::new(String::new());

        let result = editor.command(":help");

        match result {
            CommandResult::Message(message) => {
                assert!(message.contains(":help"));
                assert!(message.contains(":show"));
                assert!(message.contains(":edit"));
                assert!(message.contains(":insert"));
                assert!(message.contains(":delete"));
                assert!(message.contains(":clear"));
                assert!(message.contains(":w"));
                assert!(message.contains(":q"));
                assert!(message.contains(":wq"));
            }

            _ => panic!("expected help message"),
        }
    }

    #[test]
    fn unknown_command_is_reported() {
        let editor = Editor::new(String::new());

        assert_eq!(
            editor.command(":foobar"),
            CommandResult::UnknownCommand(":foobar".to_string())
        );
    }

    #[test]
    fn process_line_returns_save_and_quit() {
        let mut editor = Editor::new("Hello".to_string());

        let action = editor.process_line(":wq");

        assert_eq!(action, EditorAction::SaveAndQuit);
        assert_eq!(editor.text(), "Hello");
    }

    #[test]
    fn process_line_does_not_add_editor_commands() {
        let mut editor = Editor::new(String::new());

        editor.process_line(":help");
        editor.process_line(":show");
        editor.process_line(":w");
        editor.process_line(":q");
        editor.process_line(":wq");

        assert_eq!(editor.text(), "");
    }

    #[test]
    fn multiline_editing_works() {
        let mut editor = Editor::new(String::new());

        editor.process_line("First line");
        editor.process_line("Second line");
        editor.process_line("Third line");

        assert_eq!(editor.text(), "First line\nSecond line\nThird line");
    }

    #[test]
    fn complete_editing_workflow() {
        let mut editor = Editor::new("First line\nSecond line\nThird line".to_string());

        editor.process_line(":edit 2 Changed second line");
        editor.process_line("Fourth line");
        editor.process_line(":insert 3 Inserted line");
        editor.process_line(":delete 4");

        assert_eq!(
            editor.text(),
            "First line\nChanged second line\nInserted line\nFourth line"
        );
    }

    #[test]
    fn save_then_quit_preserves_saved_text() {
        let mut editor = Editor::new("First line".to_string());

        editor.process_line("Second line");
        assert_eq!(editor.process_line(":w"), EditorAction::Save);

        let saved_text = editor.text();

        editor.process_line("Unsaved line");

        assert_eq!(editor.text(), "First line\nSecond line\nUnsaved line");

        assert_eq!(editor.process_line(":q"), EditorAction::Quit);

        assert_eq!(editor.text(), "First line\nSecond line\nUnsaved line");

        assert_eq!(saved_text, "First line\nSecond line");
    }

    #[test]
    fn multiple_saves_update_the_saved_version() {
        let mut editor = Editor::new("First".to_string());

        editor.process_line(":w");

        let first_saved_text = editor.text();

        editor.process_line("Second");
        editor.process_line(":w");

        let second_saved_text = editor.text();

        editor.process_line("Third");

        assert_eq!(first_saved_text, "First");
        assert_eq!(second_saved_text, "First\nSecond");
        assert_eq!(editor.text(), "First\nSecond\nThird");

        assert_eq!(editor.process_line(":q"), EditorAction::Quit);
    }

    #[test]
    fn quit_without_saving_is_still_quit() {
        let mut editor = Editor::new("Existing journal".to_string());

        editor.process_line("Unsaved change");

        assert_eq!(editor.process_line(":q"), EditorAction::Quit);

        assert_eq!(
            editor.command(":q"),
            CommandResult::Action(EditorAction::Quit)
        );
    }

    #[test]
    fn save_and_quit_returns_save_and_quit_action() {
        let mut editor = Editor::new("First".to_string());

        editor.process_line("Second");

        assert_eq!(editor.process_line(":wq"), EditorAction::SaveAndQuit);

        assert_eq!(editor.text(), "First\nSecond");
    }
}
