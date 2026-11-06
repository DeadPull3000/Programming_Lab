//! # Spelling Bee Solution Enumeration (Trie-guided DFS)
//!
//! Provides the primary algorithmic solver for Spelling Bee puzzles using a Trie cursor
//! to explore only valid dictionary paths without redundant allocations or brute-force scanning.
//! Also provides an independent brute-force reference solver for verification.

use crate::puzzle::SpellingBeePuzzle;
use std::collections::HashSet;
use trie::{Trie, TrieCursor};

/// Minimum word length required for valid Spelling Bee solutions.
pub const MIN_WORD_LENGTH: usize = 4;

/// Generates all valid dictionary words for a given Spelling Bee puzzle using Trie-guided DFS.
///
/// A valid solution satisfies:
/// 1. Length $\ge 4$.
/// 2. Only contains letters from the 7-letter puzzle.
/// 3. Contains the puzzle's mandatory main letter.
/// 4. Exists in the dictionary as a complete word.
///
/// # Complexity
///
/// Time: $O(V_{valid})$ where $V_{valid}$ is the number of Trie nodes reachable via the 7
/// permitted letters. Completely prunes non-existent branches.
/// Space: $O(L_{max})$ recursion depth on the stack, where $L_{max}$ is the length of the longest
/// matching word, plus the storage for the returned solution set.
pub fn solve_spelling_bee(trie: &Trie, puzzle: &SpellingBeePuzzle) -> HashSet<String> {
    let mut solutions = HashSet::new();
    let mut current_word = String::with_capacity(32);
    let root_cursor = trie.root_cursor();

    dfs_traverse(root_cursor, &mut current_word, 0, puzzle, &mut solutions);

    solutions
}

/// Recursive DFS helper traversing only existing Trie branches corresponding to the 7 puzzle letters.
fn dfs_traverse(
    cursor: TrieCursor<'_>,
    current_word: &mut String,
    mask: u8,
    puzzle: &SpellingBeePuzzle,
    solutions: &mut HashSet<String>,
) {
    for (idx, &letter) in puzzle.letters.iter().enumerate() {
        if let Some(next_cursor) = cursor.advance(letter) {
            current_word.push(letter);
            let next_mask = mask | (1 << idx);

            // Check if current prefix is a valid completed word meeting all criteria
            if next_cursor.is_word()
                && current_word.len() >= MIN_WORD_LENGTH
                && SpellingBeePuzzle::mask_has_main_letter(next_mask)
            {
                solutions.insert(current_word.clone());
            }

            // Even if next_cursor is a word, words with prefixes can continue
            // (e.g. "CARE" -> "CAREFUL" or "POLL" -> "POLLEN")
            dfs_traverse(next_cursor, current_word, next_mask, puzzle, solutions);

            // Backtrack: pop last character before trying next letter in this stack frame
            current_word.pop();
        }
    }
}

/// Independent brute-force reference solver.
///
/// Iterates over every word in an input collection and filters it according to puzzle rules.
/// Used to verify the optimized Trie-based solver produces identical results.
pub fn brute_force_reference_solve<I, S>(words: I, puzzle: &SpellingBeePuzzle) -> HashSet<String>
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    let mut solutions = HashSet::new();

    for word_ref in words {
        let raw = word_ref.as_ref().trim();
        if raw.len() < MIN_WORD_LENGTH {
            continue;
        }

        let mut has_main = false;
        let mut all_valid = true;

        for ch in raw.chars() {
            let upper = ch.to_ascii_uppercase();
            if !puzzle.contains_letter(upper) {
                all_valid = false;
                break;
            }
            if upper == puzzle.main_letter {
                has_main = true;
            }
        }

        if all_valid && has_main {
            solutions.insert(raw.to_ascii_uppercase());
        }
    }

    solutions
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_solver_with_controlled_trie() {
        let mut trie = Trie::new();
        // Permitted puzzle letters: ['R', 'A', 'T', 'E', 'C', 'L', 'P'], main letter 'R'
        let words = [
            "RATE",     // valid (len 4, contains R)
            "RATER",    // valid (len 5, contains R, repeated R)
            "RAT",      // invalid (too short, len 3)
            "LATE",     // invalid (missing main letter 'R')
            "PLATE",    // invalid (missing main letter 'R')
            "PRATE",    // valid (len 5, contains R)
            "PARTERRE", // valid (len 8, contains R, pangram)
            "RACE",     // valid (len 4, contains R)
            "RACECAR",  // valid (len 7, contains R)
            "DOG",      // invalid (contains non-puzzle letters)
            "ROBOT",    // invalid ('O', 'B' not in puzzle)
        ];

        for w in &words {
            trie.insert(w).unwrap();
        }

        let letters = ['R', 'A', 'T', 'E', 'C', 'L', 'P'];
        let puzzle = SpellingBeePuzzle::new(letters, 'R').unwrap();

        let trie_solutions = solve_spelling_bee(&trie, &puzzle);
        let reference_solutions = brute_force_reference_solve(words, &puzzle);

        // Verification against independent reference implementation
        assert_eq!(trie_solutions, reference_solutions);

        // Explicit membership checks
        assert!(trie_solutions.contains("RATE"));
        assert!(trie_solutions.contains("RATER"));
        assert!(trie_solutions.contains("PRATE"));
        assert!(trie_solutions.contains("RACE"));
        assert!(trie_solutions.contains("RACECAR"));
        assert!(!trie_solutions.contains("RAT")); // too short
        assert!(!trie_solutions.contains("LATE")); // no 'R'
        assert!(!trie_solutions.contains("PLATE")); // no 'R'
        assert!(!trie_solutions.contains("DOG")); // invalid letters
    }

    #[test]
    fn test_solver_empty_trie_and_unsolvable_puzzle() {
        let empty_trie = Trie::new();
        let letters = ['A', 'B', 'C', 'D', 'E', 'F', 'G'];
        let puzzle = SpellingBeePuzzle::new(letters, 'A').unwrap();

        let solutions = solve_spelling_bee(&empty_trie, &puzzle);
        assert!(solutions.is_empty());

        let mut trie = Trie::new();
        trie.insert("XYZ").unwrap();
        trie.insert("WXYZ").unwrap();
        let solutions2 = solve_spelling_bee(&trie, &puzzle);
        assert!(solutions2.is_empty());
    }
}
