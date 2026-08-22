use crate::config::PerGpuConfig;
use crate::notify::MonitorState;
use log::info;
use std::collections::HashMap;
use std::time::{Duration, Instant};
use thiserror::Error;

// Top-level public error wrapper for the gpu check module
#[derive(Debug, Error)]
#[error("Error: {0}")]
pub struct Error(#[from] ErrorKind);

#[derive(Debug, Error)]
enum ErrorKind {
    #[error("Monitor check error: {0}")]
    Monitor(#[from] MonitorCheckError),

    #[error("Notify check error: {0}")]
    Notify(#[from] NotifyCheckError),
}

transitive_from::hierarchy! {
    Error {
        ErrorKind {
            MonitorCheckError,
            NotifyCheckError,
        }
    }
}

#[derive(Debug, Error)]
enum MonitorCheckError {
    #[error("Monitor command failed: {0}")]
    Command(#[from] crate::monitor::Error),
}

#[derive(Debug, Error)]
enum NotifyCheckError {
    #[error("Notification command failed: {0}")]
    Command(#[from] crate::notify::Error),
}

/// Evaluate a single GPU profile and update state based on monitoring cadence.
///
/// This function handles disabled profiles, interval timing, monitor command execution,
/// and notification triggering for a specific GPU.
pub fn evaluate_gpu_profile(
    cfg: &PerGpuConfig,
    states: &mut HashMap<u64, MonitorState>,
    last_checked: &mut HashMap<u64, Instant>,
) -> Result<(), Error> {
    if !cfg.enabled {
        states.remove(&cfg.gpu_id);
        last_checked.remove(&cfg.gpu_id);
        return Ok(());
    }

    let interval = Duration::from_secs_f64(cfg.monitor_interval.get());
    let due = match last_checked.get(&cfg.gpu_id) {
        None => true,
        Some(t) => t.elapsed() >= interval,
    };

    if !due {
        return Ok(());
    }

    last_checked.insert(cfg.gpu_id, Instant::now());
    let state = states.entry(cfg.gpu_id).or_insert_with(MonitorState::new);

    info!("Running monitor command for GPU {}: {}", cfg.gpu_id, cfg.monitor_command);
    let power = crate::monitor::collect_socket_power(cfg).map_err(MonitorCheckError::from)?;
    info!("GPU {} socket_power={}", cfg.gpu_id, power);

    if crate::notify::should_notify(cfg, state, power) {
        info!("Triggering notify command: {}", cfg.notify_command);
        crate::notify::execute_notify_command(&cfg.notify_command)
            .map_err(NotifyCheckError::from)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{NonZeroF64, Parser, PerGpuConfig, Seconds, Threshold};

    fn make_cfg(enabled: bool) -> PerGpuConfig {
        PerGpuConfig {
            gpu_id: 3,
            enabled,
            parser: Parser::AmdSmi,
            monitor_interval: Seconds::new(60.0).unwrap(),
            monitor_command: "echo test".to_string(),
            monitor_threshold: Threshold::PowerBelow(NonZeroF64::new(100.0).unwrap()),
            monitor_grace: None,
            notify_grace: None,
            notify_command: "echo notify".to_string(),
        }
    }

    #[test]
    fn disabled_profiles_reset_state() {
        // Disabled GPU profiles must clear their saved monitor and notification state.
        let cfg = make_cfg(false);
        let mut states = HashMap::from([(3, MonitorState::new())]);
        let mut last_checked = HashMap::from([(3, Instant::now() - Duration::from_secs(5))]);

        evaluate_gpu_profile(&cfg, &mut states, &mut last_checked).expect("disabled profile should be accepted");

        assert!(states.get(&3).is_none());
        assert!(last_checked.get(&3).is_none());
    }
}
