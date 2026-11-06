//! # Trie CLI
//!
//! Interactive command-line interface for the Word Trie dictionary.
//!
//! Provides interactive commands to search, insert, delete, and classify words,
//! as well as load and check candidate files.

pub mod batch;

use std::env;
use std::io::{self, Write};
use std::path::Path;
use std::time::Instant;
use trie::{normalize_word, Trie};

use crate::batch::{check_words_from_file, load_words_from_file, parse_tokens};

/// Processes a single CLI command string against the given Trie.
///
/// Writes output to `writer`. Returns `Ok(false)` when the `quit` command
/// is received to signal termination of the REPL.
pub fn process_command<W: Write>(trie: &mut Trie, line: &str, writer: &mut W) -> io::Result<bool> {
    let trimmed = line.trim();
    if trimmed.is_empty() {
        return Ok(true);
    }

    // Split the command keyword from the rest of the arguments
    let mut parts = trimmed.splitn(2, |c: char| c.is_whitespace());
    let cmd = parts.next().unwrap_or("").to_ascii_lowercase();
    let rest = parts.next().unwrap_or("").trim();

    match cmd.as_str() {
        "help" => {
            writeln!(writer, "Available commands:")?;
            writeln!(
                writer,
                "  find WORD[, WORD ...]     Check if words exist in dictionary"
            )?;
            writeln!(
                writer,
                "  insert WORD[, WORD ...]   Add words to dictionary"
            )?;
            writeln!(
                writer,
                "  delete WORD[, WORD ...]   Remove words from dictionary"
            )?;
            writeln!(
                writer,
                "  found WORD[, WORD ...]    List words from input that are in dictionary"
            )?;
            writeln!(
                writer,
                "  missing WORD[, WORD ...]  List words from input that are NOT in dictionary"
            )?;
            writeln!(
                writer,
                "  load PATH                 Load words from a file into dictionary"
            )?;
            writeln!(
                writer,
                "  check-file PATH           Classify words from candidate file (found & missing)"
            )?;
            writeln!(
                writer,
                "  stats                     Display dictionary statistics"
            )?;
            writeln!(
                writer,
                "  help                      Display this help guide"
            )?;
            writeln!(writer, "  quit / exit               Exit the program")?;
        }
        "find" => {
            let tokens = parse_tokens(rest);
            if tokens.is_empty() {
                writeln!(writer, "Error: 'find' requires at least one word argument.")?;
            } else {
                for token in tokens {
                    match normalize_word(token) {
                        Ok(norm) => {
                            let found = trie.contains(&norm);
                            writeln!(
                                writer,
                                "  {norm}: {}",
                                if found { "FOUND" } else { "NOT FOUND" }
                            )?;
                        }
                        Err(e) => {
                            writeln!(writer, "  {token}: INVALID ({e})")?;
                        }
                    }
                }
            }
        }
        "insert" => {
            let tokens = parse_tokens(rest);
            if tokens.is_empty() {
                writeln!(
                    writer,
                    "Error: 'insert' requires at least one word argument."
                )?;
            } else {
                for token in tokens {
                    match trie.insert(token) {
                        Ok(true) => writeln!(writer, "  {token}: inserted (new word)")?,
                        Ok(false) => {
                            writeln!(writer, "  {token}: already present (duplicate, no change)")?
                        }
                        Err(e) => writeln!(writer, "  {token}: error - {e}")?,
                    }
                }
            }
        }
        "delete" => {
            let tokens = parse_tokens(rest);
            if tokens.is_empty() {
                writeln!(
                    writer,
                    "Error: 'delete' requires at least one word argument."
                )?;
            } else {
                for token in tokens {
                    match trie.remove(token) {
                        Ok(true) => writeln!(writer, "  {token}: deleted (word removed)")?,
                        Ok(false) => writeln!(writer, "  {token}: not found (no change)")?,
                        Err(e) => writeln!(writer, "  {token}: error - {e}")?,
                    }
                }
            }
        }
        "found" => {
            let tokens = parse_tokens(rest);
            if tokens.is_empty() {
                writeln!(
                    writer,
                    "Error: 'found' requires at least one word argument."
                )?;
            } else {
                let mut found_list = Vec::new();
                for token in tokens {
                    if let Ok(norm) = normalize_word(token) {
                        if trie.contains(&norm) {
                            found_list.push(norm);
                        }
                    }
                }
                writeln!(
                    writer,
                    "Found ({count}): {list}",
                    count = found_list.len(),
                    list = found_list.join(", ")
                )?;
            }
        }
        "missing" => {
            let tokens = parse_tokens(rest);
            if tokens.is_empty() {
                writeln!(
                    writer,
                    "Error: 'missing' requires at least one word argument."
                )?;
            } else {
                let mut missing_list = Vec::new();
                for token in tokens {
                    if let Ok(norm) = normalize_word(token) {
                        if !trie.contains(&norm) {
                            missing_list.push(norm);
                        }
                    } else {
                        missing_list.push(token.to_string());
                    }
                }
                writeln!(
                    writer,
                    "Missing ({count}): {list}",
                    count = missing_list.len(),
                    list = missing_list.join(", ")
                )?;
            }
        }
        "load" => {
            if rest.is_empty() {
                writeln!(writer, "Error: 'load' requires a file path argument.")?;
            } else {
                let path = Path::new(rest);
                let start = Instant::now();
                match load_words_from_file(path, trie) {
                    Ok(rep) => {
                        let duration = start.elapsed();
                        writeln!(
                            writer,
                            "Loaded in {:.2?}: {} new words inserted, {} duplicates, {} errors.",
                            duration,
                            rep.newly_inserted,
                            rep.duplicates,
                            rep.invalid_entries.len()
                        )?;
                        writeln!(writer, "Total distinct words in dictionary: {}", trie.len())?;
                        if !rep.invalid_entries.is_empty() {
                            let preview = rep.invalid_entries.iter().take(5);
                            for (line, tok, err) in preview {
                                writeln!(writer, "  Line {line}: '{tok}' -> {err}")?;
                            }
                            if rep.invalid_entries.len() > 5 {
                                writeln!(
                                    writer,
                                    "  ... and {} more error(s)",
                                    rep.invalid_entries.len() - 5
                                )?;
                            }
                        }
                    }
                    Err(e) => {
                        writeln!(writer, "Error reading file '{rest}': {e}")?;
                    }
                }
            }
        }
        "check-file" => {
            if rest.is_empty() {
                writeln!(writer, "Error: 'check-file' requires a file path argument.")?;
            } else {
                let path = Path::new(rest);
                match check_words_from_file(path, trie) {
                    Ok(res) => {
                        writeln!(writer, "Candidate file classification:")?;
                        writeln!(
                            writer,
                            "  Found ({count}): {words}",
                            count = res.found.len(),
                            words = if res.found.len() <= 20 {
                                res.found.join(", ")
                            } else {
                                format!("{} ...", res.found[..20].join(", "))
                            }
                        )?;
                        writeln!(
                            writer,
                            "  Missing ({count}): {words}",
                            count = res.missing.len(),
                            words = if res.missing.len() <= 20 {
                                res.missing.join(", ")
                            } else {
                                format!("{} ...", res.missing[..20].join(", "))
                            }
                        )?;
                        if !res.invalid.is_empty() {
                            writeln!(
                                writer,
                                "  Invalid tokens ({count}):",
                                count = res.invalid.len()
                            )?;
                            for (line, tok, err) in res.invalid.iter().take(5) {
                                writeln!(writer, "    Line {line}: '{tok}' ({err})")?;
                            }
                        }
                    }
                    Err(e) => {
                        writeln!(writer, "Error reading candidate file '{rest}': {e}")?;
                    }
                }
            }
        }
        "stats" => {
            writeln!(writer, "Dictionary Statistics:")?;
            writeln!(writer, "  Distinct complete words: {}", trie.len())?;
            writeln!(
                writer,
                "  Status: {}",
                if trie.is_empty() {
                    "Empty"
                } else {
                    "Populated"
                }
            )?;
        }
        "quit" | "exit" => {
            writeln!(writer, "Goodbye!")?;
            return Ok(false);
        }
        unknown => {
            writeln!(
                writer,
                "Unknown command: '{unknown}'. Type 'help' to see available commands."
            )?;
        }
    }

    Ok(true)
}

