//! # Autonomous Dictionary Verification Test Suite
//!
//! Autonomous, end-to-end validation testing every game engine in the suite
//! (Trie Library, Spelling Bee, Boggle, Wordle) against the complete 176,413-word
//! dictionary (`dictionary.txt`).
//!
//! Invariants Verified:
//! 1. All 176,413 words in `dictionary.txt` are inserted and indexed into the 26-ary Trie.
//! 2. Case-insensitivity, prefix tracking, and negative rejection hold across the dictionary.
//! 3. Spelling Bee solver and game engine correctly identify valid words, compute scores,
//!    detect pangrams, and reject non-conforming words.
//! 4. Boggle solver discovers 100% valid dictionary words on 4x4, 5x5, and 6x6 boards,
//!    verifying path adjacency and scoring accuracy.
//! 5. Wordle parses all 8,812 five-letter words from the dictionary, validates two-pass
//!    feedback coloring, and simulates autonomous solver rounds.
//! 6. Concurrent multi-threaded access to the shared Trie across games is thread-safe and deterministic.

use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;

use boggle::{
    board::Board,
    game::{BoggleGame, BoggleGuessResult},
    scoring::calculate_score as boggle_score,
    validator::{can_form_word, solve_boggle_board, MIN_BOGGLE_LENGTH},
};
use spelling_bee::{
    game::{GuessResult, SpellingBeeGame},
    puzzle::SpellingBeePuzzle,
    scoring::calculate_score as bee_score,
    solver::{brute_force_reference_solve, solve_spelling_bee, MIN_WORD_LENGTH},
};
use trie::Trie;
use wordle::{
    candidate::CandidateSet,
    dictionary::WordleDictionary,
    feedback::{calculate_feedback, LetterFeedback},
    game::{GameStatus, GuessOutcome, WordleGame},
    solver::{Solver, SolverConfig, SolverStrategy},
    word::{Word, WORD_LENGTH},
};

/// Resolves the absolute path to `dictionary.txt` regardless of test execution directory.
fn locate_dictionary_file() -> PathBuf {
    let candidate_paths = [
        "dictionary.txt",
        "../dictionary.txt",
        "../../dictionary.txt",
        "Resources/dictionary.txt",
        "../Resources/dictionary.txt",
    ];

    for relative in &candidate_paths {
        let path = PathBuf::from(relative);
        if path.exists() {
            return path.canonicalize().unwrap_or(path);
        }
    }

    if let Ok(manifest_dir) = std::env::var("CARGO_MANIFEST_DIR") {
        let path = PathBuf::from(manifest_dir).join("../dictionary.txt");
        if path.exists() {
            return path.canonicalize().unwrap_or(path);
        }
    }

    panic!("Failed to locate dictionary.txt in candidate locations");
}

/// Loads and cleanses all words from `dictionary.txt`.
fn load_raw_dictionary_words() -> Vec<String> {
    let path = locate_dictionary_file();
    let content = fs::read_to_string(&path)
        .unwrap_or_else(|err| panic!("Failed to read dictionary from {path:?}: {err}"));

    content
        .lines()
        .map(|line| line.trim().to_ascii_uppercase())
        .filter(|line| !line.is_empty())
        .collect()
}

/// Builds a populated Trie from all words in `dictionary.txt`.
fn build_full_dictionary_trie(words: &[String]) -> Trie {
    let mut trie = Trie::new();
    for word in words {
        trie.insert(word).unwrap();
    }
    trie
}

