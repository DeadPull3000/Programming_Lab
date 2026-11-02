//! # Wordle Dictionary & Answer Pool Management
//!
//! Separates accepted dictionary guesses (represented as an in-memory 26-ary [`Trie`])
//! from the curated secret answer pool (`Vec<Word>`).
//!
//! In accordance with official Wordle design principles:
//! - **Accepted Guesses:** A broad lexicon of valid 5-letter English words (~176k words).
//! - **Possible Secret Answers:** A curated subset of familiar 5-letter words ($Answers \subseteq AcceptedGuesses$).
//! - **Fallback Mechanism:** If no curated `answers.txt` is found, all 5-letter words in the Trie
//!   are extracted to form a functional answer pool with an explicit fallback notice.

use std::collections::HashSet;
use std::fmt;
use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::path::Path;
use trie::Trie;

use crate::word::{Word, WordError, WORD_LENGTH};

/// Error variants encountered during dictionary and answer pool initialization.
#[derive(Debug)]
pub enum DictionaryError {
    /// An underlying I/O error occurred while reading from disk.
    IoError(io::Error),
    /// A file path was empty or the target file was missing.
    FileNotFound(String),
    /// The answer file contained zero valid 5-letter words.
    EmptyAnswerPool,
    /// An answer word was not present in the accepted guesses Trie.
    AnswerNotInAcceptedGuesses(String),
    /// A lexical parsing error on a word entry.
    InvalidWord(WordError),
}

impl fmt::Display for DictionaryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DictionaryError::IoError(e) => write!(f, "dictionary I/O error: {e}"),
            DictionaryError::FileNotFound(path) => write!(f, "file not found: {path}"),
            DictionaryError::EmptyAnswerPool => {
                write!(f, "the answer pool is empty; no valid 5-letter words found")
            }
            DictionaryError::AnswerNotInAcceptedGuesses(word) => {
                write!(
                    f,
                    "curated answer '{word}' does not exist in accepted guesses"
                )
            }
            DictionaryError::InvalidWord(e) => write!(f, "word parse error: {e}"),
        }
    }
}

impl std::error::Error for DictionaryError {}

impl From<io::Error> for DictionaryError {
    fn from(err: io::Error) -> Self {
        DictionaryError::IoError(err)
    }
}

/// The dual-tier Wordle dictionary system.
#[derive(Debug)]
pub struct WordleDictionary {
    accepted_guesses: Trie,
    answers: Vec<Word>,
    using_fallback_answers: bool,
}

