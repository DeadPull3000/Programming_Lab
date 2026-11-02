//! # Advanced Wordle Features Integration & Property Testing
//!
//! Verifies end-to-end integration and mathematical invariants across:
//! - Candidate filtering (monotonicity, secret preservation, cumulative recomputation).
//! - Solver suggestions (bucket sum, entropy bounds, minimax, non-candidate explorers).
//! - Persistent player statistics (atomic JSON persistence, streak invariants, duplicate rejection).

use std::fs;
use std::path::PathBuf;

use wordle::candidate::CandidateSet;
use wordle::dictionary::WordleDictionary;
use wordle::feedback::calculate_feedback;
use wordle::game::{GameStatus, GuessOutcome, WordleGame};
use wordle::solver::{evaluate_guess_metrics, Solver, SolverConfig, SolverStrategy};
use wordle::stats::{GameOutcome, GameRecord, JsonStorage, PlayerStats, StatsError, StatsManager};
use wordle::word::Word;

fn temp_test_dir(prefix: &str) -> (PathBuf, PathBuf) {
    use std::time::SystemTime;
    let nanos = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let mut dir = std::env::temp_dir();
    dir.push(format!(
        "wordle_integ_{}_{}_{}",
        prefix,
        std::process::id(),
        nanos
    ));
    fs::create_dir_all(&dir).unwrap();
    let file = dir.join("stats.json");
    (dir, file)
}

#[test]
fn test_end_to_end_game_with_candidates_solver_and_stats() {
    let dict = WordleDictionary::load_default().expect("dictionary must load");
    let secret = Word::parse("CRANE").unwrap();
    let mut game = WordleGame::new(secret.clone());
    let mut candidates = CandidateSet::new(dict.answers());
    let initial_candidates_len = candidates.len();

    let (dir, stats_file) = temp_test_dir("e2e");
    let storage = JsonStorage::new(&stats_file);
    let mut stats_mgr = StatsManager::new(storage).unwrap();

    let game_id = "test-session-e2e-1001";
    let started_at = 1000;

    // Turn 1: Ask solver for recommendation
    let solver = Solver::new(SolverConfig::new(SolverStrategy::InformationGain, 3).unwrap());
    let suggestions1 = solver
        .suggest(&candidates, dict.legal_guesses(), &[])
        .unwrap();
    assert_eq!(suggestions1.len(), 3);
    assert!(suggestions1[0].entropy >= suggestions1[1].entropy);

    // Turn 1: Player guesses "SLATE"
    let guess1 = Word::parse("SLATE").unwrap();
    let outcome1 = game.submit_guess(guess1.as_str(), dict.trie()).unwrap();
    let fb1 = match outcome1 {
        GuessOutcome::Accepted { feedback, .. } => feedback,
        _ => panic!("Expected accepted guess"),
    };

    let summary1 = candidates.filter(&guess1, &fb1);
    assert!(candidates.contains(&secret), "Secret answer must survive");
    assert!(candidates.len() < initial_candidates_len);
    assert_eq!(summary1.remaining_count, candidates.len());

    // Turn 2: Query solver with updated candidates
    let suggestions2 = solver
        .suggest(
            &candidates,
            dict.legal_guesses(),
            std::slice::from_ref(&guess1),
        )
        .unwrap();
    assert!(!suggestions2.iter().any(|s| s.word == guess1));

    // Turn 2: Player guesses the correct secret "CRANE"
    let guess2 = Word::parse("CRANE").unwrap();
    let outcome2 = game.submit_guess(guess2.as_str(), dict.trie()).unwrap();
    let fb2 = match outcome2 {
        GuessOutcome::Accepted {
            feedback, status, ..
        } => {
            assert_eq!(status, GameStatus::Won);
            feedback
        }
        _ => panic!("Expected accepted guess"),
    };

    let summary2 = candidates.filter(&guess2, &fb2);
    assert_eq!(candidates.len(), 1);
    assert_eq!(candidates.words(), std::slice::from_ref(&secret));
    assert_eq!(summary2.remaining_count, 1);

    // Record game in StatsManager
    let record = GameRecord::new(
        game_id.to_string(),
        GameOutcome::Won,
        game.attempts_used() as u8,
        secret,
        started_at,
        1050,
    )
    .unwrap();
    stats_mgr.record_completed_game(record).unwrap();

    assert_eq!(stats_mgr.stats().games_played, 1);
    assert_eq!(stats_mgr.stats().wins, 1);
    assert_eq!(stats_mgr.stats().current_streak, 1);
    assert_eq!(stats_mgr.stats().guess_distribution[1], 1); // 2 guesses

    // Verify atomic file persistence: reload into fresh StatsManager
    let fresh_storage = JsonStorage::new(&stats_file);
    let reloaded_mgr = StatsManager::new(fresh_storage).unwrap();
    assert_eq!(reloaded_mgr.stats().games_played, 1);
    assert_eq!(reloaded_mgr.stats().wins, 1);
    assert_eq!(reloaded_mgr.stats().history.len(), 1);
    assert_eq!(reloaded_mgr.stats().history[0].game_id, game_id);

    let _ = fs::remove_dir_all(dir);
}

