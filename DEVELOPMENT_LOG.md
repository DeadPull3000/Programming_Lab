# Development and Build Log — CS2102 Master Mind

This document maintains an evidence-based record of the toolchain environment, build executions, testing results, diagnostic logs, bug resolutions, and reproduction steps for the CS2102 Programming Lab mid-term project at Ashoka University.

---

## 1. Verified Toolchain and Environment

The following system and toolchain information was gathered directly via terminal queries in the active environment:

| Property | Value | Verification Command |
|---|---|---|
| **Operating System** | Windows 11 (64-bit) | System metadata |
| **Shell Environment** | Windows PowerShell | Active terminal environment |
| **Rust Compiler** | `rustc 1.99.0 (b940084d7 2026-09-28)` | `rustc --version` |
| **Cargo Package Manager** | `cargo 1.99.0 (5f94df478 2026-08-27)` | `cargo --version` |
| **Rust Edition** | `2024` | Configured in `Cargo.toml` |
| **Package Name & Version** | `mastermind v0.1.0` | Configured in `Cargo.toml` |
| **Primary Dependency** | `rand v0.10.3` | Locked in `Cargo.lock` |
| **Transitive Dependencies** | `chacha20 v0.10.2`, `getrandom v0.4.3`, `rand_core v0.10.1`, `cfg-if v1.0.5`, `cpufeatures v0.3.1`, `libc v0.2.190`, `r-efi v6.0.0` | Locked in `Cargo.lock` |

---

## 2. Command Execution and Verification Audit

Every command in the table below was executed in the workspace directory (`c:\Users\User\Downloads\Programming Lab\Midterm_Project\mastermind`).

| Command | Purpose | Actual Outcome | Notes |
|---|---|---|---|
| `cargo fmt --check` | Verify adherence to official Rust formatting rules | **Exit Code 0**; No output | Entire codebase complies with standard formatting. |
| `cargo check` | Fast static analysis, syntax check, and type checking | **Exit Code 0**; Finished `dev` profile in 0.05s | Zero compilation errors; zero compiler warnings. |
| `cargo test` | Execute automated unit and property-based tests | **Exit Code 0**; 6 passed, 0 failed in 0.06s | Tests: 2 scoring, 1 property (10,000 iterations), 2 parser, 1 target generation. |
| `cargo build` | Compile unoptimized debug binary with debug symbols | **Exit Code 0**; Finished `dev` profile in 0.05s | Debug binary produced at `target/debug/mastermind.exe`. |
| `cargo build --release` | Compile fully optimized production binary | **Exit Code 0**; Finished `release` profile in 0.04s | Release binary produced at `target/release/mastermind.exe`. |
| `"quit" \| cargo run --quiet` | Verify banner display, prompt flushing, and graceful quit | **Exit Code 0**; Displayed banner, accepted quit, printed exit message | Confirms terminal I/O and quit command functionality. |
| `"1 2 7`nquit" \| cargo run --quiet` | Verify input parser error handling and re-prompting | **Exit Code 0**; Displayed invalid guess warning, re-prompted, quit cleanly | Confirmed non-punitive error handling in terminal execution. |

---

## 3. Real Terminal Outputs

### Static Analysis (`cargo check`)
```text
C:\Users\User\Downloads\Programming Lab\Midterm_Project\mastermind> cargo check
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.05s
```

### Automated Test Execution (`cargo test`)
```text
C:\Users\User\Downloads\Programming Lab\Midterm_Project\mastermind> cargo test
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.05s
     Running unittests src\main.rs (target\debug\deps\mastermind-2ea5f93c76ebfba8.exe)

running 6 tests
test tests::parser_rejects_invalid_guesses ... ok
test tests::scores_basic_cases_correctly ... ok
test tests::scores_repeated_numbers_correctly ... ok
test tests::parser_accepts_whitespace_and_repetitions ... ok
test tests::generated_targets_have_correct_length_and_range ... ok
test tests::randomized_results_match_reference_implementation ... ok

test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.06s
```

