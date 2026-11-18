//! # State Management for Cozy Word Game Arcade
//!
//! Stores active game sessions (Wordle, Spelling Bee, Boggle),
//! player statistics, and settings.

use std::collections::HashSet;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use wordle::stats::storage::{quote_json_string, write_atomic, JsonParser, JsonValue};

use boggle::game::BoggleGame;
use spelling_bee::game::SpellingBeeGame;
use wordle::candidate::CandidateSet;
use wordle::dictionary::WordleDictionary;
use wordle::game::WordleGame;
use wordle::keyboard::KeyboardState;

// ============================================================================
// Settings Model
// ============================================================================

#[derive(Debug, Clone)]
pub struct ArcadeSettings {
    pub sound_enabled: bool,
    pub music_enabled: bool,
    pub reduced_motion: bool,
}

impl Default for ArcadeSettings {
    fn default() -> Self {
        Self {
            sound_enabled: true,
            music_enabled: false,
            reduced_motion: false,
        }
    }
}

impl ArcadeSettings {
    pub fn to_json(&self) -> String {
        format!(
            r#"{{"sound_enabled":{},"music_enabled":{},"reduced_motion":{}}}"#,
            self.sound_enabled, self.music_enabled, self.reduced_motion
        )
    }

    fn load(path: &Path) -> io::Result<Self> {
        let text = match fs::read_to_string(path) {
            Ok(text) => text,
            Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(Self::default()),
            Err(e) => return Err(e),
        };
        let invalid = || {
            io::Error::new(
                io::ErrorKind::InvalidData,
                "Invalid arcade settings; original file preserved",
            )
        };
        let root = JsonParser::new(&text).parse().map_err(|_| invalid())?;
        let obj = root.as_object().ok_or_else(invalid)?;
        let get = |key: &str| match obj.get(key) {
            Some(JsonValue::Bool(value)) => Ok(*value),
            _ => Err(invalid()),
        };
        Ok(Self {
            sound_enabled: get("sound_enabled")?,
            music_enabled: get("music_enabled")?,
            reduced_motion: get("reduced_motion")?,
        })
    }
}

// ============================================================================
// Statistics Model
// ============================================================================

#[derive(Debug, Clone, Default)]
pub struct WordleStatsRecord {
    pub games_played: u32,
    pub games_won: u32,
    pub current_streak: u32,
    pub max_streak: u32,
    pub guess_distribution: [u32; 6],
}

impl WordleStatsRecord {
    pub fn record_outcome(&mut self, won: bool, attempts: usize) {
        self.games_played += 1;
        if won {
            self.games_won += 1;
            self.current_streak += 1;
            if self.current_streak > self.max_streak {
                self.max_streak = self.current_streak;
            }
            if (1..=6).contains(&attempts) {
                self.guess_distribution[attempts - 1] += 1;
            }
        } else {
            self.current_streak = 0;
        }
    }

    pub fn win_rate(&self) -> f64 {
        if self.games_played == 0 {
            0.0
        } else {
            (self.games_won as f64 / self.games_played as f64) * 100.0
        }
    }

