# Project Overview: Trie Dictionary Assignment

This document establishes the official academic specification, learning outcomes, functional requirements, and implementation status for the university Programming Lab Trie dictionary assignment.

---

## 1. Assignment Purpose and Scope

As specified in the primary course assignment document (`Word Games.pdf`), this project requires designing and implementing an in-memory **Trie data structure** (prefix tree) capable of storing and querying a large English dictionary containing well over 100,000 words.

The Trie serves as the core indexing engine for dictionary operations and will act as the foundational library for future word games (e.g., Boggle, Spelling Bee, Wordle). In accordance with the prompt's instructions, **game mechanics are explicitly out of scope** for this phase; only the data structure, algorithms, file loaders, batch processors, and interactive CLI are implemented.

---

## 2. Learning Outcomes Identified in Specification

The assignment specification defines several key Rust learning outcomes:

1. **Memory & Variable Mutability**:
   - Correct discernment between mutable and immutable variables (`let` vs. `let mut`).
   - Constant definition (`pub const ALPHABET_SIZE: usize = 26`).
2. **Parameter Passing & Ownership**:
   - Passing by immutable reference (`&self`, `&str`, `&[u8]`).
   - Passing by mutable reference (`&mut self`, `&mut Node`).
   - Ownership transfers and heap management via `Box<Node>`.
3. **Control Flow**:
   - Definite iteration over character iterators, slices, and lines (`for ch in ...`).
   - Indefinite iteration via recursive tree descent and REPL input loops (`loop { ... }`).
   - Early termination on missing branches or error encounters.
4. **Data Structures & Types**:
   - Fixed-size arrays: `[Option<Box<Node>>; 26]`.
   - Option types (`Option<Box<Node>>`, `Option<usize>`, `Option<char>`).
   - Result types (`Result<bool, TrieError>`, `std::io::Result<LoadReport>`).
   - Algebraic Data Types (Structs and Enums).
5. **Modular Architecture & Encapsulation**:
   - Segregation into library crate (`trie`) and binary crate (`trie-cli`).
   - Information hiding: `Node` fields and internal tree navigation remain private (`pub(crate)` and `private`), exposing a clean public API (`Trie`).
6. **Testing & Meta-Documentation**:
   - Systematic unit, integration, differential, and boundary testing.
   - Comprehensive evidence-based documentation of design and results.
7. **Toolchain Competency**:
   - Professional utilization of Cargo for building, checking, formatting, linting, testing, and documentation generation.

---

## 3. Requirements Traceability Matrix

The table below maps every assigned functional requirement to its implementation in the codebase and its verification evidence.

