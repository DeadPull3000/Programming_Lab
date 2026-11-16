use crate::error::TimerError;
use std::fmt;
use std::sync::Arc;
use std::time::{Duration, Instant};

/// Maximum allowed timer duration: 24 hours (86,400 seconds).
pub const DEFAULT_MAX_DURATION: Duration = Duration::from_secs(86_400);

/// Default tick interval: 1 second (1000ms).
pub const DEFAULT_TICK_INTERVAL: Duration = Duration::from_secs(1);

/// High-resolution tick interval: 100 milliseconds.
pub const SUBSECOND_TICK_INTERVAL: Duration = Duration::from_millis(100);

/// Clock abstraction allowing deterministic testing alongside real monotonic clock.
pub trait Clock: Send + Sync {
    fn now(&self) -> Instant;
}

/// Standard system monotonic clock powered by `std::time::Instant`.
#[derive(Debug, Default, Clone, Copy)]
pub struct SystemClock;

impl Clock for SystemClock {
    #[inline]
    fn now(&self) -> Instant {
        Instant::now()
    }
}

/// Operational states of the countdown timer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimerState {
    /// Initialized with duration, ready to start.
    Ready,
    /// Currently ticking against the system clock.
    Running,
    /// Frozen at current remaining duration; system clock progression suspended.
    Paused,
    /// Target duration has elapsed (reached 0).
    Expired,
    /// Manually cancelled or halted.
    Stopped,
}

impl fmt::Display for TimerState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TimerState::Ready => write!(f, "READY"),
            TimerState::Running => write!(f, "RUNNING"),
            TimerState::Paused => write!(f, "PAUSED"),
            TimerState::Expired => write!(f, "EXPIRED"),
            TimerState::Stopped => write!(f, "STOPPED"),
        }
    }
}

/// Configuration settings for the countdown timer.
#[derive(Debug, Clone)]
pub struct TimerConfig {
    pub initial_duration: Duration,
    pub tick_interval: Duration,
    pub max_duration: Duration,
}

impl Default for TimerConfig {
    fn default() -> Self {
        Self {
            initial_duration: Duration::from_secs(60),
            tick_interval: DEFAULT_TICK_INTERVAL,
            max_duration: DEFAULT_MAX_DURATION,
        }
    }
}

impl TimerConfig {
    pub fn new(initial_duration: Duration) -> Result<Self, TimerError> {
        Self::with_options(initial_duration, DEFAULT_TICK_INTERVAL, DEFAULT_MAX_DURATION)
    }

    pub fn with_options(
        initial_duration: Duration,
        tick_interval: Duration,
        max_duration: Duration,
    ) -> Result<Self, TimerError> {
        if initial_duration.is_zero() {
            return Err(TimerError::ZeroDuration);
        }
        if initial_duration > max_duration {
            return Err(TimerError::ExceedsMaximumDuration {
                max: max_duration,
                requested: initial_duration,
            });
        }
        Ok(Self {
            initial_duration,
            tick_interval,
            max_duration,
        })
    }
}

/// An immutable point-in-time snapshot of the countdown timer state.
#[derive(Debug, Clone, PartialEq)]
pub struct TimerSnapshot {
    pub state: TimerState,
    pub initial_duration: Duration,
    pub remaining: Duration,
    pub elapsed_active: Duration,
    pub progress: f64,
    pub tick_interval: Duration,
}

impl TimerSnapshot {
    pub fn is_expired(&self) -> bool {
        self.state == TimerState::Expired || self.remaining.is_zero()
    }

    pub fn is_running(&self) -> bool {
        self.state == TimerState::Running
    }

    pub fn is_paused(&self) -> bool {
        self.state == TimerState::Paused
    }
}

/// Core Countdown Timer engine backed by monotonic system clock.
pub struct CountdownTimer {
    config: TimerConfig,
    state: TimerState,
    accumulated_active: Duration,
    current_session_start: Option<Instant>,
    clock: Arc<dyn Clock>,
}

impl CountdownTimer {
    /// Creates a new countdown timer with default system clock.
    pub fn new(config: TimerConfig) -> Self {
        Self::with_clock(config, Arc::new(SystemClock))
    }

    /// Creates a new countdown timer with a custom clock (for tests/mocking).
    pub fn with_clock(config: TimerConfig, clock: Arc<dyn Clock>) -> Self {
        Self {
            config,
            state: TimerState::Ready,
            accumulated_active: Duration::ZERO,
            current_session_start: None,
            clock,
        }
    }

    /// Start the countdown timer from Ready or Stopped state.
    pub fn start(&mut self) -> Result<(), TimerError> {
        match self.state {
            TimerState::Ready | TimerState::Stopped => {
                self.current_session_start = Some(self.clock.now());
                self.state = TimerState::Running;
                Ok(())
            }
            TimerState::Running => Ok(()),
            TimerState::Paused => self.resume(),
            TimerState::Expired => Err(TimerError::InvalidStateTransition {
                from: self.state.to_string(),
                action: "start (reset required)",
            }),
        }
    }

