use crate::error::TimerError;
use crate::timer::{CountdownTimer, TimerConfig, TimerSnapshot, TimerState};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{channel, Receiver, RecvTimeoutError, Sender};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::Duration;

/// Commands sent from client/UI to the concurrent timer worker thread.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TimerCommand {
    Start,
    Pause,
    Resume,
    TogglePause,
    Reset,
    ResetWithDuration(Duration),
    AdjustSeconds(i64),
    Stop,
    Shutdown,
}

/// Events emitted from the timer worker thread back to the listener/UI.
#[derive(Debug, Clone, PartialEq)]
pub enum TimerEvent {
    Started,
    Tick(TimerSnapshot),
    Paused,
    Resumed,
    Expired(TimerSnapshot),
    Reset,
    Stopped,
    Terminated,
}

/// A concurrent background runner that executes a CountdownTimer on its own thread.
///
/// Implements Rust Book message-passing concurrency using `std::sync::mpsc::channel`.
pub struct ConcurrentRunner {
    command_tx: Sender<TimerCommand>,
    event_rx: Receiver<TimerEvent>,
    worker_handle: Option<JoinHandle<()>>,
    is_running: Arc<AtomicBool>,
}

impl ConcurrentRunner {
    /// Spawns a new concurrent timer worker thread with the given configuration.
    pub fn spawn(config: TimerConfig) -> Self {
        let (cmd_tx, cmd_rx) = channel::<TimerCommand>();
        let (event_tx, event_rx) = channel::<TimerEvent>();
        let is_running = Arc::new(AtomicBool::new(true));
        let is_running_clone = Arc::clone(&is_running);

        let handle = thread::spawn(move || {
            let mut timer = CountdownTimer::new(config);
            let tick_interval = timer.tick_interval();
            let mut last_expired_signaled = false;

            while is_running_clone.load(Ordering::Relaxed) {
                // Wait for commands or timeout at tick interval
                match cmd_rx.recv_timeout(tick_interval) {
                    Ok(TimerCommand::Start) => {
                        if timer.start().is_ok() {
                            last_expired_signaled = false;
                            let _ = event_tx.send(TimerEvent::Started);
                        }
                    }
                    Ok(TimerCommand::Pause) => {
                        if timer.pause().is_ok() {
                            let _ = event_tx.send(TimerEvent::Paused);
                        }
                    }
                    Ok(TimerCommand::Resume) => {
                        if timer.resume().is_ok() {
                            let _ = event_tx.send(TimerEvent::Resumed);
                        }
                    }
                    Ok(TimerCommand::TogglePause) => {
                        if let Ok(new_state) = timer.toggle_pause() {
                            match new_state {
                                TimerState::Running => {
                                    let _ = event_tx.send(TimerEvent::Resumed);
                                }
                                TimerState::Paused => {
                                    let _ = event_tx.send(TimerEvent::Paused);
                                }
                                _ => {}
                            }
                        }
                    }
                    Ok(TimerCommand::Reset) => {
                        timer.reset();
                        last_expired_signaled = false;
                        let _ = event_tx.send(TimerEvent::Reset);
                    }
                    Ok(TimerCommand::ResetWithDuration(d)) => {
                        if timer.reset_with_duration(d).is_ok() {
                            last_expired_signaled = false;
                            let _ = event_tx.send(TimerEvent::Reset);
                        }
                    }
                    Ok(TimerCommand::AdjustSeconds(secs)) => {
                        timer.adjust_seconds(secs);
                    }
                    Ok(TimerCommand::Stop) => {
                        timer.stop();
                        let _ = event_tx.send(TimerEvent::Stopped);
                    }
                    Ok(TimerCommand::Shutdown) => {
                        break;
                    }
                    Err(RecvTimeoutError::Timeout) => {
                        // Regular tick timeout
                    }
                    Err(RecvTimeoutError::Disconnected) => {
                        // Channel dropped by sender
                        break;
                    }
                }

                // If timer is active or just expired, send snapshot
                let snapshot = timer.snapshot();
                if snapshot.is_expired() && !last_expired_signaled {
                    last_expired_signaled = true;
                    let _ = event_tx.send(TimerEvent::Expired(snapshot.clone()));
                } else if snapshot.is_running() || snapshot.is_paused() {
                    let _ = event_tx.send(TimerEvent::Tick(snapshot));
                }
            }

            let _ = event_tx.send(TimerEvent::Terminated);
        });

        Self {
            command_tx: cmd_tx,
            event_rx,
            worker_handle: Some(handle),
            is_running,
        }
    }

