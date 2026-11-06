# 05. Boggle Game Engine, Pathfinding & Solver Implementation

This document provides a comprehensive technical specification of the Boggle application (`3 - Boggle/`), detailing the board geometry, 8-directional recursive pathfinding, cell-reuse prevention, dual-stage validation, automated Trie solver, and batch round lifecycle.

---

## 1. Problem Definition & Rules

Boggle is a word search game played on a square grid of letter dice:
1. **Grid Sizes:** Supports standard $4 \times 4$ (16 cells), Big Boggle $5 \times 5$ (25 cells), and Super Big Boggle $6 \times 6$ (36 cells).
2. **Movement Invariant:** Words are formed by connecting adjacent letters horizontally, vertically, or diagonally (up to 8 neighbors per cell).
3. **Physical Cell Reuse Prohibition:** A single physical cell on the board cannot be used more than once in the formation of a single word.
4. **Letter Repetition Permitted:** If the letter `'A'` appears in two distinct cells (e.g. cell $(0, 0)$ and cell $(2, 3)$), both may be used in the same word (e.g., `"AREA"`), provided each individual cell is visited at most once.
5. **Minimum Word Length:** Words under 3 letters are invalid and score 0 points.
6. **Dual Validity Criteria:** A guess is valid if and only if:
   - A contiguous, unrepeated physical cell path exists on the board.
   - The word exists in the dictionary.

---

## 2. Board Representation & Coordinate Geometry

The board is encapsulated in `board::Board`:

```rust
pub const VALID_DIMENSIONS: &[usize] = &[4, 5, 6];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Board {
    size: usize,
    cells: Vec<char>,
    neighbors: Vec<Vec<usize>>,
}
```

### 1D Flat Vector vs. 2D Coordinate Mapping:
Rather than allocating nested vectors (`Vec<Vec<char>>`) which induce pointer indirection and cache misses, all grid cells are stored in a single contiguous 1D vector of length $size \times size$:

$$\text{index} = \text{row} \times \text{size} + \text{column}$$

$$\text{row} = \lfloor \text{index} / \text{size} \rfloor$$

$$\text{column} = \text{index} \pmod{\text{size}}$$

### Sample 4x4 Board Layout:
```text
  +---+---+---+---+
0 | P | E | A | C |   Index 0: P,  Index 1: E,  Index 2: A,  Index 3: C
  +---+---+---+---+
1 | O | A | R | T |   Index 4: O,  Index 5: A,  Index 6: R,  Index 7: T
  +---+---+---+---+
2 | R | T | S | M |   Index 8: R,  Index 9: T,  Index 10: S, Index 11: M
  +---+---+---+---+
3 | K | L | E | P |   Index 12: K, Index 13: L, Index 14: E, Index 15: P
  +---+---+---+---+
    0   1   2   3
```

---

## 3. Precomputed 8-Directional Adjacency Lists

Movement is permitted across 8 spatial directions:

```text
(-1,-1) NW | (-1, 0) N | (-1,+1) NE
 (0,-1)  W |    (X)    |  (0,+1)  E
(+1,-1) SW | (+1, 0) S | (+1,+1) SE
```

### Precomputation Rationale:
Calculating boundary checks (`row >= 0 && row < size`) repeatedly during recursive DFS degrades performance. Instead, `Board::new()` precomputes the valid neighbor index list for every cell once during initialization:
- **Corner cells (4 cells):** Exactly 3 neighbors.
- **Edge cells ($4 \times (size - 2)$ cells):** Exactly 5 neighbors.
- **Interior cells ($(size - 2)^2$ cells):** Exactly 8 neighbors.

---

## 4. Recursive Pathfinding with Backtracking DFS

Path validation is implemented in `validator::can_form_word`:

```rust
pub fn can_form_word(board: &Board, word: &str) -> bool {
    let chars: Vec<char> = word.to_ascii_uppercase().chars().collect();
    if chars.len() < MIN_BOGGLE_LENGTH || chars.len() > board.total_cells() {
        return false;
    }

    let mut visited = vec![false; board.total_cells()];
    for start_idx in 0..board.total_cells() {
        if board.get_cell(start_idx) == chars[0] {
            if dfs_check_path(board, &chars, 0, start_idx, &mut visited) {
                return true;
            }
        }
    }
    false
}
```

