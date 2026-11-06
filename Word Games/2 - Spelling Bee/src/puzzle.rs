//! # Spelling Bee Puzzle Representation and Bitmask Operations
//!
//! Maintains the invariant that a Spelling Bee puzzle consists of exactly seven
//! distinct uppercase ASCII letters ('A'..='Z') with the mandatory main letter
//! positioned at index 0.

use rand::seq::SliceRandom;
use rand::Rng;
use std::fmt;

/// The exact number of distinct letters in a standard Spelling Bee puzzle.
pub const PUZZLE_LETTER_COUNT: usize = 7;

/// Bitmask with all 7 puzzle letter bits set (`0b111_1111` = 127).
pub const FULL_PANGRAM_MASK: u8 = (1 << PUZZLE_LETTER_COUNT) - 1;

/// The mask bit for the mandatory main letter, which is always at index 0.
pub const MAIN_LETTER_BIT: u8 = 1 << 0;

/// Errors that can occur when constructing or validating a Spelling Bee puzzle.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PuzzleError {
    /// A character in the puzzle is not an uppercase ASCII letter ('A'..='Z').
    InvalidCharacter(char),
    /// A letter appears more than once among the seven puzzle letters.
    DuplicateLetter(char),
    /// The specified main letter does not exist among the puzzle letters.
    MissingMainLetter(char),
    /// The main letter is not positioned at index 0 as required for display and bit 0 alignment.
    MainLetterNotFirst { expected: char, found: char },
}

impl fmt::Display for PuzzleError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PuzzleError::InvalidCharacter(c) => {
                write!(
                    f,
                    "invalid character '{c}'; must be uppercase ASCII 'A'-'Z'"
                )
            }
            PuzzleError::DuplicateLetter(c) => {
                write!(
                    f,
                    "duplicate letter '{c}'; all seven letters must be distinct"
                )
            }
            PuzzleError::MissingMainLetter(c) => {
                write!(f, "main letter '{c}' is not present in the puzzle letters")
            }
            PuzzleError::MainLetterNotFirst { expected, found } => {
                write!(
                    f,
                    "main letter '{expected}' must be first in the array (found '{found}')"
                )
            }
        }
    }
}

impl std::error::Error for PuzzleError {}

/// Represents a validated Spelling Bee puzzle.
///
/// Invariants maintained:
/// 1. Exactly 7 letters (`[char; 7]`).
/// 2. All letters are ASCII uppercase 'A'..='Z'.
/// 3. All 7 letters are strictly unique (no duplicates).
/// 4. The mandatory main letter belongs to `letters`.
/// 5. `letters[0] == main_letter` (main letter is always first).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SpellingBeePuzzle {
    /// The 7 distinct letters, with the main letter at index 0.
    pub letters: [char; PUZZLE_LETTER_COUNT],
    /// The designated mandatory main letter (`letters[0]`).
    pub main_letter: char,
    /// Fast $O(1)$ alphabet membership lookup table (A=0..Z=25).
    membership: [bool; 26],
}

impl SpellingBeePuzzle {
    /// Constructs a `SpellingBeePuzzle` after validating all invariants.
    ///
    /// Requires that `letters[0] == main_letter`. For unordered letter sets,
    /// use [`SpellingBeePuzzle::from_unordered`].
    pub fn new(
        letters: [char; PUZZLE_LETTER_COUNT],
        main_letter: char,
    ) -> Result<Self, PuzzleError> {
        let mut membership = [false; 26];

        // 1. Verify main letter is uppercase ASCII
        if !main_letter.is_ascii_uppercase() {
            return Err(PuzzleError::InvalidCharacter(main_letter));
        }

        // 2. Verify all letters are valid uppercase ASCII and distinct
        for &ch in &letters {
            if !ch.is_ascii_uppercase() {
                return Err(PuzzleError::InvalidCharacter(ch));
            }
            let idx = (ch as u8 - b'A') as usize;
            if membership[idx] {
                return Err(PuzzleError::DuplicateLetter(ch));
            }
            membership[idx] = true;
        }

        // 3. Verify main letter is in the puzzle
        let main_idx = (main_letter as u8 - b'A') as usize;
        if !membership[main_idx] {
            return Err(PuzzleError::MissingMainLetter(main_letter));
        }

        // 4. Verify main letter is at index 0
        if letters[0] != main_letter {
            return Err(PuzzleError::MainLetterNotFirst {
                expected: main_letter,
                found: letters[0],
            });
        }

        Ok(Self {
            letters,
            main_letter,
            membership,
        })
    }

