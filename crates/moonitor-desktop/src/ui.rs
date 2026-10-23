use crate::{
    config::Config,
    engine::{Engine, NodeState},
    scene::{Camera, Scene},
    view_model::{FleetViewModel, Metric, NodeStatus},
};
use iced::{
    Border, Color, Element, Font,
    Length::{self, Fill},
    Subscription, Task, Theme, alignment,
    widget::{
        self, Column, Row, Space, button, canvas, column, container, progress_bar, row, scrollable,
        text, text_input, tooltip,
    },
    window,
};
use moonitor_core::*;
use moonitor_store::HistoryPoint;
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
const FONT: Font = Font::with_name("Inter");
const MEDIUM: Font = Font {
    weight: iced::font::Weight::Medium,
    ..FONT
};
const STRONG: Font = Font {
    weight: iced::font::Weight::Semibold,
    ..FONT
};
#[derive(Clone, Copy)]
pub struct DisplayOptions {
    pub size: iced::Size,
    pub dark: bool,
    pub scene_visible: bool,
}
impl Default for DisplayOptions {
    fn default() -> Self {
        Self {
            size: iced::Size::new(1380.0, 940.0),
            dark: false,
            scene_visible: true,
        }
    }
}
fn tone(color: Color, theme: &Theme) -> Color {
    if !theme.extended_palette().is_dark {
        return color;
    }
    if color == Color::from_rgb(0.72, 0.25, 0.23) {
        return Color::from_rgb(0.94, 0.57, 0.53);
    }
    if color == Color::from_rgb(0.70, 0.45, 0.15) {
        return Color::from_rgb(0.94, 0.72, 0.40);
    }
    if color == Color::from_rgb(0.35, 0.46, 0.65) {
        return Color::from_rgb(0.62, 0.74, 0.94);
    }
    if color == Color::from_rgb(0.48, 0.51, 0.50) {
        return Color::from_rgb(0.66, 0.72, 0.68);
    }
    if color == BG {
        Color::from_rgb(0.051, 0.086, 0.075)
    } else if color == PAPER {
        Color::from_rgb(0.078, 0.125, 0.102)
    } else if color == INK {
        Color::from_rgb(0.918, 0.949, 0.925)
    } else if color == MUTED {
        Color::from_rgb(0.64, 0.72, 0.67)
    } else if color == LINE {
        Color::from_rgb(0.20, 0.29, 0.24)
    } else if color == GREEN {
        Color::from_rgb(0.48, 0.75, 0.58)
    } else if color.r > 0.8 && color.g > 0.8 && color.b > 0.7 && color != Color::WHITE {
        Color::from_rgb(0.12, 0.21, 0.16)
    } else {
        color
    }
}
trait TextTone {
    fn tone(self, color: Color) -> Self;
}
impl TextTone for widget::Text<'_> {
    fn tone(self, color: Color) -> Self {
        self.style(move |theme| text::Style {
            color: Some(if color == Color::WHITE {
                color
            } else {
                tone(color, theme)
            }),
        })
    }
}
fn themed_panel(theme: &Theme, mut style: container::Style) -> container::Style {
    if let Some(iced::Background::Color(c)) = style.background {
        style.background = Some(tone(c, theme).into());
    }
    if let Some(c) = style.text_color {
        style.text_color = Some(tone(c, theme));
    }
    style.border.color = tone(style.border.color, theme);
    style
}
fn themed_button(theme: &Theme, mut style: button::Style) -> button::Style {
    if let Some(iced::Background::Color(c)) = style.background {
        style.background = Some(tone(c, theme).into());
    }
    style.text_color = tone(style.text_color, theme);
    style.border.color = tone(style.border.color, theme);
    style
}
fn app_theme(dark: bool) -> Theme {
    if dark {
        Theme::custom(
            "moonitor dark",
            iced::theme::Palette {
                background: Color::from_rgb(0.051, 0.086, 0.075),
                text: Color::from_rgb(0.918, 0.949, 0.925),
                primary: GREEN,
                success: GREEN,
                warning: AMBER,
                danger: AMBER,
            },
        )
    } else {
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
    }
}
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
    CaptureScreenshot,
    Frame,
    Select(String),
    ToggleScene,
    Focus(bool),
    NextNode(i32),
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
    _engine: Option<Engine>,
    nodes: Vec<NodeState>,
    storage_error: Option<String>,
    dropped_events: u64,
    pending_samples: Vec<Instant>,
    selected: Option<String>,
    scene_visible: bool,
    dark: bool,
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
    display: DisplayOptions,
) -> anyhow::Result<()> {
    iced::application(
        move || {
            let engine = Engine::start(config.clone(), data_dir.join("history.sqlite"));
            let nodes = engine.state.lock().unwrap().nodes.clone();
            App {
                _engine: Some(engine),
                selected: nodes.get(selected).map(|n| n.config.id.clone()),
                scene_visible: display.scene_visible,
                dark: display.dark,
                nodes,
                storage_error: None,
                dropped_events: 0,
                pending_samples: Vec::new(),
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
        size: display.size,
        min_size: Some(iced::Size::new(1160.0, 760.0)),
        position: window::Position::Centered,
        ..Default::default()
    })
    .default_font(FONT)
    .font(include_bytes!("../assets/Inter-Regular.ttf").as_slice())
    .font(include_bytes!("../assets/Inter-Medium.ttf").as_slice())
    .font(include_bytes!("../assets/Inter-SemiBold.ttf").as_slice())
    .theme(|app: &App| app_theme(app.dark))
    .subscription(|app: &App| {
        Subscription::batch([
            iced::time::every(Duration::from_secs(1)).map(|_| Message::Tick),
            window::events().map(|(id, e)| Message::Window(id, e)),
            if app.pending_samples.is_empty() {
                Subscription::none()
            } else {
                window::frames().map(|_| Message::Frame)
            },
            iced::event::listen_with(|event, status, _| {
                if status == iced::event::Status::Captured {
                    return None;
                }
                let iced::Event::Keyboard(iced::keyboard::Event::KeyPressed {
                    key, modifiers, ..
                }) = event
                else {
                    return None;
                };
                use iced::keyboard::{Key, key::Named};
                match key {
                    Key::Named(Named::Tab) => Some(Message::Focus(modifiers.shift())),
                    Key::Named(Named::ArrowLeft) if modifiers.alt() => Some(Message::NextNode(-1)),
                    Key::Named(Named::ArrowRight) if modifiers.alt() => Some(Message::NextNode(1)),
                    Key::Character(ref c) if modifiers.command() => c
                        .parse::<usize>()
                        .ok()
                        .filter(|i| (1..=6).contains(i))
                        .map(|i| Message::Tab(TABS[i - 1])),
                    _ => None,
                }
            }),
        ])
    })
    .run()
    .map_err(|e| anyhow::anyhow!(e.to_string()))
}
impl App {
    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Tick => {
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
                self.selected =
                    FleetViewModel::new(&self.nodes, self.selected.as_deref(), Instant::now())
                        .selected_id;
                if self
                    .exit_after
                    .is_some_and(|s| self.started.elapsed().as_secs_f32() >= s)
                {
                    return iced::exit();
                }
                if self.started.elapsed() >= Duration::from_secs(8)
                    && self.window.is_some()
                    && self.screenshot.is_some()
                {
                    // Capture on a separate event-loop turn: Tick replaces paragraphs.
                    // Capturing the previous renderer batch here leaves dangling weak text references.
                    return Task::perform(
                        async {
                            std::thread::sleep(Duration::from_millis(150));
                        },
                        |_| Message::CaptureScreenshot,
                    );
                }
            }
            Message::Frame => {
                for seen in self.pending_samples.drain(..) {
                    eprintln!(
                        "moonitor-ui-latency: receipt_to_redraw_ms={:.3}",
                        seen.elapsed().as_secs_f64() * 1000.0
                    );
                }
            }
            Message::CaptureScreenshot => {
                if let (Some(window), Some(path)) = (self.window, self.screenshot.take()) {
                    return window::screenshot(window)
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
            Message::Select(id) => {
                if self.nodes.iter().any(|n| n.config.id == id) {
                    self.selected = Some(id);
                }
            }
            Message::ToggleScene => self.scene_visible = !self.scene_visible,
            Message::Focus(reverse) => {
                return if reverse {
                    widget::operation::focus_previous()
                } else {
                    widget::operation::focus_next()
                };
            }
            Message::NextNode(delta) => {
                if !self.nodes.is_empty() {
                    let current = self
                        .nodes
                        .iter()
                        .position(|n| Some(n.config.id.as_str()) == self.selected.as_deref())
                        .unwrap_or(0);
                    let next =
                        (current as i32 + delta).rem_euclid(self.nodes.len() as i32) as usize;
                    self.selected = Some(self.nodes[next].config.id.clone());
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
        let fleet = FleetViewModel::new(&self.nodes, self.selected.as_deref(), Instant::now());
        let Some(node) = self
            .nodes
            .iter()
            .find(|n| Some(n.config.id.as_str()) == fleet.selected_id.as_deref())
        else {
            return container(column![text("Nessun dispositivo configurato").size(24), text("Aggiungi un nodo local o ssh nel file fleet.toml. I dispositivi senza agente possono essere censiti come unavailable.").size(14)].spacing(16)).padding(32).into();
        };
        let header = row![
            column![
                eyebrow("WORKSPACE / HOMELAB"),
                text(self.tab.label()).size(34).font(STRONG)
            ]
            .spacing(8),
            Space::new().width(Fill),
            pill(
                &format!(
                    "{} / {} monitorati online · {} censiti",
                    fleet.online,
                    fleet.monitored,
                    fleet.devices.len()
                ),
                GREEN
            ),
            text("LOCALE  /  VPN").size(10).tone(MUTED)
        ]
        .spacing(20)
        .align_y(alignment::Vertical::Center);
        let mut body = column![header].spacing(22).width(Fill);
        if let Some(error) = &self.storage_error {
            body = body.push(notice(&format!("Storico non disponibile: {error}")));
        }
        if self.dropped_events > 0 {
            body = body.push(notice(&format!(
                "Coda di acquisizione piena: {} aggiornamenti scartati",
                self.dropped_events
            )));
        }
        if self.tab == Tab::Overview {
            let scene = Scene {
                selected: fleet.selected_id.clone(),
                devices: fleet.devices.clone(),
                dark: self.dark,
                camera: self.camera,
            };
            let controls = row![
                text("IL TUO HOMELAB").size(10).font(MEDIUM).tone(MUTED),
                Space::new().width(Fill),
                small_button(
                    if self.scene_visible {
                        "Solo elenco"
                    } else {
                        "Mostra 3D"
                    },
                    Message::ToggleScene
                ),
                small_button("−", Message::Zoom(-0.1)),
                small_button("+", Message::Zoom(0.1)),
                small_button("Ripristina", Message::ResetCamera)
            ]
            .spacing(6)
            .align_y(alignment::Vertical::Center);
            let scene_panel = container(
                column![
                    container(controls).padding([14, 18]),
                    if self.scene_visible {
                        Element::from(widget::stack![
                            widget::shader(scene.clone()).width(Fill).height(300),
                            canvas(scene).width(Fill).height(300)
                        ])
                    } else {
                        Element::from(container(self.scene_list()).height(300).padding(18))
                    },
                    container(
                        row![
                            text(format!("{:02}", self.nodes.len()))
                                .font(STRONG)
                                .size(12)
                                .tone(GREEN),
                            text("DISPOSITIVI").size(10).tone(MUTED),
                            Space::new().width(Fill),
                            text(if self.scene_visible {
                                "Trascina per ruotare · clicca per selezionare"
                            } else {
                                "Seleziona un dispositivo · elenco sincronizzato"
                            })
                            .size(10)
                            .tone(MUTED)
                        ]
                        .spacing(8)
                    )
                    .padding([10, 18])
                ]
                .spacing(0),
            )
            .style(|theme| {
                themed_panel(
                    theme,
                    container::Style {
                        background: Some(Color::from_rgb(0.95, 0.956, 0.938).into()),
                        border: Border {
                            radius: 18.into(),
                            color: LINE,
                            width: 1.0,
                        },
                        ..Default::default()
                    },
                )
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
            .width(Fill)
            .height(Fill)
            .into()
    }
    fn sidebar(&self) -> Element<'_, Message> {
        let brand = row![
            canvas(Icon("moon", LIME)).width(28).height(28),
            text("moonitor").size(23).font(STRONG).tone(Color::WHITE)
        ]
        .spacing(8)
        .align_y(alignment::Vertical::Center);
        let mut nav = column![
            container(brand).padding([8, 0]),
            text("PERSONAL OBSERVATORY")
                .size(8)
                .tone(Color::from_rgb(0.49, 0.60, 0.53)),
            Space::new().height(30),
            text("MONITORAGGIO")
                .size(9)
                .tone(Color::from_rgb(0.47, 0.57, 0.51))
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
                .style(move |theme, state| {
                    themed_button(
                        theme,
                        button::Style {
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
                        },
                    )
                }),
            );
        }
        nav = nav.push(Space::new().height(32)).push(
            text("I TUOI DISPOSITIVI")
                .size(9)
                .tone(Color::from_rgb(0.47, 0.57, 0.51)),
        );
        for node in &self.nodes {
            let (_, color) = status(node);
            nav = nav.push(
                button(
                    row![
                        text("•").tone(color).size(15),
                        text(&node.config.label).size(11).tone(
                            if self.selected.as_deref() == Some(node.config.id.as_str()) {
                                Color::WHITE
                            } else {
                                Color::from_rgb(0.58, 0.67, 0.61)
                            }
                        )
                    ]
                    .spacing(10),
                )
                .on_press(Message::Select(node.config.id.clone()))
                .style(button::text)
                .padding([7, 9])
                .width(Fill),
            );
        }
        let footer = column![
            row![
                text("•").tone(LIME),
                text("Raccolta attiva")
                    .size(11)
                    .tone(Color::from_rgb(0.73, 0.81, 0.74))
            ]
            .spacing(8),
            text("1 s · SQLite locale")
                .size(10)
                .tone(Color::from_rgb(0.44, 0.55, 0.48)),
            text("moonitor  /  0.1.0")
                .size(9)
                .tone(Color::from_rgb(0.37, 0.47, 0.41))
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
        .style(|theme| {
            themed_panel(
                theme,
                container::Style {
                    background: Some(NAV.into()),
                    ..Default::default()
                },
            )
        })
        .into()
    }
    fn scene_list(&self) -> Element<'_, Message> {
        let mut rows = Column::new().spacing(8);
        for node in &self.nodes {
            let (state, color) = status(node);
            rows = rows.push(
                button(
                    row![
                        text(&node.config.label).size(14).width(Fill),
                        text(state).size(12).tone(color)
                    ]
                    .spacing(16),
                )
                .on_press(Message::Select(node.config.id.clone()))
                .width(Fill)
                .padding(10)
                .style(button::secondary),
            );
        }
        scrollable(rows).height(Fill).into()
    }
    fn device_strip(&self) -> Element<'_, Message> {
        let mut devices = Row::new().spacing(10);
        for (i, n) in self.nodes.iter().enumerate() {
            let chosen = self.selected.as_deref() == Some(n.config.id.as_str());
            let (state, color) = status(n);
            let cpu = n
                .sample
                .as_ref()
                .and_then(|s| s.cpu_utilization_percent.value().copied());
            let content = column![
                row![
                    text(format!("{:02}", i + 1)).size(10).tone(MUTED),
                    Space::new().width(Fill),
                    text("•").tone(color)
                ]
                .align_y(alignment::Vertical::Center),
                text(&n.config.label).font(STRONG).size(13),
                row![
                    text(format!("{} · CPU {}", state, pct(cpu)))
                        .size(11)
                        .tone(if n.live() { INK } else { color }),
                ]
            ]
            .spacing(10);
            devices = devices.push(
                button(content)
                    .on_press(Message::Select(n.config.id.clone()))
                    .padding(14)
                    .width(190)
                    .style(move |theme, s| {
                        themed_button(
                            theme,
                            button::Style {
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
                            },
                        )
                    }),
            );
        }
        scrollable(devices)
            .direction(scrollable::Direction::Horizontal(
                scrollable::Scrollbar::default(),
            ))
            .into()
    }
    fn inspector<'a>(&self, n: &'a NodeState) -> Element<'a, Message> {
        let (state, color) = status(n);
        let mut content = column![
            row![
                eyebrow("DISPOSITIVO SELEZIONATO"),
                Space::new().width(Fill),
                text("•").tone(color)
            ]
            .spacing(8),
            text(&n.config.label).size(25).font(STRONG),
            text(state).size(10).font(MEDIUM).tone(color)
        ]
        .spacing(10);
        if let Some(s) = &n.sample {
            content = content
                .push(text(short_cpu(&s.cpu_name)).size(11).tone(MUTED))
                .push(Space::new().height(8));
            let total = s.memory.total_bytes.value().copied();
            let used = total
                .zip(s.memory.available_bytes.value().copied())
                .map(|(t, a)| t.saturating_sub(a));
            let stale = !n.live();
            let mut cpu = Metric::percent("CPU", Some(&s.cpu_utilization_percent), stale);
            cpu.detail = format!("{} core logici", s.cpu_cores_percent.len());
            content = content.push(metric(cpu, GREEN));
            let mut memory = Metric::memory(s, stale);
            memory.detail = format!("{} / {}", bytes(used), bytes(total));
            content = content.push(metric(memory, Color::from_rgb(0.49, 0.58, 0.41)));
            content = content.push(metric(
                Metric::gpu(s, stale),
                Color::from_rgb(0.51, 0.58, 0.64),
            ));
            content = content.push(
                row![
                    text("Potenza GPU").size(11).tone(MUTED),
                    Space::new().width(Fill),
                    text(watts(gpu_power(s))).size(15).font(MEDIUM)
                ]
                .align_y(alignment::Vertical::Center),
            );
            content = content.push(text("Intero dispositivo: N/D").size(9).tone(MUTED));
        } else {
            content = content
                .push(Space::new().height(18))
                .push(text(&n.config.note).size(13).tone(MUTED))
                .push(
                    text(
                        n.error
                            .as_deref()
                            .unwrap_or("In attesa dei primi campioni."),
                    )
                    .size(11)
                    .tone(MUTED),
                );
        }
        if n.sample.is_some() && !n.live() {
            content = content.push(
                text("Ultimo campione conservato · dati obsoleti")
                    .size(10)
                    .tone(AMBER),
            );
        }
        container(content)
            .padding(22)
            .width(286)
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
                    .tone(MUTED),
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
                    text(status(n).0).size(18).font(STRONG),
                    text(&n.config.note).tone(MUTED).size(12),
                    text("Questa vista richiede un agente sul dispositivo. Controlla il tipo di nodo e l'alias SSH in fleet.toml; un dispositivo senza agente resta visibile nell'inventario.").size(13),
                    small_button("Torna alla panoramica", Message::Tab(Tab::Overview))
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
                        text("ULTIMI 5 MINUTI").size(9).tone(MUTED)
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
                    None => format!("Ollama: {}", reading_reason(&s.ollama.models)),
                };
                content = content.push(widget::rule::horizontal(1)).push(
                    row![
                        canvas(Icon("ai", GREEN)).width(20).height(20),
                        column![
                            text("AI locale").size(12).font(STRONG),
                            text(runtime).size(11).tone(MUTED)
                        ]
                        .spacing(5),
                        Space::new().width(Fill),
                        text(format!("{ai} processi candidati"))
                            .size(11)
                            .tone(MUTED),
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
                    text("MISURE DEL RUNTIME E DEL SISTEMA").size(9).tone(MUTED)
                ]]
                .spacing(18);
                for g in &s.gpus {
                    let mut gpu = column![
                        text(&g.name).size(17).font(STRONG),
                        row![
                            metric(
                                Metric::percent("CARICO", Some(&g.utilization_percent), !n.live()),
                                GREEN
                            ),
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
                            mini_stat(
                                "POTENZA SCHEDA",
                                &watts(if s.gpus.len() == 1 {
                                    gpu_power(s)
                                } else {
                                    None
                                })
                            )
                        ]
                        .spacing(24)
                    ]
                    .spacing(18);
                    if g.memory_total_bytes.value().is_none()
                        || g.memory_used_bytes.value().is_none()
                    {
                        gpu = gpu.push(
                            text(format!(
                                "Memoria GPU: totale {}, allocata {}",
                                reading_reason(&g.memory_total_bytes),
                                reading_reason(&g.memory_used_bytes)
                            ))
                            .size(11)
                            .tone(MUTED),
                        );
                    }
                    if g.memory_kind == GpuMemoryKind::Unified {
                        gpu=gpu.push(text("Memoria unificata: le allocazioni GPU condividono la RAM del nodo.").size(11).tone(MUTED));
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
                        .tone(MUTED),
                    );
                    c = c.push(card(gpu));
                }
                c = c.push(text("Modelli caricati · Ollama").size(17).font(STRONG));
                match s.ollama.models.value() {
                    Some(models) if models.is_empty() => {
                        c = c.push(
                            text("Runtime disponibile. Nessun modello caricato.")
                                .size(13)
                                .tone(GREEN),
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
                                    .tone(MUTED)
                                ]
                                .spacing(10),
                            ));
                        }
                    }
                    None => {
                        c = c.push(
                            text(format!("Ollama: {}", reading_reason(&s.ollama.models)))
                                .size(12)
                                .tone(MUTED),
                        )
                    }
                }
                c=c.push(text("Token/s, latenza e code: N/D senza telemetria delle richieste dal runtime.").size(11).tone(MUTED)).push(text("Processi AI riconosciuti dal nome").size(17).font(STRONG)).push(self.processes(s,true));
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
                            text(&d.name).size(11).tone(MUTED),
                            metric(
                                Metric {label:"Spazio utilizzato".into(), value:used, detail:format!("{} disponibili / {} totali", bytes(Some(d.available_bytes)),bytes(Some(d.total_bytes))), provenance:format!("sysinfo filesystem · (totale − disponibile) / totale · {} ms Unix",d.sampled_at_unix_ms), stale:!n.live()},
                                GREEN
                            )
                        ]
                        .spacing(12),
                    ));
                }
                c.push(text("Capacità aggiornata ogni 5 s. I volumi APFS possono condividere lo stesso spazio.").size(11).tone(MUTED)).into()
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
                c=c.push(text("Interfacce fisiche, VPN e virtuali possono riportare lo stesso traffico: non vengono sommate.").size(11).tone(MUTED));
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
                            .tone(MUTED)
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
                    .tone(MUTED)
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
                            text(i.to_string()).size(8).tone(MUTED),
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
                .id("process-filter")
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
        let mut c=column![controls,text(format!("{} processi sul nodo · sottoinsieme più attivo e candidati AI · 100% CPU = un core",s.process_count)).size(10).tone(MUTED),table_row(["PROCESSO","PID","CPU","MEMORIA"],true)].spacing(14);
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
                    .tone(MUTED),
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
fn reading_reason<T>(reading: &Reading<T>) -> &str {
    match reading {
        Reading::Available { .. } => "disponibile",
        Reading::Unavailable { reason } => crate::view_model::unavailable(*reason),
    }
}
fn status(n: &NodeState) -> (&'static str, Color) {
    let status = NodeStatus::of(n, Instant::now());
    let c = status.color();
    (status.label(), Color::from_rgb(c[0], c[1], c[2]))
}
fn panel(theme: &Theme) -> container::Style {
    themed_panel(
        theme,
        container::Style {
            background: Some(PAPER.into()),
            border: Border {
                color: LINE,
                width: 1.0,
                radius: 16.into(),
            },
            ..Default::default()
        },
    )
}
fn card<'a>(content: impl Into<Element<'a, Message>>) -> widget::Container<'a, Message> {
    container(content).padding(22).width(Fill).style(panel)
}
fn eyebrow(s: &str) -> widget::Text<'_> {
    text(s).size(9).font(MEDIUM).tone(MUTED)
}
fn pill(s: &str, c: Color) -> Element<'static, Message> {
    container(text(s.to_owned()).size(10).font(MEDIUM).tone(c))
        .padding([7, 10])
        .style(move |theme| {
            themed_panel(
                theme,
                container::Style {
                    background: Some(Color { a: 0.08, ..c }.into()),
                    border: Border {
                        radius: 20.into(),
                        ..Default::default()
                    },
                    ..Default::default()
                },
            )
        })
        .into()
}
fn small_button(s: &str, message: Message) -> Element<'static, Message> {
    button(text(s.to_owned()).size(11).font(MEDIUM))
        .on_press(message)
        .padding([7, 10])
        .style(|theme, state| {
            themed_button(
                theme,
                button::Style {
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
                },
            )
        })
        .into()
}
fn notice(s: &str) -> Element<'static, Message> {
    container(text(s.to_owned()).size(11).tone(AMBER))
        .padding(12)
        .width(Fill)
        .style(|theme| {
            themed_panel(
                theme,
                container::Style {
                    background: Some(Color::from_rgb(0.98, 0.94, 0.86).into()),
                    border: Border {
                        radius: 8.into(),
                        ..Default::default()
                    },
                    ..Default::default()
                },
            )
        })
        .into()
}
fn metric<'a>(metric: Metric, c: Color) -> Element<'a, Message> {
    let explanation = metric.explanation();
    let mut content = column![
        row![
            text(metric.label.clone()).size(12).font(MEDIUM),
            Space::new().width(Fill),
            text(metric.text())
                .size(19)
                .font(STRONG)
                .wrapping(text::Wrapping::None)
        ]
        .align_y(alignment::Vertical::Center),
    ]
    .spacing(7);
    if let Some(value) = metric.value {
        content = content.push(
            progress_bar(0.0..=100.0, value as f32)
                .girth(4)
                .style(move |_| progress_bar::Style {
                    background: LINE.into(),
                    bar: c.into(),
                    border: Border {
                        radius: 2.into(),
                        ..Default::default()
                    },
                }),
        );
    } else {
        content = content.push(text(metric.provenance.clone()).size(11).tone(MUTED));
    }
    if !metric.detail.is_empty() {
        content = content.push(text(metric.detail).size(10).tone(MUTED));
    }
    if metric.stale {
        content = content.push(text("Ultima lettura · obsoleta").size(10).tone(AMBER));
    }
    tooltip(
        content.width(Fill),
        container(text(explanation).size(12))
            .padding(12)
            .style(panel),
        tooltip::Position::Bottom,
    )
    .into()
}
fn mini_stat<'a>(label: &str, value: &str) -> Element<'a, Message> {
    column![
        text(label.to_owned()).size(9).tone(MUTED),
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
                .tone(if heading { MUTED } else { INK })
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
fn gpu_power(s: &HostSnapshot) -> Option<f64> {
    let readings = s
        .power
        .iter()
        .filter(|p| p.scope == PowerScope::GpuBoard)
        .map(|p| p.watts.value().copied())
        .collect::<Option<Vec<_>>>()?;
    (!readings.is_empty()).then(|| readings.iter().sum())
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
            text(label.to_owned()).size(11).tone(MUTED),
            Space::new().width(Fill),
            text(pct(last))
                .size(15)
                .font(MEDIUM)
                .wrapping(text::Wrapping::None)
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
        theme: &Theme,
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
                canvas::Stroke::default()
                    .with_color(tone(LINE, theme))
                    .with_width(0.5),
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
                    && p.x > prev.x
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
        theme: &Theme,
        bounds: iced::Rectangle,
        _: iced::mouse::Cursor,
    ) -> Vec<canvas::Geometry> {
        let mut f = canvas::Frame::new(renderer, bounds.size());
        let s = bounds.width / 24.0;
        let stroke = canvas::Stroke::default()
            .with_width(1.5 * s)
            .with_color(tone(self.1, theme));
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

#[cfg(test)]
mod qa;
