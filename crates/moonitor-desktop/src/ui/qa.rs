use super::*;
use crate::scene::Scene;
use iced_test::Simulator;
fn app(count: usize, tab: Tab) -> App {
    App {
        _engine: None,
        nodes: crate::fixtures::fleet(count),
        storage_error: None,
        dropped_events: 0,
        pending_samples: Vec::new(),
        selected: (count > 0).then(|| "qa-0".into()),
        dark: false,
        tab,
        section: if tab == Tab::Overview {
            Section::Overview
        } else {
            Section::Device
        },
        filter: String::new(),
        sort_memory: false,
        camera: Camera::default(),
        started: Instant::now(),
        view_time_ms: crate::fixtures::TIME_MS,
        screenshot: None,
        exit_after: None,
        window: None,
        profile: None,
        preferences: Default::default(),
        data_dir: std::env::temp_dir(),
        form: None,
        setup_error: None,
        setup_notice: None,
        remove_host: None,
        applying: false,
        fleet_filter: String::new(),
        fleet_sort: instrument::FleetSort::Name,
        fleet_scroll: 0.0,
        viewport: iced::Size::new(1280.0, 800.0),
        paused: false,
        frozen: None,
        dialog_confirm_focused: false,
    }
}
fn settings() -> iced::Settings {
    iced::Settings {
        default_font: FONT,
        fonts: vec![
            include_bytes!("../../assets/Inter-Regular.ttf")
                .as_slice()
                .into(),
            include_bytes!("../../assets/Inter-Medium.ttf")
                .as_slice()
                .into(),
            include_bytes!("../../assets/Inter-SemiBold.ttf")
                .as_slice()
                .into(),
        ],
        ..Default::default()
    }
}
fn theme() -> Theme {
    app_theme(false)
}
#[test]
fn selection_messages_do_not_follow_indices() {
    let mut app = app(5, Tab::Overview);
    let _ = app.update(Message::Select("qa-3".into()));
    app.nodes.reverse();
    let _ = app.update(Message::Tick);
    assert_eq!(app.selected.as_deref(), Some("qa-3"));
    app.nodes.truncate(1);
    let _ = app.update(Message::Tick);
    assert_eq!(app.selected.as_deref(), Some("qa-4"));
    let _ = app.update(Message::Select("missing".into()));
    assert_eq!(app.selected.as_deref(), Some("qa-4"));
    app.nodes.clear();
    let _ = app.update(Message::Tick);
    assert!(app.selected.is_none());
}
#[test]
fn switching_context_keeps_the_page_and_absent_data_is_not_stale() {
    let mut app = app(5, Tab::Overview);
    let _ = app.update(Message::SwitchNode("qa-4".into()));
    assert_eq!(app.section, Section::Overview);
    assert_eq!(app.selected.as_deref(), Some("qa-4"));
    assert!(!Metric::percent("CPU", None, true).stale);
    assert!(Metric::memory(&crate::fixtures::sample("qa-0"), true).stale);
}

#[test]
fn filtered_selection_never_opens_a_hidden_host_and_keys_follow_visible_order() {
    let mut a = app(5, Tab::Overview);
    let _ = a.update(Message::FleetFilter("QA device 4".into()));
    assert!(a.visible_selection().is_none());
    let _ = a.update(Message::OpenVisibleHost);
    let _ = a.update(Message::Select("qa-0".into())); // old/stale command
    assert_eq!(a.section, Section::Overview);
    let _ = a.update(Message::NextNode(1));
    assert_eq!(a.selected.as_deref(), Some("qa-4"));
    let _ = a.update(Message::OpenVisibleHost);
    assert_eq!(a.section, Section::Device);
    let _ = a.update(Message::Navigate(Section::Overview));
    assert_eq!(a.fleet_filter, "QA device 4");
    assert_eq!(a.visible_selection().as_deref(), Some("qa-4"));
}

