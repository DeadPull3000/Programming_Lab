use countdown_timer::concurrency::{ConcurrentRunner, ConcurrentTimerManager, TimerCommand, TimerEvent};
use countdown_timer::display::{
    emit_sound, format_duration_digital, render_dashboard, render_expired_banner, render_progress_bar,
};
use countdown_timer::routine::{ExerciseRoutine, RoutineEvent, RoutinePhase};
use countdown_timer::timer::{
    parse_duration, TimerConfig, DEFAULT_MAX_DURATION, DEFAULT_TICK_INTERVAL,
    SUBSECOND_TICK_INTERVAL,
};
use std::env;
use std::io::{self, BufRead, Write};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::channel;
use std::sync::Arc;
use std::thread;
use std::time::Duration;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();

    // Check command line flags
    if args.len() > 1 {
        match args[1].as_str() {
            "--help" | "-h" => {
                print_help();
                return Ok(());
            }
            "--routine" | "-r" => {
                run_exercise_routine()?;
                return Ok(());
            }
            "--concurrent" | "-c" => {
                run_concurrent_demo()?;
                return Ok(());
            }
            val => {
                // Parse duration from argument
                let duration = parse_duration(val)?;
                let subsecond = args.iter().any(|a| a == "--subsecond" || a == "-s");
                run_single_timer(duration, subsecond)?;
                return Ok(());
            }
        }
    }

    // Interactive main menu
    interactive_menu()?;
    Ok(())
}

fn print_help() {
    println!(
        "\nCountdown Timer - High-Precision Concurrent Timer\n\
         Usage:\n  \
         countdown-timer [DURATION] [OPTIONS]\n  \
         countdown-timer --routine\n  \
         countdown-timer --concurrent\n\n\
         Arguments:\n  \
         [DURATION]    Initial duration (e.g. '180', '45s', '3m', '01:30:00', '1.5s')\n\n\
         Options:\n  \
         -s, --subsecond   Run with 100ms sub-second interval updates\n  \
         -r, --routine     Run the programmed sequential exercise routine\n  \
         -c, --concurrent  Demonstrate multiple concurrent timers running in parallel\n  \
         -h, --help        Show this help message\n"
    );
}

fn interactive_menu() -> Result<(), Box<dyn std::error::Error>> {
    let stdin = io::stdin();
    let mut reader = stdin.lock();

    loop {
        println!("\n========================================================");
        println!("             ⏱️   COUNTDOWN TIMER SUITE  ⏱️             ");
        println!("========================================================");
        println!("1. Start Countdown Timer (Custom Duration)");
        println!("2. Sequential Exercise Routine (7-Minute Workout Table)");
        println!("3. Concurrent Multi-Timer Demonstration (Parallel Threads)");
        println!("4. Exit");
        print!("\nSelect an option [1-4] > ");
        io::stdout().flush()?;

        let mut input = String::new();
        reader.read_line(&mut input)?;
        let choice = input.trim();

        match choice {
            "1" => {
                print!("Enter initial duration (e.g., 30s, 3m, 03:00, 45) [Max 24h] > ");
                io::stdout().flush()?;
                let mut dur_str = String::new();
                reader.read_line(&mut dur_str)?;
                match parse_duration(dur_str.trim()) {
                    Ok(duration) => {
                        print!("Enable 100ms sub-second resolution? (y/N) > ");
                        io::stdout().flush()?;
                        let mut sub_str = String::new();
                        reader.read_line(&mut sub_str)?;
                        let subsecond = sub_str.trim().eq_ignore_ascii_case("y");
                        run_single_timer(duration, subsecond)?;
                    }
                    Err(e) => eprintln!("Error: {}", e),
                }
            }
            "2" => {
                run_exercise_routine()?;
            }
            "3" => {
                run_concurrent_demo()?;
            }
            "4" | "q" | "exit" => {
                println!("Goodbye!");
                break;
            }
            _ => println!("Invalid option, please choose 1 to 4."),
        }
    }

    Ok(())
}

fn run_single_timer(duration: Duration, subsecond: bool) -> Result<(), Box<dyn std::error::Error>> {
    let tick_interval = if subsecond {
        SUBSECOND_TICK_INTERVAL
    } else {
        DEFAULT_TICK_INTERVAL
    };

    let config = TimerConfig::with_options(duration, tick_interval, DEFAULT_MAX_DURATION)?;
    let runner = ConcurrentRunner::spawn(config);

    println!("\nStarting Countdown Timer...");
    println!("Controls: [P] Pause/Resume | [R] Reset | [+] +10s | [-] -10s | [Q] Quit\n");

    // Spawn input listener thread
    let (key_tx, key_rx) = channel::<String>();
    let stop_input = Arc::new(AtomicBool::new(false));
    let stop_input_clone = Arc::clone(&stop_input);

    thread::spawn(move || {
        let stdin = io::stdin();
        let mut reader = stdin.lock();
        let mut line = String::new();
        while !stop_input_clone.load(Ordering::Relaxed) {
            line.clear();
            if reader.read_line(&mut line).is_ok() {
                let trimmed = line.trim().to_lowercase();
                if key_tx.send(trimmed).is_err() {
                    break;
                }
            }
        }
    });

    runner.send_command(TimerCommand::Start)?;

    let mut expired = false;
    while !expired {
        // Check keyboard input
        if let Ok(key) = key_rx.try_recv() {
            match key.as_str() {
                "p" | "pause" | "" => {
                    let _ = runner.send_command(TimerCommand::TogglePause);
                }
                "r" | "reset" => {
                    let _ = runner.send_command(TimerCommand::Reset);
                    println!("\n*** Timer Reset to Preset! ***\n");
                }
                "+" | "+10" => {
                    let _ = runner.send_command(TimerCommand::AdjustSeconds(10));
                }
                "-" | "-10" => {
                    let _ = runner.send_command(TimerCommand::AdjustSeconds(-10));
                }
                "q" | "quit" => {
                    println!("\nQuitting timer...");
                    break;
                }
                _ => {}
            }
        }

        // Process timer events
        while let Some(event) = runner.try_recv_event() {
            match event {
                TimerEvent::Tick(snap) => {
                    // ANSI clear screen and redraw dashboard
                    print!("\x1b[H\x1b[2J");
                    print!("{}", render_dashboard(&snap, subsecond));
                    println!("Controls: [Enter/P] Pause/Resume | [R] Reset | [+] +10s | [-] -10s | [Q] Quit");
                    let _ = io::stdout().flush();
                }
                TimerEvent::Paused => {
                    println!("\n[Timer Suspended / Display Frozen]");
                }
                TimerEvent::Resumed => {
                    println!("\n[Timer Resumed]");
                }
                TimerEvent::Expired(snap) => {
                    print!("\x1b[H\x1b[2J");
                    print!("{}", render_dashboard(&snap, subsecond));
                    emit_sound();
                    println!("{}", render_expired_banner(snap.initial_duration));
                    expired = true;
                    break;
                }
                _ => {}
            }
        }

        thread::sleep(Duration::from_millis(50));
    }

    stop_input.store(true, Ordering::Relaxed);
    runner.shutdown();
    Ok(())
}

