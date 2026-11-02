//! # Boggle Game State, Batch Evaluation, and Guess Management
//!
//! Maintains active round state, collected player inputs, batch evaluation logic,
//! duplicate tracking, and score calculation.

use crate::board::Board;
use crate::scoring::calculate_score;
use crate::validator::{can_form_word, is_dictionary_word, MIN_BOGGLE_LENGTH};
use std::collections::HashSet;
use trie::Trie;

/// Outcome of evaluating a single Boggle guess.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BoggleGuessResult {
    /// Guess is valid, present on the board, in the dictionary, and not previously scored.
    Accepted { points: u32, word: String },
    /// Guess has fewer than 3 letters.
    TooShort,
    /// Guess contains non-alphabetic characters.
    InvalidCharacters(char),
    /// Word can be formed on the board, but is not present in the dictionary.
    NotInDictionary,
    /// Word is in the dictionary, but cannot be traced along adjacent cells on the board.
    NotOnBoard,
    /// Word was already scored earlier in this round.
    AlreadyGuessed,
}

/// An evaluated guess token with its assigned outcome.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EvaluatedGuess {
    pub raw_token: String,
    pub result: BoggleGuessResult,
}

/// Active Boggle game round.
#[derive(Debug, Clone)]
pub struct BoggleGame {
    /// The current $N \times N$ board.
    pub board: Board,
    /// Raw un-evaluated guesses submitted by the player during the round.
    pub collected_guesses: Vec<String>,
    /// Set of unique uppercase words that have been successfully accepted and scored.
    pub accepted_words: HashSet<String>,
    /// Chronological list of evaluations for the round.
    pub evaluations: Vec<EvaluatedGuess>,
    /// Accumulated total score for this round.
    pub score: u32,
    /// Whether the round is currently accepting guesses.
    pub is_active: bool,
}

impl BoggleGame {
    /// Creates a new game round with the specified board.
    pub fn new(board: Board) -> Self {
        Self {
            board,
            collected_guesses: Vec::new(),
            accepted_words: HashSet::new(),
            evaluations: Vec::new(),
            score: 0,
            is_active: true,
        }
    }

