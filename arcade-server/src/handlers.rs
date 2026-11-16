//! # Request Handlers for Arcade API
//!
//! Provides clean, strongly-typed JSON request processing and response generation
//! for Wordle, Spelling Bee, Boggle, Statistics, and Settings.

use rand::seq::SliceRandom;
use rand::thread_rng;
use std::time::Instant;
use wordle::stats::storage::{quote_json_string, JsonParser, JsonValue};

use boggle::board::Board;
use boggle::game::{BoggleGame, BoggleGuessResult};
use boggle::scoring::calculate_score as boggle_calc_score;
use boggle::validator::solve_boggle_board;
use spelling_bee::game::{GuessResult, SpellingBeeGame};
use spelling_bee::hints::SpellingBeeHints;
use spelling_bee::puzzle::SpellingBeePuzzle;
use spelling_bee::scoring::calculate_score as bee_calc_score;
use spelling_bee::solver::solve_spelling_bee;
use wordle::candidate::CandidateSet;
use wordle::game::{GameStatus, GuessOutcome, WordleGame};
use wordle::keyboard::{KeyStatus, KeyboardState};
use wordle::solver::{Solver, SolverConfig, SolverStrategy};
use wordle::word::Word;

use crate::state::{ArcadeState, BoggleSession, SpellingBeeSession, WordleSession};

// ============================================================================
// Helper JSON Extractors
// ============================================================================

fn extract_param(body: &str, key: &str) -> Option<JsonValue> {
    let value = JsonParser::new(body).parse().ok()?;
    value.as_object()?.get(key).cloned()
}

pub fn extract_string_param(body: &str, key: &str) -> Option<String> {
    extract_param(body, key)?.as_str().map(str::to_owned)
}

pub fn extract_usize_param(body: &str, key: &str) -> Option<usize> {
    usize::try_from(extract_param(body, key)?.as_u64()?).ok()
}

pub fn extract_bool_param(body: &str, key: &str) -> Option<bool> {
    match extract_param(body, key)? {
        JsonValue::Bool(value) => Some(value),
        _ => None,
    }
}

