//! Dense resource workspaces, bounded row virtualization and truthful data adapters.
use super::*;
use iced::widget::column;
use std::ops::Range;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FleetSort {
    Name,
    Cpu,
    Memory,
    State,
}
pub fn visible_range(count: usize, offset: f32, height: f32, row: f32) -> Range<usize> {
    let start = ((offset.max(0.0) / row) as usize)
        .saturating_sub(1)
        .min(count.saturating_sub(1));
    start..(start + (height.max(0.0) / row).ceil() as usize + 3).min(count)
}
pub(super) fn panel<'a>(
    title: &str,
    body: impl Into<Element<'a, Message>>,
) -> widget::Container<'a, Message> {
    container(
        column![
            text(title.to_owned()).size(11).font(STRONG).tone(MUTED),
            widget::rule::horizontal(1),
            body.into()
        ]
        .spacing(9),
    )
    .padding(12)
    .width(Fill)
    .style(|theme| {
        themed_panel(
            theme,
            container::Style {
                background: Some(PAPER.into()),
                border: Border {
                    color: LINE,
                    width: 1.0,
                    radius: 7.into(),
                },
                ..Default::default()
            },
        )
    })
}
fn cell<'a>(value: &str, detail: &str) -> Element<'a, Message> {
    column![
        text(value.to_owned())
            .size(12)
            .font(MEDIUM)
            .wrapping(text::Wrapping::None),
        text(detail.to_owned())
            .size(10)
            .tone(MUTED)
            .wrapping(text::Wrapping::None)
    ]
    .spacing(3)
    .width(Fill)
    .into()
}
fn short(value: &str, count: usize) -> String {
    if value.chars().count() > count {
        format!(
            "{}…",
            value
                .chars()
                .take(count.saturating_sub(1))
                .collect::<String>()
        )
    } else {
        value.into()
    }
}
fn age(n: &NodeState) -> String {
    n.last_seen
        .map(|t| format!("{} s", t.elapsed().as_secs()))
        .unwrap_or_else(|| "—".into())
}
fn network(n: &NodeState) -> Option<&NetworkSnapshot> {
    n.sample.as_ref()?.networks.iter().max_by(|a, b| {
        (a.received_bytes_per_second + a.transmitted_bytes_per_second)
            .total_cmp(&(b.received_bytes_per_second + b.transmitted_bytes_per_second))
    })
}
fn net_cell(n: &NodeState) -> String {
    network(n)
        .map(|v| {
            format!(
                "↓ {}/s",
                bytes(Some(v.received_bytes_per_second.max(0.0) as u64))
            )
        })
        .unwrap_or_else(|| "N/D".into())
}
fn platform(n: &NodeState) -> String {
    n.sample
        .as_ref()
        .map(|s| format!("{} · {}", s.os, s.arch))
        .unwrap_or_else(|| n.config.kind.to_string())
}
fn mini_metric<'a>(
    m: Metric,
    history: &std::collections::VecDeque<HistoryPoint>,
    end: u64,
    field: u8,
    color: Color,
) -> Element<'a, Message> {
    let label = m.text();
    let explanation = m.explanation();
    let points = history
        .iter()
        .filter(|p| p.received_at_ms >= end.saturating_sub(300_000) && p.received_at_ms <= end)
        .map(|p| {
            (
                ((p.received_at_ms - end.saturating_sub(300_000)) as f32 / 300_000.0),
                match field {
                    0 => p.cpu_percent,
                    1 => p.memory_percent,
                    _ => p.gpu_percent,
                },
            )
        })
        .collect();
    tooltip(
        column![
            text(label)
                .size(12)
                .font(STRONG)
                .tone(if m.value.is_none() {
                    MUTED
                } else if m.stale {
                    AMBER
                } else {
                    color
                }),
            canvas(Trend { points, color }).width(Fill).height(16)
        ]
        .spacing(3)
        .width(Fill),
        container(text(explanation).size(11))
            .padding(9)
            .style(super::panel),
        tooltip::Position::Bottom,
    )
    .into()
}
impl App {
    pub(super) fn filtered_nodes(&self) -> Vec<&NodeState> {
        let filter = self.fleet_filter.to_lowercase();
        let mut nodes = self
            .nodes
            .iter()
            .filter(|n| match filter.as_str() {
                "online" => n.live(),
                "attention" => matches!(
                    NodeStatus::of(n, Instant::now()),
                    NodeStatus::Stale | NodeStatus::Unreachable
                ),
                "uninstrumented" => n.config.kind == crate::config::NodeKind::Unavailable,
                _ => {
                    n.config.label.to_lowercase().contains(&filter)
                        || n.config.id.to_lowercase().contains(&filter)
                }
            })
            .collect::<Vec<_>>();
        nodes.sort_by(|a, b| {
            let compare = match self.fleet_sort {
                FleetSort::Name => a
                    .config
                    .label
                    .to_lowercase()
                    .cmp(&b.config.label.to_lowercase()),
                FleetSort::Cpu => metric_value(b, 0).total_cmp(&metric_value(a, 0)),
                FleetSort::Memory => metric_value(b, 1).total_cmp(&metric_value(a, 1)),
                FleetSort::State => status(a).0.cmp(status(b).0),
            };
            compare.then_with(|| a.config.id.cmp(&b.config.id))
        });
        nodes
    }
    pub(super) fn fleet_monitor(&self) -> Element<'_, Message> {
        let fleet = FleetViewModel::new(&self.nodes, self.selected.as_deref(), Instant::now());
        let attention = self
            .nodes
            .iter()
            .filter(|n| {
                matches!(
                    NodeStatus::of(n, Instant::now()),
                    NodeStatus::Stale | NodeStatus::Unreachable
                )
            })
            .count();
        let title = row![
            column![
                text(if self.section == Section::Fleet {
                    "Host inventory"
                } else {
                    "Fleet monitor"
                })
                .size(23)
                .font(STRONG),
                text("Tutti gli host · valori osservati, stato e qualità delle sorgenti")
                    .size(11)
                    .tone(MUTED)
            ]
            .spacing(4),
            Space::new().width(Fill),
            small_button(
                if self.preferences.inspector {
                    "Chiudi inspector"
                } else {
                    "Inspector"
                },
                Message::Inspector
            ),
            small_button("+ Aggiungi host", Message::NewHost)
        ]
        .spacing(12)
        .align_y(alignment::Vertical::Center);
        let summary = row![
            small_button(
                &format!("{} / {} online", fleet.online, fleet.monitored),
                Message::FleetFilter("online".into())
            ),
            small_button(
                &format!("{} censiti", fleet.devices.len()),
                Message::FleetFilter(String::new())
            ),
            small_button(
                &format!("{attention} attenzione"),
                Message::FleetFilter("attention".into())
            ),
            small_button(
                &format!("{} non strumentati", self.nodes.len() - fleet.monitored),
                Message::FleetFilter("uninstrumented".into())
            ),
            Space::new().width(Fill),
            text("5 min · % host · RX/TX per interfaccia")
                .size(10)
                .tone(MUTED)
        ]
        .spacing(8)
        .align_y(alignment::Vertical::Center);
        let controls = row![
            text_input("Filtra nome / ID / online / attention…", &self.fleet_filter)
                .on_input(Message::FleetFilter)
                .size(12)
                .padding(7)
                .width(290),
            small_button("Nome", Message::FleetSort(FleetSort::Name)),
            small_button("CPU ↓", Message::FleetSort(FleetSort::Cpu)),
            small_button("RAM ↓", Message::FleetSort(FleetSort::Memory)),
            small_button("Stato", Message::FleetSort(FleetSort::State)),
            Space::new().width(Fill),
            self.selected
                .as_ref()
                .map(|id| small_button("Apri host →", Message::Select(id.clone())))
                .unwrap_or_else(|| Space::new().into())
        ]
        .spacing(6)
        .align_y(alignment::Vertical::Center);
        let nodes = self.filtered_nodes();
        let row_height = if self.preferences.comfortable {
            68.0
        } else {
            56.0
        };
        let viewport = (self.viewport.height - 340.0).max(150.0);
        let range = visible_range(nodes.len(), self.fleet_scroll, viewport, row_height);
        let mut rows = Column::new()
            .spacing(0)
            .push(Space::new().height(range.start as f32 * row_height));
        for n in &nodes[range.clone()] {
            rows = rows.push(self.fleet_row(n, row_height));
        }
        rows = rows.push(Space::new().height((nodes.len() - range.end) as f32 * row_height));
        if nodes.is_empty() {
            rows = rows.push(
                column![
                    text("Nessun host corrispondente").size(16),
                    small_button("Mostra tutti", Message::FleetFilter(String::new())),
                    small_button("+ Aggiungi host", Message::NewHost)
                ]
                .spacing(12)
                .padding(18),
            );
        }
        let head = row![
            text("HOST / PLATFORM")
                .size(9)
                .font(MEDIUM)
                .width(Length::FillPortion(3)),
            text("CPU").size(9).font(MEDIUM).width(Fill),
            text("MEMORY")
                .size(9)
                .font(MEDIUM)
                .width(Length::FillPortion(2)),
            text("GPU · MAX").size(9).font(MEDIUM).width(Fill),
            text("NETWORK")
                .size(9)
                .font(MEDIUM)
                .width(Length::FillPortion(2)),
            text("STATE")
                .size(9)
                .font(MEDIUM)
                .width(Length::FillPortion(2)),
            text("AGE").size(9).font(MEDIUM).width(48)
        ]
        .spacing(14)
        .padding([7, 12]);
        let matrix = panel(
            "RESOURCE MATRIX",
            column![
                controls,
                container(head).style(|theme| container::Style {
                    text_color: Some(tone(MUTED, theme)),
                    ..Default::default()
                }),
                scrollable(rows)
                    .id("fleet-matrix")
                    .on_scroll(|v| Message::FleetScroll(v.absolute_offset().y))
                    .height(Fill)
            ]
            .spacing(8),
        )
        .height(Fill);
        let mut middle = row![matrix].spacing(12).height(Fill);
        if self.preferences.inspector
            && let Some(n) = self
                .nodes
                .iter()
                .find(|n| Some(n.config.id.as_str()) == self.selected.as_deref())
        {
            middle = middle.push(container(self.inspector(n)).width(300).height(Fill));
        }
        let tracks = nodes.iter().take(5).map(|n| {
            row![
                text(short(&n.config.label, 22))
                    .size(10)
                    .tone(MUTED)
                    .width(150),
                canvas(HeatTrack {
                    history: n.history.iter().cloned().collect(),
                    end: self.view_time_ms
                })
                .width(Fill)
                .height(12)
            ]
            .spacing(10)
            .align_y(alignment::Vertical::Center)
            .into()
        });
        let timeline = panel(
            "FLEET TIMELINE / CPU · 0–100% · ultimi 5 minuti ricevuti · lacune conservate",
            Column::with_children(tracks).spacing(5),
        );
        column![title, summary, middle, timeline]
            .spacing(10)
            .height(Fill)
            .into()
    }
    fn fleet_row<'a>(&'a self, n: &'a NodeState, height: f32) -> Element<'a, Message> {
        let s = n.sample.as_ref();
        let stale = !n.live();
        let cpu = Metric::percent("CPU host", s.map(|s| &s.cpu_utilization_percent), stale);
        let memory = s
            .map(|s| Metric::memory(s, stale))
            .unwrap_or_else(|| Metric::percent("RAM", None, false));
        let gpu = s
            .map(|s| Metric::gpu(s, stale))
            .unwrap_or_else(|| Metric::percent("GPU", None, false));
        let total = s.and_then(|s| s.memory.total_bytes.value().copied());
        let memory_detail = if let Some(s) = s {
            match (total, s.memory.available_bytes.value()) {
                (Some(t), Some(a)) => {
                    format!("{} / {}", bytes(Some(t.saturating_sub(*a))), bytes(Some(t)))
                }
                _ => "N/D · contatori assenti".into(),
            }
        } else {
            "Nessun campione".into()
        };
        let net = network(n);
        let (state, color) = status(n);
        let selected = self.selected.as_deref() == Some(&n.config.id);
        let body = row![
            container(cell(&short(&n.config.label, 25), &short(&platform(n), 28)))
                .width(Length::FillPortion(3)),
            container(mini_metric(cpu, &n.history, self.view_time_ms, 0, CPU)).width(Fill),
            container(
                column![
                    text(memory.text())
                        .size(12)
                        .font(STRONG)
                        .tone(if memory.value.is_none() {
                            MUTED
                        } else if memory.stale {
                            AMBER
                        } else {
                            MEMORY
                        }),
                    text(memory_detail).size(10).tone(MUTED)
                ]
                .spacing(3)
            )
            .width(Length::FillPortion(2)),
            container(mini_metric(gpu, &n.history, self.view_time_ms, 2, GPU)).width(Fill),
            container(cell(
                &net_cell(n),
                &net.map(|v| format!(
                    "↑ {} /s · {}",
                    bytes(Some(v.transmitted_bytes_per_second.max(0.0) as u64)),
                    short(&v.name, 12)
                ))
                .unwrap_or_else(|| "Nessuna interfaccia".into())
            ))
            .width(Length::FillPortion(2)),
            text(state)
                .size(11)
                .tone(color)
                .width(Length::FillPortion(2)),
            text(age(n)).size(11).tone(MUTED).width(48)
        ]
        .spacing(14)
        .align_y(alignment::Vertical::Center);
        tooltip(button(body).padding([8,12]).width(Fill).height(height).on_press(Message::Preview(n.config.id.clone())).style(move|theme,state|themed_button(theme,button::Style{background:Some(if selected{SELECTED}else if matches!(state,button::Status::Hovered){HOVER}else{PAPER}.into()),text_color:INK,border:Border{color:LINE,width:0.5,radius:0.into()},..Default::default()})),container(text(format!("{} · {}\nCPU/RAM: osservazioni del nodo. GPU: massimo disponibile, non somma. Rete: interfaccia con traffico maggiore, senza aggregazione.\n{}",n.config.label,n.config.id,n.error.as_deref().unwrap_or("Click per selezionare; Apri host per diagnostica"))).size(11)).padding(10).style(super::panel),tooltip::Position::Bottom).into()
    }
    fn inspector<'a>(&'a self, n: &'a NodeState) -> Element<'a, Message> {
        let s = n.sample.as_ref();
        panel(
            "CONTEXT INSPECTOR",
            scrollable(
                column![
                    text(&n.config.label).size(20).font(STRONG),
                    text(status(n).0).size(12).tone(status(n).1),
                    text(platform(n)).size(11).tone(MUTED),
                    small_button("Apri host →", Message::Select(n.config.id.clone())),
                    metric(
                        Metric::percent("CPU", s.map(|s| &s.cpu_utilization_percent), !n.live()),
                        CPU
                    ),
                    canvas(chart::ResourceChart::new(&n.history, self.view_time_ms))
                        .width(Fill)
                        .height(115),
                    text(format!("Ultima ricezione: {} · ID {}", age(n), n.config.id))
                        .size(10)
                        .tone(MUTED),
                    text(
                        n.error.as_deref().unwrap_or(
                            "Collector read-only · nessuna causalità inferita dalle curve"
                        )
                    )
                    .size(11)
                    .tone(MUTED)
                ]
                .spacing(12),
            )
            .height(Fill),
        )
        .height(Fill)
        .into()
    }
    pub(super) fn host_monitor<'a>(&'a self, n: &'a NodeState) -> Element<'a, Message> {
        let heading = row![
            column![
                text(&n.config.label).size(22).font(STRONG),
                text(format!(
                    "{} · {} · {}",
                    platform(n),
                    n.config.id,
                    n.config.kind
                ))
                .size(10)
                .tone(MUTED)
            ]
            .spacing(4),
            Space::new().width(Fill),
            small_button("CPU / GPU", Message::Tab(Tab::Compute)),
            small_button("Storia →", Message::Tab(Tab::History)),
            small_button("← Fleet", Message::Navigate(Section::Overview))
        ]
        .spacing(8)
        .align_y(alignment::Vertical::Center);
        let state = row![
            text(status(n).0).size(11).font(STRONG).tone(status(n).1),
            text(format!(
                "Ultima ricezione {} · {}",
                age(n),
                n.config.ssh_alias.as_deref().unwrap_or("collector locale")
            ))
            .size(11)
            .tone(MUTED),
            Space::new().width(Fill),
            text("Read-only · dati del collector").size(10).tone(MUTED)
        ]
        .spacing(16);
        let Some(s) = n.sample.as_ref() else {
            return column![heading,state,notice(n.error.as_deref().unwrap_or(if n.config.kind==crate::config::NodeKind::Unavailable{"Host solo censito: nessun agente collegato. Aggiungi una sorgente da Setup."}else{"In attesa della prima osservazione valida. Verifica destinazione, host key e agente da Setup."})),small_button("Apri Setup",Message::Navigate(Section::Settings))].spacing(12).into();
        };
        let space = (self.viewport.height - 200.0).max(510.0);
        let cpu = column![
            metric(
                Metric::percent(
                    "CPU · utilizzo host",
                    Some(&s.cpu_utilization_percent),
                    !n.live()
                ),
                CPU
            ),
            canvas(SingleTrend::new(&n.history, self.view_time_ms, 0, CPU))
                .width(Fill)
                .height(45),
            text(format!(
                "PER-CORE · {} processori logici · % di ogni core",
                s.cpu_cores_percent.len()
            ))
            .size(9)
            .tone(MUTED),
            canvas(CoreBars(s.cpu_cores_percent.clone()))
                .width(Fill)
                .height(43),
            text(&s.cpu_name).size(10).tone(MUTED)
        ]
        .spacing(8);
        let mem = column![
            metric(Metric::memory(s, !n.live()), MEMORY),
            text(format!(
                "Totale fisico {}",
                bytes(s.memory.total_bytes.value().copied())
            ))
            .size(12),
            text(format!(
                "Disponibile {}",
                bytes(s.memory.available_bytes.value().copied())
            ))
            .size(12),
            text(format!(
                "Swap usata {}",
                bytes(s.memory.swap_used_bytes.value().copied())
            ))
            .size(12),
            text("Allocazioni GPU unified comprese nella RAM fisica.")
                .size(10)
                .tone(MUTED)
        ]
        .spacing(10);
        let mut compute = row![
            panel(
                "CPU / PROCESSOR",
                scrollable(container(cpu).padding(iced::Padding {
                    right: 12.0,
                    ..Default::default()
                }))
                .height(Fill)
            )
            .width(Length::FillPortion(5))
        ]
        .spacing(10)
        .height(space * 0.37);
        if !s.gpus.is_empty() {
            let mut gpus = Column::new().spacing(12);
            for g in &s.gpus {
                gpus = gpus.push(
                    column![
                        text(format!("{} / {}", g.id, g.name)).size(11).font(MEDIUM),
                        metric(
                            Metric::percent("GPU device", Some(&g.utilization_percent), !n.live()),
                            GPU
                        ),
                        text(format!(
                            "Memoria {} / {} · {:?}",
                            bytes(g.memory_used_bytes.value().copied()),
                            bytes(g.memory_total_bytes.value().copied()),
                            g.memory_kind
                        ))
                        .size(10)
                        .tone(MUTED),
                        text(format!(
                            "Temperatura {} · clock {}",
                            g.temperature_celsius
                                .value()
                                .map(|v| format!("{v:.0} °C"))
                                .unwrap_or_else(|| reading_reason(&g.temperature_celsius).into()),
                            g.clock_mhz
                                .value()
                                .map(|v| format!("{v} MHz"))
                                .unwrap_or_else(|| reading_reason(&g.clock_mhz).into())
                        ))
                        .size(10)
                        .tone(MUTED),
                        text(format!(
                            "Potenza board {}",
                            if s.gpus.len() == 1 {
                                watts(gpu_power(s))
                            } else {
                                "N/D · manca associazione sensore/GPU".into()
                            }
                        ))
                        .size(11),
                        small_button("Dettagli GPU / fonte →", Message::Tab(Tab::Ai))
                    ]
                    .spacing(7),
                );
            }
            compute = compute.push(
                panel(
                    "GPU / ACCELERATORS",
                    scrollable(container(gpus).padding(iced::Padding {
                        right: 12.0,
                        ..Default::default()
                    }))
                    .height(Fill),
                )
                .width(Length::FillPortion(4)),
            );
        }
        compute = compute.push(
            panel(
                "MEMORY / SWAP",
                scrollable(container(mem).padding(iced::Padding {
                    right: 12.0,
                    ..Default::default()
                }))
                .height(Fill),
            )
            .width(Length::FillPortion(3)),
        );
        let mut networks = Column::new().spacing(7);
        for net in &s.networks {
            networks = networks.push(
                row![
                    text(&net.name).size(11).width(Fill),
                    text(format!(
                        "↓ {}/s  ↑ {}/s",
                        bytes(Some(net.received_bytes_per_second.max(0.0) as u64)),
                        bytes(Some(net.transmitted_bytes_per_second.max(0.0) as u64))
                    ))
                    .size(11),
                    text(format!("{} err", net.errors)).size(10).tone(MUTED)
                ]
                .spacing(8),
            );
        }
        if s.networks.is_empty() {
            networks = networks.push(text("Nessuna interfaccia riportata").size(11).tone(MUTED));
        }
        networks = networks
            .push(
                text("Interfacce non sommate · niente storia I/O disponibile")
                    .size(9)
                    .tone(MUTED),
            )
            .push(small_button("Interfacce →", Message::Tab(Tab::Network)));
        let mut disks = Column::new().spacing(7);
        for d in &s.disks {
            disks = disks.push(
                row![
                    text(short(&d.mount, 24)).size(11).width(Fill),
                    text(format!(
                        "{} / {}",
                        bytes(Some(d.total_bytes.saturating_sub(d.available_bytes))),
                        bytes(Some(d.total_bytes))
                    ))
                    .size(11)
                ]
                .spacing(8),
            );
        }
        if s.disks.is_empty() {
            disks = disks.push(text("Nessun filesystem riportato").size(11).tone(MUTED));
        }
        disks = disks
            .push(
                text("Capacità, non I/O · SMART non strumentato")
                    .size(9)
                    .tone(MUTED),
            )
            .push(small_button("Filesystem →", Message::Tab(Tab::Disks)));
        let runtime = column![
            text("Ollama / model residency").size(11).font(MEDIUM),
            text(match s.ollama.models.value() {
                Some(m) => format!("{} modelli residenti", m.len()),
                None => reading_reason(&s.ollama.models).into(),
            })
            .size(11)
            .tone(MUTED),
            text("Richieste / TTFT / token/s: non strumentati")
                .size(10)
                .tone(MUTED),
            text("Servizi OS e container: non raccolti")
                .size(10)
                .tone(MUTED),
            small_button("Runtime e provenance →", Message::Tab(Tab::Ai))
        ]
        .spacing(9);
        let io = row![
            panel(
                "NETWORK / INTERFACES",
                scrollable(container(networks).padding(iced::Padding {
                    right: 12.0,
                    ..Default::default()
                }))
                .height(Fill)
            )
            .width(Length::FillPortion(5)),
            panel(
                "STORAGE / FILESYSTEMS",
                scrollable(container(disks).padding(iced::Padding {
                    right: 12.0,
                    ..Default::default()
                }))
                .height(Fill)
            )
            .width(Length::FillPortion(4)),
            panel(
                "RUNTIME / SOURCE COVERAGE",
                scrollable(container(runtime).padding(iced::Padding {
                    right: 12.0,
                    ..Default::default()
                }))
                .height(Fill)
            )
            .width(Length::FillPortion(3))
        ]
        .spacing(10)
        .height(space * 0.27);
        let mut processes = s.processes.iter().collect::<Vec<_>>();
        processes.sort_by(|a, b| b.cpu_percent.total_cmp(&a.cpu_percent));
        let mut ps = column![
            row![
                text("PROCESS")
                    .size(9)
                    .tone(MUTED)
                    .width(Length::FillPortion(3)),
                text("PID").size(9).tone(MUTED).width(Fill),
                text("CPU / CORE").size(9).tone(MUTED).width(Fill),
                text("RSS").size(9).tone(MUTED).width(Fill)
            ]
            .spacing(12)
        ]
        .spacing(6);
        for p in processes {
            ps = ps.push(
                row![
                    text(short(&p.name, 42))
                        .size(11)
                        .width(Length::FillPortion(3)),
                    text(p.pid.to_string()).size(11).width(Fill),
                    text(format!("{:.1}%", p.cpu_percent))
                        .size(11)
                        .tone(CPU)
                        .width(Fill),
                    text(bytes(Some(p.memory_bytes))).size(11).width(Fill)
                ]
                .spacing(12),
            );
        }
        let activity=row![panel("PROCESS EXPLORER / campione limitato · 100% = un core",column![row![small_button("Apri explorer →",Message::Tab(Tab::Processes)),text(format!("{} osservati / {} sul nodo",s.processes.len(),s.process_count)).size(10).tone(MUTED)].spacing(12),scrollable(container(ps).padding(iced::Padding{right:12.0,..Default::default()})).height(Fill)].spacing(8)).width(Length::FillPortion(8)),panel("DIAGNOSTICS / SOURCE & FRESHNESS",scrollable(column![text(format!("Campione sorgente {} ms Unix",s.sampled_at_unix_ms)).size(10),text(format!("Ricezione: {} · collector {:.2} ms",age(n),s.collector_duration_ms)).size(11),text(format!("Uptime {:.1} h · schema {}",s.uptime_seconds as f64/3600.0,s.schema_version)).size(11),text(n.error.as_deref().unwrap_or("Nessun errore di trasporto corrente")).size(11).tone(if n.error.is_some(){AMBER}else{HEALTHY}),text("Eventi persistiti e identità di avvio dei processi non ancora raccolti.").size(10).tone(MUTED),small_button("Sensori / pressione →",Message::Tab(Tab::Sensors))].spacing(10)).height(Fill)).width(Length::FillPortion(4))].spacing(10).height(space*0.36);
        column![heading, state, compute, io, activity]
            .spacing(10)
            .height(Fill)
            .into()
    }
}
fn metric_value(n: &NodeState, field: u8) -> f64 {
    n.sample
        .as_ref()
        .and_then(|s| {
            if field == 0 {
                s.cpu_utilization_percent.value().copied()
            } else {
                moonitor_store::memory_percent(s)
            }
        })
        .unwrap_or(-1.0)
}
struct HeatTrack {
    history: Vec<HistoryPoint>,
    end: u64,
}
impl canvas::Program<Message> for HeatTrack {
    type State = ();
    fn draw(
        &self,
        _: &(),
        renderer: &iced::Renderer,
        theme: &Theme,
        bounds: iced::Rectangle,
        _: iced::mouse::Cursor,
    ) -> Vec<canvas::Geometry> {
        let mut f = canvas::Frame::new(renderer, bounds.size());
        for i in 0..60 {
            let start = self.end.saturating_sub(300_000) + (i * 5000) as u64;
            let reading = self
                .history
                .iter()
                .rev()
                .find(|p| p.received_at_ms >= start && p.received_at_ms < start + 5000)
                .and_then(|p| p.cpu_percent)
                .filter(|v| v.is_finite() && (0.0..=100.0).contains(v));
            let color = reading
                .map(|v| {
                    let a = tone(LINE, theme);
                    let b = tone(CPU, theme);
                    let t = (v / 100.0) as f32;
                    Color::from_rgb(
                        a.r + (b.r - a.r) * t,
                        a.g + (b.g - a.g) * t,
                        a.b + (b.b - a.b) * t,
                    )
                })
                .unwrap_or_else(|| tone(BG, theme));
            f.fill_rectangle(
                iced::Point::new(i as f32 * bounds.width / 60.0, 2.0),
                iced::Size::new((bounds.width / 60.0 - 2.0).max(1.0), 7.0),
                color,
            );
        }
        vec![f.into_geometry()]
    }
}
struct CoreBars(Vec<f32>);
impl canvas::Program<Message> for CoreBars {
    type State = ();
    fn draw(
        &self,
        _: &(),
        renderer: &iced::Renderer,
        theme: &Theme,
        bounds: iced::Rectangle,
        _: iced::mouse::Cursor,
    ) -> Vec<canvas::Geometry> {
        let mut f = canvas::Frame::new(renderer, bounds.size());
        let width = bounds.width / self.0.len().max(1) as f32;
        for (i, value) in self.0.iter().enumerate() {
            f.fill_rectangle(
                iced::Point::new(i as f32 * width, 0.0),
                iced::Size::new((width - 3.0).max(1.0), 30.0),
                tone(LINE, theme),
            );
            let h = value.clamp(0.0, 100.0) / 100.0 * 30.0;
            f.fill_rectangle(
                iced::Point::new(i as f32 * width, 30.0 - h),
                iced::Size::new((width - 3.0).max(1.0), h),
                tone(CPU, theme),
            );
            if self.0.len() <= 32 {
                f.fill_text(canvas::Text {
                    content: i.to_string(),
                    position: iced::Point::new(i as f32 * width, 35.0),
                    color: tone(MUTED, theme),
                    size: 8.0.into(),
                    font: FONT,
                    ..Default::default()
                });
            }
        }
        vec![f.into_geometry()]
    }
}
struct SingleTrend(Trend);
impl SingleTrend {
    fn new(
        points: &std::collections::VecDeque<HistoryPoint>,
        end: u64,
        field: u8,
        color: Color,
    ) -> Self {
        Self(Trend {
            points: points
                .iter()
                .filter(|p| {
                    p.received_at_ms >= end.saturating_sub(300_000) && p.received_at_ms <= end
                })
                .map(|p| {
                    (
                        (p.received_at_ms - end.saturating_sub(300_000)) as f32 / 300_000.0,
                        if field == 0 {
                            p.cpu_percent
                        } else {
                            p.gpu_percent
                        },
                    )
                })
                .collect(),
            color,
        })
    }
}
impl canvas::Program<Message> for SingleTrend {
    type State = ();
    fn draw(
        &self,
        s: &(),
        r: &iced::Renderer,
        t: &Theme,
        b: iced::Rectangle,
        c: iced::mouse::Cursor,
    ) -> Vec<canvas::Geometry> {
        self.0.draw(s, r, t, b, c)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn virtualization_bounds_widgets_and_reaches_the_last_host() {
        for count in [0, 1, 2, 5, 16, 32, 500] {
            let top = visible_range(count, 0.0, 460.0, 56.0);
            assert!(top.len() <= 12);
            let end = visible_range(count, (count as f32 * 56.0 - 460.0).max(0.0), 460.0, 56.0);
            assert_eq!(end.end, count);
            assert!(end.len() <= 12);
        }
    }
}