### Detailed Recursive Step Walkthrough (`dfs_check_path`):
```rust
fn dfs_check_path(
    board: &Board,
    chars: &[char],
    char_idx: usize,
    cell_idx: usize,
    visited: &mut [bool],
) -> bool {
    // 1. Base Case: If this is the last character in the word, path is found!
    if char_idx == chars.len() - 1 {
        return true;
    }

    // 2. Mark current cell as visited for this recursive branch
    visited[cell_idx] = true;
    let next_char = chars[char_idx + 1];

    // 3. Explore all precomputed adjacent neighbors
    for &neighbor_idx in board.neighbors_of(cell_idx) {
        if !visited[neighbor_idx] && board.get_cell(neighbor_idx) == next_char {
            if dfs_check_path(board, chars, char_idx + 1, neighbor_idx, visited) {
                // RESTORE STATE even on success before returning!
                visited[cell_idx] = false;
                return true;
            }
        }
    }

    // 4. Backtrack: Unmark cell so other candidate paths can utilize it
    visited[cell_idx] = false;
    false
}
```

### Critical Invariant: Restoring `visited[cell_idx] = false`
The visited array represents the path of the *current search branch*. If the search backtracks because a neighbor failed to complete the word, or even when unwinding after a successful match, `visited[cell_idx]` **must** be reset to `false`. Failing to restore state would corrupt subsequent path evaluations!

---

## 5. Worked Example: Path Validation vs. Physical Cell Reuse

Consider a $4 \times 4$ board with letters:
- Cell 0: `'C'`
- Cell 1: `'A'`
- Cell 5: `'R'`
- Cell 6: `'T'`

### Valid Word: `"CART"`
1. Start at Cell 0 (`'C'`). Mark `visited[0] = true`.
2. Inspect neighbors of Cell 0: Cell 1 has `'A'` and is unvisited. Move to Cell 1.
3. Mark `visited[1] = true`. Inspect neighbors of Cell 1: Cell 5 has `'R'`. Move to Cell 5.
4. Mark `visited[5] = true`. Inspect neighbors of Cell 5: Cell 6 has `'T'`.
5. Last character matched! Path found: $0 \to 1 \to 5 \to 6$. Return `true`.

### Invalid Cell Reuse: `"TART"` (when only one `'T'` exists at Cell 6)
1. Start at Cell 6 (`'T'`). Mark `visited[6] = true`.
2. Move to Cell 5 (`'R'`), then Cell 1 (`'A'`).
3. To form the final `'T'`, the search looks for an adjacent `'T'`. Cell 6 is adjacent, BUT `visited[6] == true`!
4. The path cannot reuse Cell 6. Word is rejected with `NotOnBoard`.

---

## 6. Official Scoring & 9+ Letter Extension Policy

Scoring is implemented in `scoring::calculate_score`:

| Word Length | Points Awarded | Assignment Rule Source |
| :---: | :---: | :--- |
| $< 3$ letters | **0 points** | Invalid word length |
| 3 or 4 letters | **1 point** | Standard Boggle table |
| 5 letters | **2 points** | Standard Boggle table |
| 6 letters | **3 points** | Standard Boggle table |
| 7 letters | **5 points** | Standard Boggle table |
| 8 letters | **11 points** | Standard Boggle table |
| $\ge 9$ letters | **$2 \times \text{length}$ points** | Documented extension policy (e.g., 9-letter word = 18 points) |

---

## 7. Optional Enhancements

### A. The "Qu" Special Tile
In physical Boggle, the letter `Q` almost always appears as `Qu` to avoid unplayable letters.
- In `validator::can_form_word_with_qu`, tile `'Q'` matches the two-character slice `"QU"`.
- Consumes 2 characters from the target word while occupying only 1 physical board cell.
- Scored according to full word length (including `'U'`).

### B. Automated Trie Board Solver (`solve_boggle_board`)
Instead of testing words one-by-one, the automatic solver performs a Trie-guided DFS across the entire board:
- Begins at every cell on the board with `trie.root_cursor()`.
- Advances `TrieCursor` to neighboring cells.
- If a prefix does not exist in the Trie, pruning occurs instantly, eliminating entire subtrees.
- **Measured Solver Performance:**
  - $4 \times 4$ Board: **$473.7\ \mu\text{s}$ ($0.47\text{ ms}$)**
  - $5 \times 5$ Board: **$1.13\text{ ms}$**
  - $6 \times 6$ Board: **$530.4\ \mu\text{s}$ ($0.53\text{ ms}$)**
- Extracts hundreds of valid dictionary words in under a single millisecond!

### C. Round Countdown Timer (`timer::BoggleTimer`)
- Encapsulates `std::time::Instant` and a specified `Duration` (e.g. 180 seconds).
- `is_expired()` provides non-blocking time checks during the game loop.
