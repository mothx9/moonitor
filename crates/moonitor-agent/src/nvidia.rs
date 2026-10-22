use moonitor_core::*;
use nvml_wrapper::{
    Nvml,
    enum_wrappers::device::{Clock, TemperatureSensor},
    error::NvmlError,
};

fn reading<T>(r: Result<T, NvmlError>, ts: u64) -> Reading<T> {
    match r {
        Ok(v) => Reading::reported(v, "NVML", ts),
        Err(e) => Reading::unavailable(reason(&e)),
    }
}

fn reason(error: &NvmlError) -> UnavailableReason {
    match error {
        NvmlError::NotSupported => UnavailableReason::Unsupported,
        NvmlError::NoPermission => UnavailableReason::PermissionDenied,
        _ => UnavailableReason::TemporarilyUnavailable,
    }
}

pub fn collect(nvml: &Nvml, ts: u64) -> (Vec<GpuSnapshot>, Vec<PowerReading>) {
    let mut gpus = Vec::new();
    let mut power = Vec::new();
    for index in 0..nvml.device_count().unwrap_or(0).min(32) {
        let Ok(device) = nvml.device_by_index(index) else {
            continue;
        };
        let name = device
            .name()
            .unwrap_or_else(|_| format!("NVIDIA GPU {index}"));
        let unified = name.contains("GB10");
        let (memory_total_bytes, memory_used_bytes) = match device.memory_info() {
            Ok(m) => (
                Reading::reported(m.total, "NVML", ts),
                Reading::reported(m.used, "NVML", ts),
            ),
            Err(error) => (
                Reading::unavailable(reason(&error)),
                Reading::unavailable(reason(&error)),
            ),
        };
        gpus.push(GpuSnapshot {
            id: format!("nvidia-{index}"),
            name,
            memory_kind: if unified {
                GpuMemoryKind::Unified
            } else {
                GpuMemoryKind::Dedicated
            },
            utilization_percent: reading(device.utilization_rates().map(|u| f64::from(u.gpu)), ts),
            memory_total_bytes,
            memory_used_bytes,
            temperature_celsius: reading(
                device.temperature(TemperatureSensor::Gpu).map(f64::from),
                ts,
            ),
            clock_mhz: reading(device.clock_info(Clock::Graphics), ts),
            throttling: reading(
                device.current_throttle_reasons().map(|r| {
                    if r.is_empty() {
                        "None".into()
                    } else {
                        format!("{r:?}")
                    }
                }),
                ts,
            ),
        });
        power.push(PowerReading {
            scope: PowerScope::GpuBoard,
            watts: reading(
                device
                    .power_usage()
                    .map(|milliwatts| f64::from(milliwatts) / 1000.0),
                ts,
            ),
        });
    }
    (gpus, power)
}