#[test]
fn frozen_observation_clock_does_not_hide_current_transport_health() {
    let mut a = app(1, Tab::Overview);
    let _ = a.update(Message::Pause);
    // Simulate 61 seconds passing without a real sleep; frozen sample was fresh
    // at the instant of capture, live samples and transport evolve independently.
    let old = Instant::now() - Duration::from_secs(61);
    a.frozen.as_mut().unwrap().reference = old;
    a.frozen.as_mut().unwrap().nodes[0].last_seen = Some(old);
    let frozen_time = a.view_time_ms;
    let sample = std::sync::Arc::make_mut(a.nodes[0].sample.as_mut().unwrap());
    sample.cpu_utilization_percent = Reading::reported(53.0, "fixture", now_ms());
    a.nodes[0].last_seen = Some(Instant::now());
    let _ = a.update(Message::Tick);
    assert_eq!(
        a.observed_nodes()[0]
            .sample
            .as_ref()
            .unwrap()
            .cpu_utilization_percent
            .value(),
        Some(&0.0)
    );
    assert_eq!(a.health_status(&a.observed_nodes()[0]).0, "Online");
    assert!(!a.observation_stale(&a.observed_nodes()[0]));
    assert_eq!(a.view_time_ms, frozen_time);
    a.nodes[0].error = Some("fixture transport loss".into());
    assert_eq!(a.health_status(&a.observed_nodes()[0]).0, "Obsoleto");
    let _ = a.update(Message::Pause);
    assert!(a.frozen.is_none());
    assert_eq!(
        a.observed_nodes()[0]
            .sample
            .as_ref()
            .unwrap()
            .cpu_utilization_percent
            .value(),
        Some(&53.0)
    );
}

#[test]
fn thermal_quarantine_preserves_the_rest_of_the_host() {
    let mut s = crate::fixtures::sample("qa-0");
    s.temperatures.push(TemperatureSnapshot {
        label: "PMU tdev7".into(),
        celsius: -9201.1,
    });
    assert!(s.validate("qa-0").is_ok());
    assert!(
        s.temperatures[0]
            .observation(s.sampled_at_unix_ms)
            .value_celsius
            .is_none()
    );
    assert_eq!(s.cpu_utilization_percent.value(), Some(&0.0));
    assert_eq!(s.temperatures[0].celsius, -9201.1);
}

#[test]
fn removal_keyboard_defaults_to_cancel_and_escape_never_mutates_fleet() {
    use iced::keyboard::{Key, key::Named};
    let mut a = app(5, Tab::Overview);
    for key in [Named::Escape, Named::Enter] {
        let _ = a.update(Message::RequestRemove("qa-0".into()));
        let _ = a.update(Message::DialogKey(Key::Named(key)));
        assert!(a.remove_host.is_none());
        assert_eq!(a.nodes.len(), 5);
    }
    let _ = a.update(Message::RequestRemove("qa-0".into()));
    let _ = a.update(Message::DialogKey(Key::Named(Named::Tab)));
    assert!(a.dialog_confirm_focused);
    let _ = a.update(Message::DialogKey(Key::Named(Named::Escape)));
    assert_eq!(a.nodes.len(), 5);
    let _ = a.update(Message::RemoveHost("qa-0".into()));
    assert_eq!(a.nodes.len(), 5); // no pending scoped confirmation
}

