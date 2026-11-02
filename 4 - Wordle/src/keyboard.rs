//! # Wordle Cumulative Keyboard State
//!
//! Tracks cumulative player knowledge for each of the 26 alphabet letters (`A`–`Z`).
//!
//! Unlike individual board tiles which represent position-specific feedback, keyboard keys
//! aggregate information across all submitted guesses following strict monotonic priority:
//!
//! ```text
//! Unknown < Absent < Present < Correct
//! ```
//!
//! A stronger status upgrades a weaker status, but a key state can **never** be downgraded
//! (e.g., `Correct` can never revert to `Present` or `Absent`).

use crate::feedback::{Feedback, LetterFeedback};
use crate::word::Word;
use std::fmt;

/// Cumulative knowledge status for an individual alphabet key.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum KeyStatus {
    /// Letter has not yet been used in any accepted guess.
    Unknown = 0,
    /// Letter was guessed and does not occur in the answer.
    Absent = 1,
    /// Letter occurs in the answer, but its exact position is not yet pinned down.
    Present = 2,
    /// Letter's exact position has been identified in at least one guess.
    Correct = 3,
}

impl KeyStatus {
    /// Upgrades the current status with incoming feedback if the new state is higher priority.
    ///
    /// Monotonicity guarantee:
    /// `Unknown < Absent < Present < Correct`.
    pub fn upgrade_with(&mut self, feedback: LetterFeedback) {
        let new_status = match feedback {
            LetterFeedback::Correct => KeyStatus::Correct,
            LetterFeedback::Present => KeyStatus::Present,
            LetterFeedback::Absent => KeyStatus::Absent,
        };

        if new_status > *self {
            *self = new_status;
        }
    }

    /// Single character symbol representation.
    pub fn as_char(&self) -> char {
        match self {
            KeyStatus::Unknown => '.',
            KeyStatus::Absent => 'X',
            KeyStatus::Present => 'Y',
            KeyStatus::Correct => 'G',
        }
    }
}

impl fmt::Display for KeyStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_char())
    }
}

/// Tracks the 26-letter virtual keyboard state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyboardState {
    keys: [KeyStatus; 26],
}

impl Default for KeyboardState {
    fn default() -> Self {
        Self::new()
    }
}

impl KeyboardState {
    /// Constructs a clean keyboard where all 26 keys are [`KeyStatus::Unknown`].
    pub const fn new() -> Self {
        Self {
            keys: [KeyStatus::Unknown; 26],
        }
    }

    /// Resets all 26 keys back to [`KeyStatus::Unknown`] for a new round.
    pub fn reset(&mut self) {
        self.keys = [KeyStatus::Unknown; 26];
    }

    /// Returns the cumulative status of a specific Latin alphabet character.
    pub fn status_of(&self, ch: char) -> KeyStatus {
        let byte = (ch as u8).to_ascii_uppercase();
        if byte.is_ascii_uppercase() {
            let idx = (byte - b'A') as usize;
            self.keys[idx]
        } else {
            KeyStatus::Unknown
        }
    }

    /// Updates the keyboard from a submitted guess and its calculated feedback.
    ///
    /// Correctly resolves duplicate letters in the same guess by applying the
    /// strongest feedback state for each distinct letter.
    pub fn update_from_guess(&mut self, word: &Word, feedback: &Feedback) {
        let bytes = word.as_bytes();
        let states = feedback.as_slice();

        for i in 0..bytes.len() {
            let idx = (bytes[i] - b'A') as usize;
            self.keys[idx].upgrade_with(states[i]);
        }
    }

    /// Returns a reference to the raw 26-key status array.
    pub fn raw_keys(&self) -> &[KeyStatus; 26] {
        &self.keys
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_initial_keyboard_is_all_unknown() {
        let kb = KeyboardState::new();
        for c in 'A'..='Z' {
            assert_eq!(kb.status_of(c), KeyStatus::Unknown);
        }
    }

    #[test]
    fn test_monotonic_upgrades() {
        let mut key = KeyStatus::Unknown;

        // Unknown -> Absent
        key.upgrade_with(LetterFeedback::Absent);
        assert_eq!(key, KeyStatus::Absent);

        // Absent -> Present
        key.upgrade_with(LetterFeedback::Present);
        assert_eq!(key, KeyStatus::Present);

        // Present -> Correct
        key.upgrade_with(LetterFeedback::Correct);
        assert_eq!(key, KeyStatus::Correct);

        // Correct never downgrades to Absent or Present
        key.upgrade_with(LetterFeedback::Absent);
        assert_eq!(key, KeyStatus::Correct);

        key.upgrade_with(LetterFeedback::Present);
        assert_eq!(key, KeyStatus::Correct);
    }

    #[test]
    fn test_duplicate_letter_strongest_feedback_priority() {
        let mut kb = KeyboardState::new();
        let word = Word::parse("ALLEY").unwrap();

        // Suppose first 'L' was Present (Yellow) and second 'L' was Absent (Gray)
        let feedback = Feedback::new([
            LetterFeedback::Correct, // A
            LetterFeedback::Present, // L (Yellow)
            LetterFeedback::Absent,  // L (Gray)
            LetterFeedback::Absent,  // E
            LetterFeedback::Absent,  // Y
        ]);

        kb.update_from_guess(&word, &feedback);

        assert_eq!(kb.status_of('A'), KeyStatus::Correct);
        // 'L' must be Present (Yellow), NOT Absent (Gray)!
        assert_eq!(kb.status_of('L'), KeyStatus::Present);
        assert_eq!(kb.status_of('E'), KeyStatus::Absent);
        assert_eq!(kb.status_of('Y'), KeyStatus::Absent);
        assert_eq!(kb.status_of('Z'), KeyStatus::Unknown);
    }

    #[test]
    fn test_keyboard_reset() {
        let mut kb = KeyboardState::new();
        let word = Word::parse("CRANE").unwrap();
        let feedback = Feedback::new([LetterFeedback::Correct; 5]);
        kb.update_from_guess(&word, &feedback);

        assert_eq!(kb.status_of('C'), KeyStatus::Correct);
        kb.reset();
        assert_eq!(kb.status_of('C'), KeyStatus::Unknown);
    }
}
