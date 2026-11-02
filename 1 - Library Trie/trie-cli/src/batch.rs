//! File loading and batch-processing utilities for Trie.

use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::path::Path;
use trie::{normalize_word, Trie, TrieError};

/// Summary report after loading words into the trie.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct LoadReport {
    /// Total number of tokens parsed.
    pub total_tokens: usize,
    /// Number of distinct words newly added to the trie.
    pub newly_inserted: usize,
    /// Number of tokens that were already present in the trie.
    pub duplicates: usize,
    /// Invalid entries encountered along with line number, token string, and error.
    pub invalid_entries: Vec<(usize, String, TrieError)>,
}

/// Result of classifying a batch of words into found, missing, and invalid entries.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct BatchCheckResult {
    /// Words found in the trie (normalized to uppercase).
    pub found: Vec<String>,
    /// Words not found in the trie (normalized to uppercase).
    pub missing: Vec<String>,
    /// Tokens that could not be parsed as valid words, with line number.
    pub invalid: Vec<(usize, String, TrieError)>,
}

/// Extracts tokens from a string separated by commas, whitespace, or both.
pub fn parse_tokens(text: &str) -> Vec<&str> {
    text.split(|c: char| c == ',' || c.is_whitespace())
        .filter(|token| !token.is_empty())
        .collect()
}

/// Loads words from any `BufRead` reader into the provided trie.
///
/// Handles one-word-per-line, comma-separated, and whitespace-separated formats.
/// Returns a `LoadReport` tracking new words, duplicates, and malformed tokens with line numbers.
pub fn load_words_from_reader<R: BufRead>(reader: R, trie: &mut Trie) -> io::Result<LoadReport> {
    let mut report = LoadReport::default();

    for (line_idx, line_res) in reader.lines().enumerate() {
        let line = line_res?;
        let line_num = line_idx + 1;

        for token in parse_tokens(&line) {
            report.total_tokens += 1;
            match trie.insert(token) {
                Ok(true) => report.newly_inserted += 1,
                Ok(false) => report.duplicates += 1,
                Err(err) => report
                    .invalid_entries
                    .push((line_num, token.to_string(), err)),
            }
        }
    }

    Ok(report)
}

/// Loads words from a file into the provided trie.
pub fn load_words_from_file<P: AsRef<Path>>(path: P, trie: &mut Trie) -> io::Result<LoadReport> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    load_words_from_reader(reader, trie)
}

/// Classifies words from any `BufRead` reader against the trie into found and missing words.
pub fn check_words_from_reader<R: BufRead>(reader: R, trie: &Trie) -> io::Result<BatchCheckResult> {
    let mut result = BatchCheckResult::default();

    for (line_idx, line_res) in reader.lines().enumerate() {
        let line = line_res?;
        let line_num = line_idx + 1;

        for token in parse_tokens(&line) {
            match normalize_word(token) {
                Ok(normalized) => {
                    if trie.contains(&normalized) {
                        result.found.push(normalized);
                    } else {
                        result.missing.push(normalized);
                    }
                }
                Err(err) => {
                    result.invalid.push((line_num, token.to_string(), err));
                }
            }
        }
    }

    Ok(result)
}

/// Classifies words from a candidate-word file against the trie.
pub fn check_words_from_file<P: AsRef<Path>>(path: P, trie: &Trie) -> io::Result<BatchCheckResult> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    check_words_from_reader(reader, trie)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;
    use std::path::PathBuf;

    /// RAII guard to create and reliably remove temporary files.
    struct TempTestFile {
        path: PathBuf,
    }

    impl TempTestFile {
        fn new(name: &str, content: &str) -> Self {
            let mut path = std::env::temp_dir();
            path.push(format!("word_trie_test_{}_{}", std::process::id(), name));
            std::fs::write(&path, content).expect("failed to write temp test file");
            Self { path }
        }

        fn path(&self) -> &Path {
            &self.path
        }
    }

    impl Drop for TempTestFile {
        fn drop(&mut self) {
            let _ = std::fs::remove_file(&self.path);
        }
    }

    #[test]
    fn test_parse_tokens_various_separators() {
        let input = "  apple, banana   cherry\n\n date, \tfig,,,grape  ";
        let tokens = parse_tokens(input);
        assert_eq!(
            tokens,
            vec!["apple", "banana", "cherry", "date", "fig", "grape"]
        );
    }

    #[test]
    fn test_parse_tokens_empty_and_whitespace() {
        assert_eq!(parse_tokens(""), Vec::<&str>::new());
        assert_eq!(parse_tokens("    \n\t,,\t, "), Vec::<&str>::new());
    }

    #[test]
    fn test_load_words_from_reader_mixed_content() {
        let mut trie = Trie::new();
        let data = "apple\nbanana, cherry\n\n  APPLE  \n123, date!\n";
        let cursor = Cursor::new(data);

        let report = load_words_from_reader(cursor, &mut trie).unwrap();
        assert_eq!(report.total_tokens, 6);
        assert_eq!(report.newly_inserted, 3); // APPLE, BANANA, CHERRY
        assert_eq!(report.duplicates, 1); // APPLE
        assert_eq!(report.invalid_entries.len(), 2); // 123, date!

        // Check line numbers of errors
        assert_eq!(report.invalid_entries[0].0, 5); // Line 5: "123"
        assert_eq!(report.invalid_entries[0].1, "123");
        assert_eq!(report.invalid_entries[1].0, 5); // Line 5: "date!"
        assert_eq!(report.invalid_entries[1].1, "date!");

        assert_eq!(trie.len(), 3);
        assert!(trie.contains("apple"));
        assert!(trie.contains("banana"));
        assert!(trie.contains("cherry"));
    }

    #[test]
    fn test_load_empty_reader() {
        let mut trie = Trie::new();
        let report = load_words_from_reader(Cursor::new(""), &mut trie).unwrap();
        assert_eq!(report.total_tokens, 0);
        assert_eq!(report.newly_inserted, 0);
        assert_eq!(trie.len(), 0);
    }

    #[test]
    fn test_check_words_from_reader() {
        let mut trie = Trie::new();
        trie.insert("apple").unwrap();
        trie.insert("banana").unwrap();

        let candidates = "APPLE, orange\nBANANA, grape, bad#token\n";
        let res = check_words_from_reader(Cursor::new(candidates), &trie).unwrap();

        assert_eq!(res.found, vec!["APPLE", "BANANA"]);
        assert_eq!(res.missing, vec!["ORANGE", "GRAPE"]);
        assert_eq!(res.invalid.len(), 1);
        assert_eq!(res.invalid[0].1, "bad#token");
    }

    #[test]
    fn test_load_and_check_file_io() {
        let mut trie = Trie::new();
        let dict_file = TempTestFile::new("dict.txt", "rust\nprogramming\nlanguage\n");
        let candidate_file = TempTestFile::new("cand.txt", "rust, python, language\n");

        let load_rep = load_words_from_file(dict_file.path(), &mut trie).unwrap();
        assert_eq!(load_rep.newly_inserted, 3);
        assert_eq!(trie.len(), 3);

        let check_rep = check_words_from_file(candidate_file.path(), &trie).unwrap();
        assert_eq!(check_rep.found, vec!["RUST", "LANGUAGE"]);
        assert_eq!(check_rep.missing, vec!["PYTHON"]);
    }

    #[test]
    fn test_file_not_found_error() {
        let mut trie = Trie::new();
        let res = load_words_from_file("nonexistent_path_xyz123.txt", &mut trie);
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().kind(), io::ErrorKind::NotFound);
    }
}
