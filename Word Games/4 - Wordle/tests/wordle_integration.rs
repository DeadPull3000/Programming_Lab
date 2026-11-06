//! # End-to-End Wordle Integration Tests
//!
//! Tests complete gameplay sessions, victory, loss, duplicate rejection,
//! dictionary validation, and replay state isolation using fixed secret words.

use wordle::dictionary::WordleDictionary;
use wordle::game::{GameStatus, GuessOutcome, WordleGame};
use wordle::keyboard::{KeyStatus, KeyboardState};
use wordle::word::Word;

fn load_test_dictionary() -> WordleDictionary {
    WordleDictionary::load_default().expect("default dictionary must load in workspace")
}

#[test]
fn test_curated_answers_file_integrity() {
    let dict = load_test_dictionary();
    assert!(!dict.answers().is_empty(), "Answer pool must not be empty");

    // Check that every answer is 5 letters and exists in the Trie
    for answer in dict.answers() {
        assert_eq!(answer.as_str().len(), 5);
        assert!(
            dict.contains_word(answer),
            "Curated answer {} must be in accepted guesses Trie",
            answer.as_str()
        );
    }
}

#[test]
fn test_simulated_winning_round() {
    let dict = load_test_dictionary();
    let answer = Word::parse("SLATE").unwrap();
    let mut game = WordleGame::new(answer.clone());
    let mut kb = KeyboardState::new();

    // Guess 1: CRANE
    let outcome1 = game.submit_guess("CRANE", dict.trie()).unwrap();
    match outcome1 {
        GuessOutcome::Accepted {
            feedback,
            attempts_remaining,
            status,
        } => {
            assert_eq!(attempts_remaining, 5);
            assert_eq!(status, GameStatus::InProgress);
            kb.update_from_guess(&Word::parse("CRANE").unwrap(), &feedback);
        }
        _ => panic!("Expected Accepted guess"),
    }

    assert_eq!(kb.status_of('C'), KeyStatus::Absent);
    assert_eq!(kb.status_of('R'), KeyStatus::Absent);
    assert_eq!(kb.status_of('A'), KeyStatus::Correct); // A in SLATE and CRANE at pos 2
    assert_eq!(kb.status_of('N'), KeyStatus::Absent);
    assert_eq!(kb.status_of('E'), KeyStatus::Correct); // E in SLATE and CRANE at pos 4

    // Guess 2: SLATE (winning guess)
    let outcome2 = game.submit_guess("SLATE", dict.trie()).unwrap();
    match outcome2 {
        GuessOutcome::Accepted {
            feedback,
            attempts_remaining,
            status,
        } => {
            assert!(feedback.is_all_correct());
            assert_eq!(attempts_remaining, 4);
            assert_eq!(status, GameStatus::Won);
            kb.update_from_guess(&Word::parse("SLATE").unwrap(), &feedback);
        }
        _ => panic!("Expected Accepted guess"),
    }

    assert!(game.is_over());
    assert_eq!(game.status(), GameStatus::Won);
    assert_eq!(game.attempts_used(), 2);
    assert_eq!(kb.status_of('S'), KeyStatus::Correct);
    assert_eq!(kb.status_of('L'), KeyStatus::Correct);
    assert_eq!(kb.status_of('T'), KeyStatus::Correct);
}

#[test]
fn test_simulated_losing_round() {
    let dict = load_test_dictionary();
    let answer = Word::parse("APPLE").unwrap();
    let mut game = WordleGame::new(answer);

    let wrong_words = ["CRANE", "SLATE", "ROAST", "TRAIN", "CLOUD", "BRICK"];
    for (i, &w) in wrong_words.iter().enumerate() {
        let outcome = game.submit_guess(w, dict.trie()).unwrap();
        match outcome {
            GuessOutcome::Accepted {
                attempts_remaining,
                status,
                ..
            } => {
                assert_eq!(attempts_remaining, 5 - i);
                if i == 5 {
                    assert_eq!(status, GameStatus::Lost);
                } else {
                    assert_eq!(status, GameStatus::InProgress);
                }
            }
            _ => panic!("Expected accepted guess for {w}"),
        }
    }

    assert_eq!(game.status(), GameStatus::Lost);
    assert!(game.is_over());
    assert_eq!(game.attempts_remaining(), 0);
    assert_eq!(game.attempts_used(), 6);
}

#[test]
fn test_invalid_input_and_duplicates_do_not_penalize() {
    let dict = load_test_dictionary();
    let answer = Word::parse("WATER").unwrap();
    let mut game = WordleGame::new(answer);

    // Initial attempts: 6
    assert_eq!(game.attempts_remaining(), 6);

    // 1. Invalid short length
    assert!(game.submit_guess("CAT", dict.trie()).is_err());
    assert_eq!(game.attempts_remaining(), 6);

    // 2. Invalid character
    assert!(game.submit_guess("WAT3R", dict.trie()).is_err());
    assert_eq!(game.attempts_remaining(), 6);

    // 3. Valid guess
    let res = game.submit_guess("ROAST", dict.trie()).unwrap();
    assert!(matches!(res, GuessOutcome::Accepted { .. }));
    assert_eq!(game.attempts_remaining(), 5);

    // 4. Duplicate guess
    let dup_res = game.submit_guess("roast", dict.trie()).unwrap();
    assert_eq!(dup_res, GuessOutcome::AlreadyGuessed);
    assert_eq!(game.attempts_remaining(), 5); // Still 5!
}

#[test]
fn test_replay_state_isolation() {
    let dict = load_test_dictionary();

    // Round 1
    let ans1 = Word::parse("FIRST").unwrap();
    let mut game1 = WordleGame::new(ans1);
    let mut kb1 = KeyboardState::new();
    let _ = game1.submit_guess("FIRST", dict.trie());
    kb1.update_from_guess(&Word::parse("FIRST").unwrap(), &game1.guesses()[0].feedback);

    assert_eq!(game1.status(), GameStatus::Won);
    assert_eq!(kb1.status_of('F'), KeyStatus::Correct);

    // Round 2 (Fresh independent round)
    let ans2 = Word::parse("BRAIN").unwrap();
    let game2 = WordleGame::new(ans2);
    let kb2 = KeyboardState::new();

    assert_eq!(game2.status(), GameStatus::InProgress);
    assert_eq!(game2.attempts_used(), 0);
    assert_eq!(game2.attempts_remaining(), 6);
    assert_eq!(kb2.status_of('F'), KeyStatus::Unknown); // Completely isolated!
}
