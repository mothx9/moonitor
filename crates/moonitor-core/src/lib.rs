//! Initial telemetry vocabulary shared by moonitor components.
//!
//! Missing measurements must remain distinguishable from measured zeroes.
//! The schema is provisional until validated against real platform collectors.

use serde::{Deserialize, Serialize};

pub const SCHEMA_VERSION: u32 = 1;
pub const MAX_MESSAGE_BYTES: usize = 512 * 1024;

pub fn now_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

/// Whether a value came directly from its source or was estimated.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Quality {
    Reported,
    Estimated,
}

/// Why a source cannot supply a measurement.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UnavailableReason {
    Unsupported,
    PermissionDenied,
    NotConfigured,
    TemporarilyUnavailable,
}

/// A source reading, preserving the difference between absence and zero.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum Reading<T> {
    Available {
        value: T,
        /// Collector or API that supplied the value.
        source: String,
        quality: Quality,
        /// Wall-clock collection time on the originating host, in milliseconds.
        sampled_at_unix_ms: u64,
    },
    Unavailable {
        reason: UnavailableReason,
    },
}

impl<T> Reading<T> {
    pub fn reported(value: T, source: &str, sampled_at_unix_ms: u64) -> Self {
        Self::Available {
            value,
            source: source.into(),
            quality: Quality::Reported,
            sampled_at_unix_ms,
        }
    }

    pub fn unavailable(reason: UnavailableReason) -> Self {
        Self::Unavailable { reason }
    }

    pub fn value(&self) -> Option<&T> {
        match self {
            Self::Available { value, .. } => Some(value),
            Self::Unavailable { .. } => None,
        }
    }
}

/// Distinguishes GPU memory accounting across different hardware architectures.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GpuMemoryKind {
    Dedicated,
    /// GPU allocations share the host's physical memory; do not add the totals.
    Unified,
    Unknown,
}

/// A power measurement always identifies the hardware boundary it covers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PowerScope {
    CpuPackage,
    GpuBoard,
    SystemOnChip,
    WholeDevice,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PowerReading {
    pub scope: PowerScope,
    pub watts: Reading<f64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MemorySnapshot {
    pub total_bytes: Reading<u64>,
    pub available_bytes: Reading<u64>,
    pub swap_used_bytes: Reading<u64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GpuSnapshot {
    pub id: String,
    pub name: String,
    pub memory_kind: GpuMemoryKind,
    pub utilization_percent: Reading<f64>,
    pub memory_total_bytes: Reading<u64>,
    pub memory_used_bytes: Reading<u64>,
    pub temperature_celsius: Reading<f64>,
    pub clock_mhz: Reading<u32>,
    pub throttling: Reading<String>,
}

/// A bounded update from one enrolled node; not an exhaustive process dump.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HostSnapshot {
    pub schema_version: u32,
    pub node_id: String,
    pub sampled_at_unix_ms: u64,
    pub hostname: String,
    pub os: String,
    pub arch: String,
    pub cpu_name: String,
    pub uptime_seconds: u64,
    pub cpu_utilization_percent: Reading<f64>,
    pub cpu_cores_percent: Vec<f32>,
    pub memory: MemorySnapshot,
    pub gpus: Vec<GpuSnapshot>,
    pub power: Vec<PowerReading>,
    pub processes: Vec<ProcessSnapshot>,
    pub process_count: usize,
    pub disks: Vec<DiskSnapshot>,
    pub networks: Vec<NetworkSnapshot>,
    pub temperatures: Vec<TemperatureSnapshot>,
    pub pressure: Vec<PressureSnapshot>,
    pub ollama: RuntimeSnapshot,
    pub battery_percent: Reading<f64>,
    pub collector_duration_ms: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProcessSnapshot {
    pub pid: u32,
    pub name: String,
    /// Percent of one logical core; may exceed 100 for multithreaded processes.
    pub cpu_percent: f32,
    pub memory_bytes: u64,
    pub is_ai: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DiskSnapshot {
    pub mount: String,
    pub name: String,
    pub total_bytes: u64,
    pub available_bytes: u64,
    pub sampled_at_unix_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NetworkSnapshot {
    pub name: String,
    pub received_bytes_per_second: f64,
    pub transmitted_bytes_per_second: f64,
    pub errors: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TemperatureSnapshot {
    pub label: String,
    pub celsius: f32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PressureSnapshot {
    pub resource: String,
    /// Linux PSI `some avg10`: percentage of time with at least one stalled task.
    pub some_avg10_percent: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LoadedModel {
    pub name: String,
    pub size_bytes: u64,
    pub gpu_allocation_bytes: Option<u64>,
    pub context_length: Option<u64>,
    pub quantization: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RuntimeSnapshot {
    /// An empty available list means the runtime answered and no model is loaded.
    pub models: Reading<Vec<LoadedModel>>,
}

impl HostSnapshot {
    pub fn validate(&self, expected_id: &str) -> Result<(), String> {
        if self.schema_version != SCHEMA_VERSION {
            return Err("Unsupported telemetry schema".into());
        }
        if self.node_id != expected_id {
            return Err("Unexpected node identity".into());
        }
        if self.cpu_cores_percent.len() > 512
            || self.processes.len() > 128
            || self.gpus.len() > 32
            || self.disks.len() > 128
            || self.networks.len() > 128
            || self.temperatures.len() > 128
        {
            return Err("Telemetry collection exceeds limits".into());
        }
        if self
            .cpu_utilization_percent
            .value()
            .is_some_and(|n| !n.is_finite() || !(0.0..=100.0).contains(n))
        {
            return Err("Invalid CPU utilization".into());
        }
        Ok(())
    }
}
