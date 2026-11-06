//! # Boggle Integration and Robustness Tests
//!
//! Validates:
//! 1. Full dictionary integration with Boggle boards (4x4, 5x5, 6x6).
//! 2. Trie-guided automatic solver execution and timing.
//! 3. Property verification for 100% of discovered words (length, dictionary membership, path validity).
//! 4. Batch evaluation with mixed delimiters, duplicate handling, and score verification.
//! 5. Qu tile mechanics.

use boggle::board::Board;
use boggle::game::{BoggleGame, BoggleGuessResult};
use boggle::scoring::calculate_score;
use boggle::validator::{
    can_form_word, can_form_word_with_qu, solve_boggle_board, MIN_BOGGLE_LENGTH,
};
use rand::rngs::StdRng;
use rand::SeedableRng;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;
use std::time::Instant;
use trie::Trie;

fn load_test_dictionary() -> Trie {
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
    for line in reader.lines() {
        let line = line.expect("reading dictionary line");
        let trimmed = line.trim();
        if !trimmed.is_empty() {
            trie.insert(trimmed).expect("inserting into trie");
        }
    }
    trie
}

#[test]
fn test_boggle_full_integration_all_dimensions() {
    let start_load = Instant::now();
    let trie = load_test_dictionary();
    let load_time = start_load.elapsed();

    println!("== BOGGLE INTEGRATION REPORT ==");
    println!(
        "Dictionary loaded: {} words in {:.2?}",
        trie.len(),
        load_time
    );

    let mut rng = StdRng::seed_from_u64(54321);

    for &size in &[4, 5, 6] {
        let board = Board::random(size, &mut rng).unwrap();
        println!("\nTesting {}x{} Boggle Board:", size, size);

        let start_solve = Instant::now();
        let solutions = solve_boggle_board(&trie, &board);
        let solve_time = start_solve.elapsed();

        println!(
            "  Automatic solver found {} valid words in {:.2?}",
            solutions.len(),
            solve_time
        );

        // Verify properties for every single discovered word
        for word in &solutions {
            assert!(
                word.len() >= MIN_BOGGLE_LENGTH,
                "Word '{word}' violates minimum length >= 3"
            );
            assert!(
                trie.contains(word),
                "Word '{word}' must exist in dictionary"
            );
            assert!(
                can_form_word(&board, word),
                "Word '{word}' must be formable along adjacent board cells"
            );
        }
    }
}

#[test]
fn test_boggle_batch_round_simulation() {
    let trie = load_test_dictionary();
    let cells = vec![
        'P', 'E', 'A', 'C', 'L', 'A', 'T', 'E', 'R', 'O', 'S', 'E', 'D', 'U', 'C', 'K',
    ];
    let board = Board::new(4, cells).unwrap();
    let mut game = BoggleGame::new(board.clone());

    let mut available: Vec<String> = solve_boggle_board(&trie, &board).into_iter().collect();
    available.sort();
    assert!(
        available.len() >= 4,
        "Board must have at least 4 valid words"
    );

    let w1 = &available[0];
    let w2 = &available[1];
    let w3 = &available[2];
    let w4 = &available[3];

    // Add inputs via mixed delimiters
    game.add_input_line(&format!("{}, {}, {}", w1, w2, w3));
    game.add_input_line(&format!("{}, {}", w4, w1.to_ascii_lowercase())); // w1 duplicate
    game.add_input_line("XY, 123!, ZZZZZ"); // XY too short, 123! invalid, ZZZZZ not on board

    let results = game.evaluate_batch(&trie);
    assert!(!game.is_active);

    let accepted_words: Vec<&str> = results
        .iter()
        .filter_map(|e| {
            if let BoggleGuessResult::Accepted { word, .. } = &e.result {
                Some(word.as_str())
            } else {
                None
            }
        })
        .collect();

    assert!(accepted_words.contains(&w1.as_str()));
    assert!(accepted_words.contains(&w2.as_str()));
    assert!(accepted_words.contains(&w3.as_str()));
    assert!(accepted_words.contains(&w4.as_str()));

    // Check duplicate was caught
    let duplicates = results
        .iter()
        .filter(|e| e.result == BoggleGuessResult::AlreadyGuessed)
        .count();
    assert_eq!(
        duplicates, 1,
        "Duplicate w1 must be rejected as AlreadyGuessed"
    );

    // Total score check: must match sum of individual accepted scores
    let expected_score: u32 = [w1, w2, w3, w4]
        .iter()
        .map(|w| calculate_score(w.len()))
        .sum();
    assert_eq!(game.score, expected_score);
}

#[test]
fn test_boggle_qu_tile_and_scoring_integration() {
    let trie = load_test_dictionary();
    let cells = vec![
        'Q', 'I', 'T', 'E', 'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L',
    ];
    let board = Board::new(4, cells).unwrap();

    // "QUITE" starts at 'Q' matching "QU", then 'I', 'T', 'E'
    assert!(can_form_word_with_qu(&board, "QUITE"));
    assert!(trie.contains("QUITE"));

    // Verify 5-letter word scores 2 points
    assert_eq!(calculate_score(5), 2);
}
