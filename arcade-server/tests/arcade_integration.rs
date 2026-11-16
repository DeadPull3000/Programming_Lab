use std::collections::HashSet;
use std::fs;
use std::time::{Duration, Instant};

use arcade_server::handlers::*;
use arcade_server::state::*;
use boggle::{board::Board, game::BoggleGame};
use spelling_bee::{game::SpellingBeeGame, puzzle::SpellingBeePuzzle};
use trie::Trie;
use wordle::dictionary::WordleDictionary;
use wordle::stats::storage::{JsonParser, JsonValue};
use wordle::word::Word;

struct Fixture {
    state: ArcadeState,
    directory: std::path::PathBuf,
}

impl Fixture {
    fn new() -> Self {
        let directory = std::env::temp_dir().join(format!("arcade_test_{}", rand::random::<u64>()));
        fs::create_dir_all(&directory).unwrap();
        let mut trie = Trie::new();
        for word in [
            "APPLE", "CRANE", "SLATE", "BONE", "CONE", "ECONOMY", "CAT", "CATS",
        ] {
            trie.insert(word).unwrap();
        }
        let dictionary = WordleDictionary::new(
            trie,
            vec![Word::parse("APPLE").unwrap(), Word::parse("CRANE").unwrap()],
        )
        .unwrap();
        let state = ArcadeState {
            dictionary,
            wordle: None,
            spelling_bee: None,
            boggle: None,
            stats: UnifiedStats::default(),
            settings: ArcadeSettings::default(),
            stats_path: directory.join("stats.json"),
        };
        Self { state, directory }
    }

    fn boggle(&mut self) {
        let board = Board::new(4, "CATSXXXXXXXXXXXX".chars().collect()).unwrap();
        self.state.boggle = Some(BoggleSession {
            game: BoggleGame::new(board),
            size: 4,
            all_solutions: HashSet::from(["CAT".into(), "CATS".into()]),
            max_possible_score: 2,
            recorded_in_stats: false,
            started_at: Instant::now(),
            ends_at_unix_ms: BoggleSession::deadline_unix_ms(),
            duration_secs: 180,
        });
    }

    fn bee(&mut self) {
        let puzzle = SpellingBeePuzzle::new(['E', 'B', 'C', 'O', 'M', 'N', 'Y'], 'E').unwrap();
        self.state.spelling_bee = Some(SpellingBeeSession {
            game: SpellingBeeGame::new(
                puzzle,
                HashSet::from(["BONE".into(), "CONE".into(), "ECONOMY".into()]),
            ),
            outer_letters_shuffled: ['B', 'C', 'O', 'M', 'N', 'Y'],
            max_possible_score: 16,
            pangrams_found: 0,
            recorded_in_stats: false,
        });
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.directory);
    }
}

fn json(text: &str) -> JsonValue {
    JsonParser::new(text).parse().unwrap()
}
fn field(text: &str, key: &str) -> JsonValue {
    json(text).as_object().unwrap()[key].clone()
}

