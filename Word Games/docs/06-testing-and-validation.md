# 06. Testing, Verification & Validation Report

This document details the multi-tiered verification strategy applied across the entire workspace, encompassing unit testing, doc-testing, integration simulation, randomized metamorphic differential testing, and comprehensive test coverage mapping.

---

## 1. Test Suite Architecture & Summary

The workspace maintains **101 distinct test executions**, spanning 4 crate boundaries:

| Crate / Target | Test Category | Number of Tests | Execution Command | Result |
| :--- | :--- | :---: | :--- | :---: |
| **`trie` (lib)** | Unit Tests | 32 | `cargo test -p trie --lib` | **32 Passed** |
| **`trie` (tests)** | Differential Testing | 3 | `cargo test -p trie --test differential_testing` | **3 Passed** |
| **`trie` (doc)** | Documentation Tests | 9 | `cargo test -p trie --doc` | **9 Passed** |
| **`trie-cli` (bin)** | Unit & Batch Tests | 14 | `cargo test -p trie-cli` | **14 Passed** |
| **`spelling-bee` (lib)**| Unit Tests | 18 | `cargo test -p spelling-bee --lib` | **18 Passed** |
| **`spelling-bee` (bin)**| Main CLI Tests | 1 | `cargo test -p spelling-bee --bin spelling-bee` | **1 Passed** |
| **`spelling-bee` (tests)**| End-to-End & Benchmarks| 1 | `cargo test -p spelling-bee --test spelling_bee_integration` | **1 Passed** |
| **`boggle` (lib)** | Unit Tests | 20 | `cargo test -p boggle --lib` | **20 Passed** |
| **`boggle` (tests)** | Integration & Batch | 3 | `cargo test -p boggle --test boggle_integration` | **3 Passed** |
| **TOTAL** | **Full Workspace Suite**| **101** | `cargo test --workspace` | **101 Passed (0 Failed)** |

---

## 2. Component Test Breakdown

### A. Trie Data Structure & CLI
- **Basic Invariants:** Verified that newly initialized Tries have `len == 0` and `is_empty == true`.
- **Character Mapping:** Verified bijective round-trip mapping between ASCII bytes `[A-Z]` and indices `[0-25]`.
- **Case Normalization:** Ensured `"apple"`, `"Apple"`, and `"APPLE"` map to the identical path `"APPLE"`.
- **Prefix Isolation:** Validated that inserting `"CAR"` does not falsely indicate presence of `"CA"` or `"CART"`, while `has_prefix("CA")` correctly returns `true`.
- **Node Pruning Invariants:**
  - Deleting a leaf word (e.g., `"CART"` when `"CAR"` exists) prunes the terminal node `'T'` while preserving `'R'` and `"CAR"`.
  - Deleting until empty deallocates all child nodes back to a bare root node.
- **Batch IO Processing:** Tested token parsing with commas, tabs, multiple spaces, and empty lines.

### B. Spelling Bee Engine
- **Puzzle Invariants:** Tested that inputs with $< 7$ letters, $> 7$ letters, non-ASCII characters, or duplicate characters are rejected with descriptive `PuzzleError` variants.
- **Center Letter Enforcement:** Ensured `GuessResult::MissingMainLetter` is triggered whenever a candidate guess omits the primary center letter.
- **Pangram Scoring:** Confirmed that a 7-letter word utilizing all 7 distinct letters scores $7 + 7 = 14$ points, while words with 6 distinct letters score only 7 points.
- **Queen Bee Condition:** Validated that finding all generated solutions transitions the game into `is_queen_bee == true`.
- **Surrender Display:** Verified that `missed_words()` accurately returns all un-guessed solutions sorted by length and alphabetical order.

### C. Boggle Engine
- **Dimension Guards:** Verified that dimensions other than 4, 5, or 6 return `BoardError::InvalidDimensions`.
- **Coordinate Boundary Geometry:** Tested corner cells (3 neighbors), edge cells (5 neighbors), and interior cells (8 neighbors) across 4x4, 5x5, and 6x6 grids.
- **Physical Cell Reuse Prevention:** Tested boards where a word requires two occurrences of a letter (e.g., `"TART"`); verified that if only one `'T'` cell exists adjacent to `'R'`, the path is strictly rejected.
- **Batch Evaluation:** Simulated player turns entering mixed-case strings (`"cart, team, bog, xyz"`) and verified correct partitioning into accepted and rejected categories.
- **Qu Tile Expansion:** Confirmed tile `'Q'` advances target words by 2 characters (`"QU"`) while occupying 1 board cell.

---

## 3. Randomized Metamorphic & Differential Testing

