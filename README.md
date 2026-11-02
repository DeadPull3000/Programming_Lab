# Word Games in Rust: Programming Lab Suite

An industrial-grade, memory-safe, and high-performance dictionary and word-puzzle game ecosystem in Rust, developed for an academic Programming Lab submission.

The suite comprises three interconnected components sharing a unified, game-independent 26-way prefix tree (Trie) dictionary engine:
1. **Encapsulated 26-ary Trie Dictionary Library & Interactive CLI** (`1 - Library Trie/`)
2. **NYT-Style Spelling Bee Game Engine & CLI** (`2 - Spelling Bee/`)
3. **Multi-Dimension Boggle Game Engine, Solver & CLI** (`3 - Boggle/`)

---

## 1. Project Purpose & Problem Statement

Modern word puzzle applications require high-throughput lexical validation and dynamic word discovery. Searching large dictionaries (over 100,000 words) using linear scans or unindexed lookups is computationally prohibitive.

This project solves this challenge by developing a custom 26-ary prefix tree (Trie) in Rust. The Trie provides $O(L)$ exact lookups, prefix searches, and recursive node pruning, while exposing an innovative, zero-allocation `TrieCursor` traversal API. Two complete interactive games—**Spelling Bee** and **Boggle**—are constructed on top of this shared library, utilizing Trie-guided Depth-First Search (DFS) algorithms to achieve sub-millisecond solution enumeration.

---

## 2. Workspace Architecture

The workspace is organized as a Cargo virtual workspace:

```text
Word Games/
├── Cargo.toml                    # Root workspace manifest linking all packages
├── Cargo.lock                    # Dependency lockfile
├── README.md                     # Master laboratory guide (this file)
├── dictionary.txt                # Root fallback dictionary file (176,413 words)
│
├── 1 - Library Trie/             # Part 1: Core Dictionary Data Structure & CLI
│   ├── README.md                 # Module documentation & CLI instructions
│   ├── trie/                     # Encapsulated 26-ary Trie library with public TrieCursor
│   │   ├── Cargo.toml
│   │   ├── src/lib.rs            # Core Trie, Node, and TrieCursor implementations
│   │   └── tests/                # Randomized differential testing against BTreeSet
│   └── trie-cli/                 # Interactive REPL & batch file classifier
│       ├── Cargo.toml
│       └── src/ (main.rs, batch.rs)
│
├── 2 - Spelling Bee/             # Part 2: NYT-style Spelling Bee Application
│   ├── Cargo.toml
│   ├── README.md                 # Architecture, algorithm notes, and commands
│   ├── src/                      # Game engine, solver, bitmask puzzle, scoring, hints
│   └── tests/                    # Integration & solver benchmark tests
│
├── 3 - Boggle/                   # Part 3: Boggle Application (4x4, 5x5, 6x6)
│   ├── Cargo.toml
│   ├── README.md                 # Grid mechanics, DFS cell reuse, Qu tile, batch evaluation
│   ├── src/                      # Board, DFS pathfinder, automatic solver, scoring, timer
│   └── tests/                    # Integration, Qu tile, and batch round simulation tests
│
├── Resources/                    # Assignment PDFs and Lexicons
│   ├── README.md
│   ├── Word Games.pdf            # Assignment specification handout
│   ├── dictionary.pdf            # Original dictionary document
│   └── dictionary.txt            # Preprocessed newline-delimited word list
│
└── docs/                         # Technical Documentation Suite (10 Reports)
    ├── 01-project-overview.md
    ├── 02-system-architecture.md
    ├── 03-trie-implementation.md
    ├── 04-spelling-bee-implementation.md
    ├── 05-boggle-implementation.md
    ├── 06-testing-and-validation.md
    ├── 07-terminal-evidence.md
    ├── 08-design-decisions-and-tradeoffs.md
    ├── 09-known-limitations.md
    └── 10-viva-preparation.md
```

---

## 3. How the Games Reuse the Trie

Both games depend directly on the `trie` library (`trie = { path = "../1 - Library Trie/trie" }`):
- **Encapsulation via `TrieCursor`:** The Trie's internal nodes (`Node`) remain strictly private (`pub(crate)`). Client crates interact with tree branches exclusively through `TrieCursor<'a>`, an immutable, zero-allocation pointer wrapper.
- **Spelling Bee Solution Solver:** Solves the entire puzzle in **$208\ \mu\text{s}$** by recursively advancing `TrieCursor` along only the 7 allowed hive letters, instantly pruning missing subtrees.
- **Boggle Path Validation & Board Solver:** Evaluates player guesses against the Trie, and powers an automatic solver that extracts all valid board words across 4x4, 5x5, or 6x6 boards in **$< 1.2\text{ ms}$**.

---

## 4. Prerequisites & Dependencies

- **Rust Toolchain:** Rust `1.85.0` or higher (verified on `rustc 1.99.0` and `cargo 1.99.0`).
- **Standard Library Purity:** The core `trie` library has **zero external dependencies**.
- **Downstream Dependencies:**
  - `rand = "0.8.8"` (used by `spelling-bee` and `boggle` for random puzzle/board shuffling).
- **Lexicon Requirements:** A plain text file containing newline-delimited Latin uppercase words `A`–`Z` (`Resources/dictionary.txt`).

---

## 5. Build, Test & Quality Commands

Execute all commands from the workspace root (`c:\Users\User\Downloads\Programming Lab\Word Games`):

```powershell
# 1. Run all 101 unit, doc, and integration tests across the workspace
cargo test --workspace

