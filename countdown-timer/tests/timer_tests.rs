use countdown_timer::concurrency::{ConcurrentRunner, ConcurrentTimerManager, TimerCommand, TimerEvent};
use countdown_timer::display::{format_duration_digital, render_large_text, render_progress_bar};
use countdown_timer::error::TimerError;
use countdown_timer::routine::{ExerciseRoutine, ExerciseStep, RoutineEvent, RoutinePhase};
use countdown_timer::timer::{
    parse_duration, Clock, CountdownTimer, TimerConfig, TimerState, DEFAULT_MAX_DURATION,
};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

/// A deterministic mock clock for microsecond-precision testing without sleeping real time.
struct MockClock {
    base_instant: Instant,
    offset_nanos: AtomicU64,
}

impl MockClock {
    fn new() -> Self {
        Self {
            base_instant: Instant::now(),
            offset_nanos: AtomicU64::new(0),
        }
    }

    fn advance(&self, duration: Duration) {
        self.offset_nanos
            .fetch_add(duration.as_nanos() as u64, Ordering::SeqCst);
    }
}

impl Clock for MockClock {
    fn now(&self) -> Instant {
        let nanos = self.offset_nanos.load(Ordering::SeqCst);
        self.base_instant + Duration::from_nanos(nanos)
    }
}

#[test]
fn test_parse_duration_valid_formats() {
    // Pure seconds
    assert_eq!(parse_duration("45").unwrap(), Duration::from_secs(45));
    assert_eq!(parse_duration("180").unwrap(), Duration::from_secs(180));

    // Suffixes
    assert_eq!(parse_duration("30s").unwrap(), Duration::from_secs(30));
    assert_eq!(parse_duration("5m").unwrap(), Duration::from_secs(300));
    assert_eq!(parse_duration("1h").unwrap(), Duration::from_secs(3600));
    assert_eq!(parse_duration("1m30s").unwrap(), Duration::from_secs(90));
    assert_eq!(parse_duration("1h20m30s").unwrap(), Duration::from_secs(3600 + 1200 + 30));

    // Colon format
    assert_eq!(parse_duration("03:00").unwrap(), Duration::from_secs(180));
    assert_eq!(parse_duration("00:45").unwrap(), Duration::from_secs(45));
    assert_eq!(parse_duration("01:15:30").unwrap(), Duration::from_secs(3600 + 900 + 30));

    // Subsecond floats
    assert_eq!(parse_duration("1.5s").unwrap(), Duration::from_millis(1500));
    assert_eq!(parse_duration("0.5").unwrap(), Duration::from_millis(500));
}

#[test]
fn test_parse_duration_zero_and_invalid_formats() {
    assert_eq!(parse_duration("0").unwrap(), Duration::ZERO);
    assert_eq!(parse_duration("0s").unwrap(), Duration::ZERO);
    assert_eq!(parse_duration("").unwrap_err(), TimerError::InvalidFormat("Input is empty".into()));
    assert!(matches!(parse_duration("abc"), Err(TimerError::InvalidFormat(_))));
    assert!(matches!(parse_duration("10x"), Err(TimerError::InvalidFormat(_))));

    // TimerConfig rejects zero duration
    assert_eq!(TimerConfig::new(Duration::ZERO).unwrap_err(), TimerError::ZeroDuration);
}

#[test]
fn test_timer_exceeds_max_duration() {
    let too_long = DEFAULT_MAX_DURATION + Duration::from_secs(1);
    let res = TimerConfig::new(too_long);
    assert!(matches!(res, Err(TimerError::ExceedsMaximumDuration { .. })));
}

