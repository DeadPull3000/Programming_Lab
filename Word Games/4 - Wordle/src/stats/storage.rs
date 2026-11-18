//! # Wordle Statistics JSON Storage & Persistence
//!
//! Provides atomic, corruption-resistant JSON persistence for player statistics
//! without external dependencies.
//!
//! # Architecture & Safety Principles
//! 1. **OS Data Directory Resolution:** Automatically locates standard platform data directories
//!    (`%APPDATA%` on Windows, `XDG_DATA_HOME` on Linux, `Application Support` on macOS).
//! 2. **Atomic Writes:** Serializes to a sibling temporary file (`.tmp`), flushes and syncs to disk,
//!    then performs an atomic rename to prevent file corruption during sudden power loss or process kill.
//! 3. **Non-Destructive Error Handling:** Corrupt files are never silently overwritten with empty
//!    statistics; an explicit error is returned while the corrupt file remains preserved for recovery.
//! 4. **Self-Contained JSON Serialization:** Includes a robust recursive-descent JSON parser and
//!    formatter tailored for the statistics schema.

use std::collections::BTreeMap;
use std::fmt;
use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};

use crate::stats::migration::{validate_and_migrate, MigrationError};
use crate::stats::model::{
    GameOutcome, GameRecord, PlayerStats, CURRENT_SCHEMA_VERSION, MAX_GUESS_DISTRIBUTION_SLOTS,
};
use crate::word::Word;

/// Error variants encountered during statistics persistence, parsing, or loading.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StatsError {
    /// Underlying filesystem I/O error.
    Io(String),
    /// The stored file exists but contains invalid or malformed JSON.
    CorruptFile(String),
    /// The data version is newer than this application build can handle.
    UnsupportedSchemaVersion { stored: u32, supported: u32 },
    /// Forward migration between schema versions failed.
    MigrationFailure(String),
    /// Attempted to record a completed game whose `game_id` was already registered.
    DuplicateRecord(String),
    /// Completed game record failed semantic validation.
    InvalidRecord(String),
    /// Mathematical invariant check failed on aggregate statistics.
    InvariantViolation(String),
}

impl fmt::Display for StatsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StatsError::Io(e) => write!(f, "statistics I/O error: {e}"),
            StatsError::CorruptFile(msg) => write!(f, "corrupt statistics file: {msg}"),
            StatsError::UnsupportedSchemaVersion { stored, supported } => {
                write!(
                    f,
                    "statistics file created by newer version (v{stored}, current v{supported})"
                )
            }
            StatsError::MigrationFailure(msg) => write!(f, "statistics migration failed: {msg}"),
            StatsError::DuplicateRecord(id) => {
                write!(f, "game record with ID '{id}' has already been recorded")
            }
            StatsError::InvalidRecord(msg) => write!(f, "invalid game record: {msg}"),
            StatsError::InvariantViolation(msg) => {
                write!(f, "statistics invariant violated: {msg}")
            }
        }
    }
}

impl std::error::Error for StatsError {}

impl From<io::Error> for StatsError {
    fn from(err: io::Error) -> Self {
        StatsError::Io(err.to_string())
    }
}

impl From<MigrationError> for StatsError {
    fn from(err: MigrationError) -> Self {
        match err {
            MigrationError::UnsupportedNewerVersion { stored, supported } => {
                StatsError::UnsupportedSchemaVersion { stored, supported }
            }
            MigrationError::CorruptMigrationData(msg) => StatsError::MigrationFailure(msg),
        }
    }
}

// ----------------------------------------------------------------------------
// Minimal, Robust JSON AST & Parser
// ----------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
pub enum JsonValue {
    Null,
    Bool(bool),
    Number(f64),
    String(String),
    Array(Vec<JsonValue>),
    Object(BTreeMap<String, JsonValue>),
}

impl JsonValue {
    pub fn as_object(&self) -> Option<&BTreeMap<String, JsonValue>> {
        match self {
            JsonValue::Object(m) => Some(m),
            _ => None,
        }
    }

