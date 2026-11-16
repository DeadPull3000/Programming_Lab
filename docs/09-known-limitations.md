# 09. Known Limitations, Boundary Conditions & Future Enhancements

> The runtime and arcade notes in [maintenance and validation](11-maintenance-and-validation.md) describe the current implementation. Benchmark numbers below are historical observations, not performance guarantees.

This document objectively analyzes the confirmed boundaries, lexical constraints, architectural trade-offs, and planned future enhancements of the Word Games software suite.

---

## 1. Classification of Limitations

To uphold strict academic integrity, this report categorizes all observations into four clear classifications:
1. **Confirmed Lexical / Runtime Limitations:** Empirical boundaries confirmed by source-code inspection or testing.
2. **Deliberate Design Trade-offs:** Architecturally intentional choices balancing performance against complexity.
3. **Ambiguities in Assignment Specifications:** Underspecified requirements resolved through explicit engineering policies.
4. **Recommended Future Enhancements:** Non-blocking optimizations proposed for subsequent iterations.

---

## 2. Confirmed Lexical & Dictionary Constraints

### A. Lack of Lexical Metadata for Proper Noun Filtering
- **Observation:** The provided lexicon (`Resources/dictionary.txt`) contains **176,413 words** in uniform uppercase ASCII without Part-of-Speech (POS) tags or capitalization flags.
- **Impact on Boggle:** Official Boggle rules strictly disallow proper nouns (e.g., names of people, cities, days of the week). However, words like `"PARIS"`, `"MARCH"`, and `"WEDNESDAY"` are stored identically to ordinary common nouns.
- **Engineering Reality:** It is mathematically impossible to filter proper nouns through uppercase string normalization alone without an annotated lexicon (e.g. WordNet or Wiktionary tags).
- **Current Handling:** Any word present in `dictionary.txt` is accepted if formed on the board. The documentation explicitly records this lexical boundary.

### B. ASCII Alphabetical Restriction (A–Z Only)
- **Constraint:** In strict adherence to `Word Games.pdf` ("assume that the words only use letters A-Z (no special characters)"), the Trie enforces $idx = byte - \text{b'A'}$.
- **Boundary:** Words containing hyphens, apostrophes (e.g., `"O'CLOCK"`, `"DON'T"`), accented vowels (`"RÉSUMÉ"`), or non-English characters are rejected with `TrieError::InvalidCharacter`.

---

## 3. Memory & Structural Trade-offs

### Sparse 26-Way Node Array Overhead
- **Structure:** Each `Node` contains `[Option<Box<Node>>; 26]`.
- **Memory Footprint:** On 64-bit systems, each pointer occupies 8 bytes ($26 \times 8 = 208\text{ bytes}$ per node, plus the terminal flag and padding). Loading all **176,413 words** creates approximately 200,000 nodes, consuming $\approx 42\text{ MB}$ of resident memory.
- **Trade-off Justification:** Fixed arrays provide instantaneous $O(1)$ pointer indexing without hashing overhead. This trade-off prioritizes extreme traversal velocity ($3\text{ ns}$ per cursor step) over dense compression.
- **Future Alternative:** A Radix Tree (Patricia Trie) or Double-Array Trie (DAT) would reduce memory consumption to $< 8\text{ MB}$, albeit with increased algorithmic complexity during dynamic insertions and deletions.

---

## 4. Assignment Ambiguities & Resolution Policies

### A. Boggle Scoring for Words Longer than 8 Letters
- **Ambiguity:** The assignment handout specifies points for word lengths 3 through 8, but leaves open-ended the scoring for 9-letter words or longer on $5 \times 5$ and $6 \times 6$ grids.
- **Implemented Policy:** The engine implements an explicit extension policy awarding **2 points per letter** for words $\ge 9$ letters (e.g. 9 letters = 18 points, 10 letters = 20 points). This policy is tested in `test_nine_and_longer_extension_policy`.

### B. Random Puzzle Solvability & Minimum Word Thresholds
- **Risk:** Pure uniform random sampling of 7 letters in Spelling Bee can occasionally generate unplayable hives (e.g. 7 rare consonants like `[Z, X, Q, J, K, V, W]` yielding 0 valid words).
- **Implemented Guard:** `SpellingBeePuzzle::random()` retries generation if the resulting hive contains zero vowels, ensuring generated puzzles provide playable solution sets.

### C. Physical Dice Modeling vs. Uniform Letter Sampling (Boggle)
- **Observation:** Physical Boggle sets use 16 (or 25) custom cubic dice where letters are distributed according to English language frequency (e.g., more vowels and common consonants like `E`, `T`, `A`, and fewer `Q`, `Z`, `X`).
- **Current Handling:** The default generator uses uniform pseudo-random sampling across `['A'..='Z']`. An alternative seeded constructor allows deterministic tests and integration of weighted dice distributions.

---

## 5. Concurrency & Parallelism Roadmap

- **Single-Threaded Execution:** The current Trie, CLI, and game engines run sequentially on a single operating system thread.
- **Proposed Enhancement (Rayon Parallelism):** Batch evaluation of large player guess lists in Boggle and board-wide DFS solver searches could be parallelized across starting cells using `rayon::prelude::*`, scaling solver throughput across multiple CPU cores on multi-board tournaments.
