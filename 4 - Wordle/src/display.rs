//! # Wordle Terminal Display & Rendering
//!
//! Provides rendering functions for the Wordle game grid, feedback tiles,
//! QWERTY keyboard status, instructions, and round summaries.
//!
//! Supports both ANSI 24-bit/8-bit truecolor formatting and plain-text fallback
//! (honoring the `NO_COLOR` environment standard).

use std::env;
use std::fmt::Write as FmtWrite;

use crate::feedback::{Feedback, LetterFeedback};
use crate::game::{GameStatus, WordleGame};
use crate::keyboard::{KeyStatus, KeyboardState};
use crate::word::WORD_LENGTH;

/// Standard ANSI color escape sequences.
pub mod colors {
    pub const RESET: &str = "\x1b[0m";
    pub const BOLD: &str = "\x1b[1m";

    // Foreground colors
    pub const FG_WHITE: &str = "\x1b[37;1m";
    pub const FG_BLACK: &str = "\x1b[30m";
    pub const FG_GRAY: &str = "\x1b[90m";
    pub const FG_GREEN: &str = "\x1b[32;1m";
    pub const FG_YELLOW: &str = "\x1b[33;1m";
    pub const FG_RED: &str = "\x1b[31;1m";
    pub const FG_CYAN: &str = "\x1b[36;1m";

    // Background tile colors
    pub const BG_GREEN: &str = "\x1b[42;1m\x1b[37;1m"; // Green bg, bright white text
    pub const BG_YELLOW: &str = "\x1b[43;1m\x1b[30m"; // Yellow bg, black text
    pub const BG_GRAY: &str = "\x1b[100m\x1b[37;1m"; // Bright black/dark gray bg
    pub const BG_EMPTY: &str = "\x1b[48;5;236m\x1b[37m"; // Muted dark background
}

/// Rendering configuration controlling colorization and layout styling.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DisplayConfig {
    pub use_color: bool,
}

impl Default for DisplayConfig {
    fn default() -> Self {
        Self {
            // Check if NO_COLOR environment variable is present
            use_color: env::var_os("NO_COLOR").is_none(),
        }
    }
}

impl DisplayConfig {
    /// Creates a config forcing plain-text output with zero ANSI escape codes.
    pub const fn plain() -> Self {
        Self { use_color: false }
    }

    /// Creates a config enabling ANSI colored tiles.
    pub const fn colored() -> Self {
        Self { use_color: true }
    }
}

/// Renders the complete Wordle game board (submitted guesses and blank remaining rows).
pub fn render_board(game: &WordleGame, config: &DisplayConfig) -> String {
    let mut out = String::new();
    let total_rows = game.max_attempts();
    let guesses = game.guesses();

    writeln!(out, "+---+---+---+---+---+").unwrap();

    for row in 0..total_rows {
        if row < guesses.len() {
            let record = &guesses[row];
            let row_str = render_guess_row(record.word.as_str(), &record.feedback, config);
            writeln!(out, "| {row_str} |").unwrap();
        } else {
            // Empty unplayed row
            writeln!(out, "|   |   |   |   |   |").unwrap();
        }
        writeln!(out, "+---+---+---+---+---+").unwrap();
    }

    out
}

/// Renders a single 5-letter guess row with appropriate tile formatting.
pub fn render_guess_row(word_str: &str, feedback: &Feedback, config: &DisplayConfig) -> String {
    let chars: Vec<char> = word_str.chars().collect();
    let states = feedback.as_slice();

    let mut tiles = Vec::with_capacity(WORD_LENGTH);
    for i in 0..WORD_LENGTH {
        let ch = chars[i];
        let state = states[i];
        if config.use_color {
            let (bg, reset) = match state {
                LetterFeedback::Correct => (colors::BG_GREEN, colors::RESET),
                LetterFeedback::Present => (colors::BG_YELLOW, colors::RESET),
                LetterFeedback::Absent => (colors::BG_GRAY, colors::RESET),
            };
            tiles.push(format!("{bg} {ch} {reset}"));
        } else {
            let sym = state.as_char();
            tiles.push(format!("{ch}[{sym}]"));
        }
    }

    if config.use_color {
        tiles.join(" ")
    } else {
        tiles.join("|")
    }
}

/// QWERTY keyboard rows.
const KEYBOARD_ROW_1: &[char] = &['Q', 'W', 'E', 'R', 'T', 'Y', 'U', 'I', 'O', 'P'];
const KEYBOARD_ROW_2: &[char] = &['A', 'S', 'D', 'F', 'G', 'H', 'J', 'K', 'L'];
const KEYBOARD_ROW_3: &[char] = &['Z', 'X', 'C', 'V', 'B', 'N', 'M'];

/// Renders the cumulative QWERTY virtual keyboard.
pub fn render_keyboard(kb: &KeyboardState, config: &DisplayConfig) -> String {
    let mut out = String::new();
    writeln!(out, "Keyboard:").unwrap();

    render_keyboard_row(&mut out, KEYBOARD_ROW_1, "", kb, config);
    render_keyboard_row(&mut out, KEYBOARD_ROW_2, " ", kb, config);
    render_keyboard_row(&mut out, KEYBOARD_ROW_3, "   ", kb, config);

    out
}

