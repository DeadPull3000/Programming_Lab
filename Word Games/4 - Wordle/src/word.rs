//! # Validated Five-Letter Word Type
//!
//! Provides a strongly-typed `Word` struct guaranteeing all Wordle lexical invariants:
//! - Exactly 5 characters in length.
//! - Strictly ASCII uppercase letters `A` through `Z`.
//! - No punctuation, whitespace, digits, or non-ASCII characters.

use std::fmt;

/// The standard length of a Wordle word.
pub const WORD_LENGTH: usize = 5;

/// Error variants encountered during `Word` validation and parsing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WordError {
    /// The input was empty or contained only whitespace.
    EmptyWord,
    /// The input string did not have exactly 5 characters after trimming.
    InvalidLength(usize),
    /// The input contained an unsupported or non-ASCII alphabetic character.
    InvalidCharacter(char),
}

impl fmt::Display for WordError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            WordError::EmptyWord => write!(f, "word must not be empty"),
            WordError::InvalidLength(len) => {
                write!(f, "word must be exactly {WORD_LENGTH} letters, found {len}")
            }
            WordError::InvalidCharacter(c) => {
                write!(
                    f,
                    "invalid character '{c}'; only ASCII letters A-Z are allowed"
                )
            }
        }
    }
}

impl std::error::Error for WordError {}

/// A validated, normalized 5-letter Wordle word.
///
/// Encapsulates an invariant-checked uppercase ASCII string.
/// The inner `String` is kept private so that instances can only be
/// constructed via [`Word::parse`] or verified constructors.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Word(String);

impl Word {
    /// Parses, trims, normalizes, and validates a 5-letter word.
    ///
    /// # Normalization Rules
    /// 1. Trims leading and trailing whitespace.
    /// 2. Converts lowercase ASCII letters (`a`–`z`) to uppercase (`A`–`Z`).
    ///
    /// # Validation Rules
    /// 1. Rejects empty strings with [`WordError::EmptyWord`].
    /// 2. Rejects strings whose character count is not exactly 5 with [`WordError::InvalidLength`].
    /// 3. Rejects strings containing non-ASCII alphabetic characters with [`WordError::InvalidCharacter`].
    ///
    /// # Examples
    /// ```
    /// use wordle::word::{Word, WordError};
    ///
    /// let word = Word::parse("apple").unwrap();
    /// assert_eq!(word.as_str(), "APPLE");
    ///
    /// assert!(matches!(Word::parse("appl"), Err(WordError::InvalidLength(4))));
    /// ```
    pub fn parse(input: &str) -> Result<Self, WordError> {
        let trimmed = input.trim();
        if trimmed.is_empty() {
            return Err(WordError::EmptyWord);
        }

        // Check for invalid characters before length check to provide specific character errors
        for ch in trimmed.chars() {
            if !ch.is_ascii_alphabetic() {
                return Err(WordError::InvalidCharacter(ch));
            }
        }

        let char_count = trimmed.chars().count();
        if char_count != WORD_LENGTH {
            return Err(WordError::InvalidLength(char_count));
        }

        let normalized = trimmed.to_ascii_uppercase();
        Ok(Word(normalized))
    }

    /// Returns the word as an immutable string slice (`&str`).
    #[inline]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Returns the word as an immutable byte slice (`&[u8]`).
    ///
    /// Because the word is strictly ASCII, each byte corresponds 1:1 to a character.
    #[inline]
    pub fn as_bytes(&self) -> &[u8] {
        self.0.as_bytes()
    }
}

impl fmt::Display for Word {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl AsRef<str> for Word {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_word_parsing() {
        let w = Word::parse("APPLE").expect("valid uppercase");
        assert_eq!(w.as_str(), "APPLE");
        assert_eq!(w.as_bytes(), b"APPLE");
        assert_eq!(w.to_string(), "APPLE");
    }

    #[test]
    fn test_case_normalization_and_trimming() {
        let w = Word::parse("  crane  ").expect("valid lowercase with padding");
        assert_eq!(w.as_str(), "CRANE");

        let mixed = Word::parse("sLaTe").expect("mixed case");
        assert_eq!(mixed.as_str(), "SLATE");
    }

    #[test]
    fn test_invalid_length_rejection() {
        assert_eq!(Word::parse("APPL"), Err(WordError::InvalidLength(4)));
        assert_eq!(Word::parse("APPLES"), Err(WordError::InvalidLength(6)));
        assert_eq!(Word::parse("A"), Err(WordError::InvalidLength(1)));
    }

    #[test]
    fn test_invalid_characters_rejection() {
        assert_eq!(Word::parse("AP-LE"), Err(WordError::InvalidCharacter('-')));
        assert_eq!(Word::parse("12345"), Err(WordError::InvalidCharacter('1')));
        assert_eq!(Word::parse("RO ST"), Err(WordError::InvalidCharacter(' ')));
        assert_eq!(Word::parse("ÉTOIL"), Err(WordError::InvalidCharacter('É')));
    }

    #[test]
    fn test_empty_input_rejection() {
        assert_eq!(Word::parse(""), Err(WordError::EmptyWord));
        assert_eq!(Word::parse("   "), Err(WordError::EmptyWord));
    }

    #[test]
    fn test_ordering_and_equality() {
        let w1 = Word::parse("APPLE").unwrap();
        let w2 = Word::parse("apple").unwrap();
        let w3 = Word::parse("BAKER").unwrap();

        assert_eq!(w1, w2);
        assert!(w1 < w3);
    }
}
