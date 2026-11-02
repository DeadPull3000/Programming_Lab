# Word Games in Rust: Programming Lab Suite

An industrial-grade, memory-safe, and high-performance dictionary and word-puzzle game ecosystem in Rust, developed for an academic Programming Lab submission.

The suite comprises four interconnected components sharing a unified, game-independent 26-way prefix tree (Trie) dictionary engine:
1. **Encapsulated 26-ary Trie Dictionary Library & Interactive CLI** (`1 - Library Trie/`)
2. **NYT-Style Spelling Bee Game Engine & CLI** (`2 - Spelling Bee/`)
3. **Multi-Dimension Boggle Game Engine, Solver & CLI** (`3 - Boggle/`)
4. **Standalone Wordle Application & Candidate Solver** (`4 - Wordle/`)

---

## 1. Project Purpose & Problem Statement

Modern word puzzle applications require high-throughput lexical validation and dynamic word discovery. Searching large dictionaries (over 100,000 words) using linear scans or unindexed lookups is computationally prohibitive.

This project solves this challenge by developing a custom 26-ary prefix tree (Trie) in Rust. The Trie provides $O(L)$ exact lookups, prefix searches, and recursive node pruning, while exposing an innovative, zero-allocation `TrieCursor` traversal API. Three complete interactive games—**Spelling Bee**, **Boggle**, and **Wordle**—are constructed on top of this shared library, utilizing Trie validation and Depth-First Search (DFS) algorithms to achieve sub-millisecond solution enumeration and robust dictionary lookups.

---

## 2. Workspace Architecture

The workspace is organized as a Cargo virtual workspace:

```text
Word Games/
├── Cargo.toml                    # Root workspace manifest linking all packages
├── Cargo.lock                    # Dependency lockfile
├── README.md                     # Master laboratory guide (this file)
├── dictionary.txt                # Root fallback dictionary file (176,413 words)
├── answers.txt                   # Root curated Wordle answer file (632 words)
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
├── 4 - Wordle/                   # Part 4: Wordle Application & Solver
│   ├── Cargo.toml
│   ├── README.md                 # Architecture, rules, two-pass feedback, and commands
│   ├── src/                      # WordleGame, two-pass feedback, keyboard, display, candidate, stats
│   └── tests/                    # Integration & full game simulation tests
│
├── Resources/                    # Assignment PDFs and Lexicons
│   ├── README.md
│   ├── Word Games.pdf            # Assignment specification handout
│   ├── dictionary.pdf            # Original dictionary document
│   ├── dictionary.txt            # Preprocessed newline-delimited word list (176,413 words)
│   └── answers.txt               # Curated 5-letter secret answers (632 words)
│
└── docs/                         # Technical Documentation Suite (15 Reports)
    ├── 01-project-overview.md
    ├── 02-system-architecture.md
    ├── 03-trie-implementation.md
    ├── 04-spelling-bee-implementation.md
    ├── 05-boggle-implementation.md
    ├── 06-testing-and-validation.md
    ├── 07-terminal-evidence.md
    ├── 08-design-decisions-and-tradeoffs.md
    ├── 09-known-limitations.md
    ├── 10-viva-preparation.md
    ├── wordle-architecture.md
    ├── wordle-feedback-algorithm.md
    ├── wordle-game-engine.md
    ├── wordle-testing.md
    └── wordle-design-decisions.md
```

---

## 3. How the Games Reuse the Shared Trie

All three game packages depend directly on the `trie` library (`trie = { path = "../1 - Library Trie/trie" }`):
- **Encapsulation via `TrieCursor`:** The Trie's internal nodes (`Node`) remain strictly private (`pub(crate)`). Client crates interact with tree branches exclusively through `TrieCursor<'a>`, an immutable, zero-allocation pointer wrapper.
- **Spelling Bee Solution Solver:** Solves the entire puzzle in **$208\ \mu\text{s}$** by recursively advancing `TrieCursor` along only the 7 allowed hive letters, instantly pruning missing subtrees.
- **Boggle Path Validation & Board Solver:** Evaluates player guesses against the Trie, and powers an automatic solver that extracts all valid board words across 4x4, 5x5, or 6x6 boards in **$< 1.2\text{ ms}$**.
- **Wordle Guess Lexical Verification:** Rapidly verifies submitted 5-letter words against the complete 176k-word dictionary in $O(L)$ time with zero memory allocation.

---

## 4. Prerequisites & Dependencies

- **Rust Toolchain:** Rust `1.85.0` or higher (verified on `rustc 1.99.0` and `cargo 1.99.0`).
- **Standard Library Purity:** The core `trie` library has **zero external dependencies**.
- **Downstream Dependencies:**
  - `rand = "0.8.8"` (used by `spelling-bee`, `boggle`, and `wordle` for random sampling and shuffles).
- **Lexicons:**
  - `Resources/dictionary.txt`: 176,413 uppercase ASCII words.
  - `Resources/answers.txt`: 632 curated, common 5-letter English words.

---

## 5. Build, Test & Quality Commands

Execute all commands from the workspace root (`c:\Users\User\Downloads\Programming Lab\Word Games`):

```powershell
# 1. Run all 150 unit, doc, and integration tests across all packages
cargo test --workspace

