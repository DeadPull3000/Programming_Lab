# 10. Technical Viva Voce Examination Guide

This guide prepares the developer for rigorous academic defense of the Rust Word Games project, offering comprehensive questions, technically authoritative answers, and defense strategies grounded directly in the codebase.

---

## 1. Core Trie & Data Structure Questions

### Q1: What is a Trie, and why use it instead of a `HashSet<String>`?
**Answer:** A Trie (prefix tree) is an $m$-ary search tree where keys are sequences of characters. Edges represent character transitions, and nodes represent prefixes. We chose a Trie over a `HashSet` because:
1. **Prefix Pruning:** A `HashSet` cannot answer *"does any word in the dictionary start with prefix P?"* without scanning every single key in $O(N)$ time. A Trie answers this in $O(L)$ time.
2. **Solver Efficiency:** Both Spelling Bee and Boggle solvers prune millions of invalid word paths early. For instance, if no word begins with `"QZ"`, the Trie halts DFS immediately, saving millions of recursive steps.
3. **Shared Memory:** Common prefixes (e.g. `"CAR"`, `"CART"`, `"CARE"`) share the same sequence of nodes, avoiding redundant character storage.

### Q2: Why is the `is_terminal` boolean flag necessary?
**Answer:** Because words in a dictionary can be proper prefixes of other words (e.g., `"CAR"` and `"CART"`). Without a terminal flag, there is no way to determine whether an intermediate node along an active path constitutes an actual standalone word or merely a prefix of a longer word.

### Q3: Why are child pointers stored as `[Option<Box<Node>>; 26]`?
**Answer:**
1. **Fixed Alphabet Indexing:** English letters `A`–`Z` map directly to indices $0$ to $25$ via `(byte - b'A') as usize`. This yields instantaneous $O(1)$ child branch resolution without hash collisions.
2. **Recursive Sizing (`Box`):** A Rust struct cannot contain an unboxed instance of itself because its size would be infinite at compile time. `Box<Node>` introduces a fixed-size 8-byte heap pointer, giving `Node` a known, finite size.
3. **`Option` Semantics:** `None` indicates an unallocated branch, consuming 0 heap allocations for absent subtrees.

### Q4: How does deletion work, and why is recursive pruning necessary?
**Answer:** Deletion requires a two-step post-order traversal:
1. First, navigate to the target word's terminal node and set `is_terminal = false`.
2. Second, unwind the call stack to prune dangling nodes. A node is prunable if and only if `!node.is_terminal && !node.has_any_children()`. If true, the parent sets `children[idx] = None`, which instantly drops the `Box<Node>` and reclaims heap memory.
3. **Shared Prefix Invariant:** If the node is terminal for another word (e.g. deleting `"CART"` when `"CAR"` exists) or has other active children, it is **not** pruned, preserving all other dictionary paths.

### Q5: What does `TrieCursor<'a>` achieve, and how do lifetimes protect it?
**Answer:** `TrieCursor` enables zero-allocation, step-by-step traversal down the Trie. It stores `Option<&'a Node>`. Because it implements `Copy`, branching during DFS copies an 8-byte pointer without heap allocation.
The lifetime `'a` binds the cursor to `&'a Trie`. Rust's borrow checker statically guarantees that no client can mutate the Trie while a cursor is alive, and that a cursor can never become a dangling pointer by outliving the Trie. Furthermore, the internal `Node` struct remains `pub(crate)`, preserving strict data encapsulation.

---

## 2. Spelling Bee Questions

### Q6: How does sampling without replacement guarantee seven distinct letters?
**Answer:** In `SpellingBeePuzzle::random()`, we populate an array with all 26 uppercase alphabet characters, shuffle it using the Fisher-Yates algorithm (`rand::seq::SliceRandom::shuffle`), and take the first 7 elements. Because each item in the slice is unique and selected without replacement, the resulting 7 letters are mathematically guaranteed to be pairwise distinct.

### Q7: How does the 7-bit bitmask work for letter validation and pangrams?
**Answer:** Each letter of the 7-letter puzzle corresponds to a unique bit in an 8-bit integer (`u8`):
- Center letter: Bit 0 (`0x01` or `1 << 0`).
- Outer letters: Bits 1 through 6 (`1 << 1` through `1 << 6`).
- **Pangram Mask:** `0b01111111` (`0x7F`).
- **Operation:** As letters are encountered, we update `mask |= (1 << letter_index)`. Setting a bit is idempotent, meaning letter repetition does not alter an already set bit.
- **Pangram Check:** `mask == 0x7F` executes in 1 CPU instruction.
- **Mandatory Center Letter:** `(mask & 0x01) != 0` executes in 1 CPU instruction.

