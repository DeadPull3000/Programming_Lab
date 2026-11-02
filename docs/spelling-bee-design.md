# Spelling Bee: Architecture and Design Specification

## 1. System Overview

Spelling Bee is a word-puzzle game based on constructing legitimate English words from a set of seven distinct letters, where one designated letter—the **main letter**—is mandatory in every word.

The application is architected around a **shared, game-independent Trie dictionary engine**. Rather than searching a dictionary of 176,000+ words on every player guess, the entire valid solution space is precomputed at startup in **sub-millisecond time** using constrained depth-first search (DFS) guided by a zero-allocation `TrieCursor`.

---

## 2. Module Responsibilities

The crate `spelling-bee` is divided into five cohesive modules with strict separation of concerns:

| Module | File | Primary Responsibility |
| :--- | :--- | :--- |
| `puzzle` | [`src/puzzle.rs`](file:///c:/Users/User/Downloads/Programming%20Lab/Word%20Games/word-trie/spelling-bee/src/puzzle.rs) | Invariant-enforcing puzzle representation, random letter generation, and bitmask mechanics. |
| `solver` | [`src/solver.rs`](file:///c:/Users/User/Downloads/Programming%20Lab/Word%20Games/word-trie/spelling-bee/src/solver.rs) | Trie-guided DFS solution enumeration and independent brute-force reference filtering. |
| `scoring` | [`src/scoring.rs`](file:///c:/Users/User/Downloads/Programming%20Lab/Word%20Games/word-trie/spelling-bee/src/scoring.rs) | Pure, stateless scoring policies and pangram bonus calculations. |
| `game` | [`src/game.rs`](file:///c:/Users/User/Downloads/Programming%20Lab/Word%20Games/word-trie/spelling-bee/src/game.rs) | Active game session, nine-stage guess validation, score progression, Queen Bee detection, and remaining word reporting. |
| `hints` | [`src/hints.rs`](file:///c:/Users/User/Downloads/Programming%20Lab/Word%20Games/word-trie/spelling-bee/src/hints.rs) | Optional structural hints: starting letter $\times$ length matrix with totals, and two-letter prefix frequencies. |
| `main` | [`src/main.rs`](file:///c:/Users/User/Downloads/Programming%20Lab/Word%20Games/word-trie/spelling-bee/src/main.rs) | Terminal CLI loop, three-column alphabetical display formatting, and user command routing. |

---

## 3. Data Structures and Invariant Guarantees

### 3.1. `SpellingBeePuzzle`
```rust
pub struct SpellingBeePuzzle {
    pub letters: [char; 7],
    pub main_letter: char,
    membership: [bool; 26],
}
```

#### Enforced Invariants:
1. **Fixed Array Representation**: Uses `[char; 7]` instead of a heap-allocated `Vec<char>`. This implements `Copy` and `Clone` with zero allocation overhead.
2. **Alphabet Validation**: All 7 characters must be uppercase ASCII `'A'..='Z'`.
3. **Strict Uniqueness**: No letter may appear more than once among the 7 elements.
4. **Mandatory Main Letter**: The main letter must belong to `letters`.
5. **Main Letter Ordering**: `letters[0] == main_letter` is enforced both for display alignment and for canonical bit-0 bitmask semantics.

### 3.2. Seven-Bit Bitmask Mechanics
Puzzle letter tracking uses bit manipulation:
- Bit 0 corresponds to `letters[0]` (the main letter).
- Bit $i$ ($0 \le i < 7$) corresponds to `letters[i]`.
- **`FULL_PANGRAM_MASK`**: `(1 << 7) - 1 = 0b111_1111` ($127$).
- **`MAIN_LETTER_BIT`**: `1 << 0 = 0b000_0001` ($1$).

#### Why Bitmasks Are Optimal:
- **Idempotence**: Encountering the same letter multiple times executes `mask |= 1 << i`, which is an idempotent $O(1)$ CPU register operation.
- **Pangram Check**: `mask == FULL_PANGRAM_MASK` checks all seven distinct letters in a single instruction.
- **Main Letter Check**: `(mask & 1) != 0` instantly confirms inclusion of the center letter.

---

## 4. Algorithms

### 4.1. Trie-Guided DFS Solution Enumeration
Instead of generating candidate strings or scanning the entire dictionary, the solver explores only paths that physically exist in the Trie:

1. Obtain a `TrieCursor` at `trie.root_cursor()`.
2. For each permitted letter $c \in \text{puzzle.letters}$:
   - Call `cursor.advance(c)`.
   - If `None`, prune the branch immediately.
   - If `Some(next_cursor)`, push $c$ onto the reusable `current_word: &mut String` and update the bitmask.
   - If `next_cursor.is_word()`, `current_word.len() >= 4`, and `(mask & 1) != 0`, insert into `solutions: HashSet<String>`.
   - Recursively explore deeper paths.
   - Backtrack by calling `current_word.pop()`.

### 4.2. Algorithmic Complexity:
- **Time Complexity**: $O(V_{\text{valid}})$, where $V_{\text{valid}}$ is the number of Trie nodes reachable via the 7 letters. Non-existent prefixes (e.g. `QX`, `ZJ`) abort in $O(1)$ time at the first absent edge.
- **Space Complexity**: $O(L_{\text{max}})$ call stack depth where $L_{\text{max}}$ is the length of the longest English word formed (typically $< 20$). Zero heap allocations occur during branch descent because the mutable word buffer is reused.

---

## 5. Scoring Policy and Rules

| Condition | Points Awarded | Rationale |
| :--- | :--- | :--- |
| Word length $< 4$ | 0 points | Invalid word length under Spelling Bee rules. |
| 4-letter word | 1 point | Standard baseline Spelling Bee reward. |
| 5+ letter word | 1 point per letter | Words of length $N$ award $N$ points (e.g., 6 letters = 6 points). |
| Pangram bonus | +7 additional points | Bonus awarded when all seven unique puzzle letters are utilized. |

---

## 6. Optional Hint System Design

### 6.1. Hint Grid: Length $\times$ Starting Letter Matrix
Implemented using `BTreeMap<char, BTreeMap<usize, usize>>`:
- Outer map keys (`char`) sort rows alphabetically.
- Inner map keys (`usize`) sort column word lengths numerically.
- Tracks row totals per letter, column totals per length, and the grand total matching `solutions.len()`.

### 6.2. Two-Letter Prefix Frequencies
Implemented using `BTreeMap<String, usize>`:
- Iterates over all precomputed solutions, extracting `word[..2]`.
- Preserves sorted order (e.g. `BA: 4, BE: 2, BI: 5`).
