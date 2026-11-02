//! # Wordle CLI Entry Point
//!
//! Interactive command-line Wordle application.
//!
//! # Execution Lifecycle
//! 1. Ingests dictionary and answer pools via [`WordleDictionary::load_default`].
//! 2. Selects a secret 5-letter answer using a cryptographically secure / system RNG.
//! 3. Maintains interactive input loop, rejecting invalid input non-punitively.
//! 4. Renders board tiles and QWERTY keyboard with ANSI colors.
//! 5. Supports replay across consecutive rounds without reloading the lexicon.

use std::io::{self, BufRead, Write};
use wordle::dictionary::WordleDictionary;
use wordle::display::{
    colors, render_board, render_game_over, render_keyboard, render_welcome_banner, DisplayConfig,
};
use wordle::game::{GameError, GameStatus, GuessOutcome, WordleGame};
use wordle::keyboard::KeyboardState;
use wordle::stats::PlayerStats;

fn main() {
    let stdout = io::stdout();
    let mut out = stdout.lock();
    let stdin = io::stdin();
    let mut reader = stdin.lock();

    let config = DisplayConfig::default();

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
            "Loaded {dict_words} dictionary words and {ans_words} curated secret answers.",
            dict_words = dictionary.trie().len(),
            ans_words = dictionary.answer_count()
        )
        .unwrap();
    }

    let mut rng = rand::thread_rng();
    let mut stats = PlayerStats::new();

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

        writeln!(out, "{}", render_welcome_banner(&config)).unwrap();
        writeln!(out, "{}", render_board(&game, &config)).unwrap();
        writeln!(out, "{}", render_keyboard(&keyboard, &config)).unwrap();

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
                // EOF reached
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
                break;
            }

            if trimmed.eq_ignore_ascii_case("STATS") || trimmed.eq_ignore_ascii_case(":stats") {
                writeln!(out, "\n{}", stats.render_summary()).unwrap();
                continue;
            }

            if trimmed.eq_ignore_ascii_case("HELP") || trimmed.eq_ignore_ascii_case(":help") {
                writeln!(
                    out,
                    "\n--- WORDLE RULES ---\n\
                     - Guess the hidden 5-letter word in 6 attempts.\n\
                     - Each guess must be a valid 5-letter English word in the dictionary.\n\
                     - Green tile  = Correct letter in correct spot.\n\
                     - Yellow tile = Letter in secret word, but wrong spot.\n\
                     - Gray tile   = Letter not in secret word.\n\
                     - Duplicate letters in guesses are handled accurately.\n\
                     - Special commands: 'STATS' (view record), 'QUIT' (surrender), 'HELP' (rules)."
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

                    writeln!(out, "\n{}", render_board(&game, &config)).unwrap();
                    writeln!(out, "{}", render_keyboard(&keyboard, &config)).unwrap();

                    if status != GameStatus::InProgress {
                        stats.record_game(status, game.attempts_used());
                        writeln!(out, "{}", render_game_over(&game, &config)).unwrap();
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