#[test]
fn test_drift_free_state_transitions_with_mock_clock() {
    let clock = Arc::new(MockClock::new());
    let config = TimerConfig::new(Duration::from_secs(60)).unwrap();
    let mut timer = CountdownTimer::with_clock(config, clock.clone());

    // Initially Ready
    assert_eq!(timer.state(), TimerState::Ready);
    assert_eq!(timer.remaining(), Duration::from_secs(60));

    // Start timer
    timer.start().unwrap();
    assert_eq!(timer.state(), TimerState::Running);

    // Advance mock time by 15 seconds
    clock.advance(Duration::from_secs(15));
    assert_eq!(timer.remaining(), Duration::from_secs(45));

    // Pause timer
    timer.pause().unwrap();
    assert_eq!(timer.state(), TimerState::Paused);
    assert_eq!(timer.remaining(), Duration::from_secs(45));

    // Advance mock time by 30 seconds while paused - remaining time MUST remain frozen!
    clock.advance(Duration::from_secs(30));
    assert_eq!(timer.remaining(), Duration::from_secs(45));
    assert_eq!(timer.state(), TimerState::Paused);

    // Resume timer
    timer.resume().unwrap();
    assert_eq!(timer.state(), TimerState::Running);

    // Advance mock time by 40 seconds
    clock.advance(Duration::from_secs(40));
    assert_eq!(timer.remaining(), Duration::from_secs(5));

    // Advance 10 more seconds to trigger expiration
    clock.advance(Duration::from_secs(10));
    assert_eq!(timer.remaining(), Duration::ZERO);
    assert_eq!(timer.state(), TimerState::Expired);

    // Reset restores ready state
    timer.reset();
    assert_eq!(timer.state(), TimerState::Ready);
    assert_eq!(timer.remaining(), Duration::from_secs(60));
}

#[test]
fn test_toggle_pause_behavior() {
    let clock = Arc::new(MockClock::new());
    let config = TimerConfig::new(Duration::from_secs(10)).unwrap();
    let mut timer = CountdownTimer::with_clock(config, clock);

    assert_eq!(timer.toggle_pause().unwrap(), TimerState::Running);
    assert_eq!(timer.state(), TimerState::Running);

    assert_eq!(timer.toggle_pause().unwrap(), TimerState::Paused);
    assert_eq!(timer.state(), TimerState::Paused);

    assert_eq!(timer.toggle_pause().unwrap(), TimerState::Running);
    assert_eq!(timer.state(), TimerState::Running);
}

#[test]
fn test_adjust_seconds_and_clamping() {
    let clock = Arc::new(MockClock::new());
    let config = TimerConfig::new(Duration::from_secs(30)).unwrap();
    let mut timer = CountdownTimer::with_clock(config, clock.clone());

    timer.start().unwrap();
    clock.advance(Duration::from_secs(10));
    assert_eq!(timer.remaining(), Duration::from_secs(20));

    // Add 15 seconds
    timer.adjust_seconds(15);
    assert_eq!(timer.remaining(), Duration::from_secs(35));

    // Subtract 40 seconds (should clamp to 0 and expire)
    timer.adjust_seconds(-40);
    assert_eq!(timer.remaining(), Duration::ZERO);
    assert_eq!(timer.state(), TimerState::Expired);
}

#[test]
fn test_large_display_and_progress_formatting() {
    let text = "03:45";
    let rendered = render_large_text(text);
    assert_eq!(rendered.lines().count(), 5);

    let progress_bar = render_progress_bar(0.5, 20);
    assert_eq!(progress_bar, "[██████████░░░░░░░░░░]");

    let formatted_normal = format_duration_digital(Duration::from_secs(185), false);
    assert_eq!(formatted_normal, "03:05");

    let formatted_sub = format_duration_digital(Duration::from_millis(185_400), true);
    assert_eq!(formatted_sub, "03:05.4");
}

