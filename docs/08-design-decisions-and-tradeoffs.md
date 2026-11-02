# 08. Design Decisions, Architectural Trade-offs & Rust Concepts

This document provides a rigorous architectural evaluation of the software engineering decisions, trade-offs, and practical applications of Rust language idioms across the Word Games ecosystem.

---

## 1. Rust Language Idioms Applied in the Codebase

### A. Ownership, Move Semantics & Parameter Passing
- **Pass-by-Reference (`&Trie`):** The dictionary Trie occupies $\approx 42\text{ MB}$ of memory. Passing `&trie` prevents catastrophic heap copying, granting game engines read-only access while the caller retains ownership.
- **Move Semantics (`Board` into `BoggleGame`):** The constructor `BoggleGame::new(board, ...)` takes `board` by value (`Board`). Ownership is transferred directly into the game struct, enforcing that the game state owns its board without requiring an `Arc` or reference indirection.
- **Ownership Reclamation in Loops:** In Spelling Bee solution precomputation, passing `&mut String` into recursive functions allows pushing and popping single characters in-place without allocating new `String` buffers at each tree level.

### B. Lifetimes & Borrow Checker Mediation (`TrieCursor<'a>`)
```rust
pub struct TrieCursor<'a> {
    node: Option<&'a Node>,
}
```
- The explicit lifetime `'a` binds the cursor immutably to the underlying `Trie`.
- The Rust borrow checker statically forbids mutating the `Trie` while any `TrieCursor` exists.
- Because `TrieCursor` implements `Copy`, branching during recursive search incurs zero allocation: only an 8-byte pointer is copied onto the stack.

### C. Typesafe Algebraic Data Types (`enum`)
Instead of sentinel values (e.g., returning `-1` or integer error codes), the games define exhaustive domain enums:
```rust
pub enum GuessResult {
    Accepted { word: String, points: u32, is_pangram: bool },
    AlreadyGuessed,
    MissingMainLetter(char),
    InvalidLetters,
    TooShort(usize),
    NotInDictionary,
}
```
Pattern matching (`match guess_result`) forces the UI layer to handle every possible game condition at compile time, eliminating unhandled runtime errors.

### D. Bitwise Manipulation for $O(1)$ State Tracking
Rather than iterating over `HashSet<char>` collections to detect pangrams and letter validity, Spelling Bee encodes the 7 hive letters into a `u8` bitmask:
- Bitwise OR (`mask |= bit`) tracks accumulated unique letters.
- Bitwise AND (`mask & MAIN_LETTER_BIT != 0`) validates center letter inclusion in 1 CPU cycle.
- Equality (`mask == 0x7F`) identifies pangrams with zero allocations.

---

## 2. In-Depth Algorithmic Comparisons & Trade-offs

### Decision 1: Trie-Guided DFS vs. Brute-Force Dictionary Scanning (Spelling Bee)
- **What Was Implemented:** A recursive Depth-First Search traversing the Trie using `TrieCursor`, exploring only the 7 hive letters and instantly pruning absent branches.
- **Why Chosen:** The dictionary contains **176,413 words**. Checking every word against the hive takes $O(N \cdot L)$ time. Trie DFS visits only valid paths formed by the 7 letters.
- **Comparative Evidence:**
  - Brute-Force Scan: **$42.44\text{ ms}$**
  - Trie-Guided DFS: **$0.208\text{ ms}$**
  - **Speedup: $\sim 204\times$!**
- **Trade-off:** Requires an encapsulated, traversable Trie data structure with cursor support; higher implementation complexity than a simple line-by-line scanner.

---

### Decision 2: Flat Vector (`Vec<char>`) vs. 2D Vector (`Vec<Vec<char>>`) (Boggle)
- **What Was Implemented:** A 1D contiguous vector `cells: Vec<char>` of length $size^2$, mapped via $\text{index} = \text{row} \times \text{size} + \text{column}$.
- **Why Chosen:**
  1. **Cache Locality:** Contiguous memory layout ensures all grid cells reside on a single cache line, eliminating pointer chasing.
  2. **Allocation Overhead:** A 2D vector incurs $size + 1$ heap allocations; a flat vector incurs exactly 1 allocation.
  3. **Indexing Speed:** Multiplications and modulo operations on small fixed sizes ($4, 5, 6$) are optimized into bit-shifts and additions by LLVM.
- **Trade-off:** Requires arithmetic coordinate conversions rather than `board[r][c]`.

---

### Decision 3: Precomputed Adjacency Lists vs. Dynamic Boundary Checking (Boggle)
- **What Was Implemented:** `board.neighbors: Vec<Vec<usize>>` precomputed once during `Board::new()`.
- **Why Chosen:** During recursive path validation and automatic board solving, adjacent neighbors are traversed thousands of times. Dynamic coordinate checks (`row + dr >= 0 && ...`) inside the hot loop add branch mispredictions and boundary overhead.
- **Advantage:** During DFS, `for &neighbor_idx in board.neighbors_of(cell_idx)` iterates directly over pre-filtered valid indices.
- **Trade-off:** Consumes a small amount of heap memory ($\approx 16 \times 8 \times 8 = 1\text{ KB}$ for 4x4), an entirely negligible trade-off for substantial execution gains.

---

### Decision 4: Visited Boolean Array vs. In-Place Cell Mutation (Boggle)
- **What Was Implemented:** `visited: Vec<bool>` passed mutably down recursive calls, tracking physical cell visits and unmarking cells upon backtracking.
- **Alternative Evaluated:** Replacing visited characters with sentinel values (e.g. `cells[idx] = '#'`) to avoid a visited array.
- **Why Chosen:** Mutating `cells` requires `&mut Board`, making concurrent solver threads or immutable board inspection impossible. An explicit `visited` slice maintains board immutability while guaranteeing that a physical cell is never revisited in the same word.
- **Trade-off:** Requires diligent state restoration (`visited[idx] = false`) across all exit paths, including early returns.

---

### Decision 5: Batch Input Evaluation vs. Single-Word Turn Input (Boggle)
- **What Was Implemented:** Players enter all found words in a comma- or whitespace-separated line (`"CART, PEAS, ROAST, TEAMS"`). The engine evaluates the full batch, aggregates points, and reports categorized results.
- **Why Chosen:** Mirrors authentic Boggle gameplay where players write down words during a 3-minute round, pencils down, and all words are validated simultaneously without retraction.
- **Trade-off:** Requires a robust token parsing engine capable of handling mixed separators (commas, spaces, tabs, trailing delimiters).

---

### Decision 6: Deterministic Metamorphic Testing vs. Unverified Ad-Hoc Tests
- **What Was Implemented:** Randomized differential testing running 1,000 operations comparing the custom `Trie` against `std::collections::BTreeSet` with a fixed pseudo-random seed (`0xDEAD_BEEF_CAFE_BABE`).
- **Why Chosen:** Standard hand-written unit tests only test scenarios anticipated by the author. Randomized metamorphic testing explores edge cases (e.g. deleting non-existent prefixes of existing longer words) and guarantees exact behavioural equivalence with standard library collections.
- **Advantage:** Reproducible failure traces on any platform.
