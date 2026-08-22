//! Configuration types and validation helpers for RON-based gpu-notifier configuration.

use serde::{Deserialize, Deserializer};
use std::num::NonZeroU64;
use thiserror::Error;

// Public module error type following README pattern
#[derive(Debug, Error)]
#[error("Config Error: {0}")]
pub struct Error(#[from] ErrorKind);

#[derive(Debug, Error)]
enum ErrorKind {
    #[error("Parse error: {0}")]
    Parse(#[from] ParseError),

    #[error("RON error: {0}")]
    Ron(#[from] ron::de::Error),
}

#[derive(Debug, Error)]
enum ParseError {
    #[error("Failed to parse config: {0}")]
    Message(String),
}

transitive_from::hierarchy! {
    Error {
        ErrorKind {
            crate::config::ParseError,
            ron::de::Error,
        }
    }
}

#[derive(Debug, Deserialize, PartialEq)]
/// Root configuration type containing all GPU notifier profiles.
pub struct GpuNotifier {
    pub gpus: Vec<PerGpuConfig>,
}

#[derive(Debug, Deserialize, PartialEq, Clone)]
/// Configuration for a single GPU profile.
pub struct PerGpuConfig {
    pub gpu_id: u64,
    #[serde(default = "default_enabled")]
    pub enabled: bool,
    pub parser: Parser,
    pub monitor_interval: Seconds,
    pub monitor_command: String,
    pub monitor_threshold: Threshold,
    /// Optional non-zero monitor grace count. If present, the condition must be
    /// true for `n + 1` consecutive checks before a notification becomes eligible.
    pub monitor_grace: Option<NonZeroU64>,
    /// Optional non-zero notify grace count. If present, the notifier skips the
    /// next `n` eligible notification events after a notification is raised.
    pub notify_grace: Option<NonZeroU64>,
    pub notify_command: String,
}

fn default_enabled() -> bool { true }

#[derive(Debug, Deserialize, PartialEq, Clone)]
/// Supported parser types for monitor command output.
pub enum Parser {
    AmdSmi,
}

#[derive(Debug, Deserialize, PartialEq, Clone)]
/// Thresholds for GPU monitoring.
///
/// `PowerBelow` uses a non-zero float value so zero cannot be used as a threshold.
pub enum Threshold {
    PowerBelow(NonZeroF64),
}

#[derive(Debug, Deserialize, PartialEq, Clone, Copy)]
/// A non-zero duration in seconds.
pub struct Seconds(NonZeroF64);

#[derive(Debug, PartialEq, Clone, Copy)]
/// Wrapper around floating-point values that forbids zero.
pub struct NonZeroF64(f64);

impl NonZeroF64 {
    /// Create a wrapped non-zero floating point value.
    ///
    /// Returns `None` when the provided value is exactly `0.0`.
    pub fn new(value: f64) -> Option<Self> {
        if value != 0.0 {
            Some(Self(value))
        } else {
            None
        }
    }

    /// Access the underlying float value.
    pub fn get(self) -> f64 {
        self.0
    }
}

impl<'de> Deserialize<'de> for NonZeroF64 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = f64::deserialize(deserializer)?;
        Self::new(value).ok_or_else(|| serde::de::Error::custom("value must be non-zero"))
    }
}

impl Seconds {
    pub fn new(value: f64) -> Option<Self> {
        NonZeroF64::new(value).map(Self)
    }

    pub fn get(self) -> f64 {
        self.0.get()
    }
}

impl Threshold {
    pub fn power_below(value: f64) -> Option<Self> {
        NonZeroF64::new(value).map(Self::PowerBelow)
    }
}

impl GpuNotifier {
    pub fn from_ron_str(s: &str) -> Result<Self, Error> {
        ron::from_str(s).map_err(|e| ParseError::Message(e.to_string()).into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_example_config() {
        // The config must parse from a RON string and preserve non-zero grace values.
        let ron = r#"
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
            notify_command: "ffplay -v 0 -nodisp -autoexit /home/user/Downloads/notify.mp3",
        ),
    ],
)
"#;

        let cfg = GpuNotifier::from_ron_str(ron).expect("should parse");
        assert_eq!(cfg.gpus.len(), 1);
        let p = &cfg.gpus[0];
        assert_eq!(p.gpu_id, 0);
        assert!(p.enabled);
        matches!(p.parser, Parser::AmdSmi);
        assert_eq!(p.monitor_interval.get(), 60.0);
        assert_eq!(p.monitor_command, "amd-smi metric --json");
        matches!(p.monitor_threshold, Threshold::PowerBelow(v) if (v.get() - 50.0).abs() < 1e-6);
        assert_eq!(p.notify_command, "ffplay -v 0 -nodisp -autoexit /home/user/Downloads/notify.mp3");
        assert_eq!(p.notify_grace, Some(NonZeroU64::new(1).unwrap()));
    }
}