#[test]
#[ignore = "Requires a graphical backend. Run scripts/qa-ui.sh."]
fn trust_regressions_have_visible_feedback_and_scoped_actions() {
    let root = std::path::PathBuf::from(std::env::var("MOONITOR_QA_DIR").unwrap());
    for (w, h) in [(1280.0, 800.0), (1440.0, 900.0)] {
        for dark in [false, true] {
            let mut a = app(5, Tab::Overview);
            a.dark = dark;
            let _ = a.update(Message::FleetFilter("QA device 4".into()));
            let mut ui = Simulator::with_size(settings(), (w, h), a.view());
            ui.click("Seleziona una riga").unwrap();
            assert!(!ui.into_messages().any(|m| matches!(m, Message::Select(_))));
            a.section = Section::Settings;
            let _ = a.update(Message::RequestRemove("qa-0".into()));
            let mut ui = Simulator::with_size(settings(), (w, h), a.view());
            for label in ["Rimuovere QA device 0?", "Annulla", "Rimuovi host"] {
                assert!(ui.find(label).unwrap().visible_bounds().is_some());
            }
            assert!(
                ui.snapshot(&app_theme(dark))
                    .unwrap()
                    .matches_image(root.join(format!("s0-dialog-{w}-{dark}")))
                    .unwrap()
            );
            ui.click("Annulla").unwrap();
            for m in ui.into_messages() {
                let _ = a.update(m);
            }
            assert!(a.remove_host.is_none());
            a.section = Section::Device;
            a.tab = Tab::Sensors;
            let s = std::sync::Arc::make_mut(a.nodes[0].sample.as_mut().unwrap());
            s.temperatures.push(TemperatureSnapshot {
                label: "PMU tdev7".into(),
                celsius: -9201.1,
            });
            let mut ui = Simulator::with_size(settings(), (w, h), a.view());
            assert!(
                ui.find("Lettura non valida")
                    .unwrap()
                    .visible_bounds()
                    .is_some()
            );
            assert!(
                ui.snapshot(&app_theme(dark))
                    .unwrap()
                    .matches_image(root.join(format!("s0-thermal-{w}-{dark}")))
                    .unwrap()
            );
        }
    }
}

#[test]
fn form_crud_keeps_identity_preferences_and_history_file_on_restart() {
    use setup::Field;
    let root =
        std::env::temp_dir().join(format!("moonitor-crud-{}-{}", std::process::id(), now_ms()));
    std::fs::create_dir_all(&root).unwrap();
    std::fs::write(root.join("history.sqlite"), b"preserved-history-marker").unwrap();
    let mut app = super::qa::app(0, Tab::Overview);
    app.profile = Some(crate::profile::Profile::open(&root.join("settings.sqlite"), None).unwrap());
    app.nodes = app
        .profile
        .as_ref()
        .unwrap()
        .config
        .nodes
        .iter()
        .map(|c| NodeState {
            config: c.clone(),
            capacity_limited: false,
            sample: None,
            last_seen: None,
            error: None,
            history: Default::default(),
        })
        .collect();
    let _ = app.update(Message::NewHost);
    let id = app.form.as_ref().unwrap().id.clone();
    let _ = app.update(Message::HostField(Field::Label, "Compute example".into()));
    let _ = app.update(Message::HostField(
        Field::Destination,
        "node.example".into(),
    ));
    let _ = app.update(Message::HostField(Field::User, "observer".into()));
    let _ = app.update(Message::HostField(Field::Port, "2222".into()));
    let _ = app.update(Message::SaveHost);
    assert!(app.setup_error.is_none());
    assert_eq!(app.nodes.len(), 2);
    let _ = app.update(Message::EditHost(id.clone()));
    let _ = app.update(Message::HostField(Field::Label, "Renamed compute".into()));
    let _ = app.update(Message::SaveHost);
    let _ = app.update(Message::Theme(true));
    let _ = app.update(Message::Density);
    drop(app);
    let profile = crate::profile::Profile::open(&root.join("settings.sqlite"), None).unwrap();
    assert_eq!(profile.config.nodes[1].id, id);
    assert_eq!(profile.config.nodes[1].label, "Renamed compute");
    assert_eq!(profile.config.nodes[1].ssh_port, Some(2222));
    assert!(profile.preferences.dark && profile.preferences.comfortable);
    let mut app = super::qa::app(0, Tab::Overview);
    app.nodes = profile
        .config
        .nodes
        .iter()
        .map(|c| NodeState {
            config: c.clone(),
            capacity_limited: false,
            sample: None,
            last_seen: None,
            error: None,
            history: Default::default(),
        })
        .collect();
    app.profile = Some(profile);
    let _ = app.update(Message::RemoveHost(id.clone())); // Unconfirmed removals are ignored.
    assert_eq!(app.nodes.len(), 2);
    let _ = app.update(Message::RequestRemove(id.clone()));
    let _ = app.update(Message::RemoveHost(id));
    assert_eq!(app.nodes.len(), 1);
    drop(app);
    assert_eq!(
        crate::profile::Profile::open(&root.join("settings.sqlite"), None)
            .unwrap()
            .config
            .nodes
            .len(),
        1
    );
    assert_eq!(
        std::fs::read(root.join("history.sqlite")).unwrap(),
        b"preserved-history-marker"
    );
    let _ = std::fs::remove_dir_all(root);
}

