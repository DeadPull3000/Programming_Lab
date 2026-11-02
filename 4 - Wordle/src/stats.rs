//! # Wordle Player Statistics & Persistence
//!
//! Tracks player performance metrics across multiple game sessions:
//! - Total games played and won.
//! - Win percentage.
//! - Current streak and all-time maximum streak.
//! - Guess distribution histogram (wins by attempts 1 through 6).
//! - Clean plain-text persistence format without external serialization dependencies.

use std::fmt;
use std::io::{self, BufRead, Write};

use crate::game::{GameStatus, DEFAULT_MAX_ATTEMPTS};

/// Cumulative player statistics across multiple Wordle sessions.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlayerStats {
    pub games_played: u32,
    pub games_won: u32,
    pub current_streak: u32,
    pub max_streak: u32,
    pub guess_distribution: [u32; DEFAULT_MAX_ATTEMPTS],
}

impl Default for PlayerStats {
    fn default() -> Self {
        Self::new()
    }
}

impl PlayerStats {
    /// Creates a fresh `PlayerStats` record initialized to zeros.
    pub const fn new() -> Self {
        Self {
            games_played: 0,
            games_won: 0,
            current_streak: 0,
            max_streak: 0,
            guess_distribution: [0; DEFAULT_MAX_ATTEMPTS],
        }
    }

    /// Records the outcome of a completed Wordle game.
    ///
    /// # State Transitions
    /// - **Won:**
    ///   - Increments `games_played` and `games_won`.
    ///   - Increments `current_streak` and updates `max_streak`.
    ///   - Increments `guess_distribution[attempts_used - 1]`.
    /// - **Lost:**
    ///   - Increments `games_played`.
    ///   - Resets `current_streak` to 0.
    ///   - Preserves `max_streak`.
    /// - **InProgress:** No-op.
    pub fn record_game(&mut self, status: GameStatus, attempts_used: usize) {
        match status {
            GameStatus::Won => {
                self.games_played += 1;
                self.games_won += 1;
                self.current_streak += 1;
                if self.current_streak > self.max_streak {
                    self.max_streak = self.current_streak;
                }
                if (1..=DEFAULT_MAX_ATTEMPTS).contains(&attempts_used) {
                    self.guess_distribution[attempts_used - 1] += 1;
                }
            }
            GameStatus::Lost => {
                self.games_played += 1;
                self.current_streak = 0;
            }
            GameStatus::InProgress => {}
        }
    }

    /// Calculates the win percentage (0.0% to 100.0%).
    ///
    /// Returns `0.0` safely if no games have been played.
    pub fn win_percentage(&self) -> f64 {
        if self.games_played == 0 {
            0.0
        } else {
            (self.games_won as f64 / self.games_played as f64) * 100.0
        }
    }

    /// Serializes the statistics into a key-value plain text stream.
    pub fn write_to<W: Write>(&self, mut writer: W) -> io::Result<()> {
        writeln!(writer, "games_played={}", self.games_played)?;
        writeln!(writer, "games_won={}", self.games_won)?;
        writeln!(writer, "current_streak={}", self.current_streak)?;
        writeln!(writer, "max_streak={}", self.max_streak)?;
        write!(writer, "dist=")?;
        for (i, count) in self.guess_distribution.iter().enumerate() {
            if i > 0 {
                write!(writer, ",")?;
            }
            write!(writer, "{count}")?;
        }
        writeln!(writer)?;
        Ok(())
    }

