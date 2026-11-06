//! # Cozy Word Arcade Native Server Library
//!
//! Exposes the stateful session engine, HTTP handlers, and networking server
//! for Wordle, Spelling Bee, Boggle, persistent statistics, and settings.

pub mod handlers;
pub mod server;
pub mod state;

pub fn server_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}
