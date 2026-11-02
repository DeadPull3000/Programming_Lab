//! # Boggle Library
//!
//! Provides the core game engine, board representation, coordinate mapping,
//! path validation, dictionary checking, scoring, and automatic solver for Boggle.

pub mod board;
pub mod game;
pub mod scoring;
pub mod validator;

pub use board::{Board, BoardError, DIRECTIONS, VALID_DIMENSIONS};
pub use game::{BoggleGame, BoggleGuessResult, EvaluatedGuess};
pub use scoring::calculate_score;
pub use validator::{
    can_form_word, can_form_word_with_qu, is_dictionary_word, solve_boggle_board, MIN_BOGGLE_LENGTH,
};