    /// Parses a `PlayerStats` record from a key-value text stream.
    pub fn read_from<R: BufRead>(reader: R) -> io::Result<Self> {
        let mut stats = Self::new();
        for line in reader.lines() {
            let line = line?;
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }
            if let Some((key, val)) = trimmed.split_once('=') {
                match key.trim() {
                    "games_played" => {
                        stats.games_played = val.trim().parse().unwrap_or(0);
                    }
                    "games_won" => {
                        stats.games_won = val.trim().parse().unwrap_or(0);
                    }
                    "current_streak" => {
                        stats.current_streak = val.trim().parse().unwrap_or(0);
                    }
                    "max_streak" => {
                        stats.max_streak = val.trim().parse().unwrap_or(0);
                    }
                    "dist" => {
                        let parts: Vec<&str> = val.split(',').collect();
                        for (i, p) in parts.iter().take(DEFAULT_MAX_ATTEMPTS).enumerate() {
                            stats.guess_distribution[i] = p.trim().parse().unwrap_or(0);
                        }
                    }
                    _ => {}
                }
            }
        }
        Ok(stats)
    }

    /// Formats the statistics summary and guess distribution histogram.
    pub fn render_summary(&self) -> String {
        let mut out = String::new();
        use std::fmt::Write as FmtWrite;

        writeln!(out, "=== STATISTICS ===").unwrap();
        writeln!(
            out,
            "Played: {} | Win Rate: {:.1}% | Current Streak: {} | Max Streak: {}",
            self.games_played,
            self.win_percentage(),
            self.current_streak,
            self.max_streak
        )
        .unwrap();

        writeln!(out, "\nGuess Distribution:").unwrap();
        let max_val = *self.guess_distribution.iter().max().unwrap_or(&0);
        for i in 0..DEFAULT_MAX_ATTEMPTS {
            let count = self.guess_distribution[i];
            let bar_len = if max_val == 0 {
                0
            } else {
                ((count as f64 / max_val as f64) * 20.0).round() as usize
            };
            let bar = "█".repeat(bar_len);
            writeln!(out, "{} | {:<2} {}", i + 1, count, bar).unwrap();
        }

        out
    }
}

impl fmt::Display for PlayerStats {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.render_summary())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_initial_stats_zero() {
        let stats = PlayerStats::new();
        assert_eq!(stats.games_played, 0);
        assert_eq!(stats.games_won, 0);
        assert_eq!(stats.win_percentage(), 0.0);
        assert_eq!(stats.current_streak, 0);
        assert_eq!(stats.max_streak, 0);
        assert_eq!(stats.guess_distribution, [0; 6]);
    }

    #[test]
    fn test_record_wins_and_streaks() {
        let mut stats = PlayerStats::new();

        // Game 1: Win on attempt 3
        stats.record_game(GameStatus::Won, 3);
        assert_eq!(stats.games_played, 1);
        assert_eq!(stats.games_won, 1);
        assert_eq!(stats.current_streak, 1);
        assert_eq!(stats.max_streak, 1);
        assert_eq!(stats.guess_distribution[2], 1);

        // Game 2: Win on attempt 4
        stats.record_game(GameStatus::Won, 4);
        assert_eq!(stats.games_played, 2);
        assert_eq!(stats.games_won, 2);
        assert_eq!(stats.current_streak, 2);
        assert_eq!(stats.max_streak, 2);
        assert_eq!(stats.guess_distribution[3], 1);

        // Game 3: Loss
        stats.record_game(GameStatus::Lost, 6);
        assert_eq!(stats.games_played, 3);
        assert_eq!(stats.games_won, 2);
        assert_eq!(stats.current_streak, 0); // RESET
        assert_eq!(stats.max_streak, 2); // PRESERVED!
        assert!((stats.win_percentage() - 66.666).abs() < 0.1);
    }

    #[test]
    fn test_serialization_round_trip() {
        let mut stats = PlayerStats::new();
        stats.record_game(GameStatus::Won, 2);
        stats.record_game(GameStatus::Won, 3);
        stats.record_game(GameStatus::Lost, 6);

        let mut buffer = Vec::new();
        stats.write_to(&mut buffer).unwrap();

        let reader = Cursor::new(buffer);
        let loaded = PlayerStats::read_from(reader).unwrap();

        assert_eq!(stats, loaded);
    }
}
