//! # Wordle Statistics Schema Migration
//!
//! Handles schema version validation and forward migrations for persistent statistics.

use crate::stats::model::{PlayerStats, CURRENT_SCHEMA_VERSION};

/// Error variants encountered during schema verification and migration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MigrationError {
    /// The stored data has a newer schema version than supported by this build.
    UnsupportedNewerVersion { stored: u32, supported: u32 },
    /// Corrupt data encountered during migration.
    CorruptMigrationData(String),
}

impl std::fmt::Display for MigrationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MigrationError::UnsupportedNewerVersion { stored, supported } => {
                write!(
                    f,
                    "statistics data was created with a newer version (v{stored}); this version supports up to v{supported}"
                )
            }
            MigrationError::CorruptMigrationData(msg) => {
                write!(f, "schema migration error: {msg}")
            }
        }
    }
}

impl std::error::Error for MigrationError {}

/// Validates or migrates `PlayerStats` to the current schema version.
pub fn validate_and_migrate(mut stats: PlayerStats) -> Result<PlayerStats, MigrationError> {
    if stats.schema_version > CURRENT_SCHEMA_VERSION {
        return Err(MigrationError::UnsupportedNewerVersion {
            stored: stats.schema_version,
            supported: CURRENT_SCHEMA_VERSION,
        });
    }

    // Future schema migrations will be sequenced here:
    // if stats.schema_version == 1 { stats = migrate_v1_to_v2(stats)?; }

    stats.schema_version = CURRENT_SCHEMA_VERSION;
    Ok(stats)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_current_version_passes_unchanged() {
        let stats = PlayerStats::new();
        let migrated = validate_and_migrate(stats.clone()).unwrap();
        assert_eq!(migrated.schema_version, CURRENT_SCHEMA_VERSION);
    }

    #[test]
    fn test_unsupported_newer_version_rejected() {
        let mut stats = PlayerStats::new();
        stats.schema_version = CURRENT_SCHEMA_VERSION + 1;
        let res = validate_and_migrate(stats);
        assert!(matches!(
            res,
            Err(MigrationError::UnsupportedNewerVersion { .. })
        ));
    }
}
