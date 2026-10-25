//! Explicit synthetic QA fixtures. This module is never compiled into production.
use crate::{
    config::{NodeConfig, NodeKind, VisualKind},
    engine::NodeState,
};
use moonitor_core::*;
use std::{
    sync::Arc,
    time::{Duration, Instant},
};
pub const TIME_MS: u64 = 1_792_742_400_000;
pub fn sample(id: &str) -> HostSnapshot {
    let t = TIME_MS;
    HostSnapshot {
        schema_version: SCHEMA_VERSION,
        node_id: id.into(),
        sampled_at_unix_ms: t,
        hostname: "QA fixture".into(),
        os: "Fixture OS".into(),
        arch: "fixture".into(),
        cpu_name: "Fixture CPU".into(),
        uptime_seconds: 3600,
        cpu_utilization_percent: Reading::reported(0.0, "QA CPU", t),
        cpu_cores_percent: vec![0.0; 4],
        memory: MemorySnapshot {
            total_bytes: Reading::reported(24 << 30, "QA memory", t),
            available_bytes: Reading::reported(12 << 30, "QA memory", t),
            swap_used_bytes: Reading::reported(0, "QA memory", t),
        },
        gpus: vec![GpuSnapshot {
            id: "gpu-0".into(),
            name: "Fixture unified GPU".into(),
            memory_kind: GpuMemoryKind::Unified,
            utilization_percent: Reading::reported(37.0, "QA GPU", t),
            memory_total_bytes: Reading::unavailable(UnavailableReason::Unsupported),
            memory_used_bytes: Reading::unavailable(UnavailableReason::Unsupported),
            temperature_celsius: Reading::unavailable(UnavailableReason::PermissionDenied),
            clock_mhz: Reading::unavailable(UnavailableReason::TemporarilyUnavailable),
            throttling: Reading::unavailable(UnavailableReason::Unsupported),
        }],
        power: vec![],
        processes: vec![ProcessSnapshot {
            pid: 100,
            name: "fixture-inference".into(),
            cpu_percent: 25.0,
            memory_bytes: 1 << 30,
            is_ai: true,
        }],
        process_count: 1,
        disks: vec![],
        networks: vec![],
        temperatures: vec![],
        pressure: vec![],
        ollama: RuntimeSnapshot {
            models: if id == "qa-0" {
                Reading::reported(
                    vec![LoadedModel {
                        name: "QA model · fixture".into(),
                        size_bytes: 4 << 30,
                        gpu_allocation_bytes: None,
                        context_length: Some(8192),
                        quantization: Some("QA Q4".into()),
                    }],
                    "QA runtime",
                    t,
                )
            } else {
                Reading::unavailable(UnavailableReason::NotConfigured)
            },
        },
        battery_percent: Reading::unavailable(UnavailableReason::Unsupported),
        collector_duration_ms: 1.0,
    }
}
pub fn fleet(count: usize) -> Vec<NodeState> {
    (0..count)
        .map(|i| {
            let id = format!("qa-{i}");
            NodeState {
                capacity_limited: false,
                config: NodeConfig {
                    id: id.clone(),
                    label: format!("QA device {i}"),
                    kind: if i % 5 == 4 {
                        NodeKind::Unavailable
                    } else {
                        NodeKind::Ssh
                    },
                    visual_kind: [
                        VisualKind::Laptop,
                        VisualKind::Tower,
                        VisualKind::Compact,
                        VisualKind::Board,
                        VisualKind::Phone,
                    ][i % 5],
                    ssh_alias: Some("fixture-only".into()),
                    ssh_user: None,
                    ssh_port: None,
                    remote_agent: None,
                    note: "DATI SINTETICI · SOLO TEST".into(),
                },
                sample: if i % 5 < 2 {
                    Some(Arc::new(sample(&id)))
                } else {
                    None
                },
                last_seen: if i % 5 < 2 {
                    Some(Instant::now() - Duration::from_secs(if i % 5 == 1 { 10 } else { 0 }))
                } else {
                    None
                },
                error: if i % 5 == 2 {
                    Some("QA connection failure".into())
                } else {
                    None
                },
                history: if i % 5 < 2 {
                    (0..60)
                        .map(|j| moonitor_store::HistoryPoint {
                            received_at_ms: TIME_MS - (59 - j) * 5_000,
                            cpu_percent: if (20..25).contains(&j) {
                                None
                            } else {
                                Some((j * 7 % 35) as f64)
                            },
                            memory_percent: Some(50.0),
                            gpu_percent: if (20..25).contains(&j) {
                                None
                            } else {
                                Some((j * 11 % 70) as f64)
                            },
                        })
                        .collect()
                } else {
                    Default::default()
                },
            }
        })
        .collect()
}
