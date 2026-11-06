//! # Wordle Game Engine & State Management
//!
//! Provides the core game state machine, attempt tracking, guess validation,
//! duplicate policy enforcement, and victory/defeat evaluation.

use std::fmt;
use trie::Trie;

use crate::feedback::{calculate_feedback, Feedback};
use crate::word::{Word, WordError};

/// Standard maximum number of attempts in classic Wordle.
pub const DEFAULT_MAX_ATTEMPTS: usize = 6;

/// The overall progression status of a Wordle game round.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameStatus {
    /// The round is actively being played; attempts remain.
    InProgress,
    /// The player successfully guessed the secret word.
    Won,
    /// The player exhausted all attempts without finding the secret word.
    Lost,
}

impl fmt::Display for GameStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GameStatus::InProgress => write!(f, "In Progress"),
            GameStatus::Won => write!(f, "Won"),
            GameStatus::Lost => write!(f, "Lost"),
        }
    }
}

/// A historical record of an accepted guess and its corresponding feedback.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GuessRecord {
    pub word: Word,
    pub feedback: Feedback,
}

/// The result returned upon submitting a candidate guess string.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GuessOutcome {
    /// The guess was accepted and evaluated.
    Accepted {
        feedback: Feedback,
        attempts_remaining: usize,
        status: GameStatus,
    },
    /// The word is valid 5-letter ASCII but does not exist in the accepted Trie lexicon.
    NotInDictionary,
    /// The word was previously submitted in this round (under the project's non-punitive duplicate policy).
    AlreadyGuessed,
}

/// Fatal error variants returned when a guess cannot even be processed as a candidate.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GameError {
    /// The game is already over (won or lost); no further guesses may be submitted.
    GameOver(GameStatus),
    /// The input string failed syntactic 5-letter word validation.
    InvalidWord(WordError),
}

impl fmt::Display for GameError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GameError::GameOver(status) => {
                write!(
                    f,
                    "game is already completed ({status}); cannot accept further guesses"
                )
            }
            GameError::InvalidWord(e) => write!(f, "invalid word format: {e}"),
        }
    }
}

impl std::error::Error for GameError {}

impl From<WordError> for GameError {
    fn from(err: WordError) -> Self {
        GameError::InvalidWord(err)
    }
}

/// The central state machine for a Wordle game session.
#[derive(Debug, Clone)]
pub struct WordleGame {
    answer: Word,
    guesses: Vec<GuessRecord>,
    status: GameStatus,
    max_attempts: usize,
}

impl WordleGame {
    /// Initializes a new Wordle game with the default limit of 6 attempts.
    pub fn new(answer: Word) -> Self {
        Self::with_max_attempts(answer, DEFAULT_MAX_ATTEMPTS)
    }

    /// Initializes a Wordle game with a custom attempt limit.
    pub fn with_max_attempts(answer: Word, max_attempts: usize) -> Self {
        Self {
            answer,
            guesses: Vec::with_capacity(max_attempts),
            status: GameStatus::InProgress,
            max_attempts,
        }
    }

    /// Returns a reference to the secret answer.
    #[inline]
    pub fn answer(&self) -> &Word {
        &self.answer
    }

    /// Returns the history of accepted guesses.
    #[inline]
    pub fn guesses(&self) -> &[GuessRecord] {
        &self.guesses
    }

    /// Returns the current game progress status.
    #[inline]
    pub fn status(&self) -> GameStatus {
        self.status
    }

    /// Returns `true` if the game has ended (either won or lost).
    #[inline]
    pub fn is_over(&self) -> bool {
        self.status != GameStatus::InProgress
    }

    /// Returns the number of accepted attempts used so far.
    #[inline]
    pub fn attempts_used(&self) -> usize {
        self.guesses.len()
    }

    /// Returns the number of remaining attempts before defeat.
    #[inline]
    pub fn attempts_remaining(&self) -> usize {
        self.max_attempts.saturating_sub(self.guesses.len())
    }

    /// Returns the maximum allowed attempts configured for this game.
    #[inline]
    pub fn max_attempts(&self) -> usize {
        self.max_attempts
    }

