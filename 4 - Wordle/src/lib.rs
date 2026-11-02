//! # Wordle Library
//!
//! A modular, robust implementation of Wordle reusing a shared 26-ary Trie dictionary.

pub mod candidate;
pub mod dictionary;
pub mod display;
pub mod feedback;
pub mod game;
pub mod keyboard;
pub mod stats;
pub mod word;

pub use candidate::{filter_candidates, filter_candidates_multi_step};
pub use dictionary::{choose_answer, DictionaryError, WordleDictionary};
pub use display::{
    render_board, render_game_over, render_keyboard, render_welcome_banner, DisplayConfig,
};
pub use feedback::{calculate_feedback, Feedback, LetterFeedback};
pub use game::{
    GameError, GameStatus, GuessOutcome, GuessRecord, WordleGame, DEFAULT_MAX_ATTEMPTS,
};
pub use keyboard::{KeyStatus, KeyboardState};
pub use stats::PlayerStats;
pub use word::{Word, WordError, WORD_LENGTH};

pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}