    /// Constructs a `SpellingBeePuzzle` from an array of 7 letters and a designated main letter,
    /// automatically placing the main letter at index 0.
    pub fn from_unordered(
        letters: [char; PUZZLE_LETTER_COUNT],
        main_letter: char,
    ) -> Result<Self, PuzzleError> {
        // Ensure main letter is present
        if !letters.contains(&main_letter) {
            if !main_letter.is_ascii_uppercase() {
                return Err(PuzzleError::InvalidCharacter(main_letter));
            }
            return Err(PuzzleError::MissingMainLetter(main_letter));
        }

        // Reorder so that main_letter is at index 0
        let mut ordered = [main_letter; PUZZLE_LETTER_COUNT];
        let mut target_idx = 1;
        for &ch in &letters {
            if ch != main_letter {
                ordered[target_idx] = ch;
                target_idx += 1;
            }
        }

        Self::new(ordered, main_letter)
    }

    /// Generates a random puzzle using the provided random number generator.
    ///
    /// Shuffles the English alphabet A–Z, chooses 7 distinct letters, designates one
    /// as the main letter, and places it at index 0.
    pub fn random<R: Rng>(rng: &mut R) -> Self {
        let mut alphabet: Vec<char> = ('A'..='Z').collect();
        alphabet.shuffle(rng);

        let mut letters = [' '; PUZZLE_LETTER_COUNT];
        letters.copy_from_slice(&alphabet[..PUZZLE_LETTER_COUNT]);

        // Select one of the 7 letters uniformly at random to be the main letter
        let main_idx = rng.gen_range(0..PUZZLE_LETTER_COUNT);
        let main_letter = letters[main_idx];

        // Ensure main letter is placed at index 0
        letters.swap(0, main_idx);

        Self::new(letters, main_letter)
            .expect("random generation from distinct alphabet must be valid")
    }

    /// Checks in $O(1)$ time whether a character belongs to the puzzle's 7 allowed letters.
    #[inline]
    pub fn contains_letter(&self, ch: char) -> bool {
        if ch.is_ascii_uppercase() {
            self.membership[(ch as u8 - b'A') as usize]
        } else {
            false
        }
    }

    /// Returns the 0-based bit index ($0..6$) for a puzzle letter, or `None` if absent.
    ///
    /// Index 0 corresponds to `main_letter`.
    #[inline]
    pub fn letter_bit_index(&self, ch: char) -> Option<u8> {
        for (i, &c) in self.letters.iter().enumerate() {
            if c == ch {
                return Some(i as u8);
            }
        }
        None
    }

    /// Computes the 7-bit usage mask for a word.
    ///
    /// Returns `Some(mask)` if all characters in the word belong to the puzzle,
    /// or `None` if the word contains any disallowed or invalid character.
    pub fn word_mask(&self, word: &str) -> Option<u8> {
        let mut mask: u8 = 0;
        for ch in word.chars() {
            let bit_idx = self.letter_bit_index(ch)?;
            mask |= 1 << bit_idx;
        }
        Some(mask)
    }

    /// Returns `true` if the bitmask includes the mandatory main letter (bit 0).
    #[inline]
    pub fn mask_has_main_letter(mask: u8) -> bool {
        (mask & MAIN_LETTER_BIT) != 0
    }

    /// Returns `true` if the bitmask includes all 7 puzzle letters (a pangram).
    #[inline]
    pub fn mask_is_pangram(mask: u8) -> bool {
        (mask & FULL_PANGRAM_MASK) == FULL_PANGRAM_MASK
    }