    /// Pause the countdown timer, freezing remaining time.
    pub fn pause(&mut self) -> Result<(), TimerError> {
        match self.state {
            TimerState::Running => {
                if let Some(start) = self.current_session_start.take() {
                    let now = self.clock.now();
                    let elapsed = now.saturating_duration_since(start);
                    self.accumulated_active = self.accumulated_active.saturating_add(elapsed);
                }
                self.state = TimerState::Paused;
                // If it reached 0 right as paused, expire it
                if self.accumulated_active >= self.config.initial_duration {
                    self.accumulated_active = self.config.initial_duration;
                    self.state = TimerState::Expired;
                }
                Ok(())
            }
            TimerState::Paused => Ok(()),
            _ => Err(TimerError::InvalidStateTransition {
                from: self.state.to_string(),
                action: "pause",
            }),
        }
    }

    /// Resume the countdown timer from Paused state.
    pub fn resume(&mut self) -> Result<(), TimerError> {
        match self.state {
            TimerState::Paused => {
                if self.accumulated_active >= self.config.initial_duration {
                    self.state = TimerState::Expired;
                    return Ok(());
                }
                self.current_session_start = Some(self.clock.now());
                self.state = TimerState::Running;
                Ok(())
            }
            TimerState::Running => Ok(()),
            _ => Err(TimerError::InvalidStateTransition {
                from: self.state.to_string(),
                action: "resume",
            }),
        }
    }

    /// Toggle between Running and Paused modes.
    pub fn toggle_pause(&mut self) -> Result<TimerState, TimerError> {
        match self.state {
            TimerState::Running => {
                self.pause()?;
                Ok(TimerState::Paused)
            }
            TimerState::Paused => {
                self.resume()?;
                Ok(TimerState::Running)
            }
            TimerState::Ready => {
                self.start()?;
                Ok(TimerState::Running)
            }
            _ => Err(TimerError::InvalidStateTransition {
                from: self.state.to_string(),
                action: "toggle_pause",
            }),
        }
    }

    /// Reset the timer back to its initial preset duration and Ready state.
    pub fn reset(&mut self) {
        self.state = TimerState::Ready;
        self.accumulated_active = Duration::ZERO;
        self.current_session_start = None;
    }

    /// Reset the timer with a new duration.
    pub fn reset_with_duration(&mut self, new_duration: Duration) -> Result<(), TimerError> {
        if new_duration.is_zero() {
            return Err(TimerError::ZeroDuration);
        }
        if new_duration > self.config.max_duration {
            return Err(TimerError::ExceedsMaximumDuration {
                max: self.config.max_duration,
                requested: new_duration,
            });
        }
        self.config.initial_duration = new_duration;
        self.reset();
        Ok(())
    }

    /// Stop and halt the timer.
    pub fn stop(&mut self) {
        self.current_session_start = None;
        self.state = TimerState::Stopped;
    }

    /// Dynamically adjust remaining time by a delta in seconds (positive or negative).
    pub fn adjust_seconds(&mut self, delta_secs: i64) {
        let current_remaining = self.remaining();
        let new_remaining_secs = if delta_secs >= 0 {
            current_remaining.as_secs().saturating_add(delta_secs as u64)
        } else {
            current_remaining.as_secs().saturating_sub((-delta_secs) as u64)
        };

        let clamped_secs = new_remaining_secs.min(self.config.max_duration.as_secs());
        if clamped_secs == 0 {
            self.accumulated_active = self.config.initial_duration;
            self.current_session_start = None;
            self.state = TimerState::Expired;
            return;
        }

        let new_remaining = Duration::from_secs(clamped_secs);
        if new_remaining > self.config.initial_duration {
            self.config.initial_duration = new_remaining;
            self.accumulated_active = Duration::ZERO;
        } else {
            self.accumulated_active = self.config.initial_duration.saturating_sub(new_remaining);
        }

        if self.state == TimerState::Running {
            self.current_session_start = Some(self.clock.now());
        }
    }

    /// Calculates current remaining time based on the system clock.
    pub fn remaining(&mut self) -> Duration {
        self.update_state();
        self.config
            .initial_duration
            .saturating_sub(self.total_active_elapsed())
    }

    /// Calculates total active (unpaused) time elapsed so far.
    fn total_active_elapsed(&self) -> Duration {
        match (self.state, self.current_session_start) {
            (TimerState::Running, Some(start)) => {
                let session_elapsed = self.clock.now().saturating_duration_since(start);
                self.accumulated_active.saturating_add(session_elapsed)
            }
            _ => self.accumulated_active,
        }
    }

    /// Updates internal state if duration has reached 0.
    fn update_state(&mut self) {
        if self.state == TimerState::Running {
            let active = self.total_active_elapsed();
            if active >= self.config.initial_duration {
                self.accumulated_active = self.config.initial_duration;
                self.current_session_start = None;
                self.state = TimerState::Expired;
            }
        }
    }

