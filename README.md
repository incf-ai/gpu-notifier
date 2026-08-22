Notice: this program is AI generated and was created as an educational exercise.

# gpu-notifier

gpu-notifier is a Rust background service that monitors GPU power usage and runs a notification command when configured thresholds are exceeded. It is designed to run as a userspace `systemd` unit at startup, read its configuration from a RON file once per minute, and expose configurable logging levels using standard Rust logging libraries.

## Overview

- Written in Rust
- Reads configuration from a RON file every 60 seconds
- Supports GPU monitoring based on parser-specific command output
- Uses standard logging libraries with configurable log levels
- Includes extensive inline comments and unit tests covering configuration and operational behavior
- Runs as a user-level `systemd` service

## Features

- Configurable per-GPU monitoring profiles
- Supports `AmdSmi` parser output from `amd-smi` JSON metrics
- Matches the configured GPU by `gpu_id`
- Evaluates `PowerBelow(N)` thresholds against `socket_power`
- Notifies by executing a configured shell command
- Supports optional notification grace counts for consecutive threshold violations

## Configuration

gpu-notifier loads a RON configuration file with a structure like this:

```ron
GpuNotifier(
    gpus: [
        PerGpuConfig(
            gpu_id: 0,
            enabled: true,
            parser: AmdSmi,
            monitor_interval: Seconds(60.0),
            monitor_command: "amd-smi metric --json",
            monitor_threshold: PowerBelow(50.0),
            monitor_grace: Some(1),
            notify_grace: Some(1),
            notify_command: "ffplay -v 0 -nodisp -autoexit ~/.config/gpu-notifier/notify.mp3",
        ),
        PerGpuConfig(
            gpu_id: 1,
            enabled: true,
            parser: AmdSmi,
            monitor_interval: Seconds(60.0),
            monitor_command: "amd-smi metric --json",
            monitor_threshold: PowerBelow(50.0),
            monitor_grace: Some(1),
            notify_grace: Some(1),
            notify_command: "ffplay -v 0 -nodisp -autoexit ~/.config/gpu-notifier/notify.mp3",
        ),
    ],
)
```

### Configuration semantics

- `gpu_id` selects the monitored GPU.
- `enabled` toggles monitoring for that GPU profile.
- `parser` selects the data source format. `AmdSmi` causes the program to execute the configured `monitor_command` and parse its JSON output.
- `monitor_interval` defines how often the configuration is reloaded and the monitor command is evaluated.
- `monitor_command` is executed as a shell command and must return AMD SMI JSON statistics.
- `monitor_threshold` currently supports `PowerBelow(N)` and evaluates the parsed GPU's `socket_power.value`.
- `monitor_grace` can be `None` or `Some(n)` where `n` is a non-zero `u64`. If `None`, the condition is evaluated immediately. If `Some(n)`, the threshold condition must be true for `n + 1` consecutive checks before the GPU is eligible to notify.
- `notify_grace` can be `None` or `Some(n)` where `n` is a non-zero `u64`. If `None`, the notification command is run every time all monitor conditions are met. If `Some(n)`, the notifier skips the notification command for the next `n` eligible notification events after a notification has been run.
- `notify_command` is executed when the configured threshold and grace criteria are satisfied.

## Expected amd-smi JSON format

gpu-notifier expects the `AmdSmi` parser to read output from `amd-smi metric --json` in a structure similar to:

```json
{
    "gpu_data": [
        {
            "gpu": 0,
            "power": {
                "socket_power": {
                    "value": 214,
                    "unit": "W"
                }
            }
        }
    ]
}
```

The program will search `gpu_data` for the configured `gpu_id`. If a matching GPU is not present in the output, the program will raise an error for that profile.

## Installation

### Dependencies

- Rust toolchain with Cargo
- `systemd` for user service management
- `amd-smi` available in `PATH` for AMD GPU monitoring
- A command capable of producing notifications, such as `ffplay`, `paplay`, or another local notification tool

### Build

From the repository root:

```bash
cargo build --release
```

### Install

Copy the built binary to a suitable location, for example:

```bash
install -Dm755 target/release/gpu-notifier /usr/local/bin/gpu-notifier
```

Create your configuration file, for example `~/.config/gpu-notifier/config.ron`.

## systemd unit example

Create a user-level systemd service file at `~/.config/systemd/user/gpu-notifier.service`:

```ini
[Unit]
Description=GPU Notifier
After=default.target

[Service]
Type=simple
ExecStart=/usr/local/bin/gpu-notifier --config /home/user/.config/gpu-notifier/config.ron
Restart=always
RestartSec=10
Environment=RUST_LOG=info

[Install]
WantedBy=default.target
```

