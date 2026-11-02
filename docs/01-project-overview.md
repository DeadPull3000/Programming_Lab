# 01. Project Overview & Requirements Traceability

This document provides the foundational specification, educational objectives, and comprehensive requirements traceability matrices for the Rust Word Games project, comprising three interconnected components:
1. **Encapsulated 26-ary Trie Dictionary Library & Interactive CLI** (`1 - Library Trie/`)
2. **NYT-Style Spelling Bee Game Engine & CLI** (`2 - Spelling Bee/`)
3. **Multi-Dimension Boggle Game Engine, Solver & CLI** (`3 - Boggle/`)

---

## 1. Project Purpose & Scope

The primary objective of this project is to implement an industrial-grade, memory-safe, and high-performance dictionary and word-game ecosystem in Rust. Originally assigned as a Programming Lab module, the project demonstrates how an abstract data structure—a 26-way prefix tree (Trie)—serves as a shared algorithmic foundation for interactive word puzzle applications.

```
                           +----------------------------------+
                           |   Raw Lexicon (dictionary.txt)   |
                           |   176,413 Normalized Words       |
                           +----------------------------------+
                                             |
                                             v
                           +----------------------------------+
                           |     Shared Trie Library (trie)    |
                           |  - O(L) Lookup, Insert, Delete   |
                           |  - Zero-Allocation TrieCursor     |
                           +----------------------------------+
                                    /        |        \
                                   /         |         \
                                  v          v          v
                  +-------------------+  +---------+  +-------------+
                  |     trie-cli      |  |Spelling |  |   Boggle    |
                  | - Interactive REPL|  |   Bee   |  | - 4x4,5x5,6x6|
                  | - Batch Classifier|  | - DFS   |  | - Path DFS  |
                  +-------------------+  |   Solver|  | - Auto-Solve|
                                         +---------+  +-------------+
```

---

## 2. Component Specifications

### A. The Trie Library & CLI (`trie`, `trie-cli`)
Derived from the university assignment specification (`Word Games.pdf`):
- **Alphabet & Lexicon Scale:** Must support large lexicons ($\ge 100,000$ words; actual test lexicon contains **176,413 words**), strictly confined to standard Latin uppercase letters `A` through `Z` without special characters.
- **Empty Initialization:** Instantiation of a root node without heap-allocating redundant sub-nodes (`Trie::new()`).
- **Insertion:** Adding single words in $O(L)$ time, where $L$ is word length (`Trie::insert`).
- **Exact Lookup:** Returning a boolean status indicating exact lexical membership (`Trie::contains`).
- **Prefix Matching:** Verifying whether any dictionary word begins with a specified prefix (`Trie::has_prefix`).
- **Deletion & Pruning:** Removing complete words while preserving shared prefixes of longer words, and pruning dangling childless nodes to reclaim heap memory (`Trie::remove`, `Trie::remove_words`).
- **Case Normalization:** Robust normalization turning arbitrary lowercase and mixed-case ASCII into uppercase.
- **File & Batch Ingestion:** Loading words from newline-delimited text files or comma-/whitespace-delimited token streams (`batch::load_words_from_reader`).
- **Classification Routines:** Processing arbitrary input files to return partitions of words present in the Trie vs. words missing from the Trie (`batch::check_words_from_reader`).
- **Accurate Word Tracking:** Precise maintenance of the active word count across inserts, duplicates, and deletions.

### B. Spelling Bee Game Engine (`spelling-bee`)
Modeled after the New York Times Spelling Bee puzzle:
- **Seven-Letter Hive:** Construction and display of 7 distinct uppercase letters.
- **Primary / Center Letter:** Explicit designation of a single mandatory letter (must be present in every valid answer and displayed prominently in center position).
- **Repetition Permitted:** Players may repeat any of the 7 letters arbitrarily (e.g., given center `R` and hive `[R, A, T, E, C, L, P]`, the word `RATATOUILLE` or `PARTERRE` uses allowed letters).
- **Minimum Length:** Words under 4 letters are strictly invalid and score 0 points.
- **Comprehensive Solution Enumeration:** Precomputing the full solution set via Trie-guided Depth-First Search (DFS) upon puzzle creation.
- **Official NYT Scoring:**
  - 4-letter words earn **1 point**.
  - Words with 5 or more letters earn **1 point per letter**.
  - **Pangrams** (words using all 7 hive letters at least once) receive a **+7 point bonus**.