    /// Generates a current snapshot of timer state.
    pub fn snapshot(&mut self) -> TimerSnapshot {
        self.update_state();
        let active = self.total_active_elapsed().min(self.config.initial_duration);
        let remaining = self.config.initial_duration.saturating_sub(active);
        let total_nanos = self.config.initial_duration.as_nanos() as f64;
        let progress = if total_nanos > 0.0 {
            1.0 - (remaining.as_nanos() as f64 / total_nanos)
        } else {
            1.0
        };

        TimerSnapshot {
            state: self.state,
            initial_duration: self.config.initial_duration,
            remaining,
            elapsed_active: active,
            progress: progress.clamp(0.0, 1.0),
            tick_interval: self.config.tick_interval,
        }
    }

    /// Current operational state.
    pub fn state(&self) -> TimerState {
        self.state
    }

    /// Initial preset duration.
    pub fn initial_duration(&self) -> Duration {
        self.config.initial_duration
    }

    /// Tick interval.
    pub fn tick_interval(&self) -> Duration {
        self.config.tick_interval
    }
}

/// Helper function to parse user input strings into Duration.
///
/// Supported formats:
/// - Plain seconds: `"45"`, `"180"`
/// - Suffixes: `"30s"`, `"5m"`, `"1h"`, `"1m30s"`, `"2h15m"`
/// - Colon format: `"03:00"` (MM:SS), `"01:30:00"` (HH:MM:SS)
/// - Subsecond: `"1.5s"`, `"10.2"`
pub fn parse_duration(input: &str) -> Result<Duration, TimerError> {
    let s = input.trim();
    if s.is_empty() {
        return Err(TimerError::InvalidFormat("Input is empty".into()));
    }

    // Check colon format: HH:MM:SS or MM:SS
    if s.contains(':') {
        let parts: Vec<&str> = s.split(':').collect();
        match parts.len() {
            2 => {
                let mins: u64 = parts[0]
                    .trim()
                    .parse()
                    .map_err(|_| TimerError::InvalidFormat(format!("Invalid minutes: '{}'", parts[0])))?;
                let secs: u64 = parts[1]
                    .trim()
                    .parse()
                    .map_err(|_| TimerError::InvalidFormat(format!("Invalid seconds: '{}'", parts[1])))?;
                return Ok(Duration::from_secs(mins * 60 + secs));
            }
            3 => {
                let hours: u64 = parts[0]
                    .trim()
                    .parse()
                    .map_err(|_| TimerError::InvalidFormat(format!("Invalid hours: '{}'", parts[0])))?;
                let mins: u64 = parts[1]
                    .trim()
                    .parse()
                    .map_err(|_| TimerError::InvalidFormat(format!("Invalid minutes: '{}'", parts[1])))?;
                let secs: u64 = parts[2]
                    .trim()
                    .parse()
                    .map_err(|_| TimerError::InvalidFormat(format!("Invalid seconds: '{}'", parts[2])))?;
                return Ok(Duration::from_secs(hours * 3600 + mins * 60 + secs));
            }
            _ => {
                return Err(TimerError::InvalidFormat(
                    "Colon time must be MM:SS or HH:MM:SS".into(),
                ))
            }
        }
    }

    // Try parsing pure float/integer seconds
    if let Ok(secs_f) = s.parse::<f64>() {
        if secs_f < 0.0 {
            return Err(TimerError::InvalidFormat("Negative duration not supported".into()));
        }
        let millis = (secs_f * 1000.0).round() as u64;
        return Ok(Duration::from_millis(millis));
    }

    // Try unit format: e.g. "1h20m30s" or "45s" or "5m"
    let mut total_millis: u64 = 0;
    let mut num_buf = String::new();
    let mut matched_any = false;

    for ch in s.chars() {
        if ch.is_ascii_digit() || ch == '.' {
            num_buf.push(ch);
        } else {
            let unit = ch.to_ascii_lowercase();
            if num_buf.is_empty() {
                return Err(TimerError::InvalidFormat(format!(
                    "Unexpected unit '{}' without preceding number in '{}'",
                    ch, s
                )));
            }
            let val: f64 = num_buf.parse().map_err(|_| {
                TimerError::InvalidFormat(format!("Invalid number '{}' in '{}'", num_buf, s))
            })?;
            num_buf.clear();
            matched_any = true;

            match unit {
                'h' => total_millis += (val * 3600.0 * 1000.0).round() as u64,
                'm' => total_millis += (val * 60.0 * 1000.0).round() as u64,
                's' => total_millis += (val * 1000.0).round() as u64,
                _ => {
                    return Err(TimerError::InvalidFormat(format!(
                        "Unknown unit '{}' in '{}'. Use 'h', 'm', or 's'.",
                        ch, s
                    )))
                }
            }
        }
    }

    if !num_buf.is_empty() {
        // Trailing numbers without unit - treat as seconds
        let val: f64 = num_buf.parse().map_err(|_| {
            TimerError::InvalidFormat(format!("Invalid number '{}' in '{}'", num_buf, s))
        })?;
        total_millis += (val * 1000.0).round() as u64;
        matched_any = true;
    }

    if !matched_any {
        return Err(TimerError::InvalidFormat(format!("Could not parse duration from '{}'", s)));
    }

    Ok(Duration::from_millis(total_millis))
}