// ============================================================================
// TEST 1: TRIE COMPREHENSIVE VALIDATION (ALL 176,413 WORDS)
// ============================================================================
#[test]
fn test_01_dictionary_loading_and_trie_full_validation() {
    let start_load = Instant::now();
    let words = load_raw_dictionary_words();
    let load_duration = start_load.elapsed();

    assert_eq!(
        words.len(),
        176_413,
        "Expected exactly 176,413 words in dictionary.txt, found {}",
        words.len()
    );
    println!(
        "[Trie Test] Loaded {} words from disk in {:.2?}",
        words.len(),
        load_duration
    );

    let start_build = Instant::now();
    let trie = build_full_dictionary_trie(&words);
    let build_duration = start_build.elapsed();

    assert_eq!(
        trie.len(),
        176_413,
        "Trie must contain exactly 176,413 words"
    );
    assert!(!trie.is_empty(), "Trie must not be empty");
    println!(
        "[Trie Test] Inserted {} words into 26-ary Trie in {:.2?}",
        trie.len(),
        build_duration
    );

    // Invariant: EVERY SINGLE word in dictionary.txt must be found in the Trie
    let start_lookup = Instant::now();
    for (idx, word) in words.iter().enumerate() {
        assert!(
            trie.contains(word),
            "Word #{idx} '{word}' must be found in the Trie"
        );

        // Case insensitivity check on sample words
        if idx % 500 == 0 {
            let lower = word.to_ascii_lowercase();
            assert!(
                trie.contains(&lower),
                "Trie contains must be case-insensitive for '{lower}'"
            );
        }

        // Prefix verification
        if word.len() >= 3 && idx % 300 == 0 {
            let prefix = &word[..3];
            assert!(
                trie.has_prefix(prefix),
                "Trie has_prefix must be true for prefix '{prefix}'"
            );
        }
    }
    let lookup_duration = start_lookup.elapsed();
    println!(
        "[Trie Test] Successfully verified all 176,413 words in Trie in {:.2?}",
        lookup_duration
    );

    // Invariant: Non-words and invalid characters must be safely rejected
    assert!(!trie.contains("NONEXISTENTWORDXYZ123"));
    assert!(!trie.contains("ZZZZZZZZZZ"));
    assert!(!trie.contains("A1B2C3"));
    assert!(!trie.has_prefix("QQQQ"));
}

// ============================================================================
// TEST 2: SPELLING BEE AUTONOMOUS VALIDATION AGAINST FULL DICTIONARY
// ============================================================================
#[test]
fn test_02_spelling_bee_autonomous_validation_with_dictionary() {
    let words = load_raw_dictionary_words();
    let trie = build_full_dictionary_trie(&words);

    // Find pangrams in the dictionary to construct real puzzles
    let mut candidate_puzzles: Vec<SpellingBeePuzzle> = Vec::new();

    for word in &words {
        let unique_chars: HashSet<char> = word.chars().collect();
        if unique_chars.len() == 7 && word.len() >= 7 {
            let mut char_vec: Vec<char> = unique_chars.into_iter().collect();
            char_vec.sort_unstable();
            let center = char_vec[0];
            let seven: [char; 7] = [
                char_vec[0],
                char_vec[1],
                char_vec[2],
                char_vec[3],
                char_vec[4],
                char_vec[5],
                char_vec[6],
            ];
            if let Ok(puzzle) = SpellingBeePuzzle::new(seven, center) {
                candidate_puzzles.push(puzzle);
                if candidate_puzzles.len() >= 5 {
                    break;
                }
            }
        }
    }

    assert!(
        !candidate_puzzles.is_empty(),
        "Must be able to find pangram puzzles from dictionary"
    );

    for (p_idx, puzzle) in candidate_puzzles.iter().enumerate() {
        let start_solve = Instant::now();
        let fast_solutions = solve_spelling_bee(&trie, puzzle);
        let solve_duration = start_solve.elapsed();

        let ref_solutions = brute_force_reference_solve(&words, puzzle);
        assert_eq!(
            fast_solutions, ref_solutions,
            "Trie-guided solver must exactly match brute-force reference solve for puzzle #{p_idx}"
        );

        println!(
            "[Spelling Bee Test #{p_idx}] Solved in {:.2?}: {} words discovered (Center: '{}', Letters: {:?})",
            solve_duration,
            fast_solutions.len(),
            puzzle.main_letter,
            puzzle.letters
        );

        // Invariant: 100% of discovered solutions MUST be in the dictionary,
        // length >= 4, contain the center letter, and only use allowed letters.
        for sol in &fast_solutions {
            assert!(
                trie.contains(sol),
                "Discovered word '{sol}' must be in the dictionary"
            );
            assert!(
                sol.len() >= MIN_WORD_LENGTH,
                "Discovered word '{sol}' must have length >= {MIN_WORD_LENGTH}"
            );
            assert!(
                sol.contains(puzzle.main_letter),
                "Discovered word '{sol}' must contain mandatory center letter '{}'",
                puzzle.main_letter
            );
            for ch in sol.chars() {
                assert!(
                    puzzle.contains_letter(ch),
                    "Character '{ch}' in '{sol}' must be in puzzle letters"
                );
            }
        }

        // Invariant: Simulate gameplay with SpellingBeeGame
        let mut game = SpellingBeeGame::new(*puzzle, fast_solutions.clone());
        let mut accumulated_score = 0;

        for sol in &fast_solutions {
            let res = game.submit_guess(sol);
            let mask = puzzle.word_mask(sol).expect("Valid word mask");
            let is_pangram = SpellingBeePuzzle::mask_is_pangram(mask);
            let expected_score = bee_score(sol, is_pangram);

            match res {
                GuessResult::Accepted { points, pangram, word } => {
                    assert_eq!(word, *sol);
                    assert_eq!(points, expected_score);
                    assert_eq!(pangram, is_pangram);
                    accumulated_score += points;
                }
                other => panic!("Expected Accepted for valid solution '{sol}', got: {other:?}"),
            }
        }

        assert_eq!(
            game.score,
            accumulated_score,
            "Game score must equal sum of word scores"
        );
        assert!(game.is_queen_bee(), "Finding all words must achieve Queen Bee status");

        // Verify rejection invariants
        let mut test_game = SpellingBeeGame::new(*puzzle, fast_solutions.clone());
        assert_eq!(test_game.submit_guess("CAT"), GuessResult::TooShort);
        match test_game.submit_guess("NONEXISTENTWORDXYZ") {
            GuessResult::InvalidLetters(_) | GuessResult::MissingMainLetter(_) | GuessResult::NotInSolutions => {}
            other => panic!("Expected rejection for invalid word, got: {other:?}"),
        }
    }
}

