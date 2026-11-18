use crate::error::TimerError;
use crate::timer::{parse_duration, CountdownTimer, TimerConfig, TimerState};
use std::fmt::Write as FmtWrite;
use std::time::Duration;

/// An individual exercise step within a programmed sequential routine.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExerciseStep {
    pub name: String,
    pub work_duration: Duration,
    pub rest_duration: Duration,
}

impl ExerciseStep {
    pub fn new(name: impl Into<String>, work_secs: u64, rest_secs: u64) -> Self {
        Self {
            name: name.into(),
            work_duration: Duration::from_secs(work_secs),
            rest_duration: Duration::from_secs(rest_secs),
        }
    }
}

/// The current phase of the routine.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RoutinePhase {
    NotStarted,
    Work,
    Rest,
    Completed,
    Paused,
}

/// Events emitted during routine execution for announcements and sound cues.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RoutineEvent {
    /// Work interval started for an exercise.
    WorkStarted {
        index: usize,
        total: usize,
        name: String,
        duration: Duration,
        next_name: Option<String>,
    },
    /// Work interval finished, rest interval began.
    RestStarted {
        index: usize,
        rest_duration: Duration,
        next_name: String,
    },
    /// Periodic tick during active interval.
    Tick {
        phase: RoutinePhase,
        current_name: String,
        remaining_interval: Duration,
        total_remaining: Duration,
    },
    /// Routine was paused.
    Paused,
    /// Routine was resumed.
    Resumed,
    /// Entire exercise routine completed.
    Completed {
        total_time: Duration,
        total_steps: usize,
    },
}

/// Sequential programmed exercise routine manager.
pub struct ExerciseRoutine {
    pub steps: Vec<ExerciseStep>,
    pub current_index: usize,
    pub phase: RoutinePhase,
    active_timer: Option<CountdownTimer>,
    last_phase_before_pause: RoutinePhase,
}

impl ExerciseRoutine {
    /// Creates a routine from a vector of exercise steps.
    pub fn new(steps: Vec<ExerciseStep>) -> Result<Self, TimerError> {
        if steps.is_empty() {
            return Err(TimerError::InvalidFormat("Routine cannot be empty".into()));
        }
        Ok(Self {
            steps,
            current_index: 0,
            phase: RoutinePhase::NotStarted,
            active_timer: None,
            last_phase_before_pause: RoutinePhase::NotStarted,
        })
    }

    /// Builds the standard 11-exercise scientific routine specified in the assignment PDF.
    pub fn default_assignment_routine() -> Self {
        let steps = vec![
            ExerciseStep::new("Jumping Jacks", 30, 5),
            ExerciseStep::new("Wall Sit", 30, 5),
            ExerciseStep::new("Push-ups", 45, 15),
            ExerciseStep::new("Crunches", 45, 15),
            ExerciseStep::new("Squats", 45, 10),
            ExerciseStep::new("Dips", 30, 5),
            ExerciseStep::new("Plank", 60, 10),
            ExerciseStep::new("Spot running", 60, 30),
            ExerciseStep::new("Lunges", 30, 5),
            ExerciseStep::new("Side Plank Left", 20, 5),
            ExerciseStep::new("Side Plank Right", 20, 0),
        ];
        Self::new(steps).expect("Default routine is valid")
    }