#[test]
#[ignore = "Requires a graphical backend. Run scripts/qa-ui.sh."]
fn fleet_preview_sort_filter_open_and_empty_host() {
    let mut app = app(5, Tab::Overview);
    let mut ui = Simulator::with_size(settings(), (1280.0, 800.0), app.view());
    ui.click("QA device 4").unwrap();
    for m in ui.into_messages() {
        let _ = app.update(m);
    }
    assert_eq!(app.selected.as_deref(), Some("qa-4"));
    assert_eq!(app.section, Section::Overview);
    let mut ui = Simulator::with_size(settings(), (1280.0, 800.0), app.view());
    ui.click("Apri host →").unwrap();
    for m in ui.into_messages() {
        let _ = app.update(m);
    }
    assert_eq!(app.section, Section::Device);
    let mut ui = Simulator::with_size(settings(), (1280.0, 800.0), app.view());
    ui.click("Apri Setup").unwrap();
    for m in ui.into_messages() {
        let _ = app.update(m);
    }
    assert_eq!(app.section, Section::Settings);
    let _ = app.update(Message::Navigate(Section::Overview));
    let _ = app.update(Message::FleetFilter("online".into()));
    assert_eq!(app.filtered_nodes().len(), 1);
    let _ = app.update(Message::FleetFilter(String::new()));
    let _ = app.update(Message::FleetSort(instrument::FleetSort::State));
    assert_eq!(app.selected.as_deref(), Some("qa-4"));
    let _ = app.update(Message::NextNode(-1));
    // State sorting places unreachable qa-2 immediately before inventory-only qa-4.
    // Keyboard traversal follows that visible order, rather than config indices.
    assert_eq!(app.selected.as_deref(), Some("qa-2"));
}

#[test]
#[ignore = "Requires a graphical backend. Run scripts/qa-ui.sh."]
fn setup_form_keyboard_and_save() {
    let mut app = app(0, Tab::Overview);
    let _ = app.update(Message::NewHost);
    app.form.as_mut().unwrap().id = "qa-editor".into();
    let id = app.form.as_ref().unwrap().id.clone();
    let mut ui = Simulator::with_size(settings(), (1280.0, 800.0), app.view());
    ui.click(widget::Id::new("host-Label")).unwrap();
    ui.typewrite("Remote fixture");
    ui.click(widget::Id::new("host-Destination")).unwrap();
    ui.typewrite("node.example");
    for message in ui.into_messages() {
        let _ = app.update(message);
    }
    assert_eq!(app.form.as_ref().unwrap().label, "Remote fixture");
    let mut ui = Simulator::with_size(settings(), (1280.0, 800.0), app.view());
    let dir = std::path::PathBuf::from(std::env::var("MOONITOR_QA_DIR").unwrap());
    assert!(
        ui.snapshot(&theme())
            .unwrap()
            .matches_image(dir.join("fixture-setup-editor"))
            .unwrap()
    );
    ui.click("Salva host").unwrap();
    for message in ui.into_messages() {
        let _ = app.update(message);
    }
    assert!(app.setup_error.is_none(), "{:?}", app.setup_error);
    assert_eq!(app.nodes.len(), 1);
    assert_eq!(app.nodes[0].config.id, id);
    assert_eq!(
        app.nodes[0].config.ssh_alias.as_deref(),
        Some("node.example")
    );
}