// ============================================================================
// TEST 3: BOGGLE AUTONOMOUS VALIDATION (4x4, 5x5, 6x6 GRIDS)
// ============================================================================
#[test]
fn test_03_boggle_autonomous_validation_with_dictionary() {
    let words = load_raw_dictionary_words();
    let trie = build_full_dictionary_trie(&words);

    let test_boards = [
        (4, "PEACESTARLINGSXZ"),
        (4, "CATSDOGSBIRDSTOY"),
        (5, "APPLEBANANACHERRYDATELIME"),
        (6, "ABNORMALBEAUTIFULCREATIVEWONDERFULZZ"),
    ];

    for (size, grid_str) in test_boards {
        let board = Board::new(size, grid_str.chars().collect()).expect("Valid board");
        let start_solve = Instant::now();
        let solutions = solve_boggle_board(&trie, &board);
        let solve_duration = start_solve.elapsed();

        println!(
            "[Boggle Test {size}x{size}] Solved in {:.2?}: {} words discovered",
            solve_duration,
            solutions.len()
        );

        // Invariant: 100% of discovered words MUST be in dictionary, length >= 3,
        // and formable on the board via adjacent moves without tile reuse.
        for word in &solutions {
            assert!(
                trie.contains(word),
                "Boggle word '{word}' must be in dictionary"
            );
            assert!(
                word.len() >= MIN_BOGGLE_LENGTH,
                "Boggle word '{word}' must have length >= {MIN_BOGGLE_LENGTH}"
            );
            assert!(
                can_form_word(&board, word),
                "Boggle word '{word}' must be formable on the board"
            );
        }

        // Invariant: Simulate gameplay and scoring with BoggleGame
        let mut game = BoggleGame::new(board.clone());
        let mut batch_game = BoggleGame::new(board.clone());
        let mut expected_total_score = 0;

        for word in &solutions {
            let res = BoggleGame::evaluate_single_word(
                &game.board,
                &trie,
                &mut game.accepted_words,
                word,
            );

            let score = boggle_score(word.len());
            match res {
                BoggleGuessResult::Accepted { points, word: accepted_w } => {
                    assert_eq!(accepted_w, *word);
                    assert_eq!(points, score);
                    expected_total_score += points;
                }
                other => panic!("Expected Accepted for boggle word '{word}', got: {other:?}"),
            }

            batch_game.add_guess(word);
        }

        assert_eq!(
            game.accepted_words.len(),
            solutions.len(),
            "All discovered solutions must be accepted"
        );

        // Batch evaluation test
        let batch_evals = batch_game.evaluate_batch(&trie);
        assert_eq!(batch_evals.len(), solutions.len());
        assert_eq!(batch_game.score, expected_total_score);
        assert_eq!(batch_game.accepted_words.len(), solutions.len());

        // Duplicate submission check
        if let Some(first) = solutions.iter().next() {
            let res = BoggleGame::evaluate_single_word(
                &game.board,
                &trie,
                &mut game.accepted_words,
                first,
            );
            assert_eq!(
                res,
                BoggleGuessResult::AlreadyGuessed,
                "Duplicate word must be flagged as AlreadyGuessed"
            );
        }

        // Off-board rejection check
        let off_board_res = BoggleGame::evaluate_single_word(
            &game.board,
            &trie,
            &mut game.accepted_words,
            "ZEPHYR",
        );
        assert_eq!(
            off_board_res,
            BoggleGuessResult::NotOnBoard,
            "Off-board dictionary word must be rejected with NotOnBoard"
        );
    }
}

