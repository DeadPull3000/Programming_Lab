//! # Wordle Solver & Information-Theoretic Suggestion Engine
//!
//! Provides intelligent guess recommendations using information theory (Shannon entropy)
//! and minimax optimization (minimizing worst-case remaining candidates).
//!
//! # Mathematical Foundations
//!
//! ## 1. Partitioning Candidates into Feedback Buckets
//! For any proposed guess $G$ and remaining candidate set $C$ with size $N = |C|$:
//! Every candidate $c \in C$ produces a deterministic feedback pattern $F(c, G) \in \{0, \dots, 242\}$.
//! The solver partitions the $N$ candidates across the 243 discrete feedback buckets:
//!
//! $$\sum_{i=0}^{242} n_i = N$$
//!
//! where $n_i$ is the number of candidates that produce pattern $i$.
//!
//! ## 2. Shannon Entropy (Information Gain)
//! Assuming every candidate is equally likely ($p_i = \frac{n_i}{N}$), the expected information gain
//! in bits is:
//!
//! $$H(G) = -\sum_{i: n_i > 0} p_i \log_2(p_i) = \log_2(N) - \frac{1}{N} \sum_{i: n_i > 0} n_i \log_2(n_i)$$
//!
//! A higher entropy indicates that the guess divides the candidate pool into smaller, more uniform
//! groups, providing greater distinguishing power.
//!
//! ## 3. Expected Remaining Candidates
//! If feedback pattern $i$ is received, the candidate pool shrinks to size $n_i$.
//! The expected number of surviving candidates is:
//!
//! $$E(G) = \sum_{i: n_i > 0} p_i \cdot n_i = \frac{\sum_{i: n_i > 0} n_i^2}{N}$$
//!
//! Lower expected remaining count is better.
//!
//! ## 4. Minimax (Worst-Case Remaining Candidates)
//! The minimax criterion evaluates the largest possible candidate pool that could remain:
//!
//! $$W(G) = \max_{i} (n_i)$$
//!
//! Minimizing $W(G)$ guarantees the tightest worst-case bound regardless of the secret answer.

use std::fmt;

use crate::candidate::CandidateSet;
use crate::feedback::{calculate_feedback, FEEDBACK_PATTERN_COUNT};
use crate::word::Word;

/// Strategy employed by the solver to rank potential guesses.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SolverStrategy {
    /// Ranks guesses by descending Shannon entropy (information gain).
    InformationGain,
    /// Ranks guesses by ascending worst-case remaining candidates (minimax).
    Minimax,
}

impl fmt::Display for SolverStrategy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SolverStrategy::InformationGain => write!(f, "Information Gain (Entropy)"),
            SolverStrategy::Minimax => write!(f, "Minimax (Worst-Case Reduction)"),
        }
    }
}

/// Configuration parameters for the solver engine.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SolverConfig {
    /// The ranking strategy to use.
    pub strategy: SolverStrategy,
    /// Number of top-ranked suggestions to return (must be $\ge 1$).
    pub top_k: usize,
}

impl SolverConfig {
    /// Constructs a `SolverConfig` with specified strategy and top-k limit.
    pub fn new(strategy: SolverStrategy, top_k: usize) -> Result<Self, SolverError> {
        if top_k == 0 {
            return Err(SolverError::InvalidTopK);
        }
        Ok(Self { strategy, top_k })
    }
}

impl Default for SolverConfig {
    fn default() -> Self {
        Self {
            strategy: SolverStrategy::InformationGain,
            top_k: 5,
        }
    }
}

/// Error conditions produced during solver suggestion evaluation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SolverError {
    /// The remaining candidate pool is empty; cannot compute partitions.
    EmptyCandidateSet,
    /// The legal guess search pool is empty.
    EmptyLegalGuesses,
    /// The requested `top_k` was zero.
    InvalidTopK,
}

impl fmt::Display for SolverError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SolverError::EmptyCandidateSet => {
                write!(f, "cannot compute suggestions: candidate set is empty")
            }
            SolverError::EmptyLegalGuesses => {
                write!(f, "cannot compute suggestions: legal guess pool is empty")
            }
            SolverError::InvalidTopK => {
                write!(f, "top_k must be greater than zero")
            }
        }
    }
}

impl std::error::Error for SolverError {}