Enable and start the service:

```bash
systemctl --user daemon-reload
systemctl --user enable --now gpu-notifier.service
```

## Logging

gpu-notifier uses a standard Rust logging backend and supports configurable log levels through the environment. For example:

```bash
RUST_LOG=debug /usr/local/bin/gpu-notifier --config ~/.config/gpu-notifier/config.ron
```

Typical log levels include:

- `error`
- `warn`
- `info`
- `debug`
- `trace`

## Testing

The crate is designed to include unit tests for:

- parsing RON configuration into idiomatic Rust types
- parsing `AmdSmi` JSON output across 0..N GPUs
- matching the configured GPU ID and extracting `socket_power`
- threshold logic for `PowerBelow`
- `monitor_grace` consecutive-match behavior
- notification command execution eligibility

### Unit test design

The unit tests are intended to validate each layer of the program independently. They are organized around:

- configuration parsing: ensure the RON input is deserialized to the expected Rust structures and that invalid or missing values fail cleanly
- parser behavior: verify `AmdSmi` JSON payloads are parsed correctly for zero, one, or many GPU entries, and that the configured `gpu_id` is matched reliably
- threshold evaluation: exercise `PowerBelow` comparisons against `socket_power` values and confirm notifications only become eligible when the threshold is met
- grace counting: confirm `monitor_grace` semantics for `None` and `Some(n)` cases, including the required consecutive-check behavior
- notification eligibility: check that the notify command is only considered when all configured criteria are satisfied and that disabled profiles are skipped

These tests are written as standard Rust unit tests so they can be run with `cargo test` and provide fast feedback on the program's configuration and monitoring logic.

Run tests with:

```bash
cargo test
```


## Architecture and files

The program is structured as a Rust binary crate with clear separation of concerns across modules.

- `src/bin/gpu-notifier.rs` - application entrypoint, runtime loop, configuration reload scheduling, and logging initialization.
- `src/config.rs` - RON configuration deserialization for `GpuNotifier`, `PerGpuConfig`, parser variants, thresholds, intervals, and notification settings.
- `src/config_reload.rs` - configuration reload timing, file reads, RON parsing, and module-specific error handling for reloading the active configuration.
- `src/amd_smi.rs` - AMD SMI parser implementation that parses the JSON payload, locates the matching `gpu_id`, and extracts `socket_power`.
- `src/monitor.rs` - execution of the configured `monitor_command`, conversion of command output into a parsed socket power value, and its module-specific error handling.
- `src/gpu_check.rs` - per-GPU evaluation of timing, monitor-command execution, threshold matching, and notify-command execution with module-specific error handling.
- `src/notify.rs` - notification decision logic, grace counting, and execution of the configured `notify_command`.
- `src/error.rs` - centralized application error definitions, with distinct error types split by module.

Each module uses a consistent error hierarchy with a top-level `Error` type and module-specific error enums. The errors follow this pattern:

```rust
#[derive(Debug, thiserror::Error)]
#[error("Error: {0}")]
pub struct Error(#[from] Error);

#[derive(Debug, thiserror::Error)]
enum ErrorKind {
    #[error("FloobError: {0}")]
    FloobError(#[from] FloobError),
    #[error("QuxError: {0}")]
    QuxError(#[from] QuxError),
    // ...
}

transitive_from::hierarchy! {
    Error {
        ErrorKind {
            FloobError,
            QuxError,
            // ...
        }
    }
}

#[derive(Debug, thiserror::Error)]
enum FloobError {
    #[error("SomeIssue1: {0}")]
    SomeIssue1(some_library::Error),
    #[error("SomeIssue2: {0}")]
    SomeIssue2(some_library::Error),
    // ...
}

#[derive(Debug, thiserror::Error)]
enum QuxError {
    #[error("SomeIssue1: {0}")]
    SomeIssue1(some_library::Error),
    #[error("SomeIssue2: {0}")]
    SomeIssue2(some_library::Error),
    // ...
}

pub fn whatever() -> Result<(), Error> {
    // ...
}
```

This layout ensures each module defines its own error domain, and distinct errors are split into separate modules so they can be composed cleanly by the top-level application layer.

## Notes

This README describes the intended design and operation of the `gpu-notifier` program. The configuration is reloaded on a one-minute interval and each GPU profile is evaluated against its configured thresholds and grace counts to decide when to execute a notification command.
