# Wordle Application

A standalone, memory-safe, and robust Wordle game implemented in Rust as part of the Word Games Programming Lab suite.

Wordle reuses the shared 26-way in-memory [`Trie`](../1%20-%20Library%20Trie/trie) library for broad dictionary validation while drawing secret answers from a curated 5-letter answer lexicon. It features **dynamic candidate filtering**, an **information-theoretic solver assistant**, and **atomic persistent player statistics**.

---

## 1. Quick Start

Run the interactive terminal game directly from the workspace root:

```powershell
cargo run -p wordle
```

Run with a custom persistent statistics file path:

```powershell
cargo run -p wordle -- --stats-file ./custom_stats.json
```

Execute the comprehensive test suite across the workspace:

```powershell
cargo test --workspace
```

Run the dedicated advanced features and performance benchmarks:

```powershell
cargo test -p wordle --test advanced_features_integration -- --nocapture
```

---

## 2. Game Rules & Interactive Commands

- **Secret Answer:** A randomly chosen 5-letter English word drawn from a curated answer pool.
- **Attempts:** Exactly 6 valid attempts per round.
- **Tile Feedback:**
  - **Green (`Correct` / `G`):** Letter is in the correct spot.
  - **Yellow (`Present` / `Y`):** Letter is in the word but in a different position.
  - **Gray (`Absent` / `X`):** Letter does not occur among remaining unallocated letters.
- **Repeated Letter Handling:** Solved via an exact **two-pass algorithm** guaranteeing that a letter never receives more positive feedback than its occurrences in the secret word.
- **Non-Punitive Input Policy:** Invalid words, non-dictionary words, and duplicates do **not** consume an attempt.
- **Interactive Commands:**
  - `:hint` / `HINT`: Run the information-theoretic solver to recommend the top guesses ranked by Shannon entropy.
  - `:hint minimax`: Run the minimax solver to recommend the top guesses that minimize the worst-case surviving candidates.
  - `:candidates` / `CANDIDATES`: Display the count and list of all surviving secret answer candidates.
  - `:stats` / `STATS`: View cumulative game record, win rate, streaks, and guess distribution histogram.
  - `:quit` / `QUIT`: Surrender, record the round as abandoned (preserving streaks), reveal the answer, and prompt for replay.
  - `:help` / `HELP`: Display in-game rules and command summaries.

---

## 3. Advanced Features Architecture

### 3.1 Candidate Filtering (`src/candidate.rs`)
- **`CandidateSet` & `FilterSummary`:** Maintains a dynamically updated collection of every secret answer consistent with all feedback received so far.
- **Authoritative Feedback Reuse:** Avoids ad-hoc regexes; each candidate is evaluated against the player's guess using the canonical two-pass feedback algorithm. Candidates that do not produce the identical observed feedback are pruned in place.
- **Monotonicity & Invariance:** Candidate counts are strictly non-increasing ($C_{t+1} \subseteq C_t$). Verified against independent full recomputations in tests.

### 3.2 Intelligent Solver Assistant (`src/solver.rs`)
- **Base-3 Feedback Pattern Encoding:** Maps all $3^5 = 243$ discrete feedback states to compact integer IDs in `0..=242`.
- **Candidate Partitioning:** Projects proposed guesses against the candidate pool into 243 discrete frequency buckets ($\sum n_i = N$).
- **Shannon Entropy ($H$):** Measures the expected information gain in bits:
  $$H(G) = -\sum_{i: n_i > 0} p_i \log_2(p_i) \quad \text{where } p_i = \frac{n_i}{N}$$
- **Expected Remaining Candidates ($E$):** Evaluates the average surviving candidate count:
  $$E(G) = \frac{\sum_{i: n_i > 0} n_i^2}{N}$$
- **Minimax ($W$):** Measures the worst-case remaining candidates ($W(G) = \max_i n_i$).
- **Non-Candidate Explorers:** Evaluates all legal 5-letter dictionary words (8,812 words), allowing useful information-gathering guesses that distinguish candidates even if they cannot themselves be the secret answer.