/// A ranked guess suggestion with evaluation metrics.
#[derive(Debug, Clone, PartialEq)]
pub struct Suggestion {
    /// The proposed 5-letter guess word.
    pub word: Word,
    /// Expected information gain in bits ($H(G) \ge 0.0$).
    pub entropy: f64,
    /// Expected number of surviving candidates after this guess ($E(G) \ge 1.0$).
    pub expected_remaining: f64,
    /// Maximum number of surviving candidates in the worst feedback branch ($W(G) \ge 1$).
    pub worst_case_remaining: usize,
    /// Indicates whether this guess is itself a remaining viable secret answer.
    pub is_possible_answer: bool,
}

impl fmt::Display for Suggestion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let tag = if self.is_possible_answer {
            "[Possible Answer]"
        } else {
            "[Explorer Guess] "
        };
        write!(
            f,
            "{word} {tag} | Entropy: {entropy:.3} bits | Exp Rem: {exp:.2} | Worst: {worst}",
            word = self.word,
            entropy = self.entropy,
            exp = self.expected_remaining,
            worst = self.worst_case_remaining
        )
    }
}

/// Computes the exact metrics ($H, E, W$) for a proposed guess against a candidate pool.
pub fn evaluate_guess_metrics(guess: &Word, candidates: &[Word]) -> (f64, f64, usize) {
    let n = candidates.len();
    if n == 0 {
        return (0.0, 0.0, 0);
    }
    if n == 1 {
        return (0.0, 1.0, 1);
    }

    let mut buckets = [0usize; FEEDBACK_PATTERN_COUNT];
    for candidate in candidates {
        let fb = calculate_feedback(candidate, guess);
        let id = fb.to_pattern_id();
        buckets[id] += 1;
    }

    let n_f64 = n as f64;
    let mut sum_entropy = 0.0;
    let mut sum_sq = 0.0;
    let mut worst_case = 0;

    for &count in &buckets {
        if count > 0 {
            if count > worst_case {
                worst_case = count;
            }
            let c_f64 = count as f64;
            let p_i = c_f64 / n_f64;
            sum_entropy -= p_i * p_i.log2();
            sum_sq += c_f64 * c_f64;
        }
    }

    let expected_remaining = sum_sq / n_f64;
    (sum_entropy, expected_remaining, worst_case)
}

/// The intelligent Wordle suggestion engine.
#[derive(Debug, Clone)]
pub struct Solver {
    config: SolverConfig,
}

impl Solver {
    /// Creates a new `Solver` with the specified configuration.
    pub fn new(config: SolverConfig) -> Self {
        Self { config }
    }

    /// Returns a reference to the active solver configuration.
    #[inline]
    pub fn config(&self) -> &SolverConfig {
        &self.config
    }

