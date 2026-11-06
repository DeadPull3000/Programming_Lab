# Assignment 1: Library Trie & CLI

This directory contains the first lab assignment: building a high-performance, in-memory 26-way prefix tree (Trie) dictionary in Rust, and an interactive command-line interface.

## Crates

1. **`trie/`**: Core shared dictionary library crate.
   - 26-way child pointers via `[Option<Box<Node>>; 26]`.
   - $O(L)$ lookups and insertions.
   - Case normalization and ASCII validation.
   - Recursive prefix-preserving deletion (`remove`).
   - Read-only, zero-allocation `TrieCursor` for step-by-step game navigation.
   - 32 Unit tests and 3 differential randomized tests against `std::collections::HashSet`.
2. **`trie-cli/`**: Interactive REPL application.
   - Interactive commands: `find`, `insert`, `delete`, `found`, `missing`, `load`, `check-file`, `stats`, `help`, `quit`.
   - Delimited parsing of files (one-word-per-line, comma-separated, space-separated, mixed).
   - Candidate word classification into found and missing sets.

## Running

From the root directory:

```powershell
# Run the CLI
cargo run -p trie-cli

# Run library tests
cargo test -p trie

# Run CLI tests
cargo test -p trie-cli
```
