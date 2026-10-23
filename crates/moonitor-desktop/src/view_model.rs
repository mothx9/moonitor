//! Presentation contracts: no collectors, storage or fabricated observations.
use crate::{
    config::{NodeKind, VisualKind},
    engine::NodeState,
};
use moonitor_core::{Reading, UnavailableReason};
use std::time::{Duration, Instant};

pub const FRESH_FOR: Duration = Duration::from_secs(4);
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NodeStatus {
    Online,
    Stale,
    Unreachable,
    Uninstrumented,
    Connecting,
}
impl NodeStatus {
    pub fn of(node: &NodeState, now: Instant) -> Self {
        if matches!(node.config.kind, NodeKind::Unavailable) {
            Self::Uninstrumented
        } else if node.sample.is_some() {
            if node.error.is_none()
                && node
                    .last_seen
                    .is_some_and(|t| now.saturating_duration_since(t) < FRESH_FOR)
            {
                Self::Online
            } else {
                Self::Stale
            }
        } else if node.error.is_some() {
            Self::Unreachable
        } else {
            Self::Connecting
        }
    }
    pub fn label(self) -> &'static str {
        match self {
            Self::Online => "Online",
            Self::Stale => "Obsoleto",
            Self::Unreachable => "Non raggiungibile",
            Self::Uninstrumented => "Non strumentato",
            Self::Connecting => "In connessione",
        }
    }
    pub fn color(self) -> [f32; 3] {
        match self {
            Self::Online => [0.23, 0.49, 0.36],
            Self::Stale => [0.70, 0.45, 0.15],
            Self::Unreachable => [0.72, 0.25, 0.23],
            Self::Uninstrumented => [0.48, 0.51, 0.50],
            Self::Connecting => [0.35, 0.46, 0.65],
        }
    }
}
#[derive(Debug, Clone, PartialEq)]
pub struct DeviceSceneModel {
    pub id: String,
    pub label: String,
    pub kind: VisualKind,
    pub status: NodeStatus,
    pub position: [f32; 3],
}
pub struct FleetViewModel {
    pub devices: Vec<DeviceSceneModel>,
    pub selected_id: Option<String>,
    pub monitored: usize,
    pub online: usize,
}
impl FleetViewModel {
    pub fn new(nodes: &[NodeState], selected: Option<&str>, now: Instant) -> Self {
        let columns = (nodes.len() as f32).sqrt().ceil().max(1.0) as usize;
        let rows = nodes.len().div_ceil(columns);
        let devices = nodes
            .iter()
            .enumerate()
            .map(|(i, n)| DeviceSceneModel {
                id: n.config.id.clone(),
                label: n.config.label.clone(),
                kind: n.config.visual_kind,
                status: NodeStatus::of(n, now),
                position: [
                    ((i % columns) as f32 - (columns - 1) as f32 / 2.0) * 2.6,
                    0.0,
                    ((i / columns) as f32 - rows.saturating_sub(1) as f32 / 2.0) * 2.6,
                ],
            })
            .collect::<Vec<_>>();
        let selected_id = selected
            .filter(|id| devices.iter().any(|d| d.id == *id))
            .map(str::to_owned)
            .or_else(|| devices.first().map(|d| d.id.clone()));
        Self {
            monitored: devices
                .iter()
                .filter(|d| d.status != NodeStatus::Uninstrumented)
                .count(),
            online: devices
                .iter()
                .filter(|d| d.status == NodeStatus::Online)
                .count(),
            devices,
            selected_id,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Metric {
    pub label: String,
    pub value: Option<f64>,
    pub detail: String,
    pub provenance: String,
    pub stale: bool,
}
impl Metric {
    pub fn percent(label: &str, reading: Option<&Reading<f64>>, stale: bool) -> Self {
        let (value, provenance) = match reading {
            Some(Reading::Available {
                value,
                source,
                quality,
                sampled_at_unix_ms,
            }) if value.is_finite() && (0.0..=100.0).contains(value) => (
                Some(*value),
                format!(
                    "Fonte: {source} · qualità: {quality:?} · campionato: {sampled_at_unix_ms} ms Unix"
                ),
            ),
            Some(Reading::Available { .. }) => {
                (None, "Valore fuori intervallo: atteso 0–100%".into())
            }
            Some(Reading::Unavailable { reason }) => (None, unavailable(*reason).into()),
            None => (None, "Nessuna osservazione ricevuta".into()),
        };
        Self {
            label: label.into(),
            value,
            detail: String::new(),
            provenance,
            stale,
        }
    }
    pub fn memory(snapshot: &moonitor_core::HostSnapshot, stale: bool) -> Self {
        let mut m = Self::percent("Memoria", None, stale);
        match (
            &snapshot.memory.total_bytes,
            &snapshot.memory.available_bytes,
        ) {
            (
                Reading::Available {
                    value: total,
                    source,
                    sampled_at_unix_ms,
                    ..
                },
                Reading::Available {
                    value: available,
                    source: available_source,
                    sampled_at_unix_ms: available_at,
                    ..
                },
            ) if *total > 0 && available <= total => {
                m.value = Some((*total - *available) as f64 / *total as f64 * 100.0);
                m.provenance = format!(
                    "Derivata: (totale − disponibile) / totale · fonti {source}, {available_source} · timestamp {sampled_at_unix_ms}, {available_at} ms Unix"
                );
            }
            (Reading::Unavailable { reason }, _) | (_, Reading::Unavailable { reason }) => {
                m.provenance = unavailable(*reason).into()
            }
            _ => m.provenance = "Memoria: contatori non validi".into(),
        }
        m
    }
    pub fn gpu(snapshot: &moonitor_core::HostSnapshot, stale: bool) -> Self {
        let reading = snapshot
            .gpus
            .iter()
            .filter(|g| {
                g.utilization_percent
                    .value()
                    .is_some_and(|v| v.is_finite() && (0.0..=100.0).contains(v))
            })
            .max_by(|a, b| {
                a.utilization_percent
                    .value()
                    .unwrap()
                    .total_cmp(b.utilization_percent.value().unwrap())
            })
            .or_else(|| snapshot.gpus.first());
        let mut m = Self::percent(
            if snapshot.gpus.len() > 1 {
                "GPU · massimo disponibile"
            } else {
                "GPU"
            },
            reading.map(|g| &g.utilization_percent),
            stale,
        );
        if let Some(g) = reading {
            m.detail = g.name.clone();
        }
        if snapshot.gpus.len() > 1 {
            m.provenance.push_str(&format!(
                " · massimo fra {} GPU censite; vedere AI locale per ogni GPU",
                snapshot.gpus.len()
            ));
        }
        m
    }
    pub fn text(&self) -> String {
        self.value
            .map(|v| format!("{v:.0}%"))
            .unwrap_or_else(|| "N/D".into())
    }
    pub fn explanation(&self) -> String {
        let state = if self.stale {
            "Ultima lettura conservata · obsoleta"
        } else if self.value.is_none() {
            "Dato non disponibile"
        } else {
            "Campione corrente"
        };
        format!(
            "{} · {}\n{}\n{}",
            self.label,
            self.text(),
            self.provenance,
            state
        )
    }
}
pub fn unavailable(reason: UnavailableReason) -> &'static str {
    match reason {
        UnavailableReason::Unsupported => "Non supportato dal sensore",
        UnavailableReason::PermissionDenied => "Permesso negato",
        UnavailableReason::NotConfigured => "Sorgente non configurata",
        UnavailableReason::TemporarilyUnavailable => "Temporaneamente non disponibile",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::NodeConfig;
    pub fn nodes(count: usize) -> Vec<NodeState> {
        (0..count)
            .map(|i| NodeState {
                config: NodeConfig {
                    id: format!("node-{i}"),
                    label: format!("Node {i}"),
                    kind: NodeKind::Unavailable,
                    visual_kind: VisualKind::Compact,
                    ssh_alias: None,
                    remote_agent: None,
                    note: String::new(),
                },
                sample: None,
                last_seen: None,
                error: None,
                history: Default::default(),
            })
            .collect()
    }
    #[test]
    fn fleet_counts_selection_and_layout_follow_identity() {
        for count in [0, 1, 2, 5, 16, 32] {
            let nodes = nodes(count);
            let vm = FleetViewModel::new(&nodes, Some("missing"), Instant::now());
            assert_eq!(vm.devices.len(), count);
            assert_eq!(vm.monitored, 0);
            assert_eq!(vm.online, 0);
            assert_eq!(vm.selected_id.is_some(), count > 0);
            for (i, a) in vm.devices.iter().enumerate() {
                for b in &vm.devices[i + 1..] {
                    assert_ne!(a.position, b.position);
                }
            }
        }
        let mut nodes = nodes(5);
        nodes.reverse();
        assert_eq!(
            FleetViewModel::new(&nodes, Some("node-1"), Instant::now())
                .selected_id
                .as_deref(),
            Some("node-1")
        );
        nodes.truncate(1);
        let vm = FleetViewModel::new(&nodes, Some("node-1"), Instant::now());
        assert_eq!(vm.devices.len(), 1);
        assert_eq!(vm.selected_id.as_deref(), Some("node-4"));
    }
    #[test]
    fn freshness_and_instrumentation_are_distinct() {
        let nodes = crate::fixtures::fleet(5);
        let now = Instant::now();
        let vm = FleetViewModel::new(&nodes, None, now);
        assert_eq!(vm.monitored, 4);
        assert_eq!(vm.online, 1);
        assert_eq!(
            vm.devices.iter().map(|d| d.status).collect::<Vec<_>>(),
            vec![
                NodeStatus::Online,
                NodeStatus::Stale,
                NodeStatus::Unreachable,
                NodeStatus::Connecting,
                NodeStatus::Uninstrumented
            ]
        );
        assert_eq!(
            NodeStatus::of(&nodes[0], now + FRESH_FOR),
            NodeStatus::Stale
        );
    }
    #[test]
    fn unified_memory_is_not_added_and_gpu_max_is_named() {
        let mut s = crate::fixtures::sample("qa");
        assert_eq!(Metric::memory(&s, false).value, Some(50.0));
        assert!(s.gpus[0].memory_total_bytes.value().is_none());
        let mut gpu = s.gpus[0].clone();
        gpu.id = "gpu-1".into();
        gpu.utilization_percent = Reading::reported(80.0, "QA GPU 1", 10);
        s.gpus.push(gpu);
        let m = Metric::gpu(&s, false);
        assert_eq!(m.value, Some(80.0));
        assert!(m.label.contains("massimo"));
        assert_eq!(s.gpus.len(), 2);
    }
    #[test]
    fn zero_and_absence_have_different_values_and_all_reasons_survive() {
        let zero = Reading::Available {
            value: 0.0,
            source: "fixture".into(),
            quality: moonitor_core::Quality::Reported,
            sampled_at_unix_ms: 10,
        };
        assert_eq!(Metric::percent("CPU", Some(&zero), false).text(), "0%");
        let mut reasons = std::collections::HashSet::new();
        for reason in [
            UnavailableReason::Unsupported,
            UnavailableReason::PermissionDenied,
            UnavailableReason::NotConfigured,
            UnavailableReason::TemporarilyUnavailable,
        ] {
            let m = Metric::percent("GPU", Some(&Reading::Unavailable { reason }), false);
            assert_eq!(m.text(), "N/D");
            assert!(m.value.is_none());
            reasons.insert(m.provenance);
        }
        assert_eq!(reasons.len(), 4);
        assert!(Metric::percent("CPU", None, false).value.is_none());
    }
}
