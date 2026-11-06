//! # Wordle Statistics Manager
//!
//! Central coordinator for recording completed game sessions, maintaining aggregate metrics,
//! enforcing mathematical invariants, and coordinating safe persistence.

use crate::stats::model::{GameOutcome, GameRecord, PlayerStats, MAX_GUESS_DISTRIBUTION_SLOTS};
use crate::stats::storage::{JsonStorage, StatsError};

/// Central manager orchestrating player statistics updates and persistent storage.
#[derive(Debug, Clone)]
pub struct StatsManager {
    stats: PlayerStats,
    storage: JsonStorage,
}

impl StatsManager {
    /// Initialises a `StatsManager` loading existing data from the provided storage.
    pub fn new(storage: JsonStorage) -> Result<Self, StatsError> {
        let stats = storage.load()?;
        Ok(Self { stats, storage })
    }

    /// Constructs a `StatsManager` with the platform default storage location.
    pub fn default_manager() -> Result<Self, StatsError> {
        Self::new(JsonStorage::default_location())
    }

    /// Constructs a `StatsManager` directly with in-memory statistics.
    pub fn from_stats(stats: PlayerStats, storage: JsonStorage) -> Self {
        Self { stats, storage }
    }

    /// Returns an immutable reference to the active aggregate statistics.
    #[inline]
    pub fn stats(&self) -> &PlayerStats {
        &self.stats
    }

    /// Returns a reference to the storage backend.
    #[inline]
    pub fn storage(&self) -> &JsonStorage {
        &self.storage
    }

    /// Calculates the player's win percentage ($\frac{\text{wins}}{\text{wins} + \text{losses}} \times 100$).
    #[inline]
    pub fn win_rate(&self) -> f64 {
        self.stats.win_rate()
    }

    /// Calculates the average number of attempts per completed game.
    #[inline]
    pub fn average_guesses(&self) -> Option<f64> {
        self.stats.average_guesses()
    }

    /// Calculates the average number of attempts across won games.
    #[inline]
    pub fn average_guesses_to_win(&self) -> Option<f64> {
        self.stats.average_guesses_to_win()
    }

    /// Formats the summary text and histogram.
    #[inline]
    pub fn render_summary(&self) -> String {
        self.stats.render_summary()
    }