    pub fn as_array(&self) -> Option<&[JsonValue]> {
        match self {
            JsonValue::Array(a) => Some(a),
            _ => None,
        }
    }

    pub fn as_str(&self) -> Option<&str> {
        match self {
            JsonValue::String(s) => Some(s),
            _ => None,
        }
    }

    pub fn as_u64(&self) -> Option<u64> {
        match self {
            JsonValue::Number(n) => {
                if n.is_finite() && *n >= 0.0 && *n < u64::MAX as f64 && n.fract() == 0.0 {
                    Some(*n as u64)
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    pub fn as_u32(&self) -> Option<u32> {
        self.as_u64().and_then(|v| u32::try_from(v).ok())
    }

    pub fn as_u8(&self) -> Option<u8> {
        self.as_u64().and_then(|v| u8::try_from(v).ok())
    }
}

/// Tokenizer and parser for UTF-8 JSON text.
pub struct JsonParser<'a> {
    input: &'a [u8],
    pos: usize,
    depth: usize,
}

impl<'a> JsonParser<'a> {
    pub fn new(input: &'a str) -> Self {
        Self {
            input: input.as_bytes(),
            pos: 0,
            depth: 0,
        }
    }

    pub fn parse(&mut self) -> Result<JsonValue, StatsError> {
        self.skip_whitespace();
        let val = self.parse_value()?;
        self.skip_whitespace();
        if self.pos < self.input.len() {
            return Err(StatsError::CorruptFile(format!(
                "unexpected trailing character at byte {}",
                self.pos
            )));
        }
        Ok(val)
    }

    fn skip_whitespace(&mut self) {
        while self.pos < self.input.len() {
            match self.input[self.pos] {
                b' ' | b'\t' | b'\r' | b'\n' => self.pos += 1,
                _ => break,
            }
        }
    }

    fn peek(&self) -> Option<u8> {
        self.input.get(self.pos).copied()
    }

    fn next_char(&mut self) -> Option<u8> {
        let b = self.input.get(self.pos).copied()?;
        self.pos += 1;
        Some(b)
    }

    fn parse_value(&mut self) -> Result<JsonValue, StatsError> {
        if self.depth >= 64 {
            return Err(StatsError::CorruptFile(
                "JSON nesting limit exceeded".into(),
            ));
        }
        self.depth += 1;
        self.skip_whitespace();
        let result = match self.peek() {
            Some(b'{') => self.parse_object(),
            Some(b'[') => self.parse_array(),
            Some(b'"') => self.parse_string().map(JsonValue::String),
            Some(b't') | Some(b'f') => self.parse_bool(),
            Some(b'n') => self.parse_null(),
            Some(b'-') | Some(b'0'..=b'9') => self.parse_number(),
            Some(c) => Err(StatsError::CorruptFile(format!(
                "unexpected character '{}' at byte {}",
                c as char, self.pos
            ))),
            None => Err(StatsError::CorruptFile(
                "unexpected EOF while parsing value".to_string(),
            )),
        };
        self.depth -= 1;
        result
    }

    fn parse_object(&mut self) -> Result<JsonValue, StatsError> {
        self.expect_byte(b'{')?;
        self.skip_whitespace();
        let mut map = BTreeMap::new();

        if let Some(b'}') = self.peek() {
            self.pos += 1;
            return Ok(JsonValue::Object(map));
        }

        loop {
            self.skip_whitespace();
            let key = self.parse_string()?;
            self.skip_whitespace();
            self.expect_byte(b':')?;
            self.skip_whitespace();
            let val = self.parse_value()?;
            if map.insert(key, val).is_some() {
                return Err(StatsError::CorruptFile("duplicate object key".into()));
            }

            self.skip_whitespace();
            match self.peek() {
                Some(b',') => {
                    self.pos += 1;
                }
                Some(b'}') => {
                    self.pos += 1;
                    break;
                }
                Some(c) => {
                    return Err(StatsError::CorruptFile(format!(
                        "expected ',' or '}}' in object, found '{}' at byte {}",
                        c as char, self.pos
                    )))
                }
                None => {
                    return Err(StatsError::CorruptFile(
                        "unexpected EOF in object".to_string(),
                    ))
                }
            }
        }

        Ok(JsonValue::Object(map))
    }

    fn parse_array(&mut self) -> Result<JsonValue, StatsError> {
        self.expect_byte(b'[')?;
        self.skip_whitespace();
        let mut list = Vec::new();

        if let Some(b']') = self.peek() {
            self.pos += 1;
            return Ok(JsonValue::Array(list));
        }

        loop {
            self.skip_whitespace();
            let val = self.parse_value()?;
            list.push(val);

            self.skip_whitespace();
            match self.peek() {
                Some(b',') => {
                    self.pos += 1;
                }
                Some(b']') => {
                    self.pos += 1;
                    break;
                }
                Some(c) => {
                    return Err(StatsError::CorruptFile(format!(
                        "expected ',' or ']' in array, found '{}' at byte {}",
                        c as char, self.pos
                    )))
                }
                None => {
                    return Err(StatsError::CorruptFile(
                        "unexpected EOF in array".to_string(),
                    ))
                }
            }
        }

        Ok(JsonValue::Array(list))
    }

    fn parse_string(&mut self) -> Result<String, StatsError> {
        self.expect_byte(b'"')?;
        let mut out = String::new();

        while let Some(b) = self.next_char() {
            match b {
                b'"' => return Ok(out),
                b'\\' => {
                    let esc = self.next_char().ok_or_else(|| {
                        StatsError::CorruptFile("unexpected EOF in string escape".to_string())
                    })?;
                    match esc {
                        b'"' => out.push('"'),
                        b'\\' => out.push('\\'),
                        b'/' => out.push('/'),
                        b'b' => out.push('\u{0008}'),
                        b'f' => out.push('\u{000C}'),
                        b'n' => out.push('\n'),
                        b'r' => out.push('\r'),
                        b't' => out.push('\t'),
                        b'u' => {
                            let first = self.parse_hex_quad()?;
                            let scalar = if (0xD800..=0xDBFF).contains(&first) {
                                self.expect_byte(b'\\')?;
                                self.expect_byte(b'u')?;
                                let second = self.parse_hex_quad()?;
                                if !(0xDC00..=0xDFFF).contains(&second) {
                                    return Err(StatsError::CorruptFile(
                                        "invalid surrogate pair".into(),
                                    ));
                                }
                                0x10000 + ((first - 0xD800) << 10) + second - 0xDC00
                            } else {
                                first
                            };
                            out.push(char::from_u32(scalar).ok_or_else(|| {
                                StatsError::CorruptFile("invalid Unicode escape".into())
                            })?);
                        }
                        _ => return Err(StatsError::CorruptFile("invalid string escape".into())),
                    }
                }
                0..=0x1F => {
                    return Err(StatsError::CorruptFile(
                        "unescaped control character".into(),
                    ))
                }
                0x20..=0x7F => out.push(b as char),
                _ => {
                    let remaining = std::str::from_utf8(&self.input[self.pos - 1..])
                        .map_err(|e| StatsError::CorruptFile(e.to_string()))?;
                    let ch = remaining.chars().next().unwrap();
                    self.pos += ch.len_utf8() - 1;
                    out.push(ch);
                }
            }
        }

        Err(StatsError::CorruptFile(
            "unterminated string literal".to_string(),
        ))
    }

    fn parse_hex_quad(&mut self) -> Result<u32, StatsError> {
        let mut value = 0;
        for _ in 0..4 {
            let digit = self
                .next_char()
                .and_then(|b| (b as char).to_digit(16))
                .ok_or_else(|| StatsError::CorruptFile("invalid Unicode escape".into()))?;
            value = value * 16 + digit;
        }
        Ok(value)
    }

    fn parse_number(&mut self) -> Result<JsonValue, StatsError> {
        let start = self.pos;
        if let Some(b'-') = self.peek() {
            self.pos += 1;
        }
        match self.peek() {
            Some(b'0') => self.pos += 1,
            Some(b'1'..=b'9') => {
                while let Some(b'0'..=b'9') = self.peek() {
                    self.pos += 1;
                }
            }
            _ => return Err(StatsError::CorruptFile("expected number digits".into())),
        }
        if let Some(b'.') = self.peek() {
            self.pos += 1;
            let digits_start = self.pos;
            while let Some(b'0'..=b'9') = self.peek() {
                self.pos += 1;
            }
            if self.pos == digits_start {
                return Err(StatsError::CorruptFile("expected fraction digits".into()));
            }
        }
        if let Some(b'e') | Some(b'E') = self.peek() {
            self.pos += 1;
            if let Some(b'+') | Some(b'-') = self.peek() {
                self.pos += 1;
            }
            let digits_start = self.pos;
            while let Some(b'0'..=b'9') = self.peek() {
                self.pos += 1;
            }
            if self.pos == digits_start {
                return Err(StatsError::CorruptFile("expected exponent digits".into()));
            }
        }

        let slice = &self.input[start..self.pos];
        let s = std::str::from_utf8(slice)
            .map_err(|e| StatsError::CorruptFile(format!("invalid UTF-8 in number: {e}")))?;
        let num: f64 = s
            .parse()
            .map_err(|e| StatsError::CorruptFile(format!("invalid number format '{s}': {e}")))?;
        if !num.is_finite() {
            return Err(StatsError::CorruptFile("number out of range".into()));
        }
        Ok(JsonValue::Number(num))
    }

    fn parse_bool(&mut self) -> Result<JsonValue, StatsError> {
        if self.input[self.pos..].starts_with(b"true") {
            self.pos += 4;
            Ok(JsonValue::Bool(true))
        } else if self.input[self.pos..].starts_with(b"false") {
            self.pos += 5;
            Ok(JsonValue::Bool(false))
        } else {
            Err(StatsError::CorruptFile(format!(
                "invalid boolean token at byte {}",
                self.pos
            )))
        }
    }

    fn parse_null(&mut self) -> Result<JsonValue, StatsError> {
        if self.input[self.pos..].starts_with(b"null") {
            self.pos += 4;
            Ok(JsonValue::Null)
        } else {
            Err(StatsError::CorruptFile(format!(
                "invalid null token at byte {}",
                self.pos
            )))
        }
    }

    fn expect_byte(&mut self, expected: u8) -> Result<(), StatsError> {
        match self.next_char() {
            Some(b) if b == expected => Ok(()),
            Some(b) => Err(StatsError::CorruptFile(format!(
                "expected '{}', found '{}' at byte {}",
                expected as char, b as char, self.pos
            ))),
            None => Err(StatsError::CorruptFile(format!(
                "expected '{}', found EOF",
                expected as char
            ))),
        }
    }
}

// ----------------------------------------------------------------------------
// JSON Serialization & Deserialization for PlayerStats
// ----------------------------------------------------------------------------

/// Quotes arbitrary UTF-8 text as a JSON string, including control characters.
pub fn quote_json_string(text: &str) -> String {
    let mut out = String::from("\"");
    for ch in text.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            '\u{0000}'..='\u{001F}' => out.push_str(&format!("\\u{:04x}", ch as u32)),
            _ => out.push(ch),
        }
    }
    out.push('"');
    out
}

/// Converts `PlayerStats` into a formatted, human-readable JSON string.
pub fn serialize_stats_to_json(stats: &PlayerStats) -> String {
    let mut out = String::new();
    out.push_str("{\n");
    out.push_str(&format!(
        "  \"schema_version\": {},\n",
        stats.schema_version
    ));
    out.push_str(&format!("  \"games_played\": {},\n", stats.games_played));
    out.push_str(&format!("  \"wins\": {},\n", stats.wins));
    out.push_str(&format!("  \"losses\": {},\n", stats.losses));
    out.push_str(&format!(
        "  \"abandoned_games\": {},\n",
        stats.abandoned_games
    ));
    out.push_str(&format!(
        "  \"current_streak\": {},\n",
        stats.current_streak
    ));
    out.push_str(&format!(
        "  \"maximum_streak\": {},\n",
        stats.maximum_streak
    ));

    out.push_str("  \"guess_distribution\": [");
    for (i, &cnt) in stats.guess_distribution.iter().enumerate() {
        if i > 0 {
            out.push_str(", ");
        }
        out.push_str(&cnt.to_string());
    }
    out.push_str("],\n");

    out.push_str(&format!("  \"total_guesses\": {},\n", stats.total_guesses));
    out.push_str(&format!(
        "  \"total_guesses_in_wins\": {},\n",
        stats.total_guesses_in_wins
    ));

    out.push_str("  \"history\": [\n");
    for (i, record) in stats.history.iter().enumerate() {
        out.push_str("    {\n");
        out.push_str(&format!(
            "      \"game_id\": {},\n",
            quote_json_string(&record.game_id)
        ));
        out.push_str(&format!(
            "      \"outcome\": \"{}\",\n",
            record.outcome.as_str()
        ));
        out.push_str(&format!("      \"attempts\": {},\n", record.attempts));
        out.push_str(&format!(
            "      \"answer\": \"{}\",\n",
            record.answer.as_str()
        ));
        out.push_str(&format!("      \"started_at\": {},\n", record.started_at));
        out.push_str(&format!(
            "      \"completed_at\": {}\n",
            record.completed_at
        ));
        if i + 1 < stats.history.len() {
            out.push_str("    },\n");
        } else {
            out.push_str("    }\n");
        }
    }
    out.push_str("  ]\n");
    out.push_str("}\n");
    out
}

/// Parses and validates `PlayerStats` from a JSON string.
pub fn deserialize_stats_from_json(json_str: &str) -> Result<PlayerStats, StatsError> {
    let mut parser = JsonParser::new(json_str);
    let root = parser.parse()?;
    let obj = root
        .as_object()
        .ok_or_else(|| StatsError::CorruptFile("root JSON must be an object".to_string()))?;

    let schema_version = obj
        .get("schema_version")
        .and_then(|v| v.as_u32())
        .unwrap_or(CURRENT_SCHEMA_VERSION);

    let games_played = obj
        .get("games_played")
        .and_then(|v| v.as_u64())
        .unwrap_or(0);
    let wins = obj.get("wins").and_then(|v| v.as_u64()).unwrap_or(0);
    let losses = obj.get("losses").and_then(|v| v.as_u64()).unwrap_or(0);
    let abandoned_games = obj
        .get("abandoned_games")
        .and_then(|v| v.as_u64())
        .unwrap_or(0);
    let current_streak = obj
        .get("current_streak")
        .and_then(|v| v.as_u64())
        .unwrap_or(0);
    let maximum_streak = obj
        .get("maximum_streak")
        .and_then(|v| v.as_u64())
        .unwrap_or(0);

    let mut guess_distribution = [0u64; MAX_GUESS_DISTRIBUTION_SLOTS];
    if let Some(dist_arr) = obj.get("guess_distribution").and_then(|v| v.as_array()) {
        for (i, v) in dist_arr
            .iter()
            .take(MAX_GUESS_DISTRIBUTION_SLOTS)
            .enumerate()
        {
            if let Some(n) = v.as_u64() {
                guess_distribution[i] = n;
            }
        }
    }

    let total_guesses = obj
        .get("total_guesses")
        .and_then(|v| v.as_u64())
        .unwrap_or(0);
    let total_guesses_in_wins = obj
        .get("total_guesses_in_wins")
        .and_then(|v| v.as_u64())
        .unwrap_or(0);

    let mut history = Vec::new();
    if let Some(hist_arr) = obj.get("history").and_then(|v| v.as_array()) {
        for item in hist_arr {
            if let Some(rec_obj) = item.as_object() {
                let game_id = rec_obj
                    .get("game_id")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                let outcome_str = rec_obj
                    .get("outcome")
                    .and_then(|v| v.as_str())
                    .unwrap_or("Lost");
                let outcome = GameOutcome::parse(outcome_str).unwrap_or(GameOutcome::Lost);
                let attempts = rec_obj.get("attempts").and_then(|v| v.as_u8()).unwrap_or(6);
                let answer_str = rec_obj
                    .get("answer")
                    .and_then(|v| v.as_str())
                    .unwrap_or("CRANE");
                let answer =
                    Word::parse(answer_str).unwrap_or_else(|_| Word::parse("CRANE").unwrap());
                let started_at = rec_obj
                    .get("started_at")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0);
                let completed_at = rec_obj
                    .get("completed_at")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0);

                if let Ok(rec) =
                    GameRecord::new(game_id, outcome, attempts, answer, started_at, completed_at)
                {
                    history.push(rec);
                }
            }
        }
    }

    let stats = PlayerStats {
        schema_version,
        games_played,
        wins,
        losses,
        abandoned_games,
        current_streak,
        maximum_streak,
        guess_distribution,
        total_guesses,
        total_guesses_in_wins,
        history,
    };

    // Forward migrate if needed
    let migrated = validate_and_migrate(stats)?;
    migrated
        .validate_invariants()
        .map_err(|e| StatsError::InvariantViolation(e.to_string()))?;

    Ok(migrated)
}

