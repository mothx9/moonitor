//! Separate live health from the observation being inspected.
use super::*;

pub(super) struct FrozenView {
    pub nodes: Vec<NodeState>,
    pub reference: Instant,
    pub at_ms: u64,
}

impl App {
    pub(super) fn observed_nodes(&self) -> &[NodeState] {
        self.frozen.as_ref().map_or(&self.nodes, |f| &f.nodes)
    }
    pub(super) fn live_node<'a>(&'a self, observed: &'a NodeState) -> &'a NodeState {
        self.nodes
            .iter()
            .find(|n| n.config.id == observed.config.id)
            .unwrap_or(observed)
    }
    pub(super) fn health_status(&self, n: &NodeState) -> (&'static str, Color) {
        status(self.live_node(n))
    }
    pub(super) fn observation_stale(&self, n: &NodeState) -> bool {
        NodeStatus::of(
            n,
            self.frozen
                .as_ref()
                .map_or_else(Instant::now, |f| f.reference),
        ) != NodeStatus::Online
    }
    pub(super) fn visible_selection(&self) -> Option<String> {
        self.selected
            .as_ref()
            .filter(|id| self.filtered_nodes().iter().any(|n| &n.config.id == *id))
            .cloned()
    }
    pub(super) fn refresh_live(&mut self) {
        if !self.paused && self._engine.is_some() {
            self.view_time_ms = now_ms();
        }
        if let Some(engine) = &self._engine {
            let shared = engine.state.lock().unwrap();
            if std::env::var_os("MOONITOR_RENDER_DIAGNOSTICS").is_some() {
                self.pending_samples = shared
                    .nodes
                    .iter()
                    .filter(|n| {
                        self.nodes
                            .iter()
                            .find(|old| old.config.id == n.config.id)
                            .is_none_or(|old| old.last_seen != n.last_seen)
                    })
                    .filter_map(|n| n.last_seen)
                    .collect();
            }
            self.nodes = shared.nodes.clone();
            self.storage_error = shared.storage_error.clone();
            self.dropped_events = shared
                .dropped_events
                .load(std::sync::atomic::Ordering::Relaxed);
        }
    }
    pub(super) fn toggle_pause(&mut self) {
        if self.paused {
            self.paused = false;
            self.frozen = None;
            self.refresh_live();
        } else {
            self.refresh_live();
            self.frozen = Some(FrozenView {
                nodes: self.nodes.clone(),
                reference: Instant::now(),
                at_ms: self.view_time_ms,
            });
            self.paused = true;
        }
    }
}

pub(super) fn thermal(t: &TemperatureSnapshot, sampled_at: u64) -> Element<'_, Message> {
    let o = t.observation(sampled_at);
    let valid = o.value_celsius.is_some();
    let label = o
        .value_celsius
        .map(|v| format!("{v:.1} °C"))
        .unwrap_or_else(|| "Lettura non valida".into());
    let evidence = format!(
        "{} · {}\n{}\nValore grezzo: {} °C · campione {} ms Unix",
        o.label,
        o.source,
        o.reason(),
        o.raw_celsius,
        o.sampled_at_unix_ms
    );
    tooltip(
        row![
            text(&t.label).size(13).width(Fill),
            text(label).size(13).tone(if valid { INK } else { AMBER })
        ]
        .spacing(12)
        .width(Fill)
        .padding(iced::Padding::ZERO.right(24)),
        container(text(evidence).size(12))
            .padding(12)
            .max_width(520)
            .style(super::panel),
        tooltip::Position::Bottom,
    )
    .into()
}