fn main() {
    let mut trie = Trie::new();

    // Default dictionary path or command-line specified path
    let dict_path = env::args().nth(1).unwrap_or_else(|| {
        let candidates = [
            "Resources/dictionary.txt",
            "../Resources/dictionary.txt",
            "../../Resources/dictionary.txt",
            "dictionary.txt",
            "../dictionary.txt",
            "../../dictionary.txt",
        ];
        candidates
            .iter()
            .find(|p| Path::new(p).exists())
            .copied()
            .unwrap_or("dictionary.txt")
            .to_string()
    });

    println!("==================================================");
    println!("             Word Trie Dictionary CLI             ");
    println!("==================================================");

    if Path::new(&dict_path).exists() {
        println!("Loading initial dictionary from '{dict_path}'...");
        let start = Instant::now();
        match load_words_from_file(&dict_path, &mut trie) {
            Ok(report) => {
                let duration = start.elapsed();
                println!(
                    "Successfully loaded {} words in {:.2?} ({} duplicates, {} errors).",
                    report.newly_inserted,
                    duration,
                    report.duplicates,
                    report.invalid_entries.len()
                );
            }
            Err(e) => {
                eprintln!("Warning: Failed to read dictionary '{dict_path}': {e}");
                eprintln!("Starting with an empty dictionary.");
            }
        }
    } else {
        println!("Dictionary file '{dict_path}' not found.");
        println!("Starting with an empty dictionary. Use 'load <PATH>' to load words.");
    }

    println!("\nType 'help' for commands, 'quit' to exit.\n");

    let stdin = io::stdin();
    let mut stdout = io::stdout();

    loop {
        print!("trie> ");
        if stdout.flush().is_err() {
            break;
        }

        let mut line = String::new();
        match stdin.read_line(&mut line) {
            Ok(0) => break, // EOF reached
            Ok(_) => {
                match process_command(&mut trie, &line, &mut stdout) {
                    Ok(false) => break, // User asked to quit
                    Ok(true) => {}
                    Err(e) => {
                        eprintln!("Output error: {e}");
                    }
                }
            }
            Err(e) => {
                eprintln!("Error reading input: {e}");
                break;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process_command_help() {
        let mut trie = Trie::new();
        let mut out = Vec::new();
        assert!(process_command(&mut trie, "help", &mut out).unwrap());
        let text = String::from_utf8(out).unwrap();
        assert!(text.contains("find"));
        assert!(text.contains("insert"));
        assert!(text.contains("delete"));
        assert!(text.contains("stats"));
    }

    #[test]
    fn test_process_command_insert_find_delete() {
        let mut trie = Trie::new();
        let mut out = Vec::new();

        // Insert
        process_command(&mut trie, "insert apple, banana", &mut out).unwrap();
        let text = String::from_utf8(out.clone()).unwrap();
        assert!(text.contains("apple: inserted"));
        assert!(text.contains("banana: inserted"));
        assert_eq!(trie.len(), 2);

        // Duplicate insert
        out.clear();
        process_command(&mut trie, "insert apple", &mut out).unwrap();
        let text = String::from_utf8(out.clone()).unwrap();
        assert!(text.contains("already present"));
        assert_eq!(trie.len(), 2);

        // Find
        out.clear();
        process_command(&mut trie, "find apple, cherry", &mut out).unwrap();
        let text = String::from_utf8(out.clone()).unwrap();
        assert!(text.contains("APPLE: FOUND"));
        assert!(text.contains("CHERRY: NOT FOUND"));

        // Delete
        out.clear();
        process_command(&mut trie, "delete apple", &mut out).unwrap();
        let text = String::from_utf8(out.clone()).unwrap();
        assert!(text.contains("apple: deleted"));
        assert_eq!(trie.len(), 1);

        // Delete nonexistent
        out.clear();
        process_command(&mut trie, "delete apple", &mut out).unwrap();
        let text = String::from_utf8(out.clone()).unwrap();
        assert!(text.contains("apple: not found"));
    }

    #[test]
    fn test_process_command_found_missing() {
        let mut trie = Trie::new();
        trie.insert("CAR").unwrap();
        trie.insert("BUS").unwrap();

        let mut out = Vec::new();
        process_command(&mut trie, "found car, train, bus", &mut out).unwrap();
        let text = String::from_utf8(out.clone()).unwrap();
        assert!(text.contains("Found (2): CAR, BUS"));

        out.clear();
        process_command(&mut trie, "missing car, train, bus, plane", &mut out).unwrap();
        let text = String::from_utf8(out.clone()).unwrap();
        assert!(text.contains("Missing (2): TRAIN, PLANE"));
    }

    #[test]
    fn test_process_command_stats() {
        let mut trie = Trie::new();
        let mut out = Vec::new();
        process_command(&mut trie, "stats", &mut out).unwrap();
        let text = String::from_utf8(out.clone()).unwrap();
        assert!(text.contains("Distinct complete words: 0"));
        assert!(text.contains("Empty"));

        trie.insert("WORD").unwrap();
        out.clear();
        process_command(&mut trie, "stats", &mut out).unwrap();
        let text = String::from_utf8(out.clone()).unwrap();
        assert!(text.contains("Distinct complete words: 1"));
        assert!(text.contains("Populated"));
    }

    #[test]
    fn test_process_command_quit() {
        let mut trie = Trie::new();
        let mut out = Vec::new();
        assert!(!process_command(&mut trie, "quit", &mut out).unwrap());
        assert!(!process_command(&mut trie, "EXIT", &mut out).unwrap());
    }

    #[test]
    fn test_process_command_unknown_and_empty() {
        let mut trie = Trie::new();
        let mut out = Vec::new();
        assert!(process_command(&mut trie, "", &mut out).unwrap());
        assert!(process_command(&mut trie, "   ", &mut out).unwrap());

        process_command(&mut trie, "foobar", &mut out).unwrap();
        let text = String::from_utf8(out.clone()).unwrap();
        assert!(text.contains("Unknown command: 'foobar'"));
    }

    #[test]
    fn test_full_dictionary_loading_and_lookup() {
        use std::collections::HashSet;
        use std::fs::File;
        use std::io::{BufRead, BufReader};
        use std::path::PathBuf;

        let candidates = [
            PathBuf::from("Resources/dictionary.txt"),
            PathBuf::from("../Resources/dictionary.txt"),
            PathBuf::from("../../Resources/dictionary.txt"),
            PathBuf::from("dictionary.txt"),
            PathBuf::from("../dictionary.txt"),
            PathBuf::from("../../dictionary.txt"),
        ];

        let dict_path = candidates.iter().find(|p| p.exists()).cloned();
        let dict_path = match dict_path {
            Some(p) => p,
            None => {
                eprintln!("Skipping full dictionary test: dictionary.txt not found");
                return;
            }
        };

        // 1. Independently compute valid distinct words using HashSet and standard line-reading
        let file = File::open(&dict_path).expect("failed to open dictionary.txt");
        let reader = BufReader::new(file);

        let mut reference_set = HashSet::new();
        let mut total_lines = 0;
        let mut duplicate_count = 0;
        let mut invalid_count = 0;

        for line_res in reader.lines() {
            let line = line_res.expect("failed to read line from dictionary");
            total_lines += 1;
            match normalize_word(&line) {
                Ok(normalized) => {
                    if !reference_set.insert(normalized) {
                        duplicate_count += 1;
                    }
                }
                Err(_) => {
                    invalid_count += 1;
                }
            }
        }

        // 2. Load into Trie and measure time
        let mut trie = Trie::new();
        let start_load = Instant::now();
        let report = load_words_from_file(&dict_path, &mut trie)
            .expect("failed to load words into trie from file");
        let _load_duration = start_load.elapsed();

        // 3. Verify counts match independently calculated figures exactly
        assert_eq!(
            report.total_tokens, total_lines,
            "Total tokens read must match total lines"
        );
        assert_eq!(
            trie.len(),
            reference_set.len(),
            "Trie length must match reference HashSet length"
        );
        assert_eq!(
            report.newly_inserted,
            reference_set.len(),
            "Newly inserted count must match distinct words count"
        );
        assert_eq!(
            report.duplicates, duplicate_count,
            "Duplicate count must match"
        );
        assert_eq!(
            report.invalid_entries.len(),
            invalid_count,
            "Invalid entries count must match"
        );

        // 4. Verify representative words from across the dictionary
        let sample_words = [
            "AA",
            "AAH",
            "AARDVARK",
            "CAT",
            "DOG",
            "ELEPHANT",
            "MIDDLE",
            "PROGRAMMING",
            "RUST",
            "TREE",
            "ZYZZYVA",
            "ZZZ",
        ];

        for word in &sample_words {
            assert!(
                trie.contains(word),
                "Expected word '{word}' must be found in dictionary"
            );
            assert!(
                trie.contains(&word.to_ascii_lowercase()),
                "Lowercase query for '{word}' must be found"
            );
        }

        // 5. Verify non-existent words
        let missing_words = [
            "NOTAWORDSURELY123",
            "FOOBARXYZ",
            "ZYZZYVAZZZ",
            "PROGRAMMINGLAB",
            "CPLUSPLUS",
        ];
        for missing in &missing_words {
            assert!(
                !trie.contains(missing),
                "Missing candidate '{missing}' must not be found"
            );
        }

        // 6. Test prefix behavior on full dictionary
        assert!(trie.has_prefix("PROG"));
        assert!(trie.has_prefix("TRI"));
        assert!(trie.has_prefix("ZZ"));
        assert!(!trie.has_prefix("QQQ"));

        // 7. Smoke test mutation: delete a word and verify invariants
        assert_eq!(trie.remove("AA"), Ok(true));
        assert_eq!(trie.len(), reference_set.len() - 1);
        assert!(!trie.contains("AA"));
        assert!(trie.contains("AAH")); // Neighbor preserved
        assert!(trie.has_prefix("AA")); // Path preserved because "AAH" exists
    }
}