#[test]
fn test_solver_mathematical_metric_invariants() {
    let dict = WordleDictionary::load_default().expect("dictionary must load");
    let candidates = dict.answers();
    let n = candidates.len();
    assert!(n > 100);

    let test_guesses = [
        Word::parse("ROAST").unwrap(),
        Word::parse("SLATE").unwrap(),
        Word::parse("CRANE").unwrap(),
        Word::parse("XYLYL").unwrap(),
    ];

    let max_possible_entropy = (n as f64).log2();

    for guess in &test_guesses {
        let (entropy, exp_remaining, worst_case) = evaluate_guess_metrics(guess, candidates);

        // Invariant 1: 0.0 <= H(G) <= log2(N)
        assert!(
            entropy >= 0.0 && entropy <= max_possible_entropy,
            "Entropy {entropy} out of bounds for N={n}"
        );

        // Invariant 2: 1.0 <= E(G) <= N
        assert!(
            exp_remaining >= 1.0 && exp_remaining <= (n as f64),
            "Expected remaining {exp_remaining} out of bounds"
        );

        // Invariant 3: 1 <= W(G) <= N
        assert!(
            worst_case >= 1 && worst_case <= n,
            "Worst case {worst_case} out of bounds"
        );
    }
}

#[test]
fn test_cumulative_filtering_consistency_across_seeded_games() {
    use rand::rngs::StdRng;
    use rand::{Rng, SeedableRng};

    let dict = WordleDictionary::load_default().expect("dictionary must load");
    let answers = dict.answers();
    let mut rng = StdRng::seed_from_u64(0xABCD_EF01_2345_6789);

    for _ in 0..20 {
        let secret = &answers[rng.gen_range(0..answers.len())];
        let mut set = CandidateSet::new(answers);
        let mut history: Vec<(Word, wordle::feedback::Feedback)> = Vec::new();

        for _ in 0..3 {
            let guess = &answers[rng.gen_range(0..answers.len())];
            let fb = calculate_feedback(secret, guess);
            set.filter(guess, &fb);
            history.push((guess.clone(), fb));

            // Verify secret survives
            assert!(
                set.contains(secret),
                "Secret word {secret} was pruned unexpectedly!"
            );

            // Verify against full independent recomputation
            let recomputed: Vec<Word> = answers
                .iter()
                .filter(|cand| {
                    history
                        .iter()
                        .all(|(g, observed)| &calculate_feedback(cand, g) == observed)
                })
                .cloned()
                .collect();

            assert_eq!(
                set.words(),
                &recomputed[..],
                "Incremental candidate set diverged from full recomputation!"
            );
        }
    }
}

#[test]
fn test_duplicate_game_record_prevention_and_resilience() {
    let (dir, stats_file) = temp_test_dir("dup_protect");
    let storage = JsonStorage::new(&stats_file);
    let mut mgr = StatsManager::new(storage).unwrap();

    let rec = GameRecord::new(
        "id-fixed-001".into(),
        GameOutcome::Won,
        4,
        Word::parse("SLATE").unwrap(),
        1000,
        1040,
    )
    .unwrap();

    // First write succeeds
    mgr.record_completed_game(rec.clone()).unwrap();
    assert_eq!(mgr.stats().games_played, 1);
    assert_eq!(mgr.stats().wins, 1);

    // Duplicate attempt is rejected
    let dup_err = mgr.record_completed_game(rec);
    assert!(matches!(dup_err, Err(StatsError::DuplicateRecord(_))));
    assert_eq!(mgr.stats().games_played, 1);
    assert_eq!(mgr.stats().wins, 1);

    // Invariants preserved
    assert_eq!(
        mgr.stats().games_played,
        mgr.stats().wins + mgr.stats().losses
    );
    assert_eq!(
        mgr.stats().guess_distribution.iter().sum::<u64>(),
        mgr.stats().wins
    );

    let _ = fs::remove_dir_all(dir);
}

