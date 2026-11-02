# Terminal Execution Evidence Record

This document records the exact, unmanipulated terminal command outputs captured during build, check, test, linting, dictionary analysis, and interactive CLI execution.

---

## 1. Environment & Toolchain Identification

```powershell
PS C:\Users\User\Downloads\Programming Lab\Word Games\word-trie> rustc --version; cargo --version
rustc 1.99.0 (b940084d7 2026-09-28)
cargo 1.99.0 (5f94df478 2026-08-27)
```

---

## 2. Cargo Workspace Dependency Tree

```powershell
PS C:\Users\User\Downloads\Programming Lab\Word Games\word-trie> cargo tree --workspace
trie v0.1.0 (C:\Users\User\Downloads\Programming Lab\Word Games\word-trie\trie)

trie-cli v0.1.0 (C:\Users\User\Downloads\Programming Lab\Word Games\word-trie\trie-cli)
└── trie v0.1.0 (C:\Users\User\Downloads\Programming Lab\Word Games\word-trie\trie)
```
*Note: Zero third-party dependencies are utilized; only standard-library facilities are linked.*

---

## 3. Cargo Workspace Metadata Summary

```text
Workspace root: C:\Users\User\Downloads\Programming Lab\Word Games\word-trie
Resolver: 2
Packages:
  - trie v0.1.0 (edition 2021)
    Targets: trie (lib), differential_testing (test)
  - trie-cli v0.1.0 (edition 2021)
    Targets: trie-cli (bin)
```

---

## 4. Code Formatting Validation

```powershell
PS C:\Users\User\Downloads\Programming Lab\Word Games\word-trie> cargo fmt --all -- --check
# Exit code: 0 (No formatting differences found)
```

---

## 5. Type-Checking Validation

```powershell
PS C:\Users\User\Downloads\Programming Lab\Word Games\word-trie> cargo check --workspace
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.03s
```

---

## 6. Comprehensive Test Suite Output (`cargo test --workspace`)

```powershell
PS C:\Users\User\Downloads\Programming Lab\Word Games\word-trie> cargo test --workspace
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.03s
     Running unittests src\lib.rs (target\debug\deps\trie-71d0bff968a9abdd.exe)

running 29 tests
test tests::test_insert_case_normalization ... ok
test tests::test_has_prefix ... ok
test tests::test_insert_multiple_unrelated_words ... ok
test tests::test_char_to_index_mapping ... ok
test tests::test_index_to_char_mapping ... ok
test tests::test_insert_invalid_input ... ok
test tests::test_insert_prefix_of_existing_word ... ok
test tests::test_insert_shared_prefixes ... ok
test tests::test_insert_single_word ... ok
test tests::test_lookup_contains_existing_and_missing ... ok
test tests::test_default_trie_is_empty ... ok
test tests::test_lookup_case_normalization ... ok
test tests::test_insert_duplicate ... ok
test tests::test_normalize_empty_and_whitespace ... ok
test tests::test_insert_short_and_long_words ... ok
test tests::test_new_trie_is_empty ... ok
test tests::test_node_initial_state ... ok
test tests::test_lookup_empty_and_invalid_returns_false ... ok
test tests::test_normalize_invalid_characters ... ok
test tests::test_normalize_valid_words ... ok
test tests::test_remove_invalid_input ... ok
test tests::test_remove_leaf_word_and_pruning ... ok
test tests::test_remove_case_normalization ... ok
test tests::test_remove_prefix_word_preserves_longer_word ... ok
test tests::test_remove_nonexistent_and_prefix_only ... ok
test tests::test_remove_repeated ... ok
test tests::test_remove_until_empty ... ok
test tests::test_round_trip_index_char ... ok
test tests::test_remove_words_batch ... ok

test result: ok. 29 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests\differential_testing.rs (target\debug\deps\differential_testing-971bc30972d8ebfe.exe)

running 3 tests
test test_alphabet_boundary_extremities ... ok
test test_boundary_error_handling ... ok
test test_differential_randomized_operations ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s

     Running unittests src\main.rs (target\debug\deps\trie_cli-95f68b44b3221820.exe)

running 14 tests
test batch::tests::test_parse_tokens_various_separators ... ok
test batch::tests::test_load_words_from_reader_mixed_content ... ok
test batch::tests::test_check_words_from_reader ... ok
test batch::tests::test_load_empty_reader ... ok
test batch::tests::test_file_not_found_error ... ok
test tests::test_process_command_found_missing ... ok
test tests::test_process_command_help ... ok
test batch::tests::test_parse_tokens_empty_and_whitespace ... ok
test tests::test_process_command_insert_find_delete ... ok
test tests::test_process_command_unknown_and_empty ... ok
test tests::test_process_command_quit ... ok
test tests::test_process_command_stats ... ok
test batch::tests::test_load_and_check_file_io ... ok
test tests::test_full_dictionary_loading_and_lookup ... ok

test result: ok. 14 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.89s

   Doc-tests trie

running 8 tests
test trie\src\lib.rs - Trie::contains (line 269) ... ok
test trie\src\lib.rs - Trie::new (line 165) ... ok
test trie\src\lib.rs - Trie::len (line 185) ... ok
test trie\src\lib.rs - Trie::has_prefix (line 296) ... ok
test trie\src\lib.rs - Trie::is_empty (line 199) ... ok
test trie\src\lib.rs - Trie::remove_words (line 358) ... ok
test trie\src\lib.rs - Trie::insert (line 220) ... ok
test trie\src\lib.rs - Trie::remove (line 327) ... ok

test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 3.10s
```