#[test]
fn test_exercise_routine_specification_table() {
    let routine = ExerciseRoutine::default_assignment_routine();
    assert_eq!(routine.steps.len(), 11);

    // Verify all 11 exercises from assignment PDF
    assert_eq!(routine.steps[0].name, "Jumping Jacks");
    assert_eq!(routine.steps[0].work_duration, Duration::from_secs(30));
    assert_eq!(routine.steps[0].rest_duration, Duration::from_secs(5));

    assert_eq!(routine.steps[1].name, "Wall Sit");
    assert_eq!(routine.steps[1].work_duration, Duration::from_secs(30));
    assert_eq!(routine.steps[1].rest_duration, Duration::from_secs(5));

    assert_eq!(routine.steps[2].name, "Push-ups");
    assert_eq!(routine.steps[2].work_duration, Duration::from_secs(45));
    assert_eq!(routine.steps[2].rest_duration, Duration::from_secs(15));

    assert_eq!(routine.steps[6].name, "Plank");
    assert_eq!(routine.steps[6].work_duration, Duration::from_secs(60));
    assert_eq!(routine.steps[6].rest_duration, Duration::from_secs(10));

    assert_eq!(routine.steps[7].name, "Spot running");
    assert_eq!(routine.steps[7].work_duration, Duration::from_secs(60));
    assert_eq!(routine.steps[7].rest_duration, Duration::from_secs(30));

    assert_eq!(routine.steps[10].name, "Side Plank Right");
    assert_eq!(routine.steps[10].work_duration, Duration::from_secs(20));
    assert_eq!(routine.steps[10].rest_duration, Duration::ZERO);

    // Total duration: 30+5 + 30+5 + 45+15 + 45+15 + 45+10 + 30+5 + 60+10 + 60+30 + 30+5 + 20+5 + 20+0
    // = 35 + 35 + 60 + 60 + 55 + 35 + 70 + 90 + 35 + 25 + 20 = 520s
    let total_secs = routine.total_routine_duration().as_secs();
    assert_eq!(total_secs, 520);
}

#[test]
fn test_exercise_routine_custom_table_parser() {
    let table = r#"
        Exercise Duration Rest
        Jumping Jacks 30s 5s
        Wall Sit 30s 5s
        Side Plank Right 20s 0s
    "#;
    let routine = ExerciseRoutine::from_table_str(table).unwrap();
    assert_eq!(routine.steps.len(), 3);
    assert_eq!(routine.steps[0].name, "Jumping Jacks");
    assert_eq!(routine.steps[0].work_duration, Duration::from_secs(30));
    assert_eq!(routine.steps[0].rest_duration, Duration::from_secs(5));
    assert_eq!(routine.steps[2].name, "Side Plank Right");
    assert_eq!(routine.steps[2].rest_duration, Duration::ZERO);
}

#[test]
fn test_concurrent_runner_thread_channels() {
    let config = TimerConfig::with_options(
        Duration::from_millis(500),
        Duration::from_millis(50),
        DEFAULT_MAX_DURATION,
    )
    .unwrap();

    let runner = ConcurrentRunner::spawn(config);
    runner.send_command(TimerCommand::Start).unwrap();

    // Expect started event or tick
    let mut saw_started_or_tick = false;
    let mut saw_expired = false;

    let deadline = Instant::now() + Duration::from_secs(2);
    while Instant::now() < deadline {
        if let Some(event) = runner.try_recv_event() {
            match event {
                TimerEvent::Started | TimerEvent::Tick(_) => {
                    saw_started_or_tick = true;
                }
                TimerEvent::Expired(_) => {
                    saw_expired = true;
                    break;
                }
                _ => {}
            }
        }
        thread::sleep(Duration::from_millis(20));
    }

    assert!(saw_started_or_tick, "Should receive start or tick event");
    assert!(saw_expired, "Should receive expiration event");
    runner.shutdown();
}

#[test]
fn test_concurrent_timer_manager_parallel_execution() {
    let mut manager = ConcurrentTimerManager::new();
    manager
        .add_timer("Timer A", Duration::from_millis(150))
        .unwrap();
    manager
        .add_timer("Timer B", Duration::from_millis(300))
        .unwrap();

    manager.start_all();

    let deadline = Instant::now() + Duration::from_secs(2);
    while Instant::now() < deadline && !manager.all_expired() {
        manager.poll_updates();
        thread::sleep(Duration::from_millis(30));
    }

    assert!(manager.all_expired(), "All concurrent timers should expire");
}