#[test]
#[ignore = "Requires a graphical backend. Run scripts/qa-ui.sh."]
fn virtualized_scroll_and_filter_keep_the_real_widget_state_in_sync() {
    use iced_test::futures::futures::{StreamExt, executor::block_on};
    use iced_test::{
        Selector,
        core::{
            renderer::Headless,
            widget::{Operation, operation},
        },
        runtime::{Action, UserInterface, user_interface},
    };
    for font in settings().fonts {
        iced_test::renderer::graphics::text::font_system()
            .write()
            .unwrap()
            .load_font(font);
    }
    let mut renderer =
        block_on(<iced::Renderer as Headless>::new(FONT, 16.0.into(), None)).unwrap();
    let mut app = app(500, Tab::Overview);
    let size = iced::Size::new(1280.0, 800.0);
    let mut ui = UserInterface::build(
        app.view(),
        size,
        user_interface::Cache::default(),
        &mut renderer,
    );
    let mut messages = Vec::new();
    let cursor = iced::mouse::Cursor::Available(iced::Point::new(700.0, 450.0));
    let _ = ui.update(
        &[iced::Event::Mouse(iced::mouse::Event::WheelScrolled {
            delta: iced::mouse::ScrollDelta::Lines {
                x: 0.0,
                y: -10000.0,
            },
        })],
        cursor,
        &mut renderer,
        &mut iced_test::core::clipboard::Null,
        &mut messages,
    );
    let cache = ui.into_cache();
    for message in messages {
        let _ = app.update(message);
    }
    assert!(app.fleet_scroll > 20000.0);
    let last = app.filtered_nodes().last().unwrap().config.label.clone();
    let mut ui = UserInterface::build(app.view(), size, cache, &mut renderer);
    let mut find = Selector::find(last.as_str());
    ui.operate(&renderer, &mut operation::black_box(&mut find));
    let operation::Outcome::Some(Some(target)) = find.finish() else {
        panic!("Last host was not built")
    };
    assert!(
        target.visible_bounds().is_some(),
        "Last row outside the scroll viewport"
    );
    let cache = ui.into_cache();
    let task = app.update(Message::FleetFilter("QA device 0".into()));
    let mut ui = UserInterface::build(app.view(), size, cache, &mut renderer);
    let mut stream = iced_test::runtime::task::into_stream(task).unwrap();
    while let Some(action) = block_on(stream.next()) {
        if let Action::Widget(mut operation) = action {
            ui.operate(&renderer, operation.as_mut());
        }
    }
    let mut find = Selector::find("Online");
    ui.operate(&renderer, &mut operation::black_box(&mut find));
    let operation::Outcome::Some(Some(target)) = find.finish() else {
        panic!("Filtered host missing")
    };
    assert!(
        target.visible_bounds().is_some(),
        "Filter retained an obsolete scroll offset"
    );
}
#[test]
#[ignore = "Requires a graphical backend. Run scripts/qa-ui.sh."]
fn visual_matrix_and_interactions() {
    let dir =
        std::env::var("MOONITOR_QA_DIR").expect("Explicit ignored artifact directory required");
    let dir = std::path::Path::new(&dir);
    std::fs::create_dir_all(dir).unwrap();
    let case = std::env::var("MOONITOR_QA_CASE").ok();
    let mut covered = 0;
    for (width, height) in [(1280, 800), (1440, 900), (3840, 2160)] {
        for (name, section, tab) in [
            ("overview", Section::Overview, Tab::Overview),
            ("host", Section::Device, Tab::Overview),
            ("metrics", Section::Metrics, Tab::Overview),
            ("fleet", Section::Fleet, Tab::Overview),
            ("workloads", Section::Workloads, Tab::Overview),
            ("energy", Section::Energy, Tab::Overview),
            ("events", Section::Events, Tab::Overview),
            ("explore", Section::Explore, Tab::Overview),
            ("settings", Section::Settings, Tab::Overview),
            ("ai", Section::Device, Tab::Ai),
            ("history", Section::Device, Tab::History),
            ("processes", Section::Device, Tab::Processes),
        ] {
            for dark in [false, true] {
                let case_name = format!(
                    "{name}-{width}x{height}-{}",
                    if dark { "dark" } else { "light" }
                );
                if case.as_ref().is_some_and(|c| c != &case_name) {
                    continue;
                }
                covered += 1;
                let mut app = app(5, tab);
                app.dark = dark;
                app.section = section;
                app.viewport = iced::Size::new(width as f32, height as f32);
                let mut ui =
                    Simulator::with_size(settings(), (width as f32, height as f32), app.view());
                let target = ui.find(section.label()).unwrap();
                assert!(target.visible_bounds().is_some());
                if section == Section::Overview {
                    for label in [
                        "QA device 0",
                        "QA device 1",
                        "QA device 2",
                        "QA device 3",
                        "QA device 4",
                        "CPU",
                        "MEMORY",
                        "GPU · MAX",
                        "NETWORK",
                        "STATE",
                        "AGE",
                    ] {
                        assert!(
                            ui.find(label).unwrap().visible_bounds().is_some(),
                            "{label} below fold in {case_name}"
                        );
                    }
                }
                if section == Section::Device && tab == Tab::Overview {
                    for label in [
                        "CPU / PROCESSOR",
                        "GPU / ACCELERATORS",
                        "MEMORY / SWAP",
                        "NETWORK / INTERFACES",
                        "STORAGE / FILESYSTEMS",
                        "PROCESS EXPLORER / campione limitato · 100% = un core",
                        "DIAGNOSTICS / SOURCE & FRESHNESS",
                    ] {
                        assert!(
                            ui.find(label).unwrap().visible_bounds().is_some(),
                            "{label} below fold in {case_name}"
                        );
                    }
                }
                let snapshot = ui.snapshot(&app_theme(dark)).unwrap();
                let path = dir.join(format!(
                    "fixture-{name}-{width}x{height}-{}",
                    if dark { "dark" } else { "light" }
                ));
                assert!(
                    snapshot.matches_image(&path).unwrap(),
                    "Visual diff: {}",
                    path.display()
                );
                // A second render must be identical with frozen observations.
                assert!(
                    ui.snapshot(&app_theme(dark))
                        .unwrap()
                        .matches_image(&path)
                        .unwrap()
                );
            }
        }
    }
    if let Some(case) = case
        && case != "fleet"
    {
        assert_eq!(covered, 1, "Unknown QA case {case}");
        return;
    }
    for count in [0, 1, 2, 5, 16, 32, 500] {
        let app = app(count, Tab::Overview);
        let mut ui = Simulator::with_size(settings(), (1280.0, 800.0), app.view());
        assert!(
            ui.snapshot(&theme())
                .unwrap()
                .matches_image(dir.join(format!("fixture-population-{count}")))
                .unwrap()
        );
    }
}
#[test]
#[ignore = "Requires a graphical backend. Run scripts/qa-ui.sh."]
fn metric_rendering() {
    let dir = std::env::var("MOONITOR_QA_DIR").unwrap();
    for value in [0.0, 1.0, 7.0, 36.0, 37.0, 99.0, 100.0] {
        let reading = Reading::reported(value, "fixture", 100);
        let m = Metric::percent("CPU", Some(&reading), false);
        let mut ui = Simulator::with_size(
            settings(),
            (286.0, 100.0),
            container(metric(m, ACCENT)).padding(22).style(panel),
        );
        let label = format!("{value:.0}%");
        let value_bounds = ui.find(label.as_str()).unwrap();
        assert!(value_bounds.visible_bounds().is_some());
        assert!(
            ui.snapshot(&theme())
                .unwrap()
                .matches_image(std::path::Path::new(&dir).join(format!("metric-{value}")))
                .unwrap()
        );
    }
}

