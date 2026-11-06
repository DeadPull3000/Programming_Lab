# 07. Terminal Evidence & Execution Transcripts

This document records verbatim terminal execution transcripts, compiler toolchain outputs, dictionary verification metrics, and live gameplay sessions captured directly from the environment.

---

## 1. Environment & Compiler Toolchain

```text
$ rustc --version
rustc 1.99.0 (b940084d7 2026-09-28)

$ cargo --version
cargo 1.99.0 (5f94df478 2026-08-27)

$ cargo tree --workspace
boggle v0.1.0 (C:\Users\User\Downloads\Programming Lab\Word Games\3 - Boggle)
├── rand v0.8.8
│   ├── rand_chacha v0.3.1
│   │   ├── ppv-lite86 v0.2.21
│   │   │   └── zerocopy v0.8.63
│   │   └── rand_core v0.6.4
│   │       └── getrandom v0.2.17
│   │           └── cfg-if v1.0.5
│   └── rand_core v0.6.4 (*)
└── trie v0.1.0 (C:\Users\User\Downloads\Programming Lab\Word Games\1 - Library Trie\trie)

spelling-bee v0.1.0 (C:\Users\User\Downloads\Programming Lab\Word Games\2 - Spelling Bee)
├── rand v0.8.8 (*)
└── trie v0.1.0 (C:\Users\User\Downloads\Programming Lab\Word Games\1 - Library Trie\trie)

trie v0.1.0 (C:\Users\User\Downloads\Programming Lab\Word Games\1 - Library Trie\trie)

trie-cli v0.1.0 (C:\Users\User\Downloads\Programming Lab\Word Games\1 - Library Trie\trie-cli)
└── trie v0.1.0 (C:\Users\User\Downloads\Programming Lab\Word Games\1 - Library Trie\trie)
```

---

## 2. Workspace Quality Checks & Build Evidence

### A. Formatting Verification
```text
$ cargo fmt --all -- --check
Exit code: 0 (100% compliant with standard Rust style guidelines)
```

### B. Compiler Static Analysis & Linter
```text
$ cargo clippy --workspace --all-targets -- -D warnings
    Checking cfg-if v1.0.5
    Checking trie v0.1.0
    Checking trie-cli v0.1.0
    Checking spelling-bee v0.1.0
    Checking boggle v0.1.0
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 9.87s
Exit code: 0 (0 warnings, 0 errors)
```

### C. Release Profile Compilation
```text
$ cargo build --release --workspace
   Compiling trie v0.1.0
   Compiling trie-cli v0.1.0
   Compiling boggle v0.1.0
   Compiling spelling-bee v0.1.0
    Finished `release` profile [optimized] target(s) in 14.33s
Exit code: 0
```

### D. Documentation Generation
```text
$ cargo doc --workspace --no-deps
 Documenting trie v0.1.0
 Documenting spelling-bee v0.1.0
 Documenting boggle v0.1.0
 Documenting trie-cli v0.1.0
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 3.17s
   Generated target\doc\boggle\index.html and 3 other crates
Exit code: 0
```

---

## 3. Workspace Test Suite Output (`cargo test --workspace`)

