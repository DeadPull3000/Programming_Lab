//! Bounded loopback HTTP server for the local arcade frontend.

use crate::handlers::*;
use crate::state::SharedArcadeState;
use std::io::{self, BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{mpsc, Arc, Mutex};
use std::thread;
use std::time::Duration;
use wordle::stats::storage::{JsonParser, JsonValue};

const MAX_HEADERS: usize = 16 * 1024;
const MAX_BODY: usize = 8 * 1024;
const ALLOWED_ORIGINS: &[&str] = &[
    "http://127.0.0.1:5173",
    "http://localhost:5173",
    "http://127.0.0.1:4173",
    "http://localhost:4173",
];

pub fn start_server(state: SharedArcadeState, port: u16) -> Result<(), Box<dyn std::error::Error>> {
    let listener = TcpListener::bind(("127.0.0.1", port))?;
    println!("Cozy Arcade backend listening on http://127.0.0.1:{port}");
    let (sender, receiver) = mpsc::sync_channel::<TcpStream>(32);
    let receiver = Arc::new(Mutex::new(receiver));
    for _ in 0..4 {
        let receiver = Arc::clone(&receiver);
        let state = Arc::clone(&state);
        thread::spawn(move || loop {
            let stream = receiver.lock().unwrap().recv();
            match stream {
                Ok(stream) => {
                    if let Err(e) = handle_client(stream, Arc::clone(&state)) {
                        eprintln!("HTTP connection: {e}");
                    }
                }
                Err(_) => break,
            }
        });
    }
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                if sender.send(stream).is_err() {
                    break;
                }
            }
            Err(e) => eprintln!("Connection failed: {e}"),
        }
    }
    Ok(())
}

#[derive(Debug)]
struct Request {
    method: String,
    path: String,
    origin: Option<String>,
    body: String,
}

type RequestError = (&'static str, &'static str);

fn read_line(reader: &mut impl BufRead, remaining: &mut usize) -> Result<String, RequestError> {
    let mut bytes = Vec::new();
    reader
        .take(*remaining as u64 + 1)
        .read_until(b'\n', &mut bytes)
        .map_err(|_| ("400 Bad Request", "Incomplete request headers"))?;
    if bytes.len() > *remaining {
        return Err((
            "431 Request Header Fields Too Large",
            "Request headers too large",
        ));
    }
    *remaining -= bytes.len();
    if !bytes.ends_with(b"\r\n") {
        return Err(("400 Bad Request", "Expected CRLF-terminated headers"));
    }
    String::from_utf8(bytes).map_err(|_| ("400 Bad Request", "Headers must be UTF-8"))
}

fn read_request(reader: &mut impl BufRead) -> Result<Request, RequestError> {
    let mut remaining = MAX_HEADERS;
    let line = read_line(reader, &mut remaining)?;
    let parts: Vec<_> = line.split_whitespace().collect();
    if parts.len() != 3
        || !matches!(parts[2], "HTTP/1.1" | "HTTP/1.0")
        || !parts[1].starts_with('/')
    {
        return Err(("400 Bad Request", "Invalid request line"));
    }
    let (method, path) = (
        parts[0].to_string(),
        parts[1].split('?').next().unwrap().to_string(),
    );
    let mut content_length = None;
    let mut origin = None;
    let mut json_content_type = false;
    loop {
        let line = read_line(reader, &mut remaining)?;
        if line == "\r\n" {
            break;
        }
        let (key, value) = line
            .trim_end()
            .split_once(':')
            .ok_or(("400 Bad Request", "Invalid header"))?;
        let value = value.trim();
        match key.to_ascii_lowercase().as_str() {
            "content-length" => {
                if content_length.is_some()
                    || value.is_empty()
                    || !value.bytes().all(|b| b.is_ascii_digit())
                {
                    return Err(("400 Bad Request", "Invalid or duplicate Content-Length"));
                }
                content_length = Some(
                    value
                        .parse::<usize>()
                        .map_err(|_| ("413 Content Too Large", "Request body too large"))?,
                );
            }
            "transfer-encoding" => {
                return Err(("400 Bad Request", "Transfer-Encoding is not supported"))
            }
            "origin" => {
                if origin.is_some() || !ALLOWED_ORIGINS.contains(&value) {
                    return Err(("403 Forbidden", "Origin is not allowed"));
                }
                origin = Some(value.to_string());
            }
            "content-type" => {
                json_content_type = value
                    .split(';')
                    .next()
                    .is_some_and(|v| v.trim().eq_ignore_ascii_case("application/json"))
            }
            _ => (),
        }
    }
    let length = content_length.unwrap_or(0);
    if length > MAX_BODY {
        return Err(("413 Content Too Large", "Request body too large"));
    }
    if method == "POST" && !json_content_type {
        return Err(("415 Unsupported Media Type", "Use application/json"));
    }
    let mut bytes = vec![0; length];
    reader
        .read_exact(&mut bytes)
        .map_err(|_| ("400 Bad Request", "Incomplete request body"))?;
    let body = String::from_utf8(bytes).map_err(|_| ("400 Bad Request", "Body must be UTF-8"))?;
    Ok(Request {
        method,
        path,
        origin,
        body,
    })
}

