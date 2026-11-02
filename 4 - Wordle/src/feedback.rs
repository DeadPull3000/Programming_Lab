//! # Wordle Two-Pass Feedback Algorithm
//!
//! Provides the mathematical core of Wordle: determining tile feedback
//! ([`LetterFeedback::Correct`], [`LetterFeedback::Present`], [`LetterFeedback::Absent`])
//! for each letter in a 5-letter guess relative to a secret answer.
//!
//! # The Two-Pass Matching Algorithm
//!
//! A naive algorithm that simply checks `answer.contains(guess[i])` produces incorrect
//! results when either the answer or guess contains duplicate letters.
//! For example:
//! - Answer: `APPLE` (one `L`, two `P`s)
//! - Guess: `ALLEY` (two `L`s)
//!
//! The first `L` in `ALLEY` is yellow (Present), but the second `L` must be gray (Absent)
//! because `APPLE` contains only one `L`.
//!
//! To guarantee exact frequency conservation, we employ a canonical **two-pass algorithm**:
//! 1. **Pass 1 (Exact Green Matches):** Identifies all positions where `guess[i] == answer[i]`.
//!    These exact matches claim the corresponding letter occurrences in the answer.
//! 2. **Pass 2 (Yellow Allocation):** Counts remaining unmatched letters in the answer using
//!    a 26-element frequency array (`[usize; 26]`). Unmatched guessed letters are marked
//!    `Present` if their remaining count is $> 0$ (decrementing the count), or `Absent` otherwise.

use crate::word::{Word, WORD_LENGTH};
use std::fmt;

/// The feedback status of an individual letter tile in Wordle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LetterFeedback {
    /// Correct letter in the correct position (Green / 'G').
    Correct,
    /// Letter exists in the secret answer but at a different position (Yellow / 'Y').
    Present,
    /// Letter does not occur among remaining unallocated letters in the answer (Gray / 'X').
    Absent,
}

impl LetterFeedback {
    /// Returns the standard single-character symbolic representation:
    /// - `'G'` for [`LetterFeedback::Correct`]
    /// - `'Y'` for [`LetterFeedback::Present`]
    /// - `'X'` for [`LetterFeedback::Absent`]
    pub fn as_char(&self) -> char {
        match self {
            LetterFeedback::Correct => 'G',
            LetterFeedback::Present => 'Y',
            LetterFeedback::Absent => 'X',
        }
    }
}

impl fmt::Display for LetterFeedback {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_char())
    }
}

/// The complete 5-tile feedback for a Wordle guess.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Feedback {
    pub states: [LetterFeedback; WORD_LENGTH],
}

impl Feedback {
    /// Creates a `Feedback` from an explicit array of states.
    pub const fn new(states: [LetterFeedback; WORD_LENGTH]) -> Self {
        Self { states }
    }

    /// Returns `true` if all 5 tiles are [`LetterFeedback::Correct`] (winning condition).
    pub fn is_all_correct(&self) -> bool {
        self.states.iter().all(|&s| s == LetterFeedback::Correct)
    }

    /// Returns the states as an immutable slice.
    pub fn as_slice(&self) -> &[LetterFeedback; WORD_LENGTH] {
        &self.states
    }

    /// Total number of unique feedback patterns for a 5-letter word ($3^5 = 243$).
    pub const PATTERN_COUNT: usize = 243;

    /// Formats the feedback as a 5-character string (e.g. `"GGGGX"`).
    pub fn to_string_pattern(&self) -> String {
        self.states.iter().map(|s| s.as_char()).collect()
    }

    /// Converts the 5-tile feedback into a unique deterministic integer identifier in the range `0..243`.
    ///
    /// # Base-3 Encoding Scheme
    /// Position $i \in [0, 4]$ contributes $v_i \cdot 3^i$, where:
    /// - `Absent` $\rightarrow 0$
    /// - `Present` $\rightarrow 1$
    /// - `Correct` $\rightarrow 2$
    ///
    /// This establishes a bijective mapping between the $3^5 = 243$ feedback states and `0..=242`.
    #[inline]
    pub fn to_pattern_id(&self) -> usize {
        let mut id = 0;
        let mut multiplier = 1;
        for &state in &self.states {
            let val = match state {
                LetterFeedback::Absent => 0,
                LetterFeedback::Present => 1,
                LetterFeedback::Correct => 2,
            };
            id += val * multiplier;
            multiplier *= 3;
        }
        id
    }

    /// Reconstructs a `Feedback` instance from a valid base-3 pattern ID (`0..243`).
    ///
    /// Returns `None` if `id >= FEEDBACK_PATTERN_COUNT`.
    #[inline]
    pub fn from_pattern_id(mut id: usize) -> Option<Self> {
        if id >= Self::PATTERN_COUNT {
            return None;
        }
        let mut states = [LetterFeedback::Absent; WORD_LENGTH];
        for state in &mut states {
            *state = match id % 3 {
                0 => LetterFeedback::Absent,
                1 => LetterFeedback::Present,
                2 => LetterFeedback::Correct,
                _ => unreachable!(),
            };
            id /= 3;
        }
        Some(Self { states })
    }
}

