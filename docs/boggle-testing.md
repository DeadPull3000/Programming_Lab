# Boggle: Testing and Validation Methodology

## 1. Testing Philosophy

The Boggle test suite verifies:
1. **Geometric and Topological Correctness**: Boundary constraints, corner/edge neighbor counts, and index conversions.
2. **Path Adjacency and Cell Tracking**: Verifying horizontal, vertical, diagonal, and zig-zag paths while strictly prohibiting physical cell reuse.
3. **Dictionary Separation**: Ensuring words that exist on the board but not in the dictionary (and vice-versa) are accurately differentiated.
4. **Batch Parsing and Evaluation**: Handling mixed delimiters (commas, spaces, tabs), duplicate guesses, and scoring schedules.
5. **Multi-Dimension Integration**: Full dictionary verification across 4x4, 5x5, and 6x6 boards.

---

## 2. Test Coverage Matrix

### 2.1. Module Unit Tests (`boggle/src`)

| Test Name | Target Module | Condition Verified |
| :--- | :--- | :--- |
| `test_valid_board_construction_all_dimensions` | `board` | Tests construction of 4x4 (16 cells), 5x5 (25 cells), 6x6 (36 cells). |
| `test_invalid_dimensions_rejected` | `board` | Rejects dimensions 3, 7, and negative sizes. |
| `test_cell_count_mismatch_rejected` | `board` | Rejects too few or too many cells for a declared dimension. |
| `test_invalid_characters_rejected` | `board` | Rejects lowercase letters, digits, and punctuation. |
| `test_coordinate_mapping` | `board` | Verifies `(r, c) <-> index` bidirectional conversion. |
| `test_neighbor_counts_corner_edge_interior` | `board` | Checks exact counts: corner = 3, edge = 5, interior = 8. |
| `test_neighbor_indices_no_duplicates_or_self` | `board` | Self-exclusion and uniqueness of neighbor sets. |
| `test_random_board_generation` | `board` | Reproducible generation with seeded RNG (`StdRng`). |
| `test_horizontal_vertical_diagonal_paths` | `validator` | Multi-direction path traversal on fixed board. |
| `test_non_neighbor_jump_rejected` | `validator` | Rejection of non-adjacent coordinate transitions. |
| `test_cell_reuse_rejected` | `validator` | Rejection of words requiring reuse of the same physical cell. |
| `test_repeated_letters_accepted` | `validator` | Acceptance when identical letters appear in different physical cells. |
| `test_qu_tile_support` | `validator` | 'Q' cell matching "QU" sequence in candidate word. |
| `test_automatic_solver_with_controlled_trie` | `validator` | Trie-guided solver finding words on controlled board. |
| `test_under_three_letters_score_zero` | `scoring` | Score 0 for lengths 0, 1, and 2. |
| `test_standard_boggle_length_boundaries` | `scoring` | Boundaries: 3–4 (1 pt), 5 (2 pts), 6 (3 pts), 7 (5 pts), 8 (11 pts). |
| `test_nine_and_longer_extension_policy` | `scoring` | Lengths 9, 10, 16 award $2 \times \text{length}$. |
| `test_parse_input_tokens_mixed_delimiters` | `game` | Splitting comma, space, tab, and mixed whitespace. |
| `test_batch_evaluation_flow` | `game` | Full round batch: valid, duplicate, invalid characters, not in dict, not on board. |

---

## 3. Integration Results and Benchmarks

Executed via:
```bash
cargo test -p boggle --test boggle_integration -- --nocapture
```

### Actual Benchmark Measurements (176,413 Words):
- **Dictionary Loading**: $608.66\text{ ms}$
- **4x4 Board Solving (16 cells)**: **$473.70\ \mu\text{s}$** (37 valid words found)
- **5x5 Board Solving (25 cells)**: **$1.13\text{ ms}$** (62 valid words found)
- **6x6 Board Solving (36 cells)**: **$530.40\ \mu\text{s}$** (31 valid words found)

### Invariant Verification:
Across all three board sizes, 100% of discovered words were verified against:
1. Minimum length $\ge 3$.
2. Confirmed presence in the Trie dictionary.
3. Path validity along adjacent physical cells with zero cell reuse.