    /// Records a completed game record, updates aggregate counters, and persists changes.
    ///
    /// # Safety & Invariants
    /// 1. Rejects duplicate game IDs: if `record.game_id` was already recorded, returns [`StatsError::DuplicateRecord`].
    /// 2. Performs checked addition to prevent counter overflow.
    /// 3. Validates invariants before committing.
    /// 4. In-memory state is committed **only** after persistent storage write succeeds.
    ///    If the write fails, in-memory state remains untouched and the error is surfaced.
    pub fn record_completed_game(&mut self, record: GameRecord) -> Result<(), StatsError> {
        // 1. Check for duplicate game ID
        if self
            .stats
            .history
            .iter()
            .any(|r| r.game_id == record.game_id)
        {
            return Err(StatsError::DuplicateRecord(record.game_id));
        }

        // 2. Clone in-memory state to prepare candidate update
        let mut updated = self.stats.clone();

        match record.outcome {
            GameOutcome::Won => {
                if record.attempts == 0 || record.attempts as usize > MAX_GUESS_DISTRIBUTION_SLOTS {
                    return Err(StatsError::InvalidRecord(format!(
                        "winning attempts must be in 1..={MAX_GUESS_DISTRIBUTION_SLOTS}, found {}",
                        record.attempts
                    )));
                }

                updated.games_played = updated.games_played.checked_add(1).ok_or_else(|| {
                    StatsError::InvalidRecord("games_played counter overflow".into())
                })?;
                updated.wins = updated
                    .wins
                    .checked_add(1)
                    .ok_or_else(|| StatsError::InvalidRecord("wins counter overflow".into()))?;
                updated.current_streak = updated
                    .current_streak
                    .checked_add(1)
                    .ok_or_else(|| StatsError::InvalidRecord("current_streak overflow".into()))?;

                if updated.current_streak > updated.maximum_streak {
                    updated.maximum_streak = updated.current_streak;
                }

                let slot = (record.attempts - 1) as usize;
                updated.guess_distribution[slot] = updated.guess_distribution[slot]
                    .checked_add(1)
                    .ok_or_else(|| StatsError::InvalidRecord("distribution overflow".into()))?;

                updated.total_guesses = updated
                    .total_guesses
                    .checked_add(record.attempts as u64)
                    .ok_or_else(|| {
                    StatsError::InvalidRecord("total_guesses overflow".into())
                })?;
                updated.total_guesses_in_wins = updated
                    .total_guesses_in_wins
                    .checked_add(record.attempts as u64)
                    .ok_or_else(|| {
                        StatsError::InvalidRecord("total_guesses_in_wins overflow".into())
                    })?;
            }
            GameOutcome::Lost => {
                updated.games_played = updated.games_played.checked_add(1).ok_or_else(|| {
                    StatsError::InvalidRecord("games_played counter overflow".into())
                })?;
                updated.losses = updated
                    .losses
                    .checked_add(1)
                    .ok_or_else(|| StatsError::InvalidRecord("losses counter overflow".into()))?;
                // Streak resets on loss
                updated.current_streak = 0;

                updated.total_guesses = updated
                    .total_guesses
                    .checked_add(record.attempts as u64)
                    .ok_or_else(|| {
                    StatsError::InvalidRecord("total_guesses overflow".into())
                })?;
            }
            GameOutcome::Abandoned => {
                // Abandoned games increment separate counter; streaks and games_played are untouched
                updated.abandoned_games = updated
                    .abandoned_games
                    .checked_add(1)
                    .ok_or_else(|| StatsError::InvalidRecord("abandoned_games overflow".into()))?;
            }
        }

        updated.history.push(record);

        // 3. Validate mathematical invariants
        updated
            .validate_invariants()
            .map_err(|e| StatsError::InvariantViolation(e.to_string()))?;

        // 4. Persist to disk FIRST
        self.storage.save(&updated)?;

        // 5. Commit in-memory state on success
        self.stats = updated;
        Ok(())
    }

