//! # Spelling Bee Game State and Guess Evaluation
//!
//! Encapsulates game state management, strict multi-stage guess validation,
//! score progression, Queen Bee achievement detection, and remaining word reporting.

use crate::puzzle::SpellingBeePuzzle;
use crate::scoring::calculate_score;
use crate::solver::MIN_WORD_LENGTH;
use std::collections::HashSet;

/// Outcome of submitting a player guess.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GuessResult {
    /// Guess was valid, previously unguessed, and exists in the solution set.
    Accepted {
        points: u32,
        pangram: bool,
        word: String,
    },
    /// The guess was shorter than the minimum required length (4 letters).
    TooShort,
    /// The guess contained a non-alphabetic character.
    InvalidCharacters(char),
    /// The guess did not include the mandatory main letter.
    MissingMainLetter(char),
    /// The guess used a letter that does not belong to the 7-letter puzzle.
    InvalidLetters(char),
    /// The word is not in the precomputed dictionary solution set.
    NotInSolutions,
    /// The word was already successfully guessed in this game session.
    AlreadyGuessed,
}

/// Active game session for a Spelling Bee puzzle.
#[derive(Debug, Clone)]
pub struct SpellingBeeGame {
    /// The 7-letter puzzle with designated main letter.
    pub puzzle: SpellingBeePuzzle,
    /// Precomputed full set of valid dictionary solutions.
    pub solutions: HashSet<String>,
    /// Set of words successfully guessed by the player.
    pub guessed: HashSet<String>,
    /// Accumulated player score.
    pub score: u32,
}

impl SpellingBeeGame {
    /// Creates a new game instance for a given puzzle and its precomputed solution set.
    pub fn new(puzzle: SpellingBeePuzzle, solutions: HashSet<String>) -> Self {
        Self {
            puzzle,
            solutions,
            guessed: HashSet::new(),
            score: 0,
        }
    }

    /// Evaluates a raw player guess according to the strict validation sequence.
    ///
    /// State is mutated (score increased, word recorded) only if the guess is `Accepted`.
    pub fn submit_guess(&mut self, raw_input: &str) -> GuessResult {
        // 1. Trim whitespace
        let trimmed = raw_input.trim();

        // 2 & 3. Validate characters and normalize to uppercase
        let mut normalized = String::with_capacity(trimmed.len());
        for ch in trimmed.chars() {
            if ch.is_ascii_alphabetic() {
                normalized.push(ch.to_ascii_uppercase());
            } else {
                return GuessResult::InvalidCharacters(ch);
            }
        }

        // 4. Check minimum word length
        if normalized.len() < MIN_WORD_LENGTH {
            return GuessResult::TooShort;
        }

        // 5. Check permitted letters
        for ch in normalized.chars() {
            if !self.puzzle.contains_letter(ch) {
                return GuessResult::InvalidLetters(ch);
            }
        }

        // 6. Check main letter inclusion
        if !normalized.contains(self.puzzle.main_letter) {
            return GuessResult::MissingMainLetter(self.puzzle.main_letter);
        }

        // 7. Check precomputed solutions
        if !self.solutions.contains(&normalized) {
            return GuessResult::NotInSolutions;
        }

        // 8. Check whether already guessed
        if self.guessed.contains(&normalized) {
            return GuessResult::AlreadyGuessed;
        }

        // 9. Add valid guess and award points
        let mask = self
            .puzzle
            .word_mask(&normalized)
            .expect("letters already verified");
        let is_pangram = SpellingBeePuzzle::mask_is_pangram(mask);
        let points = calculate_score(&normalized, is_pangram);

        self.score += points;
        self.guessed.insert(normalized.clone());

        GuessResult::Accepted {
            points,
            pangram: is_pangram,
            word: normalized,
        }
    }

    /// Returns `true` if all possible solutions have been discovered.
    #[inline]
    pub fn is_queen_bee(&self) -> bool {
        !self.solutions.is_empty() && self.guessed.len() == self.solutions.len()
    }

