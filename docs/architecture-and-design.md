# Architecture and Design: 26-Way Trie Dictionary

This document details the internal architectural design, data structures, memory layout, ownership semantics, deletion algorithms, and computational complexity of the Trie dictionary implementation.

---

## 1. What is a Trie and Why Use It for Dictionaries?

A **Trie** (derived from re**trie**val, pronounced "try") is a multi-way tree structure designed for rapid retrieval of strings over a finite alphabet $\Sigma$.

In a conventional flat array or balanced binary search tree (such as `std::collections::BTreeSet`), every string is stored in full. If a dictionary contains `"CAR"`, `"CART"`, `"CARE"`, and `"CARD"`, the prefix `"CAR"` is redundantly duplicated in memory 4 times. Furthermore, searching a balanced BST requires $O(L \cdot \log N)$ string comparisons, where each comparison may compare up to $L$ characters.

A Trie solves this by:
1. **Factoring common prefixes**: All words starting with `"CAR"` share exactly the same three nodes for `'C'`, `'A'`, and `'R'`.
2. **Predictable $O(L)$ lookups**: Lookups depend strictly on the length of the query string $L$, completely independent of the total dictionary size $N$ (whether 10 words or 176,000 words).
3. **Instant prefix existence testing**: Determining if *any* word starts with a prefix `"CAR"` takes $O(L)$ time by following edges down to `'R'`.

---

## 2. Core Data Structures and Memory Layout