// ----------------------------------------------------------------------------
// Persistent File Storage & Atomic Writes
// ----------------------------------------------------------------------------

/// Resolves the default OS application data directory for Wordle.
pub fn default_stats_path() -> PathBuf {
    #[cfg(target_os = "windows")]
    {
        if let Ok(app_data) = std::env::var("APPDATA") {
            return PathBuf::from(app_data).join("Wordle").join("stats.json");
        }
        if let Ok(local_app_data) = std::env::var("LOCALAPPDATA") {
            return PathBuf::from(local_app_data)
                .join("Wordle")
                .join("stats.json");
        }
    }

    #[cfg(target_os = "macos")]
    {
        if let Ok(home) = std::env::var("HOME") {
            return PathBuf::from(home)
                .join("Library")
                .join("Application Support")
                .join("Wordle")
                .join("stats.json");
        }
    }

    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    {
        if let Ok(data_home) = std::env::var("XDG_DATA_HOME") {
            return PathBuf::from(data_home).join("wordle").join("stats.json");
        }
        if let Ok(home) = std::env::var("HOME") {
            return PathBuf::from(home)
                .join(".local")
                .join("share")
                .join("wordle")
                .join("stats.json");
        }
    }

    PathBuf::from("wordle_stats.json")
}

/// Persistent JSON file storage manager.
#[derive(Debug, Clone)]
pub struct JsonStorage {
    file_path: PathBuf,
}