impl WordleDictionary {
    /// Candidate filesystem search paths for lexicons.
    pub const CANDIDATE_DICT_PATHS: &'static [&'static str] = &[
        "Resources/dictionary.txt",
        "../Resources/dictionary.txt",
        "../../Resources/dictionary.txt",
        "dictionary.txt",
        "../dictionary.txt",
        "../../dictionary.txt",
    ];

    /// Candidate filesystem search paths for curated answers.
    pub const CANDIDATE_ANSWER_PATHS: &'static [&'static str] = &[
        "Resources/answers.txt",
        "../Resources/answers.txt",
        "../../Resources/answers.txt",
        "answers.txt",
        "../answers.txt",
        "../../answers.txt",
    ];

    /// Constructs a `WordleDictionary` directly from an existing `Trie` and answer slice.
    ///
    /// Validates that every word in `answers` is present in `accepted_guesses`.
    pub fn new(accepted_guesses: Trie, answers: Vec<Word>) -> Result<Self, DictionaryError> {
        if answers.is_empty() {
            return Err(DictionaryError::EmptyAnswerPool);
        }

        // Verify Answers ⊆ AcceptedGuesses
        for answer in &answers {
            if !accepted_guesses.contains(answer.as_str()) {
                return Err(DictionaryError::AnswerNotInAcceptedGuesses(
                    answer.as_str().to_string(),
                ));
            }
        }

        Ok(Self {
            accepted_guesses,
            answers,
            using_fallback_answers: false,
        })
    }

    /// Loads the dictionary and answer pool using standard candidate search paths.
    ///
    /// If a curated `answers.txt` is found, it is loaded and verified against the Trie.
    /// If `answers.txt` is missing or empty, all 5-letter words are extracted from the Trie
    /// to form a fallback answer pool.
    pub fn load_default() -> Result<Self, DictionaryError> {
        let dict_path = Self::find_existing_path(Self::CANDIDATE_DICT_PATHS).ok_or_else(|| {
            DictionaryError::FileNotFound(
                "Could not locate dictionary.txt in any candidate location".to_string(),
            )
        })?;

        let trie = Self::load_trie_from_file(&dict_path)?;

        if let Some(answer_path) = Self::find_existing_path(Self::CANDIDATE_ANSWER_PATHS) {
            match Self::load_answers_from_file(answer_path, &trie) {
                Ok(answers) if !answers.is_empty() => {
                    return Ok(Self {
                        accepted_guesses: trie,
                        answers,
                        using_fallback_answers: false,
                    });
                }
                _ => {
                    // Fall back if answer file was empty or failed validation
                }
            }
        }

        // Fallback: extract 5-letter words from dictionary
        let fallback_answers = Self::extract_five_letter_words(dict_path)?;
        if fallback_answers.is_empty() {
            return Err(DictionaryError::EmptyAnswerPool);
        }

        Ok(Self {
            accepted_guesses: trie,
            answers: fallback_answers,
            using_fallback_answers: true,
        })
    }

    /// Helper to find the first existing path from a candidate list.
    fn find_existing_path(candidates: &[&str]) -> Option<String> {
        candidates
            .iter()
            .find(|&&p| Path::new(p).exists())
            .map(|&p| p.to_string())
    }

    /// Ingests words from a file into a new `Trie`.
    pub fn load_trie_from_file<P: AsRef<Path>>(path: P) -> Result<Trie, DictionaryError> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut trie = Trie::new();

        for line in reader.lines() {
            let line = line?;
            let trimmed = line.trim();
            if !trimmed.is_empty() {
                // Split on commas or whitespace to support varied formats
                for token in trimmed.split(|c: char| c == ',' || c.is_whitespace()) {
                    let clean = token.trim();
                    if !clean.is_empty() {
                        let _ = trie.insert(clean);
                    }
                }
            }
        }

        Ok(trie)
    }

    /// Loads curated answers from a file, filtering strictly for valid 5-letter words
    /// and ensuring every answer belongs to the accepted `Trie`.
    pub fn load_answers_from_file<P: AsRef<Path>>(
        path: P,
        accepted: &Trie,
    ) -> Result<Vec<Word>, DictionaryError> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        Self::load_answers_from_reader(reader, accepted)
    }

    /// Ingests curated answers from an arbitrary [`BufRead`] stream.
    pub fn load_answers_from_reader<R: BufRead>(
        reader: R,
        accepted: &Trie,
    ) -> Result<Vec<Word>, DictionaryError> {
        let mut answers = Vec::new();
        let mut seen = HashSet::new();

        for line in reader.lines() {
            let line = line?;
            for token in line.split(|c: char| c == ',' || c.is_whitespace()) {
                let trimmed = token.trim();
                if trimmed.is_empty() {
                    continue;
                }

                // If word is valid 5-letter ASCII, check Trie inclusion
                if let Ok(word) = Word::parse(trimmed) {
                    if accepted.contains(word.as_str()) && seen.insert(word.clone()) {
                        answers.push(word);
                    }
                }
            }
        }

        if answers.is_empty() {
            return Err(DictionaryError::EmptyAnswerPool);
        }

        Ok(answers)
    }

    /// Extracts all 5-letter words from a raw dictionary file as fallback answers.
    pub fn extract_five_letter_words<P: AsRef<Path>>(
        path: P,
    ) -> Result<Vec<Word>, DictionaryError> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut words = Vec::new();
        let mut seen = HashSet::new();

        for line in reader.lines() {
            let line = line?;
            for token in line.split(|c: char| c == ',' || c.is_whitespace()) {
                let trimmed = token.trim();
                if trimmed.len() == WORD_LENGTH {
                    if let Ok(word) = Word::parse(trimmed) {
                        if seen.insert(word.clone()) {
                            words.push(word);
                        }
                    }
                }
            }
        }

        Ok(words)
    }

    /// Verifies if a word is an accepted guess in the Trie dictionary.
    #[inline]
    pub fn is_accepted(&self, word: &str) -> bool {
        self.accepted_guesses.contains(word)
    }

    /// Checks if a validated `Word` is an accepted guess.
    #[inline]
    pub fn contains_word(&self, word: &Word) -> bool {
        self.accepted_guesses.contains(word.as_str())
    }

    /// Returns a reference to the accepted guesses Trie.
    #[inline]
    pub fn trie(&self) -> &Trie {
        &self.accepted_guesses
    }

    /// Returns an immutable slice of all candidate secret answers.
    #[inline]
    pub fn answers(&self) -> &[Word] {
        &self.answers
    }

    /// Returns the number of curated answers.
    #[inline]
    pub fn answer_count(&self) -> usize {
        self.answers.len()
    }

    /// Indicates whether the dictionary is operating with an uncurated fallback answer pool.
    #[inline]
    pub fn is_using_fallback(&self) -> bool {
        self.using_fallback_answers
    }

    /// Selects a random answer from the pool using the provided RNG.
    pub fn choose_random_answer<R: rand::Rng + ?Sized>(&self, rng: &mut R) -> Option<Word> {
        choose_answer(&self.answers, rng)
    }
}

