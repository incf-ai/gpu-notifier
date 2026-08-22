//! Notification decision and command execution logic.
use crate::config::{PerGpuConfig, Threshold};
use serde::Serialize;
use std::process::Command;
use thiserror::Error;

// Top-level public error wrapper for the notify module
#[derive(Debug, Error)]
#[error("Error: {0}")]
pub struct Error(#[from] ErrorKind);

#[derive(Debug, Error)]
enum ErrorKind {
    #[error("ExecError: {0}")]
    Exec(#[from] ExecError),
}

transitive_from::hierarchy! {
    Error {
        ErrorKind {
            ExecError,
        }
    }
}

#[derive(Debug, Error)]
enum ExecError {
    #[error("Failed to spawn shell: {0}")]
    Spawn(#[from] std::io::Error),
    #[error("Non-zero exit status: {0}")]
    NonZero(String),
}

#[derive(Debug, Default, Serialize)]
/// Tracks the current notification state for a single GPU profile.
///
/// This is updated across interval ticks to implement monitor and notify grace behavior.
pub struct MonitorState {
    pub consecutive_count: u64,
    pub notify_skip_remaining: u64,
}

impl MonitorState {
    pub fn new() -> Self { Self::default() }
}

/// Determine whether a notification should be run for the provided config and observed socket_power.
///
/// This function updates the provided state according to monitor and notify grace semantics and
/// returns `true` when the notification command should be executed now.
pub fn should_notify(cfg: &PerGpuConfig, state: &mut MonitorState, socket_power: f64) -> bool {
    if !cfg.enabled {
        // reset state when disabled
        state.consecutive_count = 0;
        state.notify_skip_remaining = 0;
        return false;
    }

    // Evaluate threshold
    let condition = match cfg.monitor_threshold {
        Threshold::PowerBelow(th) => socket_power < th.get(),
    };

    // Handle monitor_grace: require n+1 consecutive true checks when Some(n)
    if condition {
        state.consecutive_count = state.consecutive_count.saturating_add(1);
    } else {
        state.consecutive_count = 0;
    }

    let monitor_ok = match cfg.monitor_grace {
        None => condition,
        Some(n) => state.consecutive_count > n.get(),
    };

    if !monitor_ok {
        return false;
    }

    // At this point, the profile is eligible to notify.
    match cfg.notify_grace {
        None => true,
        Some(n) => {
            if state.notify_skip_remaining > 0 {
                state.notify_skip_remaining = state.notify_skip_remaining.saturating_sub(1);
                false
            } else {
                // after running, set skip to n
                state.notify_skip_remaining = n.get();
                true
            }
        }
    }
}

/// Execute the notification command as a shell string and return whether it succeeded.
///
/// Uses `/bin/sh -c` so arbitrary shell expressions can be executed from config.
pub fn execute_notify_command(cmd: &str) -> Result<(), Error> {
    // Use /bin/sh -c to run arbitrary shell commands like in README
    let status = Command::new("/bin/sh")
        .arg("-c")
        .arg(cmd)
        .status()
        .map_err(ExecError::from)?;

    if status.success() {
        Ok(())
    } else {
        Err(ExecError::NonZero(format!("{}", status)).into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{NonZeroF64, PerGpuConfig, Parser, Threshold, Seconds};
    use std::num::NonZeroU64;

    fn make_cfg(monitor_grace: Option<NonZeroU64>, notify_grace: Option<NonZeroU64>) -> PerGpuConfig {
        PerGpuConfig {
            gpu_id: 0,
            enabled: true,
            parser: Parser::AmdSmi,
            monitor_interval: Seconds::new(60.0).unwrap(),
            monitor_command: "amd-smi metric --json".to_string(),
            monitor_threshold: Threshold::PowerBelow(NonZeroF64::new(100.0).unwrap()),
            monitor_grace,
            notify_grace,
            notify_command: "echo notify".to_string(),
        }
    }

    #[test]
    fn monitor_grace_none_triggers_immediately() {
        // When no monitor grace is configured, the threshold is evaluated immediately.
        let cfg = make_cfg(None, None);
        let mut s = MonitorState::new();
        assert!(should_notify(&cfg, &mut s, 50.0));
    }

    #[test]
    fn monitor_grace_some_requires_consecutive() {
        // Monitor grace of 2 means the threshold must be met 3 times in a row.
        let cfg = make_cfg(Some(NonZeroU64::new(2).unwrap()), None);
        let mut s = MonitorState::new();

        // first true -> not yet
        assert!(!should_notify(&cfg, &mut s, 50.0));
        // second true -> not yet
        assert!(!should_notify(&cfg, &mut s, 50.0));
        // third true -> now
        assert!(should_notify(&cfg, &mut s, 50.0));
    }

    #[test]
    fn notify_grace_skips_next_n_eligible() {
        let cfg = make_cfg(None, Some(NonZeroU64::new(2).unwrap())); // after notifying, skip next 2 eligible
        let mut s = MonitorState::new();
        // first eligible -> notify
        assert!(should_notify(&cfg, &mut s, 50.0));
        // next eligible -> skip (1)
        assert!(!should_notify(&cfg, &mut s, 50.0));
        // next eligible -> skip (0)
        assert!(!should_notify(&cfg, &mut s, 50.0));
        // next eligible -> notify again
        assert!(should_notify(&cfg, &mut s, 50.0));
    }
}