    /// Evaluates all legal guesses against the remaining candidates and returns the top suggestions.
    ///
    /// # Search Constraints
    /// 1. `candidates`: The remaining possible secret answers.
    /// 2. `legal_guesses`: The full lexicon of legal 5-letter words accepted as guesses.
    /// 3. `previous_guesses`: Words already submitted by the player in this round (excluded from suggestions).
    ///
    /// # Deterministic Ranking Order
    /// - **Information Gain:**
    ///   1. Descending entropy (`entropy`)
    ///   2. Ascending expected remaining (`expected_remaining`)
    ///   3. Ascending worst-case remaining (`worst_case_remaining`)
    ///   4. Candidate answer priority (`is_possible_answer` == true first)
    ///   5. Ascending alphabetical word order
    ///
    /// - **Minimax:**
    ///   1. Ascending worst-case remaining (`worst_case_remaining`)
    ///   2. Descending entropy (`entropy`)
    ///   3. Ascending expected remaining (`expected_remaining`)
    ///   4. Candidate answer priority (`is_possible_answer` == true first)
    ///   5. Ascending alphabetical word order
    pub fn suggest(
        &self,
        candidates: &CandidateSet,
        legal_guesses: &[Word],
        previous_guesses: &[Word],
    ) -> Result<Vec<Suggestion>, SolverError> {
        if candidates.is_empty() {
            return Err(SolverError::EmptyCandidateSet);
        }
        if legal_guesses.is_empty() {
            return Err(SolverError::EmptyLegalGuesses);
        }
        if self.config.top_k == 0 {
            return Err(SolverError::InvalidTopK);
        }

        let cand_words = candidates.words();
        let mut scored = Vec::with_capacity(legal_guesses.len());

        for guess in legal_guesses {
            // Exclude already submitted guesses
            if previous_guesses.contains(guess) {
                continue;
            }

            let (entropy, expected_remaining, worst_case_remaining) =
                evaluate_guess_metrics(guess, cand_words);

            let is_possible_answer = candidates.contains(guess);

            scored.push(Suggestion {
                word: guess.clone(),
                entropy,
                expected_remaining,
                worst_case_remaining,
                is_possible_answer,
            });
        }

        // Apply deterministic ranking
        match self.config.strategy {
            SolverStrategy::InformationGain => {
                scored.sort_by(|a, b| {
                    b.entropy
                        .total_cmp(&a.entropy)
                        .then_with(|| a.expected_remaining.total_cmp(&b.expected_remaining))
                        .then_with(|| a.worst_case_remaining.cmp(&b.worst_case_remaining))
                        .then_with(|| b.is_possible_answer.cmp(&a.is_possible_answer))
                        .then_with(|| a.word.cmp(&b.word))
                });
            }
            SolverStrategy::Minimax => {
                scored.sort_by(|a, b| {
                    a.worst_case_remaining
                        .cmp(&b.worst_case_remaining)
                        .then_with(|| b.entropy.total_cmp(&a.entropy))
                        .then_with(|| a.expected_remaining.total_cmp(&b.expected_remaining))
                        .then_with(|| b.is_possible_answer.cmp(&a.is_possible_answer))
                        .then_with(|| a.word.cmp(&b.word))
                });
            }
        }

        scored.truncate(self.config.top_k);
        Ok(scored)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn word(s: &str) -> Word {
        Word::parse(s).unwrap()
    }

    #[test]
    fn test_bucket_partition_sum_equals_candidate_count() {
        let candidates = vec![
            word("APPLE"),
            word("APART"),
            word("ALLEY"),
            word("ALERT"),
            word("BANAL"),
            word("CRANE"),
        ];
        let guess = word("SLATE");

        let mut buckets = [0usize; FEEDBACK_PATTERN_COUNT];
        for cand in &candidates {
            let fb = calculate_feedback(cand, &guess);
            buckets[fb.to_pattern_id()] += 1;
        }

        let sum: usize = buckets.iter().sum();
        assert_eq!(sum, candidates.len());
    }

    #[test]
    fn test_entropy_and_metrics_hand_calculated_equal_split() {
        // 4 candidates:
        // Candidate 1 & 2 produce pattern A
        // Candidate 3 & 4 produce pattern B
        // Probability p1 = 2/4 = 0.5, p2 = 2/4 = 0.5
        // Entropy H = -(0.5 * (-1) + 0.5 * (-1)) = 1.0 bit
        // Expected remaining E = (2^2 + 2^2) / 4 = 8 / 4 = 2.0
        // Worst-case W = 2
        let candidates = vec![word("ABACA"), word("ABACK"), word("BBCBB"), word("BBCBD")];

        // Guessing "ZZZZZ" puts all into the same bucket (Absent)
        let (h_same, e_same, w_same) = evaluate_guess_metrics(&word("ZZZZZ"), &candidates);
        assert_eq!(h_same, 0.0);
        assert_eq!(e_same, 4.0);
        assert_eq!(w_same, 4);

        // Guessing "AXXXX" distinguishes words starting with A (ABACA, ABACK) vs B (BBCBB, BBCBD)
        // ABACA vs AXXXX -> [Correct, Absent, Absent, Absent, Absent]
        // ABACK vs AXXXX -> [Correct, Absent, Absent, Absent, Absent]
        // BBCBB vs AXXXX -> [Absent, Absent, Absent, Absent, Absent]
        // BBCBD vs AXXXX -> [Absent, Absent, Absent, Absent, Absent]
        let (h, e, w) = evaluate_guess_metrics(&word("AXXXX"), &candidates);
        assert!(
            (h - 1.0).abs() < 1e-9,
            "Expected 1.0 bit of entropy, got {h}"
        );
        assert!(
            (e - 2.0).abs() < 1e-9,
            "Expected 2.0 candidates remaining, got {e}"
        );
        assert_eq!(w, 2);
    }

    #[test]
    fn test_perfect_partition_maximum_entropy() {
        // 4 candidates split into 4 distinct buckets of 1
        // Entropy H = log2(4) = 2.0 bits
        // Expected remaining E = (1+1+1+1)/4 = 1.0
        // Worst case W = 1
        let candidates = vec![word("AAAAA"), word("BAAAA"), word("CAAAA"), word("DAAAA")];
        let guess = word("ABCDX");

        let (h, e, w) = evaluate_guess_metrics(&guess, &candidates);
        assert!((h - 2.0).abs() < 1e-9, "Expected 2.0 bits, got {h}");
        assert!((e - 1.0).abs() < 1e-9, "Expected 1.0, got {e}");
        assert_eq!(w, 1);
    }

    #[test]
    fn test_single_candidate_scenario() {
        let candidates = vec![word("CRANE")];
        let (h, e, w) = evaluate_guess_metrics(&word("CRANE"), &candidates);
        assert_eq!(h, 0.0);
        assert_eq!(e, 1.0);
        assert_eq!(w, 1);
    }

    #[test]
    fn test_solver_information_gain_ranking() {
        let candidates =
            CandidateSet::new(&[word("APPLE"), word("ALLEY"), word("APART"), word("ALERT")]);
        let legal = vec![word("CRANE"), word("SLATE"), word("APPLE"), word("ZZZZZ")];
        let prev = vec![];

        let solver = Solver::new(SolverConfig::new(SolverStrategy::InformationGain, 3).unwrap());
        let suggestions = solver.suggest(&candidates, &legal, &prev).unwrap();

        assert_eq!(suggestions.len(), 3);
        // Verify monotonic descending entropy
        for i in 0..suggestions.len() - 1 {
            assert!(
                suggestions[i].entropy >= suggestions[i + 1].entropy,
                "Suggestions must be ordered by descending entropy"
            );
        }
    }

    #[test]
    fn test_solver_minimax_ranking() {
        let candidates =
            CandidateSet::new(&[word("APPLE"), word("ALLEY"), word("APART"), word("ALERT")]);
        let legal = vec![word("CRANE"), word("SLATE"), word("APPLE"), word("ZZZZZ")];
        let prev = vec![];

        let solver = Solver::new(SolverConfig::new(SolverStrategy::Minimax, 3).unwrap());
        let suggestions = solver.suggest(&candidates, &legal, &prev).unwrap();

        assert_eq!(suggestions.len(), 3);
        // Verify ascending worst case
        for i in 0..suggestions.len() - 1 {
            assert!(
                suggestions[i].worst_case_remaining <= suggestions[i + 1].worst_case_remaining,
                "Minimax suggestions must be ordered by ascending worst case"
            );
        }
    }

    #[test]
    fn test_solver_excludes_previous_guesses() {
        let candidates = CandidateSet::new(&[word("APPLE"), word("ALLEY")]);
        let legal = vec![word("CRANE"), word("SLATE"), word("APPLE")];
        let prev = vec![word("CRANE")];

        let solver = Solver::new(SolverConfig::default());
        let suggestions = solver.suggest(&candidates, &legal, &prev).unwrap();

        assert!(!suggestions.iter().any(|s| s.word == word("CRANE")));
        assert!(suggestions.iter().any(|s| s.word == word("APPLE")));
    }

    #[test]
    fn test_non_candidate_explorer_guesses_permitted() {
        let candidates = CandidateSet::new(&[word("APPLE"), word("APART")]);
        // "CRANE" is not in candidates, but is a legal guess
        let legal = vec![word("CRANE"), word("APPLE")];
        let prev = vec![];

        let solver = Solver::new(SolverConfig::default());
        let suggestions = solver.suggest(&candidates, &legal, &prev).unwrap();

        let crane_sugg = suggestions.iter().find(|s| s.word == word("CRANE"));
        assert!(crane_sugg.is_some());
        assert!(!crane_sugg.unwrap().is_possible_answer);

        let apple_sugg = suggestions.iter().find(|s| s.word == word("APPLE"));
        assert!(apple_sugg.is_some());
        assert!(apple_sugg.unwrap().is_possible_answer);
    }

    #[test]
    fn test_empty_candidate_set_error() {
        let candidates = CandidateSet::new(&[]);
        let legal = vec![word("CRANE")];
        let prev = vec![];

        let solver = Solver::new(SolverConfig::default());
        let err = solver.suggest(&candidates, &legal, &prev);
        assert_eq!(err, Err(SolverError::EmptyCandidateSet));
    }

    #[test]
    fn test_invalid_top_k_error() {
        assert_eq!(
            SolverConfig::new(SolverStrategy::InformationGain, 0),
            Err(SolverError::InvalidTopK)
        );
    }
}
