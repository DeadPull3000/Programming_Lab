use crate::timer::{TimerSnapshot, TimerState};
use std::fmt::Write as FmtWrite;
use std::io::{self, Write};
use std::time::Duration;

/// 5-line block font representation of digits and characters for large terminal format.
const FONT_HEIGHT: usize = 5;

fn digit_glyph(c: char) -> [&'static str; FONT_HEIGHT] {
    match c {
        '0' => [
            "████", //
            "█  █", //
            "█  █", //
            "█  █", //
            "████", //
        ],
        '1' => [
            "  ██", //
            " ███", //
            "  ██", //
            "  ██", //
            "████", //
        ],
        '2' => [
            "████", //
            "   █", //
            "████", //
            "█   ", //
            "████", //
        ],
        '3' => [
            "████", //
            "   █", //
            "████", //
            "   █", //
            "████", //
        ],
        '4' => [
            "█  █", //
            "█  █", //
            "████", //
            "   █", //
            "   █", //
        ],
        '5' => [
            "████", //
            "█   ", //
            "████", //
            "   █", //
            "████", //
        ],
        '6' => [
            "████", //
            "█   ", //
            "████", //
            "█  █", //
            "████", //
        ],
        '7' => [
            "████", //
            "   █", //
            "   █", //
            "   █", //
            "   █", //
        ],
        '8' => [
            "████", //
            "█  █", //
            "████", //
            "█  █", //
            "████", //
        ],
        '9' => [
            "████", //
            "█  █", //
            "████", //
            "   █", //
            "████", //
        ],
        ':' => [
            " ",  //
            "█",  //
            " ",  //
            "█",  //
            " ",  //
        ],
        '.' => [
            " ",  //
            " ",  //
            " ",  //
            " ",  //
            "█",  //
        ],
        '-' => [
            "    ", //
            "    ", //
            "████", //
            "    ", //
            "    ", //
        ],
        _ => [
            "  ", //
            "  ", //
            "  ", //
            "  ", //
            "  ", //
        ],
    }
}

/// Renders a string of characters (digits, colon, dot) into large 5-line banner text.
pub fn render_large_text(text: &str) -> String {
    let mut lines = vec![String::new(); FONT_HEIGHT];
    for ch in text.chars() {
        let glyph = digit_glyph(ch);
        for row in 0..FONT_HEIGHT {
            lines[row].push_str(glyph[row]);
            lines[row].push(' ');
        }
    }
    lines.join("\n")
}

/// Formats a duration as a digital clock string.
///
/// If `show_subsecond` is true: `MM:SS.d` or `HH:MM:SS.d`
/// Otherwise: `MM:SS` or `HH:MM:SS`
pub fn format_duration_digital(duration: Duration, show_subsecond: bool) -> String {
    let total_secs = duration.as_secs();
    let hours = total_secs / 3600;
    let mins = (total_secs % 3600) / 60;
    let secs = total_secs % 60;
    let millis = duration.subsec_millis();
    let tenths = millis / 100;

    if hours > 0 {
        if show_subsecond {
            format!("{:02}:{:02}:{:02}.{}", hours, mins, secs, tenths)
        } else {
            format!("{:02}:{:02}:{:02}", hours, mins, secs)
        }
    } else {
        if show_subsecond {
            format!("{:02}:{:02}.{}", mins, secs, tenths)
        } else {
            format!("{:02}:{:02}", mins, secs)
        }
    }
}

/// Generates a visual ASCII progress bar.
pub fn render_progress_bar(progress: f64, width: usize) -> String {
    let clamped = progress.clamp(0.0, 1.0);
    let filled = (clamped * width as f64).round() as usize;
    let empty = width.saturating_sub(filled);

    let mut bar = String::with_capacity(width + 2);
    bar.push('[');
    for _ in 0..filled {
        bar.push('█');
    }
    for _ in 0..empty {
        bar.push('░');
    }
    bar.push(']');
    bar
}

/// Formats a complete multi-line dashboard frame for the countdown timer.
pub fn render_dashboard(snapshot: &TimerSnapshot, show_subsecond: bool) -> String {
    let mut out = String::new();
    let time_str = format_duration_digital(snapshot.remaining, show_subsecond);
    let preset_str = format_duration_digital(snapshot.initial_duration, false);
    let elapsed_str = format_duration_digital(snapshot.elapsed_active, show_subsecond);

    let state_badge = match snapshot.state {
        TimerState::Ready => "[ READY ]",
        TimerState::Running => "[ RUNNING ]",
        TimerState::Paused => "[ PAUSED ]",
        TimerState::Expired => "[ EXPIRED! ]",
        TimerState::Stopped => "[ STOPPED ]",
    };

    let _ = writeln!(out, "┌────────────────────────────────────────────────────────┐");
    let _ = writeln!(
        out,
        "│  PRESET: {:<10} │  STATUS: {:<10} │  ACTIVE: {:<8} │",
        preset_str, state_badge, elapsed_str
    );
    let _ = writeln!(out, "├────────────────────────────────────────────────────────┤");

    // Large formatted digits
    let large_clock = render_large_text(&time_str);
    for line in large_clock.lines() {
        let _ = writeln!(out, "│  {:<50}  │", line);
    }

    let _ = writeln!(out, "├────────────────────────────────────────────────────────┤");
    let bar = render_progress_bar(snapshot.progress, 36);
    let percent = (snapshot.progress * 100.0).clamp(0.0, 100.0);
    let _ = writeln!(out, "│  Progress: {} {:>5.1}%  │", bar, percent);
    let _ = writeln!(out, "└────────────────────────────────────────────────────────┘");

    out
}

/// Emits the terminal audio bell (`\x07`) to produce a system sound notification.
pub fn emit_sound() {
    print!("\x07");
    let _ = io::stdout().flush();
}

/// Renders a celebration/expiration banner when the timer hits zero.
pub fn render_expired_banner(initial_duration: Duration) -> String {
    let preset = format_duration_digital(initial_duration, false);
    format!(
        "\n╔════════════════════════════════════════════════════════════════╗\n\
           ║               🔔 *** COUNTDOWN COMPLETE! *** 🔔               ║\n\
           ║    Target countdown of {} has reached 00:00.0!           ║\n\
           ╚════════════════════════════════════════════════════════════════╝\n",
        preset
    )
}
