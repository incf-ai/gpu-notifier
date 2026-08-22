use thiserror::Error;

/// Top-level error aggregation for the entire application.
// Top-level public error wrapper
#[derive(Debug, Error)]
#[error("Error: {0}")]
pub struct Error(#[from] ErrorKind);

#[derive(Debug, Error)]
enum ErrorKind {
    #[error("ConfigError: {0}")]
    Config(#[from] crate::config::Error),

    #[error("AmdSmiError: {0}")]
    AmdSmi(#[from] crate::amd_smi::Error),

    #[error("NotifyError: {0}")]
    Notify(#[from] crate::notify::Error),

    #[error("ConfigReloadError: {0}")]
    ConfigReload(#[from] crate::config_reload::Error),

    #[error("GpuCheckError: {0}")]
    GpuCheck(#[from] crate::gpu_check::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("RON error: {0}")]
    Ron(#[from] ron::de::Error),
}

transitive_from::hierarchy! {
    Error {
        ErrorKind {
            crate::config::Error,
            crate::amd_smi::Error,
            crate::notify::Error,
            crate::config_reload::Error,
            crate::gpu_check::Error,
            std::io::Error,
            serde_json::Error,
            ron::de::Error,
        }
    }
}
