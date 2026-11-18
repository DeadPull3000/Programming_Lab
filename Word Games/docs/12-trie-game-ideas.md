# More games using the existing trie

Start with **Ghost** for a small, playable extension, or **Anagram Rack** for the
closest reuse of the existing Spelling Bee solver.

| Game | Player objective | Trie reuse | Additional logic |
| --- | --- | --- | --- |
| Ghost | Take turns adding one letter to a shared prefix; avoid completing a word of at least four letters. | `TrieCursor::advance` checks legal continuations; `is_word` detects a losing move. `completions` provides examples after a challenge. | Turn tracking, minimum word length, optional minimax computer play. A legal move must have a completion meeting the minimum length. |
| Anagram Rack | Make words from a rack, using each tile once. | Adapt Spelling Bee's cursor DFS, consuming and restoring a `[u8; 26]` letter-frequency array. Missing branches prune the search. | Rack generation, duplicate-letter accounting, scoring, optional blank tiles. Pick a dictionary word first so every rack is solvable. |
| Word Ladder | Turn one word into another by changing one letter per move. | Generate substitutions and use `contains` to validate neighbors, or traverse wildcard positions using cursors. | Breadth-first search for the shortest ladder, parent links, same-length words, difficulty based on path length. |
| Hangman | Guess a hidden word before running out of misses. | Traverse a pattern such as `C?T`; fixed positions take one branch, unknown positions try allowed branches. | Guessed letters, exclusions for missed letters, the full revealed pattern, curated secret words. |
| Crossword Sprint | Solve short crossing words against the clock. | Traverse known-letter patterns; use `contains` for finished answers and prefix pruning while filling entries. | Clues and a curated answer list are separate data. Grid generation and crossing consistency are the harder parts. |
| Word Search | Find words hidden in straight lines in a letter grid. | Walk a cursor along each of eight directions; stop when a prefix disappears and record terminal words. | Place words first, fill spare cells, retain coordinates, allow reverse directions. Unlike Boggle, paths cannot bend. |
| Prefix Race | Find different words beginning with a random prefix before time runs out. | `contains` validates guesses and `completions(prefix, limit)` supplies bounded hints. | Select prefixes with enough qualifying words, prevent duplicate scoring, vary minimum word length. |
| Letter Box | Make a chain of words using letters on four sides of a square. | Cursor DFS restricts choices to a different side from the previous letter. | Chain words by last/first letter, track used-letter bitmasks, search for a short chain covering every letter. |

The new `Trie::completions(prefix, limit)` returns normalized, alphabetically
ordered words and stops at the requested limit. An empty prefix searches the
whole dictionary. For solvers, retain `TrieCursor` traversal so constrained
searches can prune early without copying the dictionary.

Keep game rules and scoring in separate crates, following the current workspace
structure. Reuse the dictionary without modifying it during play: a game's found
words belong in its own set. Use the broad lexicon for validation, and curate
secret words and clues where obscure dictionary entries would hurt playability.