pub fn error_json(message: &str) -> String {
    format!(r#"{{"error":{}}}"#, quote_json_string(message))
}

fn rejected_guess_json(result: &str, message: &str) -> String {
    format!(
        r#"{{"result":{},"message":{},"points":0}}"#,
        quote_json_string(result),
        quote_json_string(message)
    )
}

// ============================================================================
// Status and Summary Handlers
// ============================================================================

pub fn handle_status(state: &ArcadeState) -> String {
    format!(
        r#"{{"status":"ok","version":"{}","answers_count":{},"legal_guess_count":{},"dictionary_loaded":true}}"#,
        crate::server_version(),
        state.dictionary.answers().len(),
        state.dictionary.legal_guesses().len()
    )
}

pub fn handle_games_summary(state: &ArcadeState) -> String {
    let wordle_active = state
        .wordle
        .as_ref()
        .map(|s| s.game.status() == GameStatus::InProgress)
        .unwrap_or(false);
    let spelling_bee_active = state
        .spelling_bee
        .as_ref()
        .map(|s| !s.game.is_queen_bee())
        .unwrap_or(false);
    let boggle_active = state
        .boggle
        .as_ref()
        .map(|s| s.game.is_active && !s.has_expired())
        .unwrap_or(false);

    format!(
        r#"{{"wordle_active":{},"spelling_bee_active":{},"boggle_active":{}}}"#,
        wordle_active, spelling_bee_active, boggle_active
    )
}

// ============================================================================
// Wordle Handlers
// ============================================================================

fn wordle_keyboard_json(keyboard: &KeyboardState) -> String {
    let mut entries = Vec::new();
    for b in b'A'..=b'Z' {
        let ch = b as char;
        let status_str = match keyboard.status_of(ch) {
            KeyStatus::Correct => "Correct",
            KeyStatus::Present => "Present",
            KeyStatus::Absent => "Absent",
            KeyStatus::Unknown => "Unknown",
        };
        entries.push(format!(r#""{}":"{}""#, ch, status_str));
    }
    format!("{{{}}}", entries.join(","))
}

fn wordle_guesses_json(game: &WordleGame) -> String {
    let mut records = Vec::new();
    for guess in game.guesses() {
        let mut fb_strs = Vec::new();
        for &fb in guess.feedback.as_slice() {
            let s = match fb {
                wordle::feedback::LetterFeedback::Correct => "Correct",
                wordle::feedback::LetterFeedback::Present => "Present",
                wordle::feedback::LetterFeedback::Absent => "Absent",
            };
            fb_strs.push(format!(r#""{}""#, s));
        }
        records.push(format!(
            r#"{{"word":"{}","feedback":[{}]}}"#,
            guess.word.as_str(),
            fb_strs.join(",")
        ));
    }
    format!("[{}]", records.join(","))
}

pub fn handle_wordle_state(state: &ArcadeState) -> String {
    match &state.wordle {
        Some(session) => {
            let status_str = match session.game.status() {
                GameStatus::InProgress => "InProgress",
                GameStatus::Won => "Won",
                GameStatus::Lost => "Lost",
            };
            let secret_val = if session.game.status() != GameStatus::InProgress {
                format!(r#""{}""#, session.game.answer().as_str())
            } else {
                "null".to_string()
            };

            format!(
                r#"{{"attempts_remaining":{},"max_attempts":{},"guesses":{},"game_status":"{}","secret":{},"keyboard_state":{},"remaining_candidates_count":{}}}"#,
                session.game.attempts_remaining(),
                session.game.max_attempts(),
                wordle_guesses_json(&session.game),
                status_str,
                secret_val,
                wordle_keyboard_json(&session.keyboard),
                session.candidate_set.len()
            )
        }
        None => r#"{"game_status":"NotStarted","attempts_remaining":6,"max_attempts":6,"guesses":[],"secret":null,"keyboard_state":{},"remaining_candidates_count":0}"#.to_string(),
    }
}

pub fn handle_wordle_start(state: &mut ArcadeState, body: &str) -> String {
    let mut rng = thread_rng();
    let secret = if let Some(sec_str) = extract_string_param(body, "secret") {
        match Word::parse(&sec_str) {
            Ok(word) if state.dictionary.answers().contains(&word) => word,
            _ => return error_json("Secret must belong to the curated answer pool"),
        }
    } else {
        match state.dictionary.choose_random_answer(&mut rng) {
            Some(word) => word,
            None => return error_json("No answers available"),
        }
    };

    let game = WordleGame::new(secret);
    let candidate_set = CandidateSet::new(state.dictionary.answers());
    let keyboard = KeyboardState::new();

    state.wordle = Some(WordleSession {
        game,
        candidate_set,
        keyboard,
        recorded_in_stats: false,
    });

    handle_wordle_state(state)
}

pub fn handle_wordle_guess(state: &mut ArcadeState, body: &str) -> String {
    let guess_str = match extract_string_param(body, "guess") {
        Some(g) => g.trim().to_owned(),
        None => return r#"{"error":"Missing 'guess' parameter"}"#.to_string(),
    };

    let guess_word = match Word::parse(&guess_str) {
        Ok(w) => w,
        Err(_) => return r#"{"error":"Guess must be a 5-letter English word"}"#.to_string(),
    };

    let (status, attempts_used) = {
        let trie = state.dictionary.trie();
        let session = match &mut state.wordle {
            Some(s) => s,
            None => return r#"{"error":"No active Wordle game. Start one first."}"#.to_string(),
        };

        if session.game.status() != GameStatus::InProgress {
            return r#"{"error":"Current game has already concluded. Start a new one."}"#
                .to_string();
        }

        match session.game.submit_guess(&guess_str, trie) {
            Ok(GuessOutcome::Accepted {
                feedback, status, ..
            }) => {
                session.candidate_set.filter(&guess_word, &feedback);
                session.keyboard.update_from_guess(&guess_word, &feedback);
                let attempts = session.game.guesses().len();
                (status, attempts)
            }
            Ok(GuessOutcome::NotInDictionary) => {
                return r#"{"error":"Word not in dictionary"}"#.to_string();
            }
            Ok(GuessOutcome::AlreadyGuessed) => {
                return r#"{"error":"Word already guessed"}"#.to_string();
            }
            Err(e) => {
                return error_json(&e.to_string());
            }
        }
    };

    if status != GameStatus::InProgress
        && state.wordle.as_ref().is_some_and(|s| !s.recorded_in_stats)
    {
        if let Err(e) = state.update_stats(|stats| {
            stats
                .wordle
                .record_outcome(status == GameStatus::Won, attempts_used)
        }) {
            return error_json(&format!(
                "Game completed, but statistics could not be saved: {e}"
            ));
        }
        state.wordle.as_mut().unwrap().recorded_in_stats = true;
    }

    handle_wordle_state(state)
}

pub fn handle_wordle_hint(state: &ArcadeState, body: &str) -> String {
    let session = match &state.wordle {
        Some(s) => s,
        None => return r#"{"error":"No active Wordle game"}"#.to_string(),
    };

    let count = session.candidate_set.len();

    if count == 0 {
        return r#"{"remaining_candidates":0,"is_single_candidate":false,"suggestions":[]}"#
            .to_string();
    }

    if count == 1 {
        let only_word = session.candidate_set.words()[0].as_str();
        return format!(
            r#"{{"remaining_candidates":1,"is_single_candidate":true,"suggestions":[{{"word":"{}","expected_info_bits":0.0,"expected_remaining":1.0,"worst_case_remaining":1,"is_possible_answer":true}}]}}"#,
            only_word
        );
    }

    let top_n = extract_usize_param(body, "top_n").unwrap_or(5).clamp(1, 20);
    let strategy = match extract_string_param(body, "strategy").as_deref() {
        Some("minimax") => SolverStrategy::Minimax,
        _ => SolverStrategy::InformationGain,
    };

    let config = SolverConfig::new(strategy, top_n).unwrap_or_default();
    let solver = Solver::new(config);

    let previous_guesses: Vec<Word> = session
        .game
        .guesses()
        .iter()
        .map(|g| g.word.clone())
        .collect();

    let ranked = solver
        .suggest(
            &session.candidate_set,
            state.dictionary.legal_guesses(),
            &previous_guesses,
        )
        .unwrap_or_default();

    let mut sugg_jsons = Vec::new();
    for s in ranked {
        sugg_jsons.push(format!(
            r#"{{"word":"{}","expected_info_bits":{:.2},"expected_remaining":{:.1},"worst_case_remaining":{},"is_possible_answer":{}}}"#,
            s.word.as_str(),
            s.entropy,
            s.expected_remaining,
            s.worst_case_remaining,
            s.is_possible_answer
        ));
    }

    format!(
        r#"{{"remaining_candidates":{},"is_single_candidate":false,"suggestions":[{}]}}"#,
        count,
        sugg_jsons.join(",")
    )
}

// ============================================================================
// Spelling Bee Handlers
// ============================================================================

fn bee_rank_name(score: u32, max_score: u32) -> &'static str {
    if max_score == 0 {
        return "Beginner";
    }
    let pct = (score as f64 / max_score as f64) * 100.0;
    if pct >= 100.0 {
        "Queen Bee 👑"
    } else if pct >= 70.0 {
        "Genius"
    } else if pct >= 50.0 {
        "Amazing"
    } else if pct >= 30.0 {
        "Solid"
    } else if pct >= 15.0 {
        "Moving Up"
    } else if pct >= 5.0 {
        "Good Start"
    } else {
        "Beginner"
    }
}

pub fn handle_spelling_bee_state(state: &ArcadeState) -> String {
    match &state.spelling_bee {
        Some(session) => {
            let max_score = session.max_possible_score;
            let rank = bee_rank_name(session.game.score, max_score);
            let mut words_sorted: Vec<String> = session.game.guessed.iter().cloned().collect();
            words_sorted.sort();

            let words_json = words_sorted
                .iter()
                .map(|w| format!(r#""{}""#, w))
                .collect::<Vec<_>>()
                .join(",");

            let outer_json = session
                .outer_letters_shuffled
                .iter()
                .map(|c| format!(r#""{}""#, c))
                .collect::<Vec<_>>()
                .join(",");

            format!(
                r#"{{"main_letter":"{}","outer_letters":[{}],"score":{},"max_possible_score":{},"words_found":[{}],"total_solutions":{},"pangrams_found":{},"is_queen_bee":{},"rank":"{}"}}"#,
                session.game.puzzle.main_letter,
                outer_json,
                session.game.score,
                max_score,
                words_json,
                session.game.total_solutions(),
                session.pangrams_found,
                session.game.is_queen_bee(),
                rank
            )
        }
        None => r#"{"main_letter":"","outer_letters":[],"score":0,"max_possible_score":0,"words_found":[],"total_solutions":0,"pangrams_found":0,"is_queen_bee":false,"rank":"Beginner"}"#.to_string(),
    }
}

pub fn handle_spelling_bee_start(state: &mut ArcadeState) -> String {
    if let Some(session) = &state.spelling_bee {
        if !session.recorded_in_stats && session.game.count_guessed() > 0 {
            let (score, words, pangrams, queen) = (
                session.game.score,
                session.game.count_guessed() as u32,
                session.pangrams_found,
                session.game.is_queen_bee(),
            );
            if let Err(e) = state.update_stats(|stats| {
                stats
                    .spelling_bee
                    .record_game(score, words, pangrams, queen)
            }) {
                return error_json(&format!("Could not save previous puzzle: {e}"));
            }
            state.spelling_bee.as_mut().unwrap().recorded_in_stats = true;
        }
    }
    let mut rng = thread_rng();
    let trie = state.dictionary.trie();

    // Find a puzzle with at least 15 valid solutions
    let mut chosen_puzzle = None;
    let mut chosen_solutions = None;

    for _ in 0..100 {
        let puzzle = SpellingBeePuzzle::random(&mut rng);
        let solutions = solve_spelling_bee(trie, &puzzle);
        if solutions.len() >= 15 {
            chosen_puzzle = Some(puzzle);
            chosen_solutions = Some(solutions);
            break;
        }
    }

    let (puzzle, solutions) = match (chosen_puzzle, chosen_solutions) {
        (Some(p), Some(s)) => (p, s),
        _ => {
            // Curated fallback puzzle [E, B, C, O, M, N, Y]
            let fallback_puzzle = SpellingBeePuzzle::new(['E', 'B', 'C', 'O', 'M', 'N', 'Y'], 'E')
                .expect("fallback puzzle valid");
            let fallback_solutions = solve_spelling_bee(trie, &fallback_puzzle);
            (fallback_puzzle, fallback_solutions)
        }
    };

    let max_possible_score: u32 = solutions
        .iter()
        .map(|w| {
            let mask = puzzle.word_mask(w).unwrap_or(0);
            let is_pangram = SpellingBeePuzzle::mask_is_pangram(mask);
            bee_calc_score(w, is_pangram)
        })
        .sum();

    let mut outer = [
        puzzle.letters[1],
        puzzle.letters[2],
        puzzle.letters[3],
        puzzle.letters[4],
        puzzle.letters[5],
        puzzle.letters[6],
    ];
    outer.shuffle(&mut rng);

    let game = SpellingBeeGame::new(puzzle, solutions);

    state.spelling_bee = Some(SpellingBeeSession {
        game,
        outer_letters_shuffled: outer,
        max_possible_score,
        pangrams_found: 0,
        recorded_in_stats: false,
    });

    handle_spelling_bee_state(state)
}

pub fn handle_spelling_bee_submit(state: &mut ArcadeState, body: &str) -> String {
    let word = match extract_string_param(body, "word") {
        Some(w) => w.trim().to_owned(),
        None => return r#"{"error":"Missing 'word' parameter"}"#.to_string(),
    };

    let (res, score, words_count, pangrams, is_queen, max_score) = {
        let session = match &mut state.spelling_bee {
            Some(s) => s,
            None => return r#"{"error":"No active Spelling Bee game"}"#.to_string(),
        };

        let result = session.game.submit_guess(&word);
        if let GuessResult::Accepted { pangram, .. } = result {
            if pangram {
                session.pangrams_found += 1;
            }
        }
        let is_queen = session.game.is_queen_bee();
        (
            result,
            session.game.score,
            session.game.count_guessed(),
            session.pangrams_found,
            is_queen,
            session.max_possible_score,
        )
    };

    if is_queen
        && state
            .spelling_bee
            .as_ref()
            .is_some_and(|s| !s.recorded_in_stats)
    {
        if let Err(e) = state.update_stats(|stats| {
            stats
                .spelling_bee
                .record_game(score, words_count as u32, pangrams, true)
        }) {
            return error_json(&format!(
                "Puzzle completed, but statistics could not be saved: {e}"
            ));
        }
        state.spelling_bee.as_mut().unwrap().recorded_in_stats = true;
    }

    let rank = bee_rank_name(score, max_score);

    match res {
        GuessResult::Accepted {
            points,
            pangram,
            word: _,
        } => {
            let msg = if pangram {
                format!("PANGRAM! +{} points!", points)
            } else {
                format!("Nice! +{} points", points)
            };
            format!(
                r#"{{"result":"Accepted","message":"{}","points":{},"is_pangram":{},"score":{},"rank":"{}","state":{}}}"#,
                msg,
                points,
                pangram,
                score,
                rank,
                handle_spelling_bee_state(state)
            )
        }
        GuessResult::TooShort => {
            r#"{"result":"TooShort","message":"Word is too short (must be 4+ letters)","points":0}"#
                .to_string()
        }
        GuessResult::InvalidCharacters(c) => rejected_guess_json(
            "InvalidCharacters",
            &format!("Contains invalid character '{c}'"),
        ),
        GuessResult::MissingMainLetter(c) => format!(
            r#"{{"result":"MissingMainLetter","message":"Missing mandatory center letter [{}]","points":0}}"#,
            c
        ),
        GuessResult::InvalidLetters(c) => format!(
            r#"{{"result":"InvalidLetters","message":"Contains letter '{}' not in puzzle","points":0}}"#,
            c
        ),
        GuessResult::NotInSolutions => {
            r#"{"result":"NotInSolutions","message":"Word is not in the dictionary","points":0}"#
                .to_string()
        }
        GuessResult::AlreadyGuessed => {
            r#"{"result":"AlreadyGuessed","message":"Already found that word!","points":0}"#
                .to_string()
        }
    }
}

pub fn handle_spelling_bee_shuffle(state: &mut ArcadeState) -> String {
    let session = match &mut state.spelling_bee {
        Some(s) => s,
        None => return r#"{"error":"No active Spelling Bee game"}"#.to_string(),
    };

    let mut rng = thread_rng();
    session.outer_letters_shuffled.shuffle(&mut rng);

    let outer_json = session
        .outer_letters_shuffled
        .iter()
        .map(|c| format!(r#""{}""#, c))
        .collect::<Vec<_>>()
        .join(",");

    format!(r#"{{"outer_letters":[{}]}}"#, outer_json)
}

pub fn handle_spelling_bee_hints(state: &ArcadeState) -> String {
    let session = match &state.spelling_bee {
        Some(s) => s,
        None => return r#"{"error":"No active Spelling Bee game"}"#.to_string(),
    };

    let hints = SpellingBeeHints::compute(&session.game.solutions);

    let len_jsons = hints
        .grid
        .col_totals
        .iter()
        .map(|(len, cnt)| format!(r#""{}":{}"#, len, cnt))
        .collect::<Vec<_>>()
        .join(",");

    let first_letter_jsons = hints
        .grid
        .row_totals
        .iter()
        .map(|(ch, cnt)| format!(r#""{}":{}"#, ch, cnt))
        .collect::<Vec<_>>()
        .join(",");

    let prefix_jsons = hints
        .prefixes
        .counts
        .iter()
        .map(|(pref, cnt)| format!(r#""{}":{}"#, pref, cnt))
        .collect::<Vec<_>>()
        .join(",");

    let pangram_count: usize = session
        .game
        .solutions
        .iter()
        .filter(|w| {
            session
                .game
                .puzzle
                .word_mask(w)
                .map(SpellingBeePuzzle::mask_is_pangram)
                .unwrap_or(false)
        })
        .count();

    format!(
        r#"{{"total_words":{},"total_points":{},"pangram_count":{},"length_counts":{{{}}},"first_letter_counts":{{{}}},"prefix_counts":{{{}}}}}"#,
        hints.grid.grand_total,
        session.max_possible_score,
        pangram_count,
        len_jsons,
        first_letter_jsons,
        prefix_jsons
    )
}

// ============================================================================
// Boggle Handlers
// ============================================================================

pub fn handle_boggle_state(state: &ArcadeState) -> String {
    match &state.boggle {
        Some(session) => {
            let size = session.size;
            let cells_json = session
                .game
                .board
                .cells
                .iter()
                .map(|c| format!(r#""{}""#, c))
                .collect::<Vec<_>>()
                .join(",");

            let mut row_jsons = Vec::new();
            for r in 0..size {
                let start = r * size;
                let end = start + size;
                let row_chars = session.game.board.cells[start..end]
                    .iter()
                    .map(|c| format!(r#""{}""#, c))
                    .collect::<Vec<_>>()
                    .join(",");
                row_jsons.push(format!("[{}]", row_chars));
            }
            let grid_json = format!("[{}]", row_jsons.join(","));

            let mut words_sorted: Vec<String> = session.game.accepted_words.iter().cloned().collect();
            words_sorted.sort();

            let accepted_json = words_sorted
                .iter()
                .map(|w| format!(r#""{}""#, w))
                .collect::<Vec<_>>()
                .join(",");

            let ends_at_str = match session.ends_at_unix_ms {
                Some(ms) => ms.to_string(),
                None => "null".to_string(),
            };

            format!(
                r#"{{"size":{},"cells":[{}],"grid":{},"score":{},"accepted_words":[{}],"total_possible_words":{},"max_possible_score":{},"is_active":{},"ends_at_unix_ms":{}}}"#,
                size,
                cells_json,
                grid_json,
                session.game.score,
                accepted_json,
                session.all_solutions.len(),
                session.max_possible_score,
                session.game.is_active && !session.has_expired(),
                ends_at_str
            )
        }
        None => r#"{"size":4,"cells":[],"grid":[],"score":0,"accepted_words":[],"total_possible_words":0,"max_possible_score":0,"is_active":false,"ends_at_unix_ms":null}"#.to_string(),
    }
}

pub fn handle_boggle_start(state: &mut ArcadeState, body: &str) -> String {
    let size = extract_usize_param(body, "size").unwrap_or(4);
    if size != 4 && size != 5 && size != 6 {
        return r#"{"error":"Invalid board dimension. Supported sizes are 4, 5, or 6."}"#
            .to_string();
    }

    let duration_secs = extract_usize_param(body, "duration_secs")
        .map(|d| d as u64)
        .unwrap_or(180);

    if state.boggle.as_ref().is_some_and(|s| !s.recorded_in_stats) {
        let result = handle_boggle_end(state);
        if result.contains("\"error\"") {
            return result;
        }
    }
    let mut rng = thread_rng();
    let trie = state.dictionary.trie();

    let board = Board::random(size, &mut rng).expect("board generation succeeds");
    let all_solutions = solve_boggle_board(trie, &board);

    let max_possible_score: u32 = all_solutions
        .iter()
        .map(|w| boggle_calc_score(w.len()))
        .sum();

    let game = BoggleGame::new(board);

    state.boggle = Some(BoggleSession {
        game,
        size,
        all_solutions,
        max_possible_score,
        recorded_in_stats: false,
        started_at: Instant::now(),
        ends_at_unix_ms: BoggleSession::deadline_for_duration(duration_secs),
        duration_secs,
    });

    handle_boggle_state(state)
}

pub fn handle_boggle_submit(state: &mut ArcadeState, body: &str) -> String {
    let word = match extract_string_param(body, "word") {
        Some(w) => w.trim().to_owned(),
        None => return r#"{"error":"Missing 'word' parameter"}"#.to_string(),
    };

    if state.boggle.as_ref().is_some_and(|s| s.has_expired()) {
        let result = handle_boggle_end(state);
        if result.contains("\"error\"") {
            return result;
        }
        return error_json("Boggle round has ended. Start a new round.");
    }
    let trie = state.dictionary.trie();

    let (res, score) = {
        let session = match &mut state.boggle {
            Some(s) => s,
            None => return r#"{"error":"No active Boggle round"}"#.to_string(),
        };

        if !session.game.is_active {
            return r#"{"error":"Boggle round has ended. Start a new round."}"#.to_string();
        }

        let r = BoggleGame::evaluate_single_word(
            &session.game.board,
            trie,
            &mut session.game.accepted_words,
            &word,
        );
        if let BoggleGuessResult::Accepted { points, .. } = &r {
            session.game.score += points;
        }
        (r, session.game.score)
    };

    let state_json = handle_boggle_state(state);

    match res {
        BoggleGuessResult::Accepted { points, word } => {
            format!(
                r#"{{"result":"Accepted","message":"Found {}! +{} pts","points":{},"score":{},"word":"{}","state":{}}}"#,
                word, points, points, score, word, state_json
            )
        }
        BoggleGuessResult::TooShort => r#"{"result":"TooShort","message":"Word must be at least 3 letters","points":0}"#.to_string(),
        BoggleGuessResult::InvalidCharacters(c) => rejected_guess_json("InvalidCharacters", &format!("Invalid letter '{c}'")),
        BoggleGuessResult::NotInDictionary => r#"{"result":"NotInDictionary","message":"Word is not in the dictionary","points":0}"#.to_string(),
        BoggleGuessResult::NotOnBoard => r#"{"result":"NotOnBoard","message":"Cannot trace word across adjacent cells","points":0}"#.to_string(),
        BoggleGuessResult::AlreadyGuessed => r#"{"result":"AlreadyGuessed","message":"Already found that word!","points":0}"#.to_string(),
    }
}

pub fn handle_boggle_end(state: &mut ArcadeState) -> String {
    let (score, total, max_score, acc_cnt, sample, maybe_words) = {
        let session = match &mut state.boggle {
            Some(s) => s,
            None => return r#"{"error":"No active Boggle round"}"#.to_string(),
        };

        session.game.is_active = false;

        let words: Vec<String> = session.game.accepted_words.iter().cloned().collect();
        let score = session.game.score;
        let total = session.all_solutions.len();
        let max_score = session.max_possible_score;
        let acc_cnt = session.game.accepted_words.len();

        let mut missed: Vec<String> = session
            .all_solutions
            .difference(&session.game.accepted_words)
            .cloned()
            .collect();
        missed.sort_by(|a, b| b.len().cmp(&a.len()).then_with(|| a.cmp(b)));
        let sample = missed.iter().take(30).cloned().collect::<Vec<_>>();

        let should_record = !session.recorded_in_stats;
        (
            score,
            total,
            max_score,
            acc_cnt,
            sample,
            should_record.then_some(words),
        )
    };

    if let Some(words) = maybe_words {
        if let Err(e) = state.update_stats(|stats| stats.boggle.record_game(score, &words)) {
            return error_json(&format!(
                "Round ended, but statistics could not be saved: {e}"
            ));
        }
        state.boggle.as_mut().unwrap().recorded_in_stats = true;
    }

    let missed_json = sample
        .iter()
        .map(|w| format!(r#""{}""#, w))
        .collect::<Vec<_>>()
        .join(",");

    let state_json = handle_boggle_state(state);

    format!(
        r#"{{"score":{},"total_possible_words":{},"max_possible_score":{},"accepted_count":{},"missed_sample":[{}],"state":{}}}"#,
        score, total, max_score, acc_cnt, missed_json, state_json
    )
}

// ============================================================================
// Statistics & Settings Handlers
// ============================================================================

pub fn handle_get_stats(state: &ArcadeState) -> String {
    state.stats.to_json()
}

pub fn handle_reset_stats(state: &mut ArcadeState) -> String {
    if let Err(e) = state.update_stats(|stats| *stats = crate::state::UnifiedStats::default()) {
        return error_json(&format!("Could not reset statistics: {e}"));
    }
    r#"{"success":true,"message":"Player statistics reset successfully"}"#.to_string()
}

pub fn handle_get_settings(state: &ArcadeState) -> String {
    state.settings.to_json()
}

pub fn handle_update_settings(state: &mut ArcadeState, body: &str) -> String {
    let mut settings = state.settings.clone();
    if let Some(value) = extract_bool_param(body, "sound_enabled") {
        settings.sound_enabled = value;
    }
    if let Some(value) = extract_bool_param(body, "music_enabled") {
        settings.music_enabled = value;
    }
    if let Some(value) = extract_bool_param(body, "reduced_motion") {
        settings.reduced_motion = value;
    }
    if let Err(e) = state.save_settings(&settings) {
        return error_json(&format!("Could not save settings: {e}"));
    }
    state.settings = settings;
    handle_get_settings(state)
}
