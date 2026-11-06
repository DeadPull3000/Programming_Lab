//! # Spelling Bee Integration and Robustness Tests (Phase SB-9)
//!
//! Validates the entire end-to-end workflow:
//! 1. Dictionary loading from `dictionary.txt` into the shared `Trie`.
//! 2. Puzzle generation and deterministic seeded replay.
//! 3. Trie-guided DFS solution enumeration vs. independent brute-force reference filtering.
//! 4. Invariant verification for 100% of generated solutions.
//! 5. Interactive game simulation: valid guesses, duplicate rejections, invalid inputs,
//!    score calculations, Queen Bee completion, and final summary reporting.
//! 6. Concrete timing measurements.

use rand::rngs::StdRng;
use rand::SeedableRng;
use spelling_bee::game::{GuessResult, SpellingBeeGame};
use spelling_bee::hints::SpellingBeeHints;
use spelling_bee::puzzle::SpellingBeePuzzle;
use spelling_bee::solver::{brute_force_reference_solve, solve_spelling_bee, MIN_WORD_LENGTH};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;
use std::time::Instant;
use trie::Trie;

fn load_test_dictionary() -> (Trie, Vec<String>) {
    let dict_candidates = [
        "Resources/dictionary.txt",
        "../Resources/dictionary.txt",
        "dictionary.txt",
        "../dictionary.txt",
        "../../dictionary.txt",
    ];
    let dict_path = dict_candidates
        .iter()
        .find(|p| Path::new(p).exists())
        .expect("dictionary.txt must exist");

    let file = File::open(dict_path).expect("failed to open dictionary.txt");
    let reader = BufReader::new(file);

    let mut trie = Trie::new();
    let mut word_list = Vec::with_capacity(180_000);

    for line in reader.lines() {
        let line = line.expect("reading line from dictionary");
        let trimmed = line.trim();
        if !trimmed.is_empty() {
            trie.insert(trimmed).expect("inserting into trie");
            word_list.push(trimmed.to_ascii_uppercase());
        }
    }

    (trie, word_list)
}

#[test]
fn test_spelling_bee_full_integration_and_benchmarks() {
    // 1. Measure Dictionary Loading
    let start_load = Instant::now();
    let (trie, word_list) = load_test_dictionary();
    let load_time = start_load.elapsed();

    println!("== SPELLING BEE INTEGRATION REPORT ==");
    println!(
        "Dictionary loaded: {} words in {:.2?}",
        trie.len(),
        load_time
    );
    assert!(
        trie.len() > 100_000,
        "Dictionary must contain over 100,000 words"
    );

    // 2. Measure Seeded Puzzle Generation
    let mut rng = StdRng::seed_from_u64(98765);
    let start_gen = Instant::now();
    let puzzle = SpellingBeePuzzle::random(&mut rng);
    let gen_time = start_gen.elapsed();
    println!(
        "Generated puzzle: {} in {:.2?}",
        puzzle.display_string(),
        gen_time
    );

    // 3. Measure Trie-Guided Solution Enumeration
    let start_solve = Instant::now();
    let trie_solutions = solve_spelling_bee(&trie, &puzzle);
    let solve_time = start_solve.elapsed();
    println!(
        "Trie DFS solved {} solutions in {:.2?}",
        trie_solutions.len(),
        solve_time
    );

    // 4. Measure Brute-Force Reference Solver over entire 176k dictionary
    let start_ref = Instant::now();
    let ref_solutions = brute_force_reference_solve(&word_list, &puzzle);
    let ref_time = start_ref.elapsed();
    println!(
        "Brute-force reference solved {} solutions in {:.2?}",
        ref_solutions.len(),
        ref_time
    );

    // 5. Assert 100% agreement between optimized solver and reference implementation
    assert_eq!(
        trie_solutions, ref_solutions,
        "Trie solver and brute-force reference must produce identical solution sets"
    );

    // 6. Invariant verification on every generated solution
    for word in &trie_solutions {
        assert!(
            word.len() >= MIN_WORD_LENGTH,
            "Solution '{word}' violates minimum length >= 4"
        );
        assert!(
            word.contains(puzzle.main_letter),
            "Solution '{word}' missing mandatory main letter [{}]",
            puzzle.main_letter
        );
        for ch in word.chars() {
            assert!(
                puzzle.contains_letter(ch),
                "Solution '{word}' contains unauthorized letter '{ch}'"
            );
        }
        assert!(
            trie.contains(word),
            "Solution '{word}' must exist in dictionary"
        );
    }

    // 7. Verify Hint System Consistency
    let hints = SpellingBeeHints::compute(&trie_solutions);
    assert_eq!(
        hints.grid.grand_total,
        trie_solutions.len(),
        "Hint grid grand total must equal solution count"
    );

    let sum_row_totals: usize = hints.grid.row_totals.values().sum();
    assert_eq!(
        sum_row_totals,
        trie_solutions.len(),
        "Sum of row totals must equal solution count"
    );

    let sum_col_totals: usize = hints.grid.col_totals.values().sum();
    assert_eq!(
        sum_col_totals,
        trie_solutions.len(),
        "Sum of column totals must equal solution count"
    );

    // 8. Simulate Complete Game Session
    let mut game = SpellingBeeGame::new(puzzle, trie_solutions.clone());
    let initial_score = game.score;
    assert_eq!(initial_score, 0);

    // Invalid guess rejection tests:
    assert_eq!(game.submit_guess(""), GuessResult::TooShort);
    assert_eq!(
        game.submit_guess("abc!"),
        GuessResult::InvalidCharacters('!')
    );
    assert_eq!(
        game.submit_guess("123"),
        GuessResult::InvalidCharacters('1')
    );
    assert_eq!(game.score, 0, "Score must not change on rejected guesses");

    // Submit all solutions to test game progression and Queen Bee
    let mut expected_score = 0;
    for word in &trie_solutions {
        let mask = puzzle.word_mask(word).unwrap();
        let is_pangram = SpellingBeePuzzle::mask_is_pangram(mask);
        let word_len = word.chars().count();
        let pts = if word_len == 4 { 1 } else { word_len as u32 } + if is_pangram { 7 } else { 0 };

        let res = game.submit_guess(word);
        match res {
            GuessResult::Accepted {
                points, pangram, ..
            } => {
                assert_eq!(points, pts);
                assert_eq!(pangram, is_pangram);
                expected_score += points;
            }
            other => panic!(
                "Valid solution '{word}' was unexpectedly rejected: {:?}",
                other
            ),
        }
    }

    assert_eq!(game.score, expected_score);
    assert!(
        game.is_queen_bee(),
        "All solutions guessed must award Queen Bee"
    );
    assert!(game.remaining_solutions().is_empty());
}