    /// Resets all statistics to clean default state and updates persistent storage.
    pub fn reset(&mut self) -> Result<(), StatsError> {
        let fresh = PlayerStats::new();
        self.storage.save(&fresh)?;
        self.stats = fresh;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::word::Word;
    use std::fs;
    use std::path::PathBuf;

    fn temp_storage(name: &str) -> (JsonStorage, PathBuf) {
        let mut p = std::env::temp_dir();
        p.push(format!("wordle_mgr_test_{}_{}", name, std::process::id()));
        fs::create_dir_all(&p).unwrap();
        let file = p.join("stats.json");
        (JsonStorage::new(&file), p)
    }

    #[test]
    fn test_initial_manager_state_zeros() {
        let (storage, dir) = temp_storage("initial");
        let mgr = StatsManager::new(storage).unwrap();
        assert_eq!(mgr.stats().games_played, 0);
        assert_eq!(mgr.stats().wins, 0);
        assert_eq!(mgr.stats().losses, 0);
        assert_eq!(mgr.stats().abandoned_games, 0);
        assert_eq!(mgr.win_rate(), 0.0);
        assert_eq!(mgr.average_guesses(), None);
        assert_eq!(mgr.average_guesses_to_win(), None);
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn test_record_wins_all_six_attempts() {
        let (storage, dir) = temp_storage("six_wins");
        let mut mgr = StatsManager::new(storage).unwrap();

        for attempts in 1..=6 {
            let rec = GameRecord::new(
                format!("game-win-{attempts}"),
                GameOutcome::Won,
                attempts,
                Word::parse("APPLE").unwrap(),
                1000,
                1050,
            )
            .unwrap();
            mgr.record_completed_game(rec).unwrap();
        }

        assert_eq!(mgr.stats().games_played, 6);
        assert_eq!(mgr.stats().wins, 6);
        assert_eq!(mgr.stats().losses, 0);
        assert_eq!(mgr.stats().current_streak, 6);
        assert_eq!(mgr.stats().maximum_streak, 6);
        assert_eq!(mgr.stats().guess_distribution, [1, 1, 1, 1, 1, 1]);
        assert_eq!(mgr.win_rate(), 100.0);
        // Total guesses = 1+2+3+4+5+6 = 21. Avg = 21/6 = 3.5
        assert_eq!(mgr.average_guesses(), Some(3.5));
        assert_eq!(mgr.average_guesses_to_win(), Some(3.5));

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn test_record_loss_resets_streak_preserves_maximum() {
        let (storage, dir) = temp_storage("streak_reset");
        let mut mgr = StatsManager::new(storage).unwrap();

        // 2 wins
        mgr.record_completed_game(
            GameRecord::new(
                "g1".into(),
                GameOutcome::Won,
                3,
                Word::parse("APPLE").unwrap(),
                10,
                20,
            )
            .unwrap(),
        )
        .unwrap();
        mgr.record_completed_game(
            GameRecord::new(
                "g2".into(),
                GameOutcome::Won,
                4,
                Word::parse("CRANE").unwrap(),
                30,
                40,
            )
            .unwrap(),
        )
        .unwrap();

        assert_eq!(mgr.stats().current_streak, 2);
        assert_eq!(mgr.stats().maximum_streak, 2);

        // 1 loss
        mgr.record_completed_game(
            GameRecord::new(
                "g3".into(),
                GameOutcome::Lost,
                6,
                Word::parse("SLATE").unwrap(),
                50,
                60,
            )
            .unwrap(),
        )
        .unwrap();

        assert_eq!(mgr.stats().games_played, 3);
        assert_eq!(mgr.stats().wins, 2);
        assert_eq!(mgr.stats().losses, 1);
        assert_eq!(mgr.stats().current_streak, 0); // RESET
        assert_eq!(mgr.stats().maximum_streak, 2); // PRESERVED

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn test_abandonment_preserves_streak_and_played_count() {
        let (storage, dir) = temp_storage("abandonment");
        let mut mgr = StatsManager::new(storage).unwrap();

        // 1 win
        mgr.record_completed_game(
            GameRecord::new(
                "g1".into(),
                GameOutcome::Won,
                2,
                Word::parse("APPLE").unwrap(),
                10,
                20,
            )
            .unwrap(),
        )
        .unwrap();

        assert_eq!(mgr.stats().current_streak, 1);
        assert_eq!(mgr.stats().games_played, 1);

        // Abandon game
        mgr.record_completed_game(
            GameRecord::new(
                "g2".into(),
                GameOutcome::Abandoned,
                3,
                Word::parse("ROAST").unwrap(),
                30,
                40,
            )
            .unwrap(),
        )
        .unwrap();

        assert_eq!(mgr.stats().current_streak, 1); // Streak preserved!
        assert_eq!(mgr.stats().games_played, 1); // Not counted as played win/loss!
        assert_eq!(mgr.stats().abandoned_games, 1);

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn test_duplicate_game_id_rejection() {
        let (storage, dir) = temp_storage("duplicate");
        let mut mgr = StatsManager::new(storage).unwrap();

        let rec = GameRecord::new(
            "unique-game-123".into(),
            GameOutcome::Won,
            3,
            Word::parse("APPLE").unwrap(),
            10,
            20,
        )
        .unwrap();

        mgr.record_completed_game(rec.clone()).unwrap();
        assert_eq!(mgr.stats().games_played, 1);

        // Attempt second recording with identical ID
        let dup_res = mgr.record_completed_game(rec);
        assert!(matches!(dup_res, Err(StatsError::DuplicateRecord(_))));
        assert_eq!(
            mgr.stats().games_played,
            1,
            "Duplicate must not increment stats"
        );

        let _ = fs::remove_dir_all(dir);
    }
}