    /// Submits a raw input guess string, validating against the provided dictionary Trie.
    ///
    /// # Pipeline Sequence:
    /// 1. Verifies that `status == GameStatus::InProgress` (rejects post-game submissions).
    /// 2. Parses, trims, and validates 5-letter ASCII structure ([`Word::parse`]).
    /// 3. Checks if the word was already submitted in this round (returns [`GuessOutcome::AlreadyGuessed`]).
    /// 4. Checks Trie dictionary membership (returns [`GuessOutcome::NotInDictionary`]).
    /// 5. Computes two-pass feedback ([`calculate_feedback`]).
    /// 6. Records the guess and updates the status to [`GameStatus::Won`] or [`GameStatus::Lost`].
    ///
    /// # Idempotency Guarantee
    /// Any rejected guess (invalid format, unknown word, or duplicate) leaves the game state,
    /// guess list, and remaining attempt count **completely unmodified**.
    pub fn submit_guess(
        &mut self,
        input: &str,
        dictionary: &Trie,
    ) -> Result<GuessOutcome, GameError> {
        if self.is_over() {
            return Err(GameError::GameOver(self.status));
        }

        let word = Word::parse(input)?;

        // Check duplicate submission policy
        if self.guesses.iter().any(|record| record.word == word) {
            return Ok(GuessOutcome::AlreadyGuessed);
        }

        // Check Trie dictionary membership
        if !dictionary.contains(word.as_str()) {
            return Ok(GuessOutcome::NotInDictionary);
        }

        // Calculate feedback
        let feedback = calculate_feedback(&self.answer, &word);

        // Record guess
        self.guesses.push(GuessRecord { word, feedback });

        // Update status
        if feedback.is_all_correct() {
            self.status = GameStatus::Won;
        } else if self.guesses.len() >= self.max_attempts {
            self.status = GameStatus::Lost;
        }

        Ok(GuessOutcome::Accepted {
            feedback,
            attempts_remaining: self.attempts_remaining(),
            status: self.status,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn build_test_trie() -> Trie {
        let mut trie = Trie::new();
        trie.insert("APPLE").unwrap();
        trie.insert("CRANE").unwrap();
        trie.insert("SLATE").unwrap();
        trie.insert("ROAST").unwrap();
        trie.insert("TRAIN").unwrap();
        trie.insert("ALERT").unwrap();
        trie.insert("ALLEY").unwrap();
        trie
    }

    #[test]
    fn test_initial_game_state() {
        let ans = Word::parse("APPLE").unwrap();
        let game = WordleGame::new(ans.clone());

        assert_eq!(game.answer(), &ans);
        assert_eq!(game.status(), GameStatus::InProgress);
        assert_eq!(game.attempts_used(), 0);
        assert_eq!(game.attempts_remaining(), 6);
        assert!(!game.is_over());
        assert!(game.guesses().is_empty());
    }

    #[test]
    fn test_winning_first_attempt() {
        let trie = build_test_trie();
        let mut game = WordleGame::new(Word::parse("APPLE").unwrap());

        let outcome = game.submit_guess("apple", &trie).unwrap();
        match outcome {
            GuessOutcome::Accepted {
                feedback,
                attempts_remaining,
                status,
            } => {
                assert!(feedback.is_all_correct());
                assert_eq!(attempts_remaining, 5);
                assert_eq!(status, GameStatus::Won);
            }
            _ => panic!("Expected Accepted"),
        }

        assert_eq!(game.status(), GameStatus::Won);
        assert!(game.is_over());
        assert_eq!(game.attempts_used(), 1);
    }

    #[test]
    fn test_multiple_guesses_and_loss_on_sixth() {
        let trie = build_test_trie();
        let words = ["CRANE", "SLATE", "ROAST", "TRAIN", "ALERT", "ALLEY"];
        let mut game = WordleGame::new(Word::parse("APPLE").unwrap());

        for (i, &w) in words.iter().enumerate() {
            let outcome = game.submit_guess(w, &trie).unwrap();
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
    fn test_win_on_sixth_attempt() {
        let trie = build_test_trie();
        let mut game = WordleGame::new(Word::parse("APPLE").unwrap());
        let words = ["CRANE", "SLATE", "ROAST", "TRAIN", "ALERT", "APPLE"];

        for (i, &w) in words.iter().enumerate() {
            let outcome = game.submit_guess(w, &trie).unwrap();
            match outcome {
                GuessOutcome::Accepted { status, .. } => {
                    if i == 5 {
                        assert_eq!(status, GameStatus::Won);
                    } else {
                        assert_eq!(status, GameStatus::InProgress);
                    }
                }
                _ => panic!("Expected accepted guess"),
            }
        }

        assert_eq!(game.status(), GameStatus::Won);
        assert_eq!(game.attempts_used(), 6);
    }

    #[test]
    fn test_duplicate_guess_does_not_consume_attempt() {
        let trie = build_test_trie();
        let mut game = WordleGame::new(Word::parse("APPLE").unwrap());

        let res1 = game.submit_guess("CRANE", &trie).unwrap();
        assert!(matches!(res1, GuessOutcome::Accepted { .. }));
        assert_eq!(game.attempts_used(), 1);

        // Submit duplicate
        let res2 = game.submit_guess("crane", &trie).unwrap();
        assert_eq!(res2, GuessOutcome::AlreadyGuessed);
        assert_eq!(game.attempts_used(), 1); // UNCHANGED!
        assert_eq!(game.attempts_remaining(), 5);
    }

    #[test]
    fn test_unknown_dictionary_word_does_not_consume_attempt() {
        let trie = build_test_trie();
        let mut game = WordleGame::new(Word::parse("APPLE").unwrap());

        // "XYZZY" is a valid 5-letter word format, but not in test trie
        let res = game.submit_guess("XYZZY", &trie).unwrap();
        assert_eq!(res, GuessOutcome::NotInDictionary);
        assert_eq!(game.attempts_used(), 0); // UNCHANGED!
        assert_eq!(game.attempts_remaining(), 6);
    }

    #[test]
    fn test_invalid_format_returns_game_error_and_does_not_consume_attempt() {
        let trie = build_test_trie();
        let mut game = WordleGame::new(Word::parse("APPLE").unwrap());

        assert!(matches!(
            game.submit_guess("CAT", &trie),
            Err(GameError::InvalidWord(WordError::InvalidLength(3)))
        ));
        assert!(matches!(
            game.submit_guess("12345", &trie),
            Err(GameError::InvalidWord(WordError::InvalidCharacter('1')))
        ));
        assert!(matches!(
            game.submit_guess("", &trie),
            Err(GameError::InvalidWord(WordError::EmptyWord))
        ));

        assert_eq!(game.attempts_used(), 0);
    }

    #[test]
    fn test_submission_after_game_over_rejected() {
        let trie = build_test_trie();
        let mut game = WordleGame::new(Word::parse("APPLE").unwrap());

        // Win
        game.submit_guess("APPLE", &trie).unwrap();
        assert!(game.is_over());

        // Attempt further guess
        let err = game.submit_guess("CRANE", &trie).unwrap_err();
        assert_eq!(err, GameError::GameOver(GameStatus::Won));
    }
}
