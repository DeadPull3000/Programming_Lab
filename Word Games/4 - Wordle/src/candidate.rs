//! # Wordle Candidate Filtering & Solver Assistant
//!
//! Provides candidate elimination routines that prune possible secret answers
//! consistent with observed feedback patterns.
//!
//! # Algorithmic Architecture
//!
//! Rather than inventing custom regexes or heuristic pattern matching, candidate filtering
//! reuses the authoritative [`calculate_feedback`] function directly.
//!
//! For each candidate word $C$ in the remaining pool:
//! - Calculate hypothetical feedback: $F = \text{calculate\_feedback}(C, \text{guess})$.
//! - If $F == \text{observed}$, then candidate $C$ is mathematically consistent with
//!   the observed evidence and is retained.
//! - If $F \ne \text{observed}$, then candidate $C$ could not possibly be the secret
//!   answer and is eliminated.
//!
//! # Conservation of Letter Frequency
//! Because candidate filtering reuses [`calculate_feedback`], duplicate letters are
//! handled with mathematical rigor: words containing fewer or more occurrences of
//! letters than verified by the observed feedback are automatically pruned.

use crate::feedback::{calculate_feedback, Feedback};
use crate::word::Word;

/// Summary metrics produced after a candidate pruning operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FilterSummary {
    /// Number of candidates before filtering.
    pub initial_count: usize,
    /// Number of candidates remaining after filtering.
    pub remaining_count: usize,
    /// Number of candidates pruned during this step.
    pub eliminated_count: usize,
}

impl FilterSummary {
    /// Returns `true` if any candidates were eliminated in this step.
    #[inline]
    pub fn has_eliminated(&self) -> bool {
        self.eliminated_count > 0
    }
}

/// Encapsulates the collection of all possible secret answers consistent
/// with the historical feedback received during a Wordle round.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CandidateSet {
    words: Vec<Word>,
}

impl CandidateSet {
    /// Initialises a new `CandidateSet` from an immutable slice of the answer pool.
    ///
    /// Preserves the input order and deduplicates entries if any duplicate words exist.
    pub fn new(answer_pool: &[Word]) -> Self {
        Self {
            words: answer_pool.to_vec(),
        }
    }

    /// Constructs a `CandidateSet` directly from a vector of words, avoiding reallocation.
    pub fn from_vec(words: Vec<Word>) -> Self {
        Self { words }
    }

    /// Prunes the candidate pool in place, retaining only words that would produce
    /// exactly the `observed` feedback if they were the true secret answer.
    ///
    /// # Invariants
    /// 1. Candidate count is strictly non-increasing: $C_{t+1} \le C_t$.
    /// 2. If the secret answer satisfies the observed feedback, it is guaranteed to remain.
    /// 3. In-place retention avoids heap allocations for surviving entries.
    pub fn filter(&mut self, guess: &Word, observed: &Feedback) -> FilterSummary {
        let initial_count = self.words.len();
        self.words.retain(|candidate| {
            let hypothetical = calculate_feedback(candidate, guess);
            &hypothetical == observed
        });
        let remaining_count = self.words.len();
        let eliminated_count = initial_count.saturating_sub(remaining_count);

        FilterSummary {
            initial_count,
            remaining_count,
            eliminated_count,
        }
    }

    /// Returns an immutable slice of the remaining candidates.
    #[inline]
    pub fn words(&self) -> &[Word] {
        &self.words
    }

    /// Returns the number of surviving candidate words.
    #[inline]
    pub fn len(&self) -> usize {
        self.words.len()
    }

    /// Returns `true` if no candidates remain in the pool.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.words.is_empty()
    }

    /// Returns `true` if the candidate set contains the specified word.
    #[inline]
    pub fn contains(&self, word: &Word) -> bool {
        self.words.contains(word)
    }

    /// Resets the candidate set to an explicit answer pool.
    pub fn reset(&mut self, answer_pool: &[Word]) {
        self.words.clear();
        self.words.extend_from_slice(answer_pool);
    }
}

/// Filters an external candidate slice of potential answers, returning a new vector.
///
/// Provided for functional pipeline workflows and backward compatibility.
pub fn filter_candidates(candidates: &[Word], guess: &Word, observed: &Feedback) -> Vec<Word> {
    let mut set = CandidateSet::new(candidates);
    set.filter(guess, observed);
    set.words().to_vec()
}

