//! # Spelling Bee Library
//!
//! Provides the core game engine, puzzle validation, Trie-guided solution solver,
//! scoring rules, hint generation, and game state management for Spelling Bee.

pub mod game;
pub mod hints;
pub mod puzzle;
pub mod scoring;
pub mod solver;

pub use game::{GuessResult, SpellingBeeGame};
pub use hints::{HintGrid, SpellingBeeHints, TwoLetterPrefixes};
pub use puzzle::{
    PuzzleError, SpellingBeePuzzle, FULL_PANGRAM_MASK, MAIN_LETTER_BIT, PUZZLE_LETTER_COUNT,
};
pub use scoring::calculate_score;
pub use solver::{brute_force_reference_solve, solve_spelling_bee, MIN_WORD_LENGTH};