    /// Sends a command to the background worker thread.
    pub fn send_command(&self, cmd: TimerCommand) -> Result<(), TimerError> {
        self.command_tx
            .send(cmd)
            .map_err(|_| TimerError::ChannelDisconnected)
    }

    /// Attempts to read the next event without blocking.
    pub fn try_recv_event(&self) -> Option<TimerEvent> {
        self.event_rx.try_recv().ok()
    }

    /// Receives the next event, waiting up to `timeout`.
    pub fn recv_event_timeout(&self, timeout: Duration) -> Result<TimerEvent, RecvTimeoutError> {
        self.event_rx.recv_timeout(timeout)
    }

    /// Shuts down the background thread and waits for it to join cleanly.
    pub fn shutdown(mut self) {
        self.is_running.store(false, Ordering::Relaxed);
        let _ = self.command_tx.send(TimerCommand::Shutdown);
        if let Some(handle) = self.worker_handle.take() {
            let _ = handle.join();
        }
    }
}

impl Drop for ConcurrentRunner {
    fn drop(&mut self) {
        self.is_running.store(false, Ordering::Relaxed);
        let _ = self.command_tx.send(TimerCommand::Shutdown);
        if let Some(handle) = self.worker_handle.take() {
            let _ = handle.join();
        }
    }
}

/// A concurrent multi-timer manager that coordinates multiple concurrent timers running in parallel.
///
/// Demonstrates shared-state concurrency (`Arc<Mutex<...>>`) and multi-threaded synchronization.
pub struct ConcurrentTimerManager {
    timers: Vec<(String, ConcurrentRunner)>,
    snapshots: Arc<Mutex<Vec<(String, Option<TimerSnapshot>)>>>,
}

impl ConcurrentTimerManager {
    pub fn new() -> Self {
        Self {
            timers: Vec::new(),
            snapshots: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Adds a new named timer running concurrently on its own thread.
    pub fn add_timer(&mut self, label: impl Into<String>, duration: Duration) -> Result<usize, TimerError> {
        let label = label.into();
        let config = TimerConfig::new(duration)?;
        let runner = ConcurrentRunner::spawn(config);
        let index = self.timers.len();

        let mut snaps = self.snapshots.lock().unwrap();
        snaps.push((label.clone(), None));
        drop(snaps);

        self.timers.push((label, runner));
        Ok(index)
    }

    /// Starts all concurrent timers simultaneously.
    pub fn start_all(&self) {
        for (_, runner) in &self.timers {
            let _ = runner.send_command(TimerCommand::Start);
        }
    }

    /// Pauses all concurrent timers.
    pub fn pause_all(&self) {
        for (_, runner) in &self.timers {
            let _ = runner.send_command(TimerCommand::Pause);
        }
    }

    /// Resumes all concurrent timers.
    pub fn resume_all(&self) {
        for (_, runner) in &self.timers {
            let _ = runner.send_command(TimerCommand::Resume);
        }
    }

    /// Polls updates from all background timer threads and updates snapshots.
    pub fn poll_updates(&mut self) {
        let mut snaps = self.snapshots.lock().unwrap();
        for (idx, (_, runner)) in self.timers.iter().enumerate() {
            while let Some(event) = runner.try_recv_event() {
                match event {
                    TimerEvent::Tick(snap) => {
                        snaps[idx].1 = Some(snap);
                    }
                    TimerEvent::Expired(snap) => {
                        snaps[idx].1 = Some(snap);
                    }
                    _ => {}
                }
            }
        }
    }

    /// Returns the live snapshots of all concurrent timers.
    pub fn get_snapshots(&self) -> Vec<(String, Option<TimerSnapshot>)> {
        self.snapshots.lock().unwrap().clone()
    }

    /// Checks if all concurrent timers have expired.
    pub fn all_expired(&self) -> bool {
        let snaps = self.snapshots.lock().unwrap();
        if snaps.is_empty() {
            return false;
        }
        snaps.iter().all(|(_, snap)| {
            snap.as_ref()
                .map(|s| s.is_expired())
                .unwrap_or(false)
        })
    }
}

impl Default for ConcurrentTimerManager {
    fn default() -> Self {
        Self::new()
    }
}