# 2. Run static analysis (Clippy) with warnings treated as errors
cargo clippy --workspace --all-targets -- -D warnings

# 3. Verify standard code formatting
cargo fmt --all -- --check

# 4. Compile optimized release binaries
cargo build --release --workspace

# 5. Generate local HTML documentation
cargo doc --workspace --no-deps
```

---

## 6. Running the Applications

### A. Trie CLI (`trie-cli`)
```powershell
cargo run -p trie-cli
```
Interactive commands:
- `find <word>`: Check if a word exists in the dictionary.
- `insert <word>`: Insert a custom word into the Trie.
- `delete <word>`: Delete a word, recursively pruning dangling nodes.
- `load <file>`: Ingest words from a newline- or comma-delimited file.
- `check <file>`: Partition words into `Found` and `Missing` lists.
- `stats`: Display total active word count.
- `quit`: Exit the REPL.

### B. Spelling Bee (`spelling-bee`)
```powershell
cargo run -p spelling-bee
```
Gameplay features:
- Procedural generation of 7 distinct letters with a mandatory center letter.
- Enter words directly; valid guesses score points (4 letters = 1 pt; $\ge 5$ letters = length; pangram = $+7$ bonus).
- Special commands:
  - `:hint`: View word length breakdown and 2-letter prefix frequency matrix.
  - `:list`: View discovered words formatted in alphabetical order across 3 columns.
  - `:shuffle`: Shuffle outer honeycomb letters.
  - `:quit`: Surrender and display all missed words sorted by length.

### C. Boggle (`boggle`)
```powershell
cargo run -p boggle
```
Gameplay features:
- Choose grid dimensions: `1` for 4x4 (Classic), `2` for 5x5 (Big Boggle), `3` for 6x6 (Super Big).
- Enable/disable optional round countdown timer.
- Enable/disable optional `"Qu"` special tile expansion.
- Enter batch guesses in a single line separated by commas or whitespace (e.g. `CART, TEAMS, ROAST, BOGGLE`).
- Type `DONE` to submit the round for irrevocable scoring.
- View automatic board solver output revealing all missed words.

---

## 7. Technical Documentation Suite

The complete technical report suite is located in the [`docs/`](file:///c:/Users/User/Downloads/Programming%20Lab/Word%20Games/docs) directory:

1. [**01. Project Overview & Requirements Traceability**](file:///c:/Users/User/Downloads/Programming%20Lab/Word%20Games/docs/01-project-overview.md): Scope, educational objectives, and 3 verification matrices.
2. [**02. System Architecture & Component Interaction**](file:///c:/Users/User/Downloads/Programming%20Lab/Word%20Games/docs/02-system-architecture.md): Diagrams, crate dependencies, file inventory, and execution sequences.
3. [**03. Trie Implementation & Cursor Mechanics**](file:///c:/Users/User/Downloads/Programming%20Lab/Word%20Games/docs/03-trie-implementation.md): Node structures, recursive pruning, zero-allocation `TrieCursor`, and algorithmic complexity.
4. [**04. Spelling Bee Implementation & Solver**](file:///c:/Users/User/Downloads/Programming%20Lab/Word%20Games/docs/04-spelling-bee-implementation.md): 7-bit bitmask logic, Trie DFS solver ($\sim 204\times$ speedup), hints, and 3-column formatter.
5. [**05. Boggle Implementation & Pathfinding**](file:///c:/Users/User/Downloads/Programming%20Lab/Word%20Games/docs/05-boggle-implementation.md): Coordinate math, 8-directional DFS, cell-reuse prevention, Qu tile, and automated board solver.
6. [**06. Testing, Verification & Validation Report**](file:///c:/Users/User/Downloads/Programming%20Lab/Word%20Games/docs/06-testing-and-validation.md): 101-test suite analysis, randomized differential testing against `BTreeSet`, and coverage matrix.
7. [**07. Terminal Evidence & Execution Transcripts**](file:///c:/Users/User/Downloads/Programming%20Lab/Word%20Games/docs/07-terminal-evidence.md): Verbatim compiler logs, test outputs, dictionary metrics, and interactive session transcripts.
8. [**08. Design Decisions, Trade-offs & Rust Concepts**](file:///c:/Users/User/Downloads/Programming%20Lab/Word%20Games/docs/08-design-decisions-and-tradeoffs.md): Ownership, lifetimes, enums, cache locality, and algorithmic benchmarks.
9. [**09. Known Limitations & Future Enhancements**](file:///c:/Users/User/Downloads/Programming%20Lab/Word%20Games/docs/09-known-limitations.md): Lexical constraints, proper noun filtering boundaries, and parallelization roadmap.
10. [**10. Technical Viva Voce Examination Guide**](file:///c:/Users/User/Downloads/Programming%20Lab/Word%20Games/docs/10-viva-preparation.md): Comprehensive examination questions, code-grounded answers, and cross-examination defenses.

---

## 8. Known Limitations Summary

- **Lexical Metadata:** The dictionary contains uppercase strings without Part-of-Speech tags, preventing automatic proper noun filtering in Boggle.
- **Alphabet Bounds:** Strictly constrained to ASCII `A`–`Z` as specified by the assignment; accented and hyphenated words are excluded.
- **Node Overhead:** Fixed 26-pointer node arrays prioritize extreme traversal speed ($3\text{ ns}$ per cursor step) over dense compression ($\approx 42\text{ MB}$ memory footprint for 176k words).
- **Concurrency:** Engines currently execute single-threaded; future iterations may introduce Rayon for multi-board parallelism.