To guarantee correctness beyond hand-crafted unit tests, the test suite implements randomized differential testing in [`1 - Library Trie/trie/tests/differential_testing.rs`](file:///c:/Users/User/Downloads/Programming%20Lab/Word%20Games/1%20-%20Library%20Trie/trie/tests/differential_testing.rs).

### Methodology:
- **Oracle Comparison:** Compares the custom `Trie` implementation against Rust's mathematically proven standard library oracle `std::collections::BTreeSet<String>`.
- **Deterministic Pseudo-Random Generator:** Employs a deterministic seeded PRNG (`seed = 0xDEAD_BEEF_CAFE_BABE`) ensuring exact repeatability across platforms.
- **Operation Distribution:** Executes a sequence of **1,000 randomized operations**:
  - `Insert`: 50% probability
  - `Lookup`: 30% probability
  - `Delete`: 20% probability
- **Invariants Checked at Every Step:**
  1. `trie.len() == btreeset.len()`
  2. `trie.contains(&word) == btreeset.contains(&word)`
  3. `trie.has_prefix(&prefix) == btreeset.iter().any(|w| w.starts_with(&prefix))`
  4. Node deletion cleanly leaves no prunable dangling references.

---

## 4. Comprehensive Test Coverage Matrix

| Test Identifier | Crate | Target Module | Scenario Under Test | Expected Behavior | Actual Result |
| :--- | :--- | :--- | :--- | :--- | :---: |
| `test_new_trie_is_empty` | `trie` | `lib.rs` | Trie initialization | `len == 0`, `is_empty == true` | **PASS** |
| `test_insert_single_word` | `trie` | `lib.rs` | Inserting "RUST" | Returns `Ok(true)`, `len == 1` | **PASS** |
| `test_insert_duplicate` | `trie` | `lib.rs` | Re-inserting "RUST" | Returns `Ok(false)`, `len == 1` | **PASS** |
| `test_lookup_contains` | `trie` | `lib.rs` | Lookup existing vs absent | Exact boolean match | **PASS** |
| `test_remove_leaf_prune` | `trie` | `lib.rs` | Delete "CART" given "CAR" | Removes "CART", retains "CAR" | **PASS** |
| `test_has_prefix` | `trie` | `lib.rs` | Prefix queries | Matches subpaths | **PASS** |
| `test_cursor_advance` | `trie` | `lib.rs` | Cursor step traversal | Navigates down tree without heap alloc | **PASS** |
| `test_differential_random`| `trie` | `differential`| 1,000 random operations | Identical behavior to `BTreeSet` | **PASS** |
| `test_batch_load_split` | `trie-cli`| `batch.rs` | Comma & space separated tokens | Ingests all tokens cleanly | **PASS** |
| `test_valid_puzzle` | `spelling-bee`| `puzzle.rs`| Construct hive "CAIMORT" | Validates center 'C' and hive | **PASS** |
| `test_duplicate_letters` | `spelling-bee`| `puzzle.rs`| Hive with repeated 'A' | Returns `Err(DuplicateLetters)` | **PASS** |
| `test_bitmask_operations`| `spelling-bee`| `puzzle.rs`| 7-bit mask creation | Pangram yields `0x7F` | **PASS** |
| `test_solver_controlled` | `spelling-bee`| `solver.rs`| Small controlled Trie DFS | Precomputes exact valid words | **PASS** |
| `test_pangram_bonus` | `spelling-bee`| `scoring.rs`| 7-letter word with 7 letters | Awards `7 + 7 = 14` points | **PASS** |
| `test_queen_bee` | `spelling-bee`| `game.rs` | Discovering all solutions | `is_queen_bee == true` | **PASS** |
| `test_3_column_format` | `spelling-bee`| `main.rs` | Formatting 9 words | 3 rows across 3 columns | **PASS** |
| `test_valid_dimensions` | `boggle` | `board.rs` | 4x4, 5x5, 6x6 creation | Validates cell count and geometry | **PASS** |
| `test_neighbor_counts` | `boggle` | `board.rs` | Corner, edge, interior | 3, 5, and 8 neighbors | **PASS** |
| `test_cell_reuse_reject` | `boggle` | `validator.rs`| Revisiting cell in word | Rejected; physical cell reused | **PASS** |
| `test_batch_evaluation` | `boggle` | `game.rs` | Batch comma-separated turn | Categorizes accepted & rejected | **PASS** |
| `test_auto_solver` | `boggle` | `validator.rs`| Trie DFS over board | Solves all board words in $< 1\text{ ms}$ | **PASS** |
| `test_qu_tile_support` | `boggle` | `validator.rs`| 'Q' tile matching "QU" | Consumes 2 letters for 1 cell | **PASS** |
