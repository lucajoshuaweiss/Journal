# Journal
A small command-line journaling application written in Rust.

```text
A simple command-line journaling application

Usage: journal <COMMAND>

Commands:
  new     Create a new journal entry
  list    List existing journal entries
  search  Search journal entries for an expression
  read    Read a journal entry
  edit    Edit an existing journal entry
  delete  Delete a journal entry
  help    Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version
```

Journal entries are stored locally as JSON files in:

```text
~/.journal/
```

## Testing
Run all tests:

```text
cargo test
```

## Documentation
Generate documentation for this project without its dependencies:

```text
cargo doc --no-deps --open
```

## Building
Build the application:

```text
cargo build --release
```

You will find the compiled program under "target/release/journal"

# Note
This was tested on a linux machine.