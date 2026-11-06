//! # Spelling Bee Scoring System
//!
//! Implements a pure, stateless scoring function based on word length and pangram status.
//!
//! # Scoring Policy
//!
//! - Words under 4 letters: 0 points (invalid for Spelling Bee).
//! - 4-letter words: 1 point.
//! - Words with 5 or more letters: 1 point per letter (e.g. 5 letters = 5 points, 8 letters = 8 points).
//! - Pangrams (words using all 7 unique puzzle letters): Base score + 7 bonus points.
//!
//! This policy is a clear, standard Spelling Bee scoring formula and is documented as an
//! explicit design decision.

/// Calculates the score for an accepted word given its length and whether it is a pangram.
///
/// Pure calculation with no I/O, formatting, or side effects.
pub fn calculate_score(word: &str, is_pangram: bool) -> u32 {
    let len = word.chars().count();
    let base_score = match len {
        0..=3 => 0,
        4 => 1,
        n => n as u32,
    };

    if is_pangram && base_score > 0 {
        base_score + 7
    } else {
        base_score
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_short_words_score_zero() {
        assert_eq!(calculate_score("", false), 0);
        assert_eq!(calculate_score("a", false), 0);
        assert_eq!(calculate_score("cat", false), 0);
        assert_eq!(calculate_score("cat", true), 0);
    }

    #[test]
    fn test_four_letter_word() {
        assert_eq!(calculate_score("rate", false), 1);
        assert_eq!(calculate_score("rate", true), 8); // 1 + 7 bonus
    }

    #[test]
    fn test_five_and_longer_words() {
        assert_eq!(calculate_score("rater", false), 5);
        assert_eq!(calculate_score("racers", false), 6);
        assert_eq!(calculate_score("parterre", false), 8);
    }

    #[test]
    fn test_pangram_bonus() {
        // 7-letter pangram: 7 base + 7 bonus = 14
        assert_eq!(calculate_score("abcdefg", true), 14);
        assert_eq!(calculate_score("abcdefg", false), 7);

        // 8-letter pangram with repeated letter: 8 base + 7 bonus = 15
        assert_eq!(calculate_score("parterre", true), 15);
    }
}