### 3.3 Persistent Player Statistics (`src/stats/`)
- **`model.rs`:** `GameOutcome` (`Won`, `Lost`, `Abandoned`), `GameRecord` with unique IDs and timestamps, and `PlayerStats`.
- **`manager.rs`:** `StatsManager` enforcing duplicate protection (idempotent game IDs), checked arithmetic, derived metrics (`win_rate`, `average_guesses`, `average_guesses_to_win`), and mathematical invariant validation.
- **`storage.rs`:** Robust, zero-dependency JSON serialization, OS application data directory resolution (`%APPDATA%`, `XDG_DATA_HOME`, `Application Support`), and atomic file replacement (`.tmp` write, sync, atomic rename) to eliminate corruption.
- **`migration.rs`:** Explicit schema versioning (`schema_version = 1`) and forward migration hooks.

---

## 4. Package Directory Layout

```text
4 - Wordle/
├── Cargo.toml
├── src/
│   ├── lib.rs            # Public crate API re-exports
│   ├── main.rs           # Interactive CLI game loop and session management
│   ├── word.rs           # Validated 5-letter ASCII Word newtype
│   ├── dictionary.rs     # Dual-tier dictionary and answer-pool loader
│   ├── feedback.rs       # Canonical two-pass feedback & base-3 pattern encoder
│   ├── game.rs           # WordleGame state machine and attempt tracking
│   ├── keyboard.rs       # Monotonic cumulative QWERTY keyboard state
│   ├── display.rs        # Terminal grid, tile formatting, and ANSI colors
│   ├── candidate.rs      # CandidateSet, FilterSummary, and pruning engine
│   ├── solver.rs         # Shannon entropy, minimax, and guess ranker
│   └── stats/            # Persistent statistics subsystem
│       ├── mod.rs        # Subsystem module re-exports
│       ├── model.rs      # GameRecord, GameOutcome, PlayerStats
│       ├── manager.rs    # StatsManager invariant coordinator
│       ├── storage.rs    # Atomic JSON persistence & OS path resolution
│       └── migration.rs  # Schema versioning & forward migration engine
└── tests/
    ├── feedback_differential.rs       # 89,073-pair differential test suite
    ├── wordle_integration.rs          # Core gameplay & simulated round tests
    └── advanced_features_integration.rs # End-to-end solver, candidate, & persistence tests
```

---

## 5. Measured Performance Benchmarks

Measured on the actual workspace lexicons (176,413 total words, 8,812 legal 5-letter guesses, 629 curated answers):

| Benchmark Operation | Scope / Input Size | Measured Time |
| :--- | :--- | :---: |
| **Candidate Filtering** | Pruning 629 candidates down to 19 after guess | **180.4 µs** |
| **Solver Evaluation (Opening Pool)** | 8,812 legal guesses $\times$ 629 candidates (**5,542,748 pairs**) | **1.58 s** (debug) / **<250 ms** (release) |
| **Solver Evaluation (Pruned Pool)** | 8,812 legal guesses $\times$ 25 candidates (**220,300 pairs**) | **83.2 ms** |
| **Atomic JSON Persistence (Save)** | 100 complete `GameRecord` logs written, synced, atomically replaced | **43.0 ms** |
| **JSON Storage Load & Parse** | Reading, parsing, and validating 100 game records | **27.4 ms** |

---

## 6. Documentation Links

- [**Wordle Architecture & System Design**](../docs/wordle-architecture.md)
- [**Two-Pass Feedback Algorithm & Mathematical Proofs**](../docs/wordle-feedback-algorithm.md)
- [**Game Engine & State Management**](../docs/wordle-game-engine.md)
- [**Testing, Verification & Validation Report**](../docs/wordle-testing.md)
- [**Design Decisions, Trade-offs & Rust Concepts**](../docs/wordle-design-decisions.md)