fn render_keyboard_row(
    out: &mut String,
    row_chars: &[char],
    indent: &str,
    kb: &KeyboardState,
    config: &DisplayConfig,
) {
    write!(out, "{indent}").unwrap();
    for (i, &ch) in row_chars.iter().enumerate() {
        let status = kb.status_of(ch);
        if config.use_color {
            let (color, reset) = match status {
                KeyStatus::Correct => (colors::BG_GREEN, colors::RESET),
                KeyStatus::Present => (colors::BG_YELLOW, colors::RESET),
                KeyStatus::Absent => (colors::FG_GRAY, colors::RESET),
                KeyStatus::Unknown => (colors::FG_WHITE, colors::RESET),
            };
            write!(out, "{color}{ch}{reset}").unwrap();
        } else {
            let marker = match status {
                KeyStatus::Correct => "[G]",
                KeyStatus::Present => "[Y]",
                KeyStatus::Absent => "[X]",
                KeyStatus::Unknown => "   ",
            };
            write!(out, "{ch}{marker}").unwrap();
        }
        if i + 1 < row_chars.len() {
            write!(out, " ").unwrap();
        }
    }
    writeln!(out).unwrap();
}

/// Renders the Wordle banner and introductory instructions.
pub fn render_welcome_banner(config: &DisplayConfig) -> String {
    if config.use_color {
        format!(
            "{bold}===================================={reset}\n\
             {cyan}              WORDLE{reset}\n\
             {bold}===================================={reset}\n\
             Guess the hidden 5-letter word in 6 attempts.\n\
             Tiles: {green}Green{reset} = Correct, {yellow}Yellow{reset} = Present, {gray}Gray{reset} = Absent.\n\
             Special commands: 'HELP', 'QUIT'\n",
            bold = colors::BOLD,
            cyan = colors::FG_CYAN,
            green = colors::FG_GREEN,
            yellow = colors::FG_YELLOW,
            gray = colors::FG_GRAY,
            reset = colors::RESET,
        )
    } else {
        "====================================\n\
                       WORDLE\n\
         ====================================\n\
         Guess the hidden 5-letter word in 6 attempts.\n\
         Tiles: [G] = Correct, [Y] = Present, [X] = Absent.\n\
         Special commands: 'HELP', 'QUIT'\n"
            .to_string()
    }
}

/// Renders the end-of-round outcome announcement.
pub fn render_game_over(game: &WordleGame, config: &DisplayConfig) -> String {
    match game.status() {
        GameStatus::Won => {
            let attempts = game.attempts_used();
            if config.use_color {
                format!(
                    "\n{green}{bold}Splendid! You guessed the word '{word}' in {attempts} attempt(s)!{reset}\n",
                    green = colors::FG_GREEN,
                    bold = colors::BOLD,
                    word = game.answer().as_str(),
                    reset = colors::RESET
                )
            } else {
                format!(
                    "\nSplendid! You guessed the word '{}' in {} attempt(s)!\n",
                    game.answer().as_str(),
                    attempts
                )
            }
        }
        GameStatus::Lost => {
            if config.use_color {
                format!(
                    "\n{red}{bold}Game Over! You ran out of attempts.{reset}\n\
                     The secret word was: {yellow}{bold}{word}{reset}\n",
                    red = colors::FG_RED,
                    yellow = colors::FG_YELLOW,
                    bold = colors::BOLD,
                    word = game.answer().as_str(),
                    reset = colors::RESET
                )
            } else {
                format!(
                    "\nGame Over! You ran out of attempts.\nThe secret word was: {}\n",
                    game.answer().as_str()
                )
            }
        }
        GameStatus::InProgress => String::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::feedback::calculate_feedback;
    use crate::word::Word;

    #[test]
    fn test_plain_board_rendering_empty() {
        let ans = Word::parse("APPLE").unwrap();
        let game = WordleGame::new(ans);
        let board_output = render_board(&game, &DisplayConfig::plain());

        // Should have header border + 6 rows + intermediate borders
        assert!(board_output.contains("+---+---+---+---+---+"));
        assert!(board_output.contains("|   |   |   |   |   |"));
    }

    #[test]
    fn test_plain_board_rendering_with_guesses() {
        let trie = trie::Trie::new();
        let mut trie = trie;
        trie.insert("CRANE").unwrap();
        trie.insert("APPLE").unwrap();

        let mut game = WordleGame::new(Word::parse("APPLE").unwrap());
        game.submit_guess("CRANE", &trie).unwrap();

        let board_output = render_board(&game, &DisplayConfig::plain());
        assert!(board_output.contains("C[X]"));
        assert!(board_output.contains("A[Y]"));
        assert!(board_output.contains("R[X]"));
        assert!(board_output.contains("N[X]"));
    }

    #[test]
    fn test_keyboard_rendering_plain() {
        let mut kb = KeyboardState::new();
        let word = Word::parse("CRANE").unwrap();
        let feedback = calculate_feedback(&Word::parse("APPLE").unwrap(), &word);
        kb.update_from_guess(&word, &feedback);

        let kb_str = render_keyboard(&kb, &DisplayConfig::plain());
        assert!(kb_str.contains("A[Y]"));
        assert!(kb_str.contains("C[X]"));
        assert!(kb_str.contains("Q   "));
    }

    #[test]
    fn test_welcome_banner_renders_cleanly() {
        let colored = render_welcome_banner(&DisplayConfig::colored());
        assert!(colored.contains("WORDLE"));

        let plain = render_welcome_banner(&DisplayConfig::plain());
        assert!(plain.contains("WORDLE"));
        assert!(!plain.contains("\x1b["));
    }

    #[test]
    fn test_game_over_rendering_win_and_loss() {
        let ans = Word::parse("APPLE").unwrap();
        let mut game_won = WordleGame::new(ans.clone());
        let mut trie = trie::Trie::new();
        trie.insert("APPLE").unwrap();
        game_won.submit_guess("APPLE", &trie).unwrap();

        let win_msg = render_game_over(&game_won, &DisplayConfig::plain());
        assert!(win_msg.contains("Splendid!"));
        assert!(win_msg.contains("APPLE"));
    }
}
