//! Binary entrypoint for the gpu-notifier daemon.
//!
//! This executable loads configuration, regularly reloads it, and evaluates each GPU profile
//! to decide when to execute notification commands.
use std::{thread, time::{Duration, Instant}};
use log::{info, error, warn};
use env_logger;
use std::path::PathBuf;
use clap::Parser as ClapParser;
use std::collections::HashMap;

/// Command-line arguments accepted by the gpu-notifier daemon.
#[derive(clap::Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path to the RON configuration file
    #[arg(short, long, default_value = "~/.config/gpu-notifier/config.ron")]
    config: String,
}

const CONFIG_RELOAD_SECS: f64 = 60.0;
const LOOP_TICK_MS: u64 = 1000; // 1s tick

// Config reload interval and the main loop sleep duration.

fn main() {
    env_logger::init();
    let args = Args::parse();

    let config_path = shellexpand::tilde(&args.config).into_owned();
    let config_path = PathBuf::from(config_path);

    info!("Starting gpu-notifier daemon, config={:?}", config_path);

    // State preserved across iterations per GPU
    let mut states: HashMap<u64, gpu_notifier::notify::MonitorState> = HashMap::new();
    let mut last_checked: HashMap<u64, Instant> = HashMap::new();
    let mut config_reload = gpu_notifier::config_reload::ConfigReloadState::new(CONFIG_RELOAD_SECS);

    loop {
        match config_reload.maybe_reload(&config_path, CONFIG_RELOAD_SECS) {
            Ok(Some(cfg)) => {
                info!("Reloaded configuration with {} GPU profiles", cfg.gpus.len());
            }
            Ok(None) => {}
            Err(e) => error!("Failed to reload config (keeping previous): {}", e),
        }

        if let Some(cfg) = config_reload.current_config() {
            for g in cfg.gpus.iter() {
                if let Err(e) = gpu_notifier::gpu_check::evaluate_gpu_profile(g, &mut states, &mut last_checked) {
                    warn!("gpu check error for gpu {}: {}", g.gpu_id, e);
                }
            }
        } else {
            // No valid config loaded yet; try reading it immediately next loop tick
            info!("No valid configuration loaded yet; waiting to reload...");
        }

        thread::sleep(Duration::from_millis(LOOP_TICK_MS));
    }
}
