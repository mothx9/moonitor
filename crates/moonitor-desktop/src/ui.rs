use crate::{
    config::Config,
    engine::{Engine, NodeState},
    scene::Camera,
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

use crate::palette::*;

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
    pub theme_explicit: bool,
    pub scene_visible: bool,
    pub section: Option<Section>,
}
impl Default for DisplayOptions {
    fn default() -> Self {
        Self {
            size: iced::Size::new(1380.0, 940.0),
            dark: false,
            theme_explicit: false,
            scene_visible: false,
            section: None,
        }
    }
}
fn app_icon() -> Option<window::Icon> {
    let pixels = image::load_from_memory(include_bytes!("../assets/moonitor-256.png"))
        .ok()?
        .into_rgba8();
    window::icon::from_rgba(pixels.to_vec(), pixels.width(), pixels.height()).ok()
}
fn brand_icon() -> widget::Image {
    static ICON: std::sync::OnceLock<widget::image::Handle> = std::sync::OnceLock::new();
    let handle = ICON.get_or_init(|| {
        // RGBA handles upload synchronously: the first frame must contain the brand too.
        let pixels = image::load_from_memory(include_bytes!("../assets/moonitor-128.png"))
            .expect("embedded brand PNG")
            .into_rgba8();
        widget::image::Handle::from_rgba(pixels.width(), pixels.height(), pixels.into_raw())
    });
    widget::image(handle.clone()).width(34).height(34)
}
fn tone(color: Color, theme: &Theme) -> Color {
    resolve(color, theme.extended_palette().is_dark)
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
    Theme::custom(
        if dark {
            "Moonitor · Deep Space"
        } else {
            "Moonitor · Moonlight"
        },
        iced::theme::Palette {
            background: resolve(BG, dark),
            text: resolve(INK, dark),
            primary: resolve(ACCENT, dark),
            success: resolve(HEALTHY, dark),
            warning: resolve(AMBER, dark),
            danger: resolve(DANGER, dark),
        },
    )
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Tab {
    Overview,
    Compute,
    Sensors,
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
            "compute" => Self::Compute,
            "sensors" => Self::Sensors,
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
            Self::Overview => "Summary",
            Self::Compute => "Compute",
            Self::Sensors => "Sensori",
            Self::Processes => "Processi",
            Self::Ai => "AI locale",
            Self::Disks => "Archiviazione",
            Self::Network => "Rete",
            Self::History => "Storico",
        }
    }
}
const TABS: [Tab; 8] = [
    Tab::Overview,
    Tab::Compute,
    Tab::Sensors,
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
    SwitchNode(String),
    Focus(bool),
    NextNode(i32),
    Tab(Tab),
    Navigate(Section),
    Theme(bool),
    Filter(String),
    Sort(bool),
    Orbit(f32),
    Window(window::Id, window::Event),
    Screenshot(window::Screenshot, PathBuf),
    Preview(String),
    FleetFilter(String),
    FleetSort(instrument::FleetSort),
    FleetScroll(f32),
    Inspector,
    Rail,
    Density,
    Pause,
    OpenVisibleHost,
    DialogKey(iced::keyboard::Key),
    NewHost,
    EditHost(String),
    HostField(setup::Field, String),
    HostKind(crate::config::NodeKind),
    SaveHost,
    CancelHost,
    RequestRemove(String),
    RemoveHost(String),
    EngineReady(std::sync::Arc<std::sync::Mutex<Option<Engine>>>),
}
pub struct App {
    _engine: Option<Engine>,
    nodes: Vec<NodeState>,
    storage_error: Option<String>,
    dropped_events: u64,
    pending_samples: Vec<Instant>,
    selected: Option<String>,
    dark: bool,
    tab: Tab,
    section: Section,
    filter: String,
    sort_memory: bool,
    camera: Camera,
    started: Instant,
    view_time_ms: u64,
    screenshot: Option<PathBuf>,
    exit_after: Option<f32>,
    window: Option<window::Id>,
    profile: Option<crate::profile::Profile>,
    preferences: crate::profile::Preferences,
    data_dir: PathBuf,
    form: Option<setup::HostForm>,
    setup_error: Option<String>,
    setup_notice: Option<String>,
    remove_host: Option<String>,
    applying: bool,
    fleet_filter: String,
    fleet_sort: instrument::FleetSort,
    fleet_scroll: f32,
    viewport: iced::Size,
    paused: bool,
    frozen: Option<trust::FrozenView>,
    dialog_confirm_focused: bool,
}
pub fn run(
    data_dir: PathBuf,
    selected: usize,
    tab: Tab,
    screenshot: Option<PathBuf>,
    exit_after: Option<f32>,
    display: DisplayOptions,
    profile: crate::profile::Profile,
) -> anyhow::Result<()> {
    let config = profile.config.clone();
    let profile = std::cell::RefCell::new(Some(profile));
    iced::application(
        move || {
            let profile = profile
                .borrow_mut()
                .take()
                .expect("native application boot runs once");
            let preferences = profile.preferences.clone();
            let engine = Engine::start(config.clone(), data_dir.join("history.sqlite"));
            let nodes = engine.state.lock().unwrap().nodes.clone();
            App {
                _engine: Some(engine),
                selected: nodes.get(selected).map(|n| n.config.id.clone()),
                dark: if display.theme_explicit {
                    display.dark
                } else {
                    preferences.dark
                },
                nodes,
                storage_error: None,
                dropped_events: 0,
                pending_samples: Vec::new(),
                tab,
                section: display.section.unwrap_or(if tab == Tab::Overview {
                    Section::Overview
                } else {
                    Section::Device
                }),
                filter: String::new(),
                sort_memory: false,
                camera: Camera::default(),
                started: Instant::now(),
                view_time_ms: now_ms(),
                screenshot: screenshot.clone(),
                exit_after,
                window: None,
                profile: Some(profile),
                preferences,
                data_dir: data_dir.clone(),
                form: None,
                setup_error: None,
                setup_notice: None,
                remove_host: None,
                applying: false,
                fleet_filter: String::new(),
                fleet_sort: instrument::FleetSort::Name,
                fleet_scroll: 0.0,
                viewport: display.size,
                paused: false,
                frozen: None,
                dialog_confirm_focused: false,
            }
        },
        App::update,
        App::view,
    )
    .title("moonitor · Compute Observatory")
    .window(window::Settings {
        size: display.size,
        icon: app_icon(),
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
            if app.remove_host.is_some() {
                iced::event::listen_with(|event, _, _| match event {
                    iced::Event::Keyboard(iced::keyboard::Event::KeyPressed { key, .. }) => {
                        Some(Message::DialogKey(key))
                    }
                    _ => None,
                })
            } else {
                iced::event::listen_with(|event, status, _| {
                    let iced::Event::Keyboard(iced::keyboard::Event::KeyPressed {
                        key,
                        modifiers,
                        ..
                    }) = event
                    else {
                        return None;
                    };
                    if status == iced::event::Status::Captured {
                        return None;
                    }
                    use iced::keyboard::{Key, key::Named};
                    match key {
                        Key::Named(Named::Tab) => Some(Message::Focus(modifiers.shift())),
                        Key::Named(Named::ArrowLeft) if modifiers.alt() => {
                            Some(Message::NextNode(-1))
                        }
                        Key::Named(Named::ArrowRight) if modifiers.alt() => {
                            Some(Message::NextNode(1))
                        }
                        Key::Character(ref c) if modifiers.command() => c
                            .parse::<usize>()
                            .ok()
                            .filter(|i| (1..=7).contains(i))
                            .map(|i| Message::Navigate(SECTIONS[i - 1])),
                        _ => None,
                    }
                })
            },
        ])
    })
    .run()
    .map_err(|e| anyhow::anyhow!(e.to_string()))
}
impl App {
    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Tick => {
                self.refresh_live();
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
            Message::Window(id, window::Event::Opened { size, .. }) => {
                self.window = Some(id);
                self.viewport = size;
            }
            Message::Window(_, window::Event::Resized(size)) => self.viewport = size,
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
                if self.nodes.iter().any(|n| n.config.id == id)
                    && (!matches!(self.section, Section::Overview | Section::Fleet)
                        || self.filtered_nodes().iter().any(|n| n.config.id == id))
                {
                    self.selected = Some(id);
                    self.section = Section::Device;
                    self.tab = Tab::Overview;
                }
            }
            Message::Focus(reverse) => {
                return if reverse {
                    widget::operation::focus_previous()
                } else {
                    widget::operation::focus_next()
                };
            }
            Message::SwitchNode(id) => {
                if self.nodes.iter().any(|n| n.config.id == id) {
                    self.selected = Some(id);
                }
            }
            Message::NextNode(delta) => {
                let ids = if matches!(self.section, Section::Overview | Section::Fleet) {
                    self.filtered_nodes()
                        .iter()
                        .map(|n| n.config.id.clone())
                        .collect::<Vec<_>>()
                } else {
                    self.nodes.iter().map(|n| n.config.id.clone()).collect()
                };
                if !ids.is_empty() {
                    let next = self
                        .selected
                        .as_ref()
                        .and_then(|id| ids.iter().position(|v| v == id))
                        .map_or(0, |i| {
                            (i as i32 + delta).rem_euclid(ids.len() as i32) as usize
                        });
                    self.selected = Some(ids[next].clone());
                }
            }
            Message::Tab(tab) => {
                self.tab = tab;
                self.section = Section::Device;
            }
            Message::Navigate(section) => {
                self.section = section;
                if matches!(section, Section::Overview | Section::Fleet) {
                    self.fleet_scroll = 0.0;
                    return widget::operation::scroll_to(
                        "fleet-matrix",
                        widget::operation::AbsoluteOffset { x: 0.0, y: 0.0 },
                    );
                }
            }
            Message::Theme(dark) => {
                let mut preferences = self.preferences.clone();
                preferences.dark = dark;
                if self.save_preferences(preferences) {
                    self.dark = dark;
                }
            }
            Message::Filter(s) => self.filter = s,
            Message::Sort(memory) => self.sort_memory = memory,
            Message::Orbit(dx) => self.camera.yaw += dx,
            Message::Preview(id) => {
                if self.nodes.iter().any(|n| n.config.id == id) {
                    self.selected = Some(id);
                }
            }
            Message::FleetFilter(value) => {
                self.fleet_filter = value.chars().take(128).collect();
                self.fleet_scroll = 0.0;
                return widget::operation::scroll_to(
                    "fleet-matrix",
                    widget::operation::AbsoluteOffset { x: 0.0, y: 0.0 },
                );
            }
            Message::FleetSort(sort) => {
                self.fleet_sort = sort;
                self.fleet_scroll = 0.0;
                return widget::operation::scroll_to(
                    "fleet-matrix",
                    widget::operation::AbsoluteOffset { x: 0.0, y: 0.0 },
                );
            }
            Message::FleetScroll(offset) => self.fleet_scroll = offset,
            Message::Inspector => {
                let mut p = self.preferences.clone();
                p.inspector = !p.inspector;
                self.save_preferences(p);
            }
            Message::Rail => {
                let mut p = self.preferences.clone();
                p.expanded_rail = !p.expanded_rail;
                self.save_preferences(p);
            }
            Message::Density => {
                let mut p = self.preferences.clone();
                p.comfortable = !p.comfortable;
                self.save_preferences(p);
                self.fleet_scroll = 0.0;
            }
            Message::Pause => self.toggle_pause(),
            Message::OpenVisibleHost => {
                if let Some(id) = self.visible_selection() {
                    return self.update(Message::Select(id));
                }
            }
            Message::DialogKey(key) => return self.dialog_key(key),
            Message::NewHost
            | Message::EditHost(_)
            | Message::HostField(_, _)
            | Message::HostKind(_)
            | Message::SaveHost
            | Message::CancelHost
            | Message::RequestRemove(_)
            | Message::RemoveHost(_) => return self.setup_update(message),
            Message::EngineReady(slot) => {
                self._engine = slot.lock().unwrap().take();
                self.applying = false;
                self.setup_notice = Some("Configurazione applicata. Raccolta aggiornata.".into());
            }
        }
        Task::none()
    }
    fn view(&self) -> Element<'_, Message> {
        self.with_dialog(self.workbench_view())
    }
    fn node_heading<'a>(&self, n: &'a NodeState) -> Element<'a, Message> {
        let (state, color) = self.health_status(n);
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
        if self.observation_stale(n) && n.sample.is_some() {
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
                    text(self.health_status(n).0).size(18).font(STRONG),
                    text(&n.config.note).tone(MUTED).size(12),
                    text("Questa vista richiede un agente sul dispositivo. Controlla il tipo di nodo e l'alias SSH in fleet.toml; un dispositivo senza agente resta visibile nell'inventario.").size(13),
                    small_button("Vai alla flotta", Message::Navigate(Section::Fleet))
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
                        plot("CPU", &n.history, self.view_time_ms, 0, CPU),
                        plot("Memoria", &n.history, self.view_time_ms, 1, MEMORY),
                        plot("GPU", &n.history, self.view_time_ms, 2, GPU)
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
                        canvas(Icon("ai", ACCENT)).width(20).height(20),
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
            Tab::Compute => {
                let mut c =
                    column![text("CPU · carico per core").size(18).font(STRONG)].spacing(14);
                for (i, value) in s.cpu_cores_percent.iter().enumerate() {
                    let reading = Reading::reported(
                        *value as f64,
                        "sysinfo · CPU core",
                        s.sampled_at_unix_ms,
                    );
                    c = c.push(metric(
                        Metric::percent(
                            &format!("Core {i}"),
                            Some(&reading),
                            self.observation_stale(n),
                        ),
                        ACCENT,
                    ));
                }
                for gpu in &s.gpus {
                    c = c.push(card(
                        column![
                            text(&gpu.name).size(17).font(STRONG),
                            metric(
                                Metric::percent(
                                    "Utilizzo GPU",
                                    Some(&gpu.utilization_percent),
                                    self.observation_stale(n)
                                ),
                                ACCENT
                            )
                        ]
                        .spacing(12),
                    ));
                }
                c.into()
            }
            Tab::Sensors => {
                let mut c = column![text("Sensori disponibili").size(18).font(STRONG)].spacing(14);
                if s.temperatures.is_empty()
                    && s.pressure.is_empty()
                    && s.battery_percent.value().is_none()
                {
                    c = c.push(notice(
                        "Questo agente non espone sensori aggiuntivi su questo nodo.",
                    ));
                }
                for t in &s.temperatures {
                    c = c.push(trust::thermal(t, s.sampled_at_unix_ms));
                }
                for p in &s.pressure {
                    c = c.push(table_row(
                        [
                            &p.resource,
                            &format!("{:.1}%", p.some_avg10_percent),
                            "Stallo PSI · avg10",
                            "Linux",
                        ],
                        false,
                    ));
                }
                c.push(metric(
                    Metric::percent(
                        "Batteria",
                        Some(&s.battery_percent),
                        self.observation_stale(n),
                    ),
                    ACCENT,
                ))
                .into()
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
                                Metric::percent(
                                    "CARICO",
                                    Some(&g.utilization_percent),
                                    self.observation_stale(n)
                                ),
                                ACCENT
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
                                .tone(ACCENT),
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
                                Metric {label:"Spazio utilizzato".into(), value:used, detail:format!("{} disponibili / {} totali", bytes(Some(d.available_bytes)),bytes(Some(d.total_bytes))), provenance:format!("sysinfo filesystem · (totale − disponibile) / totale · {} ms Unix",d.sampled_at_unix_ms), stale:self.observation_stale(n)},
                                ACCENT
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
                        plot("CPU", &n.history, self.view_time_ms, 0, CPU),
                        plot("Memoria", &n.history, self.view_time_ms, 1, MEMORY),
                        plot("GPU", &n.history, self.view_time_ms, 2, GPU),
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
                    health = health.push(trust::thermal(t, s.sampled_at_unix_ms));
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
                radius: 12.into(),
            },
            ..Default::default()
        },
    )
}
fn card<'a>(content: impl Into<Element<'a, Message>>) -> widget::Container<'a, Message> {
    container(content).padding(18).width(Fill).style(panel)
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
                    background: Some(WARNING_BG.into()),
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
        content = content.push(progress_bar(0.0..=100.0, value as f32).girth(4).style(
            move |theme| progress_bar::Style {
                background: tone(LINE, theme).into(),
                bar: tone(c, theme).into(),
                border: Border {
                    radius: 2.into(),
                    ..Default::default()
                },
            },
        ));
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
    let mut r = Row::new().spacing(20).width(Fill);
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
struct Trend {
    points: Vec<(f32, Option<f64>)>,
    color: Color,
}
fn plot<'a>(
    label: &str,
    points: &std::collections::VecDeque<HistoryPoint>,
    end_ms: u64,
    field: u8,
    color: Color,
) -> Element<'a, Message> {
    let since = end_ms.saturating_sub(300_000);
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
                            .with_color(tone(self.color, theme))
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
                "energy" => {
                    line((14., 2.), (5., 14.));
                    line((5., 14.), (11., 14.));
                    line((11., 14.), (10., 22.));
                    line((10., 22.), (20., 10.));
                    line((20., 10.), (13., 10.));
                    line((13., 10.), (14., 2.));
                }
                "events" => {
                    line((4., 5.), (20., 5.));
                    line((20., 5.), (20., 18.));
                    line((20., 18.), (4., 18.));
                    line((4., 18.), (4., 5.));
                    line((8., 9.), (16., 9.));
                    line((8., 13.), (13., 13.));
                    line((8., 21.), (16., 21.));
                }
                "settings" => {
                    line((3., 6.), (21., 6.));
                    line((3., 12.), (21., 12.));
                    line((3., 18.), (21., 18.));
                    line((8., 3.), (8., 9.));
                    line((16., 9.), (16., 15.));
                    line((10., 15.), (10., 21.));
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

mod chart;
mod dialog;
mod instrument;
mod setup;
mod trust;
mod workbench;
use workbench::SECTIONS;
pub use workbench::Section;
#[cfg(test)]
mod qa;