- **Queen Bee State:** Achieving the maximum possible score by finding every single valid dictionary word.
- **Formatted Presentation:** Displaying guessed words in alphabetical order across three columns.
- **Graceful Termination:** Displaying all remaining missed words sorted by length and alphabet upon player surrender.
- **Hint Matrix:** Detailed breakdown grid showing word counts by starting letter, length distribution, and 2-letter prefix counts.

### C. Boggle Game Engine (`boggle`)
Based on the classic Parker Brothers Boggle rules:
- **Configurable Grid Dimensions:** Fully supporting $4 \times 4$, $5 \times 5$, and $6 \times 6$ board sizes.
- **Board Population:** Random generation allowing letter repetition across different cells.
- **Minimum Length:** Words under 3 letters are strictly invalid and score 0 points.
- **Adjacency & Movement:** Words must follow contiguous paths across horizontal, vertical, and diagonal neighbors (up to 8 neighbors per cell).
- **Physical Cell Reuse Prohibition:** A single physical board cell cannot be visited more than once within the formation of a single word (distinguishing cell identity from letter character equality).
- **Dual Validation Pipeline:** Requiring both valid board path formation and valid dictionary membership.
- **Batch Evaluation:** Players enter comma- or whitespace-separated guesses in a single turn, which are processed without allowing retraction.
- **Standard Scoring Table:**
  - 3–4 letters: **1 point**
  - 5 letters: **2 points**
  - 6 letters: **3 points**
  - 7 letters: **5 points**
  - 8 letters: **11 points**
  - $\ge 9$ letters: **2 points per letter** (extension policy for large boards)
- **Specialized Enhancements:**
  - **Qu Tile Support:** Configurable expansion treating tile `Q` as the two-letter sequence `QU`.
  - **Automatic Board Solver:** Trie-guided DFS exploring all valid board paths against the dictionary in $< 1.2\text{ ms}$.
  - **Countdown Timer:** Non-blocking round timer simulating real-time match constraints.

---

## 3. Educational Objectives & Rust Language Mapping

The project satisfies all stated learning outcomes from the university Programming Lab syllabus:

| Learning Outcome | Rust Language Mechanism | Specific Implementation Example |
| :--- | :--- | :--- |
| **Variable Mutability** | `let` vs. `let mut` binding | `let mut current = &mut self.root;` in Trie insertion; immutable game boards. |
| **Constants** | `const` definitions | `pub const ALPHABET_SIZE: usize = 26;`, `FULL_PANGRAM_MASK: u8 = 0x7F;`. |
| **Ownership & Borrowing** | `&`, `&mut`, and move semantics | Moving `Board` into `BoggleGame::new(board, ...)`; borrowing `&Trie` across rounds. |
| **Lifetime Annotations** | Explicit lifetimes (`'a`) | `TrieCursor<'a>` tied immutably to the underlying `&'a Trie` instance. |
| **Memory Allocation** | Smart pointers (`Box<T>`) | Sparse Trie node links `[Option<Box<Node>>; 26]` preventing recursive sizing errors. |
| **Algebraic Data Types** | `enum` and `struct` | `TrieError`, `GuessResult`, `BoggleGuessResult`, `Board`, `SpellingBeePuzzle`. |
| **Error Handling** | `Option<T>` and `Result<T, E>` | Propagating missing branches via `None` and parse errors via `Result::Err`. |
| **Iterators & Closures** | Functional combinators | `tokens.split_whitespace().filter(...)`, `.any()`, `.map()`, `.collect()`. |
| **Bitwise Manipulation** | Bit shifts and masks | 7-bit puzzle letter masks (`u8`) for $O(1)$ pangram checking and validation. |
| **Modular Encapsulation** | Crate & visibility boundaries | `pub(crate)` internal `Node` struct kept strictly hidden behind public `TrieCursor`. |
| **Testing & Tooling** | Cargo ecosystem | Unit tests, doc-tests, integration tests, randomized differential tests, Clippy. |

