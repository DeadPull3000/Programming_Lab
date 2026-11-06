//! # Boggle Scoring Rules
//!
//! Implements the scoring schedule specified in the Boggle assignment rules:
//!
//! | Word Length | Score |
//! | :--- | :--- |
//! | 3–4 letters | 1 point |
//! | 5 letters | 2 points |
//! | 6 letters | 3 points |
//! | 7 letters | 5 points |
//! | 8 letters | 11 points |
//! | 9+ letters | 2 points per letter |
//!
//! Words under 3 letters receive 0 points. For words of 9 letters and above, the assignment
//! specifies "2 points per letter"; we extend this rule consistently to all lengths $\ge 9$
//! (e.g. 9 letters = 18 points, 10 letters = 20 points). This is documented as an explicit design policy.

/// Calculates the Boggle score for a word of the given character length.
///
/// Pure calculation with no I/O, state mutation, or side effects.
pub fn calculate_score(length: usize) -> u32 {
    match length {
        0..=2 => 0,
        3 | 4 => 1,
        5 => 2,
        6 => 3,
        7 => 5,
        8 => 11,
        n => (n as u32) * 2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_under_three_letters_score_zero() {
        assert_eq!(calculate_score(0), 0);
        assert_eq!(calculate_score(1), 0);
        assert_eq!(calculate_score(2), 0);
    }

    #[test]
    fn test_standard_boggle_length_boundaries() {
        // 3 and 4 letters -> 1 point
        assert_eq!(calculate_score(3), 1);
        assert_eq!(calculate_score(4), 1);

        // 5 letters -> 2 points
        assert_eq!(calculate_score(5), 2);

        // 6 letters -> 3 points
        assert_eq!(calculate_score(6), 3);

        // 7 letters -> 5 points
        assert_eq!(calculate_score(7), 5);

        // 8 letters -> 11 points
        assert_eq!(calculate_score(8), 11);
    }

    #[test]
    fn test_nine_and_longer_extension_policy() {
        // 9 letters -> 18 points
        assert_eq!(calculate_score(9), 18);
        // 10 letters -> 20 points
        assert_eq!(calculate_score(10), 20);
        // 16 letters -> 32 points
        assert_eq!(calculate_score(16), 32);
    }
}