// ============================================================================
// TEST 4: WORDLE AUTONOMOUS VALIDATION (ALL 8,812 FIVE-LETTER WORDS)
// ============================================================================
#[test]
fn test_04_wordle_autonomous_validation_with_dictionary() {
    let words = load_raw_dictionary_words();
    let trie = build_full_dictionary_trie(&words);

    // Extract all 5-letter words from dictionary.txt
    let five_letter_words: Vec<String> = words
        .into_iter()
        .filter(|w| w.len() == WORD_LENGTH)
        .collect();

    assert_eq!(
        five_letter_words.len(),
        8_812,
        "Expected exactly 8,812 5-letter words in dictionary.txt, found {}",
        five_letter_words.len()
    );

    // Invariant: Every 5-letter word must parse into a valid Word struct
    let mut parsed_words: Vec<Word> = Vec::with_capacity(five_letter_words.len());
    for word_str in &five_letter_words {
        let parsed = Word::parse(word_str).unwrap_or_else(|err| {
            panic!("Word::parse failed on dictionary word '{word_str}': {err:?}")
        });
        parsed_words.push(parsed);
    }
    assert_eq!(parsed_words.len(), 8_812);

    println!(
        "[Wordle Test] Successfully parsed all {} 5-letter dictionary words into Word structs",
        parsed_words.len()
    );

    // Invariant: Two-pass feedback algorithm mathematical invariants
    // Test on words with duplicate letters:
    // Secret: SPEED ('S','P','E','E','D')
    // Guess:  ERASE ('E','R','A','S','E')
    let secret = Word::parse("SPEED").unwrap();
    let guess = Word::parse("ERASE").unwrap();
    let fb = calculate_feedback(&secret, &guess);

    // In ERASE vs SPEED:
    // Index 0 ('E'): 'E' is in SPEED, not index 0 -> Present
    // Index 1 ('R'): 'R' not in SPEED -> Absent
    // Index 2 ('A'): 'A' not in SPEED -> Absent
    // Index 3 ('S'): 'S' is in SPEED, not index 3 -> Present
    // Index 4 ('E'): SPEED has 2 Es, index 0 used one, so this second E is Present
    assert_eq!(fb.states[0], LetterFeedback::Present);
    assert_eq!(fb.states[1], LetterFeedback::Absent);
    assert_eq!(fb.states[2], LetterFeedback::Absent);
    assert_eq!(fb.states[3], LetterFeedback::Present);
    assert_eq!(fb.states[4], LetterFeedback::Present);

    // Guess: ALLOY vs Secret: LLAMA
    let secret_llama = Word::parse("LLAMA").unwrap();
    let guess_alloy = Word::parse("ALLOY").unwrap();
    let fb_alloy = calculate_feedback(&secret_llama, &guess_alloy);
    // ALLOY:
    // A at 0 -> Present (matches A in LLAMA)
    // L at 1 -> Correct (exact match)
    // L at 2 -> Present (second L in LLAMA)
    // O at 3 -> Absent
    // Y at 4 -> Absent
    assert_eq!(fb_alloy.states[0], LetterFeedback::Present);
    assert_eq!(fb_alloy.states[1], LetterFeedback::Correct);
    assert_eq!(fb_alloy.states[2], LetterFeedback::Present);
    assert_eq!(fb_alloy.states[3], LetterFeedback::Absent);
    assert_eq!(fb_alloy.states[4], LetterFeedback::Absent);

    // Exact match invariant: all letters Correct
    let exact_fb = calculate_feedback(&secret, &secret);
    assert!(exact_fb.is_all_correct());

    // Invariant: Build WordleDictionary with candidate pool and verify validation
    let dict = WordleDictionary::new(trie, parsed_words[..500].to_vec()).unwrap();
    assert!(dict.contains_word(&secret));
    assert!(dict.contains_word(&guess));

    // Autonomous solver round simulation: 5 games with secrets from dictionary
    let test_secrets = ["CRANE", "SLATE", "PLANT", "BRICK", "GHOST"];
    for secret_str in test_secrets {
        let target = Word::parse(secret_str).unwrap();
        let mut candidates_vec = parsed_words[..250].to_vec();
        if !candidates_vec.contains(&target) {
            candidates_vec.push(target.clone());
        }

        let mut candidate_set = CandidateSet::new(&candidates_vec);
        let mut game = WordleGame::new(target.clone());
        let solver = Solver::new(SolverConfig {
            strategy: SolverStrategy::InformationGain,
            top_k: 3,
        });

        let mut round = 0;
        let mut previous_guesses: Vec<Word> = Vec::new();

        while !game.is_over() && round < 6 {
            round += 1;
            let suggestions = solver
                .suggest(&candidate_set, &candidates_vec, &previous_guesses)
                .expect("Solver suggestion");

            let guess_word = suggestions[0].word.clone();
            previous_guesses.push(guess_word.clone());

            let res = game
                .submit_guess(guess_word.as_str(), dict.trie())
                .expect("Valid guess submission");

            match res {
                GuessOutcome::Accepted { feedback, status, .. } => {
                    if feedback.is_all_correct() || status == GameStatus::Won {
                        break;
                    }

                    // Monotonic candidate reduction invariant
                    let before_count = candidate_set.len();
                    let summary = candidate_set.filter(&guess_word, &feedback);

                    assert!(
                        summary.remaining_count <= before_count,
                        "Candidate set must strictly decrease or stay equal"
                    );
                    assert!(
                        candidate_set.contains(&target),
                        "Target word must NEVER be accidentally pruned"
                    );
                }
                other => panic!("Unexpected guess outcome: {other:?}"),
            }
        }

        println!(
            "[Wordle Test] Autonomous solver solved target '{secret_str}' in {round} attempts"
        );
        assert!(
            game.is_over(),
            "Game must reach terminal state within 6 attempts"
        );
    }
}

