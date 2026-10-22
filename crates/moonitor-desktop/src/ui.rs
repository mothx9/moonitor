use crate::{
    config::NodeKind,
    engine::{Engine, NodeState},
};
use eframe::egui::{self, Color32, RichText, Stroke, Vec2};
use moonitor_core::*;
use moonitor_store::{HistoryPoint, memory_percent};
use std::{
    path::PathBuf,
    time::{Duration, Instant},
};

const BG: Color32 = Color32::from_rgb(11, 16, 23);
const CARD: Color32 = Color32::from_rgb(19, 27, 38);
const BORDER: Color32 = Color32::from_rgb(37, 49, 64);
const TEXT: Color32 = Color32::from_rgb(228, 236, 245);
const MUTED: Color32 = Color32::from_rgb(132, 151, 172);
const CYAN: Color32 = Color32::from_rgb(86, 211, 215);
const GREEN: Color32 = Color32::from_rgb(130, 214, 160);
const AMBER: Color32 = Color32::from_rgb(239, 187, 105);

#[derive(Clone, Copy, PartialEq)]
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
}

pub fn set_style(ctx: &egui::Context) {
    let mut visuals = egui::Visuals::dark();
    visuals.panel_fill = BG;
    visuals.window_fill = CARD;
    visuals.override_text_color = Some(TEXT);
    visuals.selection.bg_fill = Color32::from_rgb(35, 78, 88);
    visuals.widgets.inactive.bg_fill = CARD;
    visuals.widgets.inactive.weak_bg_fill = CARD;
    visuals.widgets.hovered.bg_fill = Color32::from_rgb(31, 49, 63);
    ctx.set_visuals(visuals);
    ctx.global_style_mut(|s| {
        s.spacing.item_spacing = Vec2::new(12.0, 10.0);
        s.animation_time = 0.0;
    });
}

pub struct App {
    engine: Engine,
    selected: Option<usize>,
    tab: Tab,
    filter: String,
    sort_memory: bool,
    started: Instant,
    screenshot: Option<PathBuf>,
    exit_after: Option<f32>,
}
impl App {
    pub fn new(
        engine: Engine,
        selected: Option<usize>,
        tab: Tab,
        screenshot: Option<PathBuf>,
        exit_after: Option<f32>,
    ) -> Self {
        Self {
            engine,
            selected,
            tab,
            filter: String::new(),
            sort_memory: false,
            started: Instant::now(),
            screenshot,
            exit_after,
        }
    }
}

