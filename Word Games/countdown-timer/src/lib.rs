//! # Countdown Timer
//!
//! A versatile, drift-free countdown timer library and CLI application written in idiomatic Rust.
//!
//! ## Core Features
//! - **System Clock Timing**: Monotonically driven by `std::time::Instant` with zero cumulative clock drift.
//! - **Configurable Durations & Limits**: Explicit maximum allowed duration limits with validation.
//! - **Dual Resolutions**: 1-second standard countdown and 100ms sub-second precision.
//! - **Large ASCII Banner Display**: 5-line block font for high-visibility terminal display.
//! - **Fearless Concurrency**: Background thread workers, `std::sync::mpsc` channels, and concurrent multi-timer coordinator.
//! - **Sequential Programmed Routines**: Exercise routine execution with distinct audio/visual phase cues.
//! - **Auditory & Visual Alerts**: Emits terminal bell sound (`\x07`) and celebration banners upon completion.

pub mod concurrency;
pub mod display;
pub mod error;
pub mod routine;
pub mod timer;

pub use concurrency::{
    ConcurrentRunner, ConcurrentTimerManager, TimerCommand, TimerEvent,
};
pub use display::{
    emit_sound, format_duration_digital, render_dashboard, render_expired_banner,
    render_large_text, render_progress_bar,
};
pub use error::TimerError;
pub use routine::{
    ExerciseRoutine, ExerciseStep, RoutineEvent, RoutinePhase,
};
pub use timer::{
    parse_duration, Clock, CountdownTimer, SystemClock, TimerConfig, TimerSnapshot,
    TimerState, DEFAULT_MAX_DURATION, DEFAULT_TICK_INTERVAL, SUBSECOND_TICK_INTERVAL,
};