    /// Splits an input line into individual word tokens separated by spaces, commas, or mixed delimiters.
    pub fn parse_input_tokens(input: &str) -> Vec<String> {
        input
            .split(|c: char| c == ',' || c.is_whitespace())
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string())
            .collect()
    }

    /// Adds a raw guess token to the pending collection if the round is active.
    pub fn add_guess(&mut self, token: &str) {
        if self.is_active {
            let trimmed = token.trim();
            if !trimmed.is_empty() {
                self.collected_guesses.push(trimmed.to_string());
            }
        }
    }

    /// Adds multiple tokens from an input line.
    pub fn add_input_line(&mut self, line: &str) {
        for token in Self::parse_input_tokens(line) {
            self.add_guess(&token);
        }
    }

    /// Evaluates a single word candidate against the dictionary and the board.
    pub fn evaluate_single_word(
        board: &Board,
        trie: &Trie,
        accepted: &mut HashSet<String>,
        raw_token: &str,
    ) -> BoggleGuessResult {
        let trimmed = raw_token.trim();

        // 1. Validate characters & normalize to uppercase
        let mut normalized = String::with_capacity(trimmed.len());
        for ch in trimmed.chars() {
            if ch.is_ascii_alphabetic() {
                normalized.push(ch.to_ascii_uppercase());
            } else {
                return BoggleGuessResult::InvalidCharacters(ch);
            }
        }

        // 2. Check minimum length
        if normalized.len() < MIN_BOGGLE_LENGTH {
            return BoggleGuessResult::TooShort;
        }

        // 3. Check for duplicates
        if accepted.contains(&normalized) {
            return BoggleGuessResult::AlreadyGuessed;
        }

        // 4. Check dictionary membership
        let in_dict = is_dictionary_word(trie, &normalized);

        // 5. Check board path formation
        let on_board = can_form_word(board, &normalized);

        match (in_dict, on_board) {
            (true, true) => {
                let pts = calculate_score(normalized.len());
                accepted.insert(normalized.clone());
                BoggleGuessResult::Accepted {
                    points: pts,
                    word: normalized,
                }
            }
            (false, true) => BoggleGuessResult::NotInDictionary,
            (true, false) => BoggleGuessResult::NotOnBoard,
            (false, false) => BoggleGuessResult::NotInDictionary,
        }
    }

    /// Evaluates all collected guesses in batch.
    ///
    /// Locks the round (`is_active = false`), preventing subsequent modifications or retraction.
    /// Returns the complete list of evaluated guesses.
    pub fn evaluate_batch(&mut self, trie: &Trie) -> Vec<EvaluatedGuess> {
        self.is_active = false;
        self.evaluations.clear();
        self.accepted_words.clear();
        self.score = 0;

        for token in &self.collected_guesses {
            let res =
                Self::evaluate_single_word(&self.board, trie, &mut self.accepted_words, token);
            if let BoggleGuessResult::Accepted { points, .. } = res {
                self.score += points;
            }
            self.evaluations.push(EvaluatedGuess {
                raw_token: token.clone(),
                result: res,
            });
        }

        self.evaluations.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Test board:
    ///   +---+---+---+---+
    ///   | P | E | A | C |
    ///   +---+---+---+---+
    ///   | L | A | T | E |
    ///   +---+---+---+---+
    ///   | R | O | S | E |
    ///   +---+---+---+---+
    ///   | D | U | C | K |
    ///   +---+---+---+---+
    fn sample_board() -> Board {
        let cells = vec![
            'P', 'E', 'A', 'C', 'L', 'A', 'T', 'E', 'R', 'O', 'S', 'E', 'D', 'U', 'C', 'K',
        ];
        Board::new(4, cells).unwrap()
    }

    fn sample_trie() -> Trie {
        let mut trie = Trie::new();
        trie.insert("LATE").unwrap();
        trie.insert("ROSE").unwrap();
        trie.insert("DUCK").unwrap();
        trie.insert("PEAL").unwrap();
        trie.insert("PYTHON").unwrap(); // In dict, but not on board
        trie
    }

    #[test]
    fn test_parse_input_tokens_mixed_delimiters() {
        let input = "late, rose duck ,  , peal\trose";
        let tokens = BoggleGame::parse_input_tokens(input);
        assert_eq!(tokens, vec!["late", "rose", "duck", "peal", "rose"]);
    }

    #[test]
    fn test_batch_evaluation_flow() {
        let board = sample_board();
        let trie = sample_trie();
        let mut game = BoggleGame::new(board);

        // Add various guesses
        game.add_guess("late"); // valid (len 4 -> 1 pt)
        game.add_guess("rose"); // valid (len 4 -> 1 pt)
        game.add_guess("late"); // duplicate -> AlreadyGuessed
        game.add_guess("no"); // too short
        game.add_guess("cat!"); // invalid char
        game.add_guess("python"); // in dict, not on board -> NotOnBoard
        game.add_guess("pask"); // on board, not in dict -> NotInDictionary

        let results = game.evaluate_batch(&trie);
        assert_eq!(results.len(), 7);

        assert_eq!(
            results[0].result,
            BoggleGuessResult::Accepted {
                points: 1,
                word: "LATE".to_string()
            }
        );
        assert_eq!(
            results[1].result,
            BoggleGuessResult::Accepted {
                points: 1,
                word: "ROSE".to_string()
            }
        );
        assert_eq!(results[2].result, BoggleGuessResult::AlreadyGuessed);
        assert_eq!(results[3].result, BoggleGuessResult::TooShort);
        assert_eq!(results[4].result, BoggleGuessResult::InvalidCharacters('!'));
        assert_eq!(results[5].result, BoggleGuessResult::NotOnBoard);
        assert_eq!(results[6].result, BoggleGuessResult::NotInDictionary);

        // Score: 1 + 1 = 2
        assert_eq!(game.score, 2);
        assert!(!game.is_active);
    }
}
