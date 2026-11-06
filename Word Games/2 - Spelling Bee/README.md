# Assignment 2: Spelling Bee

An interactive terminal word puzzle based on The New York Times Spelling Bee.

## Implementation Overview

- **Shared Dictionary Engine**: Precomputes all solutions at startup using the `trie` crate via a zero-allocation `TrieCursor`.
- **7-Letter Puzzle Generation**: Shuffles the alphabet without replacement and designates one letter as the mandatory center letter (`letters[0] == main_letter`).
- **Trie-Guided DFS Solver**: Recursively explores only existing branches of the Trie matching the 7 permitted letters. Solves the full 176k dictionary in **~200 microseconds** (~204x faster than brute force).
- **7-Bit Bitmask Mechanics**: $O(1)$ tracking of letter usage and pangram detection (`FULL_PANGRAM_MASK = 127`).
- **Scoring Policy**: 1 point for 4-letter words, $N$ points for $N \ge 5$, and $+7$ bonus points for pangrams.
- **Nine-Stage Guess Validation**: Rejects words that are too short, have non-alphabetic characters, lack the main letter, use unauthorized letters, or are duplicate submissions.
- **Queen Bee & Accolades**: Detects when all precomputed solutions are found.
- **3-Column Alphabetical Display**: Displays discovered and missed words sorted down three columns.
- **Optional Hint System**: Length-by-letter matrix with totals, plus two-letter prefix frequencies.

## Running

From the root directory:

```powershell
# Run Spelling Bee
cargo run -p spelling-bee

# Run Spelling Bee tests
cargo test -p spelling-bee
```
