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

use crate::feedback::{calculate_feedback, Feedback};
use crate::word::Word;

/// Filters a candidate list of potential answers, retaining only those words that
/// would produce the exact `observed` feedback if they were the true secret answer.
///
/// # Invariants
/// 1. Reuses the exact same [`calculate_feedback`] function used in gameplay.
/// 2. If the true secret answer was present in `candidates`, it is guaranteed to remain.
/// 3. The length of the filtered candidate vector is strictly $\le$ the input length.
///
/// # Examples
/// ```
/// use wordle::candidate::filter_candidates;
/// use wordle::feedback::calculate_feedback;
/// use wordle::word::Word;
///
/// let candidates = vec![
///     Word::parse("APPLE").unwrap(),
///     Word::parse("CRANE").unwrap(),
///     Word::parse("SLATE").unwrap(),
/// ];
/// let guess = Word::parse("CRANE").unwrap();
/// let answer = Word::parse("SLATE").unwrap();
/// let observed = calculate_feedback(&answer, &guess);
///
/// let remaining = filter_candidates(&candidates, &guess, &observed);
/// assert!(remaining.contains(&answer));
/// assert!(!remaining.contains(&Word::parse("APPLE").unwrap()));
/// ```
pub fn filter_candidates(candidates: &[Word], guess: &Word, observed: &Feedback) -> Vec<Word> {
    candidates
        .iter()
        .filter(|&candidate| {
            let hypothetical = calculate_feedback(candidate, guess);
            &hypothetical == observed
        })
        .cloned()
        .collect()
}

/// Applies a sequence of guesses and their observed feedback to progressively narrow
/// down candidate solutions.
pub fn filter_candidates_multi_step(
    candidates: &[Word],
    history: &[(&Word, &Feedback)],
) -> Vec<Word> {
    let mut remaining = candidates.to_vec();
    for (guess, observed) in history {
        remaining = filter_candidates(&remaining, guess, observed);
    }
    remaining
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
        ]
    }

    #[test]
    fn test_secret_word_always_survives_filtering() {
        let pool = sample_candidates();
        let secret = Word::parse("APPLE").unwrap();
        let guess = Word::parse("CRANE").unwrap();
        let observed = calculate_feedback(&secret, &guess);

        let filtered = filter_candidates(&pool, &guess, &observed);
        assert!(
            filtered.contains(&secret),
            "True answer must survive filtering"
        );
    }

    #[test]
    fn test_inconsistent_candidates_are_eliminated() {
        let pool = sample_candidates();
        let secret = Word::parse("APPLE").unwrap();
        let guess = Word::parse("ALLEY").unwrap();
        let observed = calculate_feedback(&secret, &guess);

        let filtered = filter_candidates(&pool, &guess, &observed);
        assert!(filtered.contains(&secret));
        // Words like "CRANE" or "ROAST" could never produce the same feedback
        assert!(!filtered.contains(&Word::parse("CRANE").unwrap()));
        assert!(!filtered.contains(&Word::parse("ROAST").unwrap()));
    }

    #[test]
    fn test_monotonic_candidate_reduction() {
        let pool = sample_candidates();
        let secret = Word::parse("APPLE").unwrap();

        let guess1 = Word::parse("CRANE").unwrap();
        let fb1 = calculate_feedback(&secret, &guess1);
        let step1 = filter_candidates(&pool, &guess1, &fb1);

        assert!(step1.len() <= pool.len());

        let guess2 = Word::parse("ALLEY").unwrap();
        let fb2 = calculate_feedback(&secret, &guess2);
        let step2 = filter_candidates(&step1, &guess2, &fb2);

        assert!(step2.len() <= step1.len());
        assert!(step2.contains(&secret));
    }

    #[test]
    fn test_empty_candidate_pool_handling() {
        let empty_pool: Vec<Word> = Vec::new();
        let guess = Word::parse("CRANE").unwrap();
        let observed = Feedback::new([crate::feedback::LetterFeedback::Absent; 5]);

        let filtered = filter_candidates(&empty_pool, &guess, &observed);
        assert!(filtered.is_empty());
    }
}