/// Selects a random answer from a slice of [`Word`]s using an injected RNG.
///
/// Returns `None` if the provided answer slice is empty.
pub fn choose_answer<R: rand::Rng + ?Sized>(answers: &[Word], rng: &mut R) -> Option<Word> {
    if answers.is_empty() {
        None
    } else {
        let idx = rng.gen_range(0..answers.len());
        Some(answers[idx].clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    fn build_test_trie() -> Trie {
        let mut trie = Trie::new();
        trie.insert("APPLE").unwrap();
        trie.insert("CRANE").unwrap();
        trie.insert("SLATE").unwrap();
        trie.insert("ROAST").unwrap();
        trie.insert("TRAIN").unwrap();
        trie
    }

    #[test]
    fn test_valid_answer_loading_and_deduplication() {
        let trie = build_test_trie();
        let answers_data = "apple\ncrane\n  SLATE  \napple\nroast\n";
        let reader = Cursor::new(answers_data);

        let answers = WordleDictionary::load_answers_from_reader(reader, &trie).unwrap();
        assert_eq!(answers.len(), 4); // apple, crane, slate, roast (deduplicated)
        assert_eq!(answers[0].as_str(), "APPLE");
        assert_eq!(answers[1].as_str(), "CRANE");
        assert_eq!(answers[2].as_str(), "SLATE");
        assert_eq!(answers[3].as_str(), "ROAST");
    }

    #[test]
    fn test_answers_not_in_trie_filtered() {
        let trie = build_test_trie();
        // "XYZWQ" is not in the trie
        let answers_data = "apple, xyzwq, crane";
        let reader = Cursor::new(answers_data);

        let answers = WordleDictionary::load_answers_from_reader(reader, &trie).unwrap();
        assert_eq!(answers.len(), 2);
        assert_eq!(answers[0].as_str(), "APPLE");
        assert_eq!(answers[1].as_str(), "CRANE");
    }

    #[test]
    fn test_non_five_letter_words_filtered() {
        let mut trie = build_test_trie();
        trie.insert("CAT").unwrap();
        trie.insert("ELEPHANT").unwrap();

        let answers_data = "CAT, APPLE, ELEPHANT, CRANE";
        let reader = Cursor::new(answers_data);

        let answers = WordleDictionary::load_answers_from_reader(reader, &trie).unwrap();
        assert_eq!(answers.len(), 2);
        assert_eq!(answers[0].as_str(), "APPLE");
        assert_eq!(answers[1].as_str(), "CRANE");
    }

    #[test]
    fn test_empty_answer_pool_error() {
        let trie = build_test_trie();
        let answers_data = "   \n\n  ";
        let reader = Cursor::new(answers_data);

        let err = WordleDictionary::load_answers_from_reader(reader, &trie).unwrap_err();
        assert!(matches!(err, DictionaryError::EmptyAnswerPool));
    }

    #[test]
    fn test_subset_invariant_in_constructor() {
        let trie = build_test_trie();
        let valid_answers = vec![Word::parse("APPLE").unwrap(), Word::parse("CRANE").unwrap()];
        let dict = WordleDictionary::new(trie, valid_answers).expect("valid subset");
        assert_eq!(dict.answer_count(), 2);

        let mut trie2 = Trie::new();
        trie2.insert("APPLE").unwrap();
        // "CRANE" is not in trie2
        let invalid_answers = vec![Word::parse("APPLE").unwrap(), Word::parse("CRANE").unwrap()];
        let err = WordleDictionary::new(trie2, invalid_answers).unwrap_err();
        assert!(matches!(err, DictionaryError::AnswerNotInAcceptedGuesses(w) if w == "CRANE"));
    }

    #[test]
    fn test_random_choice() {
        let trie = build_test_trie();
        let valid_answers = vec![Word::parse("APPLE").unwrap(), Word::parse("CRANE").unwrap()];
        let dict = WordleDictionary::new(trie, valid_answers).unwrap();

        let mut rng = rand::thread_rng();
        let choice = dict.choose_random_answer(&mut rng).unwrap();
        assert!(choice.as_str() == "APPLE" || choice.as_str() == "CRANE");
    }
}
