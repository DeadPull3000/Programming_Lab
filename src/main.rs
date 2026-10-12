use rand::RngExt;
use std::io::{self, Write};

// Master Mind / Bulls and Cows
//
// The computer generates a secret sequence of M numbers from 1..=N.
// The player repeatedly enters guesses.
// A Bull is a correct number in the correct position.
// A Cow is a correct number in a different position.
// Numbers may repeat, so matched positions are tracked explicitly.

const M: usize = 4;
const N: u32 = 6;

fn score_guess(target: &[u32], guess: &[u32]) -> (usize, usize) {
    //both sequences must have the same length.
    assert_eq!(
        target.len(),
        guess.len(),
        "Target and guess sequences must have the same length."
    );

    //Record which positions have already been matched.
    let mut target_used = vec![false; target.len()];
    let mut guess_used = vec![false; guess.len()];

    let mut bulls = 0;
    let mut cows = 0;

    // First pass: count exact matches (bulls)
    for i in 0..target.len() {
        if target[i] == guess[i] {
            bulls += 1;
            target_used[i] = true;
            guess_used[i] = true;
        }
    }

    // Second pass: Match remaining numbers in different positions (cows)
    for i in 0..guess.len() {
        if guess_used[i] {
            continue;
        }
        for j in 0..target.len() {
            if !target_used[j] && guess[i] == target[j] {
                cows += 1;

                //Each number occurence can only be matched once.
                guess_used[i] = true;
                target_used[j] = true;

                break;
            }
        }
    }

    (bulls, cows)
}

fn parse_guess(line: &str) -> Result<Vec<u32>, String> {
    // Split on arbitrary whitespace, including tabs and repeated spaces.
    let tokens: Vec<&str> = line.split_whitespace().collect();

    if tokens.len() != M {
        return Err(format!(
            "Please enter exactly {M} numbers, separated by spaces or tabs."
        ));
    }

    let mut guess = Vec::with_capacity(M);

    for token in tokens {
        // Convert each token from text into an unsigned integer.
        let value = token
            .parse::<u32>()
            .map_err(|_| format!("'{token}' is not a valid whole number."))?;

        // Enforce the allowed range.
        if !(1..=N).contains(&value) {
            return Err(format!("Every number must be between 1 and {N}."));
        }

        guess.push(value);
    }

    Ok(guess)
}

fn generate_target() -> Vec<u32> {
    let mut rng = rand::rng();

    (0..M).map(|_| rng.random_range(1..=N)).collect()
}

fn play_game(target: &[u32]) -> bool {
    let mut attempts = 0;

    loop {
        print!(
            "\nEnter {M} numbers from 1 to {N}, separated by spaces or tabs \
             (or type 'quit'): "
        );

        if let Err(error) = io::stdout().flush() {
            eprintln!("Warning: could not flush the prompt: {error}");
        }

        let mut line = String::new();

        // Read the next line from the terminal.
        match io::stdin().read_line(&mut line) {
            Ok(0) => {
                println!("\nInput has closed. Leaving the game.");
                return false;
            }
            Ok(_) => {}
            Err(error) => {
                eprintln!("Could not read input: {error}");
                return false;
            }
        }

        // Handle the quit command before parsing numbers.
        if line.trim().eq_ignore_ascii_case("quit") {
            println!("You chose to quit the current game.");
            return false;
        }

        // Reject invalid input without consuming a game attempt.
        let guess = match parse_guess(&line) {
            Ok(guess) => guess,
            Err(message) => {
                println!("Invalid guess: {message}");
                continue;
            }
        };

        // Only valid guesses count as attempts.
        attempts += 1;

        let (bulls, cows) = score_guess(target, &guess);

        println!("Result: {bulls} Bull(s), {cows} Cow(s).");

        if bulls == M {
            println!("Correct! You solved the code in {attempts} valid guess(es).");

            return true;
        }
    }
}

fn ask_play_again() -> bool {
    loop {
        print!("\nWould you like to play again? (y/n): ");

        if let Err(error) = io::stdout().flush() {
            eprintln!("Could not flush the prompt: {error}");
            return false;
        }

        let mut answer = String::new();

        match io::stdin().read_line(&mut answer) {
            Ok(0) => return false,
            Ok(_) => {}
            Err(error) => {
                eprintln!("Could not read your response: {error}");
                return false;
            }
        }

        let choice = answer.trim().to_ascii_lowercase();

        match choice.as_str() {
            "y" | "yes" => return true,
            "n" | "no" | "quit" | "exit" => return false,
            _ => println!("Please enter y or n."),
        }
    }
}

