# 02. System Architecture & Component Interaction

This document specifies the software architecture, crate boundaries, data flow pipelines, module encapsulation, and execution workflows across the entire Word Games workspace.

---

## 1. High-Level Workspace Architecture

The system is engineered as a unified Cargo virtual workspace comprising four specialized packages. The core data structure (`trie`) is decoupled into an autonomous, zero-dependency library crate consumed by three distinct downstream applications.

```mermaid
graph TD
    Dict["Resources/dictionary.txt<br/>(176,413 Raw Words)"] -->|Loaded by IO Reader| TrieLib["Trie Library (1 - Library Trie/trie)<br/>- 26-way Sparse Node Hierarchy<br/>- O(L) Insert, Lookup, Delete<br/>- Zero-Alloc TrieCursor API"]

    TrieLib -->|Crate Dependency| TrieCLI["trie-cli (1 - Library Trie/trie-cli)<br/>- Interactive REPL<br/>- Batch Loader & Splitter"]
    TrieLib -->|Crate Dependency| SB["Spelling Bee (2 - Spelling Bee)<br/>- 7-Letter Hexagonal Puzzle<br/>- TrieCursor DFS Precomputing Solver<br/>- 7-Bit Bitmask Letter Validator"]
    TrieLib -->|Crate Dependency| Boggle["Boggle (3 - Boggle)<br/>- 4x4, 5x5, 6x6 Coordinate Grid<br/>- Visited-Array Path DFS<br/>- Automatic Board Trie Solver"]

    SB -->|Precomputed Solutions| SBGame["SpellingBeeGame State<br/>- Guessed Words (BTreeSet)<br/>- Score Accumulator<br/>- Queen Bee Check"]
    Boggle -->|Batch Guesses| BoggleGame["BoggleGame State<br/>- Evaluated Guesses<br/>- Score Accumulator<br/>- Missed Words Finder"]
```

---

## 2. Architectural Design Rationale

### Why the Trie is a Separate Library Crate
1. **Separation of Concerns:** The Trie is an abstract dictionary engine. It has no intrinsic knowledge of word puzzle rules, board geometries, or scoring formulas. Decoupling it into `trie` allows independent evolution, testing, and optimization without triggering recompilation of game code.
2. **Reusability Across Game Engines:** Both Spelling Bee and Boggle require fast prefix queries and dictionary validation. By referencing `trie = { path = "../1 - Library Trie/trie" }`, both games share identical data structure semantics and memory representations.
3. **Encapsulation via `TrieCursor`:** The internal node tree (`pub(crate) struct Node`) contains raw recursive pointers (`[Option<Box<Node>>; 26]`). To prevent client crates from tampering with node linkages or causing memory corruption, traversal is mediated exclusively through a lightweight, read-only `TrieCursor<'a>` bound to the Trie's lifetime.
4. **Standard Library Purity:** The core `trie` crate has zero external dependencies. It relies strictly on Rust's `core` and `alloc` abstractions, maximizing portability and compile-time efficiency.

---

## 3. Dictionary Ingestion & Fallback Path Resolution

To guarantee seamless execution regardless of the terminal's current working directory (whether launched from the workspace root, within a sub-package directory, or via cargo test runners), all applications utilize a multi-tier fallback path resolution:

```rust
const CANDIDATE_PATHS: &[&str] = &[
    "Resources/dictionary.txt",
    "../Resources/dictionary.txt",
    "../../Resources/dictionary.txt",
    "dictionary.txt",
    "../dictionary.txt",
    "../../dictionary.txt",
];
```

The loader iterates sequentially through `CANDIDATE_PATHS`, opening the first existing file via `std::fs::File::open`. An independent buffered reader (`std::io::BufReader`) processes the stream line-by-line, normalizing ASCII characters and populating the Trie.

---

## 4. Source File Inventory & Architectural Responsibilities