---

## 4. Requirements Traceability Matrices

### Matrix 1: Trie Library & CLI (`trie`, `trie-cli`)

| Requirement | Assignment Source | Implementation Location | Validation Evidence | Status |
| :--- | :--- | :--- | :--- | :---: |
| **Create empty Trie** | `Word Games.pdf` p.1 | [`Trie::new`](file:///c:/Users/User/Downloads/Programming%20Lab/Word%20Games/1%20-%20Library%20Trie/trie/src/lib.rs#L165) | `test_new_trie_is_empty` | Implemented and tested |
| **Lookup word (boolean)** | `Word Games.pdf` p.1 | [`Trie::contains`](file:///c:/Users/User/Downloads/Programming%20Lab/Word%20Games/1%20-%20Library%20Trie/trie/src/lib.rs#L269) | `test_lookup_contains_existing_and_missing` | Implemented and tested |
| **Add single word** | `Word Games.pdf` p.1 | [`Trie::insert`](file:///c:/Users/User/Downloads/Programming%20Lab/Word%20Games/1%20-%20Library%20Trie/trie/src/lib.rs#L220) | `test_insert_single_word`, `test_insert_duplicate` | Implemented and tested |
| **Read words from file** | `Word Games.pdf` p.1 | [`batch::load_words_from_reader`](file:///c:/Users/User/Downloads/Programming%20Lab/Word%20Games/1%20-%20Library%20Trie/trie-cli/src/batch.rs#L34) | `test_load_words_from_reader_mixed_content` | Implemented and tested |
| **Comma/space delimited lists** | `Word Games.pdf` p.1 | [`batch::parse_tokens`](file:///c:/Users/User/Downloads/Programming%20Lab/Word%20Games/1%20-%20Library%20Trie/trie-cli/src/batch.rs#L11) | `test_parse_tokens_various_separators` | Implemented and tested |
| **Lowercase to uppercase** | `Word Games.pdf` p.1 | [`trie::normalize_word`](file:///c:/Users/User/Downloads/Programming%20Lab/Word%20Games/1%20-%20Library%20Trie/trie/src/lib.rs#L131) | `test_normalize_valid_words`, `test_insert_case_normalization` | Implemented and tested |
| **Delete word from Trie** | `Word Games.pdf` p.1 | [`Trie::remove`](file:///c:/Users/User/Downloads/Programming%20Lab/Word%20Games/1%20-%20Library%20Trie/trie/src/lib.rs#L327) | `test_remove_leaf_word_and_pruning` | Implemented and tested |
| **Delete list of words** | `Word Games.pdf` p.1 | [`Trie::remove_words`](file:///c:/Users/User/Downloads/Programming%20Lab/Word%20Games/1%20-%20Library%20Trie/trie/src/lib.rs#L358) | `test_remove_words_batch` | Implemented and tested |
| **Return words in Trie** | `Word Games.pdf` p.1 | [`batch::check_words_from_reader`](file:///c:/Users/User/Downloads/Programming%20Lab/Word%20Games/1%20-%20Library%20Trie/trie-cli/src/batch.rs#L54) | `test_check_words_from_reader` | Implemented and tested |
| **Return words not in Trie**| `Word Games.pdf` p.1 | [`batch::check_words_from_reader`](file:///c:/Users/User/Downloads/Programming%20Lab/Word%20Games/1%20-%20Library%20Trie/trie-cli/src/batch.rs#L54) | `test_check_words_from_reader` | Implemented and tested |
| **Prefix verification** | Specification / Extension | [`Trie::has_prefix`](file:///c:/Users/User/Downloads/Programming%20Lab/Word%20Games/1%20-%20Library%20Trie/trie/src/lib.rs#L296) | `test_has_prefix` | Implemented and tested |
| **Zero-alloc TrieCursor** | Specification / Game Ext | [`TrieCursor`](file:///c:/Users/User/Downloads/Programming%20Lab/Word%20Games/1%20-%20Library%20Trie/trie/src/lib.rs#L106) | `test_cursor_advance_existing_prefix_and_absent` | Implemented and tested |
| **Differential Testing** | Meta-Testing requirement | [`tests/differential_testing.rs`](file:///c:/Users/User/Downloads/Programming%20Lab/Word%20Games/1%20-%20Library%20Trie/trie/tests/differential_testing.rs) | `test_differential_randomized_operations` | Implemented and tested |

---

### Matrix 2: Spelling Bee (`spelling-bee`)

| Requirement | Assignment Source | Implementation Location | Validation Evidence | Status |
| :--- | :--- | :--- | :--- | :---: |
| **7 distinct letters** | Lab Spec / NYT Rules | [`SpellingBeePuzzle::new`](file:///c:/Users/User/Downloads/Programming%20Lab/Word%20Games/2%20-%20Spelling%20Bee/src/puzzle.rs#L39) | `test_valid_puzzle_construction`, `test_duplicate_letters_rejected` | Implemented and tested |
| **Designate main letter** | Lab Spec / NYT Rules | [`SpellingBeePuzzle::main_letter`](file:///c:/Users/User/Downloads/Programming%20Lab/Word%20Games/2%20-%20Spelling%20Bee/src/puzzle.rs#L94) | `test_main_letter_not_first_rejected_by_new` | Implemented and tested |
| **Mandatory main letter** | Lab Spec / NYT Rules | [`SpellingBeeGame::guess`](file:///c:/Users/User/Downloads/Programming%20Lab/Word%20Games/2%20-%20Spelling%20Bee/src/game.rs#L44) | `test_rejection_conditions_do_not_modify_state` | Implemented and tested |
| **Allow letter repetition**| Lab Spec / NYT Rules | [`SpellingBeePuzzle::bitmask_for_word`](file:///c:/Users/User/Downloads/Programming%20Lab/Word%20Games/2%20-%20Spelling%20Bee/src/puzzle.rs#L112) | `test_bitmask_operations` | Implemented and tested |
| **Minimum 4 letters** | Lab Spec / NYT Rules | [`scoring::calculate_score`](file:///c:/Users/User/Downloads/Programming%20Lab/Word%20Games/2%20-%20Spelling%20Bee/src/scoring.rs#L11) | `test_short_words_score_zero` | Implemented and tested |
| **Trie DFS Solver** | Design Decision | [`solver::solve_spelling_bee`](file:///c:/Users/User/Downloads/Programming%20Lab/Word%20Games/2%20-%20Spelling%20Bee/src/solver.rs#L16) | `test_solver_with_controlled_trie`, Benchmark test | Implemented and tested |
| **Pangram detection (+7)**| Lab Spec / NYT Rules | [`scoring::calculate_score`](file:///c:/Users/User/Downloads/Programming%20Lab/Word%20Games/2%20-%20Spelling%20Bee/src/scoring.rs#L11) | `test_pangram_bonus` | Implemented and tested |
| **Queen Bee detection** | Lab Spec / NYT Rules | [`SpellingBeeGame::is_queen_bee`](file:///c:/Users/User/Downloads/Programming%20Lab/Word%20Games/2%20-%20Spelling%20Bee/src/game.rs#L90) | `test_queen_bee_and_remaining_words` | Implemented and tested |
| **3-Column word display** | Lab Spec Requirement | [`main::format_three_columns`](file:///c:/Users/User/Downloads/Programming%20Lab/Word%20Games/2%20-%20Spelling%20Bee/src/main.rs#L17) | `test_format_three_columns_empty_and_populated` | Implemented and tested |
| **Surrender / Missed list**| Lab Spec Requirement | [`SpellingBeeGame::missed_words`](file:///c:/Users/User/Downloads/Programming%20Lab/Word%20Games/2%20-%20Spelling%20Bee/src/game.rs#L104) | `test_queen_bee_and_remaining_words` | Implemented and tested |
| **Hint grid matrix** | Optional Feature | [`hints::SpellingBeeHints::build`](file:///c:/Users/User/Downloads/Programming%20Lab/Word%20Games/2%20-%20Spelling%20Bee/src/hints.rs#L25) | `test_hint_grid_and_prefix_counts` | Implemented and tested |

---

### Matrix 3: Boggle Game Engine (`boggle`)

| Requirement | Assignment Source | Implementation Location | Validation Evidence | Status |
| :--- | :--- | :--- | :--- | :---: |
| **4x4, 5x5, 6x6 Boards** | Lab Spec Requirement | [`Board::new`](file:///c:/Users/User/Downloads/Programming%20Lab/Word%20Games/3%20-%20Boggle/src/board.rs#L28) | `test_valid_board_construction_all_dimensions` | Implemented and tested |
| **Reject invalid sizes** | Robustness Requirement| [`Board::new`](file:///c:/Users/User/Downloads/Programming%20Lab/Word%20Games/3%20-%20Boggle/src/board.rs#L28) | `test_invalid_dimensions_rejected` | Implemented and tested |
| **Letter repetition** | Lab Spec Requirement | [`Board::new`](file:///c:/Users/User/Downloads/Programming%20Lab/Word%20Games/3%20-%20Boggle/src/board.rs#L28) | `test_repeated_letters_in_different_cells_accepted` | Implemented and tested |
| **Min length 3 letters** | Official Rules | [`scoring::calculate_score`](file:///c:/Users/User/Downloads/Programming%20Lab/Word%20Games/3%20-%20Boggle/src/scoring.rs#L12) | `test_under_three_letters_score_zero` | Implemented and tested |
| **8-Directional movement** | Official Rules | [`board::DIRECTIONS`](file:///c:/Users/User/Downloads/Programming%20Lab/Word%20Games/3%20-%20Boggle/src/board.rs#L14) | `test_horizontal_vertical_diagonal_paths` | Implemented and tested |
| **No physical cell reuse**| Official Rules | [`validator::can_form_word`](file:///c:/Users/User/Downloads/Programming%20Lab/Word%20Games/3%20-%20Boggle/src/validator.rs#L15) | `test_cell_reuse_rejected` | Implemented and tested |
| **Dictionary verification**| Lab Spec Requirement | [`validator::is_dictionary_word`](file:///c:/Users/User/Downloads/Programming%20Lab/Word%20Games/3%20-%20Boggle/src/validator.rs#L126) | `test_boggle_full_integration_all_dimensions` | Implemented and tested |
| **Batch word evaluation** | Lab Spec Requirement | [`BoggleGame::evaluate_batch`](file:///c:/Users/User/Downloads/Programming%20Lab/Word%20Games/3%20-%20Boggle/src/game.rs#L64) | `test_batch_evaluation_flow` | Implemented and tested |
| **Official scoring table**| Lab Spec Requirement | [`scoring::calculate_score`](file:///c:/Users/User/Downloads/Programming%20Lab/Word%20Games/3%20-%20Boggle/src/scoring.rs#L12) | `test_standard_boggle_length_boundaries` | Implemented and tested |
| **Optional Qu Tile** | Optional Feature | [`validator::can_form_word_with_qu`](file:///c:/Users/User/Downloads/Programming%20Lab/Word%20Games/3%20-%20Boggle/src/validator.rs#L48) | `test_qu_tile_support` | Implemented and tested |
| **Automatic Board Solver**| Optional Feature | [`validator::solve_boggle_board`](file:///c:/Users/User/Downloads/Programming%20Lab/Word%20Games/3%20-%20Boggle/src/validator.rs#L133) | `test_automatic_solver_with_controlled_trie` | Implemented and tested |
| **Round countdown timer** | Optional Feature | [`timer::BoggleTimer`](file:///c:/Users/User/Downloads/Programming%20Lab/Word%20Games/3%20-%20Boggle/src/timer.rs#L10) | Interactive CLI verification | Implemented and tested |