### Release Build Compilation (`cargo build --release`)
```text
C:\Users\User\Downloads\Programming Lab\Midterm_Project\mastermind> cargo build --release
   Compiling rand_core v0.10.1
   Compiling getrandom v0.4.3
   Compiling cfg-if v1.0.5
   Compiling cpufeatures v0.3.1
   Compiling chacha20 v0.10.2
   Compiling rand v0.10.3
   Compiling mastermind v0.1.0 (C:\Users\User\Downloads\Programming Lab\Midterm_Project\mastermind)
    Finished `release` profile [optimized] target(s) in 4.34s
```

### Piped Stdin Interactive Run
```text
C:\Users\User\Downloads\Programming Lab\Midterm_Project\mastermind> "quit" | cargo run --quiet
====================================
       MASTER MIND: BULLS & COWS
====================================
I'm thinking of 4 numbers, each between 1 and 6.
Numbers may repeat.
A Bull means the number and position are correct.
A Cow means the number is correct but the position is not.
Type 'quit' during a guess to exit.

A new secret code has been selected.

Enter 4 numbers from 1 to 6, separated by spaces or tabs (or type 'quit'): You chose to quit the current game.

Thanks for playing!
```

---

## 4. Diagnostic and Bug Resolution History

### Issue 1: Scope Resolution Error for `ask_play_again`
- **Timestamp**: 2026-10-12T11:36:52+05:30
- **Diagnostic Message**:
  ```text
  error[E0425]: cannot find function `ask_play_again` in this scope
     --> src\main.rs:208:13
      |
  208 |         if !ask_play_again() {
      |             ^^^^^^^^^^^^^^ not found in this scope
  ```
- **Investigation**: Inspection of `src/main.rs` revealed that `fn ask_play_again()` had been accidentally enclosed within `fn play_game(target: &[u32]) -> bool` due to a missing closing brace after `play_game`'s inner `loop`. Because items declared inside a function body are private to that block's scope, `main()` was unable to resolve `ask_play_again`.
- **Resolution**:
  1. Inserted the closing brace `}` after `play_game`'s loop to terminate `play_game`.
  2. Moved `fn ask_play_again() -> bool` out to module scope.
  3. Re-aligned indentation to adhere to Rust formatting conventions.
- **Verification**: Executed `cargo check` (exited with code 0) and `cargo test` (all 6 tests passed).

### Issue 2: Compiler Warning for Dead Code on `play_game`
- **Diagnostic Message**:
  ```text
  warning: function `play_game` is never used
    --> src\main.rs:99:4
     |
  99 | fn play_game(target: &[u32]) -> bool {
  ```
- **Investigation**: Occurred before `main()` was connected to execute `play_game(&target)`.
- **Resolution**: Connecting `main()` to invoke `play_game(&target)` within the replay loop completely resolved the warning.
- **Verification**: `cargo test` compiled with zero warnings.

---

## 5. Step-by-Step Reproduction Guide

To independently reproduce the build and test results on any machine with Rust installed:

### Step 1: Open Terminal and Navigate to Project
```bash
cd "c:\Users\User\Downloads\Programming Lab\Midterm_Project\mastermind"
```

### Step 2: Validate Code Formatting
```bash
cargo fmt --check
```
*Expected Result*: Silent return with exit code 0.

### Step 3: Run Compiler Check
```bash
cargo check
```
*Expected Result*: `Finished dev profile` with zero warnings or errors.

### Step 4: Run the Test Suite
```bash
cargo test
```
*Expected Result*: `test result: ok. 6 passed; 0 failed`.

### Step 5: Build Debug and Release Artifacts
```bash
cargo build
cargo build --release
```
*Expected Result*: Successful compilation of debug and release binaries.

### Step 6: Launch Interactive Game
```bash
cargo run
```
*Expected Result*: Interactive Master Mind game prompt in the terminal.

---

## 6. Recommendations for Future Work

The following items are optional extensions for future project iterations (not currently implemented):
1. **Automated CI Workflow**: Establish a GitHub Actions pipeline running `cargo fmt --check`, `cargo clippy`, and `cargo test` on every push.
2. **Clippy Linting**: Run `cargo clippy -- -D warnings` for deeper idiomatic analysis.
3. **Property Testing with `proptest`**: Transition property-based testing to the official `proptest` crate for automated test case shrinking on failure.
