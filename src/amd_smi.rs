use serde::Deserialize;
use thiserror::Error;

/// AMD SMI JSON parser and power extraction helpers.
// Public module error type following README pattern
#[derive(Debug, Error)]
#[error("AmdSmi Error: {0}")]
pub struct Error(#[from] ErrorKind);

#[derive(Debug, Error)]
enum ErrorKind {
    #[error("Domain error: {0}")]
    Domain(#[from] DomainError),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
}

#[derive(Debug, Error)]
enum DomainError {
    #[error("Missing GPU {0} in payload")]
    MissingGpu(u64),

}

transitive_from::hierarchy! {
    Error {
        ErrorKind {
            crate::amd_smi::DomainError,
            serde_json::Error,
        }
    }
}

#[derive(Debug, Deserialize)]
struct Root {
    gpu_data: Vec<GpuEntry>,
}

#[derive(Debug, Deserialize)]
struct GpuEntry {
    gpu: u64,
    power: Power,
}

#[derive(Debug, Deserialize)]
struct Power {
    socket_power: SocketPower,
}

#[derive(Debug, Deserialize)]
struct SocketPower {
    value: f64,
    #[serde(rename = "unit")]
    _unit: String,
}

/// Parse AMD SMI JSON and extract the socket power for a specific GPU.
///
/// Returns an error if the JSON cannot be parsed or the requested GPU is not found.
pub fn extract_socket_power(json: &str, gpu_id: u64) -> Result<f64, Error> {
    let root: Root = serde_json::from_str(json)?;
    for entry in root.gpu_data {
        if entry.gpu == gpu_id {
            return Ok(entry.power.socket_power.value);
        }
    }
    Err(DomainError::MissingGpu(gpu_id).into())
}

#[cfg(test)]
mod tests {
    use std::fmt::format;

use super::*;

    fn gen_multi_gpu_data(power1: f32, power2: f32) -> String {
        let s = r#"
{
    "gpu_data": [
        {
            "gpu": 0,
            "usage": {
                "gfx_activity": {
                    "value": 91,
                    "unit": "%"
                },
                "umc_activity": {
                    "value": 66,
                    "unit": "%"
                },
                "mm_activity": {
                    "value": 0,
                    "unit": "%"
                },
                "vcn_activity": [
                    {
                        "value": 0,
                        "unit": "%"
                    },
                    "N/A",
                    "N/A",
                    "N/A"
                ],
                "jpeg_activity": [
                    "N/A",
                    "N/A",
                    "N/A",
                    "N/A",
                    "N/A",
                    "N/A",
                    "N/A",
                    "N/A",
                    "N/A",
                    "N/A",
                    "N/A",
                    "N/A",
                    "N/A",
                    "N/A",
                    "N/A",
                    "N/A",
                    "N/A",
                    "N/A",
                    "N/A",
                    "N/A",
                    "N/A",
                    "N/A",
                    "N/A",
                    "N/A",
                    "N/A",
                    "N/A",
                    "N/A",
                    "N/A",
                    "N/A",
                    "N/A",
                    "N/A",
                    "N/A"
                ],
                "gfx_busy_inst": "N/A",
                "jpeg_busy": "N/A",
                "vcn_busy": "N/A"
            },
            "power": {
                "socket_power": {
                    "value": {power1},
                    "unit": "W"
                },
                "gfx_voltage": {
                    "value": 933,
                    "unit": "mV"
                },
                "soc_voltage": {
                    "value": 807,
                    "unit": "mV"
                },
                "mem_voltage": {
                    "value": 1350,
                    "unit": "mV"
                },
                "throttle_status": "THROTTLED",
                "power_management": "ENABLED"
            },
            "clock": {
                "gfx_0": {
                    "clk": {
                        "value": 2774,
                        "unit": "MHz"
                    },
                    "min_clk": "N/A",
                    "max_clk": {
                        "value": 0,
                        "unit": "MHz"
                    },
                    "clk_locked": "N/A",
                    "deep_sleep": "N/A"
                },
                "gfx_1": {
                    "clk": "N/A",
                    "min_clk": "N/A",
                    "max_clk": "N/A",
                    "clk_locked": "N/A",
                    "deep_sleep": "N/A"
                },
                "gfx_2": {
                    "clk": "N/A",
                    "min_clk": "N/A",
                    "max_clk": "N/A",
                    "clk_locked": "N/A",
                    "deep_sleep": "N/A"
                },
                "gfx_3": {
                    "clk": "N/A",
                    "min_clk": "N/A",
                    "max_clk": "N/A",
                    "clk_locked": "N/A",
                    "deep_sleep": "N/A"
                },
                "gfx_4": {
                    "clk": "N/A",
                    "min_clk": "N/A",
                    "max_clk": "N/A",
                    "clk_locked": "N/A",
                    "deep_sleep": "N/A"
                },
                "gfx_5": {
                    "clk": "N/A",
                    "min_clk": "N/A",
                    "max_clk": "N/A",
                    "clk_locked": "N/A",
                    "deep_sleep": "N/A"
                },
                "gfx_6": {
                    "clk": "N/A",
                    "min_clk": "N/A",
                    "max_clk": "N/A",
                    "clk_locked": "N/A",
                    "deep_sleep": "N/A"
                },
                "gfx_7": {
                    "clk": "N/A",
                    "min_clk": "N/A",
                    "max_clk": "N/A",
                    "clk_locked": "N/A",
                    "deep_sleep": "N/A"
                },
                "mem_0": {
                    "clk": {
                        "value": 1258,
                        "unit": "MHz"
                    },
                    "min_clk": {
                        "value": 97,
                        "unit": "MHz"
                    },
                    "max_clk": {
                        "value": 1259,
                        "unit": "MHz"
                    },
                    "clk_locked": "N/A",
                    "deep_sleep": "DISABLED"
                },
                "vclk_0": {
                    "clk": {
                        "value": 25,
                        "unit": "MHz"
                    },
                    "min_clk": "N/A",
                    "max_clk": "N/A",
                    "clk_locked": "N/A",
                    "deep_sleep": "N/A"
                },
                "vclk_1": {
                    "clk": "N/A",
                    "min_clk": "N/A",
                    "max_clk": "N/A",
                    "clk_locked": "N/A",
                    "deep_sleep": "N/A"
                },
                "vclk_2": {
                    "clk": "N/A",
                    "min_clk": "N/A",
                    "max_clk": "N/A",
                    "clk_locked": "N/A",
                    "deep_sleep": "N/A"
                },
                "vclk_3": {
                    "clk": "N/A",
                    "min_clk": "N/A",
                    "max_clk": "N/A",
                    "clk_locked": "N/A",
                    "deep_sleep": "N/A"
                },
                "dclk_0": {
                    "clk": {
                        "value": 25,
                        "unit": "MHz"
                    },
                    "min_clk": "N/A",
                    "max_clk": "N/A",
                    "clk_locked": "N/A",
                    "deep_sleep": "N/A"
                },
                "dclk_1": {
                    "clk": "N/A",
                    "min_clk": "N/A",
                    "max_clk": "N/A",
                    "clk_locked": "N/A",
                    "deep_sleep": "N/A"
                },
                "dclk_2": {
                    "clk": "N/A",
                    "min_clk": "N/A",
                    "max_clk": "N/A",
                    "clk_locked": "N/A",
                    "deep_sleep": "N/A"
                },
                "dclk_3": {
                    "clk": "N/A",
                    "min_clk": "N/A",
                    "max_clk": "N/A",
                    "clk_locked": "N/A",
                    "deep_sleep": "N/A"
                },
                "fclk_0": {
                    "clk": {
                        "value": 2016,
                        "unit": "MHz"
                    },
                    "min_clk": {
                        "value": 313,
                        "unit": "MHz"
                    },
                    "max_clk": {
                        "value": 2400,
                        "unit": "MHz"
                    },
                    "clk_locked": "N/A",
                    "deep_sleep": "DISABLED"
                },
                "socclk_0": {
                    "clk": {
                        "value": 1280,
                        "unit": "MHz"
                    },
                    "min_clk": {
                        "value": 417,
                        "unit": "MHz"
                    },
                    "max_clk": {
                        "value": 1476,
                        "unit": "MHz"
                    },
                    "clk_locked": "N/A",
                    "deep_sleep": "DISABLED"
                }
            },
            "temperature": {
                "edge": {
                    "value": 54,
                    "unit": "C"
                },
                "hotspot": {
                    "value": 63,
                    "unit": "C"
                },
                "mem": {
                    "value": 70,
                    "unit": "C"
                }
            },
            "pcie": {
                "width": 8,
                "speed": {
                    "value": 2.5,
                    "unit": "GT/s"
                },
                "bandwidth": "N/A",
                "replay_count": "N/A",
                "l0_to_recovery_count": "N/A",
                "replay_roll_over_count": "N/A",
                "nak_sent_count": "N/A",
                "nak_received_count": "N/A",
                "current_bandwidth_sent": "N/A",
                "current_bandwidth_received": "N/A",
                "max_packet_size": "N/A",
                "lc_perf_other_end_recovery": "N/A"
            },
            "ecc": {
                "total_correctable_count": 0,
                "total_uncorrectable_count": 0,
                "total_deferred_count": 0,
                "cache_correctable_count": 0,
                "cache_uncorrectable_count": 0
            },
            "ecc_blocks": {
                "UMC": {
                    "correctable_count": 0,
                    "uncorrectable_count": 0,
                    "deferred_count": 0
                }
            },
            "fan": {
                "speed": 127,
                "max": 255,
                "rpm": 2570,
                "usage": {
                    "value": 49.8,
                    "unit": "%"
                }
            },
            "voltage_curve": {
                "point_0_frequency": "N/A",
                "point_0_voltage": "N/A",
                "point_1_frequency": "N/A",
                "point_1_voltage": "N/A",
                "point_2_frequency": "N/A",
                "point_2_voltage": "N/A"
            },
            "overdrive": "N/A",
            "mem_overdrive": "N/A",
            "perf_level": "AMDSMI_DEV_PERF_LEVEL_AUTO",
            "xgmi_err": "N/A",
            "voltage": {
                "vddboard": "N/A"
            },
            "energy": {
                "total_energy_consumption": {
                    "value": 0.0,
                    "unit": "J"
                }
            },
            "mem_usage": {
                "total_vram": {
                    "value": 30576,
                    "unit": "MB"
                },
                "used_vram": {
                    "value": 20450,
                    "unit": "MB"
                },
                "free_vram": {
                    "value": 10126,
                    "unit": "MB"
                },
                "total_visible_vram": {
                    "value": 30576,
                    "unit": "MB"
                },
                "used_visible_vram": {
                    "value": 20450,
                    "unit": "MB"
                },
                "free_visible_vram": {
                    "value": 10126,
                    "unit": "MB"
                },
                "total_gtt": {
                    "value": 32023,
                    "unit": "MB"
                },
                "used_gtt": {
                    "value": 1162,
                    "unit": "MB"
                },
                "free_gtt": {
                    "value": 30861,
                    "unit": "MB"
                }
            },
            "throttle": {
                "accumulation_counter": "N/A",
                "prochot_accumulated": "N/A",
                "ppt_accumulated": "N/A",
                "socket_thermal_accumulated": "N/A",
                "vr_thermal_accumulated": "N/A",
                "hbm_thermal_accumulated": "N/A",
                "gfx_clk_below_host_limit_accumulated": "N/A",
                "gfx_clk_below_host_limit_power_accumulated": "N/A",
                "gfx_clk_below_host_limit_thermal_accumulated": "N/A",
                "total_gfx_clk_below_host_limit_accumulated": "N/A",
                "low_utilization_accumulated": "N/A",
                "prochot_violation_status": "N/A",
                "ppt_violation_status": "N/A",
                "socket_thermal_violation_status": "N/A",
                "vr_thermal_violation_status": "N/A",
                "hbm_thermal_violation_status": "N/A",
                "gfx_clk_below_host_limit_violation_status": "N/A",
                "gfx_clk_below_host_limit_power_violation_status": "N/A",
                "gfx_clk_below_host_limit_thermal_violation_status": "N/A",
                "total_gfx_clk_below_host_limit_violation_status": "N/A",
                "low_utilization_violation_status": "N/A",
                "prochot_violation_activity": "N/A",
                "ppt_violation_activity": "N/A",
                "socket_thermal_violation_activity": "N/A",
                "vr_thermal_violation_activity": "N/A",
                "hbm_thermal_violation_activity": "N/A",
                "gfx_clk_below_host_limit_violation_activity": "N/A",
                "gfx_clk_below_host_limit_power_violation_activity": "N/A",
                "gfx_clk_below_host_limit_thermal_violation_activity": "N/A",
                "total_gfx_clk_below_host_limit_violation_activity": "N/A",
                "low_utilization_violation_activity": "N/A"
            }
        },
        {
            "gpu": 1,
            "usage": {
                "gfx_activity": {
                    "value": 1,
                    "unit": "%"
                },
                "umc_activity": {
                    "value": 0,
                    "unit": "%"
                },
                "mm_activity": {
                    "value": 0,
                    "unit": "%"
                },
                "vcn_activity": [
                    {
                        "value": 0,
                        "unit": "%"
                    },
                    "N/A",
                    "N/A",
                    "N/A"
                ],
                "jpeg_activity": [
                    "N/A",
                    "N/A",
                    "N/A",
                    "N/A",
                    "N/A",
                    "N/A",
                    "N/A",
                    "N/A",
                    "N/A",
                    "N/A",
                    "N/A",
                    "N/A",
                    "N/A",
                    "N/A",
                    "N/A",
                    "N/A",
                    "N/A",
                    "N/A",
                    "N/A",
                    "N/A",
                    "N/A",
                    "N/A",
                    "N/A",
                    "N/A",
                    "N/A",
                    "N/A",
                    "N/A",
                    "N/A",
                    "N/A",
                    "N/A",
                    "N/A",
                    "N/A"
                ],
                "gfx_busy_inst": "N/A",
                "jpeg_busy": "N/A",
                "vcn_busy": "N/A"
            },
            "power": {
                "socket_power": {
                    "value": {power2},
                    "unit": "W"
                },
                "gfx_voltage": {
                    "value": 82,
                    "unit": "mV"
                },
                "soc_voltage": {
                    "value": 829,
                    "unit": "mV"
                },
                "mem_voltage": {
                    "value": 1249,
                    "unit": "mV"
                },
                "throttle_status": "THROTTLED",
                "power_management": "ENABLED"
            },
            "clock": {
                "gfx_0": {
                    "clk": {
                        "value": 15,
                        "unit": "MHz"
                    },
                    "min_clk": "N/A",
                    "max_clk": {
                        "value": 0,
                        "unit": "MHz"
                    },
                    "clk_locked": "N/A",
                    "deep_sleep": "N/A"
                },
                "gfx_1": {
                    "clk": "N/A",
                    "min_clk": "N/A",
                    "max_clk": "N/A",
                    "clk_locked": "N/A",
                    "deep_sleep": "N/A"
                },
                "gfx_2": {
                    "clk": "N/A",
                    "min_clk": "N/A",
                    "max_clk": "N/A",
                    "clk_locked": "N/A",
                    "deep_sleep": "N/A"
                },
                "gfx_3": {
                    "clk": "N/A",
                    "min_clk": "N/A",
                    "max_clk": "N/A",
                    "clk_locked": "N/A",
                    "deep_sleep": "N/A"
                },
                "gfx_4": {
                    "clk": "N/A",
                    "min_clk": "N/A",
                    "max_clk": "N/A",
                    "clk_locked": "N/A",
                    "deep_sleep": "N/A"
                },
                "gfx_5": {
                    "clk": "N/A",
                    "min_clk": "N/A",
                    "max_clk": "N/A",
                    "clk_locked": "N/A",
                    "deep_sleep": "N/A"
                },
                "gfx_6": {
                    "clk": "N/A",
                    "min_clk": "N/A",
                    "max_clk": "N/A",
                    "clk_locked": "N/A",
                    "deep_sleep": "N/A"
                },
                "gfx_7": {
                    "clk": "N/A",
                    "min_clk": "N/A",
                    "max_clk": "N/A",
                    "clk_locked": "N/A",
                    "deep_sleep": "N/A"
                },
                "mem_0": {
                    "clk": {
                        "value": 96,
                        "unit": "MHz"
                    },
                    "min_clk": {
                        "value": 97,
                        "unit": "MHz"
                    },
                    "max_clk": {
                        "value": 1259,
                        "unit": "MHz"
                    },
                    "clk_locked": "N/A",
                    "deep_sleep": "ENABLED"
                },
                "vclk_0": {
                    "clk": {
                        "value": 25,
                        "unit": "MHz"
                    },
                    "min_clk": "N/A",
                    "max_clk": "N/A",
                    "clk_locked": "N/A",
                    "deep_sleep": "N/A"
                },
                "vclk_1": {
                    "clk": "N/A",
                    "min_clk": "N/A",
                    "max_clk": "N/A",
                    "clk_locked": "N/A",
                    "deep_sleep": "N/A"
                },
                "vclk_2": {
                    "clk": "N/A",
                    "min_clk": "N/A",
                    "max_clk": "N/A",
                    "clk_locked": "N/A",
                    "deep_sleep": "N/A"
                },
                "vclk_3": {
                    "clk": "N/A",
                    "min_clk": "N/A",
                    "max_clk": "N/A",
                    "clk_locked": "N/A",
                    "deep_sleep": "N/A"
                },
                "dclk_0": {
                    "clk": {
                        "value": 25,
                        "unit": "MHz"
                    },
                    "min_clk": "N/A",
                    "max_clk": "N/A",
                    "clk_locked": "N/A",
                    "deep_sleep": "N/A"
                },
                "dclk_1": {
                    "clk": "N/A",
                    "min_clk": "N/A",
                    "max_clk": "N/A",
                    "clk_locked": "N/A",
                    "deep_sleep": "N/A"
                },
                "dclk_2": {
                    "clk": "N/A",
                    "min_clk": "N/A",
                    "max_clk": "N/A",
                    "clk_locked": "N/A",
                    "deep_sleep": "N/A"
                },
                "dclk_3": {
                    "clk": "N/A",
                    "min_clk": "N/A",
                    "max_clk": "N/A",
                    "clk_locked": "N/A",
                    "deep_sleep": "N/A"
                },
                "fclk_0": {
                    "clk": {
                        "value": 582,
                        "unit": "MHz"
                    },
                    "min_clk": {
                        "value": 313,
                        "unit": "MHz"
                    },
                    "max_clk": {
                        "value": 2400,
                        "unit": "MHz"
                    },
                    "clk_locked": "N/A",
                    "deep_sleep": "DISABLED"
                },
                "socclk_0": {
                    "clk": {
                        "value": 417,
                        "unit": "MHz"
                    },
                    "min_clk": {
                        "value": 417,
                        "unit": "MHz"
                    },
                    "max_clk": {
                        "value": 1476,
                        "unit": "MHz"
                    },
                    "clk_locked": "N/A",
                    "deep_sleep": "DISABLED"
                }
            },
            "temperature": {
                "edge": {
                    "value": 31,
                    "unit": "C"
                },
                "hotspot": {
                    "value": 32,
                    "unit": "C"
                },
                "mem": {
                    "value": 30,
                    "unit": "C"
                }
            },
            "pcie": {
                "width": 8,
                "speed": {
                    "value": 2.5,
                    "unit": "GT/s"
                },
                "bandwidth": "N/A",
                "replay_count": "N/A",
                "l0_to_recovery_count": "N/A",
                "replay_roll_over_count": "N/A",
                "nak_sent_count": "N/A",
                "nak_received_count": "N/A",
                "current_bandwidth_sent": "N/A",
                "current_bandwidth_received": "N/A",
                "max_packet_size": "N/A",
                "lc_perf_other_end_recovery": "N/A"
            },
            "ecc": {
                "total_correctable_count": 0,
                "total_uncorrectable_count": 0,
                "total_deferred_count": 0,
                "cache_correctable_count": 0,
                "cache_uncorrectable_count": 0
            },
            "ecc_blocks": {
                "UMC": {
                    "correctable_count": 0,
                    "uncorrectable_count": 0,
                    "deferred_count": 0
                }
            },
            "fan": {
                "speed": 76,
                "max": 255,
                "rpm": 2050,
                "usage": {
                    "value": 29.8,
                    "unit": "%"
                }
            },
            "voltage_curve": {
                "point_0_frequency": "N/A",
                "point_0_voltage": "N/A",
                "point_1_frequency": "N/A",
                "point_1_voltage": "N/A",
                "point_2_frequency": "N/A",
                "point_2_voltage": "N/A"
            },
            "overdrive": "N/A",
            "mem_overdrive": "N/A",
            "perf_level": "AMDSMI_DEV_PERF_LEVEL_AUTO",
            "xgmi_err": "N/A",
            "voltage": {
                "vddboard": "N/A"
            },
            "energy": {
                "total_energy_consumption": {
                    "value": 0.0,
                    "unit": "J"
                }
            },
            "mem_usage": {
                "total_vram": {
                    "value": 32624,
                    "unit": "MB"
                },
                "used_vram": {
                    "value": 57,
                    "unit": "MB"
                },
                "free_vram": {
                    "value": 32567,
                    "unit": "MB"
                },
                "total_visible_vram": {
                    "value": 32624,
                    "unit": "MB"
                },
                "used_visible_vram": {
                    "value": 57,
                    "unit": "MB"
                },
                "free_visible_vram": {
                    "value": 32567,
                    "unit": "MB"
                },
                "total_gtt": {
                    "value": 32023,
                    "unit": "MB"
                },
                "used_gtt": {
                    "value": 28,
                    "unit": "MB"
                },
                "free_gtt": {
                    "value": 31995,
                    "unit": "MB"
                }
            },
            "throttle": {
                "accumulation_counter": "N/A",
                "prochot_accumulated": "N/A",
                "ppt_accumulated": "N/A",
                "socket_thermal_accumulated": "N/A",
                "vr_thermal_accumulated": "N/A",
                "hbm_thermal_accumulated": "N/A",
                "gfx_clk_below_host_limit_accumulated": "N/A",
                "gfx_clk_below_host_limit_power_accumulated": "N/A",
                "gfx_clk_below_host_limit_thermal_accumulated": "N/A",
                "total_gfx_clk_below_host_limit_accumulated": "N/A",
                "low_utilization_accumulated": "N/A",
                "prochot_violation_status": "N/A",
                "ppt_violation_status": "N/A",
                "socket_thermal_violation_status": "N/A",
                "vr_thermal_violation_status": "N/A",
                "hbm_thermal_violation_status": "N/A",
                "gfx_clk_below_host_limit_violation_status": "N/A",
                "gfx_clk_below_host_limit_power_violation_status": "N/A",
                "gfx_clk_below_host_limit_thermal_violation_status": "N/A",
                "total_gfx_clk_below_host_limit_violation_status": "N/A",
                "low_utilization_violation_status": "N/A",
                "prochot_violation_activity": "N/A",
                "ppt_violation_activity": "N/A",
                "socket_thermal_violation_activity": "N/A",
                "vr_thermal_violation_activity": "N/A",
                "hbm_thermal_violation_activity": "N/A",
                "gfx_clk_below_host_limit_violation_activity": "N/A",
                "gfx_clk_below_host_limit_power_violation_activity": "N/A",
                "gfx_clk_below_host_limit_thermal_violation_activity": "N/A",
                "total_gfx_clk_below_host_limit_violation_activity": "N/A",
                "low_utilization_violation_activity": "N/A"
            }
        }
    ]
}
        "#;
        let mut s = s.to_string();
        s = s.replace("{power1}", &format!("{power1:0.1}"));
        s = s.replace("{power2}", &format!("{power2:0.1}"));
        s
    }

    #[test]
    fn parse_multi_gpu() {
        let json = &gen_multi_gpu_data(12.3, 23.4);
        let v = extract_socket_power(json, 0).expect("found");
        assert_eq!(v, 12.3);
        let v = extract_socket_power(json, 1).expect("found");
        assert_eq!(v, 23.4);
    }

    #[test]
    fn parse_single_gpu() {
        // Verify that valid AMD SMI JSON output returns the GPU socket power.
        let json = r#"
{
    "gpu_data": [
        {
            "gpu": 0,
            "power": {
                "socket_power": { "value": 214.0, "unit": "W" }
            }
        }
    ]
}
"#;
        let v = extract_socket_power(json, 0).expect("found");
        assert_eq!(v, 214.0);
    }

    #[test]
    fn missing_gpu_error() {
        // Missing GPU entries should produce a domain-level MissingGpu error.
        let json = r#"{ "gpu_data": [] }"#;
        let err = extract_socket_power(json, 1).unwrap_err();
        // Ensure the top-level Error contains the DomainError::MissingGpu when downcast
        let err_str = format!("{}", err);
        assert!(err_str.contains("Missing GPU 1"));
    }

    #[test]
    fn missing_no_output() {
        let json = r#""#;
        let err = extract_socket_power(json, 0).unwrap_err();
        // Ensure the top-level Error contains the DomainError::MissingGpu when downcast
        let err_str = format!("{}", err);
        assert!(err_str.contains("JSON error: EOF"));
    }
}
