# Wordle Game Engine & State Management

This document specifies the internal state machine, attempt budgeting, validation pipelines, and cumulative keyboard tracking implemented in `4 - Wordle/src/game.rs` and `4 - Wordle/src/keyboard.rs`.

---

## 1. Game State Representation

The game session is encapsulated within the `WordleGame` struct:

```rust
pub const DEFAULT_MAX_ATTEMPTS: usize = 6;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameStatus {
    InProgress,
    Won,
    Lost,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GuessRecord {
    pub word: Word,
    pub feedback: Feedback,
}

#[derive(Debug, Clone)]
pub struct WordleGame {
    answer: Word,
    guesses: Vec<GuessRecord>,
    status: GameStatus,
    max_attempts: usize,
}
```

### Invariants Maintained:
1. **Secret Word Confidentiality:** The `answer` field is private and never exposed to the player until the game transitions to `Won` or `Lost`.
2. **Strict Vector Growth:** Guesses are appended to `guesses` in exact chronological order. The vector length is guaranteed to be $\le max\_attempts$ ($6$).
3. **Irreversible Terminal States:** Once `status` becomes `Won` or `Lost`, it can never revert to `InProgress`. All subsequent calls to `submit_guess()` return `Err(GameError::GameOver(status))`.

---

## 2. Guess Submission Pipeline & Attempt Budgeting

```text
                  +----------------------------------+
                  |  submit_guess(input, dictionary) |
                  +----------------------------------+
                                    |
                        Is game.is_over() true?
                       /                      \
                    [Yes]                     [No]
                     /                          \
          Return GameError::GameOver      Parse Word::parse(input)
                                                /              \
                                            [Valid]         [Invalid]
                                              /                 \
                                   Already in guesses?     Return GameError::InvalidWord
                                       /          \
                                    [Yes]         [No]
                                     /              \
                 Return AlreadyGuessed        In dictionary Trie?
                 (Attempts UNCHANGED)             /          \
                                               [Yes]         [No]
                                                /              \
                                      Calculate Feedback   Return NotInDictionary
                                                |          (Attempts UNCHANGED)
                                      Append to guesses
                                                |
                                      All Green? -> Won
                                      Attempts == 6? -> Lost
                                                |
                                      Return Accepted
```

### The Non-Punitive Validation Policy:
In accordance with modern UX standards:
- **Typographical errors** (wrong length, invalid symbols) do not penalize the player.
- **Lexical omissions** (words missing from the dictionary) do not consume an attempt.
- **Accidental duplicates** (submitting a previously entered word) are rejected with a helpful notice without consuming an attempt.
- **Only legitimate, dictionary-verified, novel guesses** consume one of the 6 attempts.

---

## 3. Cumulative Virtual Keyboard State Machine

While individual board tiles display guess-specific feedback, the on-screen QWERTY keyboard tracks the player's accumulated knowledge about all 26 letters of the alphabet.

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum KeyStatus {
    Unknown = 0,
    Absent = 1,
    Present = 2,
    Correct = 3,
}
```

### Monotonic Knowledge Hierarchy:
$$\text{Unknown} < \text{Absent} < \text{Present} < \text{Correct}$$

```text
+-----------+       +----------+       +-----------+       +-----------+
|  Unknown  |  -->  |  Absent  |  -->  |  Present  |  -->  |  Correct  |
+-----------+       +----------+       +-----------+       +-----------+
      |                   ^                  ^                   ^
      +-------------------+------------------+-------------------+
```

### The Priority Invariant:
When upgrading key knowledge:
```rust
pub fn upgrade_with(&mut self, feedback: LetterFeedback) {
    let new_status = match feedback {
        LetterFeedback::Correct => KeyStatus::Correct,
        LetterFeedback::Present => KeyStatus::Present,
        LetterFeedback::Absent => KeyStatus::Absent,
    };
    if new_status > *self {
        *self = new_status;
    }
}
```

- If a letter has already been discovered as Green (`Correct`), subsequent guesses where that letter is Gray (`Absent`) or Yellow (`Present`) **cannot downgrade** the key.
- If a guess contains repeated letters (e.g. `ALLEY` where one `'L'` is Yellow and the other is Gray), the keyboard applies the strongest feedback state (`Present > Absent`), coloring `'L'` Yellow.

---

## 4. Replay Lifecycle & Round Isolation

When a round concludes, the CLI prompts: `Would you like to play another round? (y/n)`.

To maintain strict state isolation between consecutive rounds:
1. **Dictionary Persistence:** The expensive in-memory `Trie` ($\approx 42\text{ MB}$) is retained in memory, avoiding redundant disk I/O.
2. **State Reset:**
   - A fresh secret word is sampled from `answers.txt`.
   - A new `WordleGame` instance is created with `guesses.len() == 0` and `status == InProgress`.
   - `keyboard.reset()` resets all 26 keys back to `KeyStatus::Unknown`.
3. **Statistics Accumulation:** Cumulative statistics (`PlayerStats`) persist across rounds, updating total games played, win rate, and current streak.
