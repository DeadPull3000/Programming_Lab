//! # Wordle Persistent Player Statistics Subsystem
//!
//! Provides a modular, robust persistence architecture for Wordle player records:
//! - [`model`]: Strongly-typed outcome enums, game session logs, and aggregate metrics.
//! - [`manager`]: Central state coordinator enforcing duplicate protection and invariant validity.
//! - [`storage`]: Safe atomic JSON persistence, OS data directory discovery, and corrupt file preservation.
//! - [`migration`]: Forward schema versioning and upgrade paths.

pub mod manager;
pub mod migration;
pub mod model;
pub mod storage;

pub use manager::StatsManager;
pub use migration::{validate_and_migrate, MigrationError};
pub use model::{
    GameOutcome, GameRecord, PlayerStats, CURRENT_SCHEMA_VERSION, MAX_GUESS_DISTRIBUTION_SLOTS,
};
pub use storage::{default_stats_path, JsonStorage, StatsError};