    /// Parses a tabular string of exercises, e.g.:
    /// "Jumping Jacks\t30s\t5s\nWall Sit\t30s\t5s"
    pub fn from_table_str(table: &str) -> Result<Self, TimerError> {
        let mut steps = Vec::new();
        for line in table.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty()
                || trimmed.starts_with('#')
                || trimmed.eq_ignore_ascii_case("Exercise Duration Rest")
            {
                continue;
            }

            // Split by tab, comma, or multiple spaces
            let parts: Vec<&str> = if trimmed.contains('\t') {
                trimmed.split('\t').map(|s| s.trim()).filter(|s| !s.is_empty()).collect()
            } else if trimmed.contains(',') {
                trimmed.split(',').map(|s| s.trim()).filter(|s| !s.is_empty()).collect()
            } else {
                // Heuristic: split last two tokens as duration and rest
                let words: Vec<&str> = trimmed.split_whitespace().collect();
                if words.len() < 3 {
                    continue;
                }
                let rest_str = words[words.len() - 1];
                let dur_str = words[words.len() - 2];
                let name = words[..words.len() - 2].join(" ");
                let work_dur = parse_duration(dur_str)?;
                let rest_dur = parse_duration(rest_str)?;
                steps.push(ExerciseStep {
                    name,
                    work_duration: work_dur,
                    rest_duration: rest_dur,
                });
                continue;
            };

            if parts.len() >= 3 {
                let name = parts[0].to_string();
                let work_dur = parse_duration(parts[1])?;
                let rest_dur = parse_duration(parts[2])?;
                steps.push(ExerciseStep {
                    name,
                    work_duration: work_dur,
                    rest_duration: rest_dur,
                });
            }
        }

