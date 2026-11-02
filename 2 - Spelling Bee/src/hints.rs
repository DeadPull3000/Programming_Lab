//! # Spelling Bee Hint System (Optional Feature)
//!
//! Generates structural hints for the current puzzle based on the precomputed solution set:
//! 1. **Hint Grid**: A matrix of word counts categorized by first letter and word length,
//!    with per-letter row totals, per-length column totals, and a grand total.
//! 2. **Two-Letter Prefix Counts**: Frequencies of all two-letter starting prefixes.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write;

/// Matrix of solution word counts categorized by starting letter and word length.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HintGrid {
    /// Counts indexed by starting letter and word length: `counts[letter][length]`.
    pub matrix: BTreeMap<char, BTreeMap<usize, usize>>,
    /// Sorted set of all distinct word lengths present among solutions.
    pub lengths: BTreeSet<usize>,
    /// Total count of words per starting letter.
    pub row_totals: BTreeMap<char, usize>,
    /// Total count of words per word length.
    pub col_totals: BTreeMap<usize, usize>,
    /// Grand total of all words across the grid.
    pub grand_total: usize,
}

impl HintGrid {
    /// Computes the hint grid from a collection of valid solutions.
    pub fn compute<'a, I>(solutions: I) -> Self
    where
        I: IntoIterator<Item = &'a String>,
    {
        let mut matrix: BTreeMap<char, BTreeMap<usize, usize>> = BTreeMap::new();
        let mut lengths: BTreeSet<usize> = BTreeSet::new();
        let mut row_totals: BTreeMap<char, usize> = BTreeMap::new();
        let mut col_totals: BTreeMap<usize, usize> = BTreeMap::new();
        let mut grand_total = 0;

        for word in solutions {
            if let Some(first_char) = word.chars().next() {
                let len = word.chars().count();
                lengths.insert(len);

                *matrix
                    .entry(first_char)
                    .or_default()
                    .entry(len)
                    .or_insert(0) += 1;

                *row_totals.entry(first_char).or_insert(0) += 1;
                *col_totals.entry(len).or_insert(0) += 1;
                grand_total += 1;
            }
        }

        Self {
            matrix,
            lengths,
            row_totals,
            col_totals,
            grand_total,
        }
    }

    /// Formats the hint matrix into a clean ASCII table.
    pub fn format_table(&self) -> String {
        let mut out = String::new();
        if self.matrix.is_empty() {
            return "No solutions available to generate hint grid.\n".to_string();
        }

        // Header: Letter | len1 len2 ... | Total
        let _ = write!(out, "{:<6}", "Letter");
        for &len in &self.lengths {
            let _ = write!(out, "{:>5}", len);
        }
        let _ = writeln!(out, " {:>6}", "Total");

        // Separator line
        let width = 6 + self.lengths.len() * 5 + 7;
        let _ = writeln!(out, "{}", "-".repeat(width));

        // Rows for each starting letter
        for (&letter, length_map) in &self.matrix {
            let _ = write!(out, "{:<6}", letter);
            for &len in &self.lengths {
                if let Some(&count) = length_map.get(&len) {
                    let _ = write!(out, "{:>5}", count);
                } else {
                    let _ = write!(out, "{:>5}", "-");
                }
            }
            let total = self.row_totals.get(&letter).copied().unwrap_or(0);
            let _ = writeln!(out, " {:>6}", total);
        }

        // Column totals row
        let _ = writeln!(out, "{}", "-".repeat(width));
        let _ = write!(out, "{:<6}", "Total");
        for &len in &self.lengths {
            let total = self.col_totals.get(&len).copied().unwrap_or(0);
            let _ = write!(out, "{:>5}", total);
        }
        let _ = writeln!(out, " {:>6}", self.grand_total);

        out
    }
}

/// Frequencies of all two-letter prefixes among solutions.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TwoLetterPrefixes {
    /// Counts indexed by the 2-letter uppercase prefix (e.g. "RA" => 3).
    pub counts: BTreeMap<String, usize>,
}

impl TwoLetterPrefixes {
    /// Computes two-letter prefix frequencies from solutions.
    pub fn compute<'a, I>(solutions: I) -> Self
    where
        I: IntoIterator<Item = &'a String>,
    {
        let mut counts: BTreeMap<String, usize> = BTreeMap::new();
        for word in solutions {
            if word.chars().count() >= 2 {
                let prefix: String = word.chars().take(2).collect();
                *counts.entry(prefix).or_insert(0) += 1;
            }
        }
        Self { counts }
    }

    /// Formats the prefix list into a clean multi-column summary.
    pub fn format_summary(&self) -> String {
        if self.counts.is_empty() {
            return "No two-letter prefixes available.\n".to_string();
        }

        let mut out = String::new();
        let items: Vec<String> = self
            .counts
            .iter()
            .map(|(pfx, cnt)| format!("{}: {}", pfx, cnt))
            .collect();

        // Format in lines of 6 items each
        for chunk in items.chunks(6) {
            let _ = writeln!(out, "{}", chunk.join("   "));
        }
        out
    }
}

/// Complete hint package.
#[derive(Debug, Clone)]
pub struct SpellingBeeHints {
    pub grid: HintGrid,
    pub prefixes: TwoLetterPrefixes,
}

impl SpellingBeeHints {
    /// Computes both the length matrix and prefix counts from solutions.
    pub fn compute<'a, I>(solutions: I) -> Self
    where
        I: IntoIterator<Item = &'a String> + Clone,
    {
        let grid = HintGrid::compute(solutions.clone());
        let prefixes = TwoLetterPrefixes::compute(solutions);
        Self { grid, prefixes }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hint_grid_and_prefix_counts() {
        let solutions = vec![
            "RATE".to_string(),     // len 4, pfx RA
            "RATER".to_string(),    // len 5, pfx RA
            "RACE".to_string(),     // len 4, pfx RA
            "RACECAR".to_string(),  // len 7, pfx RA
            "PARTERRE".to_string(), // len 8, pfx PA
            "PLATE".to_string(),    // len 5, pfx PL
        ];

        let hints = SpellingBeeHints::compute(&solutions);

        // Grid tests
        assert_eq!(hints.grid.grand_total, 6);
        assert_eq!(hints.grid.row_totals.get(&'R'), Some(&4));
        assert_eq!(hints.grid.row_totals.get(&'P'), Some(&2));

        // Column totals: len 4 has 2 (RATE, RACE), len 5 has 2 (RATER, PLATE), len 7 has 1, len 8 has 1
        assert_eq!(hints.grid.col_totals.get(&4), Some(&2));
        assert_eq!(hints.grid.col_totals.get(&5), Some(&2));
        assert_eq!(hints.grid.col_totals.get(&7), Some(&1));
        assert_eq!(hints.grid.col_totals.get(&8), Some(&1));

        // Prefix tests
        assert_eq!(hints.prefixes.counts.get("RA"), Some(&4));
        assert_eq!(hints.prefixes.counts.get("PA"), Some(&1));
        assert_eq!(hints.prefixes.counts.get("PL"), Some(&1));

        // Formatted table produces valid non-empty string
        let table = hints.grid.format_table();
        assert!(table.contains("Letter"));
        assert!(table.contains("Total"));
        assert!(table.contains("6"));
    }
}