/// Applies a sequence of guesses and their observed feedback to progressively narrow
/// down candidate solutions.
pub fn filter_candidates_multi_step(
    candidates: &[Word],
    history: &[(&Word, &Feedback)],
) -> Vec<Word> {
    let mut set = CandidateSet::new(candidates);
    for (guess, observed) in history {
        set.filter(guess, observed);
    }
    set.words().to_vec()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_candidates() -> Vec<Word> {
        vec![
            Word::parse("APPLE").unwrap(),
            Word::parse("ALLEY").unwrap(),
            Word::parse("APART").unwrap(),
            Word::parse("CRANE").unwrap(),
            Word::parse("SLATE").unwrap(),
            Word::parse("ROAST").unwrap(),
            Word::parse("TRAIN").unwrap(),
            Word::parse("ALERT").unwrap(),
            Word::parse("BANAL").unwrap(),
            Word::parse("PEPPY").unwrap(),
            Word::parse("PUPPY").unwrap(),
        ]
    }

    #[test]
    fn test_candidate_set_initialization() {
        let pool = sample_candidates();
        let set = CandidateSet::new(&pool);
        assert_eq!(set.len(), pool.len());
        assert!(!set.is_empty());
        assert_eq!(set.words(), &pool[..]);
        assert!(set.contains(&Word::parse("APPLE").unwrap()));
    }

    #[test]
    fn test_secret_word_always_survives_filtering() {
        let pool = sample_candidates();
        let secret = Word::parse("APPLE").unwrap();
        let guess = Word::parse("CRANE").unwrap();
        let observed = calculate_feedback(&secret, &guess);

        let mut set = CandidateSet::new(&pool);
        let summary = set.filter(&guess, &observed);

        assert!(set.contains(&secret), "True answer must survive filtering");
        assert!(summary.remaining_count <= summary.initial_count);
        assert_eq!(summary.remaining_count, set.len());
        assert_eq!(
            summary.eliminated_count,
            summary.initial_count - summary.remaining_count
        );
    }

    #[test]
    fn test_inconsistent_candidates_are_eliminated() {
        let pool = sample_candidates();
        let secret = Word::parse("APPLE").unwrap();
        let guess = Word::parse("ALLEY").unwrap();
        let observed = calculate_feedback(&secret, &guess);

        let mut set = CandidateSet::new(&pool);
        set.filter(&guess, &observed);

        assert!(set.contains(&secret));
        // "CRANE", "ROAST", "SLATE" could never produce observed feedback
        assert!(!set.contains(&Word::parse("CRANE").unwrap()));
        assert!(!set.contains(&Word::parse("ROAST").unwrap()));
        assert!(!set.contains(&Word::parse("SLATE").unwrap()));
    }

    #[test]
    fn test_duplicate_letter_feedback_pruning() {
        // "PEPPY" has three P's; "PUPPY" has three P's; "APPLE" has two P's
        let pool = sample_candidates();
        let secret = Word::parse("PEPPY").unwrap();
        let guess = Word::parse("PUPPY").unwrap();
        let observed = calculate_feedback(&secret, &guess);

        let mut set = CandidateSet::new(&pool);
        set.filter(&guess, &observed);

        assert!(set.contains(&secret));
        assert!(
            !set.contains(&Word::parse("APPLE").unwrap()),
            "APPLE has only two P's and must be pruned"
        );
    }

    #[test]
    fn test_candidate_count_monotonicity() {
        let pool = sample_candidates();
        let secret = Word::parse("APPLE").unwrap();
        let mut set = CandidateSet::new(&pool);

        let guess1 = Word::parse("CRANE").unwrap();
        let fb1 = calculate_feedback(&secret, &guess1);
        let s1 = set.filter(&guess1, &fb1);
        assert!(s1.remaining_count <= s1.initial_count);
        assert!(set.contains(&secret));

        let guess2 = Word::parse("ALLEY").unwrap();
        let fb2 = calculate_feedback(&secret, &guess2);
        let s2 = set.filter(&guess2, &fb2);
        assert!(s2.remaining_count <= s2.initial_count);
        assert!(set.contains(&secret));
    }

    #[test]
    fn test_incremental_filtering_equals_full_recomputation() {
        let pool = sample_candidates();
        let secret = Word::parse("APPLE").unwrap();

        let guess1 = Word::parse("CRANE").unwrap();
        let fb1 = calculate_feedback(&secret, &guess1);
        let guess2 = Word::parse("ALLEY").unwrap();
        let fb2 = calculate_feedback(&secret, &guess2);

        // Incremental filtering
        let mut incremental_set = CandidateSet::new(&pool);
        incremental_set.filter(&guess1, &fb1);
        incremental_set.filter(&guess2, &fb2);

        // Independent full recomputation reference
        let reference_recomputed: Vec<Word> = pool
            .iter()
            .filter(|w| {
                calculate_feedback(w, &guess1) == fb1 && calculate_feedback(w, &guess2) == fb2
            })
            .cloned()
            .collect();

        assert_eq!(incremental_set.words(), &reference_recomputed[..]);
    }

    #[test]
    fn test_empty_candidate_set_handling() {
        let mut set = CandidateSet::new(&[]);
        assert!(set.is_empty());
        assert_eq!(set.len(), 0);

        let guess = Word::parse("CRANE").unwrap();
        let observed = Feedback::new([crate::feedback::LetterFeedback::Absent; 5]);
        let summary = set.filter(&guess, &observed);

        assert_eq!(summary.initial_count, 0);
        assert_eq!(summary.remaining_count, 0);
        assert_eq!(summary.eliminated_count, 0);
        assert!(set.is_empty());
    }

    #[test]
    fn test_idempotent_filtering() {
        let pool = sample_candidates();
        let secret = Word::parse("APPLE").unwrap();
        let guess = Word::parse("CRANE").unwrap();
        let observed = calculate_feedback(&secret, &guess);

        let mut set = CandidateSet::new(&pool);
        let s1 = set.filter(&guess, &observed);
        let s2 = set.filter(&guess, &observed);

        assert_eq!(s1.remaining_count, s2.remaining_count);
        assert_eq!(s2.eliminated_count, 0);
        assert!(!s2.has_eliminated());
    }
}
