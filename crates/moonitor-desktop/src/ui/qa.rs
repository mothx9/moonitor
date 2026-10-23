use super::*;
use iced_test::Simulator;
fn app(count: usize, tab: Tab) -> App {
    App {
        _engine: None,
        nodes: crate::fixtures::fleet(count),
        storage_error: None,
        dropped_events: 0,
        pending_samples: Vec::new(),
        selected: (count > 0).then(|| "qa-0".into()),
        scene_visible: true,
        dark: false,
        tab,
        filter: String::new(),
        sort_memory: false,
        camera: Camera::default(),
        started: Instant::now(),
        screenshot: None,
        exit_after: None,
        window: None,
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
#[ignore = "Requires a graphical backend. Run scripts/qa-ui.sh."]
fn visual_matrix_and_interactions() {
    let dir =
        std::env::var("MOONITOR_QA_DIR").expect("Explicit ignored artifact directory required");
    let dir = std::path::Path::new(&dir);
    std::fs::create_dir_all(dir).unwrap();
    let case = std::env::var("MOONITOR_QA_CASE").ok();
    let mut covered = 0;
    for (width, height) in [(1280, 800), (1440, 900), (3840, 2160)] {
        for (name, tab) in [
            ("overview", Tab::Overview),
            ("ai", Tab::Ai),
            ("history", Tab::History),
            ("processes", Tab::Processes),
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
                let mut ui =
                    Simulator::with_size(settings(), (width as f32, height as f32), app.view());
                let target = ui.find("QA device 0").unwrap();
                assert!(target.visible_bounds().is_some());
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
    for count in [0, 1, 2, 16, 32] {
        let app = app(count, Tab::Overview);
        let mut ui = Simulator::with_size(settings(), (1440.0, 900.0), app.view());
        assert!(
            ui.snapshot(&theme())
                .unwrap()
                .matches_image(dir.join(format!("fixture-fleet-{count}")))
                .unwrap()
        );
    }
    let mut app = app(5, Tab::Overview);
    let mut ui = Simulator::with_size(settings(), (1440.0, 900.0), app.view());
    ui.click("AI locale").unwrap();
    for message in ui.into_messages() {
        let _ = app.update(message);
    }
    assert_eq!(app.tab, Tab::Ai);
    let mut ui = Simulator::with_size(settings(), (1440.0, 900.0), app.view());
    ui.click("QA device 4").unwrap();
    for message in ui.into_messages() {
        let _ = app.update(message);
    }
    assert_eq!(app.selected.as_deref(), Some("qa-4"));
    let mut ui = Simulator::with_size(settings(), (1440.0, 900.0), app.view());
    assert!(ui.find("Non strumentato").is_ok());
    ui.click("Torna alla panoramica").unwrap();
    for message in ui.into_messages() {
        let _ = app.update(message);
    }
    assert_eq!(app.tab, Tab::Overview);
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
            container(metric(m, GREEN)).padding(22).style(panel),
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
        let element = container(metric(Metric::percent("CPU", Some(&reading), false), GREEN))
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