    /// Returns all undiscovered solutions sorted alphabetically.
    pub fn remaining_solutions(&self) -> Vec<String> {
        let mut remaining: Vec<String> =
            self.solutions.difference(&self.guessed).cloned().collect();
        remaining.sort();
        remaining
    }

    /// Returns the player's guessed words sorted alphabetically.
    pub fn sorted_guessed(&self) -> Vec<String> {
        let mut list: Vec<String> = self.guessed.iter().cloned().collect();
        list.sort();
        list
    }

    /// Total number of possible solutions.
    #[inline]
    pub fn total_solutions(&self) -> usize {
        self.solutions.len()
    }

    /// Number of solutions guessed so far.
    #[inline]
    pub fn count_guessed(&self) -> usize {
        self.guessed.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_test_game() -> SpellingBeeGame {
        let letters = ['R', 'A', 'T', 'E', 'C', 'L', 'P'];
        let puzzle = SpellingBeePuzzle::new(letters, 'R').unwrap();
        let mut solutions = HashSet::new();
        solutions.insert("RATE".to_string());
        solutions.insert("RATER".to_string());
        solutions.insert("PLACATER".to_string()); // Pangram containing R, A, T, E, C, L, P
        SpellingBeeGame::new(puzzle, solutions)
    }

    #[test]
    fn test_valid_and_duplicate_guesses() {
        let mut game = setup_test_game();

        // 1. Valid guess: "RATE" (len 4 -> 1 pt)
        let res1 = game.submit_guess("rate");
        assert_eq!(
            res1,
            GuessResult::Accepted {
                points: 1,
                pangram: false,
                word: "RATE".to_string(),
            }
        );
        assert_eq!(game.score, 1);
        assert_eq!(game.count_guessed(), 1);

        // 2. Duplicate guess rejected with no score change
        let res_dup = game.submit_guess("RATE");
        assert_eq!(res_dup, GuessResult::AlreadyGuessed);
        assert_eq!(game.score, 1);
        assert_eq!(game.count_guessed(), 1);

        // 3. Pangram guess: "PLACATER" (len 8 + 7 bonus = 15 pts)
        let res2 = game.submit_guess("placater");
        assert_eq!(
            res2,
            GuessResult::Accepted {
                points: 15,
                pangram: true,
                word: "PLACATER".to_string(),
            }
        );
        assert_eq!(game.score, 16);
        assert_eq!(game.count_guessed(), 2);
    }

    #[test]
    fn test_rejection_conditions_do_not_modify_state() {
        let mut game = setup_test_game();

        // Too short
        assert_eq!(game.submit_guess("rat"), GuessResult::TooShort);
        assert_eq!(game.score, 0);

        // Invalid character
        assert_eq!(
            game.submit_guess("rate!"),
            GuessResult::InvalidCharacters('!')
        );
        assert_eq!(game.score, 0);

        // Missing main letter 'R'
        assert_eq!(
            game.submit_guess("plate"),
            GuessResult::MissingMainLetter('R')
        );
        assert_eq!(game.score, 0);

        // Invalid letter not in puzzle ('Z')
        assert_eq!(game.submit_guess("raze"), GuessResult::InvalidLetters('Z'));
        assert_eq!(game.score, 0);

        // Valid letters, contains main letter, but not in solutions
        assert_eq!(game.submit_guess("race"), GuessResult::NotInSolutions);
        assert_eq!(game.score, 0);
    }

    #[test]
    fn test_queen_bee_and_remaining_words() {
        let mut game = setup_test_game();
        assert!(!game.is_queen_bee());
        assert_eq!(game.remaining_solutions().len(), 3);

        game.submit_guess("rate");
        game.submit_guess("rater");
        assert!(!game.is_queen_bee());
        assert_eq!(game.remaining_solutions(), vec!["PLACATER".to_string()]);

        game.submit_guess("placater");
        assert!(game.is_queen_bee());
        assert!(game.remaining_solutions().is_empty());
    }
}