fn handle_client(mut stream: TcpStream, state: SharedArcadeState) -> io::Result<()> {
    stream.set_read_timeout(Some(Duration::from_secs(5)))?;
    stream.set_write_timeout(Some(Duration::from_secs(5)))?;
    let request = read_request(&mut BufReader::new(stream.try_clone()?));
    let (status, body, origin) = match request {
        Ok(request) => {
            let (status, body) = if request.method == "OPTIONS" {
                ("204 No Content", String::new())
            } else {
                route_request(&state, &request.method, &request.path, &request.body)
            };
            (status, body, request.origin)
        }
        Err((status, message)) => (status, error_json(message), None),
    };
    let cors = origin
        .map(|o| format!("Access-Control-Allow-Origin: {o}\r\nVary: Origin\r\n"))
        .unwrap_or_default();
    let response = format!(
        "HTTP/1.1 {status}\r\nContent-Type: application/json; charset=utf-8\r\n{cors}Access-Control-Allow-Methods: GET, POST, OPTIONS\r\nAccess-Control-Allow-Headers: Content-Type\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}", body.len());
    stream.write_all(response.as_bytes())?;
    stream.flush()
}

fn route_request(
    state_arc: &SharedArcadeState,
    method: &str,
    path: &str,
    body: &str,
) -> (&'static str, String) {
    if method == "POST" {
        let valid = JsonParser::new(body).parse().ok().is_some_and(|value| {
            value.as_object().is_some_and(|obj| {
                obj.iter().all(|(key, value)| match key.as_str() {
                    "word" | "guess" | "secret" | "strategy" => value.as_str().is_some(),
                    "size" | "top_n" => value.as_u64().is_some(),
                    "sound_enabled" | "music_enabled" | "reduced_motion" => {
                        matches!(value, JsonValue::Bool(_))
                    }
                    _ => true,
                })
            })
        });
        if !valid {
            return (
                "400 Bad Request",
                error_json("Expected a JSON object with correctly typed parameters"),
            );
        }
    }
    let mut state = match state_arc.lock() {
        Ok(state) => state,
        Err(_) => {
            return (
                "500 Internal Server Error",
                error_json("Game state unavailable; restart the server"),
            )
        }
    };
    if state
        .boggle
        .as_ref()
        .is_some_and(|s| s.game.is_active && s.has_expired())
    {
        let result = handle_boggle_end(&mut state);
        if result.contains("\"error\"") {
            return ("500 Internal Server Error", result);
        }
    }
    let (status, response) = dispatch_request(&mut state, method, path, body);
    if JsonParser::new(&response)
        .parse()
        .ok()
        .and_then(|v| v.as_object().map(|o| o.contains_key("error")))
        .unwrap_or(false)
        && status == "200 OK"
    {
        return ("400 Bad Request", response);
    }
    (status, response)
}