#[test]
fn parameter_parsing_requires_exact_top_level_keys_and_types() {
    assert_eq!(
        extract_string_param(r#"{"notword":"wrong","word":"a\"b\\c"}"#, "word"),
        Some("a\"b\\c".into())
    );
    assert_eq!(
        extract_string_param(r#"{"nested":{"word":"wrong"}}"#, "word"),
        None
    );
    assert_eq!(extract_string_param(r#"{"word":123}"#, "word"), None);
    assert_eq!(extract_usize_param(r#"{"size":4.5}"#, "size"), None);
    assert_eq!(
        extract_bool_param(r#"{"sound_enabled":"true"}"#, "sound_enabled"),
        None
    );
    assert_eq!(extract_string_param("not JSON", "word"), None);
}

#[test]
fn wordle_validates_custom_answers_and_records_a_win_once() {
    let mut fixture = Fixture::new();
    let state = &mut fixture.state;
    assert!(handle_wordle_start(state, r#"{"secret":"SLATE"}"#).contains("error"));
    assert!(state.wordle.is_none());
    handle_wordle_start(state, r#"{"secret":"APPLE"}"#);
    assert_eq!(
        field(&handle_wordle_state(state), "secret"),
        JsonValue::Null
    );
    handle_wordle_guess(state, r#"{"guess":"ZZZZZ"}"#);
    handle_wordle_guess(state, r#"{"guess":"ſLATE"}"#);
    assert_eq!(state.wordle.as_ref().unwrap().game.attempts_remaining(), 6);
    let won = handle_wordle_guess(state, r#"{"guess":"apple"}"#);
    assert_eq!(field(&won, "game_status"), JsonValue::String("Won".into()));
    handle_wordle_guess(state, r#"{"guess":"APPLE"}"#);
    assert_eq!(state.stats.wordle.games_played, 1);
    assert_eq!(
        UnifiedStats::load_from_file(&state.stats_path)
            .unwrap()
            .wordle
            .games_won,
        1
    );
}

#[test]
fn boggle_duplicate_guesses_and_repeated_end_do_not_double_count() {
    let mut fixture = Fixture::new();
    fixture.boggle();
    let state = &mut fixture.state;
    assert_eq!(
        field(&handle_boggle_submit(state, r#"{"word":"cat"}"#), "result"),
        JsonValue::String("Accepted".into())
    );
    handle_boggle_submit(state, r#"{"word":"CAT"}"#);
    handle_boggle_end(state);
    handle_boggle_end(state);
    assert_eq!(state.stats.boggle.games_played, 1);
    assert_eq!(state.stats.boggle.total_score_accumulated, 1);
    assert!(handle_boggle_submit(state, r#"{"word":"CATS"}"#).contains("error"));
}

#[test]
fn expired_boggle_round_rejects_late_guesses_and_records_once() {
    let mut fixture = Fixture::new();
    fixture.boggle();
    fixture.state.boggle.as_mut().unwrap().started_at = Instant::now() - Duration::from_secs(181);
    assert_eq!(
        field(&handle_boggle_state(&fixture.state), "is_active"),
        JsonValue::Bool(false)
    );
    assert!(handle_boggle_submit(&mut fixture.state, r#"{"word":"CAT"}"#).contains("error"));
    handle_boggle_end(&mut fixture.state);
    assert_eq!(fixture.state.stats.boggle.games_played, 1);
    assert_eq!(fixture.state.stats.boggle.total_score_accumulated, 0);
}

#[test]
fn replacing_boggle_round_preserves_previous_score() {
    let mut fixture = Fixture::new();
    fixture.boggle();
    handle_boggle_submit(&mut fixture.state, r#"{"word":"CAT"}"#);
    handle_boggle_start(&mut fixture.state, r#"{"size":4}"#);
    assert_eq!(fixture.state.stats.boggle.games_played, 1);
    assert_eq!(fixture.state.stats.boggle.highest_score, 1);
}

#[test]
fn incomplete_bee_puzzles_are_recorded_when_replaced() {
    let mut fixture = Fixture::new();
    fixture.bee();
    handle_spelling_bee_submit(&mut fixture.state, r#"{"word":"BONE"}"#);
    handle_spelling_bee_start(&mut fixture.state);
    assert_eq!(fixture.state.stats.spelling_bee.games_played, 1);
    assert_eq!(fixture.state.stats.spelling_bee.total_words_found, 1);
    handle_spelling_bee_start(&mut fixture.state);
    assert_eq!(fixture.state.stats.spelling_bee.games_played, 1);
}

#[test]
fn invalid_guess_characters_always_produce_valid_json() {
    let mut fixture = Fixture::new();
    fixture.bee();
    fixture.boggle();
    for body in [
        r#"{"word":"a\"bc"}"#,
        r#"{"word":"a\\bc"}"#,
        r#"{"word":"a\u0000bc"}"#,
        r#"{"word":"ßONE"}"#,
    ] {
        assert_eq!(
            field(
                &handle_spelling_bee_submit(&mut fixture.state, body),
                "result"
            ),
            JsonValue::String("InvalidCharacters".into())
        );
        assert_eq!(
            field(&handle_boggle_submit(&mut fixture.state, body), "result"),
            JsonValue::String("InvalidCharacters".into())
        );
    }
}

#[test]
fn failed_stat_write_does_not_commit_memory() {
    let mut fixture = Fixture::new();
    fixture.state.stats_path = fixture.directory.clone();
    assert!(fixture
        .state
        .update_stats(|stats| stats.wordle.record_outcome(true, 1))
        .is_err());
    assert_eq!(fixture.state.stats.wordle.games_played, 0);
    assert!(fixture.directory.is_dir());
}

#[test]
fn custom_and_untimed_boggle_duration() {
    let mut fixture = Fixture::new();
    // Test custom 60-second round
    let custom_res = handle_boggle_start(&mut fixture.state, r#"{"size":4,"duration_secs":60}"#);
    assert!(custom_res.contains(r#""ends_at_unix_ms":"#));
    assert!(!custom_res.contains(r#""ends_at_unix_ms":null"#));
    assert_eq!(fixture.state.boggle.as_ref().unwrap().duration_secs, 60);

    // Test untimed round (duration_secs: 0)
    let untimed_res = handle_boggle_start(&mut fixture.state, r#"{"size":4,"duration_secs":0}"#);
    assert!(untimed_res.contains(r#""ends_at_unix_ms":null"#));
    assert_eq!(fixture.state.boggle.as_ref().unwrap().duration_secs, 0);
    assert!(!fixture.state.boggle.as_ref().unwrap().has_expired());
}
