# Countdown Timer

A versatile, drift-free, multi-threaded countdown timer application and reusable Rust library built for CS2102 Programming Lab Assignment 5.

---

## Features

- **Drift-Free Monotonic Clock**: Powered by `std::time::Instant` to prevent accumulative sleep drift.
- **Configurable Durations & Ceilings**: Supports arbitrary durations up to 24 hours (`86,400s`) with format parsing (`45s`, `3m`, `03:00`, `01:30:00`, `1.5s`).
- **Dual Resolution Modes**: Supports standard 1-second ticks or high-precision 100ms sub-second intervals (`--subsecond`).
- **Large Format ASCII Display**: 5-line tall block-character digital clock with status badges and dynamic progress bars.
- **Interactive Controls**: Real-time hotkeys to Pause/Resume (`P`), Reset (`R`), Add 10s (`+`), Subtract 10s (`-`), and Quit (`Q`).
- **Fearless Concurrency**: Fully asynchronous architecture using `std::thread` and `std::sync::mpsc` channels with non-blocking user input.
- **Sequential Programmed Exercise Routine**: Built-in 7-minute scientific workout routine (11 exercises) with automated work/rest transitions and auditory announcements.
- **Concurrent Multi-Timer Manager**: Run multiple countdown timers simultaneously in parallel threads.
- **Auditory & Visual Alerts**: Emits system terminal bell audio cues (`\x07`) and celebration banners on completion.

---

## Quick Start

### 1. Run Interactive Menu
```bash
cargo run
```

### 2. Run Direct Countdown Timer
```bash
# Countdown for 3 minutes
cargo run -- 3m

# Countdown with 100ms subsecond resolution
cargo run -- 45s --subsecond

# Countdown using MM:SS format
cargo run -- 03:00
```

### 3. Run Sequential Exercise Routine
```bash
cargo run -- --routine
```

### 4. Run Concurrent Multi-Timer Demonstration
```bash
cargo run -- --concurrent
```

### 5. Run Automated Tests
```bash
cargo test
```

---

## Exercise Routine Specification

Implements the official assignment 11-step circuit:

| # | Exercise | Duration | Rest Interval |
|---|---|---|---|
| 1 | Jumping Jacks | 30s | 5s |
| 2 | Wall Sit | 30s | 5s |
| 3 | Push-ups | 45s | 15s |
| 4 | Crunches | 45s | 15s |
| 5 | Squats | 45s | 10s |
| 6 | Dips | 30s | 5s |
| 7 | Plank | 60s (1m) | 10s |
| 8 | Spot running | 60s (1m) | 30s |
| 9 | Lunges | 30s | 5s |
| 10 | Side Plank Left | 20s | 5s |
| 11 | Side Plank Right | 20s | 0s |

**Total Routine Duration**: 8 minutes 40 seconds (520 seconds).
