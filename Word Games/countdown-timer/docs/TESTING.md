# Verification and Testing Specification: Countdown Timer

## 1. Test Strategy & Architecture

The testing harness validates every layer of the countdown timer system:
1. **Deterministic Mock Clock Testing**: Verifies state transitions and mathematical invariants with zero real-time waiting using `MockClock`.
2. **Stress & Rapid Pause/Resume**: Simulates rapid interleaving of active and paused intervals to ensure time accumulation is strictly lossless.
3. **Format Parsing Coverage**: Validates pure seconds, compound unit strings (`1h20m30s`), colon formats (`MM:SS`, `HH:MM:SS`), float decimals (`1.5s`), zero-duration rejection, and negative format rejections.
4. **Sequential Routine Phase Validation**: Verifies phase cycling across all 11 specified exercises, total routine duration sum ($520\text{s}$), and automatic work $\to$ rest transitions.
5. **Multi-Threaded Concurrency Testing**: Tests background worker channel communication, graceful shutdown, and concurrent multi-timer management across multiple threads.

---

## 2. Test Suite Overview

| Test Identifier | Category | Invariant Verified |
|---|---|---|
| `test_parse_duration_valid_formats` | Parser | Correctly parses `45`, `180`, `30s`, `5m`, `1h`, `1m30s`, `03:00`, `01:15:30`, `1.5s` |
| `test_parse_duration_zero_and_invalid_formats` | Parser | Validates `0s`, empty inputs, non-numeric tokens, and config zero-duration errors |
| `test_timer_exceeds_max_duration` | Bounds | Rejects initial durations exceeding the 24-hour limit (`DEFAULT_MAX_DURATION`) |
| `test_drift_free_state_transitions_with_mock_clock` | Timing | Verifies Ready $\to$ Running $\to$ Paused $\to$ Running $\to$ Expired $\to$ Reset |
| `test_toggle_pause_behavior` | Controls | Verifies toggle transitions between Running and Paused states |
| `test_adjust_seconds_and_clamping` | Controls | Dynamically adds and subtracts seconds, clamping properly at zero and upper bounds |
| `test_rapid_pause_resume_cycles_preserve_accuracy` | Stress | Executes 10 rapid pause/resume cycles; verifies active time is preserved |
| `test_large_display_and_progress_formatting` | Display | Checks 5-line banner height, progress bar fill, and subsecond text |
| `test_hours_duration_formatting` | Display | Verifies digital clock formatting when duration spans multiple hours |
| `test_exercise_routine_specification_table` | Routine | Verifies exact assignment specifications: 11 exercises, durations, rest intervals, and total time |
| `test_exercise_routine_custom_table_parser` | Routine | Parses whitespace-separated tables and TSV exercise plans |
| `test_routine_automatic_phase_advancement` | Routine | Cycles through sequential exercises and rest periods to full completion |
| `test_reset_with_new_duration_and_bounds` | Controls | Reconfigures timer duration dynamically while validating bounds |
| `test_concurrent_runner_thread_channels` | Concurrency | Verifies worker thread spawn, mpsc command delivery, tick streaming, and shutdown |
| `test_concurrent_timer_manager_parallel_execution` | Concurrency | Manages multiple parallel countdown timers running simultaneously |

---

## 3. Reproduction & Execution

To execute the test suite:

```bash
# Navigate to the countdown-timer directory
cd countdown-timer

# Run all automated tests
cargo test

# Run tests with stdout output
cargo test -- --nocapture
```