The core library in [`trie/src/lib.rs`](file:///c:/Users/User/Downloads/Programming%20Lab/Word%20Games/word-trie/trie/src/lib.rs) defines two primary structs:

```rust
pub const ALPHABET_SIZE: usize = 26;

pub(crate) struct Node {
    pub(crate) children: [Option<Box<Node>>; ALPHABET_SIZE],
    pub(crate) is_terminal: bool,
}

pub struct Trie {
    pub(crate) root: Node,
    pub(crate) word_count: usize,
}
```

### 2.1 The Role of the Root Node
- The `root` node represents the empty string prefix `""`.
- The root is never assigned an alphabet character.
- The root's `is_terminal` flag is always `false` (since empty words are prohibited).
- The root is never removed or deallocated during deletion operations.

### 2.2 The 26-Way Child Array
- The alphabet $\Sigma$ is restricted to English letters `A`–`Z`.
- Each node contains a fixed-size array of 26 slots, indexed by `(ch as u8 - b'A') as usize`.
- Character `'A'` maps directly to index `0`.
- Character `'Z'` maps directly to index `25`.
- **Why an array rather than a `HashMap` or `Vec`?**
  Direct indexing `children[idx]` compiles to a single memory offset instruction `base + (idx * pointer_size)`. It requires no hashing overhead, no collision resolution, and no dynamic array reallocation.

### 2.3 Memory Layout: `Option<Box<Node>>`
- **Why `Box`?**
  In Rust, every struct must have a known, fixed size at compile time to be placed on the stack or inline inside another type. A struct that directly contains itself (`struct Node { children: [Option<Node>; 26] }`) would be infinite in size. `Box<Node>` introduces heap indirection, replacing the unknown recursive size with an 8-byte pointer (on 64-bit systems).
- **Why `Option`?**
  Rust guarantees that an absent pointer is safely represented by `Option::None`. Thanks to Rust's **Null Pointer Optimization (NPO)**, an `Option<Box<T>>` has the exact same size as a raw pointer (8 bytes). `None` is physically represented as `0x0`, and `Some(ptr)` is the non-zero heap memory address.
- **Node Size in Memory**:
  - `children`: 26 pointers $\times$ 8 bytes = 208 bytes.
  - `is_terminal`: 1 byte bool + 7 bytes alignment padding = 8 bytes.
  - **Total per node**: 216 bytes on 64-bit systems.

### 2.4 The `is_terminal` Marker
A path in the Trie can exist solely to support a longer word. For example, inserting `"CARPET"` creates nodes along the path `C-A-R-P-E-T`. At this stage:
- The path `C-A-R` exists.
- However, `"CAR"` is **not** a word in the dictionary unless its node at `'R'` has `is_terminal == true`.
- Therefore, `contains("CAR")` checks `node.is_terminal`, returning `false`.
- Meanwhile, `has_prefix("CAR")` only checks if the node exists, returning `true`.

### 2.5 Word Counting and Invariants
- `word_count: usize` maintains the exact number of distinct complete words stored in the Trie.
- **Invariants**:
  1. `word_count == 0` if and only if `is_empty() == true`.
  2. Inserting an existing word returns `Ok(false)` and leaves `word_count` unchanged.
  3. Deleting an existing word returns `Ok(true)` and decrements `word_count` by 1.
  4. Attempting to delete a word that is absent (or only a prefix) returns `Ok(false)` and leaves `word_count` unchanged.

---

## 3. Conceptual Diagrams

*(Note: The following diagrams are conceptual architectural illustrations to facilitate understanding of tree topologies and pruning).*

### Diagram 1: Empty Trie
```text
(Root: is_terminal=false, count=0)
 [0..25] -> all None
```

---

### Diagram 2: Several Words Sharing Prefixes (`CAR`, `CARD`, `CART`, `CARE`)
```text
                  (Root)
                    │
                   [C]
                    │
                   [A]
                    │
                   [R] (is_terminal = true)  --> Complete Word: "CAR"
             ┌──────┼──────┐
            [D]*   [E]*   [T]*               --> Complete Words: "CARD", "CARE", "CART"
```
*Nodes marked with `*` indicate `is_terminal = true`.*

---

### Diagram 3: Prefix Path vs. Complete Stored Word
Suppose only `"CATALOG"` is stored:
```text
(Root) -> [C] -> [A] -> [T] -> [A] -> [L] -> [O] -> [G]*
```
- Query `contains("CAT")`:
  - Path `C-A-T` exists in tree.
  - Node `[T]` has `is_terminal = false`.
  - **Result**: `false` (prefix only, not a complete stored word).
- Query `has_prefix("CAT")`:
  - Path `C-A-T` exists.
  - **Result**: `true`.

---

### Diagram 4: Deleting a Prefix Word while Preserving Longer Words
Suppose `"CAR"` and `"CARD"` are both stored:
```text
Before removing "CAR":
(Root) -> [C] -> [A] -> [R]* -> [D]*     ("CAR" & "CARD" exist)

Action: trie.remove("CAR")
1. Follow path to [R].
2. Set [R].is_terminal = false.
3. Check if [R] can be pruned:
   - [R].is_terminal == false.
   - BUT [R] still has child [D]!
   - Pruning stops. [R] is retained.

After removing "CAR":
(Root) -> [C] -> [A] -> [R]  -> [D]*     ("CAR" is gone; "CARD" remains intact!)
```

---

### Diagram 5: Pruning a Childless Branch
Suppose `"CAR"` and `"CARD"` are stored, and we remove `"CARD"`:
```text
Before removing "CARD":
(Root) -> [C] -> [A] -> [R]* -> [D]*

Action: trie.remove("CARD")
1. Recurse down to [D].
2. Set [D].is_terminal = false.
3. [D] is not terminal and has NO children:
   - Prunable! [D] is deallocated (children[3] becomes None).
4. Recursion unwinds back to [R].
5. Check if [R] is prunable:
   - [R].is_terminal == true (because "CAR" is stored).
   - NOT prunable! Pruning stops.

After removing "CARD":
(Root) -> [C] -> [A] -> [R]*             (Node [D] freed; "CAR" preserved!)
```

---

## 4. Design Decisions, Alternatives, and Trade-Offs

| Decision | Choice Made in Implementation | Alternative Approaches | Justification & Observed Trade-offs | Status in Code |
|---|---|---|---|---|
| **Child Link Storage** | Fixed-size array `[Option<Box<Node>>; 26]` | `HashMap<char, Box<Node>>`, `Vec<(char, Box<Node>)>`, Radix Tree / Ternary Tree | Array indexing requires zero hashing calculations and direct offset jumps. Trade-off: Each node consumes 216 bytes regardless of child density. | **Observed in source** (`trie/src/lib.rs:45`) |
| **Heap Indirection** | `Option<Box<Node>>` | Raw pointers with manual `free()`, arena allocators (`bumpalo`), arena index vectors | Idiomatic safe Rust. Leverages RAII so dropping a `None` or dropping a `Node` automatically drops its boxed children recursively with no leaks and no `unsafe`. | **Observed in source** (`trie/src/lib.rs:46`) |
| **Input Normalization** | Strict upfront trimming & uppercase conversion (`A`–`Z`) | In-place lowercase conversion, case-sensitive storage, accepting Unicode punctuation | Matches Scrabble/dictionary rules. Rejecting invalid characters upfront guarantees transactional safety (trie never mutated on invalid input). | **Observed in source** (`trie/src/lib.rs:108`) |
| **Encapsulation** | `pub(crate)` internal fields; small public API on `Trie` | Public node fields, exposing pointer traversals | Prevents external callers from corrupting `word_count` or breaking pruning invariants. | **Observed in source** (`trie/src/lib.rs:44`) |
| **Crate Separation** | Library crate `trie` separated from CLI crate `trie-cli` | Monolithic single binary, embedding CLI inside library | Clean function-level abstraction. The `trie` library contains pure algorithms with zero filesystem dependencies. `trie-cli` handles I/O and user interaction. | **Observed in Cargo workspace** |
| **Deletion Strategy** | Post-order recursive descent with pruning predicate | Iterative two-pass with parent pointers / vector trail | Recursive unwinding allows ancestor inspection after terminal unmarking in a single pass without parent pointer overhead or unsafe cycles. | **Observed in source** (`trie/src/lib.rs:74`) |

---

## 5. Algorithmic Complexity

Let:
- $L$ = Length of the input word (number of characters).
- $|\Sigma|$ = Alphabet size (constant 26).
- $N$ = Number of distinct words currently in the Trie.

### 5.1 Insertion: `Trie::insert`
- **Time Complexity**: $O(L)$
  - Upfront normalization: $O(L)$ to scan and convert characters.
  - Tree traversal: Exactly $L$ steps. Each step computes an array index `(ch as u8 - b'A')` in $O(1)$ and either follows a pointer or allocates a node.
  - Total time: $O(L)$. Completely independent of $N$.
- **Space Complexity**: $O(L)$ worst-case
  - If the word shares no prefixes, up to $L$ nodes are allocated.
  - If the word is already present or shares existing nodes, $O(1)$ additional space.

### 5.2 Lookup: `Trie::contains` and `Trie::has_prefix`
- **Time Complexity**: $O(L)$
  - Exactly $L$ pointer dereferences in the worst case.
  - Terminates early in $O(k)$ ($k < L$) as soon as any child is `None`.
  - Zero heap allocations during lookup.
- **Space Complexity**: $O(1)$ auxiliary space.

### 5.3 Deletion: `Trie::remove`
- **Time Complexity**: $O(L)$
  - Recursion descends to depth $L$.
  - Post-order stack unwinding visits $L$ nodes, checking `is_prunable()` in $O(|\Sigma|) = O(26) = O(1)$ time.
  - Total time: $O(L)$.
- **Space Complexity**: $O(L)$ call-stack frames during recursion.

### 5.4 Word Count: `Trie::len` and `Trie::is_empty`
- **Time Complexity**: $O(1)$ constant time (returns cached `word_count`).
- **Space Complexity**: $O(1)$.

### 5.5 Dictionary Loading: `load_words_from_file`
- **Time Complexity**: $O(W \cdot \bar{L})$, where $W$ is the number of tokens in the file and $\bar{L}$ is the average word length.
- **Measured Performance**: Loading the full 176,413-word dictionary completes in **~134 milliseconds** in release mode.