#[test]
#[ignore = "Requires a graphical backend. Run scripts/qa-ui.sh."]
fn changing_metrics_keep_visible_glyphs() {
    use iced_test::core::renderer::Headless;
    use iced_test::runtime::{UserInterface, user_interface};
    for font in settings().fonts {
        iced_test::renderer::graphics::text::font_system()
            .write()
            .unwrap()
            .load_font(font);
    }
    let mut renderer = iced_test::futures::futures::executor::block_on(
        <iced::Renderer as Headless>::new(FONT, 16.0.into(), None),
    )
    .unwrap();
    let mut cache = user_interface::Cache::default();
    let dir = std::path::PathBuf::from(std::env::var("MOONITOR_QA_DIR").unwrap());
    std::fs::create_dir_all(&dir).unwrap();
    for (i, value) in [0.0, 37.0, 7.0, 0.0, 100.0, 1.0, 99.0, 8.0, 10.0, 50.0]
        .into_iter()
        .enumerate()
    {
        let reading = Reading::reported(value, "fixture", 100 + i as u64);
        let element = container(metric(
            Metric::percent("CPU", Some(&reading), false),
            ACCENT,
        ))
        .padding(22)
        .style(panel);
        let mut ui =
            UserInterface::build(element, iced::Size::new(286.0, 100.0), cache, &mut renderer);
        ui.draw(
            &mut renderer,
            &theme(),
            &iced_test::core::renderer::Style { text_color: INK },
            iced::mouse::Cursor::Unavailable,
        );
        let rgba = Headless::screenshot(&mut renderer, iced::Size::new(572, 200), 2.0, BG);
        // Inspect the numeric value region, independently of widget selection or bar pixels.
        let ink = (400..535)
            .flat_map(|x| (44..98).map(move |y| (x, y)))
            .filter(|(x, y)| {
                let at = (y * 572 + x) * 4;
                rgba[at] < 100 && rgba[at + 1] < 120 && rgba[at + 2] < 120
            })
            .count();
        image::save_buffer(
            dir.join(format!("dynamic-{i}-{value}.png")),
            &rgba,
            572,
            200,
            image::ColorType::Rgba8,
        )
        .unwrap();
        assert!(
            ink > 30,
            "Value {value} lost visible glyphs after update {i}: {ink} ink pixels"
        );
        cache = ui.into_cache();
    }
}