    pub fn average_guesses(&self) -> f64 {
        let total_guesses: u64 = self
            .guess_distribution
            .iter()
            .enumerate()
            .map(|(idx, &count)| (idx as u64 + 1) * count as u64)
            .sum();
        if self.games_won == 0 {
            0.0
        } else {
            total_guesses as f64 / self.games_won as f64
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct SpellingBeeStatsRecord {
    pub games_played: u32,
    pub highest_score: u32,
    pub total_words_found: u32,
    pub pangrams_found: u32,
    pub queen_bees: u32,
}

impl SpellingBeeStatsRecord {
    pub fn record_game(&mut self, score: u32, words: u32, pangrams: u32, is_queen: bool) {
        self.games_played += 1;
        if score > self.highest_score {
            self.highest_score = score;
        }
        self.total_words_found += words;
        self.pangrams_found += pangrams;
        if is_queen {
            self.queen_bees += 1;
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct BoggleStatsRecord {
    pub games_played: u32,
    pub highest_score: u32,
    pub total_words_found: u32,
    pub total_score_accumulated: u32,
    pub best_words: Vec<String>,
}

impl BoggleStatsRecord {
    pub fn record_game(&mut self, score: u32, words: &[String]) {
        self.games_played += 1;
        if score > self.highest_score {
            self.highest_score = score;
        }
        self.total_words_found += words.len() as u32;
        self.total_score_accumulated += score;

        for w in words {
            if !self.best_words.contains(w) {
                self.best_words.push(w.clone());
            }
        }
        // Keep best words sorted by length descending, top 10
        self.best_words
            .sort_by(|a, b| b.len().cmp(&a.len()).then_with(|| a.cmp(b)));
        if self.best_words.len() > 10 {
            self.best_words.truncate(10);
        }
    }

    pub fn average_score(&self) -> f64 {
        if self.games_played == 0 {
            0.0
        } else {
            self.total_score_accumulated as f64 / self.games_played as f64
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct UnifiedStats {
    pub wordle: WordleStatsRecord,
    pub spelling_bee: SpellingBeeStatsRecord,
    pub boggle: BoggleStatsRecord,
}

impl UnifiedStats {
    pub fn load_from_file(path: &Path) -> io::Result<Self> {
        match fs::read_to_string(path) {
            Ok(contents) => Self::parse_json(&contents).ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::InvalidData,
                    "Invalid arcade statistics; original file preserved",
                )
            }),
            Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(Self::default()),
            Err(e) => Err(e),
        }
    }

    pub fn save_to_file(&self, path: &Path) -> io::Result<()> {
        write_atomic(path, self.to_json().as_bytes())
    }

    pub fn to_json(&self) -> String {
        format!(
            r#"{{"wordle":{{"games_played":{},"games_won":{},"current_streak":{},"max_streak":{},"guess_distribution":[{},{},{},{},{},{}],"win_rate":{:.1},"average_guesses":{:.2}}},"spelling_bee":{{"games_played":{},"highest_score":{},"total_words_found":{},"pangrams_found":{},"queen_bees":{}}},"boggle":{{"games_played":{},"highest_score":{},"total_words_found":{},"average_score":{:.2},"total_score_accumulated":{},"best_words":[{}]}}}}"#,
            self.wordle.games_played,
            self.wordle.games_won,
            self.wordle.current_streak,
            self.wordle.max_streak,
            self.wordle.guess_distribution[0],
            self.wordle.guess_distribution[1],
            self.wordle.guess_distribution[2],
            self.wordle.guess_distribution[3],
            self.wordle.guess_distribution[4],
            self.wordle.guess_distribution[5],
            self.wordle.win_rate(),
            self.wordle.average_guesses(),
            self.spelling_bee.games_played,
            self.spelling_bee.highest_score,
            self.spelling_bee.total_words_found,
            self.spelling_bee.pangrams_found,
            self.spelling_bee.queen_bees,
            self.boggle.games_played,
            self.boggle.highest_score,
            self.boggle.total_words_found,
            self.boggle.average_score(),
            self.boggle.total_score_accumulated,
            self.boggle
                .best_words
                .iter()
                .map(|w| quote_json_string(w))
                .collect::<Vec<_>>()
                .join(",")
        )
    }

    fn parse_json(json: &str) -> Option<Self> {
        let root = JsonParser::new(json).parse().ok()?;
        let root = root.as_object()?;
        let wordle = root.get("wordle")?.as_object()?;
        let bee = root.get("spelling_bee")?.as_object()?;
        let boggle = root.get("boggle")?.as_object()?;
        let count = |object: &std::collections::BTreeMap<String, JsonValue>, key: &str| {
            object.get(key)?.as_u32()
        };
        let distribution = wordle.get("guess_distribution")?.as_array()?;
        if distribution.len() != 6 {
            return None;
        }
        let mut guess_distribution = [0; 6];
        for (slot, value) in guess_distribution.iter_mut().zip(distribution) {
            *slot = value.as_u32()?;
        }
        let best_words = boggle
            .get("best_words")?
            .as_array()?
            .iter()
            .map(|v| v.as_str().map(str::to_owned))
            .collect::<Option<Vec<_>>>()?;
        let games_played = count(boggle, "games_played")?;
        // Older versions stored only a rounded average; recover the closest total.
        let total_score_accumulated = match boggle.get("total_score_accumulated") {
            Some(value) => value.as_u32()?,
            None => match boggle.get("average_score")? {
                JsonValue::Number(avg)
                    if *avg >= 0.0 && *avg * games_played as f64 <= u32::MAX as f64 =>
                {
                    (*avg * games_played as f64).round() as u32
                }
                _ => return None,
            },
        };
        Some(Self {
            wordle: WordleStatsRecord {
                games_played: count(wordle, "games_played")?,
                games_won: count(wordle, "games_won")?,
                current_streak: count(wordle, "current_streak")?,
                max_streak: count(wordle, "max_streak")?,
                guess_distribution,
            },
            spelling_bee: SpellingBeeStatsRecord {
                games_played: count(bee, "games_played")?,
                highest_score: count(bee, "highest_score")?,
                total_words_found: count(bee, "total_words_found")?,
                pangrams_found: count(bee, "pangrams_found")?,
                queen_bees: count(bee, "queen_bees")?,
            },
            boggle: BoggleStatsRecord {
                games_played,
                highest_score: count(boggle, "highest_score")?,
                total_words_found: count(boggle, "total_words_found")?,
                total_score_accumulated,
                best_words,
            },
        })
    }
}

// ============================================================================
// Active Game Sessions
// ============================================================================

#[derive(Debug, Clone)]
pub struct WordleSession {
    pub game: WordleGame,
    pub candidate_set: CandidateSet,
    pub keyboard: KeyboardState,
    pub recorded_in_stats: bool,
}

#[derive(Debug, Clone)]
pub struct SpellingBeeSession {
    pub game: SpellingBeeGame,
    pub outer_letters_shuffled: [char; 6],
    pub max_possible_score: u32,
    pub pangrams_found: u32,
    pub recorded_in_stats: bool,
}

#[derive(Debug, Clone)]
pub struct BoggleSession {
    pub game: BoggleGame,
    pub size: usize,
    pub all_solutions: HashSet<String>,
    pub max_possible_score: u32,
    pub recorded_in_stats: bool,
    pub started_at: Instant,
    pub ends_at_unix_ms: Option<u64>,
    pub duration_secs: u64,
}

impl BoggleSession {
    pub fn has_expired(&self) -> bool {
        if self.duration_secs == 0 {
            return false;
        }
        self.started_at.elapsed() >= Duration::from_secs(self.duration_secs)
    }

    pub fn deadline_unix_ms() -> Option<u64> {
        Self::deadline_for_duration(180)
    }

    pub fn deadline_for_duration(duration_secs: u64) -> Option<u64> {
        if duration_secs == 0 {
            None
        } else {
            let now_ms = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64;
            Some(now_ms + duration_secs * 1000)
        }
    }
}

// ============================================================================
// Central Arcade State
// ============================================================================

pub struct ArcadeState {
    pub dictionary: WordleDictionary,
    pub wordle: Option<WordleSession>,
    pub spelling_bee: Option<SpellingBeeSession>,
    pub boggle: Option<BoggleSession>,
    pub stats: UnifiedStats,
    pub settings: ArcadeSettings,
    pub stats_path: PathBuf,
}

impl ArcadeState {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let dictionary = WordleDictionary::load_default()
            .map_err(|e| format!("Failed to load Wordle dictionary: {e}"))?;

        let data_dir = std::env::var_os("ARCADE_DATA_DIR")
            .map(PathBuf::from)
            .unwrap_or_default();
        let stats_path = data_dir.join("arcade_stats.json");
        let stats = UnifiedStats::load_from_file(&stats_path)?;
        let settings = ArcadeSettings::load(&data_dir.join("arcade_settings.json"))?;

        Ok(Self {
            dictionary,
            wordle: None,
            spelling_bee: None,
            boggle: None,
            stats,
            settings,
            stats_path,
        })
    }

    pub fn update_stats(&mut self, update: impl FnOnce(&mut UnifiedStats)) -> io::Result<()> {
        let mut updated = self.stats.clone();
        update(&mut updated);
        updated.save_to_file(&self.stats_path)?;
        self.stats = updated;
        Ok(())
    }

    pub fn save_settings(&self, settings: &ArcadeSettings) -> io::Result<()> {
        write_atomic(
            &self.stats_path.with_file_name("arcade_settings.json"),
            settings.to_json().as_bytes(),
        )
    }
}

pub type SharedArcadeState = Arc<Mutex<ArcadeState>>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn statistics_round_trip_keeps_game_counters_separate() {
        let mut stats = UnifiedStats::default();
        stats.wordle.record_outcome(true, 3);
        stats.wordle.record_outcome(false, 6);
        stats.spelling_bee.record_game(100, 20, 2, false);
        stats.spelling_bee.record_game(150, 30, 3, true);
        stats.spelling_bee.record_game(70, 15, 1, false);
        stats.boggle.record_game(12, &["CATS".into()]);
        stats
            .boggle
            .record_game(23, &["DOGS".into(), "QUOTE\"\\🐈".into()]);
        let restored = UnifiedStats::parse_json(&stats.to_json()).unwrap();
        assert_eq!(restored.to_json(), stats.to_json());
        assert_eq!(restored.wordle.games_played, 2);
        assert_eq!(restored.spelling_bee.games_played, 3);
        assert_eq!(restored.boggle.games_played, 2);
        assert_eq!(restored.boggle.average_score(), 17.5);
    }

    #[test]
    fn legacy_boggle_average_is_migrated() {
        let mut stats = UnifiedStats::default();
        stats.boggle.record_game(12, &[]);
        stats.boggle.record_game(23, &[]);
        let old = stats
            .to_json()
            .replace(",\"total_score_accumulated\":35", "");
        assert_eq!(
            UnifiedStats::parse_json(&old)
                .unwrap()
                .boggle
                .total_score_accumulated,
            35
        );
        assert!(UnifiedStats::parse_json("{}").is_none());
        assert!(UnifiedStats::parse_json("not json").is_none());
    }

    #[test]
    fn corrupt_statistics_are_preserved() {
        let path =
            std::env::temp_dir().join(format!("arcade_corrupt_{}.json", rand::random::<u64>()));
        fs::write(&path, "bad json").unwrap();
        assert!(UnifiedStats::load_from_file(&path).is_err());
        assert_eq!(fs::read_to_string(&path).unwrap(), "bad json");
        fs::remove_file(path).unwrap();
    }

    #[test]
    fn settings_survive_save_and_reload() {
        let path =
            std::env::temp_dir().join(format!("arcade_settings_{}.json", rand::random::<u64>()));
        let settings = ArcadeSettings {
            sound_enabled: false,
            music_enabled: true,
            reduced_motion: true,
        };
        write_atomic(&path, settings.to_json().as_bytes()).unwrap();
        assert_eq!(
            ArcadeSettings::load(&path).unwrap().to_json(),
            settings.to_json()
        );
        fs::remove_file(path).unwrap();
    }
}