fn run_exercise_routine() -> Result<(), Box<dyn std::error::Error>> {
    let mut routine = ExerciseRoutine::default_assignment_routine();
    println!("\n========================================================");
    println!("          🏋️   PROGRAMMED EXERCISE ROUTINE   🏋️         ");
    println!("========================================================");
    print!("{}", routine.format_table());
    println!("\nPress Enter to begin the workout routine...");
    let mut dummy = String::new();
    io::stdin().read_line(&mut dummy)?;

    let mut event = routine.start()?;
    emit_sound();

    loop {
        match &event {
            RoutineEvent::WorkStarted {
                index,
                total,
                name,
                duration,
                next_name,
            } => {
                emit_sound();
                println!("\n🟢 ========================================================");
                println!(
                    "   EXERCISE [{}/{}]: {} ({}s)",
                    index + 1,
                    total,
                    name.to_uppercase(),
                    duration.as_secs()
                );
                if let Some(nxt) = next_name {
                    println!("   Next up: {}", nxt);
                }
                println!("========================================================\n");
            }
            RoutineEvent::RestStarted {
                rest_duration,
                next_name,
                ..
            } => {
                emit_sound();
                println!("\n🟡 ========================================================");
                println!("   REST INTERVAL: {}s!", rest_duration.as_secs());
                println!("   Catch your breath. Prepare for: {}", next_name);
                println!("========================================================\n");
            }
            RoutineEvent::Tick {
                phase,
                current_name,
                remaining_interval,
                total_remaining,
            } => {
                let phase_tag = match phase {
                    RoutinePhase::Work => "WORK",
                    RoutinePhase::Rest => "REST",
                    _ => "",
                };
                let rem_str = format_duration_digital(*remaining_interval, false);
                let total_str = format_duration_digital(*total_remaining, false);
                print!(
                    "\r[{:<4}] {:<18} | Interval Left: {:<6} | Total Left: {:<6}",
                    phase_tag, current_name, rem_str, total_str
                );
                let _ = io::stdout().flush();
            }
            RoutineEvent::Completed {
                total_time,
                total_steps,
            } => {
                emit_sound();
                println!("\n\n🏆 ========================================================");
                println!("   WORKOUT COMPLETE!");
                println!(
                    "   You finished all {} exercises in {}m {:02}s!",
                    total_steps,
                    total_time.as_secs() / 60,
                    total_time.as_secs() % 60
                );
                println!("========================================================\n");
                break;
            }
            _ => {}
        }

        thread::sleep(Duration::from_millis(250));
        if let Some(next_ev) = routine.tick()? {
            event = next_ev;
        }
    }

    Ok(())
}

fn run_concurrent_demo() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n========================================================");
    println!("     🔀  CONCURRENT MULTI-TIMER DEMONSTRATION  🔀      ");
    println!("========================================================");
    println!("Spawning 3 independent countdown timers on parallel threads:");
    println!(" - Timer 1 (Quick Sprint): 10 seconds");
    println!(" - Timer 2 (Mid Interval): 20 seconds");
    println!(" - Timer 3 (Long Period) : 30 seconds\n");

    let mut manager = ConcurrentTimerManager::new();
    manager.add_timer("Sprint (10s)", Duration::from_secs(10))?;
    manager.add_timer("Medium (20s)", Duration::from_secs(20))?;
    manager.add_timer("Long   (30s)", Duration::from_secs(30))?;

    manager.start_all();

    while !manager.all_expired() {
        manager.poll_updates();
        let snapshots = manager.get_snapshots();

        print!("\x1b[H\x1b[2J");
        println!("================ CONCURRENT TIMERS RUNNING ================");
        for (label, snap_opt) in &snapshots {
            if let Some(snap) = snap_opt {
                let time_str = format_duration_digital(snap.remaining, false);
                let bar = render_progress_bar(snap.progress, 20);
                let state_str = snap.state.to_string();
                println!(
                    "[{:<12}] {:<8} {} {:>5.1}% [{}]",
                    label,
                    time_str,
                    bar,
                    snap.progress * 100.0,
                    state_str
                );
            }
        }
        println!("===========================================================");
        let _ = io::stdout().flush();
        thread::sleep(Duration::from_millis(100));
    }

    emit_sound();
    println!("\n🔔 All concurrent timers have completed!\n");
    Ok(())
}
