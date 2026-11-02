# Implementation Walkthrough: Codebase & Algorithms

This document serves as an exhaustive, step-by-step code walkthrough designed for technical review and Programming Lab oral examination (viva). It walks through all structures, algorithms, ownership decisions, and error-handling mechanisms in execution order.

---

## 1. Module and Architectural Layout

The project comprises two crates:
1. **`trie` (Library Crate)**:
   - Located at [`trie/src/lib.rs`](file:///c:/Users/User/Downloads/Programming%20Lab/Word%20Games/word-trie/trie/src/lib.rs).
   - Contains pure data structures, normalization logic, and trie traversal algorithms.
   - Contains 29 unit tests and 8 doctests.
   - Integration tests in [`trie/tests/differential_testing.rs`](file:///c:/Users/User/Downloads/Programming%20Lab/Word%20Games/word-trie/trie/tests/differential_testing.rs).
2. **`trie-cli` (Binary Crate)**:
   - Located at [`trie-cli/src/`](file:///c:/Users/User/Downloads/Programming%20Lab/Word%20Games/word-trie/trie-cli/src/).
   - `batch.rs`: File I/O, buffered reading, delimited string parsing, and batch candidate checking.
   - `main.rs`: Interactive REPL, command dispatching, terminal reporting, and full-dictionary integration tests.

---

## 2. Library Components (`trie/src/lib.rs`)

### 2.1 Constants and Error Types

```rust
pub const ALPHABET_SIZE: usize = 26;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TrieError {
    EmptyWord,
    InvalidCharacter(char),
}
```

- **`ALPHABET_SIZE`**: Fixed constant of type `usize`. Specifies array bounds throughout the library.
- **`TrieError`**: Custom algebraic enum capturing domain-specific failures:
  - `EmptyWord`: Occurs when an input is empty or contains exclusively whitespace.
  - `InvalidCharacter(char)`: Holds the exact offending non-ASCII-alphabetic character.
- **Traits implemented**:
  - `std::fmt::Display`: Provides user-friendly error formatting.
  - `std::error::Error`: Integrates with the standard Rust error ecosystem.
  - `PartialEq, Eq`: Enables direct assertions in unit tests (e.g. `assert_eq!(res, Err(TrieError::EmptyWord))`).

---

### 2.2 Input Normalization and Index Mapping

#### Function: `normalize_word`
```rust
pub fn normalize_word(input: &str) -> Result<String, TrieError> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Err(TrieError::EmptyWord);
    }

    let mut normalized = String::with_capacity(trimmed.len());
    for ch in trimmed.chars() {
        if ch.is_ascii_alphabetic() {
            normalized.push(ch.to_ascii_uppercase());
        } else {
            return Err(TrieError::InvalidCharacter(ch));
        }
    }

    Ok(normalized)
}
```
1. **Purpose**: Validates candidate words and normalizes ASCII letters to uppercase.
2. **Parameters**: `input: &str` (borrowed string slice).
3. **Return Value**: `Result<String, TrieError>`.
4. **Preconditions**: None. Safely handles empty strings, leading/trailing whitespace, and non-ASCII Unicode.
5. **Algorithm**:
   - Trim whitespace with `input.trim()`. If empty, immediately return `Err(TrieError::EmptyWord)`.
   - Allocate `String` pre-sized to `trimmed.len()` to avoid reallocations.
   - Iterate over characters: if `is_ascii_alphabetic()`, convert via `to_ascii_uppercase()` and push to result; otherwise abort immediately with `Err(TrieError::InvalidCharacter(ch))`.
6. **Edge Cases**: `"  \t\n  "` $\to$ `EmptyWord`; `"hello world"` $\to$ `InvalidCharacter(' ')`; `"café"` $\to$ `InvalidCharacter('é')`.
7. **Validated by**: `test_normalize_valid_words`, `test_normalize_empty_and_whitespace`, `test_normalize_invalid_characters`.

#### Helper: `char_to_index` and `index_to_char`
```rust
#[inline]
pub fn char_to_index(ch: char) -> Option<usize> {
    if ch.is_ascii_uppercase() {
        Some((ch as u8 - b'A') as usize)
    } else {
        None
    }
}

#[inline]
pub fn index_to_char(index: usize) -> Option<char> {
    if index < ALPHABET_SIZE {
        Some((b'A' + index as u8) as char)
    } else {
        None
    }
}
```
- Direct arithmetic `(ch as u8 - b'A')` converts ASCII byte values `'A'` (65) through `'Z'` (90) to `0..=25` in $O(1)$ time with zero branching in release builds.
- Annotated with `#[inline]` for zero-cost abstraction.
- Validated by `test_char_to_index_mapping`, `test_index_to_char_mapping`, and `test_round_trip_index_char`.

---

### 2.3 The Node Struct and Helpers

```rust
pub(crate) struct Node {
    pub(crate) children: [Option<Box<Node>>; ALPHABET_SIZE],
    pub(crate) is_terminal: bool,
}
```

#### Method: `Node::new()`
- Initializes 26 child links to `None` and `is_terminal` to `false`.
- Uses a `const NONE_CHILD: Option<Box<Node>> = None;` expression to populate the array without requiring `Clone` or runtime loops.

#### Method: `Node::is_prunable(&self) -> bool`
```rust
pub(crate) fn has_any_children(&self) -> bool {
    self.children.iter().any(|c| c.is_some())
}

pub(crate) fn is_prunable(&self) -> bool {
    !self.is_terminal && !self.has_any_children()
}
```
- A node is prunable if and only if it represents neither a complete word nor the ancestor of any complete word.

---

### 2.4 Trie Creation and Properties

```rust
pub struct Trie {
    pub(crate) root: Node,
    pub(crate) word_count: usize,
}

impl Trie {
    pub fn new() -> Self {
        Self {
            root: Node::new(),
            word_count: 0,
        }
    }

    pub fn len(&self) -> usize {
        self.word_count
    }

    pub fn is_empty(&self) -> bool {
        self.word_count == 0
    }
}
```
- Initialized with an empty root node and zero word count.
- `len()` and `is_empty()` query `self.word_count` in $O(1)$ time.

---

### 2.5 Insertion Algorithm: `Trie::insert`

```rust
pub fn insert(&mut self, word: &str) -> Result<bool, TrieError> {
    let normalized = normalize_word(word)?;
    let mut current = &mut self.root;

    for ch in normalized.chars() {
        let idx = char_to_index(ch).expect("normalized char must be valid A-Z");
        current = current.children[idx].get_or_insert_with(|| Box::new(Node::new()));
    }

    if current.is_terminal {
        Ok(false)
    } else {
        current.is_terminal = true;
        self.word_count += 1;
        Ok(true)
    }
}
```

1. **Borrowing & Mutability**: Takes `&mut self` because the tree structure and `word_count` are modified.
2. **Precondition & Transactional Safety**: `normalize_word(word)?` executes first. If the word contains invalid characters, the function early-returns `Err` without altering `self.root` or `self.word_count`.
3. **Traversal**:
   - `mut current` is a mutable reference to the current `Node`, starting at `&mut self.root`.
   - `get_or_insert_with(|| Box::new(Node::new()))`: If `children[idx]` is `None`, allocates a new boxed node on the heap. It then returns a `&mut Box<Node>`, which dereferences to `&mut Node`, rebinding `current` to the child.
4. **Termination**:
   - Checks `current.is_terminal`.
   - If already `true`: word is a duplicate. Returns `Ok(false)` without incrementing count.
   - If `false`: sets `current.is_terminal = true`, increments `self.word_count += 1`, returns `Ok(true)`.
5. **Validated by**: `test_insert_single_word`, `test_insert_shared_prefixes`, `test_insert_duplicate`, `test_insert_case_normalization`, `test_insert_invalid_input`.

---

### 2.6 Lookup and Prefix Checking: `contains` and `has_prefix`

```rust
fn find_node(&self, text: &str) -> Option<&Node> {
    let normalized = normalize_word(text).ok()?;
    let mut current = &self.root;

    for ch in normalized.chars() {
        let idx = char_to_index(ch)?;
        current = current.children[idx].as_deref()?;
    }

    Some(current)
}

pub fn contains(&self, word: &str) -> bool {
    self.find_node(word).is_some_and(|node| node.is_terminal)
}

pub fn has_prefix(&self, prefix: &str) -> bool {
    self.find_node(prefix).is_some()
}
```

1. **Borrowing**: Takes `&self` (immutable shared borrow). Multiple concurrent threads can safely query the Trie simultaneously.
2. **`find_node` Helper**:
   - Normalizes text; if invalid, returns `None`.
   - Iterates through characters; uses `.as_deref()?` on `Option<Box<Node>>` to borrow the inner `&Node`.
   - If any character child is absent (`None`), returns `None` immediately.
3. **`contains` vs. `has_prefix`**:
   - `contains` verifies both that the path exists and that `node.is_terminal == true`.
   - `has_prefix` only checks that the path exists (`self.find_node(prefix).is_some()`).
4. **Validated by**: `test_lookup_contains_existing_and_missing`, `test_has_prefix`, `test_lookup_empty_and_invalid_returns_false`.

---

### 2.7 Deletion and Pruning: `remove` and `remove_words`

```rust
pub fn remove(&mut self, word: &str) -> Result<bool, TrieError> {
    let normalized = normalize_word(word)?;
    let removed = Node::remove_recursive(&mut self.root, normalized.as_bytes(), 0);
    if removed {
        self.word_count -= 1;
        Ok(true)
    } else {
        Ok(false)
    }
}
```

#### The Recursive Helper: `Node::remove_recursive`
```rust
pub(crate) fn remove_recursive(node: &mut Node, bytes: &[u8], depth: usize) -> bool {
    if depth == bytes.len() {
        if node.is_terminal {
            node.is_terminal = false;
            true
        } else {
            false
        }
    } else {
        let idx = (bytes[depth] - b'A') as usize;
        if let Some(child) = node.children[idx].as_deref_mut() {
            let removed = Self::remove_recursive(child, bytes, depth + 1);
            if removed && node.children[idx].as_ref().unwrap().is_prunable() {
                node.children[idx] = None;
            }
            removed
        } else {
            false
        }
    }
}
```

1. **Algorithm Walkthrough**:
   - **Base Case (`depth == bytes.len()`)**:
     We have reached the target node.
     - If `node.is_terminal == true`: Clear `node.is_terminal = false` and return `true`.
     - If `node.is_terminal == false`: Word was not present (only an incomplete prefix). Return `false`.
   - **Recursive Step (`depth < bytes.len()`)**:
     - Compute child index `idx = (bytes[depth] - b'A') as usize`.
     - If `children[idx]` is `None`, path does not exist; return `false`.
     - Recurse into child with `depth + 1`.
     - When recursion returns:
       If `removed == true`, check `node.children[idx].as_ref().unwrap().is_prunable()`.
       If `is_prunable()` is true, set `node.children[idx] = None`. This triggers Rust's automatic memory deallocation of the `Box<Node>`.
     - Return `removed`.
2. **Preservation of Root**: Pruning only acts on `node.children[idx] = None`. The root node itself is never pruned.
3. **Preservation of Shared Prefixes**:
   - If an ancestor has other children, `is_prunable()` evaluates to `false`, halting deallocation.
   - If an ancestor represents another complete word, `is_terminal` is `true`, so `is_prunable()` is `false`, halting deallocation.
4. **Batch Deletion (`remove_words`)**:
```rust
pub fn remove_words<I, S>(&mut self, words: I) -> Vec<Result<bool, TrieError>>
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    words.into_iter().map(|w| self.remove(w.as_ref())).collect()
}
```
   - Uses generic trait bounds `I: IntoIterator<Item = S>, S: AsRef<str>`.
   - Accepts arrays (`["a", "b"]`), slices (`&["a", "b"]`), and vectors (`vec![String::from("a")]`).
5. **Validated by**: `test_remove_leaf_word_and_pruning`, `test_remove_prefix_word_preserves_longer_word`, `test_remove_nonexistent_and_prefix_only`, `test_remove_repeated`, `test_remove_until_empty`, `test_remove_words_batch`.

---

## 3. CLI and Batch Components (`trie-cli/src/`)

### 3.1 Token Parsing: `parse_tokens`
```rust
pub fn parse_tokens(text: &str) -> Vec<&str> {
    text.split(|c: char| c == ',' || c.is_whitespace())
        .filter(|token| !token.is_empty())
        .collect()
}
```
- Splits input text on commas OR any whitespace character (`' '`, `\t`, `\n`, `\r`).
- Filters out empty tokens, safely handling repeated delimiters (e.g. `",,  ,"`).

### 3.2 Buffered File Loading: `load_words_from_reader`
- Takes any type implementing `std::io::BufRead`.
- Iterates with `.lines()`, maintaining an exact 1-based line counter `line_num`.
- Accumulates a `LoadReport`:
  - `total_tokens`: Total non-empty tokens parsed.
  - `newly_inserted`: Distinct words added to Trie.
  - `duplicates`: Words already present.
  - `invalid_entries`: Vector of `(line_num, token, TrieError)`.

### 3.3 Candidate Classification: `check_words_from_reader`
- Reads tokens from candidate file.
- Categorizes tokens into:
  - `found`: Valid words present in Trie.
  - `missing`: Valid words absent from Trie.
  - `invalid`: Malformed tokens that failed normalization.

### 3.4 Interactive REPL: `process_command` and `main`
- Decoupled from standard I/O:
  `pub fn process_command<W: Write>(trie: &mut Trie, line: &str, writer: &mut W) -> io::Result<bool>`
- Dispatches commands: `help`, `find`, `insert`, `delete`, `found`, `missing`, `load`, `check-file`, `stats`, `quit`.
- Returning `Ok(false)` signals clean exit to the outer REPL loop.
- Error resilience: Invalid commands or bad files report friendly messages and allow the user to continue without terminating or panicking.