impl eframe::App for App {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        ui.ctx().request_repaint_after(Duration::from_secs(1));
        let elapsed = self.started.elapsed().as_secs_f32();
        if elapsed >= 8.0
            && let Some(path) = self.screenshot.take()
        {
            ui.ctx()
                .send_viewport_cmd(egui::ViewportCommand::Screenshot(
                    egui::ScreenshotCallback::new(move |img| {
                        let bytes: Vec<_> = img.pixels.iter().flat_map(|c| c.to_array()).collect();
                        if let Err(e) = image::save_buffer(
                            &path,
                            &bytes,
                            img.width() as u32,
                            img.height() as u32,
                            image::ColorType::Rgba8,
                        ) {
                            eprintln!("Screenshot failed: {e}");
                        }
                    }),
                ));
        }
        if self.exit_after.is_some_and(|s| elapsed >= s) {
            ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
        }
        let (nodes, storage_error) = {
            let shared = self.engine.state.lock().unwrap();
            (shared.nodes.clone(), shared.storage_error.clone())
        };
        egui::Frame::new().fill(BG).inner_margin(20).show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new("moonitor").size(32.0).strong());
                ui.add_space(12.0);
                ui.label(
                    RichText::new("HOMELAB / TELEMETRIA LOCALE")
                        .size(11.0)
                        .color(MUTED)
                        .monospace(),
                );
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(
                        RichText::new(format!(
                            "{} nodi attivi",
                            nodes.iter().filter(|n| n.live()).count()
                        ))
                        .color(GREEN),
                    );
                });
            });
            ui.add_space(12.0);
            ui.separator();
            ui.add_space(12.0);
            egui::Panel::left("fleet")
                .exact_size(190.0)
                .resizable(false)
                .frame(egui::Frame::new().fill(BG).inner_margin(egui::Margin {
                    right: 18,
                    ..Default::default()
                }))
                .show(ui, |ui| {
                    ui.label(
                        RichText::new("DISPOSITIVI")
                            .color(MUTED)
                            .size(11.0)
                            .monospace(),
                    );
                    if ui
                        .add_sized(
                            [170.0, 38.0],
                            egui::Button::new("Tutti i nodi").selected(self.selected.is_none()),
                        )
                        .clicked()
                    {
                        self.selected = None;
                    }
                    ui.add_space(8.0);
                    for (index, node) in nodes.iter().enumerate() {
                        let (status, color) = status(node);
                        egui::Frame::new()
                            .fill(if self.selected == Some(index) {
                                Color32::from_rgb(28, 49, 60)
                            } else {
                                CARD
                            })
                            .inner_margin(12)
                            .corner_radius(8)
                            .show(ui, |ui| {
                                ui.set_min_width(146.0);
                                if ui
                                    .add(
                                        egui::Button::new(
                                            RichText::new(&node.config.label).strong(),
                                        )
                                        .frame(false),
                                    )
                                    .clicked()
                                {
                                    self.selected = Some(index);
                                }
                                ui.label(RichText::new(status).size(11.0).color(color));
                                if let Some(s) = &node.sample {
                                    ui.label(
                                        RichText::new(format!(
                                            "CPU {} / RAM {}",
                                            pct(s.cpu_utilization_percent.value().copied()),
                                            pct(memory_percent(s))
                                        ))
                                        .size(10.5)
                                        .color(MUTED),
                                    );
                                }
                            });
                    }
                    ui.add_space(14.0);
                    ui.label(
                        RichText::new("1 s live\n5 s storico / 7 giorni\nSSH persistente")
                            .size(11.0)
                            .color(MUTED),
                    );
                });
            egui::CentralPanel::default()
                .frame(egui::Frame::new().fill(BG).inner_margin(egui::Margin {
                    left: 14,
                    ..Default::default()
                }))
                .show(ui, |ui| {
                    if let Some(e) = storage_error {
                        ui.colored_label(AMBER, format!("Storico non disponibile: {e}"));
                    }
                    egui::ScrollArea::vertical().id_salt("main").show(ui, |ui| {
                        if let Some(index) = self.selected {
                            self.device(ui, &nodes[index]);
                        } else {
                            self.fleet(ui, &nodes);
                        }
                    });
                });
        });
    }
}

impl App {
    fn fleet(&mut self, ui: &mut egui::Ui, nodes: &[NodeState]) {
        ui.heading("Il tuo homelab");
        ui.label(
            RichText::new("Risorse, GPU e carichi AI dei dispositivi in LAN / VPN").color(MUTED),
        );
        ui.add_space(16.0);
        let columns = if ui.available_width() > 760.0 { 2 } else { 1 };
        for chunk in nodes.chunks(columns) {
            ui.columns(columns, |cols| {
                for (col, node) in cols.iter_mut().zip(chunk) {
                    card(col, |ui| {
                        let (label, color) = status(node);
                        ui.horizontal(|ui| {
                            ui.label(RichText::new(&node.config.label).size(20.0).strong());
                            ui.colored_label(color, label);
                        });
                        if let Some(s) = &node.sample {
                            ui.label(RichText::new(&s.cpu_name).size(11.0).color(MUTED));
                            meter(ui, "CPU", s.cpu_utilization_percent.value().copied(), CYAN);
                            meter(ui, "RAM", memory_percent(s), GREEN);
                            meter(
                                ui,
                                "GPU",
                                s.gpus
                                    .iter()
                                    .filter_map(|g| g.utilization_percent.value().copied())
                                    .reduce(f64::max),
                                AMBER,
                            );
                            ui.label(
                                RichText::new(format!(
                                    "{}  /  {} core  /  {} processi",
                                    bytes(s.memory.total_bytes.value().copied()),
                                    s.cpu_cores_percent.len(),
                                    s.process_count
                                ))
                                .size(11.0)
                                .color(MUTED),
                            );
                        } else {
                            ui.add_space(14.0);
                            ui.label(
                                RichText::new(if node.config.note.is_empty() {
                                    "In attesa di telemetria"
                                } else {
                                    &node.config.note
                                })
                                .color(MUTED),
                            );
                            if let Some(e) = &node.error {
                                ui.label(RichText::new(e).color(AMBER).size(11.0));
                            }
                            ui.add_space(14.0);
                        }
                        if ui.button("Apri dispositivo").clicked() {
                            self.selected =
                                nodes.iter().position(|n| n.config.id == node.config.id);
                        }
                    });
                }
            });
            ui.add_space(8.0);
        }
        ui.add_space(12.0);
        ui.label(RichText::new("N/D indica una misura non disponibile. I campioni interrotti restano visibili con lo stato obsoleto.").color(MUTED).size(11.0));
    }

