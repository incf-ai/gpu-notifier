use crate::config::GpuNotifier;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};
use thiserror::Error;

/// Configuration reload module that caches config data and refreshes it on a timer.

// Top-level public error wrapper for the config reload module
#[derive(Debug, Error)]
#[error("Error: {0}")]
pub struct Error(#[from] ErrorKind);

#[derive(Debug, Error)]
enum ErrorKind {
    #[error("Config read error: {0}")]
    Read(#[from] ReadConfigError),

    #[error("Config parse error: {0}")]
    Parse(#[from] ParseConfigError),
}

transitive_from::hierarchy! {
    Error {
        ErrorKind {
            ReadConfigError,
            ParseConfigError,
        }
    }
}

#[derive(Debug, Error)]
enum ReadConfigError {
    #[error("Failed to read config file {path:?}: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
}

#[derive(Debug, Error)]
enum ParseConfigError {
    #[error("Failed to parse config file {path:?}: {source}")]
    Parse {
        path: PathBuf,
        #[source]
        source: crate::config::Error,
    },
}

/// Holds cached configuration state and the timestamp of the last successful reload.
pub struct ConfigReloadState {
    last_reload: Instant,
    config: Option<GpuNotifier>,
}

impl ConfigReloadState {
    pub fn new(reload_interval_secs: f64) -> Self {
        Self {
            last_reload: Instant::now() - Duration::from_secs_f64(reload_interval_secs),
            config: None,
        }
    }

    /// Reloads the configuration file if the configured interval has elapsed.
    ///
    /// Returns the cached config when the interval has not yet passed, or reloads the file
    /// and parses it into a `GpuNotifier` on success.
    pub fn maybe_reload(&mut self, config_path: &Path, reload_interval_secs: f64) -> Result<Option<&GpuNotifier>, Error> {
        if self.last_reload.elapsed().as_secs_f64() < reload_interval_secs {
            return Ok(self.config.as_ref());
        }

        let contents = std::fs::read_to_string(config_path).map_err(|source| ReadConfigError::Io {
            path: config_path.to_path_buf(),
            source,
        })?;

        let cfg = crate::config::GpuNotifier::from_ron_str(&contents).map_err(|source| ParseConfigError::Parse {
            path: config_path.to_path_buf(),
            source,
        })?;

        self.config = Some(cfg);
        self.last_reload = Instant::now();
        Ok(self.config.as_ref())
    }

    pub fn current_config(&self) -> Option<&GpuNotifier> {
        self.config.as_ref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::time::Duration;

    #[test]
    fn reloads_once_until_interval_elapsed() {
        // Ensure config reload is cached until the interval passes, then updates from disk.
        let dir = std::env::temp_dir().join(format!("gpu-notifier-test-{}", std::process::id()));
        let path = dir.join("config.ron");
        fs::create_dir_all(&dir).expect("temp dir should be created");

        let initial = r#"GpuNotifier(gpus: [])"#;
        fs::write(&path, initial).expect("initial config should be written");

        let mut state = ConfigReloadState::new(0.05);
        let first = state.maybe_reload(&path, 0.05).expect("first reload should succeed").expect("config should load");
        assert_eq!(first.gpus.len(), 0);

        let updated = r#"GpuNotifier(gpus: [PerGpuConfig(gpu_id: 7, enabled: true, parser: AmdSmi, monitor_interval: Seconds(60.0), monitor_command: "echo hi", monitor_threshold: PowerBelow(10.0), monitor_grace: None, notify_grace: None, notify_command: "echo notify" )])"#;
        fs::write(&path, updated).expect("updated config should be written");

        let cached = state.maybe_reload(&path, 0.05).expect("reload should return cached config");
        assert!(cached.is_some());
        assert_eq!(cached.unwrap().gpus.len(), 0);

        std::thread::sleep(Duration::from_millis(60));

        let reloaded = state.maybe_reload(&path, 0.05).expect("reload after delay should succeed").expect("config should reload");
        assert_eq!(reloaded.gpus.len(), 1);
        assert_eq!(reloaded.gpus[0].gpu_id, 7);

        fs::remove_dir_all(&dir).expect("temp dir should be removed");
    }
}
