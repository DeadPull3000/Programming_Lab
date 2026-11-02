# Word Games in Rust: Programming Lab Suite

A university Programming Lab suite implementing high-performance word games and an in-memory prefix tree dictionary in Rust.

The suite is organized into three sequential lab assignments sharing a unified, game-independent Trie dictionary engine:

```text
Word Games/
├── Cargo.toml                      # Unified Workspace manifest linking all crates
├── Cargo.lock                      # Pinned dependency lockfile
├── README.md                       # Master University Lab Guide (this file)
│
├── 1 - Library Trie/               # Lab Assignment 1: Prefix Tree Engine & CLI
│   ├── README.md                   # Assignment 1 Guide
│   ├── trie/                       # Core 26-way Trie library crate
│   └── trie-cli/                   # Interactive REPL & batch file classifier
│
├── 2 - Spelling Bee/               # Lab Assignment 2: Spelling Bee Game
│   ├── Cargo.toml
│   ├── README.md                   # Assignment 2 Guide
│   ├── src/                        # DFS solver, 7-bit bitmask, hints, REPL
│   └── tests/                      # Full-dictionary integration tests & benchmarks
│
├── 3 - Boggle/                     # Lab Assignment 3: Boggle Game
│   ├── Cargo.toml
│   ├── README.md                   # Assignment 3 Guide
│   ├── src/                        # Board topology, 8-way DFS, scoring, solver
│   └── tests/                      # 4x4, 5x5, 6x6 full-dictionary integration
│
├── Resources/                      # Primary Lab Resources & Specifications
│   ├── Word Games.pdf              # Assignment specification handout
│   ├── dictionary.pdf              # Dictionary specification
│   └── dictionary.txt              # Primary 176,413-word dictionary
│
└── docs/                           # Technical Architecture & Test Documentation
    ├── trie-integration.md         # TrieCursor architecture and game integration
    ├── spelling-bee-design.md      # Spelling Bee architecture, DFS solver, bitmasks
    ├── spelling-bee-testing.md     # Spelling Bee test matrix, benchmarks, invariants
    ├── boggle-design.md            # Boggle architecture, 8-way DFS, scoring rules
    ├── boggle-testing.md           # Boggle test matrix, benchmarks, multi-dim runs
    ├── terminal-evidence.md        # Verified command outputs, benchmarks, transcripts
    ├── project-overview.md         # Assignment requirements & traceability
    ├── architecture-and-design.md  # Tree structures, memory & complexity
    ├── implementation-walkthrough.md # Step-by-step code analysis
    ├── testing-and-validation.md   # Comprehensive test matrix & differential tests
    └── known-limitations.md        # Design trade-offs & constraints
```

---

## Quick Start & Execution

Run all commands directly from this root directory:

```powershell
# 1. Run Spelling Bee (Interactive Terminal Game)
cargo run -p spelling-bee

# 2. Run Boggle (Interactive Terminal Game)
cargo run -p boggle

# 3. Run the original Trie Dictionary REPL
cargo run -p trie-cli

# 4. Execute the complete test suite (101 unit, integration, and doc tests)
cargo test --workspace

# 5. Run Clippy with zero warnings
cargo clippy --workspace --all-targets -- -D warnings

# 6. Verify formatting
cargo fmt --all -- --check

# 7. Compile optimized release binaries
cargo build --release --workspace

# 8. Generate local HTML documentation
cargo doc --workspace --no-deps
```

---

## Assignments Breakdown

### [1 - Library Trie](file:///c:/Users/User/Downloads/Programming%20Lab/Word%20Games/1%20-%20Library%20Trie/README.md)
- **`trie`**: A 26-way prefix tree dictionary with $O(L)$ lookups, case-normalization, prefix-preserving node deallocation, and a zero-allocation `TrieCursor` for step-by-step game navigation.
- **`trie-cli`**: Command-line REPL for live word insertion, deletion, queries, and batch file ingestion.

### [2 - Spelling Bee](file:///c:/Users/User/Downloads/Programming%20Lab/Word%20Games/2%20-%20Spelling%20Bee/README.md)
- **Procedural 7-Letter Generation**: Guaranteed 7 distinct uppercase letters with a designated mandatory main letter.
- **Trie-Guided DFS Solver**: Precomputes all valid solutions at startup in **$\sim 200\ \mu\text{s}$** (~204x faster than brute-force dictionary filtering).
- **7-Bit Bitmask Mechanics**: $O(1)$ register operations for letter usage and pangram identification.
- **Rules & Scored Game Loop**: 4-letter words (1 pt), $N$-letter words ($N$ pts), pangrams ($+7$ bonus), Queen Bee detection, 3-column alphabetical display, and optional hint grid.

### [3 - Boggle](file:///c:/Users/User/Downloads/Programming%20Lab/Word%20Games/3%20-%20Boggle/README.md)
- **Multi-Dimension Grid**: 4x4, 5x5, and 6x6 boards stored as a contiguous 1D array.
- **8-Directional Neighbors**: Mathematical neighbor discovery with corner (3), edge (5), and interior (8) neighbor validation.
- **Backtracking Path Search**: Recursive DFS preventing physical cell reuse while allowing repeated letters across distinct cells.
- **Batch Evaluation & Features**: Flexible delimiter parsing (commas, spaces), irrevocable batch evaluation upon `DONE` or `QUIT`, official scoring table, optional Qu tile support, optional automatic Trie solver, and optional 3-minute timer.

---

## Benchmark Highlights (176,413 Words)

- **Dictionary Load**: $\sim 610\text{ ms}$ (176,413 entries into Trie).
- **Spelling Bee Solution Search**: **$208.10\ \mu\text{s}$** (Trie DFS) vs. $42.44\text{ ms}$ (Brute-force).
- **Boggle Board Solver**: **$473.70\ \mu\text{s}$** (4x4), **$1.13\text{ ms}$** (5x5), **$530.40\ \mu\text{s}$** (6x6).
