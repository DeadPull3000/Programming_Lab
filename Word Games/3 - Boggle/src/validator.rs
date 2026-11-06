//! # Boggle Word Path and Dictionary Validation
//!
//! Provides recursive depth-first search with backtracking to determine if a word can be formed
//! on a Boggle board according to physical adjacency rules (no cell reuse, 8 directions).
//! Also includes dictionary validation, optional Qu tile support, and an automatic Trie-guided solver.

use crate::board::Board;
use std::collections::HashSet;
use trie::{Trie, TrieCursor};

/// Minimum word length required for Boggle guesses.
pub const MIN_BOGGLE_LENGTH: usize = 3;

/// Checks whether a given word can be formed on the board via adjacent cells without cell reuse.
///
/// Invariants:
/// - Word must be at least 3 letters.
/// - Letters must be horizontally, vertically, or diagonally adjacent.
/// - The exact same physical cell cannot be reused within a single word.
/// - Different physical cells with identical letters may be used.
pub fn can_form_word(board: &Board, word: &str) -> bool {
    let trimmed = word.trim();
    if trimmed.len() < MIN_BOGGLE_LENGTH || trimmed.len() > board.cells.len() {
        return false;
    }

    let chars: Vec<char> = trimmed.chars().map(|c| c.to_ascii_uppercase()).collect();
    for &c in &chars {
        if !c.is_ascii_uppercase() {
            return false;
        }
    }

    let mut visited = vec![false; board.cells.len()];

    // Attempt to start the word from any matching board cell
    for start_idx in 0..board.cells.len() {
        if board.cells[start_idx] == chars[0]
            && dfs_search_path(board, start_idx, 1, &chars, &mut visited)
        {
            return true;
        }
    }

    false
}

/// Recursive DFS helper for single-word path validation with backtracking.
fn dfs_search_path(
    board: &Board,
    current_idx: usize,
    char_idx: usize,
    chars: &[char],
    visited: &mut [bool],
) -> bool {
    if char_idx == chars.len() {
        return true;
    }

    visited[current_idx] = true;

    for neighbor_idx in board.get_neighbor_indices(current_idx) {
        if !visited[neighbor_idx]
            && board.cells[neighbor_idx] == chars[char_idx]
            && dfs_search_path(board, neighbor_idx, char_idx + 1, chars, visited)
        {
            visited[current_idx] = false; // Restore visited state before returning
            return true;
        }
    }

    visited[current_idx] = false; // Backtrack
    false
}

/// Checks whether a word exists in the dictionary Trie.
#[inline]
pub fn is_dictionary_word(trie: &Trie, word: &str) -> bool {
    trie.contains(word)
}

/// Path validation with standard Boggle Qu tile support (Phase B-10).
///
/// When encountering 'Q' on the board, it can match either a single 'Q'
/// or the two-letter sequence 'Q', 'U' in the candidate word while occupying 1 cell.
pub fn can_form_word_with_qu(board: &Board, word: &str) -> bool {
    let trimmed = word.trim();
    if trimmed.len() < MIN_BOGGLE_LENGTH {
        return false;
    }

    let chars: Vec<char> = trimmed.chars().map(|c| c.to_ascii_uppercase()).collect();
    for &c in &chars {
        if !c.is_ascii_uppercase() {
            return false;
        }
    }

    let mut visited = vec![false; board.cells.len()];

    for start_idx in 0..board.cells.len() {
        let cell_ch = board.cells[start_idx];
        // Standard single-letter match
        if cell_ch == chars[0] && dfs_search_qu(board, start_idx, 1, &chars, &mut visited) {
            return true;
        }
        // Qu tile match: cell is 'Q' and word starts with "QU"
        if cell_ch == 'Q'
            && chars.len() >= 2
            && chars[0] == 'Q'
            && chars[1] == 'U'
            && dfs_search_qu(board, start_idx, 2, &chars, &mut visited)
        {
            return true;
        }
    }

    false
}

fn dfs_search_qu(
    board: &Board,
    current_idx: usize,
    char_idx: usize,
    chars: &[char],
    visited: &mut [bool],
) -> bool {
    if char_idx == chars.len() {
        return true;
    }

    visited[current_idx] = true;

    for neighbor_idx in board.get_neighbor_indices(current_idx) {
        if !visited[neighbor_idx] {
            let nbr_ch = board.cells[neighbor_idx];
            // Case 1: single character match
            if nbr_ch == chars[char_idx]
                && dfs_search_qu(board, neighbor_idx, char_idx + 1, chars, visited)
            {
                visited[current_idx] = false;
                return true;
            }
            // Case 2: Qu tile match (nbr is 'Q' and remaining chars start with 'Q', 'U')
            if nbr_ch == 'Q'
                && char_idx + 1 < chars.len()
                && chars[char_idx] == 'Q'
                && chars[char_idx + 1] == 'U'
                && dfs_search_qu(board, neighbor_idx, char_idx + 2, chars, visited)
            {
                visited[current_idx] = false;
                return true;
            }
        }
    }

    visited[current_idx] = false;
    false
}

