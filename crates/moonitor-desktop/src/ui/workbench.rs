//! W1 shell: operational questions, shared node state, no synthetic production data.
use super::*;
use iced::widget::column;
use iced::widget::pick_list;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Section {
    Overview,
    Fleet,
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
    Section::Workloads,
    Section::Energy,
    Section::Events,
    Section::Explore,
    Section::Settings,
];
impl Section {
    pub fn parse(value: &str) -> Option<Self> {
        SECTIONS.into_iter().chain([Self::Device]).find(|s| {
            s.label().eq_ignore_ascii_case(value) || (*s == Self::Device && value == "device")
        })
    }
    pub(super) fn label(self) -> &'static str {
        match self {
            Self::Overview => "Overview",
            Self::Fleet => "Fleet",
            Self::Workloads => "Workloads",
            Self::Energy => "Energy",
            Self::Events => "Events",
            Self::Explore => "Explore",
            Self::Settings => "Settings",
            Self::Device => "Dispositivo",
        }
    }
    fn icon(self) -> &'static str {
        match self {
            Self::Overview => "overview",
            Self::Fleet => "network",
            Self::Workloads => "ai",
            Self::Energy => "energy",
            Self::Events => "events",
            Self::Explore => "history",
            Self::Settings => "settings",
            Self::Device => "overview",
        }
    }
    fn subtitle(self) -> &'static str {
        match self {
            Self::Overview => "Il laboratorio, adesso.",
            Self::Fleet => "Nodi, capacità e disponibilità.",
            Self::Workloads => "Runtime e modelli osservati nel laboratorio.",
            Self::Energy => "Potenza misurata, con confini espliciti.",
            Self::Events => "Cambiamenti e qualità delle sorgenti.",
            Self::Explore => "Andamento recente delle risorse.",
            Self::Settings => "Sorgenti e preferenze di visualizzazione.",
            Self::Device => "Telemetria e risorse del nodo selezionato.",
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
        write!(f, "{}", self.label)
    }
}
impl App {
    pub(super) fn workbench_view(&self) -> Element<'_, Message> {
        let fleet = FleetViewModel::new(&self.nodes, self.selected.as_deref(), Instant::now());
        let node = self
            .nodes
            .iter()
            .find(|n| Some(n.config.id.as_str()) == fleet.selected_id.as_deref());
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
            .find(|n| Some(n.id.as_str()) == fleet.selected_id.as_deref())
            .cloned();
        let switcher = pick_list(choices, selected, |n| Message::SwitchNode(n.id))
            .placeholder("Seleziona nodo")
            .text_size(12)
            .padding([9, 12])
            .width(190)
            .style(|theme, state| {
                let mut style = pick_list::default(theme, state);
                style.background = tone(PAPER, theme).into();
                style.border.color = tone(LINE, theme);
                style.border.radius = 7.0.into();
                style
            });
        let top = row![
            column![
                text(self.section.label()).size(26).font(STRONG),
                text(self.section.subtitle()).size(12).tone(MUTED)
            ]
            .spacing(5),
            Space::new().width(Fill),
            pill("LIVE · 1 s", HEALTHY),
            switcher
        ]
        .spacing(16)
        .align_y(alignment::Vertical::Center);
        let content = match self.section {
            Section::Overview => self.overview(&fleet, node),
            Section::Fleet => self.fleet(&fleet),
            Section::Workloads => self.workloads(),
            Section::Energy => self.energy(),
            Section::Events => self.events(),
            Section::Explore => node.map(|n| self.explore(n)).unwrap_or_else(empty_fleet),
            Section::Settings => self.settings(),
            Section::Device => node.map(|n| self.device(n)).unwrap_or_else(empty_fleet),
        };
        let mut body = column![content].spacing(18);
        if let Some(error) = &self.storage_error {
            body = body.push(notice(&format!("Storico: {error}")));
        }
        if self.dropped_events > 0 {
            body = body.push(notice(&format!(
                "Acquisizione: {} aggiornamenti scartati per coda piena",
                self.dropped_events
            )));
        }
        container(row![
            self.workbench_sidebar(),
            column![
                container(top).padding([24, 26]),
                scrollable(
                    container(body)
                        .padding(iced::Padding {
                            top: 0.0,
                            right: 26.0,
                            bottom: 24.0,
                            left: 26.0
                        })
                        .width(Fill)
                )
                .height(Fill)
            ]
            .height(Fill)
            .width(Fill)
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
    fn workbench_sidebar(&self) -> Element<'_, Message> {
        let brand = row![
            brand_icon(),
            text("moonitor").size(22).font(STRONG).color(Color::WHITE)
        ]
        .spacing(9)
        .align_y(alignment::Vertical::Center);
        let mut nav = column![
            brand,
            text("COMPUTE OBSERVATORY").size(9).color(NAV_MUTED),
            Space::new().height(26)
        ]
        .spacing(14);
        for section in SECTIONS {
            let selected = self.section == section
                || (self.section == Section::Device && section == Section::Fleet);
            let color = if selected { ACCENT_SOFT } else { NAV_TEXT };
            nav = nav.push(
                button(
                    row![
                        canvas(Icon(section.icon(), color)).width(18).height(18),
                        text(section.label()).size(13).font(MEDIUM)
                    ]
                    .spacing(12)
                    .align_y(alignment::Vertical::Center),
                )
                .width(Fill)
                .padding([12, 13])
                .on_press(Message::Navigate(section))
                .style(move |_, state| button::Style {
                    background: Some(
                        if selected {
                            NAV_SELECTED
                        } else if matches!(state, button::Status::Hovered) {
                            NAV_HOVER
                        } else {
                            Color::TRANSPARENT
                        }
                        .into(),
                    ),
                    text_color: color,
                    border: Border {
                        radius: 9.into(),
                        ..Default::default()
                    },
                    ..Default::default()
                }),
            );
        }
        let monitored = self
            .nodes
            .iter()
            .filter(|n| !matches!(n.config.kind, crate::config::NodeKind::Unavailable))
            .count();
        let footer = column![
            text("RACCOLTA LOCALE").size(9).color(NAV_MUTED),
            text(format!("{monitored} nodi strumentati"))
                .size(12)
                .color(NAV_TEXT),
            text("SSH · archivio sul dispositivo")
                .size(10)
                .color(NAV_MUTED),
            text("Moonitor / Native workbench").size(9).color(NAV_MUTED)
        ]
        .spacing(9);
        container(column![nav, Space::new().height(Fill), footer].spacing(18))
            .padding([28, 20])
            .width(220)
            .height(Fill)
            .style(|_| container::Style {
                background: Some(NAV.into()),
                ..Default::default()
            })
            .into()
    }
    fn overview<'a>(
        &'a self,
        fleet: &FleetViewModel,
        node: Option<&'a NodeState>,
    ) -> Element<'a, Message> {
        let observed_runtime = self
            .nodes
            .iter()
            .filter(|n| n.live())
            .filter_map(|n| n.sample.as_ref())
            .filter(|s| s.ollama.models.value().is_some())
            .count();
        let models = self
            .nodes
            .iter()
            .filter(|n| n.live())
            .filter_map(|n| n.sample.as_ref())
            .filter_map(|s| s.ollama.models.value())
            .map(Vec::len)
            .sum::<usize>();
        let gpu_values = self
            .nodes
            .iter()
            .filter(|n| n.live())
            .filter_map(|n| n.sample.as_ref())
            .flat_map(|s| s.gpus.iter())
            .filter_map(|g| g.utilization_percent.value().copied())
            .collect::<Vec<_>>();
        let power = self.measured_power();
        let stats = row![
            stat(
                "NODI MONITORATI",
                &format!("{} / {}", fleet.online, fleet.monitored),
                &format!(
                    "{} censiti · {} non strumentati",
                    fleet.devices.len(),
                    fleet.devices.len() - fleet.monitored
                )
            ),
            stat(
                "MODELLI RESIDENTI",
                &if observed_runtime > 0 {
                    models.to_string()
                } else {
                    "N/D".into()
                },
                &format!("Ollama osservato su {observed_runtime} nodi")
            ),
            stat(
                "GPU · MASSIMO OSSERVATO",
                &pct(gpu_values.iter().copied().reduce(f64::max)),
                &format!("{} acceleratori con misura", gpu_values.len())
            ),
            stat(
                "POTENZA GPU MISURATA",
                &watts(power.0),
                &format!("{} nodi con lettura · board", power.1)
            )
        ]
        .spacing(12);
        let timeline = if let Some(n) = node {
            self.resource_timeline(n)
        } else {
            empty_fleet()
        };
        let middle = row![
            container(timeline).width(Length::FillPortion(3)),
            container(self.lab_map(fleet, 220.0)).width(Length::FillPortion(2))
        ]
        .spacing(16);
        column![stats, middle, self.model_table(true), self.source_summary()]
            .spacing(18)
            .into()
    }
    fn resource_timeline<'a>(&self, n: &'a NodeState) -> Element<'a, Message> {
        let s = n.sample.as_ref();
        let title = row![
            column![
                text("Resource timeline").size(16).font(STRONG),
                text(format!("{} · ultimi 5 minuti", n.config.label))
                    .size(11)
                    .tone(MUTED)
            ]
            .spacing(4),
            Space::new().width(Fill),
            small_button("Apri nodo →", Message::Select(n.config.id.clone()))
        ]
        .align_y(alignment::Vertical::Center);
        let metrics = row![
            metric(
                Metric::percent("CPU", s.map(|s| &s.cpu_utilization_percent), !n.live()),
                chart::SERIES[0]
            ),
            metric(
                s.map(|s| Metric::memory(s, !n.live()))
                    .unwrap_or_else(|| Metric::percent("RAM", None, false)),
                chart::SERIES[1]
            ),
            metric(
                s.map(|s| Metric::gpu(s, !n.live()))
                    .unwrap_or_else(|| Metric::percent("GPU", None, false)),
                chart::SERIES[2]
            )
        ]
        .spacing(16);
        card(
            column![
                title,
                metrics,
                canvas(chart::ResourceChart::new(&n.history, self.view_time_ms))
                    .width(Fill)
                    .height(135),
                text("Utilizzo % · passa sul grafico per ispezionare")
                    .size(10)
                    .tone(MUTED)
            ]
            .spacing(12),
        )
        .height(320)
        .into()
    }
    fn lab_map<'a>(&'a self, fleet: &FleetViewModel, height: f32) -> Element<'a, Message> {
        let scene = Scene {
            selected: fleet.selected_id.clone(),
            devices: fleet.devices.clone(),
            camera: self.camera,
            dark: self.dark,
        };
        let content: Element<'a, Message> = if fleet.devices.is_empty() {
            empty_fleet()
        } else if self.scene_visible {
            widget::stack![
                widget::shader(scene.clone()).width(Fill).height(height),
                canvas(scene).width(Fill).height(height)
            ]
            .into()
        } else {
            container(self.scene_list()).height(height).into()
        };
        card(
            column![
                row![
                    text("Lab Map").size(16).font(STRONG),
                    Space::new().width(Fill),
                    small_button(
                        if self.scene_visible { "Elenco" } else { "3D" },
                        Message::ToggleScene
                    ),
                    small_button("−", Message::Zoom(-0.1)),
                    small_button("+", Message::Zoom(0.1)),
                    small_button("↺", Message::ResetCamera)
                ]
                .align_y(alignment::Vertical::Center),
                content,
                text("Seleziona un nodo per aprire il dettaglio")
                    .size(10)
                    .tone(MUTED)
            ]
            .spacing(10),
        )
        .height(height + 100.0)
        .into()
    }
    fn fleet<'a>(&'a self, fleet: &FleetViewModel) -> Element<'a, Message> {
        let header = row![
            text(format!("{} nodi censiti", fleet.devices.len()))
                .size(17)
                .font(STRONG),
            Space::new().width(Fill),
            text(format!(
                "{} online / {} monitorati",
                fleet.online, fleet.monitored
            ))
            .size(12)
            .tone(MUTED)
        ];
        let mut rows = column![
            header,
            table_row(["DISPOSITIVO", "STATO", "CPU / RAM", "GPU"], true)
        ]
        .spacing(10);
        for n in &self.nodes {
            let cpu = n
                .sample
                .as_ref()
                .and_then(|s| s.cpu_utilization_percent.value().copied());
            let memory = n
                .sample
                .as_ref()
                .and_then(|s| moonitor_store::memory_percent(s));
            let gpu = n
                .sample
                .as_ref()
                .and_then(|s| Metric::gpu(s, !n.live()).value);
            rows = rows.push(
                button(
                    column![
                        table_row(
                            [
                                &n.config.label,
                                status(n).0,
                                &format!("{} / {}", pct(cpu), pct(memory)),
                                &pct(gpu)
                            ],
                            false
                        ),
                        text(
                            n.sample
                                .as_ref()
                                .map(|s| format!("{} · {}", s.cpu_name, s.os))
                                .unwrap_or_else(|| n.config.note.clone())
                        )
                        .size(11)
                        .tone(MUTED)
                    ]
                    .spacing(3),
                )
                .width(Fill)
                .padding(10)
                .on_press(Message::Select(n.config.id.clone()))
                .style(|theme, state| {
                    themed_button(
                        theme,
                        button::Style {
                            background: Some(
                                if matches!(state, button::Status::Hovered) {
                                    BG
                                } else {
                                    PAPER
                                }
                                .into(),
                            ),
                            text_color: INK,
                            border: Border {
                                color: LINE,
                                width: 0.7,
                                radius: 6.0.into(),
                            },
                            ..Default::default()
                        },
                    )
                }),
            );
        }
        column![card(rows), self.lab_map(fleet, 300.0)]
            .spacing(18)
            .into()
    }
    fn device<'a>(&'a self, n: &'a NodeState) -> Element<'a, Message> {
        let tabs = Row::with_children(TABS.iter().map(|tab| {
            button(text(tab.label()).size(12))
                .on_press(Message::Tab(*tab))
                .padding([8, 12])
                .style(move |theme, state| {
                    themed_button(
                        theme,
                        button::Style {
                            background: Some(
                                if self.tab == *tab {
                                    SELECTED
                                } else if matches!(state, button::Status::Hovered) {
                                    LINE
                                } else {
                                    Color::TRANSPARENT
                                }
                                .into(),
                            ),
                            text_color: if self.tab == *tab { ACCENT } else { MUTED },
                            border: Border {
                                radius: 7.0.into(),
                                ..Default::default()
                            },
                            ..Default::default()
                        },
                    )
                })
                .into()
        }))
        .spacing(6);
        let mut body = column![self.node_heading(n), tabs].spacing(20);
        if self.tab == Tab::Overview && n.sample.is_some() {
            let s = n.sample.as_ref();
            body = body.push(
                row![
                    card(metric(
                        Metric::percent("CPU", s.map(|s| &s.cpu_utilization_percent), !n.live()),
                        CPU
                    )),
                    card(metric(
                        s.map(|s| Metric::memory(s, !n.live()))
                            .unwrap_or_else(|| Metric::percent("RAM", None, false)),
                        MEMORY
                    )),
                    card(metric(
                        s.map(|s| Metric::gpu(s, !n.live()))
                            .unwrap_or_else(|| Metric::percent("GPU", None, false)),
                        GPU
                    ))
                ]
                .spacing(12),
            );
        }
        body.push(self.details(n)).into()
    }
    fn model_table(&self, compact: bool) -> Element<'_, Message> {
        let mut rows = column![
            row![
                text("Modelli osservati").size(16).font(STRONG),
                Space::new().width(Fill),
                small_button("Workloads →", Message::Navigate(Section::Workloads))
            ]
            .align_y(alignment::Vertical::Center),
            table_row(["MODELLO", "NODO", "RUNTIME", "MEMORIA MODELLO"], true)
        ]
        .spacing(8);
        let mut count = 0;
        for n in &self.nodes {
            if let Some(s) = &n.sample
                && let Some(models) = s.ollama.models.value()
            {
                for m in models {
                    if !compact || count < 3 {
                        rows = rows.push(table_row(
                            [
                                &m.name,
                                &n.config.label,
                                if n.live() {
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
        for n in &self.nodes {
            let state = n
                .sample
                .as_ref()
                .map(|s| reading_reason(&s.ollama.models))
                .unwrap_or("Nessuna osservazione");
            runtime = runtime.push(table_row(
                [&n.config.label, "Ollama", state, status(n).0],
                false,
            ));
        }
        column![self.model_table(false),card(runtime),notice("Questa vista osserva lo stato dei modelli. Tempi e risorse delle singole richieste non sono ancora esposti dalle sorgenti collegate.")].spacing(18).into()
    }
    fn measured_power(&self) -> (Option<f64>, usize) {
        let values = self
            .nodes
            .iter()
            .filter(|n| n.live())
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
        for n in &self.nodes {
            rows = rows.push(table_row(
                [
                    &n.config.label,
                    &watts(n.sample.as_ref().and_then(|s| gpu_power(s))),
                    "GPU board",
                    status(n).0,
                ],
                false,
            ));
        }
        column![row![stat("POTENZA GPU MISURATA",&watts(power.0),&format!("{} nodi con lettura corrente",power.1)),stat("INTERO LABORATORIO","N/D","Nessun totale whole-device disponibile"),stat("ENERGIA","N/D","Intervalli e copertura non disponibili")].spacing(12),card(rows),notice("La potenza GPU non rappresenta il consumo totale del dispositivo. La memoria unificata non aggiunge capacità alla RAM del nodo.")].spacing(18).into()
    }
    fn source_summary(&self) -> Element<'_, Message> {
        let mut row = Row::new().spacing(18);
        for n in &self.nodes {
            let (state, color) = status(n);
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
        card(
            column![
                text(format!("{} · storico recente", n.config.label))
                    .size(18)
                    .font(STRONG),
                text("Ultimi 5 minuti · dati ricevuti · campioni persistiti ogni 5 s")
                    .size(12)
                    .tone(MUTED),
                plot("CPU", &n.history, self.view_time_ms, 0, CPU),
                plot("RAM", &n.history, self.view_time_ms, 1, MEMORY),
                plot(
                    "GPU · massimo disponibile",
                    &n.history,
                    self.view_time_ms,
                    2,
                    GPU
                ),
                text("0–100% · le interruzioni delle misure restano visibili come lacune.")
                    .size(11)
                    .tone(MUTED)
            ]
            .spacing(20),
        )
        .into()
    }
    fn settings(&self) -> Element<'_, Message> {
        let themes = row![
            theme_choice("Moonlight", "Luce lunare · chiaro", false, self.dark),
            theme_choice("Deep Space", "Cielo notturno · scuro", true, self.dark),
        ]
        .spacing(12);
        column![
            card(column![
                text("Aspetto · sessione corrente").size(18).font(STRONG),
                themes,
                small_button(if self.scene_visible { "Disattiva 3D" } else { "Attiva 3D" }, Message::ToggleScene),
                text("Il rendering 3D si aggiorna su dati e interazioni. Nessuna animazione continua.").size(12).tone(MUTED),
            ].spacing(16)),
            card(column![
                text("Collegamento delle sorgenti").size(18).font(STRONG),
                text("I nodi e gli alias SSH sono configurati in fleet.toml. Le credenziali restano nella configurazione SSH del sistema. Le modifiche alla flotta richiedono il riavvio dell'app.").size(13).tone(MUTED),
                text("Archivio locale SQLite · riepiloghi ogni 5 s · conservazione 7 giorni. La raccolta si ferma chiudendo il desktop.").size(13).tone(MUTED),
                text("Tastiera: Cmd/Ctrl+1…7 navigazione · Alt+←/→ nodo · Tab e Shift+Tab focus.").size(12).tone(MUTED),
            ].spacing(14)),
            self.source_summary(),
        ].spacing(18).into()
    }
}
fn theme_choice(
    name: &'static str,
    description: &'static str,
    dark: bool,
    current: bool,
) -> Element<'static, Message> {
    let selected = dark == current;
    button(
        column![
            row![
                text(name).size(15).font(STRONG),
                Space::new().width(Fill),
                text(if selected { "Attivo" } else { "Seleziona" })
                    .size(11)
                    .tone(if selected { ACCENT } else { MUTED }),
            ]
            .spacing(12)
            .align_y(alignment::Vertical::Center),
            text(description).size(12).tone(MUTED),
        ]
        .spacing(8),
    )
    .padding(16)
    .width(Fill)
    .on_press(Message::Theme(dark))
    .style(move |theme, status| {
        themed_button(
            theme,
            button::Style {
                background: Some(
                    if selected {
                        SELECTED
                    } else if matches!(status, button::Status::Hovered) {
                        HOVER
                    } else {
                        PAPER
                    }
                    .into(),
                ),
                text_color: INK,
                border: Border {
                    color: if selected { ACCENT } else { LINE },
                    width: 1.0,
                    radius: 9.0.into(),
                },
                ..Default::default()
            },
        )
    })
    .into()
}
fn stat<'a>(label: &str, value: &str, detail: &str) -> Element<'a, Message> {
    card(
        column![
            text(label.to_owned()).size(9).font(MEDIUM).tone(MUTED),
            text(value.to_owned()).size(28).font(STRONG),
            text(detail.to_owned()).size(10).tone(MUTED)
        ]
        .spacing(8),
    )
    .into()
}
fn empty_fleet<'a>() -> Element<'a, Message> {
    card(column![text("Nessun dispositivo configurato").size(18).font(STRONG),text("Aggiungi un nodo local o ssh in fleet.toml per iniziare la raccolta. Puoi censire dispositivi senza agente come unavailable.").size(13).tone(MUTED)].spacing(12)).into()
}