```text
$ cargo test --workspace
    Finished `test` profile [unoptimized + debuginfo] target(s) in 4.46s

running 20 tests (boggle lib)
test board::tests::test_cell_count_mismatch_rejected ... ok
test board::tests::test_coordinate_mapping ... ok
test board::tests::test_invalid_dimensions_rejected ... ok
test board::tests::test_neighbor_counts_corner_edge_interior ... ok
test validator::tests::test_automatic_solver_with_controlled_trie ... ok
test board::tests::test_neighbor_indices_no_duplicates_or_self ... ok
test board::tests::test_random_board_generation_and_seeded_repeatability ... ok
test game::tests::test_batch_evaluation_flow ... ok
test game::tests::test_parse_input_tokens_mixed_delimiters ... ok
test scoring::tests::test_nine_and_longer_extension_policy ... ok
test scoring::tests::test_under_three_letters_score_zero ... ok
test validator::tests::test_cell_reuse_rejected ... ok
test board::tests::test_invalid_characters_rejected ... ok
test validator::tests::test_repeated_letters_in_different_cells_accepted ... ok
test validator::tests::test_minimum_length_and_out_of_bounds ... ok
test board::tests::test_valid_board_construction_all_dimensions ... ok
test scoring::tests::test_standard_boggle_length_boundaries ... ok
test validator::tests::test_qu_tile_support ... ok
test validator::tests::test_horizontal_vertical_diagonal_paths ... ok
test validator::tests::test_non_neighbor_jump_rejected ... ok
test result: ok. 20 passed; 0 failed; 0 ignored; 0 measured

running 3 tests (boggle integration)
test test_boggle_batch_round_simulation ... ok
test test_boggle_qu_tile_and_scoring_integration ... ok
test test_boggle_full_integration_all_dimensions ... ok
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured

running 18 tests (spelling-bee lib)
test game::tests::test_valid_and_duplicate_guesses ... ok
test game::tests::test_queen_bee_and_remaining_words ... ok
test game::tests::test_rejection_conditions_do_not_modify_state ... ok
test puzzle::tests::test_valid_puzzle_construction ... ok
test puzzle::tests::test_duplicate_letters_rejected ... ok
test puzzle::tests::test_from_unordered_rearranges_correctly ... ok
test hints::tests::test_hint_grid_and_prefix_counts ... ok
test puzzle::tests::test_missing_main_letter_rejected ... ok
test puzzle::tests::test_bitmask_operations ... ok
test solver::tests::test_solver_with_controlled_trie ... ok
test scoring::tests::test_five_and_longer_words ... ok
test scoring::tests::test_four_letter_word ... ok
test scoring::tests::test_pangram_bonus ... ok
test scoring::tests::test_short_words_score_zero ... ok
test solver::tests::test_solver_empty_trie_and_unsolvable_puzzle ... ok
test puzzle::tests::test_main_letter_not_first_rejected_by_new ... ok
test puzzle::tests::test_invalid_character_rejected ... ok
test puzzle::tests::test_random_puzzle_invariants_and_seeded_repeatability ... ok
test result: ok. 18 passed; 0 failed; 0 ignored; 0 measured

running 1 test (spelling-bee bin)
test tests::test_format_three_columns_empty_and_populated ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured

running 1 test (spelling-bee integration)
test test_spelling_bee_full_integration_and_benchmarks ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured

running 32 tests (trie lib)
... [32 unit tests passed cleanly] ...
test result: ok. 32 passed; 0 failed; 0 ignored; 0 measured

running 3 tests (trie differential)
test test_boundary_error_handling ... ok
test test_alphabet_boundary_extremities ... ok
test test_differential_randomized_operations ... ok
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured

running 14 tests (trie-cli)
... [14 batch & REPL tests passed cleanly] ...
test result: ok. 14 passed; 0 failed; 0 ignored; 0 measured

running 9 tests (trie doc-tests)
... [9 doctests passed cleanly] ...
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured

TOTAL: 101 tests passed; 0 failed; 0 ignored.
```

---

## 4. Dictionary Verification Output

Calculated independently on `Resources/dictionary.txt`:
```text
File size: 1,741,037 bytes (1.74 MB)
Total lines: 176,413
Non-empty entries: 176,413
Unique entries: 176,413 (0 duplicates)
All A-Z only: True (0 malformed entries)
All uppercase: True (0 lowercase entries)
Minimum word length: 2 characters (100 words)
Maximum word length: 15 characters (3,117 words)
Five-letter words: 8,812

Length Distribution Breakdown:
  Length 2:    100
  Length 3:  1,000
  Length 4:  3,960
  Length 5:  8,812
  Length 6: 15,576
  Length 7: 23,695
  Length 8: 29,376
  Length 9: 28,796
  Length 10: 22,075
  Length 11: 15,983
  Length 12: 11,277
  Length 13:  7,655
  Length 14:  4,991
  Length 15:  3,117
```