### Q8: Why use Trie-guided DFS instead of scanning the dictionary?
**Answer:** A brute-force dictionary scan checks all 176,413 words in $O(N \cdot L)$ time, taking **$42.44\text{ ms}$**.
In contrast, Trie-guided DFS begins at `trie.root_cursor()` and explores *only* branches matching the 7 allowed letters. Any non-existent branch is pruned in $O(1)$ time. This solves the entire puzzle in **$0.208\text{ ms}$**, an empirical **$\sim 204\times$ speedup**!

### Q9: Why store precomputed solutions in a `HashSet<String>`?
**Answer:** During interactive gameplay, players submit guesses one at a time. Checking whether a guess is in a precomputed `HashSet` takes $O(1)$ average time ($O(L)$ to hash the word). It also provides instant set difference calculations to compute `missed_words()` upon quitting.

---

## 3. Boggle Questions

### Q10: How is the board represented, and why use a flat 1D vector?
**Answer:** The board is stored as `cells: Vec<char>` of length $size^2$, mapped via:
$$\text{index} = \text{row} \times \text{size} + \text{column}$$
We chose a flat vector over `Vec<Vec<char>>` because:
1. It guarantees contiguous cache locality, eliminating pointer indirection.
2. It requires exactly 1 heap allocation instead of $size + 1$ allocations.
3. Coordinate conversions use simple arithmetic which LLVM optimizes into fast bit operations.

### Q11: How do you prevent physical cell reuse while allowing letter repetition?
**Answer:** We maintain a boolean array `visited: Vec<bool>` of size $size^2$.
- When a cell at index $i$ is stepped into, we set `visited[i] = true`.
- An adjacent cell is valid only if `!visited[neighbor_idx]`.
- If the letter `'A'` appears in Cell 0 and Cell 5, both can be visited because they have distinct indices ($0 \ne 5$). However, Cell 0 cannot be visited twice in the same word.
- **Backtracking Invariant:** When the search returns (whether succeeding or failing), `visited[cell_idx] = false` is restored so that alternative candidate paths can use the cell.

### Q12: Why is dictionary membership insufficient to validate a Boggle guess?
**Answer:** Boggle requires two independent conditions:
1. **Physical Adjacency:** The letters must form a continuous 8-directional path on the grid without cell reuse.
2. **Lexical Validity:** The formed string must exist in the dictionary.
A word might be a valid English word (e.g. `"ROAST"`) but impossible to form on the board, or it might be formable on the board (e.g. `"XYZQ"`) but not be a recognized word. Both checks must pass.

### Q13: How does the automatic Boggle solver work?
**Answer:** In `validator::solve_boggle_board`, we launch a Trie-guided DFS starting at every cell $(0..size^2)$. At each step, we advance `trie.root_cursor()`. If the adjacent letter does not advance the cursor, the branch is dead and pruned immediately. This extracts all valid board words in **$< 1.2\text{ ms}$**.

---

## 4. Rust Language & Paradigm Questions

### Q14: What is the difference between `String` and `&str`?
**Answer:** `String` is an owned, heap-allocated, growable buffer of UTF-8 bytes (consisting of a pointer, length, and capacity on the stack). `&str` is an immutable borrowed string slice (consisting of a pointer and length) referencing UTF-8 data stored elsewhere (in binary rodata, on the stack, or inside a `String`). We accept `&str` in lookup functions for flexibility, and store `String` in game states where persistent ownership is required.

### Q15: What is the difference between `&self` and `&mut self`?
**Answer:** `&self` borrows the struct immutably, allowing concurrent readers but forbidding modification (e.g., `Trie::contains`, `Board::neighbors_of`). `&mut self` grants exclusive mutable access, allowing in-place state modification (e.g., `Trie::insert`, `SpellingBeeGame::guess`). Rust enforces at compile time that no immutable references can coexist with a mutable reference, eliminating data races.

---

## 5. Challenging Defense Questions & Cross-Examinations

### Q16: What happens if two dictionary words share a prefix, like "CAR" and "CART"?
**Defense:** In the Trie, both words share the exact same nodes for `'C'`, `'A'`, and `'R'`. The node for `'R'` has `is_terminal = true` (representing `"CAR"`), and also has a non-empty child at index 19 pointing to `'T'`, which also has `is_terminal = true` (representing `"CART"`). Both are recognized independently.

### Q17: What happens if a player guesses the same word twice in Boggle or Spelling Bee?
**Defense:** Both engines track submitted guesses in a dedicated set (`BTreeSet<String>` in Spelling Bee, duplicate detection loop in Boggle). The second submission returns `AlreadyGuessed` / `Duplicate` and awards 0 points.

### Q18: What if a Boggle board contains multiple different paths for the same word?
**Defense:** During single-word validation (`can_form_word`), DFS terminates and returns `true` as soon as the *first* valid path is discovered. During batch scoring, guesses are deduplicated so the word is scored exactly once regardless of how many paths exist on the board.
