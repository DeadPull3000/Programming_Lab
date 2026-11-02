//! # Cozy Word Arcade Native Server Binary
//!
//! Initializes the authoritative game engine, loads lexicons and Trie structures,
//! loads player persistence records, and listens for GUI requests on 127.0.0.1:4567.

use std::sync::{Arc, Mutex};
use arcade_server::server::start_server;
use arcade_server::state::ArcadeState;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("------------------------------------------------------------");
    println!("     A Cozy Word Game Arcade — Native Rust Backend Engine    ");
    println!("------------------------------------------------------------");
    println!("Loading Trie dictionary and curated answer pools...");

    let state = ArcadeState::new()?;
    println!(
        "Dictionaries loaded successfully! ({} answers, {} legal guesses)",
        state.dictionary.answers().len(),
        state.dictionary.legal_guesses().len()
    );

    let shared_state = Arc::new(Mutex::new(state));
    start_server(shared_state, 4567)?;

    Ok(())
}
