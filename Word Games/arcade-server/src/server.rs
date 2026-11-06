//! # HTTP Server Implementation for Cozy Word Arcade
//!
//! Provides a zero-dependency, high-performance, multithreaded HTTP server
//! binding to 127.0.0.1:4567 with robust CORS and clean JSON routing.

use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::Arc;
use std::thread;

use crate::handlers::*;
use crate::state::SharedArcadeState;

pub fn start_server(
    state: SharedArcadeState,
    port: u16,
) -> Result<(), Box<dyn std::error::Error>> {
    let addr = format!("127.0.0.1:{}", port);
    let listener = TcpListener::bind(&addr)?;
    println!(" Cozy Word Game Arcade Native Backend listening on http://{}", addr);

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let state_clone = Arc::clone(&state);
                thread::spawn(move || {
                    let _ = handle_client(stream, state_clone);
                });
            }
            Err(e) => {
                eprintln!("Connection failed: {}", e);
            }
        }
    }

    Ok(())
}

fn handle_client(mut stream: TcpStream, state: SharedArcadeState) -> std::io::Result<()> {
    let mut reader = BufReader::new(stream.try_clone()?);
    let mut request_line = String::new();
    if reader.read_line(&mut request_line)? == 0 {
        return Ok(());
    }

    let parts: Vec<&str> = request_line.split_whitespace().collect();
    if parts.len() < 2 {
        return Ok(());
    }

    let method = parts[0];
    let path = parts[1];

    // Read headers to determine Content-Length
    let mut content_length: usize = 0;
    loop {
        let mut header_line = String::new();
        if reader.read_line(&mut header_line)? == 0 || header_line == "\r\n" || header_line == "\n" {
            break;
        }
        let lower = header_line.to_ascii_lowercase();
        if lower.starts_with("content-length:") {
            if let Some(val) = lower.split(':').nth(1) {
                if let Ok(len) = val.trim().parse::<usize>() {
                    content_length = len;
                }
            }
        }
    }

    // Read body if Content-Length > 0
    let mut body = String::new();
    if content_length > 0 {
        let mut buffer = vec![0u8; content_length];
        reader.read_exact(&mut buffer)?;
        body = String::from_utf8_lossy(&buffer).to_string();
    }

    // Handle OPTIONS preflight for CORS
    if method == "OPTIONS" {
        let response = "HTTP/1.1 204 No Content\r\n\
Access-Control-Allow-Origin: *\r\n\
Access-Control-Allow-Methods: GET, POST, OPTIONS\r\n\
Access-Control-Allow-Headers: Content-Type\r\n\
Access-Control-Max-Age: 86400\r\n\
Connection: close\r\n\r\n";
        stream.write_all(response.as_bytes())?;
        stream.flush()?;
        return Ok(());
    }

    // Route request
    let (status_code, json_resp) = route_request(&state, method, path, &body);

    let response = format!(
        "HTTP/1.1 {}\r\n\
Content-Type: application/json; charset=utf-8\r\n\
Access-Control-Allow-Origin: *\r\n\
Access-Control-Allow-Methods: GET, POST, OPTIONS\r\n\
Access-Control-Allow-Headers: Content-Type\r\n\
Content-Length: {}\r\n\
Connection: close\r\n\r\n{}",
        status_code,
        json_resp.as_bytes().len(),
        json_resp
    );

    stream.write_all(response.as_bytes())?;
    stream.flush()?;
    Ok(())
}

fn route_request(
    state_arc: &SharedArcadeState,
    method: &str,
    path: &str,
    body: &str,
) -> (&'static str, String) {
    let mut state = state_arc.lock().expect("mutex lock succeeds");

    match (method, path) {
        // System
        ("GET", "/api/status") => ("200 OK", handle_status(&state)),
        ("GET", "/api/games/summary") => ("200 OK", handle_games_summary(&state)),

        // Wordle
        ("GET", "/api/wordle/state") => ("200 OK", handle_wordle_state(&state)),
        ("POST", "/api/wordle/start") => ("200 OK", handle_wordle_start(&mut state, body)),
        ("POST", "/api/wordle/guess") => ("200 OK", handle_wordle_guess(&mut state, body)),
        ("POST", "/api/wordle/hint") => ("200 OK", handle_wordle_hint(&state, body)),

        // Spelling Bee
        ("GET", "/api/spelling-bee/state") => ("200 OK", handle_spelling_bee_state(&state)),
        ("POST", "/api/spelling-bee/start") => ("200 OK", handle_spelling_bee_start(&mut state)),
        ("POST", "/api/spelling-bee/submit") => {
            ("200 OK", handle_spelling_bee_submit(&mut state, body))
        }
        ("POST", "/api/spelling-bee/shuffle") => {
            ("200 OK", handle_spelling_bee_shuffle(&mut state))
        }
        ("GET", "/api/spelling-bee/hints") => ("200 OK", handle_spelling_bee_hints(&state)),

        // Boggle
        ("GET", "/api/boggle/state") => ("200 OK", handle_boggle_state(&state)),
        ("POST", "/api/boggle/start") => ("200 OK", handle_boggle_start(&mut state, body)),
        ("POST", "/api/boggle/submit") => ("200 OK", handle_boggle_submit(&mut state, body)),
        ("POST", "/api/boggle/end") => ("200 OK", handle_boggle_end(&mut state)),

        // Statistics & Settings
        ("GET", "/api/stats") => ("200 OK", handle_get_stats(&state)),
        ("POST", "/api/stats/reset") => ("200 OK", handle_reset_stats(&mut state)),
        ("GET", "/api/settings") => ("200 OK", handle_get_settings(&state)),
        ("POST", "/api/settings") => ("200 OK", handle_update_settings(&mut state, body)),

        _ => ("404 Not Found", r#"{"error":"Endpoint not found"}"#.to_string()),
    }
}