impl JsonStorage {
    /// Creates a storage manager targeting the specified path.
    pub fn new<P: Into<PathBuf>>(file_path: P) -> Self {
        Self {
            file_path: file_path.into(),
        }
    }

    /// Creates a storage manager using the platform default location.
    pub fn default_location() -> Self {
        Self::new(default_stats_path())
    }

    /// Returns the target path.
    #[inline]
    pub fn file_path(&self) -> &Path {
        &self.file_path
    }

    /// Loads `PlayerStats` from disk.
    ///
    /// If the file does not exist, returns freshly initialized default statistics.
    /// If the file contains invalid JSON, returns [`StatsError::CorruptFile`].
    pub fn load(&self) -> Result<PlayerStats, StatsError> {
        if !self.file_path.exists() {
            return Ok(PlayerStats::new());
        }

        let mut file = File::open(&self.file_path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;

        if contents.trim().is_empty() {
            return Ok(PlayerStats::new());
        }

        deserialize_stats_from_json(&contents)
    }

    /// Atomically saves `PlayerStats` to disk.
    ///
    /// 1. Creates parent directory if missing.
    /// 2. Serializes data to formatted JSON.
    /// 3. Writes to temporary sibling file in the same directory.
    /// 4. Flushes and syncs to physical disk.
    /// 5. Atomically replaces destination file.
    pub fn save(&self, stats: &PlayerStats) -> Result<(), StatsError> {
        // Enforce invariants before writing
        stats
            .validate_invariants()
            .map_err(|e| StatsError::InvariantViolation(e.to_string()))?;

        let json_text = serialize_stats_to_json(stats);
        write_atomic(&self.file_path, json_text.as_bytes())?;
        Ok(())
    }
}

/// Generates a pseudo-random nonce for safe temporary filenames.
fn rand_nonce() -> u64 {
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::time::SystemTime;
    static SEQUENCE: AtomicU64 = AtomicU64::new(0);
    let nanos = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .map(|d| d.as_nanos() as u64)
        .unwrap_or(0);
    nanos.wrapping_add(SEQUENCE.fetch_add(1, Ordering::Relaxed)) ^ (std::process::id() as u64)
}

#[cfg(test)]
mod json_regression_tests {
    use super::*;

