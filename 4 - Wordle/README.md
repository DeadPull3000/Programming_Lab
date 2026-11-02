# Wordle Application

A standalone, memory-safe, and robust Wordle game implemented in Rust as part of the Word Games Programming Lab suite.

Wordle reuses the shared 26-way in-memory [`Trie`](../1%20-%20Library%20Trie/trie) library for broad dictionary validation while drawing secret answers from a curated 5-letter answer lexicon.

---

## 1. Quick Start

Run the interactive terminal game directly from the workspace root:

```powershell
cargo run -p wordle
```

Or execute the complete test suite:

```powershell
cargo test -p wordle
```

---

## 2. Game Rules & Gameplay

- **Secret Answer:** A randomly chosen 5-letter English word.
- **Attempts:** Exactly 6 valid attempts per round.
- **Tile Feedback:**
  - **Green (`Correct` / `G`):** Letter is in the correct spot.
  - **Yellow (`Present` / `Y`):** Letter is in the word but in a different position.
  - **Gray (`Absent` / `X`):** Letter does not occur in unallocated positions.
- **Repeated Letter Handling:** Solved via an exact **two-pass algorithm** guaranteeing that a letter never receives more positive feedback than its occurrences in the secret word.
- **Non-Punitive Input Policy:** Invalid words, non-dictionary words, and duplicates do **not** consume an attempt.
- **Special Commands:**
  - `HELP`: Display in-game rules and command summaries.
  - `STATS`: View cumulative game record, win rate, and guess distribution.
  - `QUIT`: Surrender, reveal the secret answer, and prompt for replay.

---

## 3. Package Architecture

```text
4 - Wordle/
├── Cargo.toml
├── src/
│   ├── lib.rs            # Public crate API re-exports
│   ├── main.rs           # Interactive CLI game loop and session management
│   ├── word.rs           # Validated 5-letter ASCII Word newtype
│   ├── dictionary.rs     # Dual-tier dictionary and answer-pool loader
│   ├── feedback.rs       # Canonical two-pass Green/Yellow/Gray matching
│   ├── game.rs           # WordleGame state machine and attempt tracking
│   ├── keyboard.rs       # Monotonic cumulative QWERTY keyboard state
│   ├── display.rs        # Terminal grid, tile formatting, and ANSI colors
│   ├── candidate.rs      # Consistency-based candidate elimination assistant
│   └── stats.rs          # Player streaks, guess distribution histogram & I/O
└── tests/
    └── wordle_integration.rs # Simulated full sessions, victory, and replay isolation
```

---

## 4. Documentation Links

- [**Wordle Architecture & System Design**](../docs/wordle-architecture.md)
- [**Two-Pass Feedback Algorithm & Mathematical Proofs**](../docs/wordle-feedback-algorithm.md)
- [**Game Engine & State Management**](../docs/wordle-game-engine.md)
- [**Testing, Verification & Validation Report**](../docs/wordle-testing.md)
- [**Design Decisions, Trade-offs & Rust Concepts**](../docs/wordle-design-decisions.md)
