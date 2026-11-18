//! # Boggle CLI Application
//!
//! Interactive terminal game for Boggle. Supports 4x4, 5x5, and 6x6 board sizes,
//! flexible guess entry (one per line, comma/space-separated), batch evaluation upon DONE,
//! optional automatic board solving, and optional timed rounds.

use std::fs::File;
use std::io::{self, BufRead, BufReader, Write};
use std::path::Path;
use std::time::{Duration, Instant};
use trie::Trie;

use boggle::board::Board;
use boggle::game::{BoggleGame, BoggleGuessResult};
use boggle::validator::solve_boggle_board;
use countdown_timer::display::{emit_sound, render_dashboard, render_expired_banner};
use countdown_timer::timer::{parse_duration, CountdownTimer, TimerConfig};

/// Loads dictionary words from a file into a shared Trie.
pub fn load_dictionary<P: AsRef<Path>>(path: P) -> io::Result<(Trie, usize)> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut trie = Trie::new();
    let mut word_count = 0;

    for line in reader.lines() {
        let line = line?;
        let trimmed = line.trim();
        if !trimmed.is_empty() {
            if let Ok(true) = trie.insert(trimmed) {
                word_count += 1;
            }
        }
    }

    Ok((trie, word_count))
}

fn read_line_trimmed<R: BufRead>(reader: &mut R) -> io::Result<Option<String>> {
    let mut buf = String::new();
    if reader.read_line(&mut buf)? == 0 {
        return Ok(None);
    }
    Ok(Some(buf.trim().to_string()))
}