/// The total number of unique feedback patterns for a 5-letter word ($3^5 = 243$).
pub const FEEDBACK_PATTERN_COUNT: usize = Feedback::PATTERN_COUNT;

impl fmt::Display for Feedback {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} {} {} {} {}",
            self.states[0], self.states[1], self.states[2], self.states[3], self.states[4]
        )
    }
}

/// Calculates the exact two-pass Wordle feedback for a guess against a secret answer.
///
/// # Invariants Guaranteed
/// 1. Exactly 5 states are returned.
/// 2. If `guess[i] == answer[i]`, `states[i]` is strictly `Correct`.
/// 3. The total number of `Correct` + `Present` for any letter `c` never exceeds
///    the total count of `c` in `answer`.
/// 4. Repeated runs with identical inputs produce identical deterministic output.
///
/// # Complexity
/// - Time Complexity: $O(L) = O(5) = O(1)$
/// - Space Complexity: $O(1)$ stack allocation (26-element `usize` frequency table).
pub fn calculate_feedback(answer: &Word, guess: &Word) -> Feedback {
    let a_bytes = answer.as_bytes();
    let g_bytes = guess.as_bytes();

    let mut states = [LetterFeedback::Absent; WORD_LENGTH];
    let mut remaining = [0usize; 26];

    // PASS 1: Identify all exact matches (Green)
    for i in 0..WORD_LENGTH {
        if g_bytes[i] == a_bytes[i] {
            states[i] = LetterFeedback::Correct;
        } else {
            // Count letters in answer that were NOT matched by exact green tiles
            let idx = (a_bytes[i] - b'A') as usize;
            remaining[idx] += 1;
        }
    }

    // PASS 2: Allocate remaining letters to non-exact positions (Yellow vs Gray)
    for i in 0..WORD_LENGTH {
        if states[i] != LetterFeedback::Correct {
            let idx = (g_bytes[i] - b'A') as usize;
            if remaining[idx] > 0 {
                states[i] = LetterFeedback::Present;
                remaining[idx] -= 1;
            } else {
                states[i] = LetterFeedback::Absent;
            }
        }
    }

    Feedback { states }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    /// Independent reference implementation using explicit character masking.
    ///
    /// Implements feedback via an alternative mechanism:
    /// - Stores mutable answer characters in a `[Option<u8>; 5]`.
    /// - First consumes matching characters for Green.
    /// - Then linearly searches for unconsumed matching characters for Yellow.
    pub fn reference_calculate_feedback(answer: &Word, guess: &Word) -> Feedback {
        let a = answer.as_bytes();
        let g = guess.as_bytes();
        let mut available = [Some(a[0]), Some(a[1]), Some(a[2]), Some(a[3]), Some(a[4])];
        let mut states = [LetterFeedback::Absent; 5];

        // Pass 1: Green matches
        for i in 0..5 {
            if g[i] == a[i] {
                states[i] = LetterFeedback::Correct;
                available[i] = None; // consumed
            }
        }

        // Pass 2: Yellow matches
        for i in 0..5 {
            if states[i] == LetterFeedback::Absent {
                for slot in &mut available {
                    if *slot == Some(g[i]) {
                        states[i] = LetterFeedback::Present;
                        *slot = None; // consumed
                        break;
                    }
                }
            }
        }

        Feedback { states }
    }

    fn check_case(ans: &str, gss: &str, expected_pattern: &str) {
        let answer = Word::parse(ans).unwrap();
        let guess = Word::parse(gss).unwrap();
        let result = calculate_feedback(&answer, &guess);
        let ref_result = reference_calculate_feedback(&answer, &guess);

        assert_eq!(
            result.to_string_pattern(),
            expected_pattern,
            "Failed for answer={ans}, guess={gss}"
        );
        assert_eq!(
            result, ref_result,
            "Reference implementation mismatch for answer={ans}, guess={gss}"
        );
    }

    #[test]
    fn test_required_duplicate_letter_cases() {
        // From project specification table:
        check_case("APPLE", "APPLE", "GGGGG");
        check_case("APPLE", "ALLEY", "GYXYX");
        check_case("APPLE", "AAAAA", "GXXXX");
        check_case("LEVEL", "ELVES", "YYGGX");
        check_case("BANAL", "ALLAY", "YYXGX");
        check_case("CRATE", "TRACE", "YGGYG");
    }

    #[test]
    fn test_all_absent_and_all_present() {
        check_case("ROAST", "PLUME", "XXXXX");
        // S L A T E vs T E A L S: A at index 2 is an exact match (Green)
        check_case("SLATE", "TEALS", "YYGYY");
    }

    #[test]
    fn test_multiple_occurrences_in_guess_fewer_in_answer() {
        // ROAST has 1 'R', EERIE has 'R' at index 2
        check_case("ROAST", "EERIE", "XXYXX");
        // CRANE has 1 'E' at index 4; GEESE has 'E' at index 4 (Green) and other 'E's (Absent)
        check_case("CRANE", "GEESE", "XXXXG");
    }

    #[test]
    fn test_multiple_occurrences_in_answer_fewer_in_guess() {
        // Answer has 3 'E's at 1, 2, 4. Guess CRANE has 'E' at index 4 (Green match)
        check_case("GEESE", "CRANE", "XXXXG");
        // Answer has 2 'A's at 1 and 3; guess APART has 'A' at 0 and 2 (both Yellow)
        check_case("BANAL", "APART", "YXYXX");
    }

    #[test]
    fn test_is_all_correct_and_display() {
        let ans = Word::parse("CRANE").unwrap();
        let gss = Word::parse("CRANE").unwrap();
        let fb = calculate_feedback(&ans, &gss);
        assert!(fb.is_all_correct());
        assert_eq!(fb.to_string(), "G G G G G");
    }

    #[test]
    fn test_differential_randomized_feedback_pairs() {
        // Deterministic pseudo-random generation of test words
        use rand::rngs::StdRng;
        use rand::{Rng, SeedableRng};

        let mut rng = StdRng::seed_from_u64(0xFACE_CAFE_DEAD_BEEF);
        let alphabet = b"ABCDE"; // small alphabet guarantees heavy duplicate collisions

        for _ in 0..1000 {
            let mut a_bytes = [0u8; 5];
            let mut g_bytes = [0u8; 5];
            for i in 0..5 {
                a_bytes[i] = alphabet[rng.gen_range(0..alphabet.len())];
                g_bytes[i] = alphabet[rng.gen_range(0..alphabet.len())];
            }

            let ans = Word::parse(std::str::from_utf8(&a_bytes).unwrap()).unwrap();
            let gss = Word::parse(std::str::from_utf8(&g_bytes).unwrap()).unwrap();

            let prod = calculate_feedback(&ans, &gss);
            let refer = reference_calculate_feedback(&ans, &gss);

            assert_eq!(
                prod,
                refer,
                "Mismatch on random pair: ans={}, gss={}",
                ans.as_str(),
                gss.as_str()
            );

            // Verify property: identical inputs produce identical outputs
            assert_eq!(prod, calculate_feedback(&ans, &gss));

            // Verify property: exact match produces GGGGG
            let self_match = calculate_feedback(&ans, &ans);
            assert!(self_match.is_all_correct());

            // Verify count property: count(G) + count(Y) for letter <= count in answer
            let mut ans_counts = [0usize; 26];
            for &b in ans.as_bytes() {
                ans_counts[(b - b'A') as usize] += 1;
            }
            let mut feedback_counts = [0usize; 26];
            for i in 0..5 {
                if prod.states[i] != LetterFeedback::Absent {
                    let idx = (gss.as_bytes()[i] - b'A') as usize;
                    feedback_counts[idx] += 1;
                }
            }
            for i in 0..26 {
                assert!(
                    feedback_counts[i] <= ans_counts[i],
                    "Allocated more feedback than answer letters for character index {i}"
                );
            }
        }
    }

    #[test]
    fn test_pattern_id_round_trip_all_243() {
        use std::collections::HashSet;
        let mut seen_ids = HashSet::new();

        for id in 0..FEEDBACK_PATTERN_COUNT {
            let fb = Feedback::from_pattern_id(id).expect("valid ID in range 0..243");
            let encoded = fb.to_pattern_id();
            assert_eq!(encoded, id, "Round-trip failed for pattern ID {id}");
            assert!(seen_ids.insert(encoded), "Duplicate pattern ID: {id}");
        }

        assert_eq!(seen_ids.len(), 243);
        assert_eq!(Feedback::from_pattern_id(243), None);
        assert_eq!(Feedback::from_pattern_id(1000), None);
    }

    #[test]
    fn test_pattern_id_specific_extremes() {
        // All Absent = 0
        let all_absent = Feedback::new([LetterFeedback::Absent; 5]);
        assert_eq!(all_absent.to_pattern_id(), 0);
        assert_eq!(Feedback::from_pattern_id(0), Some(all_absent));

        // All Correct = 242 (2 + 2*3 + 2*9 + 2*27 + 2*81 = 2 * 121 = 242)
        let all_correct = Feedback::new([LetterFeedback::Correct; 5]);
        assert_eq!(all_correct.to_pattern_id(), 242);
        assert_eq!(Feedback::from_pattern_id(242), Some(all_correct));

        // Mixed pattern: [Present, Absent, Correct, Absent, Present]
        // 1*1 + 0*3 + 2*9 + 0*27 + 1*81 = 1 + 18 + 81 = 100
        let mixed = Feedback::new([
            LetterFeedback::Present,
            LetterFeedback::Absent,
            LetterFeedback::Correct,
            LetterFeedback::Absent,
            LetterFeedback::Present,
        ]);
        assert_eq!(mixed.to_pattern_id(), 100);
        assert_eq!(Feedback::from_pattern_id(100), Some(mixed));
    }
}
