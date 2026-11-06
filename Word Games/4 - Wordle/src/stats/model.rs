//! # Wordle Statistics & Game History Models
//!
//! Strongly-typed data structures for completed game records, game outcomes,
//! aggregate metrics, and player history.

use std::fmt;

use crate::word::Word;

/// Maximum number of attempts allowed in standard Wordle.
pub const MAX_GUESS_DISTRIBUTION_SLOTS: usize = 6;

/// Current active schema version for persistent JSON storage.
pub const CURRENT_SCHEMA_VERSION: u32 = 1;

/// The final outcome of an individual Wordle game session.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GameOutcome {
    /// The player successfully guessed the secret word within allowed attempts.
    Won,
    /// The player exhausted all attempts without finding the secret word.
    Lost,
    /// The player explicitly abandoned or surrendered the game before reaching terminal state.
    Abandoned,
}

impl GameOutcome {
    /// Returns the canonical string representation for persistence and display.
    pub fn as_str(&self) -> &'static str {
        match self {
            GameOutcome::Won => "Won",
            GameOutcome::Lost => "Lost",
            GameOutcome::Abandoned => "Abandoned",
        }
    }

    /// Parses a string into a `GameOutcome`.
    pub fn parse(s: &str) -> Option<Self> {
        match s.trim() {
            "Won" | "won" => Some(GameOutcome::Won),
            "Lost" | "lost" => Some(GameOutcome::Lost),
            "Abandoned" | "abandoned" => Some(GameOutcome::Abandoned),
            _ => None,
        }
    }
}

impl fmt::Display for GameOutcome {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// An immutable record of an individual finished Wordle game session.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GameRecord {
    /// Unique identifier for this game session (UUID v4 or non-colliding token).
    pub game_id: String,
    /// Terminal outcome of the game.
    pub outcome: GameOutcome,
    /// Number of valid attempts consumed (1..=6 for Won, 6 for Lost, 0..=6 for Abandoned).
    pub attempts: u8,
    /// The true secret answer for this round.
    pub answer: Word,
    /// Unix timestamp (seconds) when the game began.
    pub started_at: u64,
    /// Unix timestamp (seconds) when the game reached terminal state.
    pub completed_at: u64,
}

impl GameRecord {
    /// Constructs and validates a new `GameRecord`.
    pub fn new(
        game_id: String,
        outcome: GameOutcome,
        attempts: u8,
        answer: Word,
        started_at: u64,
        completed_at: u64,
    ) -> Result<Self, &'static str> {
        if game_id.trim().is_empty() {
            return Err("game_id must not be empty");
        }
        if completed_at < started_at {
            return Err("completed_at must be >= started_at");
        }
        match outcome {
            GameOutcome::Won => {
                if !(1..=6).contains(&attempts) {
                    return Err("winning game must have attempts in 1..=6");
                }
            }
            GameOutcome::Lost => {
                if attempts == 0 {
                    return Err("lost game must record consumed attempts");
                }
            }
            GameOutcome::Abandoned => {}
        }
        Ok(Self {
            game_id,
            outcome,
            attempts,
            answer,
            started_at,
            completed_at,
        })
    }
}

/// Cumulative player statistics across multiple Wordle sessions.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlayerStats {
    /// Schema version for forward-compatible persistence.
    pub schema_version: u32,

    /// Number of completed games (Wins + Losses). Excludes abandoned games.
    pub games_played: u64,
    /// Number of completed games won.
    pub wins: u64,
    /// Number of completed games lost.
    pub losses: u64,
    /// Number of games explicitly abandoned.
    pub abandoned_games: u64,

    /// Consecutive wins in the active streak.
    pub current_streak: u64,
    /// Highest consecutive winning streak achieved all-time.
    pub maximum_streak: u64,

    /// Guess distribution histogram for wins (attempts 1 through 6).
    pub guess_distribution: [u64; MAX_GUESS_DISTRIBUTION_SLOTS],
    /// Total guesses across all completed games (Wins + Losses).
    pub total_guesses: u64,
    /// Total guesses across won games.
    pub total_guesses_in_wins: u64,

    /// Chronological list of completed games.
    pub history: Vec<GameRecord>,
}