fn dispatch_request(
    state: &mut crate::state::ArcadeState,
    method: &str,
    path: &str,
    body: &str,
) -> (&'static str, String) {
    match (method, path) {
        // System
        ("GET", "/api/status") => ("200 OK", handle_status(state)),
        ("GET", "/api/games/summary") => ("200 OK", handle_games_summary(state)),

        // Wordle
        ("GET", "/api/wordle/state") => ("200 OK", handle_wordle_state(state)),
        ("POST", "/api/wordle/start") => ("200 OK", handle_wordle_start(state, body)),
        ("POST", "/api/wordle/guess") => ("200 OK", handle_wordle_guess(state, body)),
        ("POST", "/api/wordle/hint") => ("200 OK", handle_wordle_hint(state, body)),

        // Spelling Bee
        ("GET", "/api/spelling-bee/state") => ("200 OK", handle_spelling_bee_state(state)),
        ("POST", "/api/spelling-bee/start") => ("200 OK", handle_spelling_bee_start(state)),
        ("POST", "/api/spelling-bee/submit") => ("200 OK", handle_spelling_bee_submit(state, body)),
        ("POST", "/api/spelling-bee/shuffle") => ("200 OK", handle_spelling_bee_shuffle(state)),
        ("GET", "/api/spelling-bee/hints") => ("200 OK", handle_spelling_bee_hints(state)),

        // Boggle
        ("GET", "/api/boggle/state") => ("200 OK", handle_boggle_state(state)),
        ("POST", "/api/boggle/start") => ("200 OK", handle_boggle_start(state, body)),
        ("POST", "/api/boggle/submit") => ("200 OK", handle_boggle_submit(state, body)),
        ("POST", "/api/boggle/end") => ("200 OK", handle_boggle_end(state)),

        // Statistics & Settings
        ("GET", "/api/stats") => ("200 OK", handle_get_stats(state)),
        ("POST", "/api/stats/reset") => ("200 OK", handle_reset_stats(state)),
        ("GET", "/api/settings") => ("200 OK", handle_get_settings(state)),
        ("POST", "/api/settings") => ("200 OK", handle_update_settings(state, body)),

        _ => (
            "404 Not Found",
            r#"{"error":"Endpoint not found"}"#.to_string(),
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    fn parse(text: &str) -> Result<Request, RequestError> {
        read_request(&mut Cursor::new(text.as_bytes()))
    }

    #[test]
    fn valid_request_and_origin_are_parsed() {
        let request = parse("POST /api/boggle/start?test=1 HTTP/1.1\r\nContent-Type: application/json; charset=utf-8\r\nContent-Length: 10\r\nOrigin: http://127.0.0.1:5173\r\n\r\n{\"size\":4}").unwrap();
        assert_eq!(request.path, "/api/boggle/start");
        assert_eq!(request.body, r#"{"size":4}"#);
        assert!(request.origin.is_some());
    }

    #[test]
    fn malformed_framing_and_untrusted_origins_are_rejected() {
        for text in [
            "GET / HTTP/1.1\r\nOrigin: https://untrusted.example\r\n\r\n",
            "POST / HTTP/1.1\r\nContent-Length: -1\r\n\r\n",
            "POST / HTTP/1.1\r\nContent-Length: 0\r\nContent-Length: 0\r\n\r\n",
            "POST / HTTP/1.1\r\nTransfer-Encoding: chunked\r\n\r\n",
            "POST / HTTP/1.1\r\nContent-Type: text/plain\r\n\r\n",
            "POST / HTTP/1.1\r\nContent-Type: application/json\r\nContent-Length: 10\r\n\r\n{}",
            "GET / HTTP/1.1\r\n",
        ] {
            assert!(parse(text).is_err(), "accepted {text}");
        }
    }

    #[test]
    fn request_sizes_are_bounded_before_allocation() {
        assert_eq!(
            parse("POST / HTTP/1.1\r\nContent-Length: 999999999\r\n\r\n")
                .unwrap_err()
                .0,
            "413 Content Too Large"
        );
        assert_eq!(
            parse(&format!(
                "GET / HTTP/1.1\r\nX-Large: {}\r\n\r\n",
                "x".repeat(MAX_HEADERS)
            ))
            .unwrap_err()
            .0,
            "431 Request Header Fields Too Large"
        );
    }

    #[test]
    fn routes_reject_non_object_and_wrongly_typed_json() {
        use crate::state::{ArcadeSettings, ArcadeState, UnifiedStats};
        use wordle::{dictionary::WordleDictionary, word::Word};
        let mut trie = trie::Trie::new();
        trie.insert("APPLE").unwrap();
        let state = Arc::new(Mutex::new(ArcadeState {
            dictionary: WordleDictionary::new(trie, vec![Word::parse("APPLE").unwrap()]).unwrap(),
            wordle: None,
            spelling_bee: None,
            boggle: None,
            stats: UnifiedStats::default(),
            settings: ArcadeSettings::default(),
            stats_path: std::env::temp_dir().join("unused_arcade_route_stats.json"),
        }));
        for body in [
            "[]",
            "null",
            "{",
            r#"{"guess":12}"#,
            r#"{"size":4.5}"#,
            r#"{"sound_enabled":"true"}"#,
        ] {
            assert_eq!(
                route_request(&state, "POST", "/api/wordle/start", body).0,
                "400 Bad Request"
            );
        }
        assert_eq!(route_request(&state, "GET", "/api/status", "").0, "200 OK");
        assert_eq!(
            route_request(&state, "GET", "/unknown", "").0,
            "404 Not Found"
        );
    }
}