#[test]
#[ignore = "Requires a graphical backend. Run scripts/qa-ui.sh."]
fn scene_picking_filter_keyboard_and_scroll() {
    let nodes = crate::fixtures::fleet(2);
    let fleet = FleetViewModel::new(&nodes, None, Instant::now());
    let scene = Scene {
        selected: None,
        devices: fleet.devices,
        camera: Camera::default(),
        dark: false,
    };
    let size = iced::Size::new(800.0, 400.0);
    for index in 0..2 {
        let mut ui: Simulator<'_, Message> = Simulator::with_size(
            settings(),
            size,
            widget::shader(scene.clone()).width(Fill).height(Fill),
        );
        ui.point_at(scene.test_device_point(index, size));
        ui.simulate(iced_test::simulator::click());
        assert!(
            ui.into_messages()
                .any(|m| matches!(m,Message::Select(id) if id==format!("qa-{index}")))
        );
    }
    let mut app = app(5, Tab::Processes);
    let mut ui = Simulator::with_size(settings(), (1280.0, 800.0), app.view());
    ui.click(widget::Id::new("process-filter")).unwrap();
    ui.typewrite("missing");
    ui.tap_key(iced::keyboard::key::Named::Backspace);
    for m in ui.into_messages() {
        let _ = app.update(m);
    }
    assert_eq!(app.filter, "missin");
    let mut ui = Simulator::with_size(settings(), (1280.0, 800.0), app.view());
    assert!(
        ui.find("Nessun processo corrispondente nel campione.")
            .is_ok()
    );
    ui.point_at((900.0, 600.0));
    ui.simulate([iced::Event::Mouse(iced::mouse::Event::WheelScrolled {
        delta: iced::mouse::ScrollDelta::Lines { x: 0.0, y: -10.0 },
    })]);
    assert!(ui.snapshot(&theme()).is_ok());
    drop(ui);
    let _ = app.update(Message::NextNode(1));
    assert_eq!(app.selected.as_deref(), Some("qa-1"));
    let _ = app.update(Message::NextNode(-1));
    assert_eq!(app.selected.as_deref(), Some("qa-0"));
}

