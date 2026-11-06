//! # Spelling Bee CLI Application
//!
//! Interactive terminal game for Spelling Bee. Uses the shared `trie` library
//! to precompute solutions and evaluate guesses against standard Spelling Bee rules.

use std::fmt::Write as FmtWrite;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Write};
use std::path::Path;
use std::time::Instant;
use trie::Trie;

use spelling_bee::game::{GuessResult, SpellingBeeGame};
use spelling_bee::hints::SpellingBeeHints;
use spelling_bee::puzzle::SpellingBeePuzzle;
use spelling_bee::solver::solve_spelling_bee;

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

/// Formats a list of words in alphabetical order across three columns.
pub fn format_three_columns(words: &[String]) -> String {
    if words.is_empty() {
        return "  (No words discovered yet)\n".to_string();
    }

    let mut out = String::new();
    let col_width = 22;
    let n = words.len();
    let rows = n.div_ceil(3);

    for r in 0..rows {
        let w1 = &words[r];
        let w2 = if r + rows < n { &words[r + rows] } else { "" };
        let w3 = if r + 2 * rows < n {
            &words[r + 2 * rows]
        } else {
            ""
        };
        let _ = writeln!(
            out,
            "  {:<width$} {:<width$} {:<width$}",
            w1,
            w2,
            w3,
            width = col_width
        );
    }
    out
}

/// Displays the banner and puzzle details.
pub fn display_puzzle<W: Write>(writer: &mut W, puzzle: &SpellingBeePuzzle) -> io::Result<()> {
    writeln!(
        writer,
        "\n========================================================"
    )?;
    writeln!(
        writer,
        "                 SPELLING BEE PUZZLE                    "
    )?;
    writeln!(
        writer,
        "========================================================"
    )?;
    writeln!(
        writer,
        "  Mandatory Center Letter : [ {} ]",
        puzzle.main_letter
    )?;
    writeln!(
        writer,
        "  Surrounding Letters     : {}  {}  {}  {}  {}  {}",
        puzzle.letters[1],
        puzzle.letters[2],
        puzzle.letters[3],
        puzzle.letters[4],
        puzzle.letters[5],
        puzzle.letters[6]
    )?;
    writeln!(
        writer,
        "--------------------------------------------------------"
    )?;
    writeln!(
        writer,
        "  Rules: Words must be >= 4 letters, include [ {} ], and use",
        puzzle.main_letter
    )?;
    writeln!(
        writer,
        "         only the 7 puzzle letters (repetition allowed)."
    )?;
    writeln!(
        writer,
        "  Special Commands: HINT (view grid), WORDS (guessed), QUIT (exit)"
    )?;
    writeln!(
        writer,
        "========================================================\n"
    )?;
    Ok(())
}

/// Displays the current game score and progress.
pub fn display_status<W: Write>(writer: &mut W, game: &SpellingBeeGame) -> io::Result<()> {
    writeln!(
        writer,
        "[Score: {} | Words Found: {} / {}]",
        game.score,
        game.count_guessed(),
        game.total_solutions()
    )
}

