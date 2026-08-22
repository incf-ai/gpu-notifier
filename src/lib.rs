//! Core crate module exports and public API surface for the gpu-notifier library.
//! This module re-exports important types from child modules so consumers can import them easily.

pub mod config;
pub mod amd_smi;
pub mod notify;
pub mod error;
pub mod monitor;
pub mod config_reload;
pub mod gpu_check;

pub use amd_smi::*;
pub use notify::*;

// Re-export selected public config items to avoid ambiguous `Error` re-exports
pub use config::{GpuNotifier, PerGpuConfig, Parser, Threshold, Seconds};

// Re-export the crate-level aggregated Error
pub use error::Error;
