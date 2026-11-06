# Wordle Two-Pass Feedback Algorithm

This document provides a rigorous mathematical and algorithmic explanation of the two-pass tile feedback algorithm implemented in `4 - Wordle/src/feedback.rs`.

---

## 1. The Challenge of Repeated Letters

In Wordle, each letter of a 5-letter guess receives one of three feedback states:
- **Correct (`G`):** The letter is in the secret word and at the correct position.
- **Present (`Y`):** The letter occurs in the secret word, but in a different position.
- **Absent (`X`):** No unmatched occurrences of this letter remain in the secret word.

### Why Naive Single-Pass Matching Fails:
Consider the classic test case:
- **Answer:** `A P P L E` (contains exactly **one** `'L'`)
- **Guess:** `A L L E Y` (contains **two** `'L'`s)

A naive algorithm that checks whether `answer.contains(guess[i])` would mark both `'L'`s at positions 1 and 2 as Yellow (`Present`). This violates Wordle's fundamental rule: **a guess cannot receive more feedback for a letter than the number of times that letter appears in the secret word.**

Because `APPLE` contains only one `'L'`, exactly one of the `'L'`s in `ALLEY` may receive a Yellow tile; the second must be Gray (`Absent`).

Furthermore, exact matches (Green) take absolute priority over displaced matches (Yellow).
Consider:
- **Answer:** `A P P L E` (contains two `'P'`s at positions 1 and 2)
- **Guess:** `P P P P P` (contains five `'P'`s)

Positions 1 and 2 are exact Green matches. The remaining three `'P'`s must be Gray (`Absent`) because both `'P'`s in `APPLE` have already been consumed by exact matches!

---

## 2. The Two-Pass Matching Algorithm

To guarantee exact frequency conservation, we implement a canonical **two-pass algorithm** with a fixed-size 26-element integer frequency table.

```rust
pub fn calculate_feedback(answer: &Word, guess: &Word) -> Feedback {
    let a_bytes = answer.as_bytes();
    let g_bytes = guess.as_bytes();

    let mut states = [LetterFeedback::Absent; WORD_LENGTH];
    let mut remaining = [0usize; 26];

    // PASS 1: Identify all exact matches (Green)
    for i in 0..WORD_LENGTH {
        if g_bytes[i] == a_bytes[i] {
            states[i] = LetterFeedback::Correct;
        } else {
            // Count letters in answer that were NOT matched by exact green tiles
            let idx = (a_bytes[i] - b'A') as usize;
            remaining[idx] += 1;
        }
    }

    // PASS 2: Allocate remaining letters to non-exact positions (Yellow vs Gray)
    for i in 0..WORD_LENGTH {
        if states[i] != LetterFeedback::Correct {
            let idx = (g_bytes[i] - b'A') as usize;
            if remaining[idx] > 0 {
                states[i] = LetterFeedback::Present;
                remaining[idx] -= 1;
            } else {
                states[i] = LetterFeedback::Absent;
            }
        }
    }

    Feedback { states }
}
```

---

## 3. Step-by-Step Algorithmic Trace

### Case 1: `APPLE` (Answer) vs. `ALLEY` (Guess)

```text
Answer:  A   P   P   L   E
Guess:   A   L   L   E   Y
Index:   0   1   2   3   4
```

#### Pass 1 (Exact Matches):
- Index 0: `guess[0] ('A') == answer[0] ('A')` $\to$ **Correct (`G`)**.
- Index 1: `guess[1] ('L') != answer[1] ('P')` $\to$ Unmatched. `remaining['P'] += 1`.
- Index 2: `guess[2] ('L') != answer[2] ('P')` $\to$ Unmatched. `remaining['P'] += 1`.
- Index 3: `guess[3] ('E') != answer[3] ('L')` $\to$ Unmatched. `remaining['L'] += 1`.
- Index 4: `guess[4] ('Y') != answer[4] ('E')` $\to$ Unmatched. `remaining['E'] += 1`.

*State after Pass 1:*
- `states`: `[G, ?, ?, ?, ?]`
- `remaining`: `'P': 2`, `'L': 1`, `'E': 1`, all other letters `0`.

#### Pass 2 (Displaced Matches):
- Index 1: `guess[1] == 'L'`. `remaining['L'] == 1 > 0`!
  $\to$ Mark **Present (`Y`)**, decrement `remaining['L'] = 0`.
- Index 2: `guess[2] == 'L'`. `remaining['L'] == 0`!
  $\to$ Mark **Absent (`X`)**.
- Index 3: `guess[3] == 'E'`. `remaining['E'] == 1 > 0`!
  $\to$ Mark **Present (`Y`)**, decrement `remaining['E'] = 0`.
- Index 4: `guess[4] == 'Y'`. `remaining['Y'] == 0`!
  $\to$ Mark **Absent (`X`)**.

*Final Feedback:* `G Y X Y X` (Exact match with reference expectation!).

---

## 4. Reference Test Matrix Verification

| Answer | Guess | Pattern | Trace Rationale |
| :--- | :--- | :---: | :--- |
| `APPLE` | `APPLE` | **`G G G G G`** | All positions match identically in Pass 1. |
| `APPLE` | `ALLEY` | **`G Y X Y X`** | Single `L` in answer claims index 1; index 2 is Gray. |
| `APPLE` | `AAAAA` | **`G X X X X`** | Index 0 consumes the single `A`; indices 1–4 are Gray. |
| `LEVEL` | `ELVES` | **`Y Y G G X`** | Indices 2 (`V`) and 3 (`E`) match Green; indices 0 and 1 are Yellow. |
| `BANAL` | `ALLAY` | **`Y Y X G X`** | Two `A`s in answer: index 3 is Green (`A`), index 0 is Yellow (`A`). |
| `CRATE` | `TRACE` | **`Y G G Y G`** | Indices 1 (`R`), 2 (`A`), 4 (`E`) match Green; indices 0 and 3 are Yellow. |

---

## 5. Mathematical Invariants & Complexity

1. **Frequency Conservation Invariant:**
   $$\forall c \in ['A'..='Z'], \quad \text{count}(G, c) + \text{count}(Y, c) \le \text{count}(\text{answer}, c)$$
2. **Determinism:**
   $$\text{calculate\_feedback}(A, G) = \text{calculate\_feedback}(A, G)$$
   The function is purely functional, free of side-effects, and does not perform I/O.
3. **Time Complexity:**
   Two linear passes over fixed length $L = 5$. Total operations: exactly $10$ comparisons.
   $$\text{Time Complexity} = O(L) = O(1)$$
4. **Space Complexity:**
   Fixed 26-element stack allocation `[usize; 26]`. Zero heap allocations.
   $$\text{Space Complexity} = O(1)$$
