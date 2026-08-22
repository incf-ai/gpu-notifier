//! Executes monitor commands and parses their GPU power output.

use crate::config::PerGpuConfig;
use std::process::Command;
use std::string::FromUtf8Error;
use thiserror::Error;

// Top-level public error wrapper for the monitor module
#[derive(Debug, Error)]
#[error("Error: {0}")]
pub struct Error(#[from] ErrorKind);

#[derive(Debug, Error)]
enum ErrorKind {
    #[error("Monitor command error: {0}")]
    Monitor(#[from] MonitorCommandError),

    #[error("AmdSmi parser error: {0}")]
    AmdSmi(#[from] crate::amd_smi::Error),

    #[error("Output encoding error: {0}")]
    Utf8(#[from] Utf8Error),
}

transitive_from::hierarchy! {
    Error {
        ErrorKind {
            MonitorCommandError,
            crate::amd_smi::Error,
            Utf8Error,
        }
    }
}

#[derive(Debug, Error)]
enum MonitorCommandError {
    #[error("Failed to spawn shell: {0}")]
    Spawn(#[from] std::io::Error),

    #[error("Monitor command exited with non-zero status: {0}")]
    NonZero(String),
}

#[derive(Debug, Error)]
enum Utf8Error {
    #[error("Monitor command output was not valid UTF-8: {0}")]
    Invalid(#[from] FromUtf8Error),
}

/// Execute the configured monitor command and return the parsed socket power for the configured GPU.
pub fn collect_socket_power(cfg: &PerGpuConfig) -> Result<f64, Error> {
    let output = Command::new("/bin/sh")
        .arg("-c")
        .arg(&cfg.monitor_command)
        .output()
        .map_err(MonitorCommandError::from)?;

    if output.status.success() {
        let body = String::from_utf8(output.stdout).map_err(Utf8Error::from)?;
        crate::amd_smi::extract_socket_power(&body, cfg.gpu_id).map_err(Into::into)
    } else {
        Err(MonitorCommandError::NonZero(format!("{}", output.status)).into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{NonZeroF64, Parser, PerGpuConfig, Seconds, Threshold};

    fn make_cfg(command: &str) -> PerGpuConfig {
        PerGpuConfig {
            gpu_id: 0,
            enabled: true,
            parser: Parser::AmdSmi,
            monitor_interval: Seconds::new(60.0).unwrap(),
            monitor_command: command.to_string(),
            monitor_threshold: Threshold::PowerBelow(NonZeroF64::new(100.0).unwrap()),
            monitor_grace: None,
            notify_grace: None,
            notify_command: "echo notify".to_string(),
        }
    }

    #[test]
    fn collects_socket_power_from_json_output() {
        // Ensure the monitor module accepts shell output and extracts the correct power value.
        let payload = r#"{"gpu_data":[{"gpu":0,"power":{"socket_power":{"value":214.0,"unit":"W"}}}]}"#;
        let cfg = make_cfg(&format!("printf '{payload}'"));
        let power = collect_socket_power(&cfg).expect("monitor output should parse");
        assert_eq!(power, 214.0);
    }
}