# 2. Run static analysis (Clippy) with warnings treated as errors
cargo clippy --workspace --all-targets -- -D warnings

# 3. Verify standard code formatting
cargo fmt --all -- --check

# 4. Compile optimized release binaries for all applications
cargo build --release --workspace

# 5. Generate local HTML documentation
cargo doc --workspace --no-deps
```

---

## 6. Running the Applications

### A. Wordle (`wordle`)
```powershell
cargo run -p wordle
```
Features:
- Validated 5-letter `Word` newtype with automatic case normalization and whitespace trimming.
- Exact two-pass Green/Yellow/Gray feedback matching with strict duplicate-letter frequency conservation.
- Cumulative QWERTY virtual keyboard tracking knowledge monotonically (`Unknown < Absent < Present < Correct`).
- Non-punitive input policy: duplicates, non-dictionary words, and format errors do not consume attempts.
- In-game commands: `HELP`, `STATS` (view streak and attempt histogram), `QUIT` (surrender and reveal).
- Replay support: consecutive rounds reuse in-memory Trie without reloading from disk.

### B. Spelling Bee (`spelling-bee`)
```powershell
cargo run -p spelling-bee
```
Features:
- Procedural generation of 7 distinct letters with a mandatory center letter.
- Trie DFS solver precomputes all solutions in **$208\ \mu\text{s}$** ($\sim 204\times$ speedup).
- 7-bit bitmask validation and pangram detection.
- Commands: `:hint`, `:list` (3-column alphabetical layout), `:shuffle`, `:quit`.

### C. Boggle (`boggle`)
```powershell
cargo run -p boggle
```
Features:
- Configurable grid dimensions: 4x4 (Classic), 5x5 (Big Boggle), 6x6 (Super Big).
- 8-directional recursive DFS pathfinding preventing physical cell reuse.
- Optional `"Qu"` special tile expansion, round countdown timer, and automated board solver.

### D. Trie CLI (`trie-cli`)
```powershell
cargo run -p trie-cli
```
Interactive dictionary management commands: `find`, `insert`, `delete`, `load`, `check`, `stats`, `quit`.

---

## 7. Technical Documentation Suite

The complete technical report suite is located in the [`docs/`](file:///c:/Users/User/Downloads/Programming%20Lab/Word%20Games/docs) directory:

### Core Lab Reports:
1. [**01. Project Overview & Requirements Traceability**](file:///c:/Users/User/Downloads/Programming%20Lab/Word%20Games/docs/01-project-overview.md)
2. [**02. System Architecture & Component Interaction**](file:///c:/Users/User/Downloads/Programming%20Lab/Word%20Games/docs/02-system-architecture.md)
3. [**03. Trie Implementation & Cursor Mechanics**](file:///c:/Users/User/Downloads/Programming%20Lab/Word%20Games/docs/03-trie-implementation.md)
4. [**04. Spelling Bee Implementation & Solver**](file:///c:/Users/User/Downloads/Programming%20Lab/Word%20Games/docs/04-spelling-bee-implementation.md)
5. [**05. Boggle Implementation & Pathfinding**](file:///c:/Users/User/Downloads/Programming%20Lab/Word%20Games/docs/05-boggle-implementation.md)
6. [**06. Testing, Verification & Validation Report**](file:///c:/Users/User/Downloads/Programming%20Lab/Word%20Games/docs/06-testing-and-validation.md)
7. [**07. Terminal Evidence & Execution Transcripts**](file:///c:/Users/User/Downloads/Programming%20Lab/Word%20Games/docs/07-terminal-evidence.md)
8. [**08. Design Decisions, Trade-offs & Rust Concepts**](file:///c:/Users/User/Downloads/Programming%20Lab/Word%20Games/docs/08-design-decisions-and-tradeoffs.md)
9. [**09. Known Limitations & Future Enhancements**](file:///c:/Users/User/Downloads/Programming%20Lab/Word%20Games/docs/09-known-limitations.md)
10. [**10. Technical Viva Voce Examination Guide**](file:///c:/Users/User/Downloads/Programming%20Lab/Word%20Games/docs/10-viva-preparation.md)

### Wordle Deep-Dive Reports:
11. [**Wordle Architecture & System Design**](file:///c:/Users/User/Downloads/Programming%20Lab/Word%20Games/docs/wordle-architecture.md)
12. [**Wordle Two-Pass Feedback Algorithm & Mathematical Proofs**](file:///c:/Users/User/Downloads/Programming%20Lab/Word%20Games/docs/wordle-feedback-algorithm.md)
13. [**Wordle Game Engine & State Management**](file:///c:/Users/User/Downloads/Programming%20Lab/Word%20Games/docs/wordle-game-engine.md)
14. [**Wordle Testing, Verification & Validation Report**](file:///c:/Users/User/Downloads/Programming%20Lab/Word%20Games/docs/wordle-testing.md)
15. [**Wordle Design Decisions, Trade-offs & Rust Concepts**](file:///c:/Users/User/Downloads/Programming%20Lab/Word%20Games/docs/wordle-design-decisions.md)
