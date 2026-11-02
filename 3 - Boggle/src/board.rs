//! # Boggle Board Representation, Coordinate Mapping, and Adjacency
//!
//! Provides the $N \times N$ Boggle grid data structure (for dimensions 4, 5, and 6),
//! 8-directional neighbor discovery, index-to-coordinate conversions, random board generation,
//! and clean visual grid display.

use rand::Rng;
use std::fmt;

/// Valid board dimensions supported by Boggle.
pub const VALID_DIMENSIONS: [usize; 3] = [4, 5, 6];

/// Eight possible neighbor directions: horizontal, vertical, and diagonal.
pub const DIRECTIONS: [(isize, isize); 8] = [
    (-1, -1), // Top-Left
    (-1, 0),  // Top
    (-1, 1),  // Top-Right
    (0, -1),  // Left
    (0, 1),   // Right
    (1, -1),  // Bottom-Left
    (1, 0),   // Bottom
    (1, 1),   // Bottom-Right
];

/// Errors produced during board creation or manipulation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BoardError {
    /// Dimension is not 4, 5, or 6.
    InvalidDimension(usize),
    /// The number of cells does not equal `size * size`.
    CellCountMismatch { expected: usize, found: usize },
    /// A cell character is not an uppercase ASCII letter ('A'..='Z').
    InvalidCharacter(char),
}

impl fmt::Display for BoardError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BoardError::InvalidDimension(d) => {
                write!(
                    f,
                    "invalid board dimension {d}x{d}; only 4x4, 5x5, and 6x6 are supported"
                )
            }
            BoardError::CellCountMismatch { expected, found } => {
                write!(
                    f,
                    "cell count mismatch: expected {expected} cells, but found {found}"
                )
            }
            BoardError::InvalidCharacter(c) => {
                write!(
                    f,
                    "invalid cell character '{c}'; must be uppercase ASCII 'A'-'Z'"
                )
            }
        }
    }
}

impl std::error::Error for BoardError {}

/// An $N \times N$ Boggle game board stored as a contiguous 1D vector.
///
/// Flattened representation:
/// - Coordinate `(row, col)` maps to `row * size + col`.
/// - 1D index `idx` maps to `(idx / size, idx % size)`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Board {
    /// Grid dimension (4, 5, or 6).
    pub size: usize,
    /// Flattened row-major array of uppercase ASCII characters.
    pub cells: Vec<char>,
}

impl Board {
    /// Constructs a validated `Board` from dimension and cell characters.
    pub fn new(size: usize, cells: Vec<char>) -> Result<Self, BoardError> {
        if !VALID_DIMENSIONS.contains(&size) {
            return Err(BoardError::InvalidDimension(size));
        }

        let expected_cells = size * size;
        if cells.len() != expected_cells {
            return Err(BoardError::CellCountMismatch {
                expected: expected_cells,
                found: cells.len(),
            });
        }

        for &c in &cells {
            if !c.is_ascii_uppercase() {
                return Err(BoardError::InvalidCharacter(c));
            }
        }

        Ok(Self { size, cells })
    }

    /// Generates a random board of dimension `size` using the provided RNG.
    ///
    /// Letters are chosen uniformly from uppercase ASCII 'A'..='Z'.
    /// Repetition of letters across cells is explicitly allowed.
    pub fn random<R: Rng>(size: usize, rng: &mut R) -> Result<Self, BoardError> {
        if !VALID_DIMENSIONS.contains(&size) {
            return Err(BoardError::InvalidDimension(size));
        }

        let count = size * size;
        let mut cells = Vec::with_capacity(count);
        for _ in 0..count {
            let ch = rng.gen_range(b'A'..=b'Z') as char;
            cells.push(ch);
        }

        Self::new(size, cells)
    }

    /// Converts 2D `(row, col)` coordinates to a flattened 1D cell index.
    #[inline]
    pub fn to_index(&self, row: usize, col: usize) -> Option<usize> {
        if row < self.size && col < self.size {
            Some(row * self.size + col)
        } else {
            None
        }
    }

    /// Converts a flattened 1D cell index to 2D `(row, col)` coordinates.
    #[inline]
    pub fn to_coords(&self, index: usize) -> Option<(usize, usize)> {
        if index < self.cells.len() {
            Some((index / self.size, index % self.size))
        } else {
            None
        }
    }

    /// Retrieves the character at coordinate `(row, col)`.
    #[inline]
    pub fn get_cell(&self, row: usize, col: usize) -> Option<char> {
        let idx = self.to_index(row, col)?;
        Some(self.cells[idx])
    }

    /// Retrieves the character at a 1D cell index.
    #[inline]
    pub fn get_cell_by_index(&self, index: usize) -> Option<char> {
        self.cells.get(index).copied()
    }

    /// Returns the coordinates of all valid 8-directional neighbors of `(row, col)`.
    ///
    /// Uses signed integer offsets and checks boundary conditions `0 <= r, c < size`.
    /// Excludes the current cell itself.
    pub fn get_neighbors(&self, row: usize, col: usize) -> Vec<(usize, usize)> {
        let mut neighbors = Vec::with_capacity(8);
        let s = self.size as isize;

        for (dr, dc) in DIRECTIONS {
            let nr = row as isize + dr;
            let nc = col as isize + dc;

            if nr >= 0 && nr < s && nc >= 0 && nc < s {
                neighbors.push((nr as usize, nc as usize));
            }
        }

        neighbors
    }