#[test]
fn test_rapid_pause_resume_cycles_preserve_accuracy() {
    let clock = Arc::new(MockClock::new());
    let config = TimerConfig::new(Duration::from_secs(100)).unwrap();
    let mut timer = CountdownTimer::with_clock(config, clock.clone());

    timer.start().unwrap();

    // 10 cycles of running 2s then pausing 5s
    for _ in 0..10 {
        clock.advance(Duration::from_secs(2));
        timer.pause().unwrap();
        clock.advance(Duration::from_secs(5)); // Paused time should NOT count
        timer.resume().unwrap();
    }

    // Total active run time should be exactly 20 seconds
    assert_eq!(timer.remaining(), Duration::from_secs(80));
    let snap = timer.snapshot();
    assert_eq!(snap.elapsed_active, Duration::from_secs(20));
    assert!((snap.progress - 0.20).abs() < 1e-4);
}

#[test]
fn test_routine_automatic_phase_advancement() {
    let mut routine = ExerciseRoutine::new(vec![
        ExerciseStep::new("Push-ups", 1, 1),
        ExerciseStep::new("Squats", 1, 0),
    ])
    .unwrap();

    // Start -> Work on Push-ups
    let ev = routine.start().unwrap();
    assert!(matches!(ev, RoutineEvent::WorkStarted { .. }));
    assert_eq!(routine.phase, RoutinePhase::Work);

    // Sleep 1.1s so work interval expires
    thread::sleep(Duration::from_millis(1100));

    // Tick -> should transition to Rest for Push-ups
    let rest_ev = routine.tick().unwrap().unwrap();
    assert!(matches!(rest_ev, RoutineEvent::RestStarted { .. }));
    assert_eq!(routine.phase, RoutinePhase::Rest);

    // Sleep 1.1s so rest interval expires
    thread::sleep(Duration::from_millis(1100));

    // Tick -> should transition to Squats work
    let work2_ev = routine.tick().unwrap().unwrap();
    assert!(matches!(work2_ev, RoutineEvent::WorkStarted { .. }));
    assert_eq!(routine.phase, RoutinePhase::Work);

    // Sleep 1.1s so Squats work expires (rest is 0, so should complete directly)
    thread::sleep(Duration::from_millis(1100));

    let comp_ev = routine.tick().unwrap().unwrap();
    assert!(matches!(comp_ev, RoutineEvent::Completed { .. }));
    assert_eq!(routine.phase, RoutinePhase::Completed);
}

#[test]
fn test_hours_duration_formatting() {
    let d = Duration::from_secs(3600 * 2 + 60 * 15 + 9);
    assert_eq!(format_duration_digital(d, false), "02:15:09");
    assert_eq!(format_duration_digital(d, true), "02:15:09.0");
}

#[test]
fn test_reset_with_new_duration_and_bounds() {
    let mut timer = CountdownTimer::new(TimerConfig::new(Duration::from_secs(10)).unwrap());
    timer.start().unwrap();

    // Reset with valid duration
    assert!(timer.reset_with_duration(Duration::from_secs(45)).is_ok());
    assert_eq!(timer.state(), TimerState::Ready);
    assert_eq!(timer.remaining(), Duration::from_secs(45));

    // Reset with zero rejected
    assert_eq!(
        timer.reset_with_duration(Duration::ZERO).unwrap_err(),
        TimerError::ZeroDuration
    );

    // Reset with too large rejected
    assert!(matches!(
        timer.reset_with_duration(DEFAULT_MAX_DURATION + Duration::from_secs(10)),
        Err(TimerError::ExceedsMaximumDuration { .. })
    ));
}