    fn device(&mut self, ui: &mut egui::Ui, node: &NodeState) {
        ui.horizontal(|ui| {
            ui.heading(&node.config.label);
            let (s, c) = status(node);
            ui.colored_label(c, s);
        });
        let Some(s) = &node.sample else {
            ui.add_space(20.0);
            ui.label(&node.config.note);
            ui.label(
                node.error.as_deref().unwrap_or(
                    "In attesa dei primi campioni. La raccolta riprova automaticamente.",
                ),
            );
            return;
        };
        ui.label(
            RichText::new(format!("{} / {} / {}", s.os, s.arch, s.cpu_name))
                .color(MUTED)
                .size(12.0),
        );
        if !node.live() {
            ui.colored_label(
                AMBER,
                format!(
                    "Ultimo campione ricevuto {:.0} s fa. {}",
                    node.last_seen
                        .map(|t| t.elapsed().as_secs_f64())
                        .unwrap_or_default(),
                    node.error.as_deref().unwrap_or("Dati obsoleti")
                ),
            );
        }
        ui.add_space(14.0);
        ui.columns(4, |c| {
            stat(
                &mut c[0],
                "CPU",
                &pct(s.cpu_utilization_percent.value().copied()),
                &format!("{} core logici", s.cpu_cores_percent.len()),
                CYAN,
            );
            let total = s.memory.total_bytes.value().copied();
            let used = total
                .zip(s.memory.available_bytes.value().copied())
                .map(|(t, a)| t.saturating_sub(a));
            stat(
                &mut c[1],
                "MEMORIA",
                &pct(memory_percent(s)),
                &format!("{} / {}", bytes(used), bytes(total)),
                GREEN,
            );
            let gpu = s
                .gpus
                .iter()
                .filter_map(|g| g.utilization_percent.value().copied())
                .reduce(f64::max);
            stat(
                &mut c[2],
                "GPU",
                &pct(gpu),
                s.gpus
                    .first()
                    .map(|g| g.name.as_str())
                    .unwrap_or("Sensore non disponibile"),
                AMBER,
            );
            let power = s
                .power
                .iter()
                .find(|p| p.scope == PowerScope::GpuBoard)
                .and_then(|p| p.watts.value())
                .map(|p| format!("{p:.1} W"))
                .unwrap_or_else(|| "N/D".into());
            stat(
                &mut c[3],
                "POTENZA GPU",
                &power,
                "Intero dispositivo: N/D",
                CYAN,
            );
        });
        ui.add_space(12.0);
        ui.horizontal_wrapped(|ui| {
            for (tab, label) in [
                (Tab::Overview, "Panoramica"),
                (Tab::Processes, "Processi"),
                (Tab::Ai, "AI locale"),
                (Tab::Disks, "Dischi"),
                (Tab::Network, "Rete"),
                (Tab::History, "Storico"),
            ] {
                if ui.selectable_label(self.tab == tab, label).clicked() {
                    self.tab = tab;
                }
            }
        });
        ui.separator();
        ui.add_space(10.0);
        match self.tab {
            Tab::Overview => {
                ui.columns(2, |cols| {
                    chart(
                        &mut cols[0],
                        "CPU / ultimi 5 minuti",
                        &node.history,
                        |p| p.cpu_percent,
                        CYAN,
                    );
                    chart(
                        &mut cols[1],
                        "GPU / ultimi 5 minuti",
                        &node.history,
                        |p| p.gpu_percent,
                        AMBER,
                    );
                });
                ui.add_space(12.0);
                card(ui, |ui| {
                    ui.label(
                        RichText::new("CORE LOGICI")
                            .color(MUTED)
                            .monospace()
                            .size(11.0),
                    );
                    egui::Grid::new("cores")
                        .num_columns(4)
                        .spacing([14.0, 8.0])
                        .show(ui, |ui| {
                            for (i, value) in s.cpu_cores_percent.iter().enumerate() {
                                ui.horizontal(|ui| {
                                    ui.label(
                                        RichText::new(format!("{:02}", i)).monospace().color(MUTED),
                                    );
                                    ui.add(
                                        egui::ProgressBar::new(*value / 100.0)
                                            .desired_width(135.0)
                                            .fill(CYAN)
                                            .text(format!("{value:.0}%")),
                                    );
                                });
                                if i % 4 == 3 {
                                    ui.end_row();
                                }
                            }
                        });
                });
                ui.add_space(10.0);
                card(ui, |ui| {
                    ui.label(format!(
                        "Swap: {}   /   Uptime: {:.1} ore   /   Raccolta: {:.2} ms",
                        bytes(s.memory.swap_used_bytes.value().copied()),
                        s.uptime_seconds as f64 / 3600.0,
                        s.collector_duration_ms
                    ));
                    if let Some(b) = s.battery_percent.value() {
                        ui.label(format!("Batteria: {b:.0}%"));
                    }
                    for p in &s.pressure {
                        ui.label(format!(
                            "Pressione {} (stall avg10): {:.2}%",
                            p.resource, p.some_avg10_percent
                        ));
                    }
                    for t in s.temperatures.iter().take(8) {
                        ui.label(format!("{}: {:.1} °C", t.label, t.celsius));
                    }
                    if s.temperatures.is_empty() {
                        ui.label(RichText::new("Temperature di sistema: N/D").color(MUTED));
                    }
                });
            }
            Tab::Processes => self.processes(ui, s, false),
            Tab::Ai => {
                for gpu in &s.gpus {
                    card(ui, |ui| {
                        ui.label(RichText::new(&gpu.name).size(17.0).strong());
                        ui.label(format!(
                            "Memoria GPU: {} / {} ({:?})",
                            bytes(gpu.memory_used_bytes.value().copied()),
                            bytes(gpu.memory_total_bytes.value().copied()),
                            gpu.memory_kind
                        ));
                        if gpu.memory_kind == GpuMemoryKind::Unified {
                            ui.label(
                                RichText::new(
                                    "Memoria condivisa con la CPU: i totali non si sommano.",
                                )
                                .color(MUTED),
                            );
                        }
                        ui.label(format!(
                            "Temperatura: {}   /   Clock: {}",
                            gpu.temperature_celsius
                                .value()
                                .map(|n| format!("{n:.0} °C"))
                                .unwrap_or_else(|| "N/D".into()),
                            gpu.clock_mhz
                                .value()
                                .map(|n| format!("{n} MHz"))
                                .unwrap_or_else(|| "N/D".into())
                        ));
                        ui.label(format!(
                            "Limitazioni GPU: {}",
                            gpu.throttling.value().map(String::as_str).unwrap_or("N/D")
                        ));
                    });
                }
                ui.add_space(10.0);
                ui.heading("Modelli caricati / Ollama");
                match s.ollama.models.value() {
                    Some(models) if models.is_empty() => {
                        ui.colored_label(GREEN, "Ollama disponibile / nessun modello caricato");
                    }
                    Some(models) => {
                        for model in models {
                            ui.label(format!(
                                "{}  /  {}  /  contesto {}  /  {}",
                                model.name,
                                bytes(Some(model.size_bytes)),
                                model
                                    .context_length
                                    .map(|n| n.to_string())
                                    .unwrap_or_else(|| "N/D".into()),
                                model.quantization.as_deref().unwrap_or("N/D")
                            ));
                        }
                    }
                    None => {
                        ui.label(
                            RichText::new("Ollama non raggiungibile su questo nodo.").color(MUTED),
                        );
                    }
                }
                ui.label(RichText::new("Token/s, latenza e code: N/D senza telemetria delle richieste dal runtime.").color(MUTED).size(12.0));
                ui.add_space(12.0);
                ui.heading("Processi AI riconosciuti dal nome");
                self.processes(ui, s, true);
            }
            Tab::Disks => {
                for d in &s.disks {
                    card(ui, |ui| {
                        ui.label(RichText::new(&d.mount).strong());
                        ui.label(RichText::new(&d.name).color(MUTED).size(11.0));
                        let percent = (d.total_bytes > 0).then(|| {
                            d.total_bytes.saturating_sub(d.available_bytes) as f64
                                / d.total_bytes as f64
                                * 100.0
                        });
                        meter(ui, "Uso", percent, CYAN);
                        ui.label(format!(
                            "{} disponibili / {} totali",
                            bytes(Some(d.available_bytes)),
                            bytes(Some(d.total_bytes))
                        ));
                    });
                    ui.add_space(8.0);
                }
                ui.label(RichText::new("Capacità aggiornata ogni 5 s. Volumi APFS possono condividere lo stesso spazio.").color(MUTED));
            }
            Tab::Network => {
                egui::Grid::new("networks")
                    .striped(true)
                    .spacing([26.0, 12.0])
                    .show(ui, |ui| {
                        for label in ["INTERFACCIA", "RICEZIONE", "INVIO", "ERRORI TOTALI"] {
                            ui.label(RichText::new(label).color(MUTED).size(11.0));
                        }
                        ui.end_row();
                        for n in &s.networks {
                            ui.monospace(&n.name);
                            ui.monospace(format!(
                                "{}/s",
                                bytes(Some(n.received_bytes_per_second as u64))
                            ));
                            ui.monospace(format!(
                                "{}/s",
                                bytes(Some(n.transmitted_bytes_per_second as u64))
                            ));
                            ui.monospace(n.errors.to_string());
                            ui.end_row();
                        }
                    });
                ui.add_space(16.0);
                ui.label(RichText::new("Interfacce fisiche, VPN e virtuali possono riportare lo stesso traffico: non vengono sommate.").color(MUTED));
            }
            Tab::History => {
                chart(
                    ui,
                    "CPU / ultimi 5 minuti",
                    &node.history,
                    |p| p.cpu_percent,
                    CYAN,
                );
                chart(
                    ui,
                    "Memoria / ultimi 5 minuti",
                    &node.history,
                    |p| p.memory_percent,
                    GREEN,
                );
                chart(
                    ui,
                    "GPU / ultimi 5 minuti",
                    &node.history,
                    |p| p.gpu_percent,
                    AMBER,
                );
                ui.label(RichText::new("Storico SQLite: riepilogo ogni 5 s, conservazione 7 giorni. Vista attuale: ultimi 5 minuti.").color(MUTED));
            }
        }
    }