#[test]
#[ignore = "Requires a graphical backend. Run scripts/qa-ui.sh."]
fn list_alternative_selection_and_theme() {
    let dir = std::path::PathBuf::from(std::env::var("MOONITOR_QA_DIR").unwrap());
    std::fs::create_dir_all(&dir).unwrap();
    for dark in [false, true] {
        let mut app = app(5, Tab::Overview);
        app.dark = dark;
        let mut ui = Simulator::with_size(settings(), (1280.0, 800.0), app.view());
        let path = dir.join(if dark {
            "fixture-list-dark"
        } else {
            "fixture-list-light"
        });
        assert!(
            ui.snapshot(&app_theme(dark))
                .unwrap()
                .matches_image(&path)
                .unwrap()
        );
        assert!(
            ui.snapshot(&app_theme(dark))
                .unwrap()
                .matches_image(&path)
                .unwrap()
        );
        ui.click("QA device 4").unwrap();
        for message in ui.into_messages() {
            let _ = app.update(message);
        }
        assert_eq!(app.selected.as_deref(), Some("qa-4"));
        assert_eq!(app.section, Section::Overview);
    }
}

#[test]
#[ignore = "Requires a graphical backend. Run scripts/qa-ui.sh."]
fn theme_switch_preserves_device_camera_and_resource_context() {
    let dir = std::path::PathBuf::from(std::env::var("MOONITOR_QA_DIR").unwrap());
    std::fs::create_dir_all(&dir).unwrap();
    let mut app = app(5, Tab::History);
    app.section = Section::Settings;
    app.selected = Some("qa-2".into());
    app.camera.yaw = 0.67;
    app.camera.zoom = 1.25;
    let camera = app.camera;
    for (index, (label, dark)) in [
        ("Deep Space", true),
        ("Moonlight", false),
        ("Deep Space", true),
    ]
    .into_iter()
    .enumerate()
    {
        let mut ui = Simulator::with_size(settings(), (1280.0, 800.0), app.view());
        ui.click(label).unwrap();
        for message in ui.into_messages() {
            let _ = app.update(message);
        }
        assert_eq!(app.dark, dark);
        assert_eq!(app.selected.as_deref(), Some("qa-2"));
        assert_eq!(app.tab, Tab::History);
        assert_eq!(app.camera, camera);
        let mut ui = Simulator::with_size(settings(), (1280.0, 800.0), app.view());
        assert!(
            ui.snapshot(&app_theme(dark))
                .unwrap()
                .matches_image(dir.join(format!("fixture-theme-switch-{index}")))
                .unwrap()
        );
    }
    let _ = app.update(Message::Navigate(Section::Overview));
    let mut ui = Simulator::with_size(settings(), (1280.0, 800.0), app.view());
    assert!(
        ui.snapshot(&app_theme(true))
            .unwrap()
            .matches_image(dir.join("fixture-theme-switch-scene"))
            .unwrap()
    );
}