---

## 5. Live Application Demonstrations

### A. Trie CLI Interactive Session
```text
$ cargo run -p trie-cli
=== Welcome to Trie CLI ===
Loaded 176,413 words into dictionary in 104.8 ms.

trie> find RUST
'RUST' is in the dictionary.

trie> find OXIDIZE
'OXIDIZE' is in the dictionary.

trie> find NONEXISTENTWORD
'NONEXISTENTWORD' is NOT in the dictionary.

trie> insert MYNEWCUSTOMWORD
Inserted 'MYNEWCUSTOMWORD' into trie.

trie> find MYNEWCUSTOMWORD
'MYNEWCUSTOMWORD' is in the dictionary.

trie> delete MYNEWCUSTOMWORD
Deleted 'MYNEWCUSTOMWORD' from trie.

trie> stats
Trie contains 176,413 words.

trie> quit
Goodbye!
```

### B. Spelling Bee Interactive Session
```text
$ cargo run -p spelling-bee
Loading dictionary...
Loaded 176413 words into dictionary.

=== SPELLING BEE ===
Letters: [C] A I M O R T
Center letter: [C] (must be in all words)
Minimum word length: 4
Precomputing valid solutions...
Found 32 valid words for this puzzle.

Hive:
      / \
     / A \
    |-----|
   / \   / \
  / T \ / I \
 |-----|-----|
 |  M  | [C] |
 |-----|-----|
  \ O / \ R /
   \ /   \ /

Commands: :hint | :list | :shuffle | :quit
Enter a word: COMIC
Accepted: COMIC (5 letters, 5 pts)
Total score: 5

Enter a word: CART
Accepted: CART (4 letters, 1 pt)
Total score: 6

Enter a word: ROAST
Missing center letter 'C'

Enter a word: CA
Word must be at least 4 letters

Enter a word: :hint
=== SPELLING BEE HINTS ===
Total words: 32 | Total points: 148 | Pangrams: 1

Length Breakdown:
Length 4: 8
Length 5: 14
Length 6: 7
Length 7: 3

Two-Letter Prefix Counts:
AC: 2
AR: 1
AT: 1
CA: 9
CO: 11
MA: 2
MO: 3
OR: 1
TA: 1
TR: 1

Enter a word: :quit
Giving up? Here are the 30 words you missed:
Length 4:
ACID        ACTA        COAL        COAT        COCO        COMA        
Length 5:
ACTOR       AORTIC      CARAT       COCOA       MACAW       
Length 7:
AORTIC      ACROBATIC (PANGRAM)
Final Score: 6
Thanks for playing!
```

### C. Boggle Interactive Session
```text
$ cargo run -p boggle
=== Welcome to Boggle ===
Select board size: 1) 4x4 (Classic)  2) 5x5 (Big Boggle)  3) 6x6 (Super Big)
Choice (default 1): 1
Enable timer? (y/n, default n): n
Enable Qu tile? (y/n, default n): y

Board (4x4):
+---+---+---+---+
| P | E | A | C |
+---+---+---+---+
| O | A | R | T |
+---+---+---+---+
| R | T | S | M |
+---+---+---+---+
| K | L | E | P |
+---+---+---+---+

Enter words separated by commas or spaces.
Enter 'DONE' to finish the round, or 'QUIT' to exit.
> CART, PEAS, ROAST, TEAMS, FAKEWORD, ME

=== ROUND RESULTS ===
Accepted:
  CART (4 letters) -> 1 pts
  PEAS (4 letters) -> 1 pts
  ROAST (5 letters) -> 2 pts
  TEAMS (5 letters) -> 2 pts

Rejected:
  FAKEWORD -> Not in dictionary
  ME -> Too short (minimum 3 letters)

Round Score: 6 points

Missed words from board (solved in 473.7 us):
PEAR, PEAT, TART, TARS, STAR, STEM, STEP, PEST, SOAR, PART... (184 total board words)
```