fn main() {
    println!("====================================");
    println!("       MASTER MIND: BULLS & COWS");
    println!("====================================");

    println!("I'm thinking of {M} numbers, each between 1 and {N}.");
    println!("Numbers may repeat.");
    println!("A Bull means the number and position are correct.");
    println!("A Cow means the number is correct but the position is not.");
    println!("Type 'quit' during a guess to exit.");

    loop {
        // Every new game receives a fresh secret sequence.
        let target = generate_target();

        println!("\nA new secret code has been selected.");

        // Stop if the user quits or the input stream closes.
        let won = play_game(&target);

        if !won {
            break;
        }

        // After a win, start another game only if requested.
        if !ask_play_again() {
            break;
        }
    }

    println!("\nThanks for playing!");
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::RngExt;

    // An independent scoring method used to check the main algorithm.
    // It counts how many occurrences each number has in both sequences.
    fn reference_score(target: &[u32], guess: &[u32]) -> (usize, usize) {
        let mut bulls = 0;

        for i in 0..target.len() {
            if target[i] == guess[i] {
                bulls += 1;
            }
        }

        let mut target_counts = vec![0usize; (N + 1) as usize];
        let mut guess_counts = vec![0usize; (N + 1) as usize];

        for &value in target {
            target_counts[value as usize] += 1;
        }

        for &value in guess {
            guess_counts[value as usize] += 1;
        }

        // Count all matching occurrences, ignoring position.
        let mut total_matches = 0;

        for value in 1..=N {
            total_matches += target_counts[value as usize].min(guess_counts[value as usize]);
        }

        // Exact matches are already included in total_matches.
        let cows = total_matches - bulls;

        (bulls, cows)
    }

    #[test]
    fn scores_basic_cases_correctly() {
        assert_eq!(score_guess(&[1, 2, 3, 4], &[1, 2, 3, 4]), (4, 0));

        assert_eq!(score_guess(&[1, 2, 3, 4], &[4, 3, 2, 1]), (0, 4));

        assert_eq!(score_guess(&[1, 2, 3, 4], &[1, 2, 4, 3]), (2, 2));

        assert_eq!(score_guess(&[1, 2, 3, 4], &[5, 5, 6, 6]), (0, 0));
    }

    #[test]
    fn scores_repeated_numbers_correctly() {
        assert_eq!(score_guess(&[1, 1, 2, 2], &[1, 2, 1, 1]), (1, 2));

        assert_eq!(score_guess(&[1, 1, 1, 1], &[1, 1, 2, 2]), (2, 0));

        assert_eq!(score_guess(&[1, 1, 2, 2], &[2, 2, 1, 1]), (0, 4));
    }

    #[test]
    fn randomized_results_match_reference_implementation() {
        let mut rng = rand::rng();

        for _ in 0..10_000 {
            let target: Vec<u32> = (0..M).map(|_| rng.random_range(1..=N)).collect();

            let guess: Vec<u32> = (0..M).map(|_| rng.random_range(1..=N)).collect();

            assert_eq!(
                score_guess(&target, &guess),
                reference_score(&target, &guess),
                "Mismatch for target {:?} and guess {:?}",
                target,
                guess
            );
        }
    }

    #[test]
    fn parser_accepts_whitespace_and_repetitions() {
        assert_eq!(parse_guess("1\t2  2\t4").unwrap(), vec![1, 2, 2, 4]);
    }

    #[test]
    fn parser_rejects_invalid_guesses() {
        let invalid_inputs = [
            "",
            "1 2 3",
            "1 2 3 4 5",
            "1 hello 3 4",
            "0 2 3 4",
            "1 2 3 7",
        ];

        for input in invalid_inputs {
            assert!(
                parse_guess(input).is_err(),
                "Input {:?} should have been rejected.",
                input
            );
        }
    }

    #[test]
    fn generated_targets_have_correct_length_and_range() {
        for _ in 0..1_000 {
            let target = generate_target();

            assert_eq!(target.len(), M);

            assert!(target.iter().all(|&value| (1..=N).contains(&value)));
        }
    }
}
