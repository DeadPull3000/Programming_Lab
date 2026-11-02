# Known Limitations and Design Trade-Offs

This document provides a candid, evidence-based assessment of the known constraints, deliberate scope boundaries, memory trade-offs, and potential areas for future enhancement in the Trie dictionary implementation.

---

## 1. Character Set and Orthographic Constraints

- **Strict ASCII `A`–`Z` Alphabet**:
  - The implementation strictly adheres to the assignment specification: only the 26 ASCII uppercase letters are permitted.
  - Punctuation, apostrophes (e.g. `"don't"`, `"o'clock"`), hyphens (e.g. `"well-being"`), digits, spaces, and non-ASCII accented letters (e.g. `"café"`, `"naïve"`) are rejected with `TrieError::InvalidCharacter`.
  - While this perfectly matches Scrabble and standard tournament word list rules (TWL06 / NASPA), it is unsuitable for general-purpose natural language processing where contractions and loanwords are common.

- **Case Normalization**:
  - Input strings are normalized to uppercase prior to Trie insertion and queries.
  - As a result, the data structure cannot differentiate between common nouns and proper nouns that differ only in casing (e.g., `"march"` vs. `"March"`).

---

## 2. Memory Overhead of the 26-Way Array

- **Fixed Allocation per Node**:
  - Each `Node` struct contains `[Option<Box<Node>>; 26]`, which occupies 208 bytes of child pointers plus 8 bytes for `is_terminal` and alignment padding, totaling **216 bytes per node** on 64-bit systems.
  - In a sparse Trie, many nodes have only 1 or 2 active children, leaving 24–25 pointers set to `None`.
  - **Trade-off Justification**: This trade-off was explicitly chosen to guarantee direct $O(1)$ child indexing without hash map overhead, tree balancing, or binary searching.
  - **Potential Future Optimization**: For memory-constrained environments, a Compressed Radix Tree (Patricia Trie) or a Ternary Search Tree (TST) could reduce node count and pointer density at the cost of slight algorithmic complexity.

---

## 3. Deliberate Scope Boundaries

- **Word Games Excluded**:
  - In strict compliance with the assignment instructions, word games (Boggle solver, Spelling Bee, Wordle, Anagram finder) and graphical user interfaces were omitted. The focus remains exclusively on the Trie dictionary data structure and CLI operations.
- **Single-Threaded Mutability**:
  - The `Trie` struct uses standard Rust `&mut self` for mutation. While multiple threads can read from an immutable `&Trie` concurrently, concurrent multi-threaded writes would require explicit synchronization primitives (such as `RwLock` or a lock-free Trie).

---

## 4. Benchmark Frameworks and Micro-benchmarking

- **Loading Benchmark**:
  - The CLI and full-dictionary test report load times using `std::time::Instant`. The 176,413-word dictionary loads in ~134 milliseconds in release mode.
- **Micro-benchmarks**:
  - Dedicated micro-benchmarking suites (e.g., using `criterion`) are not currently incorporated into the workspace to keep dependencies strictly confined to the standard library.

---

## 5. File Parsing and Error Tolerance

- **Strict Validation on Mutation**:
  - When batch-loading words from a file, individual malformed tokens are recorded in `LoadReport::invalid_entries` and skipped, allowing valid words to still be inserted.
  - However, when calling `trie.insert("VALID INVALID")`, the single operation is strictly rejected upfront with `Err(TrieError::InvalidCharacter(' '))`, guaranteeing that the Trie is never partially modified by an invalid word.