---

## 7. Clippy Linter Verification

```powershell
PS C:\Users\User\Downloads\Programming Lab\Word Games\word-trie> cargo clippy --workspace --all-targets -- -D warnings
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.14s
# Exit code: 0 (Zero warnings, zero errors)
```

---

## 8. Release Build Verification

```powershell
PS C:\Users\User\Downloads\Programming Lab\Word Games\word-trie> cargo build --release --workspace
    Finished `release` profile [optimized] target(s) in 0.14s
# Binary created: target/release/trie-cli.exe
```

---

## 9. Documentation Build Verification

```powershell
PS C:\Users\User\Downloads\Programming Lab\Word Games\word-trie> cargo doc --workspace --no-deps
 Documenting trie-cli v0.1.0 (C:\Users\User\Downloads\Programming Lab\Word Games\word-trie\trie-cli)
 Documenting trie v0.1.0 (C:\Users\User\Downloads\Programming Lab\Word Games\word-trie\trie)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 1.76s
   Generated C:\Users\User\Downloads\Programming Lab\Word Games\word-trie\target\doc\trie\index.html and 1 other file
```

---

## 10. Dictionary File Exact Statistics

Calculated independently on `dictionary.txt`:

```text
File size in bytes:            1,741,037 bytes (~1.74 MB)
Total line count:              176,413 lines
Non-empty entries:             176,413
Blank lines:                   0
Unique distinct entries:       176,413
Duplicate entries:             0
Valid ASCII uppercase:         176,413
Invalid entries:               0
Shortest word length:          2 characters (e.g. "AA")
Longest word length:           15 characters (e.g. "AASVOGELS", "ZYMURGIES")
```

---

## 11. Live CLI Demonstration Transcript

The following transcript records a live interactive session with `trie-cli.exe` executing standard and edge commands:

```text
==================================================
             Word Trie Dictionary CLI             
==================================================
Loading initial dictionary from 'dictionary.txt'...
Successfully loaded 176413 words in 131.30ms (0 duplicates, 0 errors).

Type 'help' for commands, 'quit' to exit.

trie> stats
Dictionary Statistics:
  Distinct complete words: 176413
  Status: Populated

trie> find AA, TREE, ZZZ
  AA: FOUND
  TREE: FOUND
  ZZZ: FOUND

trie> find NOTAWORD
  NOTAWORD: NOT FOUND

trie> insert XYZABC, XYZABCD
  XYZABC: inserted (new word)
  XYZABCD: inserted (new word)

trie> find XYZABC, XYZABCD
  XYZABC: FOUND
  XYZABCD: FOUND

trie> delete XYZABCD
  XYZABCD: deleted (word removed)

trie> find XYZABC, XYZABCD
  XYZABC: FOUND
  XYZABCD: NOT FOUND

trie> found CAR, TRUCK, BICYCLE, XYZABC, GIBBERISHWORD
Found (4): CAR, TRUCK, BICYCLE, XYZABC

trie> missing CAR, TRUCK, BICYCLE, XYZABC, GIBBERISHWORD
Missing (1): GIBBERISHWORD

trie> stats
Dictionary Statistics:
  Distinct complete words: 176414
  Status: Populated

trie> quit
Goodbye!
```

---

## 10. Spelling Bee Test Suite & Benchmark Evidence

```text
PS C:\Users\User\Downloads\Programming Lab\Word Games\word-trie> cargo test -p spelling-bee --test spelling_bee_integration -- --nocapture
   Compiling spelling-bee v0.1.0 (C:\Users\User\Downloads\Programming Lab\Word Games\word-trie\spelling-bee)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 6.81s
     Running tests\spelling_bee_integration.rs (target\debug\deps\spelling_bee_integration-b5242d6b593a78e6.exe)

running 1 test
== SPELLING BEE INTEGRATION REPORT ==
Dictionary loaded: 176413 words in 651.63ms
Generated puzzle: Main Letter: [N]  |  Other Letters: A M P V W G in 81.30µs
Trie DFS solved 26 solutions in 208.10µs
Brute-force reference solved 26 solutions in 42.44ms
test test_spelling_bee_full_integration_and_benchmarks ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.85s
```