// ============================================================================
// TEST 5: CONCURRENT MULTI-GAME DICTIONARY ACCESS STRESS TEST
// ============================================================================
#[test]
fn test_05_concurrent_multi_game_dictionary_stress_test() {
    let words = load_raw_dictionary_words();
    let trie = Arc::new(build_full_dictionary_trie(&words));

    let mut handles = Vec::new();

    // Spawn 8 concurrent threads executing different game queries against the same Trie
    for thread_id in 0..8 {
        let trie_clone = Arc::clone(&trie);
        let handle = std::thread::spawn(move || {
            let start = Instant::now();
            let mut checks = 0;

            if thread_id % 2 == 0 {
                // Spelling Bee worker
                let puzzle = SpellingBeePuzzle::new(
                    ['E', 'C', 'O', 'M', 'P', 'L', 'X'],
                    'E'
                ).unwrap();
                let solutions = solve_spelling_bee(&trie_clone, &puzzle);
                assert!(!solutions.is_empty());
                checks += solutions.len();
            } else {
                // Boggle worker
                let board = Board::new(4, "TESTINGBOGGLEFUN".chars().collect()).unwrap();
                let solutions = solve_boggle_board(&trie_clone, &board);
                assert!(!solutions.is_empty());
                checks += solutions.len();
            }

            (thread_id, checks, start.elapsed())
        });
        handles.push(handle);
    }

    for handle in handles {
        let (id, checks, elapsed) = handle.join().expect("Thread joined cleanly");
        println!(
            "[Concurrency Test] Worker thread {id} verified {checks} words in {:.2?}",
            elapsed
        );
    }
}