impl Default for PlayerStats {
    fn default() -> Self {
        Self::new()
    }
}

impl PlayerStats {
    /// Initialises fresh statistics for a new player with schema version 1.
    pub const fn new() -> Self {
        Self {
            schema_version: CURRENT_SCHEMA_VERSION,
            games_played: 0,
            wins: 0,
            losses: 0,
            abandoned_games: 0,
            current_streak: 0,
            maximum_streak: 0,
            guess_distribution: [0; MAX_GUESS_DISTRIBUTION_SLOTS],
            total_guesses: 0,
            total_guesses_in_wins: 0,
            history: Vec::new(),
        }
    }

    /// Verifies all mathematical invariants on aggregate counters.
    pub fn validate_invariants(&self) -> Result<(), &'static str> {
        if self.games_played != self.wins.saturating_add(self.losses) {
            return Err("Invariant violated: games_played != wins + losses");
        }
        let dist_sum: u64 = self.guess_distribution.iter().sum();
        if dist_sum != self.wins {
            return Err("Invariant violated: sum(guess_distribution) != wins");
        }
        if self.current_streak > self.maximum_streak {
            return Err("Invariant violated: current_streak > maximum_streak");
        }
        Ok(())
    }

    /// Calculates the win percentage: $\frac{\text{wins}}{\text{wins} + \text{losses}} \times 100$.
    ///
    /// Abandoned games are strictly excluded from the denominator.
    /// Returns `0.0` safely if no completed games have been played.
    pub fn win_rate(&self) -> f64 {
        let completed = self.wins + self.losses;
        if completed == 0 {
            0.0
        } else {
            (self.wins as f64 / completed as f64) * 100.0
        }
    }

    /// Calculates the average number of guesses per completed game.
    ///
    /// Returns `None` if no completed games have been played.
    pub fn average_guesses(&self) -> Option<f64> {
        let completed = self.wins + self.losses;
        if completed == 0 {
            None
        } else {
            Some(self.total_guesses as f64 / completed as f64)
        }
    }

    /// Calculates the average number of guesses across won games.
    ///
    /// Returns `None` if no wins have been recorded yet.
    pub fn average_guesses_to_win(&self) -> Option<f64> {
        if self.wins == 0 {
            None
        } else {
            Some(self.total_guesses_in_wins as f64 / self.wins as f64)
        }
    }

    /// Renders a human-readable summary with a histogram of the guess distribution.
    pub fn render_summary(&self) -> String {
        let mut out = String::new();
        use std::fmt::Write as FmtWrite;

        writeln!(out, "=== WORDLE PLAYER STATISTICS ===").unwrap();
        writeln!(
            out,
            "Played: {} | Wins: {} | Losses: {} | Abandoned: {}",
            self.games_played, self.wins, self.losses, self.abandoned_games
        )
        .unwrap();

        let avg_guesses_str = self
            .average_guesses()
            .map(|v| format!("{v:.2}"))
            .unwrap_or_else(|| "N/A".to_string());
        let avg_win_str = self
            .average_guesses_to_win()
            .map(|v| format!("{v:.2}"))
            .unwrap_or_else(|| "N/A".to_string());

        writeln!(
            out,
            "Win Rate: {:.1}% | Current Streak: {} | Max Streak: {}",
            self.win_rate(),
            self.current_streak,
            self.maximum_streak
        )
        .unwrap();
        writeln!(
            out,
            "Avg Guesses: {} | Avg Guesses (Wins): {}",
            avg_guesses_str, avg_win_str
        )
        .unwrap();

        writeln!(out, "\nGuess Distribution:").unwrap();
        let max_val = *self.guess_distribution.iter().max().unwrap_or(&0);
        for (i, &count) in self.guess_distribution.iter().enumerate() {
            let bar_len = if max_val == 0 {
                0
            } else {
                ((count as f64 / max_val as f64) * 20.0).round() as usize
            };
            let bar = "█".repeat(bar_len);
            writeln!(out, "{} | {:<3} {}", i + 1, count, bar).unwrap();
        }

        out
    }
}

impl fmt::Display for PlayerStats {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.render_summary())
    }
}