fn main() -> io::Result<()> {
    let stdout = io::stdout();
    let mut writer = stdout.lock();
    let stdin = io::stdin();
    let mut reader = stdin.lock();

    writeln!(
        writer,
        "========================================================"
    )?;
    writeln!(
        writer,
        "                 WELCOME TO BOGGLE (RUST)               "
    )?;
    writeln!(
        writer,
        "========================================================"
    )?;

    // Locate dictionary
    let dict_candidates = [
        "Resources/dictionary.txt",
        "../Resources/dictionary.txt",
        "dictionary.txt",
        "../dictionary.txt",
        "../../dictionary.txt",
    ];
    let dict_path = dict_candidates
        .iter()
        .find(|p| Path::new(p).exists())
        .copied()
        .unwrap_or("dictionary.txt");

    write!(writer, "Loading dictionary from '{}'... ", dict_path)?;
    writer.flush()?;
    let start_load = Instant::now();
    let (trie, word_count) = match load_dictionary(dict_path) {
        Ok(res) => res,
        Err(err) => {
            writeln!(writer, "\nFailed to open dictionary: {err}")?;
            return Err(err);
        }
    };
    writeln!(
        writer,
        "loaded {} words in {:.2?}.",
        word_count,
        start_load.elapsed()
    )?;

    let mut rng = rand::thread_rng();

    // Outer Game Loop: multiple rounds
    'outer: loop {
        writeln!(writer, "\n--- NEW BOGGLE ROUND ---")?;
        write!(
            writer,
            "Select board dimension (4, 5, or 6) [default: 4] > "
        )?;
        writer.flush()?;

        let Some(dim_str) = read_line_trimmed(&mut reader)? else {
            break;
        };
        let size: usize = match dim_str.as_str() {
            "5" => 5,
            "6" => 6,
            _ => 4,
        };

        write!(
            writer,
            "Enable countdown timer? [y/N or custom e.g. '3m', '90s'] > "
        )?;
        writer.flush()?;
        let Some(timer_choice) = read_line_trimmed(&mut reader)? else {
            break;
        };

        let mut round_timer = if timer_choice.eq_ignore_ascii_case("y")
            || timer_choice.eq_ignore_ascii_case("yes")
        {
            TimerConfig::new(Duration::from_secs(180)).ok().map(CountdownTimer::new)
        } else if !timer_choice.is_empty()
            && !timer_choice.eq_ignore_ascii_case("n")
            && !timer_choice.eq_ignore_ascii_case("no")
        {
            if let Ok(dur) = parse_duration(&timer_choice) {
                TimerConfig::new(dur).ok().map(CountdownTimer::new)
            } else {
                None
            }
        } else {
            None
        };

        let board = Board::random(size, &mut rng).expect("board generation must succeed");
        let mut game = BoggleGame::new(board);

        writeln!(writer, "\nBoard ({}x{}):", game.board.size, game.board.size)?;
        write!(writer, "{}", game.board.display_grid())?;

        writeln!(writer, "Instructions:")?;
        writeln!(
            writer,
            "  - Words must be >= 3 letters formed by adjacent cells (horizontal, vertical, diagonal)."
        )?;
        writeln!(writer, "  - No cell may be reused within a single word.")?;
        writeln!(
            writer,
            "  - Type words one per line, or comma/space-separated."
        )?;
        writeln!(
            writer,
            "  - Type 'TIME' to show large countdown clock and progress."
        )?;
        writeln!(
            writer,
            "  - Type 'PAUSE' to freeze the countdown timer."
        )?;
        writeln!(
            writer,
            "  - Type 'DONE' to finish and evaluate all guesses."
        )?;
        writeln!(
            writer,
            "  - Type 'QUIT' to evaluate current guesses and exit game."
        )?;
        if let Some(timer) = round_timer.as_ref() {
            writeln!(
                writer,
                "  - High-Precision Monotonic Timer: {} active!",
                countdown_timer::display::format_duration_digital(timer.initial_duration(), false)
            )?;
        }
        writeln!(
            writer,
            "--------------------------------------------------------"
        )?;

        if let Some(timer) = round_timer.as_mut() {
            let _ = timer.start();
        }

        // Inner Guess Collection Loop
        loop {
            // Check timer
            if let Some(timer) = round_timer.as_mut() {
                if timer.remaining().is_zero() {
                    emit_sound();
                    writeln!(writer, "{}", render_expired_banner(timer.initial_duration()))?;
                    break;
                }
            }

            let time_indicator = if let Some(timer) = round_timer.as_mut() {
                format!(" | Time: {}s", timer.remaining().as_secs())
            } else {
                String::new()
            };

            write!(
                writer,
                "[Pending: {}{}] Enter word(s) > ",
                game.collected_guesses.len(),
                time_indicator
            )?;
            writer.flush()?;

            let mut line = String::new();
            if reader.read_line(&mut line)? == 0 {
                // EOF
                break;
            }

            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }

            if trimmed.eq_ignore_ascii_case("done") {
                writeln!(writer, "\nFinished collecting guesses. Evaluating batch...")?;
                break;
            } else if trimmed.eq_ignore_ascii_case("quit") {
                writeln!(writer, "\nQuitting round. Evaluating collected guesses...")?;
                let results = game.evaluate_batch(&trie);
                print_round_summary(&mut writer, &game, &results)?;
                break 'outer;
            } else if trimmed.eq_ignore_ascii_case("board") {
                write!(writer, "{}", game.board.display_grid())?;
                continue;
            } else if trimmed.eq_ignore_ascii_case("time") || trimmed.eq_ignore_ascii_case("timer") {
                if let Some(timer) = round_timer.as_mut() {
                    write!(writer, "{}", render_dashboard(&timer.snapshot(), false))?;
                } else {
                    writeln!(writer, "No timer active for this round.")?;
                }
                continue;
            } else if trimmed.eq_ignore_ascii_case("pause") {
                if let Some(timer) = round_timer.as_mut() {
                    let _ = timer.pause();
                    writeln!(writer, "\n[Round Paused] Time frozen. Type 'RESUME' to continue > ")?;
                    writer.flush()?;
                    while let Some(res_line) = read_line_trimmed(&mut reader)? {
                        if res_line.eq_ignore_ascii_case("resume") || res_line.is_empty() {
                            let _ = timer.resume();
                            writeln!(writer, "[Round Resumed!]")?;
                            break;
                        }
                    }
                }
                continue;
            }

            if let Some(timer) = round_timer.as_mut() {
                if timer.remaining().is_zero() {
                    emit_sound();
                    writeln!(writer, "{}", render_expired_banner(timer.initial_duration()))?;
                    break;
                }
            }
            game.add_input_line(trimmed);
        }

        // Batch Evaluation
        let results = game.evaluate_batch(&trie);
        print_round_summary(&mut writer, &game, &results)?;

        // Automatic Solver Showcase (Phase B-11)
        write!(
            writer,
            "\nComputing all possible words with automatic solver... "
        )?;
        writer.flush()?;
        let start_solve = Instant::now();
        let all_solutions = solve_boggle_board(&trie, &game.board);
        let solve_time = start_solve.elapsed();
        writeln!(
            writer,
            "found {} valid words in {:.2?}.",
            all_solutions.len(),
            solve_time
        )?;

        let missed: Vec<String> = all_solutions
            .difference(&game.accepted_words)
            .cloned()
            .collect();
        let mut sorted_missed = missed;
        sorted_missed.sort();

        writeln!(
            writer,
            "Sample of undiscovered words (first 20 of {}):",
            sorted_missed.len()
        )?;
        let sample_limit = sorted_missed.len().min(20);
        for chunk in sorted_missed[..sample_limit].chunks(5) {
            writeln!(writer, "  {}", chunk.join(", "))?;
        }

        // Prompt for another round
        write!(writer, "\nPlay another round? (Y/n) > ")?;
        writer.flush()?;
        let Some(again) = read_line_trimmed(&mut reader)? else {
            break;
        };
        if again.eq_ignore_ascii_case("n") || again.eq_ignore_ascii_case("no") {
            writeln!(writer, "Thanks for playing Boggle!")?;
            break;
        }
    }

    Ok(())
}