    fn processes(&mut self, ui: &mut egui::Ui, s: &HostSnapshot, only_ai: bool) {
        ui.horizontal(|ui| {
            ui.label("Filtro");
            ui.text_edit_singleline(&mut self.filter);
            ui.selectable_value(&mut self.sort_memory, false, "CPU");
            ui.selectable_value(&mut self.sort_memory, true, "RAM");
        });
        ui.label(RichText::new(format!("{} processi sul nodo / sottoinsieme dei più attivi e candidati AI. CPU: 100% = un core.", s.process_count)).color(MUTED).size(11.0));
        let filter = self.filter.to_lowercase();
        let mut processes: Vec<_> = s
            .processes
            .iter()
            .filter(|p| (!only_ai || p.is_ai) && p.name.to_lowercase().contains(&filter))
            .collect();
        if self.sort_memory {
            processes.sort_by_key(|p| std::cmp::Reverse(p.memory_bytes));
        }
        egui::Grid::new(if only_ai { "ai-procs" } else { "processes" })
            .striped(true)
            .spacing([18.0, 8.0])
            .show(ui, |ui| {
                for label in ["PID", "PROCESSO", "CPU", "RAM"] {
                    ui.label(RichText::new(label).color(MUTED).size(11.0));
                }
                ui.end_row();
                for p in processes {
                    ui.monospace(p.pid.to_string());
                    ui.label(if p.name.len() > 70 {
                        p.name.chars().take(65).collect::<String>()
                    } else {
                        p.name.clone()
                    });
                    ui.monospace(format!("{:.1}%", p.cpu_percent));
                    ui.monospace(bytes(Some(p.memory_bytes)));
                    ui.end_row();
                }
            });
    }
}

