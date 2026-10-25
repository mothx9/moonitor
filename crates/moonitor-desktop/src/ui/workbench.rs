//! V3 native instrument shell. The HTML concept is a design reference only.
use super::*;
use iced::widget::column;
use iced::widget::pick_list;
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Section {
    Overview,
    Fleet,
    Metrics,
    Processes,
    Workloads,
    Energy,
    Events,
    Explore,
    Settings,
    Device,
}
pub(super) const SECTIONS: [Section; 7] = [
    Section::Overview,
    Section::Fleet,
    Section::Metrics,
    Section::Processes,
    Section::Explore,
    Section::Events,
    Section::Settings,
];
impl Section {
    pub fn parse(value: &str) -> Option<Self> {
        Some(match value.to_ascii_lowercase().as_str() {
            "overview" | "fleet" => Self::Overview,
            "hosts" => Self::Fleet,
            "metrics" => Self::Metrics,
            "processes" => Self::Processes,
            "workloads" => Self::Workloads,
            "energy" => Self::Energy,
            "events" => Self::Events,
            "investigate" | "explore" => Self::Explore,
            "setup" | "settings" => Self::Settings,
            "host" | "device" => Self::Device,
            _ => return None,
        })
    }
    pub(super) fn label(self) -> &'static str {
        match self {
            Self::Overview => "Fleet",
            Self::Fleet => "Hosts",
            Self::Metrics => "Metrics",
            Self::Processes => "Processes",
            Self::Workloads => "AI runtimes",
            Self::Energy => "Power",
            Self::Events => "Events",
            Self::Explore => "Investigate",
            Self::Settings => "Setup",
            Self::Device => "Host monitor",
        }
    }
    fn icon(self) -> &'static str {
        match self {
            Self::Overview => "overview",
            Self::Fleet | Self::Device => "network",
            Self::Metrics | Self::Workloads => "ai",
            Self::Processes => "process",
            Self::Energy => "energy",
            Self::Events => "events",
            Self::Explore => "history",
            Self::Settings => "settings",
        }
    }
}
#[derive(Debug, Clone, PartialEq, Eq)]
struct NodeChoice {
    id: String,
    label: String,
}
impl std::fmt::Display for NodeChoice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.label)
    }
}
impl App {
    pub(super) fn workbench_view(&self) -> Element<'_, Message> {
        let node = self
            .observed_nodes()
            .iter()
            .find(|n| Some(n.config.id.as_str()) == self.selected.as_deref());
        let choices = self
            .nodes
            .iter()
            .map(|n| NodeChoice {
                id: n.config.id.clone(),
                label: n.config.label.clone(),
            })
            .collect::<Vec<_>>();
        let selected = choices
            .iter()
            .find(|n| Some(n.id.as_str()) == self.selected.as_deref())
            .cloned();
        let switcher = pick_list(choices, selected, |n| Message::SwitchNode(n.id))
            .placeholder("Seleziona host")
            .text_size(12)
            .padding([7, 10])
            .width(180);
        let top = row![
            text("moonitor").size(18).font(STRONG),
            text("/").size(14).tone(MUTED),
            text(self.section.label()).size(12).font(MEDIUM),
            Space::new().width(Fill),
            text(self.frozen.as_ref().map_or_else(
                || "LIVE · aggiornamento 1 s".to_owned(),
                |f| {
                    let sec = f.at_ms / 1000;
                    format!(
                        "VISTA CONGELATA · {:02}:{:02}:{:02} UTC · salute live",
                        sec / 3600 % 24,
                        sec / 60 % 60,
                        sec % 60
                    )
                }
            ))
            .size(10)
            .tone(if self.paused { AMBER } else { HEALTHY }),
            switcher,
            small_button(
                if self.paused { "Riprendi" } else { "Pausa" },
                Message::Pause
            )
        ]
        .spacing(16)
        .align_y(alignment::Vertical::Center);
        let content = match self.section {
            Section::Overview | Section::Fleet => self.fleet_monitor(),
            Section::Device if self.tab == Tab::Overview => node
                .map(|n| self.host_monitor(n))
                .unwrap_or_else(empty_fleet),
            Section::Device => node.map(|n| self.device(n)).unwrap_or_else(empty_fleet),
            Section::Settings => self.setup_view(),
            Section::Metrics => self.metrics_catalog(node),
            Section::Processes => node
                .map(|n| self.device_processes(n))
                .unwrap_or_else(empty_fleet),
            Section::Explore => node.map(|n| self.explore(n)).unwrap_or_else(empty_fleet),
            Section::Events => self.events(),
            Section::Workloads => self.workloads(),
            Section::Energy => self.energy(),
        };
        let fixed = matches!(self.section, Section::Overview | Section::Fleet)
            || (self.section == Section::Device && self.tab == Tab::Overview);
        let content: Element<'_, Message> = if fixed {
            content
        } else {
            scrollable(content).height(Fill).into()
        };
        let mut body = column![content].spacing(8);
        if self.section != Section::Settings
            && let Some(error) = &self.setup_error
        {
            body = body.push(notice(error));
        }
        if let Some(e) = &self.storage_error {
            body = body.push(notice(&format!("Archivio: {e}")));
        }
        if self.dropped_events > 0 {
            body = body.push(notice(&format!(
                "Coda: {} aggiornamenti scartati",
                self.dropped_events
            )));
        }
        container(row![
            self.rail(),
            column![
                container(top)
                    .padding([9, 18])
                    .height(52)
                    .width(Fill)
                    .style(|theme| themed_panel(
                        theme,
                        container::Style {
                            background: Some(PAPER.into()),
                            border: Border {
                                color: LINE,
                                width: 1.0,
                                ..Default::default()
                            },
                            ..Default::default()
                        }
                    )),
                container(body).padding(16).width(Fill).height(Fill)
            ]
            .width(Fill)
            .height(Fill)
        ])
        .width(Fill)
        .height(Fill)
        .style(|theme| {
            themed_panel(
                theme,
                container::Style {
                    background: Some(BG.into()),
                    text_color: Some(INK),
                    ..Default::default()
                },
            )
        })
        .into()
    }
    fn rail(&self) -> Element<'_, Message> {
        let expanded = self.preferences.expanded_rail;
        let mut nav = column![
            container(brand_icon()).padding([4, 3]),
            Space::new().height(14)
        ]
        .spacing(8);
        for section in SECTIONS {
            let selected = self.section == section
                || (self.section == Section::Device && section == Section::Fleet);
            let color = if selected { ACCENT_SOFT } else { NAV_MUTED };
            let icon = canvas(Icon(section.icon(), color)).width(20).height(20);
            let inner: Element<'_, Message> = if expanded {
                row![icon, text(section.label()).size(12)]
                    .spacing(12)
                    .align_y(alignment::Vertical::Center)
                    .into()
            } else {
                icon.into()
            };
            let action = button(inner)
                .padding(11)
                .width(Fill)
                .on_press(Message::Navigate(section))
                .style(move |_, state| button::Style {
                    background: Some(
                        if selected {
                            NAV_SELECTED
                        } else if matches!(state, button::Status::Hovered) {
                            NAV_HOVER
                        } else {
                            NAV
                        }
                        .into(),
                    ),
                    text_color: color,
                    border: Border {
                        radius: 8.into(),
                        ..Default::default()
                    },
                    ..Default::default()
                });
            nav = nav.push(tooltip(
                action,
                container(text(section.label()).size(12))
                    .padding(8)
                    .style(panel),
                tooltip::Position::Right,
            ));
        }
        container(
            column![
                nav,
                Space::new().height(Fill),
                tooltip(
                    rail_control(
                        text(if expanded { "Comprimi ←" } else { "☰" }).size(12),
                        Message::Rail
                    ),
                    "Espandi / comprimi navigazione",
                    tooltip::Position::Right
                ),
                tooltip(
                    rail_control(
                        canvas(Icon("network", NAV_TEXT)).width(20).height(20),
                        Message::NewHost
                    ),
                    "Aggiungi host",
                    tooltip::Position::Right
                )
            ]
            .spacing(8),
        )
        .padding([12, 8])
        .width(if expanded { 180 } else { 64 })
        .height(Fill)
        .style(|_| container::Style {
            background: Some(NAV.into()),
            ..Default::default()
        })
        .into()
    }
    fn device<'a>(&'a self, n: &'a NodeState) -> Element<'a, Message> {
        let tabs = Row::with_children(
            TABS.iter()
                .map(|tab| small_button(tab.label(), Message::Tab(*tab))),
        )
        .spacing(4);
        column![
            row![
                text(&n.config.label).size(22).font(STRONG),
                Space::new().width(Fill),
                small_button("← Fleet", Message::Navigate(Section::Overview))
            ],
            tabs,
            self.node_heading(n),
            self.details(n)
        ]
        .spacing(12)
        .into()
    }
    fn device_processes<'a>(&'a self, n: &'a NodeState) -> Element<'a, Message> {
        column![
            text(format!("Processes / {}", n.config.label))
                .size(22)
                .font(STRONG),
            n.sample
                .as_ref()
                .map(|s| self.processes(s, false))
                .unwrap_or_else(|| self.details(n))
        ]
        .spacing(12)
        .into()
    }
    fn metrics_catalog<'a>(&'a self, node: Option<&'a NodeState>) -> Element<'a, Message> {
        let mut c=column![text("Metric catalog").size(23).font(STRONG),text("Capacità delle sorgenti del nodo selezionato. Apri una famiglia per ispezionare valori e provenance.").size(12).tone(MUTED)].spacing(12);
        for (title, detail, tab) in [
            (
                "CPU / cores",
                "Utilizzo aggregato e per core logico",
                Tab::Compute,
            ),
            (
                "Memory / pressure",
                "RAM fisica, swap e pressione dove misurata",
                Tab::History,
            ),
            (
                "GPU / accelerators",
                "Dispositivi separati, memoria, clock e limiti di fonte",
                Tab::Ai,
            ),
            (
                "Network / interfaces",
                "RX/TX per interfaccia, senza somme di bridge e tunnel",
                Tab::Network,
            ),
            (
                "Storage / filesystems",
                "Capacità per mount; I/O e SMART non ancora strumentati",
                Tab::Disks,
            ),
            (
                "Sensors",
                "Temperature, batteria e PSI dove disponibili",
                Tab::Sensors,
            ),
        ] {
            c = c.push(instrument::panel(
                title,
                row![
                    text(detail).size(12).tone(MUTED).width(Fill),
                    small_button("Ispeziona →", Message::Tab(tab))
                ]
                .spacing(12),
            ));
        }
        if node.is_none() {
            c = c.push(empty_fleet());
        }
        c.push(
            row![
                small_button("AI runtimes", Message::Navigate(Section::Workloads)),
                small_button("Potenza misurata", Message::Navigate(Section::Energy))
            ]
            .spacing(10),
        )
        .into()
    }
    fn model_table(&self, compact: bool) -> Element<'_, Message> {
        let mut rows = column![
            row![
                text("Modelli osservati").size(16).font(STRONG),
                Space::new().width(Fill),
                small_button("Metriche →", Message::Navigate(Section::Metrics))
            ]
            .align_y(alignment::Vertical::Center),
            table_row(["MODELLO", "NODO", "RUNTIME", "MEMORIA MODELLO"], true)
        ]
        .spacing(8);
        let mut count = 0;
        for n in self.observed_nodes() {
            if let Some(s) = &n.sample
                && let Some(models) = s.ollama.models.value()
            {
                for m in models {
                    if !compact || count < 3 {
                        rows = rows.push(table_row(
                            [
                                &m.name,
                                &n.config.label,
                                if !self.observation_stale(n) {
                                    "Ollama · residente"
                                } else {
                                    "Ollama · obsoleto"
                                },
                                &bytes(Some(m.size_bytes)),
                            ],
                            false,
                        ));
                    }
                    count += 1;
                }
            }
        }
        if count == 0 {
            rows=rows.push(text("Nessun modello residente osservato. La disponibilità dei runtime è indicata nelle sorgenti.").size(12).tone(MUTED));
        }
        if !compact {
            rows=rows.push(text("Residenza del modello ≠ inferenza attiva. Token/s, TTFT e code richiedono metriche delle richieste.").size(11).tone(MUTED));
        }
        card(rows).into()
    }
    fn workloads(&self) -> Element<'_, Message> {
        let mut runtime = column![
            text("Runtime · copertura delle sorgenti")
                .size(17)
                .font(STRONG)
        ]
        .spacing(12);
        for n in self.observed_nodes() {
            let state = n
                .sample
                .as_ref()
                .map(|s| reading_reason(&s.ollama.models))
                .unwrap_or("Nessuna osservazione");
            runtime = runtime.push(table_row(
                [&n.config.label, "Ollama", state, self.health_status(n).0],
                false,
            ));
        }
        column![self.model_table(false),card(runtime),notice("Questa vista osserva lo stato dei modelli. Tempi e risorse delle singole richieste non sono ancora esposti dalle sorgenti collegate.")].spacing(18).into()
    }
    fn measured_power(&self) -> (Option<f64>, usize) {
        let values = self
            .observed_nodes()
            .iter()
            .filter(|n| !self.observation_stale(n))
            .filter_map(|n| n.sample.as_ref())
            .filter_map(|s| gpu_power(s))
            .collect::<Vec<_>>();
        (
            (!values.is_empty()).then(|| values.iter().sum()),
            values.len(),
        )
    }
    fn energy(&self) -> Element<'_, Message> {
        let power = self.measured_power();
        let mut rows = column![
            text("Misure di potenza GPU").size(17).font(STRONG),
            table_row(["NODO", "POTENZA", "CONFINE", "STATO"], true)
        ]
        .spacing(12);
        for n in self.observed_nodes() {
            rows = rows.push(table_row(
                [
                    &n.config.label,
                    &watts(n.sample.as_ref().and_then(|s| gpu_power(s))),
                    "GPU board",
                    self.health_status(n).0,
                ],
                false,
            ));
        }
        column![row![stat("POTENZA GPU MISURATA",&watts(power.0),&format!("{} nodi con lettura corrente",power.1)),stat("INTERO LABORATORIO","N/D","Nessun totale whole-device disponibile"),stat("ENERGIA","N/D","Intervalli e copertura non disponibili")].spacing(12),card(rows),notice("La potenza GPU non rappresenta il consumo totale del dispositivo. La memoria unificata non aggiunge capacità alla RAM del nodo.")].spacing(18).into()
    }
    fn source_summary(&self) -> Element<'_, Message> {
        let mut row = Row::new().spacing(18);
        for n in self.observed_nodes() {
            let (state, color) = self.health_status(n);
            row = row.push(
                button(
                    column![
                        text(&n.config.label).size(12).font(MEDIUM),
                        text(state).size(11).tone(color)
                    ]
                    .spacing(5),
                )
                .on_press(Message::Select(n.config.id.clone()))
                .style(button::text),
            );
        }
        card(
            column![
                text("Sorgenti").size(14).font(STRONG),
                scrollable(row).direction(scrollable::Direction::Horizontal(
                    scrollable::Scrollbar::default()
                ))
            ]
            .spacing(10),
        )
        .into()
    }
    fn events(&self) -> Element<'_, Message> {
        let mut events =
            column![text("Diagnostica della raccolta").size(17).font(STRONG)].spacing(12);
        let mut count = 0;
        for n in &self.nodes {
            if let Some(error) = &n.error {
                events = events.push(notice(&format!("{} · {error}", n.config.label)));
                count += 1;
            }
        }
        if let Some(error) = &self.storage_error {
            events = events.push(notice(error));
            count += 1;
        }
        if count == 0 {
            events = events.push(
                text("Nessun errore di raccolta segnalato.")
                    .size(13)
                    .tone(ACCENT),
            );
        }
        column![card(events),card(column![text("Timeline eventi non disponibile").size(16).font(STRONG),text("L'archivio corrente conserva riepiloghi delle metriche. Non contiene ancora eventi storici di connessione o esecuzione dei workload.").size(13).tone(MUTED),small_button("Controlla la flotta →",Message::Navigate(Section::Fleet))].spacing(12)),self.source_summary()].spacing(18).into()
    }
    fn explore<'a>(&self, n: &'a NodeState) -> Element<'a, Message> {
        let mut raw = column![table_row(
            ["RICEVUTO UTC", "CPU HOST", "RAM", "GPU MAX"],
            true
        )]
        .spacing(4);
        for point in n
            .history
            .iter()
            .rev()
            .filter(|p| p.received_at_ms <= self.view_time_ms)
            .take(10)
        {
            let seconds = point.received_at_ms / 1000;
            raw = raw.push(table_row(
                [
                    &format!(
                        "{:02}:{:02}:{:02}.{:03}",
                        seconds / 3600 % 24,
                        seconds / 60 % 60,
                        seconds % 60,
                        point.received_at_ms % 1000
                    ),
                    &pct(point.cpu_percent),
                    &pct(point.memory_percent),
                    &pct(point.gpu_percent),
                ],
                false,
            ));
        }
        column![
            text(format!("Investigate / {}",n.config.label)).size(22).font(STRONG),
            text("Ultimi 5 minuti ricevuti · CPU / RAM / GPU massimo disponibile · 0–100%").size(12).tone(MUTED),
            instrument::panel("RESOURCE HISTORY / cursore e campioni sincronizzati",column![
                canvas(chart::ResourceChart::new(&n.history,self.view_time_ms)).width(Fill).height(260),
                text("CPU pervinca · RAM ametista · GPU azzurro. Passa sul grafico per timestamp e valori; i gap non vengono collegati.").size(11).tone(MUTED),
                text("Serie in memoria ~1 s; storico SQLite in riepiloghi di 5 s. Confronto multi-host, zoom e range lunghi richiedono il prossimo motore delle serie.").size(10).tone(MUTED),
            ].spacing(10)),
            instrument::panel("RAW RECEIPT SAMPLES / ultime 10 osservazioni · timestamp di ricezione, non clock sorgente",raw),
        ].spacing(12).into()
    }
}
fn empty_fleet<'a>() -> Element<'a, Message> {
    instrument::panel(
        "NESSUN HOST",
        column![
            text("Aggiungi questo computer o un host remoto per iniziare.").size(14),
            small_button("+ Aggiungi host", Message::NewHost)
        ]
        .spacing(12),
    )
    .into()
}

fn stat<'a>(label: &str, value: &str, detail: &str) -> Element<'a, Message> {
    instrument::panel(
        label,
        column![
            text(value.to_owned()).size(22).font(STRONG),
            text(detail.to_owned()).size(11).tone(MUTED)
        ]
        .spacing(8),
    )
    .into()
}

fn rail_control<'a>(
    element: impl Into<Element<'a, Message>>,
    message: Message,
) -> widget::Button<'a, Message> {
    button(element)
        .on_press(message)
        .padding(11)
        .width(Fill)
        .style(|_, state| button::Style {
            background: Some(
                if matches!(state, button::Status::Hovered) {
                    NAV_HOVER
                } else {
                    NAV
                }
                .into(),
            ),
            text_color: NAV_TEXT,
            border: Border {
                radius: 8.into(),
                ..Default::default()
            },
            ..Default::default()
        })
}
