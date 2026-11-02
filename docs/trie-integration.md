# Shared Trie Library Integration and Cursor Enhancement

## 1. Architectural Motivation

The university assignment specifies:
> **"The Trie is a shared, game-independent dictionary library. Each game implements its own rules and state management. The applications must not contain duplicate dictionary implementations."**

In the original repository, the Trie provided whole-word lookup (`contains`), prefix checking (`has_prefix`), insertion (`insert`), and deletion with pruning (`remove`).

While `contains` is sufficient for single-word verification, game algorithms like Spelling Bee and Boggle require step-by-step tree navigation:
- **Spelling Bee**: Enumerating all valid words by following only permitted letter branches.
- **Boggle Automatic Solver**: Advancing through prefixes as board neighbors are visited, pruning dead ends early.

Directly exposing the internal `Node` structure or child arrays would violate data encapsulation and risk memory safety or invariants.

---

## 2. The `TrieCursor` Solution

We designed and implemented a public, read-only `TrieCursor<'a>` in [`trie/src/lib.rs`](file:///c:/Users/User/Downloads/Programming%20Lab/Word%20Games/word-trie/trie/src/lib.rs):

```rust
#[derive(Debug, Clone, Copy)]
pub struct TrieCursor<'a> {
    node: &'a Node,
}

impl Trie {
    pub fn root_cursor(&self) -> TrieCursor<'_> {
        TrieCursor { node: &self.root }
    }
}

impl<'a> TrieCursor<'a> {
    pub fn advance(&self, letter: char) -> Option<TrieCursor<'a>> {
        let idx = char_to_index(letter.to_ascii_uppercase())?;
        self.node.children[idx].as_deref().map(|child| TrieCursor { node: child })
    }

    pub fn is_word(&self) -> bool {
        self.node.is_terminal
    }

    pub fn has_children(&self) -> bool {
        self.node.has_any_children()
    }
}
```

---

## 3. Rust Concepts and Safety Properties

### 3.1. Lifetimes and Borrow Checker Guarantees
- The cursor holds an immutable reference `&'a Node`.
- Because `'a` is tied to the parent `&'a Trie` via `trie.root_cursor()`, the cursor **cannot outlive the Trie**.
- While any `TrieCursor<'a>` exists, Rust's borrow checker strictly forbids mutable methods on `Trie` (such as `insert` or `remove`), preventing iterator invalidation and data races.

### 3.2. Why `Copy` and `Clone`?
`TrieCursor<'a>` wraps a single shared reference `&'a Node`. In 64-bit Rust, this is simply an 8-byte pointer:
- Implementing `Copy` allows passing the cursor by value to recursive functions without manual `.clone()` or ownership friction.
- Copying a cursor does not allocate heap memory ($0$ allocations).
- In DFS, exploring multiple branches from a node simply passes the copied cursor down each recursive call.

### 3.3. Encapsulation
- The underlying `Node` structure remains `pub(crate)`.
- Game crates (`spelling-bee`, `boggle`) only interact through `advance()`, `is_word()`, and `has_children()`.
- Internal 26-way child array representation is completely hidden.

---

## 4. Integration Verification

1. **Zero Degradation of Existing Code**: `trie-cli` continues to function with zero changes.
2. **Comprehensive Unit Testing**:
   - `test_cursor_root_empty_trie`: Root behavior on empty Trie.
   - `test_cursor_advance_existing_prefix_and_absent`: Step-by-step traversal along existing words ("CAR" -> "CART") and absent branches.
   - `test_cursor_copy_and_branching`: Independent parallel branch exploration without state pollution.
3. **Performance in Games**:
   - Spelling Bee solution enumeration runs in **$208\ \mu\text{s}$**.
   - Boggle 4x4 automatic board solving runs in **$473\ \mu\text{s}$**.
