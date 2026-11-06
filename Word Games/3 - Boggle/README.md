# Assignment 3: Boggle

An interactive terminal word puzzle based on the classic board game Boggle.

## Implementation Overview

- **Multiple Board Sizes**: Supports 4x4, 5x5, and 6x6 dimensions stored as a contiguous 1D array.
- **8-Directional Adjacency**: Mathematical neighbor discovery with corner (3), edge (5), and interior (8) neighbor validation.
- **Recursive Backtracking Path Validator**: Traces candidate paths on the board using a flat visited array to strictly prohibit physical cell reuse while allowing words with repeated letters that occupy distinct cells.
- **Official Scoring Table**: 3–4 letters (1 pt), 5 (2 pts), 6 (3 pts), 7 (5 pts), 8 (11 pts), and extends to $2 \times N$ for 9+ letters.
- **Batch Evaluation**: Collects player guesses continuously with flexible delimiter parsing (commas, spaces, mixed), evaluating all words irrevocably upon `DONE` or `QUIT`.
- **Optional Qu Tile**: Matches 'Q' cells against "QU" sequences in candidate guesses.
- **Optional Automatic Boggle Solver**: Uses Trie-guided DFS with prefix pruning to find all valid dictionary words on any generated board in under 2 milliseconds.
- **Optional 3-Minute Timer**: Non-blocking round timer using `std::time::Instant`.

## Running

From the root directory:

```powershell
# Run Boggle
cargo run -p boggle

# Run Boggle tests
cargo test -p boggle
```