/// Generates a viable puzzle from the dictionary with a reasonable minimum number of solutions.
pub fn generate_viable_puzzle<R: rand::Rng>(
    trie: &Trie,
    rng: &mut R,
    min_solutions: usize,
) -> (SpellingBeePuzzle, std::collections::HashSet<String>) {
    loop {
        let puzzle = SpellingBeePuzzle::random(rng);
        let solutions = solve_spelling_bee(trie, &puzzle);
        if solutions.len() >= min_solutions {
            return (puzzle, solutions);
        }
    }
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
        "             WELCOME TO SPELLING BEE (RUST)             "
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

    // Generate puzzle
    write!(writer, "Generating puzzle... ")?;
    writer.flush()?;
    let mut rng = rand::thread_rng();
    let start_gen = Instant::now();
    let (puzzle, solutions) = generate_viable_puzzle(&trie, &mut rng, 15);
    writeln!(
        writer,
        "ready in {:.2?} ({} possible solutions).",
        start_gen.elapsed(),
        solutions.len()
    )?;

    let mut game = SpellingBeeGame::new(puzzle, solutions);
    display_puzzle(&mut writer, &game.puzzle)?;

    let mut line_buf = String::new();
    loop {
        display_status(&mut writer, &game)?;
        write!(writer, "Enter guess or command > ")?;
        writer.flush()?;

        line_buf.clear();
        if reader.read_line(&mut line_buf)? == 0 {
            // EOF reached
            break;
        }

        let input = line_buf.trim();
        if input.is_empty() {
            continue;
        }

        // Handle commands
        if input.eq_ignore_ascii_case("quit") || input.eq_ignore_ascii_case("exit") {
            writeln!(writer, "\nQuitting current game session.")?;
            break;
        } else if input.eq_ignore_ascii_case("words") || input.eq_ignore_ascii_case("list") {
            let guessed_sorted = game.sorted_guessed();
            writeln!(
                writer,
                "\n--- Discovered Words ({}) ---",
                guessed_sorted.len()
            )?;
            write!(writer, "{}", format_three_columns(&guessed_sorted))?;
            writeln!(writer)?;
            continue;
        } else if input.eq_ignore_ascii_case("hint") || input.eq_ignore_ascii_case("hints") {
            let hints = SpellingBeeHints::compute(&game.solutions);
            writeln!(writer, "\n--- SPELLING BEE HINT GRID ---")?;
            write!(writer, "{}", hints.grid.format_table())?;
            writeln!(writer, "--- TWO-LETTER PREFIXES ---")?;
            write!(writer, "{}", hints.prefixes.format_summary())?;
            writeln!(writer)?;
            continue;
        } else if input.eq_ignore_ascii_case("puzzle") {
            display_puzzle(&mut writer, &game.puzzle)?;
            continue;
        }

        // Process word guess
        match game.submit_guess(input) {
            GuessResult::Accepted {
                points,
                pangram,
                word,
            } => {
                if pangram {
                    writeln!(
                        writer,
                        "  >>> PANGRAM! +{} points! Awesome word '{}'!",
                        points, word
                    )?;
                } else {
                    writeln!(writer, "  Correct! +{} points for '{}'.", points, word)?;
                }

                if game.is_queen_bee() {
                    writeln!(
                        writer,
                        "\n********************************************************"
                    )?;
                    writeln!(
                        writer,
                        "  QUEEN BEE! You have discovered ALL {} words!",
                        game.total_solutions()
                    )?;
                    writeln!(writer, "  Final Outstanding Score: {}", game.score)?;
                    writeln!(
                        writer,
                        "********************************************************\n"
                    )?;
                    return Ok(());
                }
            }
            GuessResult::TooShort => {
                writeln!(writer, "  Error: Words must be at least 4 letters.")?;
            }
            GuessResult::InvalidCharacters(c) => {
                writeln!(
                    writer,
                    "  Error: Invalid character '{c}'. Only letters A-Z are allowed."
                )?;
            }
            GuessResult::MissingMainLetter(main) => {
                writeln!(
                    writer,
                    "  Error: Guess must include the mandatory letter [{main}]."
                )?;
            }
            GuessResult::InvalidLetters(c) => {
                writeln!(
                    writer,
                    "  Error: Letter '{c}' is not among the 7 puzzle letters."
                )?;
            }
            GuessResult::NotInSolutions => {
                writeln!(writer, "  Not in word list / dictionary.")?;
            }
            GuessResult::AlreadyGuessed => {
                writeln!(writer, "  Already guessed!")?;
            }
        }
        writeln!(writer)?;
    }

    // Final summary upon quitting
    writeln!(
        writer,
        "\n========================================================"
    )?;
    writeln!(
        writer,
        "                     GAME SUMMARY                       "
    )?;
    writeln!(
        writer,
        "========================================================"
    )?;
    writeln!(writer, "  Final Score     : {}", game.score)?;
    writeln!(
        writer,
        "  Words Found     : {} / {}",
        game.count_guessed(),
        game.total_solutions()
    )?;

    let missed = game.remaining_solutions();
    writeln!(writer, "\n  Missed Words ({} remaining):", missed.len())?;
    write!(writer, "{}", format_three_columns(&missed))?;
    writeln!(
        writer,
        "========================================================\n"
    )?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_three_columns_empty_and_populated() {
        assert_eq!(format_three_columns(&[]), "  (No words discovered yet)\n");

        let words = vec![
            "APPLE".to_string(),
            "BANANA".to_string(),
            "CHERRY".to_string(),
            "DATE".to_string(),
            "ELDERBERRY".to_string(),
        ];
        let table = format_three_columns(&words);
        assert!(table.contains("APPLE"));
        assert!(table.contains("BANANA"));
        assert!(table.contains("CHERRY"));
        assert!(table.contains("DATE"));
        assert!(table.contains("ELDERBERRY"));
    }
}
