# Spelling Bee: Testing and Validation Methodology

## 1. Testing Philosophy

The testing strategy for Spelling Bee combines:
1. **Unit Tests with Controlled Fixtures**: Testing isolated edge cases and invariants on small, deterministic mock dictionaries without filesystem I/O.
2. **Deterministic Seeded Random Tests**: Reproducible procedural generation using fixed seeds (`StdRng::seed_from_u64`).
3. **Differential Reference Verification**: Comparing the high-speed Trie DFS solver against an independent brute-force reference implementation over the entire 176,413-word dictionary.
4. **End-to-End Game State Simulation**: Full round progression testing duplicate guesses, illegal character rejections, score integrity, and Queen Bee termination.

---

## 2. Test Coverage Matrix

### 2.1. Module Unit Tests (`spelling-bee/src`)

| Test Name | Target Module | Condition Verified |
| :--- | :--- | :--- |
| `test_valid_puzzle_construction` | `puzzle` | Happy path creation with valid uppercase distinct letters. |
| `test_duplicate_letters_rejected` | `puzzle` | Rejection of duplicate letters with `PuzzleError::DuplicateLetter`. |
| `test_invalid_character_rejected` | `puzzle` | Rejection of numbers, punctuation, and lowercase characters. |
| `test_missing_main_letter_rejected` | `puzzle` | Rejection when main letter is not in the 7-letter array. |
| `test_main_letter_not_first_rejected` | `puzzle` | Direct constructor requires `letters[0] == main_letter`. |
| `test_from_unordered_rearranges` | `puzzle` | Reorders arbitrary sets so main letter is positioned at index 0. |
| `test_random_puzzle_invariants` | `puzzle` | Verifies reproducibility with seeded RNG and distinct letter set. |
| `test_bitmask_operations` | `puzzle` | Validates bit shifts, pangram mask ($127$), main letter bit ($1$). |
| `test_solver_with_controlled_trie` | `solver` | Verifies Trie DFS against reference solver on controlled word set. |
| `test_solver_empty_trie` | `solver` | Robustness on empty Trie or unsolvable letter combinations. |
| `test_short_words_score_zero` | `scoring` | Bounds test: words $< 4$ letters score 0 points. |
| `test_four_letter_word` | `scoring` | 4-letter words award exactly 1 point. |
| `test_five_and_longer_words` | `scoring` | Words of length $N \ge 5$ award $N$ points. |
| `test_pangram_bonus` | `scoring` | Pangrams award base points plus 7 bonus points. |
| `test_valid_and_duplicate_guesses` | `game` | Score increments on valid guesses; duplicate guesses rejected. |
| `test_rejection_conditions` | `game` | Score remains strictly unchanged across all rejection modes. |
| `test_queen_bee_and_remaining` | `game` | Queen Bee accolade fires when `guessed.len() == solutions.len()`. |
| `test_hint_grid_and_prefix_counts` | `hints` | Alphabetical ordering, row totals, column totals, and grand total. |
| `test_format_three_columns` | `main` | Clean multi-column layout top-to-bottom across 3 columns. |

---

## 3. Integration and Benchmark Results

Executed via:
```bash
cargo test -p spelling-bee --test spelling_bee_integration -- --nocapture
```

### Actual Benchmark Measurements (176,413 Words):
- **Dictionary Loading Time**: $651.63\text{ ms}$
- **Procedural Puzzle Generation**: $81.30\ \mu\text{s}$
- **Trie-Guided DFS Solver**: **$208.10\ \mu\text{s}$**
- **Brute-Force Reference Filter**: $42.44\text{ ms}$
- **Speedup Factor**: The Trie-guided solver ran **$\sim 204\times$ faster** than brute-force filtering.
- **Correctness Check**: Both implementations discovered the exact same $26$ solutions ($0$ false positives, $0$ false negatives).
