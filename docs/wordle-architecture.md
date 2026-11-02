# Wordle Architecture & System Design

This document details the software architecture, module decomposition, data flow pipelines, and library integration for the standalone Wordle application (`4 - Wordle/`).

---

## 1. High-Level System Architecture

The Wordle application is integrated into the Cargo virtual workspace, depending directly on the existing `trie` dictionary crate:

```text
                           +-------------------------------------+
                           |    Resources/dictionary.txt         |
                           |    (176,413 Raw Accepted Words)     |
                           +-------------------------------------+
                                              |
                                              v
                           +-------------------------------------+
                           |    Existing Shared Trie Library     |
                           |    (1 - Library Trie/trie)          |
                           |    - Fast O(L) Membership Lookup    |
                           +-------------------------------------+
                                              ^
                                              | (Read-Only Borrow)
+------------------------+                    |
| Resources/answers.txt  |                    |
| (632 Curated Words)    |                    |
+------------------------+                    |
            |                                 |
            v                                 |
+----------------------------------------------------------------+
|                   wordle Crate (4 - Wordle)                    |
|                                                                |
|  +--------------------+        +----------------------------+  |
|  |  WordleDictionary  | -----> | WordleGame Engine          |  |
|  |  - Trie (Accepted) |        | - Answer: Word             |  |
|  |  - Answers: Vec    |        | - History: Vec<GuessRecord>|  |
|  +--------------------+        | - Status: GameStatus       |  |
|            |                   +----------------------------+  |
|            v                                 |                 |
|  +--------------------+                      v                 |
|  | Candidate Filtering|            +-------------------+       |
|  | (Solvers / Hints)  |            | Two-Pass Feedback |       |
|  +--------------------+            +-------------------+       |
|                                              |                 |
|                                              v                 |
|  +--------------------+            +-------------------+       |
|  | Terminal Display   | <--------- | Keyboard State    |       |
|  | (ANSI / Plain-text)|            | (Cumulative G/Y/X)|       |
|  +--------------------+            +-------------------+       |
+----------------------------------------------------------------+
```

---

## 2. Module Responsibilities

The application is cleanly partitioned into focused, single-responsibility modules:

| Source File | Module Responsibility | Key Types & Functions |
| :--- | :--- | :--- |
| **`src/word.rs`** | Invariant-enforced 5-letter ASCII word type | `struct Word`, `enum WordError`, `WORD_LENGTH = 5`, `Word::parse()` |
| **`src/dictionary.rs`** | Dual-tier dictionary and curated answer pool management | `struct WordleDictionary`, `enum DictionaryError`, `WordleDictionary::load_default()`, `choose_answer()` |
| **`src/feedback.rs`** | Two-pass tile evaluation algorithm (Green/Yellow/Gray) | `enum LetterFeedback`, `struct Feedback`, `calculate_feedback()` |
| **`src/game.rs`** | Game state machine, attempt budgeting, and guess outcomes | `struct WordleGame`, `struct GuessRecord`, `enum GameStatus`, `enum GuessOutcome`, `enum GameError` |
| **`src/keyboard.rs`** | Monotonic cumulative 26-letter virtual keyboard state | `enum KeyStatus`, `struct KeyboardState`, `KeyboardState::update_from_guess()` |
| **`src/display.rs`** | Terminal grid rendering, ANSI colorization, and summaries | `struct DisplayConfig`, `render_board()`, `render_keyboard()`, `render_welcome_banner()` |
| **`src/candidate.rs`** | Consistency-based candidate elimination solver assistant | `filter_candidates()`, `filter_candidates_multi_step()` |
| **`src/stats.rs`** | Win/loss streaks, attempt distribution histogram, and persistence | `struct PlayerStats`, `PlayerStats::record_game()`, `render_summary()` |
| **`src/main.rs`** | Interactive REPL, command dispatch, and replay lifecycle | `main()`, input handling, session management |

---

## 3. Data Flow & Guess Lifecycle

```text
Player Enters Raw String (e.g. "  crane  ")
                    |
                    v
          1. Trim & Normalize
                    |
                    v
          2. Word::parse()
     (Validates exactly 5 ASCII letters)
          /                   \
      [Valid]              [Invalid] -> Return WordError
        /
       v
3. Duplicate Check: game.guesses()
        /                   \
    [New]               [Duplicate] -> Return GuessOutcome::AlreadyGuessed
      /
     v
4. Trie Membership Check: trie.contains()
      /                     \
  [In Trie]             [Absent] -> Return GuessOutcome::NotInDictionary
    /
   v
5. calculate_feedback(&answer, &word)
   - Pass 1: Mark exact Green matches
   - Pass 2: Allocate remaining Yellow/Gray
    |
    v
6. Append GuessRecord to game.guesses
    |
    v
7. Update GameStatus (Won if 5 Green, Lost if 6 attempts exhausted)
    |
    v
8. Update KeyboardState (Monotonically upgrade key statuses)
    |
    v
9. Render Board & Keyboard
```

---

## 4. Architectural Invariants & Memory Safety

1. **Separation of Guess Validation from Secret Selection:**
   - Accepted guesses are verified against the comprehensive 176k-word `Trie`.
   - Secret answers are drawn from a curated 632-word pool (`Resources/answers.txt`).
   - Every answer in `answers.txt` is verified at startup to be an element of the Trie ($Answers \subseteq AcceptedGuesses$).
2. **Immutability of Dictionary During Gameplay:**
   - The Trie and answer vector are loaded once per application lifecycle into an immutable `WordleDictionary`.
   - Replaying rounds reuses the loaded structures without disk access.
3. **Idempotency of Failed Inputs:**
   - Invalid formats, non-dictionary words, and duplicates exit the pipeline before modifying `WordleGame`, preserving attempt counts.