    /// Returns a human-friendly visual display of the puzzle.
    ///
    /// Main letter is highlighted first, followed by the remaining 6 letters.
    pub fn display_string(&self) -> String {
        format!(
            "Main Letter: [{}]  |  Other Letters: {} {} {} {} {} {}",
            self.main_letter,
            self.letters[1],
            self.letters[2],
            self.letters[3],
            self.letters[4],
            self.letters[5],
            self.letters[6]
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::rngs::StdRng;
    use rand::SeedableRng;

    #[test]
    fn test_valid_puzzle_construction() {
        let letters = ['A', 'B', 'C', 'D', 'E', 'F', 'G'];
        let puzzle = SpellingBeePuzzle::new(letters, 'A').unwrap();
        assert_eq!(puzzle.main_letter, 'A');
        assert_eq!(puzzle.letters[0], 'A');
        assert_eq!(puzzle.letters, letters);
    }

    #[test]
    fn test_duplicate_letters_rejected() {
        let letters = ['A', 'B', 'C', 'D', 'E', 'F', 'A'];
        let err = SpellingBeePuzzle::new(letters, 'A').unwrap_err();
        assert_eq!(err, PuzzleError::DuplicateLetter('A'));
    }

    #[test]
    fn test_invalid_character_rejected() {
        let letters = ['A', 'B', 'C', '1', 'E', 'F', 'G'];
        let err = SpellingBeePuzzle::new(letters, 'A').unwrap_err();
        assert_eq!(err, PuzzleError::InvalidCharacter('1'));

        let lowercase = ['a', 'b', 'c', 'd', 'e', 'f', 'g'];
        assert_eq!(
            SpellingBeePuzzle::new(lowercase, 'a').unwrap_err(),
            PuzzleError::InvalidCharacter('a')
        );
    }

    #[test]
    fn test_missing_main_letter_rejected() {
        let letters = ['A', 'B', 'C', 'D', 'E', 'F', 'G'];
        let err = SpellingBeePuzzle::new(letters, 'Z').unwrap_err();
        assert_eq!(err, PuzzleError::MissingMainLetter('Z'));
    }

    #[test]
    fn test_main_letter_not_first_rejected_by_new() {
        let letters = ['B', 'A', 'C', 'D', 'E', 'F', 'G'];
        let err = SpellingBeePuzzle::new(letters, 'A').unwrap_err();
        assert_eq!(
            err,
            PuzzleError::MainLetterNotFirst {
                expected: 'A',
                found: 'B'
            }
        );
    }

    #[test]
    fn test_from_unordered_rearranges_correctly() {
        let letters = ['B', 'A', 'C', 'D', 'E', 'F', 'G'];
        let puzzle = SpellingBeePuzzle::from_unordered(letters, 'A').unwrap();
        assert_eq!(puzzle.main_letter, 'A');
        assert_eq!(puzzle.letters[0], 'A');
        assert!(puzzle.contains_letter('B'));
        assert!(puzzle.contains_letter('A'));
        assert!(!puzzle.contains_letter('Z'));
    }

    #[test]
    fn test_random_puzzle_invariants_and_seeded_repeatability() {
        let mut rng1 = StdRng::seed_from_u64(42);
        let puzzle1 = SpellingBeePuzzle::random(&mut rng1);

        let mut rng2 = StdRng::seed_from_u64(42);
        let puzzle2 = SpellingBeePuzzle::random(&mut rng2);

        // Deterministic reproducibility
        assert_eq!(puzzle1, puzzle2);

        // Invariants
        assert_eq!(puzzle1.letters[0], puzzle1.main_letter);
        assert!(puzzle1.contains_letter(puzzle1.main_letter));

        // Check all 7 are distinct
        let mut seen = std::collections::HashSet::new();
        for &ch in &puzzle1.letters {
            assert!(ch.is_ascii_uppercase());
            assert!(seen.insert(ch));
        }
        assert_eq!(seen.len(), 7);
    }

    #[test]
    fn test_bitmask_operations() {
        let letters = ['S', 'P', 'E', 'L', 'I', 'N', 'G'];
        let puzzle = SpellingBeePuzzle::new(letters, 'S').unwrap();

        // No letters
        assert_eq!(puzzle.word_mask(""), Some(0));
        assert!(!SpellingBeePuzzle::mask_has_main_letter(0));
        assert!(!SpellingBeePuzzle::mask_is_pangram(0));

        // One letter: main letter 'S' -> bit 0
        let s_mask = puzzle.word_mask("S").unwrap();
        assert_eq!(s_mask, 0b0000001);
        assert!(SpellingBeePuzzle::mask_has_main_letter(s_mask));
        assert!(!SpellingBeePuzzle::mask_is_pangram(s_mask));

        // One letter: non-main letter 'P' -> bit 1
        let p_mask = puzzle.word_mask("P").unwrap();
        assert_eq!(p_mask, 0b0000010);
        assert!(!SpellingBeePuzzle::mask_has_main_letter(p_mask));
        assert!(!SpellingBeePuzzle::mask_is_pangram(p_mask));

        // Disallowed letter returns None
        assert_eq!(puzzle.word_mask("SPA"), None); // 'A' not in puzzle

        // Pangram (all 7 letters used)
        let pangram_mask = puzzle.word_mask("SPELLINGS").unwrap();
        assert!(SpellingBeePuzzle::mask_has_main_letter(pangram_mask));
        assert!(SpellingBeePuzzle::mask_is_pangram(pangram_mask));
        assert_eq!(pangram_mask, FULL_PANGRAM_MASK);

        // 6 of 7 letters (missing 'G')
        let six_mask = puzzle.word_mask("SPELIN").unwrap();
        assert!(SpellingBeePuzzle::mask_has_main_letter(six_mask));
        assert!(!SpellingBeePuzzle::mask_is_pangram(six_mask));
    }
}
