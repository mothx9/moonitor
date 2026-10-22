//! Initial telemetry vocabulary shared by moonitor components.
//!
//! Missing measurements must remain distinguishable from measured zeroes.
//! The schema is provisional until validated against real platform collectors.

use serde::{Deserialize, Serialize};

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
}

/// A bounded update from one enrolled node; not an exhaustive process dump.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HostSnapshot {
    pub schema_version: u32,
    pub node_id: String,
    pub cpu_utilization_percent: Reading<f64>,
    pub memory: MemorySnapshot,
    pub gpus: Vec<GpuSnapshot>,
    pub power: Vec<PowerReading>,
}