fn print_round_summary<W: Write>(
    writer: &mut W,
    game: &BoggleGame,
    results: &[boggle::game::EvaluatedGuess],
) -> io::Result<()> {
    writeln!(
        writer,
        "\n========================================================"
    )?;
    writeln!(
        writer,
        "                     ROUND RESULTS                      "
    )?;
    writeln!(
        writer,
        "========================================================"
    )?;

    let mut accepted_count = 0;
    let mut rejected_count = 0;

    for eval in results {
        match &eval.result {
            BoggleGuessResult::Accepted { points, word } => {
                accepted_count += 1;
                writeln!(writer, "  [ACCEPTED] {:<15} +{} pts", word, points)?;
            }
            BoggleGuessResult::TooShort => {
                rejected_count += 1;
                writeln!(
                    writer,
                    "  [REJECTED] {:<15} Too short (minimum 3 letters)",
                    eval.raw_token
                )?;
            }
            BoggleGuessResult::InvalidCharacters(c) => {
                rejected_count += 1;
                writeln!(
                    writer,
                    "  [REJECTED] {:<15} Invalid character '{c}'",
                    eval.raw_token
                )?;
            }
            BoggleGuessResult::NotInDictionary => {
                rejected_count += 1;
                writeln!(
                    writer,
                    "  [REJECTED] {:<15} Not in dictionary",
                    eval.raw_token
                )?;
            }
            BoggleGuessResult::NotOnBoard => {
                rejected_count += 1;
                writeln!(
                    writer,
                    "  [REJECTED] {:<15} Cannot be formed on board",
                    eval.raw_token
                )?;
            }
            BoggleGuessResult::AlreadyGuessed => {
                rejected_count += 1;
                writeln!(
                    writer,
                    "  [REJECTED] {:<15} Already scored (duplicate)",
                    eval.raw_token
                )?;
            }
        }
    }

    writeln!(
        writer,
        "--------------------------------------------------------"
    )?;
    writeln!(writer, "  Accepted Words : {}", accepted_count)?;
    writeln!(writer, "  Rejected Words : {}", rejected_count)?;
    writeln!(writer, "  Total Score    : {} points", game.score)?;
    writeln!(
        writer,
        "========================================================"
    )?;
    Ok(())
}

#[cfg(test)]
mod input_tests {
    use super::*;

    #[test]
    fn empty_input_and_eof_are_distinct() {
        let mut input = io::Cursor::new("\n  5  \n");
        assert_eq!(read_line_trimmed(&mut input).unwrap(), Some(String::new()));
        assert_eq!(read_line_trimmed(&mut input).unwrap(), Some("5".into()));
        assert_eq!(read_line_trimmed(&mut input).unwrap(), None);
    }
}