/// Automatic Boggle Solver using Trie-guided DFS (Phase B-11).
///
/// Discovers every dictionary word of length $\ge 3$ that can be formed on the board.
/// Uses the `TrieCursor` to prune non-existent prefix subtrees immediately.
pub fn solve_boggle_board(trie: &Trie, board: &Board) -> HashSet<String> {
    let mut solutions = HashSet::new();
    let mut visited = vec![false; board.cells.len()];
    let mut word_buffer = String::with_capacity(board.cells.len());
    let root = trie.root_cursor();

    for start_idx in 0..board.cells.len() {
        let ch = board.cells[start_idx];
        if let Some(cursor) = root.advance(ch) {
            word_buffer.push(ch);
            visited[start_idx] = true;

            dfs_solve(
                board,
                start_idx,
                cursor,
                &mut word_buffer,
                &mut visited,
                &mut solutions,
            );

            visited[start_idx] = false;
            word_buffer.pop();
        }
    }

    solutions
}

fn dfs_solve(
    board: &Board,
    current_idx: usize,
    cursor: TrieCursor<'_>,
    word_buffer: &mut String,
    visited: &mut [bool],
    solutions: &mut HashSet<String>,
) {
    if word_buffer.len() >= MIN_BOGGLE_LENGTH && cursor.is_word() {
        solutions.insert(word_buffer.clone());
    }

    for neighbor_idx in board.get_neighbor_indices(current_idx) {
        if !visited[neighbor_idx] {
            let nbr_ch = board.cells[neighbor_idx];
            if let Some(next_cursor) = cursor.advance(nbr_ch) {
                visited[neighbor_idx] = true;
                word_buffer.push(nbr_ch);

                dfs_solve(
                    board,
                    neighbor_idx,
                    next_cursor,
                    word_buffer,
                    visited,
                    solutions,
                );

                word_buffer.pop();
                visited[neighbor_idx] = false;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Constructs a deterministic 4x4 test board:
    ///
    ///   +---+---+---+---+
    ///   | P | E | A | C |
    ///   +---+---+---+---+
    ///   | L | A | T | E |
    ///   +---+---+---+---+
    ///   | R | O | S | E |
    ///   +---+---+---+---+
    ///   | D | U | C | K |
    ///   +---+---+---+---+
    fn sample_4x4_board() -> Board {
        let cells = vec![
            'P', 'E', 'A', 'C', 'L', 'A', 'T', 'E', 'R', 'O', 'S', 'E', 'D', 'U', 'C', 'K',
        ];
        Board::new(4, cells).unwrap()
    }

    #[test]
    fn test_horizontal_vertical_diagonal_paths() {
        let board = sample_4x4_board();

        // 1. Horizontal: "PEAC", "LATE", "ROSE", "DUCK"
        assert!(can_form_word(&board, "LATE"));
        assert!(can_form_word(&board, "ROSE"));
        assert!(can_form_word(&board, "DUCK"));

        // 2. Vertical: (0,0)->(1,0)->(2,0) = "PLR"
        assert!(can_form_word(&board, "PLR"));

        // 3. Diagonal: (0,0)->(1,1)->(2,2)->(3,3) = "PASK"
        assert!(can_form_word(&board, "PASK"));

        // 4. Zig-zag / multi-directional: (0,0) P -> (0,1) E -> (1,1) A -> (1,0) L = "PEAL"
        assert!(can_form_word(&board, "PEAL"));
    }

    #[test]
    fn test_non_neighbor_jump_rejected() {
        let board = sample_4x4_board();
        // 'P' at (0,0) to 'K' at (3,3) cannot be reached in 2 steps
        assert!(!can_form_word(&board, "PK"));
    }

    #[test]
    fn test_cell_reuse_rejected() {
        let board = sample_4x4_board();
        // Trying to form "LAL" where only one 'L' is in row 1, col 0.
        // It has adjacent 'A' at (1,1), but to get second 'L' it would need to reuse (1,0)
        assert!(!can_form_word(&board, "LAL"));
    }

    #[test]
    fn test_repeated_letters_in_different_cells_accepted() {
        let board = sample_4x4_board();
        // Board has two 'A's: (0,2) and (1,1)
        // Path (0,1) E -> (0,2) A -> (1,2) T -> (1,1) A = "EATA"
        assert!(can_form_word(&board, "EATA"));
    }

    #[test]
    fn test_minimum_length_and_out_of_bounds() {
        let board = sample_4x4_board();
        assert!(!can_form_word(&board, "PE")); // too short (len 2)
        assert!(!can_form_word(&board, ""));
        assert!(!can_form_word(&board, "PEACEFULNESSLONGER")); // > 16 chars
    }

    #[test]
    fn test_qu_tile_support() {
        let cells = vec![
            'Q', 'I', 'C', 'K', 'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L',
        ];
        let board = Board::new(4, cells).unwrap();

        // With Qu support, 'Q' matches "QU", followed by 'I', 'C', 'K' = "QUICK"
        assert!(can_form_word_with_qu(&board, "QUICK"));
        // Without Qu support, only 'Q' matches 1 char, so "QUICK" fails
        assert!(!can_form_word(&board, "QUICK"));
    }

    #[test]
    fn test_automatic_solver_with_controlled_trie() {
        let mut trie = Trie::new();
        trie.insert("LATE").unwrap();
        trie.insert("ROSE").unwrap();
        trie.insert("DUCK").unwrap();
        trie.insert("PEAL").unwrap();
        trie.insert("ZEBRA").unwrap(); // Not on board

        let board = sample_4x4_board();
        let found = solve_boggle_board(&trie, &board);

        assert!(found.contains("LATE"));
        assert!(found.contains("ROSE"));
        assert!(found.contains("DUCK"));
        assert!(found.contains("PEAL"));
        assert!(!found.contains("ZEBRA"));

        for w in &found {
            assert!(can_form_word(&board, w));
            assert!(trie.contains(w));
        }
    }
}