| Requirement ID | Assigned Specification (`Word Games.pdf`) | Implementing Function / Symbol | Source File Location | Validation Test & Evidence | Status |
|---|---|---|---|---|---|
| **REQ-01** | Create a new, empty trie | `Trie::new()`, `Trie::default()` | [`trie/src/lib.rs`](file:///c:/Users/User/Downloads/Programming%20Lab/Word%20Games/word-trie/trie/src/lib.rs#L160-L177) | `test_new_trie_is_empty`, `test_default_trie_is_empty` | **Fully Implemented** |
| **REQ-02** | Add a word to the trie (one word at a time) | `Trie::insert(&mut self, word: &str) -> Result<bool, TrieError>` | [`trie/src/lib.rs`](file:///c:/Users/User/Downloads/Programming%20Lab/Word%20Games/word-trie/trie/src/lib.rs#L209-L244) | `test_insert_single_word`, `test_insert_shared_prefixes`, `test_insert_duplicate` | **Fully Implemented** |
| **REQ-03** | Lookup a given word and return a boolean | `Trie::contains(&self, word: &str) -> bool` | [`trie/src/lib.rs`](file:///c:/Users/User/Downloads/Programming%20Lab/Word%20Games/word-trie/trie/src/lib.rs#L261-L283) | `test_lookup_contains_existing_and_missing`, `test_lookup_case_normalization` | **Fully Implemented** |
| **REQ-04** | Check if a nonempty valid prefix has a path | `Trie::has_prefix(&self, prefix: &str) -> bool` | [`trie/src/lib.rs`](file:///c:/Users/User/Downloads/Programming%20Lab/Word%20Games/word-trie/trie/src/lib.rs#L285-L309) | `test_has_prefix` | **Fully Implemented** |
| **REQ-05** | Case normalization (convert lowercase to uppercase) | `normalize_word(input: &str) -> Result<String, TrieError>` | [`trie/src/lib.rs`](file:///c:/Users/User/Downloads/Programming%20Lab/Word%20Games/word-trie/trie/src/lib.rs#L103-L124) | `test_normalize_valid_words`, `test_insert_case_normalization` | **Fully Implemented** |
| **REQ-06** | Reject empty strings, punctuation, digits, non-ASCII | `TrieError`, `normalize_word` | [`trie/src/lib.rs`](file:///c:/Users/User/Downloads/Programming%20Lab/Word%20Games/word-trie/trie/src/lib.rs#L13-L36) | `test_normalize_empty_and_whitespace`, `test_normalize_invalid_characters` | **Fully Implemented** |
| **REQ-07** | Delete a word from the trie without damaging other words sharing prefix | `Trie::remove(&mut self, word: &str) -> Result<bool, TrieError>` | [`trie/src/lib.rs`](file:///c:/Users/User/Downloads/Programming%20Lab/Word%20Games/word-trie/trie/src/lib.rs#L311-L350) | `test_remove_leaf_word_and_pruning`, `test_remove_prefix_word_preserves_longer_word` | **Fully Implemented** |
| **REQ-08** | Delete a list of words from the trie | `Trie::remove_words<I, S>(&mut self, words: I)` | [`trie/src/lib.rs`](file:///c:/Users/User/Downloads/Programming%20Lab/Word%20Games/word-trie/trie/src/lib.rs#L352-L376) | `test_remove_words_batch` | **Fully Implemented** |
| **REQ-09** | Maintain accurate count of distinct complete words | `Trie::len(&self) -> usize`, `Trie::is_empty(&self) -> bool` | [`trie/src/lib.rs`](file:///c:/Users/User/Downloads/Programming%20Lab/Word%20Games/word-trie/trie/src/lib.rs#L179-L207) | `test_insert_duplicate`, `test_remove_repeated`, `differential_testing` | **Fully Implemented** |
| **REQ-10** | Read large list of words from text file (one per line) | `load_words_from_file`, `load_words_from_reader` | [`trie-cli/src/batch.rs`](file:///c:/Users/User/Downloads/Programming%20Lab/Word%20Games/word-trie/trie-cli/src/batch.rs#L44-L73) | `test_load_and_check_file_io`, `test_full_dictionary_loading_and_lookup` | **Fully Implemented** |
| **REQ-11** | Read words separated by commas and/or whitespace | `parse_tokens`, `load_words_from_reader` | [`trie-cli/src/batch.rs`](file:///c:/Users/User/Downloads/Programming%20Lab/Word%20Games/word-trie/trie-cli/src/batch.rs#L34-L42) | `test_parse_tokens_various_separators`, `test_load_words_from_reader_mixed_content` | **Fully Implemented** |
| **REQ-12** | Return all words from a file/list that are in the trie | `check_words_from_file`, `check_words_from_reader`, `found` cmd | [`trie-cli/src/batch.rs`](file:///c:/Users/User/Downloads/Programming%20Lab/Word%20Games/word-trie/trie-cli/src/batch.rs#L75-L113) | `test_check_words_from_reader`, `test_process_command_found_missing` | **Fully Implemented** |
| **REQ-13** | Return all words from a file/list that are NOT in the trie | `check_words_from_file`, `missing` cmd | [`trie-cli/src/batch.rs`](file:///c:/Users/User/Downloads/Programming%20Lab/Word%20Games/word-trie/trie-cli/src/batch.rs#L75-L113) | `test_check_words_from_reader`, `test_process_command_found_missing` | **Fully Implemented** |
| **REQ-14** | Command-line interface ("trie-cli") | `main()`, `process_command` | [`trie-cli/src/main.rs`](file:///c:/Users/User/Downloads/Programming%20Lab/Word%20Games/word-trie/trie-cli/src/main.rs#L20-L318) | `test_process_command_*`, live interactive CLI verification | **Fully Implemented** |
| **REQ-15** | Cargo workspace structure matching specification | `word-trie/` with `trie` and `trie-cli` | [`word-trie/Cargo.toml`](file:///c:/Users/User/Downloads/Programming%20Lab/Word%20Games/word-trie/Cargo.toml) | `cargo check --workspace`, `cargo tree` | **Fully Implemented** |
| **REQ-16** | Randomized testing vs independent reference | `SimpleRng` (Xorshift64*), `reference: HashSet<String>` | [`trie/tests/differential_testing.rs`](file:///c:/Users/User/Downloads/Programming%20Lab/Word%20Games/word-trie/trie/tests/differential_testing.rs) | `test_differential_randomized_operations` (2,000 steps) | **Fully Implemented** |

---

## 4. Verification Summary

All 16 functional requirements identified in the assignment brief and the PDF specification have been implemented, tested, and validated. No assigned functional requirement remains incomplete or unverified.