| File Path | Architectural Responsibility | Key Structs, Enums & Functions |
| :--- | :--- | :--- |
| **`1 - Library Trie/trie/src/lib.rs`** | Core 26-way prefix tree engine and cursor traversal API | `struct Trie`, `struct Node`, `struct TrieCursor<'a>`, `enum TrieError`, `Trie::insert`, `Trie::contains`, `Trie::remove`, `Trie::has_prefix`, `Trie::root_cursor` |
| **`1 - Library Trie/trie/tests/differential_testing.rs`** | Randomized metamorphic testing against `BTreeSet` | `test_differential_randomized_operations`, `test_alphabet_boundary_extremities` |
| **`1 - Library Trie/trie-cli/src/main.rs`** | Interactive REPL command dispatch and session loop | `main()`, `process_command()`, `Command` parser |
| **`1 - Library Trie/trie-cli/src/batch.rs`** | High-throughput file and token stream parsing | `parse_tokens()`, `load_words_from_reader()`, `check_words_from_reader()` |
| **`2 - Spelling Bee/src/lib.rs`** | Crate root exposing public game engine abstractions | Module re-exports: `SpellingBeeGame`, `SpellingBeePuzzle`, `solve_spelling_bee` |
| **`2 - Spelling Bee/src/puzzle.rs`** | 7-letter puzzle generation, invariants, and bitmask tracking | `struct SpellingBeePuzzle`, `FULL_PANGRAM_MASK`, `MAIN_LETTER_BIT`, `bitmask_for_word()` |
| **`2 - Spelling Bee/src/solver.rs`** | Trie-guided recursive DFS solution precomputation | `solve_spelling_bee()`, `dfs_explore()`, `brute_force_reference_solve()` |
| **`2 - Spelling Bee/src/scoring.rs`** | NYT-compliant scoring rules and pangram bonus evaluation | `calculate_score()`, `PANGRAM_BONUS_POINTS = 7` |
| **`2 - Spelling Bee/src/game.rs`** | Game state machine, guess evaluation, and victory detection | `struct SpellingBeeGame`, `enum GuessResult`, `guess()`, `is_queen_bee()`, `missed_words()` |
| **`2 - Spelling Bee/src/hints.rs`** | Hint grid calculation and 2-letter prefix frequency matrix | `struct SpellingBeeHints`, `struct HintGrid`, `struct TwoLetterPrefixes` |
| **`2 - Spelling Bee/src/main.rs`** | Interactive terminal game loop and 3-column formatter | `main()`, `run_game()`, `format_three_columns()` |
| **`3 - Boggle/src/lib.rs`** | Crate root exposing board, scoring, and solver abstractions | Module re-exports: `Board`, `BoggleGame`, `solve_boggle_board` |
| **`3 - Boggle/src/board.rs`** | NxN grid coordinate geometry and neighbor adjacency lists | `struct Board`, `VALID_DIMENSIONS = [4, 5, 6]`, `DIRECTIONS`, `neighbors_of()` |
| **`3 - Boggle/src/validator.rs`** | Recursive DFS pathfinding and automated Trie board solver | `can_form_word()`, `can_form_word_with_qu()`, `solve_boggle_board()` |
| **`3 - Boggle/src/scoring.rs`** | Official Boggle length scoring and 9+ letter extension policy | `calculate_score()` |
| **`3 - Boggle/src/game.rs`** | Batch evaluation pipeline and round state machine | `struct BoggleGame`, `enum BoggleGuessResult`, `evaluate_batch()` |
| **`3 - Boggle/src/timer.rs`** | Optional non-blocking match countdown timer | `struct BoggleTimer`, `is_expired()`, `remaining_seconds()` |
| **`3 - Boggle/src/main.rs`** | Interactive CLI game loop, setup menus, and batch input | `main()`, `run_game_loop()`, `parse_input_tokens()` |

---

## 5. Execution Flow & Lifecycle Pipelines

### A. Spelling Bee Execution Flow
```mermaid
sequenceDiagram
    autonumber
    actor Player
    participant CLI as Spelling Bee CLI
    participant Dict as Dictionary Loader
    participant Puzzle as SpellingBeePuzzle
    participant Solver as Trie DFS Solver
    participant Game as SpellingBeeGame State

    CLI->>Dict: Load dictionary.txt into shared Trie
    CLI->>Puzzle: Generate 7 distinct letters (Sample without replacement)
    Puzzle-->>CLI: Verified Puzzle (e.g., Center: C, Hive: A, I, M, O, R, T)
    CLI->>Solver: solve_spelling_bee(&trie, &puzzle)
    Note over Solver: Traverses TrieCursor along 7 letters only<br/>Prunes invalid branches in ~208 us
    Solver-->>CLI: Complete Solution Set (HashSet of ~60-150 words)
    CLI->>Game: Instantiate SpellingBeeGame::new(puzzle, solutions)

    loop Interactive Play Loop
        Player->>CLI: Enter Guess / Hint / List / Quit
        CLI->>Game: game.guess(raw_input)
        Game-->>CLI: GuessResult (Accepted, Duplicate, MissingCenter, InvalidLetters, TooShort, NotInDictionary)
        CLI-->>Player: Display updated score, guessed words (3 columns)
    end

    Player->>CLI: :quit
    CLI->>Game: game.missed_words()
    CLI-->>Player: Print all undiscovered solutions sorted by length and alphabet
```

### B. Boggle Execution Flow
```mermaid
sequenceDiagram
    autonumber
    actor Player
    participant CLI as Boggle CLI
    participant Dict as Dictionary Loader
    participant Board as Board (4x4, 5x5, 6x6)
    participant Game as BoggleGame
    participant Validator as Path DFS & Trie Validator
    participant AutoSolver as Automatic Board Solver

    CLI->>Dict: Load dictionary.txt into shared Trie
    CLI->>Board: Initialize Grid (Random letters / Seeded)
    CLI->>Game: Instantiate BoggleGame::new(board, optional_timer)
    CLI-->>Player: Display Board Grid & Start Timer

    Player->>CLI: Enter Batch Guesses (e.g. "CART, TEAM, ROAST, BOGGLE")
    CLI->>Game: game.evaluate_batch(raw_string, &trie)

    loop For each token in batch
        Game->>Validator: can_form_word(board, word) [Path DFS]
        alt Path does not exist on board
            Validator-->>Game: Rejected: NotOnBoard
        else Path exists
            Game->>Validator: is_dictionary_word(trie, word)
            alt In dictionary
                Validator-->>Game: Accepted (Score points)
            else Not in dictionary
                Validator-->>Game: Rejected: NotInDictionary
            end
        end
    end

    Game-->>CLI: EvaluatedGuess array (Points, Reasons)
    CLI-->>Player: Detailed Score Report
    CLI->>AutoSolver: solve_boggle_board(&board, &trie)
    AutoSolver-->>CLI: All possible board words (~200-800 words in < 1 ms)
    CLI-->>Player: Print Missed Words & Game Summary
```
