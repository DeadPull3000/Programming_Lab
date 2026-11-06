//! # State Management for Cozy Word Game Arcade
//!
//! Stores active game sessions (Wordle, Spelling Bee, Boggle),
//! player statistics, and settings.

use std::collections::HashSet;
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

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
        let total_guesses: u32 = self
            .guess_distribution
            .iter()
            .enumerate()
            .map(|(idx, &count)| (idx as u32 + 1) * count)
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
        self.best_words.sort_by(|a, b| b.len().cmp(&a.len()).then_with(|| a.cmp(b)));
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
    pub fn load_from_file(path: &Path) -> Self {
        if let Ok(mut file) = File::open(path) {
            let mut contents = String::new();
            if file.read_to_string(&mut contents).is_ok() {
                if let Some(stats) = Self::parse_json(&contents) {
                    return stats;
                }
            }
        }
        Self::default()
    }

    pub fn save_to_file(&self, path: &Path) -> std::io::Result<()> {
        let json = self.to_json();
        let temp_path = path.with_extension("tmp");
        {
            let mut file = File::create(&temp_path)?;
            file.write_all(json.as_bytes())?;
            file.flush()?;
        }
        fs::rename(temp_path, path)?;
        Ok(())
    }

    pub fn to_json(&self) -> String {
        format!(
            r#"{{"wordle":{{"games_played":{},"games_won":{},"current_streak":{},"max_streak":{},"guess_distribution":[{},{},{},{},{},{}],"win_rate":{:.1},"average_guesses":{:.2}}},"spelling_bee":{{"games_played":{},"highest_score":{},"total_words_found":{},"pangrams_found":{},"queen_bees":{}}},"boggle":{{"games_played":{},"highest_score":{},"total_words_found":{},"average_score":{:.2},"best_words":[{}]}}}}"#,
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
            self.boggle
                .best_words
                .iter()
                .map(|w| format!(r#""{}""#, w))
                .collect::<Vec<_>>()
                .join(",")
        )
    }

    fn parse_json(json: &str) -> Option<Self> {
        fn extract_u32(text: &str, key: &str) -> Option<u32> {
            let pattern = format!(r#""{}":"#, key);
            let start = text.find(&pattern)? + pattern.len();
            let remainder = text[start..].trim_start();
            let end = remainder.find(|c: char| !c.is_ascii_digit())?;
            remainder[..end].parse().ok()
        }

        let mut stats = Self::default();
        stats.wordle.games_played = extract_u32(json, "games_played").unwrap_or(0);
        stats.wordle.games_won = extract_u32(json, "games_won").unwrap_or(0);
        stats.wordle.current_streak = extract_u32(json, "current_streak").unwrap_or(0);
        stats.wordle.max_streak = extract_u32(json, "max_streak").unwrap_or(0);

        if let Some(dist_pos) = json.find(r#""guess_distribution":["#) {
            let start = dist_pos + r#""guess_distribution":["#.len();
            if let Some(end) = json[start..].find(']') {
                let nums_str = &json[start..start + end];
                let parts: Vec<&str> = nums_str.split(',').map(|s| s.trim()).collect();
                for (idx, part) in parts.iter().take(6).enumerate() {
                    if let Ok(v) = part.parse::<u32>() {
                        stats.wordle.guess_distribution[idx] = v;
                    }
                }
            }
        }

        stats.spelling_bee.highest_score = extract_u32(json, "highest_score").unwrap_or(0);
        stats.spelling_bee.total_words_found = extract_u32(json, "total_words_found").unwrap_or(0);
        stats.spelling_bee.pangrams_found = extract_u32(json, "pangrams_found").unwrap_or(0);
        stats.spelling_bee.queen_bees = extract_u32(json, "queen_bees").unwrap_or(0);

        stats.boggle.highest_score = extract_u32(json, "highest_score").unwrap_or(0);
        stats.boggle.total_words_found = extract_u32(json, "total_words_found").unwrap_or(0);

        Some(stats)
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

        let stats_path = PathBuf::from("arcade_stats.json");
        let stats = UnifiedStats::load_from_file(&stats_path);
        let settings = ArcadeSettings::default();

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

    pub fn save_stats(&self) {
        let _ = self.stats.save_to_file(&self.stats_path);
    }
}

pub type SharedArcadeState = Arc<Mutex<ArcadeState>>;