fn status(n: &NodeState) -> (&'static str, Color32) {
    if matches!(n.config.kind, NodeKind::Unavailable) {
        ("CENSITO / N/D", MUTED)
    } else if n.live() {
        ("LIVE", GREEN)
    } else if n.sample.is_some() {
        ("OBSOLETO", AMBER)
    } else if n.error.is_some() {
        ("NON RAGGIUNGIBILE", AMBER)
    } else {
        ("CONNESSIONE", MUTED)
    }
}

fn card(ui: &mut egui::Ui, body: impl FnOnce(&mut egui::Ui)) {
    egui::Frame::new()
        .fill(CARD)
        .stroke(Stroke::new(1.0, BORDER))
        .corner_radius(10)
        .inner_margin(16)
        .show(ui, |ui| {
            ui.set_min_width((ui.available_width() - 1.0).max(0.0));
            body(ui);
        });
}
fn stat(ui: &mut egui::Ui, title: &str, value: &str, foot: &str, color: Color32) {
    card(ui, |ui| {
        ui.label(RichText::new(title).color(MUTED).monospace().size(10.0));
        ui.label(RichText::new(value).size(28.0).color(color).strong());
        ui.label(RichText::new(foot).color(MUTED).size(10.5));
    });
}
fn pct(value: Option<f64>) -> String {
    value
        .map(|v| format!("{v:.0}%"))
        .unwrap_or_else(|| "N/D".into())
}
fn bytes(value: Option<u64>) -> String {
    let Some(v) = value else {
        return "N/D".into();
    };
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
fn meter(ui: &mut egui::Ui, label: &str, value: Option<f64>, color: Color32) {
    ui.horizontal(|ui| {
        ui.label(RichText::new(label).size(11.0).color(MUTED));
        if let Some(value) = value {
            ui.add(
                egui::ProgressBar::new(value as f32 / 100.0)
                    .desired_width((ui.available_width() - 12.0).max(40.0))
                    .fill(color)
                    .text(pct(Some(value))),
            );
        } else {
            ui.label(RichText::new("N/D").color(MUTED));
        }
    });
}
fn chart(
    ui: &mut egui::Ui,
    label: &str,
    points: &std::collections::VecDeque<HistoryPoint>,
    value: impl Fn(&HistoryPoint) -> Option<f64>,
    color: Color32,
) {
    card(ui, |ui| {
        ui.label(RichText::new(label).color(MUTED).size(11.0));
        let (response, painter) =
            ui.allocate_painter(Vec2::new(ui.available_width(), 100.0), egui::Sense::hover());
        let rect = response.rect.shrink(4.0);
        for fraction in [0.0, 0.5, 1.0] {
            let y = rect.bottom() - rect.height() * fraction;
            painter.line_segment(
                [egui::pos2(rect.left(), y), egui::pos2(rect.right(), y)],
                Stroke::new(1.0, BORDER),
            );
        }
        let now = now_ms();
        let since = now.saturating_sub(300_000);
        let mut previous = None;
        for point in points.iter().filter(|p| p.received_at_ms >= since) {
            if let Some(v) = value(point) {
                let pos = egui::pos2(
                    rect.left()
                        + rect.width()
                            * ((point.received_at_ms.saturating_sub(since)) as f32 / 300_000.0)
                                .clamp(0.0, 1.0),
                    rect.bottom() - rect.height() * (v as f32 / 100.0).clamp(0.0, 1.0),
                );
                if let Some((old_ts, old_pos)) = previous
                    && point.received_at_ms.saturating_sub(old_ts) < 15_000
                {
                    painter.line_segment([old_pos, pos], Stroke::new(1.8, color));
                }
                previous = Some((point.received_at_ms, pos));
            } else {
                previous = None;
            }
        }
        if points.iter().all(|p| value(p).is_none()) {
            painter.text(
                rect.center(),
                egui::Align2::CENTER_CENTER,
                "Nessuna misura disponibile",
                egui::FontId::proportional(12.0),
                MUTED,
            );
        }
    });
}
