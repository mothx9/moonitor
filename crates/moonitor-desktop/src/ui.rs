use crate::{
    config::{Config, NodeKind},
    engine::{Engine, NodeState},
    scene::{Camera, Scene},
};
use iced::{
    Border, Color, Element, Font,
    Length::{self, Fill},
    Subscription, Task, Theme, alignment,
    widget::{
        self, Column, Row, Space, button, canvas, column, container, progress_bar, row, scrollable,
        text, text_input,
    },
    window,
};
use moonitor_core::*;
use moonitor_store::{HistoryPoint, memory_percent};
use std::{
    path::PathBuf,
    time::{Duration, Instant},
};

const BG: Color = Color::from_rgb(0.965, 0.970, 0.955);
const PAPER: Color = Color::WHITE;
const INK: Color = Color::from_rgb(0.10, 0.15, 0.14);
const MUTED: Color = Color::from_rgb(0.45, 0.50, 0.47);
const LINE: Color = Color::from_rgb(0.87, 0.89, 0.86);
const GREEN: Color = Color::from_rgb(0.23, 0.49, 0.36);
const LIME: Color = Color::from_rgb(0.71, 0.87, 0.60);
const AMBER: Color = Color::from_rgb(0.65, 0.40, 0.18);
const NAV: Color = Color::from_rgb(0.085, 0.12, 0.11);
const DEVICE_IDS: [&str; 5] = ["macbook", "exon", "spark", "zima", "iphone"];
const FONT: Font = Font::with_name("Inter");
const MEDIUM: Font = Font {
    weight: iced::font::Weight::Medium,
    ..FONT
};
const STRONG: Font = Font {
    weight: iced::font::Weight::Semibold,
    ..FONT
};
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Tab {
    Overview,
    Processes,
    Ai,
    Disks,
    Network,
    History,
}
impl Tab {
    pub fn parse(s: &str) -> Option<Self> {
        Some(match s {
            "overview" => Self::Overview,
            "processes" => Self::Processes,
            "ai" => Self::Ai,
            "disks" => Self::Disks,
            "network" => Self::Network,
            "history" => Self::History,
            _ => return None,
        })
    }
    fn label(self) -> &'static str {
        match self {
            Self::Overview => "Panoramica",
            Self::Processes => "Processi",
            Self::Ai => "AI locale",
            Self::Disks => "Archiviazione",
            Self::Network => "Rete",
            Self::History => "Storico",
        }
    }
    fn icon(self) -> &'static str {
        match self {
            Self::Overview => "overview",
            Self::Processes => "process",
            Self::Ai => "ai",
            Self::Disks => "disk",
            Self::Network => "network",
            Self::History => "history",
        }
    }
}
const TABS: [Tab; 6] = [
    Tab::Overview,
    Tab::Ai,
    Tab::Processes,
    Tab::Disks,
    Tab::Network,
    Tab::History,
];
#[derive(Debug, Clone)]
pub enum Message {
    Tick,
    Select(usize),
    SceneSelect(usize),
    Tab(Tab),
    Filter(String),
    Sort(bool),
    Orbit(f32),
    Zoom(f32),
    ResetCamera,
    Window(window::Id, window::Event),
    Screenshot(window::Screenshot, PathBuf),
}
pub struct App {
    _engine: Engine,
    nodes: Vec<NodeState>,
    storage_error: Option<String>,
    selected: usize,
    tab: Tab,
    filter: String,
    sort_memory: bool,
    camera: Camera,
    started: Instant,
    screenshot: Option<PathBuf>,
    exit_after: Option<f32>,
    window: Option<window::Id>,
}
pub fn run(
    config: Config,
    data_dir: PathBuf,
    selected: usize,
    tab: Tab,
    screenshot: Option<PathBuf>,
    exit_after: Option<f32>,
) -> anyhow::Result<()> {
    iced::application(
        move || {
            let engine = Engine::start(config.clone(), data_dir.join("history.sqlite"));
            let nodes = engine.state.lock().unwrap().nodes.clone();
            App {
                _engine: engine,
                nodes,
                storage_error: None,
                selected,
                tab,
                filter: String::new(),
                sort_memory: false,
                camera: Camera::default(),
                started: Instant::now(),
                screenshot: screenshot.clone(),
                exit_after,
                window: None,
            }
        },
        App::update,
        App::view,
    )
    .title("moonitor")
    .window(window::Settings {
        size: iced::Size::new(1380.0, 940.0),
        min_size: Some(iced::Size::new(1160.0, 760.0)),
        position: window::Position::Centered,
        ..Default::default()
    })
    .default_font(FONT)
    .font(include_bytes!("../assets/Inter-Regular.ttf").as_slice())
    .font(include_bytes!("../assets/Inter-Medium.ttf").as_slice())
    .font(include_bytes!("../assets/Inter-SemiBold.ttf").as_slice())
    .theme(|_: &App| {
        Theme::custom(
            "moonitor",
            iced::theme::Palette {
                background: BG,
                text: INK,
                primary: GREEN,
                success: GREEN,
                warning: AMBER,
                danger: AMBER,
            },
        )
    })
    .subscription(|_: &App| {
        Subscription::batch([
            iced::time::every(Duration::from_secs(1)).map(|_| Message::Tick),
            window::events().map(|(id, e)| Message::Window(id, e)),
        ])
    })
    .run()
    .map_err(|e| anyhow::anyhow!(e.to_string()))
}
impl App {
    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Tick => {
                let shared = self._engine.state.lock().unwrap();
                self.nodes = shared.nodes.clone();
                self.storage_error = shared.storage_error.clone();
                drop(shared);
                if self
                    .exit_after
                    .is_some_and(|s| self.started.elapsed().as_secs_f32() >= s)
                {
                    return iced::exit();
                }
                if self.started.elapsed() >= Duration::from_secs(8)
                    && self.window.is_some()
                    && let Some(path) = self.screenshot.take()
                {
                    return window::screenshot(self.window.unwrap())
                        .map(move |s| Message::Screenshot(s, path.clone()));
                }
            }
            Message::Window(id, window::Event::Opened { .. }) => self.window = Some(id),
            Message::Window(_, _) => (),
            Message::Screenshot(s, path) => {
                if let Err(e) = image::save_buffer(
                    &path,
                    &s.rgba,
                    s.size.width,
                    s.size.height,
                    image::ColorType::Rgba8,
                ) {
                    eprintln!("Screenshot failed: {e}");
                }
            }
            Message::Select(i) => {
                if i < self.nodes.len() {
                    self.selected = i;
                }
            }
            Message::SceneSelect(i) => {
                if let Some(index) = self.nodes.iter().position(|n| n.config.id == DEVICE_IDS[i]) {
                    self.selected = index;
                }
            }
            Message::Tab(tab) => self.tab = tab,
            Message::Filter(s) => self.filter = s,
            Message::Sort(memory) => self.sort_memory = memory,
            Message::Orbit(dx) => self.camera.yaw += dx,
            Message::Zoom(delta) => self.camera.zoom = (self.camera.zoom + delta).clamp(0.75, 1.6),
            Message::ResetCamera => self.camera = Camera::default(),
        }
        Task::none()
    }
    fn view(&self) -> Element<'_, Message> {
        let Some(node) = self.nodes.get(self.selected) else {
            return text("Nessun dispositivo configurato").into();
        };
        let active = self.nodes.iter().filter(|n| n.live()).count();
        let header = row![
            column![
                eyebrow("WORKSPACE / HOMELAB"),
                text(self.tab.label()).size(34).font(STRONG)
            ]
            .spacing(8),
            Space::new().width(Fill),
            pill(&format!("{active} / {} online", self.nodes.len()), GREEN),
            text("LOCALE  /  VPN").size(10).color(MUTED)
        ]
        .spacing(20)
        .align_y(alignment::Vertical::Center);
        let mut body = column![header].spacing(22).width(Fill);
        if let Some(error) = &self.storage_error {
            body = body.push(notice(&format!("Storico non disponibile: {error}")));
        }
        if self.tab == Tab::Overview {
            let live = std::array::from_fn(|i| {
                self.nodes
                    .iter()
                    .find(|n| n.config.id == DEVICE_IDS[i])
                    .is_some_and(NodeState::live)
            });
            let selected = DEVICE_IDS
                .iter()
                .position(|id| *id == node.config.id)
                .unwrap_or(0);
            let scene = Scene {
                selected,
                live,
                camera: self.camera,
            };
            let controls = row![
                text("IL TUO HOMELAB").size(10).font(MEDIUM).color(MUTED),
                Space::new().width(Fill),
                small_button("−", Message::Zoom(-0.1)),
                small_button("+", Message::Zoom(0.1)),
                small_button("Ripristina", Message::ResetCamera)
            ]
            .spacing(6)
            .align_y(alignment::Vertical::Center);
            let scene_panel = container(
                column![
                    container(controls).padding([14, 18]),
                    widget::shader(scene).width(Fill).height(398),
                    container(
                        row![
                            text(format!("{:02}", self.nodes.len()))
                                .font(STRONG)
                                .size(12)
                                .color(GREEN),
                            text("DISPOSITIVI").size(10).color(MUTED),
                            Space::new().width(Fill),
                            text("Trascina per ruotare · clicca per selezionare")
                                .size(10)
                                .color(MUTED)
                        ]
                        .spacing(8)
                    )
                    .padding([10, 18])
                ]
                .spacing(0),
            )
            .style(|_| container::Style {
                background: Some(Color::from_rgb(0.95, 0.956, 0.938).into()),
                border: Border {
                    radius: 18.into(),
                    color: LINE,
                    width: 1.0,
                },
                ..Default::default()
            })
            .width(Fill);
            body = body.push(row![scene_panel, self.inspector(node)].spacing(16));
        } else {
            body = body.push(self.device_strip()).push(self.node_heading(node));
        }
        if self.tab == Tab::Overview {
            body = body.push(self.device_strip());
        }
        body = body.push(self.details(node));
        let main = scrollable(container(body).padding([28, 28]).width(Fill)).height(Fill);
        container(row![self.sidebar(), main].spacing(0))
            .style(|_| container::Style {
                background: Some(BG.into()),
                text_color: Some(INK),
                ..Default::default()
            })
            .width(Fill)
            .height(Fill)
            .into()
    }
    fn sidebar(&self) -> Element<'_, Message> {
        let brand = row![
            canvas(Icon("moon", LIME)).width(28).height(28),
            text("moonitor").size(23).font(STRONG).color(Color::WHITE)
        ]
        .spacing(8)
        .align_y(alignment::Vertical::Center);
        let mut nav = column![
            container(brand).padding([8, 0]),
            text("PERSONAL OBSERVATORY")
                .size(8)
                .color(Color::from_rgb(0.49, 0.60, 0.53)),
            Space::new().height(30),
            text("MONITORAGGIO")
                .size(9)
                .color(Color::from_rgb(0.47, 0.57, 0.51))
        ]
        .spacing(10);
        for tab in TABS {
            let chosen = self.tab == tab;
            let color = if chosen {
                LIME
            } else {
                Color::from_rgb(0.70, 0.76, 0.71)
            };
            nav = nav.push(
                button(
                    row![
                        canvas(Icon(tab.icon(), color)).width(18).height(18),
                        text(tab.label()).size(12).font(MEDIUM)
                    ]
                    .spacing(12)
                    .align_y(alignment::Vertical::Center),
                )
                .on_press(Message::Tab(tab))
                .width(Fill)
                .padding([13, 12])
                .style(move |_, state| button::Style {
                    background: Some(
                        if chosen {
                            Color::from_rgb(0.16, 0.23, 0.18)
                        } else if matches!(state, button::Status::Hovered) {
                            Color::from_rgb(0.12, 0.18, 0.14)
                        } else {
                            NAV
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
        nav = nav.push(Space::new().height(32)).push(
            text("I TUOI DISPOSITIVI")
                .size(9)
                .color(Color::from_rgb(0.47, 0.57, 0.51)),
        );
        for (i, node) in self.nodes.iter().enumerate() {
            let (_, color) = status(node);
            nav = nav.push(
                button(
                    row![
                        text("•").color(color).size(15),
                        text(&node.config.label)
                            .size(11)
                            .color(if self.selected == i {
                                Color::WHITE
                            } else {
                                Color::from_rgb(0.58, 0.67, 0.61)
                            })
                    ]
                    .spacing(10),
                )
                .on_press(Message::Select(i))
                .style(button::text)
                .padding([7, 9])
                .width(Fill),
            );
        }
        let footer = column![
            row![
                text("•").color(LIME),
                text("Raccolta attiva")
                    .size(11)
                    .color(Color::from_rgb(0.73, 0.81, 0.74))
            ]
            .spacing(8),
            text("1 s · SQLite locale")
                .size(10)
                .color(Color::from_rgb(0.44, 0.55, 0.48)),
            text("moonitor  /  0.1.0")
                .size(9)
                .color(Color::from_rgb(0.37, 0.47, 0.41))
        ]
        .spacing(10);
        container(
            column![
                scrollable(nav).height(Fill),
                container(footer).padding([14, 0])
            ]
            .spacing(0),
        )
        .padding([24, 18])
        .width(202)
        .height(Fill)
        .style(|_| container::Style {
            background: Some(NAV.into()),
            ..Default::default()
        })
        .into()
    }
    fn device_strip(&self) -> Element<'_, Message> {
        let mut devices = Row::new().spacing(10);
        for (i, n) in self.nodes.iter().enumerate() {
            let chosen = self.selected == i;
            let (state, color) = status(n);
            let cpu = n
                .sample
                .as_ref()
                .and_then(|s| s.cpu_utilization_percent.value().copied());
            let content = column![
                row![
                    text(format!("{:02}", i + 1)).size(10).color(MUTED),
                    Space::new().width(Fill),
                    text("•").color(color)
                ]
                .align_y(alignment::Vertical::Center),
                text(&n.config.label).font(STRONG).size(13),
                row![
                    text(if n.live() { pct(cpu) } else { state.to_owned() })
                        .size(11)
                        .height(18)
                        .color(if n.live() { INK } else { color }),
                    Space::new().width(Fill),
                    text(if n.live() { "CPU" } else { "" }).size(9).color(MUTED)
                ]
            ]
            .spacing(10);
            devices = devices.push(
                button(content)
                    .on_press(Message::Select(i))
                    .padding(14)
                    .width(Fill)
                    .style(move |_, s| button::Style {
                        background: Some(
                            if chosen {
                                Color::from_rgb(0.91, 0.95, 0.88)
                            } else if matches!(s, button::Status::Hovered) {
                                Color::from_rgb(0.94, 0.96, 0.92)
                            } else {
                                PAPER
                            }
                            .into(),
                        ),
                        text_color: INK,
                        border: Border {
                            color: if chosen { GREEN } else { LINE },
                            width: 1.0,
                            radius: 12.into(),
                        },
                        ..Default::default()
                    }),
            );
        }
        devices.into()
    }
    fn inspector<'a>(&self, n: &'a NodeState) -> Element<'a, Message> {
        let (state, color) = status(n);
        let mut content = column![
            row![
                eyebrow("DISPOSITIVO SELEZIONATO"),
                Space::new().width(Fill),
                text("•").color(color)
            ]
            .spacing(8),
            text(&n.config.label).size(25).font(STRONG),
            text(state).size(10).font(MEDIUM).color(color)
        ]
        .spacing(10);
        if let Some(s) = &n.sample {
            content = content
                .push(text(short_cpu(&s.cpu_name)).size(11).color(MUTED))
                .push(Space::new().height(8));
            let total = s.memory.total_bytes.value().copied();
            let used = total
                .zip(s.memory.available_bytes.value().copied())
                .map(|(t, a)| t.saturating_sub(a));
            content = content.push(metric(
                "CPU",
                s.cpu_utilization_percent.value().copied(),
                &format!("{} core logici", s.cpu_cores_percent.len()),
                GREEN,
            ));
            content = content.push(metric(
                "Memoria",
                memory_percent(s),
                &format!("{} / {}", bytes(used), bytes(total)),
                Color::from_rgb(0.49, 0.58, 0.41),
            ));
            content = content.push(metric(
                "GPU",
                gpu_percent(s),
                s.gpus
                    .first()
                    .map(|g| g.name.as_str())
                    .unwrap_or("Sensore non disponibile"),
                Color::from_rgb(0.51, 0.58, 0.64),
            ));
            content = content.push(
                row![
                    text("Potenza GPU").size(11).color(MUTED),
                    Space::new().width(Fill),
                    text(watts(gpu_power(s))).size(15).font(MEDIUM)
                ]
                .align_y(alignment::Vertical::Center),
            );
            content = content.push(text("Intero dispositivo: N/D").size(9).color(MUTED));
        } else {
            content = content
                .push(Space::new().height(18))
                .push(text(&n.config.note).size(13).color(MUTED))
                .push(
                    text(
                        n.error
                            .as_deref()
                            .unwrap_or("In attesa dei primi campioni."),
                    )
                    .size(11)
                    .color(MUTED),
                );
        }
        if n.sample.is_some() && !n.live() {
            content = content.push(
                text("Ultimo campione conservato · dati obsoleti")
                    .size(10)
                    .color(AMBER),
            );
        }
        container(content)
            .padding(22)
            .width(286)
            .height(470)
            .style(panel)
            .into()
    }
    fn node_heading<'a>(&self, n: &'a NodeState) -> Element<'a, Message> {
        let (state, color) = status(n);
        let mut c = column![
            row![
                text(&n.config.label).size(24).font(STRONG),
                Space::new().width(Fill),
                pill(state, color)
            ]
            .align_y(alignment::Vertical::Center)
        ]
        .spacing(10);
        if let Some(s) = &n.sample {
            c = c.push(
                text(format!("{}  /  {}  /  {}", s.os, s.arch, s.cpu_name))
                    .size(11)
                    .color(MUTED),
            );
        }
        if !n.live() && n.sample.is_some() {
            c = c.push(notice(&format!(
                "Campione ricevuto {} s fa. {}",
                n.last_seen.map(|t| t.elapsed().as_secs()).unwrap_or(0),
                n.error.as_deref().unwrap_or("Telemetria interrotta")
            )));
        }
        c.into()
    }
    fn details<'a>(&'a self, n: &'a NodeState) -> Element<'a, Message> {
        let Some(s) = &n.sample else {
            return card(
                column![
                    text("Telemetria non disponibile").size(18).font(STRONG),
                    text(&n.config.note).color(MUTED).size(12)
                ]
                .spacing(12),
            )
            .into();
        };
        match self.tab {
            Tab::Overview => {
                let mut content = column![
                    row![
                        text(format!("{} · attività", n.config.label))
                            .size(16)
                            .font(STRONG),
                        Space::new().width(Fill),
                        text("ULTIMI 5 MINUTI").size(9).color(MUTED)
                    ]
                    .align_y(alignment::Vertical::Center),
                    row![
                        plot("CPU", &n.history, 0, GREEN),
                        plot("Memoria", &n.history, 1, Color::from_rgb(0.49, 0.58, 0.41)),
                        plot("GPU", &n.history, 2, Color::from_rgb(0.51, 0.58, 0.64))
                    ]
                    .spacing(24)
                ]
                .spacing(20);
                let models = s.ollama.models.value();
                let ai = s.processes.iter().filter(|p| p.is_ai).count();
                let runtime = match models {
                    Some(m) if m.is_empty() => "Ollama pronto · nessun modello caricato".to_owned(),
                    Some(m) => format!("{} modelli caricati in Ollama", m.len()),
                    None => "Ollama non raggiungibile".into(),
                };
                content = content.push(widget::rule::horizontal(1)).push(
                    row![
                        canvas(Icon("ai", GREEN)).width(20).height(20),
                        column![
                            text("AI locale").size(12).font(STRONG),
                            text(runtime).size(11).color(MUTED)
                        ]
                        .spacing(5),
                        Space::new().width(Fill),
                        text(format!("{ai} processi candidati"))
                            .size(11)
                            .color(MUTED),
                        small_button("Apri AI locale →", Message::Tab(Tab::Ai))
                    ]
                    .spacing(14)
                    .align_y(alignment::Vertical::Center),
                );
                card(content).into()
            }
            Tab::Processes => self.processes(s, false),
            Tab::Ai => {
                let mut c = column![row![
                    text("Acceleratori").size(19).font(STRONG),
                    Space::new().width(Fill),
                    text("MISURE DEL RUNTIME E DEL SISTEMA")
                        .size(9)
                        .color(MUTED)
                ]]
                .spacing(18);
                for g in &s.gpus {
                    let mut gpu = column![
                        text(&g.name).size(17).font(STRONG),
                        row![
                            mini_stat("CARICO", &pct(g.utilization_percent.value().copied())),
                            mini_stat(
                                "MEMORIA GPU",
                                &format!(
                                    "{} / {}",
                                    bytes(g.memory_used_bytes.value().copied()),
                                    bytes(g.memory_total_bytes.value().copied())
                                )
                            ),
                            mini_stat(
                                "TEMPERATURA",
                                &g.temperature_celsius
                                    .value()
                                    .map(|v| format!("{v:.0} °C"))
                                    .unwrap_or("N/D".into())
                            ),
                            mini_stat("POTENZA SCHEDA", &watts(gpu_power(s)))
                        ]
                        .spacing(24)
                    ]
                    .spacing(18);
                    if g.memory_kind == GpuMemoryKind::Unified {
                        gpu=gpu.push(text("Memoria unificata: le allocazioni GPU condividono la RAM del nodo.").size(11).color(MUTED));
                    }
                    gpu = gpu.push(
                        text(format!(
                            "Clock: {}  ·  Limitazioni: {}",
                            g.clock_mhz
                                .value()
                                .map(|v| format!("{v} MHz"))
                                .unwrap_or("N/D".into()),
                            g.throttling.value().map(String::as_str).unwrap_or("N/D")
                        ))
                        .size(11)
                        .color(MUTED),
                    );
                    c = c.push(card(gpu));
                }
                c = c.push(text("Modelli caricati · Ollama").size(17).font(STRONG));
                match s.ollama.models.value() {
                    Some(models) if models.is_empty() => {
                        c = c.push(
                            text("Runtime disponibile. Nessun modello caricato.")
                                .size(13)
                                .color(GREEN),
                        )
                    }
                    Some(models) => {
                        for m in models {
                            c = c.push(card(
                                column![
                                    text(&m.name).size(16).font(STRONG),
                                    text(format!(
                                        "{} · contesto {} · {}",
                                        bytes(Some(m.size_bytes)),
                                        m.context_length
                                            .map(|v| v.to_string())
                                            .unwrap_or("N/D".into()),
                                        m.quantization.as_deref().unwrap_or("N/D")
                                    ))
                                    .size(12)
                                    .color(MUTED)
                                ]
                                .spacing(10),
                            ));
                        }
                    }
                    None => {
                        c = c.push(
                            text("Ollama non raggiungibile su questo nodo.")
                                .size(12)
                                .color(MUTED),
                        )
                    }
                }
                c=c.push(text("Token/s, latenza e code: N/D senza telemetria delle richieste dal runtime.").size(11).color(MUTED)).push(text("Processi AI riconosciuti dal nome").size(17).font(STRONG)).push(self.processes(s,true));
                c.into()
            }
            Tab::Disks => {
                let mut c = Column::new().spacing(14);
                for d in &s.disks {
                    let used = (d.total_bytes > 0).then(|| {
                        (d.total_bytes.saturating_sub(d.available_bytes)) as f64
                            / d.total_bytes as f64
                            * 100.0
                    });
                    c = c.push(card(
                        column![
                            text(&d.mount).size(17).font(STRONG),
                            text(&d.name).size(11).color(MUTED),
                            metric(
                                "Spazio utilizzato",
                                used,
                                &format!(
                                    "{} disponibili / {} totali",
                                    bytes(Some(d.available_bytes)),
                                    bytes(Some(d.total_bytes))
                                ),
                                GREEN
                            )
                        ]
                        .spacing(12),
                    ));
                }
                c.push(text("Capacità aggiornata ogni 5 s. I volumi APFS possono condividere lo stesso spazio.").size(11).color(MUTED)).into()
            }
            Tab::Network => {
                let mut c = column![table_row(
                    ["INTERFACCIA", "RICEZIONE", "INVIO", "ERRORI"],
                    true
                )]
                .spacing(12);
                for net in &s.networks {
                    c = c.push(table_row(
                        [
                            &net.name,
                            &format!("{}/s", bytes(Some(net.received_bytes_per_second as u64))),
                            &format!("{}/s", bytes(Some(net.transmitted_bytes_per_second as u64))),
                            &net.errors.to_string(),
                        ],
                        false,
                    ));
                }
                c=c.push(text("Interfacce fisiche, VPN e virtuali possono riportare lo stesso traffico: non vengono sommate.").size(11).color(MUTED));
                card(c).into()
            }
            Tab::History => {
                let plots = card(
                    column![
                        text("Utilizzo · ultimi 5 minuti").size(18).font(STRONG),
                        plot("CPU", &n.history, 0, GREEN),
                        plot("Memoria", &n.history, 1, Color::from_rgb(0.49, 0.58, 0.41)),
                        plot("GPU", &n.history, 2, Color::from_rgb(0.51, 0.58, 0.64)),
                        text("SQLite locale · riepilogo ogni 5 s · conservazione 7 giorni")
                            .size(11)
                            .color(MUTED)
                    ]
                    .spacing(20),
                );
                let mut health = column![
                    text("Sistema").size(18).font(STRONG),
                    text(format!(
                        "Swap {} · uptime {:.1} h · raccolta {:.2} ms",
                        bytes(s.memory.swap_used_bytes.value().copied()),
                        s.uptime_seconds as f64 / 3600.0,
                        s.collector_duration_ms
                    ))
                    .size(12)
                    .color(MUTED)
                ]
                .spacing(14);
                for p in &s.pressure {
                    health = health.push(
                        text(format!(
                            "Pressione {} (stall avg10): {:.2}%",
                            p.resource, p.some_avg10_percent
                        ))
                        .size(12),
                    );
                }
                for t in s.temperatures.iter().take(12) {
                    health =
                        health.push(text(format!("{}: {:.1} °C", t.label, t.celsius)).size(12));
                }
                let mut cores = Row::new().spacing(8);
                for (i, v) in s.cpu_cores_percent.iter().take(32).enumerate() {
                    cores = cores.push(
                        column![
                            text(i.to_string()).size(8).color(MUTED),
                            text(format!("{v:.0}")).size(10).font(MEDIUM)
                        ]
                        .spacing(6),
                    );
                }
                health = health
                    .push(eyebrow("CARICO PER CORE · %"))
                    .push(cores.wrap());
                if let Some(b) = s.battery_percent.value() {
                    health = health.push(text(format!("Batteria {b:.0}%")).size(12));
                }
                column![plots, card(health)].spacing(16).into()
            }
        }
    }
    fn processes<'a>(&'a self, s: &'a HostSnapshot, ai_only: bool) -> Element<'a, Message> {
        let controls = row![
            text_input("Cerca processo…", &self.filter)
                .on_input(Message::Filter)
                .size(12)
                .width(260),
            Space::new().width(Fill),
            small_button(
                if self.sort_memory { "CPU" } else { "CPU ↓" },
                Message::Sort(false)
            ),
            small_button(
                if self.sort_memory { "RAM ↓" } else { "RAM" },
                Message::Sort(true)
            )
        ]
        .spacing(8)
        .align_y(alignment::Vertical::Center);
        let mut c=column![controls,text(format!("{} processi sul nodo · sottoinsieme più attivo e candidati AI · 100% CPU = un core",s.process_count)).size(10).color(MUTED),table_row(["PROCESSO","PID","CPU","MEMORIA"],true)].spacing(14);
        let filter = self.filter.to_lowercase();
        let mut ps: Vec<_> = s
            .processes
            .iter()
            .filter(|p| (!ai_only || p.is_ai) && p.name.to_lowercase().contains(&filter))
            .collect();
        if self.sort_memory {
            ps.sort_by_key(|p| std::cmp::Reverse(p.memory_bytes));
        } else {
            ps.sort_by(|a, b| b.cpu_percent.total_cmp(&a.cpu_percent));
        }
        if ps.is_empty() {
            c = c.push(
                text("Nessun processo corrispondente nel campione.")
                    .size(12)
                    .color(MUTED),
            );
        }
        for p in ps {
            c = c.push(table_row(
                [
                    &p.name,
                    &p.pid.to_string(),
                    &format!("{:.1}%", p.cpu_percent),
                    &bytes(Some(p.memory_bytes)),
                ],
                false,
            ));
        }
        card(c).into()
    }
}
fn status(n: &NodeState) -> (&'static str, Color) {
    if matches!(n.config.kind, NodeKind::Unavailable) {
        ("Non collegato", MUTED)
    } else if n.live() {
        ("Online", GREEN)
    } else if n.sample.is_some() {
        ("Obsoleto", AMBER)
    } else if n.error.is_some() {
        ("Non raggiungibile", AMBER)
    } else {
        ("Connessione", MUTED)
    }
}
fn panel(_: &Theme) -> container::Style {
    container::Style {
        background: Some(PAPER.into()),
        border: Border {
            color: LINE,
            width: 1.0,
            radius: 16.into(),
        },
        ..Default::default()
    }
}
fn card<'a>(content: impl Into<Element<'a, Message>>) -> widget::Container<'a, Message> {
    container(content).padding(22).width(Fill).style(panel)
}
fn eyebrow(s: &str) -> widget::Text<'_> {
    text(s).size(9).font(MEDIUM).color(MUTED)
}
fn pill(s: &str, c: Color) -> Element<'static, Message> {
    container(text(s.to_owned()).size(10).font(MEDIUM).color(c))
        .padding([7, 10])
        .style(move |_| container::Style {
            background: Some(Color { a: 0.08, ..c }.into()),
            border: Border {
                radius: 20.into(),
                ..Default::default()
            },
            ..Default::default()
        })
        .into()
}
fn small_button(s: &str, message: Message) -> Element<'static, Message> {
    button(text(s.to_owned()).size(11).font(MEDIUM))
        .on_press(message)
        .padding([7, 10])
        .style(|_, state| button::Style {
            text_color: INK,
            background: Some(
                if matches!(state, button::Status::Hovered) {
                    LINE
                } else {
                    Color::TRANSPARENT
                }
                .into(),
            ),
            border: Border {
                radius: 7.into(),
                ..Default::default()
            },
            ..Default::default()
        })
        .into()
}
fn notice(s: &str) -> Element<'static, Message> {
    container(text(s.to_owned()).size(11).color(AMBER))
        .padding(12)
        .width(Fill)
        .style(|_| container::Style {
            background: Some(Color::from_rgb(0.98, 0.94, 0.86).into()),
            border: Border {
                radius: 8.into(),
                ..Default::default()
            },
            ..Default::default()
        })
        .into()
}
fn metric<'a>(label: &str, value: Option<f64>, foot: &str, c: Color) -> Element<'a, Message> {
    column![
        row![
            text(label.to_owned()).size(12).font(MEDIUM),
            Space::new().width(Fill),
            text(pct(value))
                .size(19)
                .font(STRONG)
                .width(70)
                .height(26)
                .align_x(alignment::Horizontal::Right)
        ]
        .align_y(alignment::Vertical::Center),
        progress_bar(0.0..=100.0, value.unwrap_or(0.0).clamp(0.0, 100.0) as f32)
            .girth(4)
            .style(move |_| progress_bar::Style {
                background: LINE.into(),
                bar: c.into(),
                border: Border {
                    radius: 2.into(),
                    ..Default::default()
                }
            }),
        text(foot.to_owned()).size(9).color(MUTED)
    ]
    .spacing(9)
    .into()
}
fn mini_stat<'a>(label: &str, value: &str) -> Element<'a, Message> {
    column![
        text(label.to_owned()).size(9).color(MUTED),
        text(value.to_owned()).size(19).font(MEDIUM)
    ]
    .spacing(10)
    .width(Fill)
    .into()
}
fn table_row(values: [&str; 4], heading: bool) -> Element<'static, Message> {
    let mut r = Row::new().spacing(20);
    for (i, v) in values.iter().enumerate() {
        r = r.push(
            text(v.to_string())
                .size(if heading { 9 } else { 12 })
                .color(if heading { MUTED } else { INK })
                .width(if i == 0 { Length::FillPortion(3) } else { Fill }),
        );
    }
    container(r).padding([8, 0]).width(Fill).into()
}
fn pct(v: Option<f64>) -> String {
    v.map(|v| format!("{v:.0}%")).unwrap_or("N/D".into())
}
fn watts(v: Option<f64>) -> String {
    v.map(|v| format!("{v:.1} W")).unwrap_or("N/D".into())
}
fn bytes(v: Option<u64>) -> String {
    let Some(v) = v else { return "N/D".into() };
    for (scale, unit) in [
        (1u64 << 40, "TiB"),
        (1 << 30, "GiB"),
        (1 << 20, "MiB"),
        (1 << 10, "KiB"),
    ] {
        if v >= scale {
            return format!("{:.1} {unit}", v as f64 / scale as f64);
        }
    }
    format!("{v} B")
}
fn gpu_percent(s: &HostSnapshot) -> Option<f64> {
    s.gpus
        .iter()
        .filter_map(|g| g.utilization_percent.value().copied())
        .reduce(f64::max)
}
fn gpu_power(s: &HostSnapshot) -> Option<f64> {
    s.power
        .iter()
        .find(|p| p.scope == PowerScope::GpuBoard)
        .and_then(|p| p.watts.value().copied())
}
fn short_cpu(s: &str) -> String {
    s.replace("AMD Ryzen 9 ", "Ryzen 9 ")
        .replace(" 12-Core Processor", "")
        .replace("Intel(R) Celeron(R) CPU ", "Celeron ")
        .replace(" @ 1.10GHz", "")
}
struct Trend {
    points: Vec<(f32, Option<f64>)>,
    color: Color,
}
fn plot<'a>(
    label: &str,
    points: &std::collections::VecDeque<HistoryPoint>,
    field: u8,
    color: Color,
) -> Element<'a, Message> {
    let since = now_ms().saturating_sub(300_000);
    let data: Vec<_> = points
        .iter()
        .filter(|p| p.received_at_ms >= since)
        .map(|p| {
            (
                ((p.received_at_ms - since) as f32 / 300_000.0),
                match field {
                    0 => p.cpu_percent,
                    1 => p.memory_percent,
                    _ => p.gpu_percent,
                },
            )
        })
        .collect();
    let last = data.last().and_then(|p| p.1);
    column![
        row![
            text(label.to_owned()).size(11).color(MUTED),
            Space::new().width(Fill),
            text(pct(last))
                .size(15)
                .font(MEDIUM)
                .width(60)
                .height(22)
                .align_x(alignment::Horizontal::Right)
        ],
        canvas(Trend {
            points: data,
            color
        })
        .width(Fill)
        .height(65)
    ]
    .spacing(10)
    .width(Fill)
    .into()
}
impl canvas::Program<Message> for Trend {
    type State = ();
    fn draw(
        &self,
        _: &(),
        renderer: &iced::Renderer,
        _: &Theme,
        bounds: iced::Rectangle,
        _: iced::mouse::Cursor,
    ) -> Vec<canvas::Geometry> {
        let mut frame = canvas::Frame::new(renderer, bounds.size());
        for y in [0.0, 0.5, 1.0] {
            frame.stroke(
                &canvas::Path::line(
                    iced::Point::new(0.0, y * bounds.height),
                    iced::Point::new(bounds.width, y * bounds.height),
                ),
                canvas::Stroke::default().with_color(LINE).with_width(0.5),
            );
        }
        let mut previous: Option<iced::Point> = None;
        for (x, v) in &self.points {
            if let Some(v) = v {
                let p = iced::Point::new(
                    *x * bounds.width,
                    (1.0 - (*v as f32 / 100.0).clamp(0.0, 1.0)) * bounds.height,
                );
                if let Some(prev) = previous
                    && p.x - prev.x < bounds.width * 0.025
                {
                    frame.stroke(
                        &canvas::Path::line(prev, p),
                        canvas::Stroke::default()
                            .with_color(self.color)
                            .with_width(1.8),
                    );
                }
                previous = Some(p);
            } else {
                previous = None;
            }
        }
        vec![frame.into_geometry()]
    }
}
struct Icon(&'static str, Color);
impl canvas::Program<Message> for Icon {
    type State = ();
    fn draw(
        &self,
        _: &(),
        renderer: &iced::Renderer,
        _: &Theme,
        bounds: iced::Rectangle,
        _: iced::mouse::Cursor,
    ) -> Vec<canvas::Geometry> {
        let mut f = canvas::Frame::new(renderer, bounds.size());
        let s = bounds.width / 24.0;
        let stroke = canvas::Stroke::default()
            .with_width(1.5 * s)
            .with_color(self.1);
        let p = canvas::Path::new(|p| {
            let mut line = |a: (f32, f32), b: (f32, f32)| {
                p.move_to(iced::Point::new(a.0 * s, a.1 * s));
                p.line_to(iced::Point::new(b.0 * s, b.1 * s));
            };
            match self.0 {
                "overview" => {
                    line((3., 4.), (9., 4.));
                    line((3., 4.), (3., 10.));
                    line((3., 10.), (9., 10.));
                    line((9., 4.), (9., 10.));
                    line((15., 4.), (21., 4.));
                    line((15., 4.), (15., 16.));
                    line((15., 16.), (21., 16.));
                    line((21., 4.), (21., 16.));
                    line((3., 16.), (9., 16.));
                    line((3., 16.), (3., 21.));
                    line((3., 21.), (9., 21.));
                    line((9., 16.), (9., 21.));
                }
                "ai" => {
                    line((12., 2.), (15., 9.));
                    line((15., 9.), (22., 12.));
                    line((22., 12.), (15., 15.));
                    line((15., 15.), (12., 22.));
                    line((12., 22.), (9., 15.));
                    line((9., 15.), (2., 12.));
                    line((2., 12.), (9., 9.));
                    line((9., 9.), (12., 2.));
                }
                "process" => {
                    line((3., 18.), (7., 18.));
                    line((7., 18.), (10., 5.));
                    line((10., 5.), (14., 20.));
                    line((14., 20.), (17., 11.));
                    line((17., 11.), (21., 11.));
                }
                "network" => {
                    line((12., 4.), (12., 12.));
                    line((4., 12.), (20., 12.));
                    line((4., 12.), (4., 20.));
                    line((20., 12.), (20., 20.));
                    line((12., 12.), (12., 20.));
                }
                "disk" => {
                    line((4., 5.), (20., 5.));
                    line((20., 5.), (20., 20.));
                    line((20., 20.), (4., 20.));
                    line((4., 20.), (4., 5.));
                    line((4., 15.), (20., 15.));
                    line((15., 18.), (17., 18.));
                }
                "history" => {
                    line((3., 18.), (7., 14.));
                    line((7., 14.), (11., 16.));
                    line((11., 16.), (15., 6.));
                    line((15., 6.), (21., 9.));
                }
                _ => (),
            }
        });
        if self.0 == "moon" {
            f.fill(
                &canvas::Path::circle(iced::Point::new(12. * s, 12. * s), 10. * s),
                self.1,
            );
            f.fill(
                &canvas::Path::circle(iced::Point::new(17. * s, 8. * s), 8. * s),
                NAV,
            );
        } else {
            f.stroke(&p, stroke);
        }
        vec![f.into_geometry()]
    }
}