    #[test]
    fn json_preserves_unicode_and_escaped_text() {
        let text = "quoted \"\\\n\t café 🐈";
        assert_eq!(
            JsonParser::new(&quote_json_string(text)).parse().unwrap(),
            JsonValue::String(text.into())
        );
        assert_eq!(
            JsonParser::new(r#""\u0041\u00e9\ud83d\udc08""#)
                .parse()
                .unwrap(),
            JsonValue::String("Aé🐈".into())
        );
    }

    #[test]
    fn malformed_and_excessively_nested_json_is_rejected() {
        for text in [
            r#""\q""#,
            r#""\ud800""#,
            r#""\udc00""#,
            "01",
            "1.",
            "1e",
            "1e999",
            "[1,]",
            r#"{"a":1,"a":2}"#,
            "true false",
            "\"unescaped\ncontrol\"",
        ] {
            assert!(JsonParser::new(text).parse().is_err(), "accepted {text}");
        }
        assert!(
            JsonParser::new(&format!("{}0{}", "[".repeat(100), "]".repeat(100)))
                .parse()
                .is_err()
        );
        assert_eq!(JsonValue::Number(f64::INFINITY).as_u64(), None);
        assert_eq!(JsonValue::Number(-1.0).as_u64(), None);
    }

    #[test]
    fn game_ids_with_quotes_and_unicode_round_trip() {
        let mut stats = PlayerStats::new();
        stats.history.push(
            GameRecord::new(
                "a\"\\\n🐈".into(),
                GameOutcome::Abandoned,
                0,
                Word::parse("APPLE").unwrap(),
                0,
                1,
            )
            .unwrap(),
        );
        stats.abandoned_games = 1;
        let decoded = deserialize_stats_from_json(&serialize_stats_to_json(&stats)).unwrap();
        assert_eq!(decoded.history, stats.history);
    }

    #[test]
    fn atomic_write_replaces_existing_file_and_cleans_up_on_failure() {
        let path = std::env::temp_dir().join(format!("wordle_atomic_{}", rand_nonce()));
        fs::create_dir_all(&path).unwrap();
        let file = path.join("stats.json");
        write_atomic(&file, b"first").unwrap();
        write_atomic(&file, b"second").unwrap();
        assert_eq!(fs::read(&file).unwrap(), b"second");
        assert!(write_atomic(&path, b"fail").is_err());
        assert!(path.is_dir());
        assert_eq!(fs::read_dir(&path).unwrap().count(), 1);
        fs::remove_dir_all(path).unwrap();
    }
}

/// Writes a synced sibling file and replaces the destination in one rename.
/// The old file remains intact if writing or replacement fails.
pub fn write_atomic(path: &Path, contents: &[u8]) -> io::Result<()> {
    if let Some(parent) = path.parent().filter(|p| !p.as_os_str().is_empty()) {
        fs::create_dir_all(parent)?;
    }
    let mut temp_name = path.as_os_str().to_os_string();
    temp_name.push(format!(".tmp.{}", rand_nonce()));
    let temp_path = PathBuf::from(temp_name);
    let mut file = File::options()
        .write(true)
        .create_new(true)
        .open(&temp_path)?;
    let result = (|| {
        file.write_all(contents)?;
        file.sync_all()
    })();
    drop(file);
    let result = result.and_then(|()| fs::rename(&temp_path, path));
    if result.is_err() {
        let _ = fs::remove_file(&temp_path);
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_test_dir(name: &str) -> PathBuf {
        let mut p = std::env::temp_dir();
        p.push(format!("wordle_test_{}_{}", name, rand_nonce()));
        p
    }

    #[test]
    fn test_stats_json_round_trip() {
        let mut stats = PlayerStats::new();
        stats.games_played = 2;
        stats.wins = 2;
        stats.losses = 0;
        stats.current_streak = 2;
        stats.maximum_streak = 2;
        stats.guess_distribution[2] = 1; // 3 guesses
        stats.guess_distribution[3] = 1; // 4 guesses
        stats.total_guesses = 7;
        stats.total_guesses_in_wins = 7;

        let rec = GameRecord::new(
            "test-game-1".to_string(),
            GameOutcome::Won,
            3,
            Word::parse("APPLE").unwrap(),
            1000,
            1060,
        )
        .unwrap();
        stats.history.push(rec);

        let json = serialize_stats_to_json(&stats);
        let parsed = deserialize_stats_from_json(&json).unwrap();

        assert_eq!(stats.games_played, parsed.games_played);
        assert_eq!(stats.wins, parsed.wins);
        assert_eq!(stats.losses, parsed.losses);
        assert_eq!(stats.current_streak, parsed.current_streak);
        assert_eq!(stats.maximum_streak, parsed.maximum_streak);
        assert_eq!(stats.guess_distribution, parsed.guess_distribution);
        assert_eq!(stats.history.len(), 1);
        assert_eq!(stats.history[0].game_id, "test-game-1");
    }

    #[test]
    fn test_corrupt_json_preserves_file_and_errors() {
        let dir = temp_test_dir("corrupt");
        fs::create_dir_all(&dir).unwrap();
        let file_path = dir.join("stats.json");

        // Write corrupt JSON
        let corrupt_content = "{ \"games_played\": 10, \"broken_json\": [ }";
        fs::write(&file_path, corrupt_content).unwrap();

        let storage = JsonStorage::new(&file_path);
        let load_res = storage.load();

        assert!(matches!(load_res, Err(StatsError::CorruptFile(_))));

        // Confirm original file was NOT wiped out
        let still_there = fs::read_to_string(&file_path).unwrap();
        assert_eq!(still_there, corrupt_content);

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_atomic_save_and_reload() {
        let dir = temp_test_dir("save_reload");
        let file_path = dir.join("stats.json");
        let storage = JsonStorage::new(&file_path);

        let mut stats = PlayerStats::new();
        stats.games_played = 1;
        stats.wins = 1;
        stats.guess_distribution[0] = 1;
        stats.current_streak = 1;
        stats.maximum_streak = 1;
        stats.total_guesses = 1;
        stats.total_guesses_in_wins = 1;

        storage.save(&stats).unwrap();
        assert!(file_path.exists());

        let reloaded = storage.load().unwrap();
        assert_eq!(reloaded.games_played, 1);
        assert_eq!(reloaded.wins, 1);

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_missing_file_returns_default() {
        let dir = temp_test_dir("missing");
        let file_path = dir.join("nonexistent_stats.json");
        let storage = JsonStorage::new(&file_path);

        let loaded = storage.load().unwrap();
        assert_eq!(loaded.games_played, 0);
        assert_eq!(loaded.wins, 0);
    }
}