---

## 11. Boggle Test Suite & Benchmark Evidence

```text
PS C:\Users\User\Downloads\Programming Lab\Word Games\word-trie> cargo test -p boggle --test boggle_integration -- --nocapture
   Compiling boggle v0.1.0 (C:\Users\User\Downloads\Programming Lab\Word Games\word-trie\boggle)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 2.40s
     Running tests\boggle_integration.rs (target\debug\deps\boggle_integration-94627b2d2b6da745.exe)

running 3 tests
== BOGGLE INTEGRATION REPORT ==
Dictionary loaded: 176413 words in 608.66ms

Testing 4x4 Boggle Board:
  Automatic solver found 37 valid words in 473.70µs

Testing 5x5 Boggle Board:
  Automatic solver found 62 valid words in 1.13ms

Testing 6x6 Boggle Board:
  Automatic solver found 31 valid words in 530.40µs
test test_boggle_qu_tile_and_scoring_integration ... ok
test test_boggle_full_integration_all_dimensions ... ok
test test_boggle_batch_round_simulation ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.84s
```

---

## 12. Full Workspace Test Suite Output (101 Tests)

```text
PS C:\Users\User\Downloads\Programming Lab\Word Games\word-trie> cargo test --workspace
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.09s
     Running unittests src\lib.rs (target\debug\deps\boggle-b7e862be32f95eb9.exe)
test result: ok. 20 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests\boggle_integration.rs (target\debug\deps\boggle_integration-94627b2d2b6da745.exe)
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.81s

     Running unittests src\lib.rs (target\debug\deps\spelling_bee-67a1c81ab6c82f3d.exe)
test result: ok. 18 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running unittests src\main.rs (target\debug\deps\spelling_bee-ef2750548011ae94.exe)
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests\spelling_bee_integration.rs (target\debug\deps\spelling_bee_integration-b5242d6b593a78e6.exe)
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.71s

     Running unittests src\lib.rs (target\debug\deps\trie-71d0bff968a9abdd.exe)
test result: ok. 32 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s

     Running tests\differential_testing.rs (target\debug\deps\differential_testing-971bc30972d8ebfe.exe)
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s

     Running unittests src\main.rs (target\debug\deps\trie_cli-95f68b44b3221820.exe)
test result: ok. 14 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.21s

   Doc-tests trie
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 3.99s
```

---

## 13. Clippy Strict Linter Execution

```text
PS C:\Users\User\Downloads\Programming Lab\Word Games\word-trie> cargo clippy --workspace --all-targets -- -D warnings
    Checking spelling-bee v0.1.0 (C:\Users\User\Downloads\Programming Lab\Word Games\word-trie\spelling-bee)
    Checking boggle v0.1.0 (C:\Users\User\Downloads\Programming Lab\Word Games\word-trie\boggle)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.61s
# Exit code: 0 (Zero warnings, zero errors)
```

---

## 14. Release Profile Compilation

```text
PS C:\Users\User\Downloads\Programming Lab\Word Games\word-trie> cargo build --release --workspace
   Compiling zerocopy v0.8.63
   Compiling cfg-if v1.0.5
   Compiling trie v0.1.0 (C:\Users\User\Downloads\Programming Lab\Word Games\word-trie\trie)
   Compiling getrandom v0.2.17
   Compiling rand_core v0.6.4
   Compiling trie-cli v0.1.0 (C:\Users\User\Downloads\Programming Lab\Word Games\word-trie\trie-cli)
   Compiling ppv-lite86 v0.2.21
   Compiling rand_chacha v0.3.1
   Compiling rand v0.8.8
   Compiling boggle v0.1.0 (C:\Users\User\Downloads\Programming Lab\Word Games\word-trie\boggle)
   Compiling spelling-bee v0.1.0 (C:\Users\User\Downloads\Programming Lab\Word Games\word-trie\spelling-bee)
    Finished `release` profile [optimized] target(s) in 21.50s
```

---

## 15. Workspace Documentation Build

```text
PS C:\Users\User\Downloads\Programming Lab\Word Games\word-trie> cargo doc --workspace --no-deps
 Documenting trie v0.1.0 (C:\Users\User\Downloads\Programming Lab\Word Games\word-trie\trie)
 Documenting trie-cli v0.1.0 (C:\Users\User\Downloads\Programming Lab\Word Games\word-trie\trie-cli)
 Documenting spelling-bee v0.1.0 (C:\Users\User\Downloads\Programming Lab\Word Games\word-trie\spelling-bee)
 Documenting boggle v0.1.0 (C:\Users\User\Downloads\Programming Lab\Word Games\word-trie\boggle)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 4.41s
   Generated C:\Users\User\Downloads\Programming Lab\Word Games\word-trie\target\doc\boggle\index.html and 3 other files
```