        Self::new(steps)
    }

    /// Computes the total combined duration (work + rest) of the entire routine.
    pub fn total_routine_duration(&self) -> Duration {
        self.steps
            .iter()
            .map(|s| s.work_duration + s.rest_duration)
            .sum()
    }

    /// Formats the routine as a clean ASCII table.
    pub fn format_table(&self) -> String {
        let mut out = String::new();
        let _ = writeln!(
            out,
            "┌────┬──────────────────────┬──────────┬──────────┬────────┐"
        );
        let _ = writeln!(
            out,
            "│ #  │ Exercise             │ Duration │ Rest     │ Status │"
        );
        let _ = writeln!(
            out,
            "├────┼──────────────────────┼──────────┼──────────┼────────┤"
        );

        for (idx, step) in self.steps.iter().enumerate() {
            let status = if idx < self.current_index {
                "DONE"
            } else if idx == self.current_index {
                match self.phase {
                    RoutinePhase::NotStarted => "QUEUED",
                    RoutinePhase::Work => "WORK >>",
                    RoutinePhase::Rest => "REST ..",
                    RoutinePhase::Paused => "PAUSED",
                    RoutinePhase::Completed => "DONE",
                }
            } else {
                "QUEUED"
            };

            let dur_str = format!("{}s", step.work_duration.as_secs());
            let rest_str = format!("{}s", step.rest_duration.as_secs());

            let _ = writeln!(
                out,
                "│ {:<2} │ {:<20} │ {:<8} │ {:<8} │ {:<6} │",
                idx + 1,
                step.name,
                dur_str,
                rest_str,
                status
            );
        }

        let _ = writeln!(
            out,
            "└────┴──────────────────────┴──────────┴──────────┴────────┘"
        );
        let total_secs = self.total_routine_duration().as_secs();
        let _ = writeln!(
            out,
            "Total Routine Duration: {}m {:02}s across {} exercises",
            total_secs / 60,
            total_secs % 60,
            self.steps.len()
        );
        out
    }

    /// Starts or advances the routine, returning the initial event.
    pub fn start(&mut self) -> Result<RoutineEvent, TimerError> {
        self.current_index = 0;
        self.start_work_phase()
    }

    fn start_work_phase(&mut self) -> Result<RoutineEvent, TimerError> {
        let step = &self.steps[self.current_index];
        let config = TimerConfig::new(step.work_duration)?;
        let mut timer = CountdownTimer::new(config);
        timer.start()?;
        self.active_timer = Some(timer);
        self.phase = RoutinePhase::Work;

        let next_name = if self.current_index + 1 < self.steps.len() {
            Some(self.steps[self.current_index + 1].name.clone())
        } else {
            None
        };

        Ok(RoutineEvent::WorkStarted {
            index: self.current_index,
            total: self.steps.len(),
            name: step.name.clone(),
            duration: step.work_duration,
            next_name,
        })
    }

    fn start_rest_phase(&mut self) -> Result<Option<RoutineEvent>, TimerError> {
        let step = &self.steps[self.current_index];
        if step.rest_duration.is_zero() {
            // No rest, immediately advance to next exercise
            self.current_index += 1;
            if self.current_index >= self.steps.len() {
                self.phase = RoutinePhase::Completed;
                self.active_timer = None;
                return Ok(Some(RoutineEvent::Completed {
                    total_time: self.total_routine_duration(),
                    total_steps: self.steps.len(),
                }));
            }
            return self.start_work_phase().map(Some);
        }

        let config = TimerConfig::new(step.rest_duration)?;
        let mut timer = CountdownTimer::new(config);
        timer.start()?;
        self.active_timer = Some(timer);
        self.phase = RoutinePhase::Rest;

        let next_name = if self.current_index + 1 < self.steps.len() {
            self.steps[self.current_index + 1].name.clone()
        } else {
            "Workout Finished".to_string()
        };

        Ok(Some(RoutineEvent::RestStarted {
            index: self.current_index,
            rest_duration: step.rest_duration,
            next_name,
        }))
    }

    /// Toggles pause on the active interval.
    pub fn toggle_pause(&mut self) -> Result<RoutineEvent, TimerError> {
        if let Some(timer) = self.active_timer.as_mut() {
            match timer.state() {
                TimerState::Running => {
                    timer.pause()?;
                    self.last_phase_before_pause = self.phase;
                    self.phase = RoutinePhase::Paused;
                    Ok(RoutineEvent::Paused)
                }
                TimerState::Paused => {
                    timer.resume()?;
                    self.phase = self.last_phase_before_pause;
                    Ok(RoutineEvent::Resumed)
                }
                _ => Err(TimerError::InvalidStateTransition {
                    from: self.phase_name().into(),
                    action: "toggle_pause",
                }),
            }
        } else {
            Err(TimerError::InvalidStateTransition {
                from: "No active timer".into(),
                action: "toggle_pause",
            })
        }
    }

    /// Ticks the active timer and advances phases automatically when countdown reaches 0.
    pub fn tick(&mut self) -> Result<Option<RoutineEvent>, TimerError> {
        if self.phase == RoutinePhase::Paused
            || self.phase == RoutinePhase::NotStarted
            || self.phase == RoutinePhase::Completed
        {
            return Ok(None);
        }

        let is_expired = if let Some(timer) = self.active_timer.as_mut() {
            timer.remaining().is_zero()
        } else {
            false
        };

        if is_expired {
            match self.phase {
                RoutinePhase::Work => {
                    // Work ended -> Rest phase
                    return self.start_rest_phase();
                }
                RoutinePhase::Rest => {
                    // Rest ended -> Next work phase
                    self.current_index += 1;
                    if self.current_index >= self.steps.len() {
                        self.phase = RoutinePhase::Completed;
                        self.active_timer = None;
                        return Ok(Some(RoutineEvent::Completed {
                            total_time: self.total_routine_duration(),
                            total_steps: self.steps.len(),
                        }));
                    }
                    return self.start_work_phase().map(Some);
                }
                _ => {}
            }
        }

        // Return a tick event with current interval and total remaining
        if let Some(timer) = self.active_timer.as_mut() {
            let remaining_interval = timer.remaining();
            let current_name = self.steps[self.current_index].name.clone();
            let total_rem = self.calculate_total_remaining(remaining_interval);

            Ok(Some(RoutineEvent::Tick {
                phase: self.phase,
                current_name,
                remaining_interval,
                total_remaining: total_rem,
            }))
        } else {
            Ok(None)
        }
    }

    fn calculate_total_remaining(&self, current_interval_remaining: Duration) -> Duration {
        let mut rem = current_interval_remaining;
        // If in work phase, add rest duration for current step
        if self.phase == RoutinePhase::Work {
            rem += self.steps[self.current_index].rest_duration;
        }
        // Add all future steps
        for step in &self.steps[(self.current_index + 1)..] {
            rem += step.work_duration + step.rest_duration;
        }
        rem
    }

    pub fn phase_name(&self) -> &'static str {
        match self.phase {
            RoutinePhase::NotStarted => "NOT STARTED",
            RoutinePhase::Work => "WORK",
            RoutinePhase::Rest => "REST",
            RoutinePhase::Completed => "COMPLETED",
            RoutinePhase::Paused => "PAUSED",
        }
    }
}
