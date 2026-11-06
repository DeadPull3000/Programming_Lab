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

fn read_line_trimmed<R: BufRead>(reader: &mut R) -> io::Result<String> {
    let mut buf = String::new();
    reader.read_line(&mut buf)?;
    Ok(buf.trim().to_string())
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

        let dim_str = read_line_trimmed(&mut reader)?;
        let size: usize = match dim_str.as_str() {
            "5" => 5,
            "6" => 6,
            _ => 4,
        };

        write!(writer, "Enable 3-minute timer? (y/N) > ")?;
        writer.flush()?;
        let timer_choice = read_line_trimmed(&mut reader)?;
        let timer_duration =
            if timer_choice.eq_ignore_ascii_case("y") || timer_choice.eq_ignore_ascii_case("yes") {
                Some(Duration::from_secs(180))
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
            "  - Type 'DONE' to finish and evaluate all guesses."
        )?;
        writeln!(
            writer,
            "  - Type 'QUIT' to evaluate current guesses and exit game."
        )?;
        if let Some(dur) = timer_duration {
            writeln!(writer, "  - Round Timer: {:.0?} active!", dur)?;
        }
        writeln!(
            writer,
            "--------------------------------------------------------"
        )?;

        let round_start = Instant::now();

        // Inner Guess Collection Loop
        loop {
            // Check timer
            if let Some(dur) = timer_duration {
                let elapsed = round_start.elapsed();
                if elapsed >= dur {
                    writeln!(writer, "\n*** TIME IS UP! ({:.0?}) ***", dur)?;
                    break;
                }
            }

            write!(
                writer,
                "[Guesses pending: {}] Enter word(s) > ",
                game.collected_guesses.len()
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
        let again = read_line_trimmed(&mut reader)?;
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