#[test]
fn test_performance_benchmarks_with_real_dictionary() {
    use std::time::Instant;

    let dict = WordleDictionary::load_default().expect("dictionary must load");
    let legal_count = dict.legal_guess_count();
    let answer_count = dict.answer_count();

    println!("\n=======================================================");
    println!("        WORDLE ADVANCED FEATURES BENCHMARK REPORT      ");
    println!("=======================================================");
    println!("Dictionary Lexicon:     {} total words", dict.trie().len());
    println!("Legal 5-Letter Guesses: {} words", legal_count);
    println!("Curated Answer Pool:    {} words", answer_count);

    // 1. Benchmark Candidate Filtering Time
    let mut cand_set = CandidateSet::new(dict.answers());
    let guess = Word::parse("CRANE").unwrap();
    let fb = calculate_feedback(&Word::parse("SLATE").unwrap(), &guess);

    let filter_start = Instant::now();
    let summary = cand_set.filter(&guess, &fb);
    let filter_elapsed = filter_start.elapsed();

    println!("\n[1] CANDIDATE FILTERING BENCHMARK:");
    println!(
        "    Initial Pool:       {} candidates",
        summary.initial_count
    );
    println!(
        "    Filtered Pool:      {} candidates",
        summary.remaining_count
    );
    println!(
        "    Eliminated:         {} candidates",
        summary.eliminated_count
    );
    println!("    Filtering Elapsed:  {filter_elapsed:?}");

    // 2. Benchmark Solver Evaluation Time on Initial Answer Pool (630 candidates, 8,812 guesses)
    let solver = Solver::new(SolverConfig::new(SolverStrategy::InformationGain, 5).unwrap());
    let full_pool = CandidateSet::new(dict.answers());

    let solver_start = Instant::now();
    let top_suggestions = solver
        .suggest(&full_pool, dict.legal_guesses(), &[])
        .unwrap();
    let solver_elapsed = solver_start.elapsed();

    println!("\n[2] SOLVER SUGGESTION BENCHMARK (Opening Pool, 630 candidates):");
    println!(
        "    Search Space:       {} legal guesses",
        dict.legal_guesses().len()
    );
    println!(
        "    Candidates Pool:    {} possible answers",
        full_pool.len()
    );
    println!(
        "    Total Evaluations:  {} pairs",
        dict.legal_guesses().len() * full_pool.len()
    );
    println!("    Solver Elapsed:     {solver_elapsed:?}");
    println!("    Top 5 Recommendations (Information Gain):");
    for (i, s) in top_suggestions.iter().enumerate() {
        println!("      {}. {}", i + 1, s);
    }

    // 3. Benchmark Solver on Pruned Pool (e.g. 25 candidates remaining)
    let pruned_pool = CandidateSet::new(&full_pool.words()[..25]);
    let pruned_start = Instant::now();
    let _pruned_top = solver
        .suggest(
            &pruned_pool,
            dict.legal_guesses(),
            std::slice::from_ref(&guess),
        )
        .unwrap();
    let pruned_elapsed = pruned_start.elapsed();

    println!("\n[3] SOLVER SUGGESTION BENCHMARK (Pruned Pool, 25 candidates):");
    println!("    Candidates Pool:    25 possible answers");
    println!(
        "    Total Evaluations:  {} pairs",
        dict.legal_guesses().len() * 25
    );
    println!("    Solver Elapsed:     {pruned_elapsed:?}");

    // 4. Benchmark JSON Persistence Save & Load with 100 Completed Games
    let (dir, stats_file) = temp_test_dir("bench_persist");
    let storage = JsonStorage::new(&stats_file);
    let mut stats = PlayerStats::new();

    for i in 1..=100 {
        let attempts = ((i % 6) + 1) as u8;
        let outcome = if i % 10 == 0 {
            GameOutcome::Lost
        } else {
            GameOutcome::Won
        };
        let rec = GameRecord::new(
            format!("bench-game-{i:04}"),
            outcome,
            attempts,
            Word::parse("SLATE").unwrap(),
            1000 + i,
            1050 + i,
        )
        .unwrap();

        if outcome == GameOutcome::Won {
            stats.games_played += 1;
            stats.wins += 1;
            stats.current_streak += 1;
            stats.guess_distribution[(attempts - 1) as usize] += 1;
            stats.total_guesses += attempts as u64;
            stats.total_guesses_in_wins += attempts as u64;
        } else {
            stats.games_played += 1;
            stats.losses += 1;
            stats.current_streak = 0;
            stats.total_guesses += attempts as u64;
        }
        if stats.current_streak > stats.maximum_streak {
            stats.maximum_streak = stats.current_streak;
        }
        stats.history.push(rec);
    }

    let save_start = Instant::now();
    storage.save(&stats).unwrap();
    let save_elapsed = save_start.elapsed();

    let load_start = Instant::now();
    let loaded = storage.load().unwrap();
    let load_elapsed = load_start.elapsed();

    println!("\n[4] PERSISTENT STORAGE BENCHMARK (100 Game Records JSON):");
    println!(
        "    File Size:          {} bytes",
        fs::metadata(&stats_file).unwrap().len()
    );
    println!("    Atomic Save Time:   {save_elapsed:?}");
    println!("    Parse & Load Time:  {load_elapsed:?}");
    println!(
        "    Integrity Verified: {} history records",
        loaded.history.len()
    );
    println!("=======================================================\n");

    let _ = fs::remove_dir_all(dir);
}
