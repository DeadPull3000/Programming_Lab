# Boggle: Architecture and Design Specification

## 1. System Overview

Boggle is a word game played on an $N \times N$ grid of letters (standard dimensions: 4x4, 5x5, 6x6). Words of three or more letters are formed by connecting adjacent cells horizontally, vertically, or diagonally.

### Core Architectural Principle
The application is decoupled into:
1. **Board Topology and Adjacency**: Mathematical representation of the grid and 8-directional neighbor navigation.
2. **Path Search and Validation**: Backtracking depth-first search ensuring physical cells are not reused within a word.
3. **Dictionary Verification**: Immutable lookup via the shared Trie dictionary.
4. **Batch Game Management**: Collecting input batches, single-pass evaluation, preventing retractions, and tracking scores.
5. **Automatic Solver**: Trie-guided DFS exploring all valid paths on the board with prefix pruning.

---

## 2. Module Responsibilities

| Module | File | Primary Responsibility |
| :--- | :--- | :--- |
| `board` | [`src/board.rs`](file:///c:/Users/User/Downloads/Programming%20Lab/Word%20Games/word-trie/boggle/src/board.rs) | Board struct, dimension validation (4, 5, 6), coordinate mapping, 8-way neighbors, random generation, and display. |
| `validator` | [`src/validator.rs`](file:///c:/Users/User/Downloads/Programming%20Lab/Word%20Games/word-trie/boggle/src/validator.rs) | Recursive path validation with backtracking, dictionary verification, Qu tile logic, and automatic Trie solver. |
| `scoring` | [`src/scoring.rs`](file:///c:/Users/User/Downloads/Programming%20Lab/Word%20Games/word-trie/boggle/src/scoring.rs) | Implementation of the assignment scoring table and documented extension policy for 9+ letter words. |
| `game` | [`src/game.rs`](file:///c:/Users/User/Downloads/Programming%20Lab/Word%20Games/word-trie/boggle/src/game.rs) | Round state, flexible input parsing (comma/space/mixed delimiters), batch evaluation, and duplicate prevention. |
| `main` | [`src/main.rs`](file:///c:/Users/User/Downloads/Programming%20Lab/Word%20Games/word-trie/boggle/src/main.rs) | Interactive two-level REPL, dimension selection, optional timer, and post-round missing word reporting. |

---

## 3. Data Structures and Coordinate Representation

### 3.1. Flat Vector Board Representation
```rust
pub struct Board {
    pub size: usize,
    pub cells: Vec<char>,
}
```
- **Dimension Invariants**: `size` must be in `[4, 5, 6]`. `cells.len()` must equal `size * size`.
- **Coordinate Conversion**:
  $$\text{index} = \text{row} \times \text{size} + \text{column}$$
  $$\text{row} = \lfloor\text{index} / \text{size}\rfloor,\quad \text{column} = \text{index} \pmod{\text{size}}$$
- **Memory Efficiency**: Contiguous memory layout maximizes cache locality during neighbor traversals.

### 3.2. 8-Directional Adjacency
Neighbors are generated using signed offsets:
```rust
pub const DIRECTIONS: [(isize, isize); 8] = [
    (-1, -1), (-1, 0), (-1, 1),
    ( 0, -1),          ( 0, 1),
    ( 1, -1), ( 1, 0), ( 1, 1),
];
```
Bounds are checked against $0 \le r, c < \text{size}$ before converting back to `usize`.

#### Neighbor Count Invariants:
- **Corner positions**: Exactly 3 neighbors.
- **Edge positions**: Exactly 5 neighbors.
- **Interior positions**: Exactly 8 neighbors.

---

## 4. Algorithms

### 4.1. Recursive Path Validation with Backtracking
Given a word $W = c_0 c_1 \dots c_{k-1}$:
1. Iterate over all cell indices $i$ where `board.cells[i] == c_0`.
2. Maintain a boolean array `visited: Vec<bool>` of size $N^2$.
3. Recursive helper `dfs_search_path(current_idx, char_idx)`:
   - If `char_idx == W.len()`, a complete valid path is found; return `true`.
   - Set `visited[current_idx] = true`.
   - For each neighbor `nbr` of `current_idx`:
     - If `!visited[nbr]` and `board.cells[nbr] == W[char_idx]`:
       - Recursively search `char_idx + 1`.
       - If branch succeeds, reset `visited[current_idx] = false` and return `true`.
   - Backtrack: reset `visited[current_idx] = false` and return `false`.

#### Physical Cell vs. Letter Identity:
A word may reuse the letter 'A' if multiple cells contain 'A'. However, `visited[idx]` tracks the *physical board coordinate*, ensuring the same physical cell is never used twice in one word.

### 4.2. Trie-Guided Automatic Boggle Solver (Optional Feature)
Instead of searching words one by one, the automatic solver explores the board from every cell simultaneously pruning via the Trie:
1. Start at each cell $i$, advancing `trie.root_cursor().advance(board.cells[i])`.
2. If the cursor is valid, mark cell $i$ visited and append to `word_buf`.
3. If `cursor.is_word()` and `word_buf.len() >= 3`, insert into `HashSet<String>`.
4. For each unvisited neighbor $j$, check `cursor.advance(board.cells[j])`. If valid, recurse.
5. Backtrack by popping `word_buf` and clearing `visited[i]`.

---

## 5. Scoring Table and Extension Policy

| Length | Points | Rule Category |
| :--- | :--- | :--- |
| $< 3$ letters | 0 | Invalid length under Boggle rules |
| 3–4 letters | 1 | Explicit assignment specification |
| 5 letters | 2 | Explicit assignment specification |
| 6 letters | 3 | Explicit assignment specification |
| 7 letters | 5 | Explicit assignment specification |
| 8 letters | 11 | Explicit assignment specification |
| 9+ letters | $2 \times \text{length}$ | Documented extension policy (e.g. 9 = 18 pts, 10 = 20 pts) |

---

## 6. Optional Features Implemented

1. **Qu Tile Mechanics**: Standard Boggle allows 'Q' on the board to match the two-character sequence "QU" in player guesses while consuming a single physical cell.
2. **Automatic Boggle Solver**: Uses Trie DFS with prefix pruning to find all valid dictionary words on any generated board in under 2 milliseconds.
3. **Timed Rounds**: Optional 3-minute round countdown using non-blocking timestamps (`std::time::Instant`).
