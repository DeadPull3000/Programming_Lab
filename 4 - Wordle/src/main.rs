//! # Wordle CLI Entry Point
//!
//! Interactive command-line Wordle application featuring:
//! - Full Trie-based guess validation and curated secret answer selection.
//! - Dynamic candidate filtering updating possible secret answers after each guess.
//! - Information-theoretic and minimax solver suggestions (:hint).
//! - Persistent player statistics and history across application restarts.

use std::io::{self, BufRead, Write};
use std::path::PathBuf;

use wordle::candidate::CandidateSet;
use wordle::dictionary::WordleDictionary;
use wordle::display::{
    colors, render_board, render_game_over, render_keyboard, render_welcome_banner, DisplayConfig,
};
use wordle::game::{GameError, GameStatus, GuessOutcome, WordleGame};
use wordle::keyboard::KeyboardState;
use wordle::solver::{Solver, SolverConfig, SolverStrategy};
use wordle::stats::{default_stats_path, GameOutcome, GameRecord, JsonStorage, StatsManager};
use wordle::word::Word;

fn main() {
    let stdout = io::stdout();
    let mut out = stdout.lock();
    let stdin = io::stdin();
    let mut reader = stdin.lock();

    let config = DisplayConfig::default();

    // Parse command line arguments: e.g. --stats-file <PATH>
    let mut stats_path = default_stats_path();
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        if arg == "--stats-file" {
            if let Some(path_str) = args.next() {
                stats_path = PathBuf::from(path_str);
            }
        }
    }

    writeln!(out, "Loading Wordle dictionary and answer pool...").unwrap();
    let dictionary = match WordleDictionary::load_default() {
        Ok(dict) => dict,
        Err(e) => {
            eprintln!("Fatal error: Could not load dictionary: {e}");
            std::process::exit(1);
        }
    };

    if dictionary.is_using_fallback() {
        writeln!(
            out,
            "{yellow}[NOTICE] answers.txt was not found or was empty. Using full 5-letter dictionary fallback ({count} answers).{reset}",
            yellow = colors::FG_YELLOW,
            count = dictionary.answer_count(),
            reset = colors::RESET
        )
        .unwrap();
    } else {
        writeln!(
            out,
            "Loaded {dict_words} dictionary words ({legal_guesses} 5-letter) and {ans_words} curated answers.",
            dict_words = dictionary.trie().len(),
            legal_guesses = dictionary.legal_guess_count(),
            ans_words = dictionary.answer_count()
        )
        .unwrap();
    }

    // Initialize persistent statistics manager
    let storage = JsonStorage::new(stats_path);
    let mut stats_manager = match StatsManager::new(storage) {
        Ok(mgr) => mgr,
        Err(e) => {
            eprintln!("Warning: Could not initialize statistics ({e}); starting fresh.");
            StatsManager::default_manager().unwrap_or_else(|_| {
                StatsManager::from_stats(
                    wordle::stats::PlayerStats::new(),
                    JsonStorage::new("stats_fallback.json"),
                )
            })
        }
    };

    let mut rng = rand::thread_rng();

    // Outer session loop: supports replay
    loop {
        let answer = match dictionary.choose_random_answer(&mut rng) {
            Some(w) => w,
            None => {
                eprintln!("Error: Answer pool is empty.");
                break;
            }
        };

        let mut game = WordleGame::new(answer);
        let mut keyboard = KeyboardState::new();
        let mut candidates = CandidateSet::new(dictionary.answers());
        let game_id = generate_game_id();
        let started_at = current_timestamp();

        writeln!(out, "{}", render_welcome_banner(&config)).unwrap();
        writeln!(out, "{}", render_board(&game, &config)).unwrap();
        writeln!(out, "{}", render_keyboard(&keyboard, &config)).unwrap();
        writeln!(
            out,
            "{cyan}Candidates remaining: {count}{reset}",
            cyan = colors::FG_CYAN,
            count = candidates.len(),
            reset = colors::RESET
        )
        .unwrap();

        // Round loop
        while !game.is_over() {
            write!(
                out,
                "\nEnter guess (attempts remaining: {}): ",
                game.attempts_remaining()
            )
            .unwrap();
            out.flush().unwrap();

            let mut input_line = String::new();
            if reader.read_line(&mut input_line).unwrap_or(0) == 0 {
                // EOF reached - mark game abandoned
                let _ = record_terminal_game(
                    &mut stats_manager,
                    &game_id,
                    GameOutcome::Abandoned,
                    game.attempts_used() as u8,
                    game.answer(),
                    started_at,
                );
                writeln!(out, "\nExiting Wordle. Goodbye!").unwrap();
                return;
            }

            let trimmed = input_line.trim();
            if trimmed.is_empty() {
                continue;
            }

            // Command handling
            if trimmed.eq_ignore_ascii_case("QUIT") || trimmed.eq_ignore_ascii_case(":quit") {
                writeln!(
                    out,
                    "\nGiving up? The secret word was: {yellow}{bold}{word}{reset}",
                    yellow = colors::FG_YELLOW,
                    bold = colors::BOLD,
                    word = game.answer().as_str(),
                    reset = colors::RESET
                )
                .unwrap();

                let _ = record_terminal_game(
                    &mut stats_manager,
                    &game_id,
                    GameOutcome::Abandoned,
                    game.attempts_used() as u8,
                    game.answer(),
                    started_at,
                );
                break;
            }

            if trimmed.eq_ignore_ascii_case("STATS") || trimmed.eq_ignore_ascii_case(":stats") {
                writeln!(out, "\n{}", stats_manager.render_summary()).unwrap();
                continue;
            }

            if trimmed.eq_ignore_ascii_case("CANDIDATES")
                || trimmed.eq_ignore_ascii_case(":candidates")
            {
                writeln!(
                    out,
                    "\n{cyan}--- REMAINING CANDIDATES ({count}) ---{reset}",
                    cyan = colors::FG_CYAN,
                    count = candidates.len(),
                    reset = colors::RESET
                )
                .unwrap();

                let words = candidates.words();
                let limit = words.len().min(30);
                for (i, w) in words[..limit].iter().enumerate() {
                    write!(out, "{w} ").unwrap();
                    if (i + 1) % 10 == 0 {
                        writeln!(out).unwrap();
                    }
                }
                if words.len() > limit {
                    writeln!(out, "... and {} more", words.len() - limit).unwrap();
                } else if !words.len().is_multiple_of(10) {
                    writeln!(out).unwrap();
                }
                continue;
            }

            if trimmed.to_ascii_uppercase().starts_with("HINT")
                || trimmed.to_ascii_lowercase().starts_with(":hint")
            {
                let strategy = if trimmed.to_ascii_uppercase().contains("MINIMAX") {
                    SolverStrategy::Minimax
                } else {
                    SolverStrategy::InformationGain
                };

                let config = match SolverConfig::new(strategy, 5) {
                    Ok(c) => c,
                    Err(e) => {
                        writeln!(out, "Config error: {e}").unwrap();
                        continue;
                    }
                };

                let solver = Solver::new(config);
                let prev_words: Vec<Word> = game.guesses().iter().map(|g| g.word.clone()).collect();

                writeln!(out, "\nCalculating top suggestions ({strategy})...").unwrap();
                match solver.suggest(&candidates, dictionary.legal_guesses(), &prev_words) {
                    Ok(suggestions) => {
                        writeln!(
                            out,
                            "{cyan}--- LITTLE HELPER SUGGESTIONS ---{reset}",
                            cyan = colors::FG_CYAN,
                            reset = colors::RESET
                        )
                        .unwrap();
                        for (i, sugg) in suggestions.iter().enumerate() {
                            writeln!(out, "{}. {}", i + 1, sugg).unwrap();
                        }
                    }
                    Err(e) => writeln!(out, "Solver could not produce suggestions: {e}").unwrap(),
                }
                continue;
            }

            if trimmed.eq_ignore_ascii_case("HELP") || trimmed.eq_ignore_ascii_case(":help") {
                writeln!(
                    out,
                    "\n--- WORDLE RULES & COMMANDS ---\n\
                     - Guess the hidden 5-letter word in 6 attempts.\n\
                     - Each guess must be a valid 5-letter English word in the dictionary.\n\
                     - Green tile  = Correct letter in correct spot.\n\
                     - Yellow tile = Letter in secret word, but wrong spot.\n\
                     - Gray tile   = Letter not in secret word.\n\
                     - Commands:\n\
                       :hint           Get top suggestions ranked by Shannon entropy\n\
                       :hint minimax   Get top suggestions ranked by worst-case remaining\n\
                       :candidates     Show count and list of surviving possible answers\n\
                       :stats          View persistent player statistics and history\n\
                       :quit           Surrender active round\n\
                       :help           Show this rules reference"
                )
                .unwrap();
                continue;
            }

            match game.submit_guess(trimmed, dictionary.trie()) {
                Ok(GuessOutcome::Accepted {
                    feedback,
                    attempts_remaining,
                    status,
                }) => {
                    // Update keyboard with last guess
                    let last_guess = &game.guesses().last().unwrap().word;
                    keyboard.update_from_guess(last_guess, &feedback);

                    // Filter candidate set
                    let summary = candidates.filter(last_guess, &feedback);

                    writeln!(out, "\n{}", render_board(&game, &config)).unwrap();
                    writeln!(out, "{}", render_keyboard(&keyboard, &config)).unwrap();
                    writeln!(
                        out,
                        "{cyan}Candidates: {} remaining (pruned {}){reset}",
                        candidates.len(),
                        summary.eliminated_count,
                        cyan = colors::FG_CYAN,
                        reset = colors::RESET
                    )
                    .unwrap();

                    if status != GameStatus::InProgress {
                        let outcome = if status == GameStatus::Won {
                            GameOutcome::Won
                        } else {
                            GameOutcome::Lost
                        };

                        if let Err(e) = record_terminal_game(
                            &mut stats_manager,
                            &game_id,
                            outcome,
                            game.attempts_used() as u8,
                            game.answer(),
                            started_at,
                        ) {
                            writeln!(
                                out,
                                "{red}[Statistics Save Error: {e}]{reset}",
                                red = colors::FG_RED,
                                reset = colors::RESET
                            )
                            .unwrap();
                        }

                        writeln!(out, "{}", render_game_over(&game, &config)).unwrap();
                        writeln!(out, "\n{}", stats_manager.render_summary()).unwrap();
                        break;
                    } else {
                        writeln!(out, "Attempts remaining: {attempts_remaining}").unwrap();
                    }
                }
                Ok(GuessOutcome::NotInDictionary) => {
                    writeln!(
                        out,
                        "{yellow}Word '{trimmed}' is not in the accepted dictionary. (Does not consume attempt){reset}",
                        yellow = colors::FG_YELLOW,
                        reset = colors::RESET
                    )
                    .unwrap();
                }
                Ok(GuessOutcome::AlreadyGuessed) => {
                    writeln!(
                        out,
                        "{yellow}You have already guessed '{trimmed}'! (Does not consume attempt){reset}",
                        yellow = colors::FG_YELLOW,
                        reset = colors::RESET
                    )
                    .unwrap();
                }
                Err(GameError::InvalidWord(err)) => {
                    writeln!(
                        out,
                        "{red}Invalid input: {err}. (Does not consume attempt){reset}",
                        red = colors::FG_RED,
                        reset = colors::RESET
                    )
                    .unwrap();
                }
                Err(GameError::GameOver(status)) => {
                    writeln!(out, "Round is already finished ({status}).").unwrap();
                    break;
                }
            }
        }

        // Replay prompt
        write!(out, "\nWould you like to play another round? (y/n): ").unwrap();
        out.flush().unwrap();

        let mut replay_line = String::new();
        if reader.read_line(&mut replay_line).unwrap_or(0) == 0 {
            break;
        }

        let replay_trimmed = replay_line.trim().to_ascii_lowercase();
        if replay_trimmed != "y" && replay_trimmed != "yes" {
            writeln!(out, "Thanks for playing Wordle! Goodbye!").unwrap();
            break;
        }
        writeln!(out, "\nStarting a new round with a fresh secret word!\n").unwrap();
    }
}

/// Helper to record completed game session atomically into the stats manager.
fn record_terminal_game(
    mgr: &mut StatsManager,
    game_id: &str,
    outcome: GameOutcome,
    attempts: u8,
    answer: &Word,
    started_at: u64,
) -> Result<(), wordle::stats::StatsError> {
    let completed_at = current_timestamp();
    let record = GameRecord::new(
        game_id.to_string(),
        outcome,
        attempts,
        answer.clone(),
        started_at,
        completed_at,
    )
    .map_err(|e| wordle::stats::StatsError::InvalidRecord(e.to_string()))?;

    mgr.record_completed_game(record)
}

/// Generates a unique pseudo-random game identifier.
fn generate_game_id() -> String {
    use std::time::SystemTime;
    let nanos = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let rand_val: u32 = rand::random();
    format!("game-{nanos:x}-{rand_val:08x}")
}

/// Retrieves the current system timestamp in seconds since Unix epoch.
fn current_timestamp() -> u64 {
    use std::time::SystemTime;
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}