    /// Returns the flattened 1D indices of all valid 8-directional neighbors of `index`.
    pub fn get_neighbor_indices(&self, index: usize) -> Vec<usize> {
        match self.to_coords(index) {
            Some((row, col)) => self
                .get_neighbors(row, col)
                .into_iter()
                .map(|(r, c)| r * self.size + c)
                .collect(),
            None => Vec::new(),
        }
    }

    /// Formats the board into a neat terminal ASCII grid.
    pub fn display_grid(&self) -> String {
        let mut out = String::new();
        let border = "+---".repeat(self.size) + "+";

        for row in 0..self.size {
            out.push_str("  ");
            out.push_str(&border);
            out.push('\n');

            out.push_str("  |");
            for col in 0..self.size {
                let ch = self.get_cell(row, col).unwrap_or(' ');
                out.push_str(&format!(" {} |", ch));
            }
            out.push('\n');
        }

        out.push_str("  ");
        out.push_str(&border);
        out.push('\n');

        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::rngs::StdRng;
    use rand::SeedableRng;

    #[test]
    fn test_valid_board_construction_all_dimensions() {
        for &size in &[4, 5, 6] {
            let cells = vec!['A'; size * size];
            let board = Board::new(size, cells).unwrap();
            assert_eq!(board.size, size);
            assert_eq!(board.cells.len(), size * size);
        }
    }

    #[test]
    fn test_invalid_dimensions_rejected() {
        assert_eq!(
            Board::new(3, vec!['A'; 9]).unwrap_err(),
            BoardError::InvalidDimension(3)
        );
        assert_eq!(
            Board::new(7, vec!['A'; 49]).unwrap_err(),
            BoardError::InvalidDimension(7)
        );
    }

    #[test]
    fn test_cell_count_mismatch_rejected() {
        assert_eq!(
            Board::new(4, vec!['A'; 15]).unwrap_err(),
            BoardError::CellCountMismatch {
                expected: 16,
                found: 15
            }
        );
        assert_eq!(
            Board::new(4, vec!['A'; 17]).unwrap_err(),
            BoardError::CellCountMismatch {
                expected: 16,
                found: 17
            }
        );
    }

    #[test]
    fn test_invalid_characters_rejected() {
        let mut cells = vec!['A'; 16];
        cells[5] = '1';
        assert_eq!(
            Board::new(4, cells).unwrap_err(),
            BoardError::InvalidCharacter('1')
        );

        let mut cells_lower = vec!['A'; 16];
        cells_lower[0] = 'a';
        assert_eq!(
            Board::new(4, cells_lower).unwrap_err(),
            BoardError::InvalidCharacter('a')
        );
    }

    #[test]
    fn test_coordinate_mapping() {
        let cells = vec!['A'; 16];
        let board = Board::new(4, cells).unwrap();

        // Corners
        assert_eq!(board.to_index(0, 0), Some(0));
        assert_eq!(board.to_coords(0), Some((0, 0)));

        assert_eq!(board.to_index(0, 3), Some(3));
        assert_eq!(board.to_coords(3), Some((0, 3)));

        assert_eq!(board.to_index(3, 0), Some(12));
        assert_eq!(board.to_coords(12), Some((3, 0)));

        assert_eq!(board.to_index(3, 3), Some(15));
        assert_eq!(board.to_coords(15), Some((3, 3)));

        // Out of bounds
        assert_eq!(board.to_index(4, 0), None);
        assert_eq!(board.to_index(0, 4), None);
        assert_eq!(board.to_coords(16), None);
    }

    #[test]
    fn test_neighbor_counts_corner_edge_interior() {
        let cells = vec!['A'; 16];
        let board = Board::new(4, cells).unwrap();

        // 1. Corners must have exactly 3 neighbors
        assert_eq!(board.get_neighbors(0, 0).len(), 3);
        assert_eq!(board.get_neighbors(0, 3).len(), 3);
        assert_eq!(board.get_neighbors(3, 0).len(), 3);
        assert_eq!(board.get_neighbors(3, 3).len(), 3);

        // 2. Edges must have exactly 5 neighbors
        assert_eq!(board.get_neighbors(0, 1).len(), 5);
        assert_eq!(board.get_neighbors(1, 0).len(), 5);
        assert_eq!(board.get_neighbors(2, 3).len(), 5);
        assert_eq!(board.get_neighbors(3, 2).len(), 5);

        // 3. Interior must have exactly 8 neighbors
        assert_eq!(board.get_neighbors(1, 1).len(), 8);
        assert_eq!(board.get_neighbors(2, 2).len(), 8);
        assert_eq!(board.get_neighbors(1, 2).len(), 8);
        assert_eq!(board.get_neighbors(2, 1).len(), 8);
    }

    #[test]
    fn test_neighbor_indices_no_duplicates_or_self() {
        let cells = vec!['A'; 16];
        let board = Board::new(4, cells).unwrap();

        for idx in 0..16 {
            let nbrs = board.get_neighbor_indices(idx);
            assert!(!nbrs.contains(&idx), "Cell cannot be its own neighbor");

            let mut set = std::collections::HashSet::new();
            for n in nbrs {
                assert!(n < 16, "Neighbor index must be within board");
                assert!(set.insert(n), "Neighbor index must be unique");
            }
        }
    }

    #[test]
    fn test_random_board_generation_and_seeded_repeatability() {
        let mut rng1 = StdRng::seed_from_u64(12345);
        let board1 = Board::random(4, &mut rng1).unwrap();

        let mut rng2 = StdRng::seed_from_u64(12345);
        let board2 = Board::random(4, &mut rng2).unwrap();

        assert_eq!(board1, board2);
        assert_eq!(board1.cells.len(), 16);
    }
}
