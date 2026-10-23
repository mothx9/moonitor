mod config;
mod engine;
mod scene;
mod ui;

use anyhow::{Context, Result, bail};
use std::path::PathBuf;

struct Diagnostics;
impl log::Log for Diagnostics {
    fn enabled(&self, m: &log::Metadata<'_>) -> bool {
        m.level() <= log::Level::Info && m.target().starts_with("iced_wgpu")
    }
    fn log(&self, r: &log::Record<'_>) {
        if self.enabled(r.metadata()) {
            eprintln!("{}: {}", r.target(), r.args());
        }
    }
    fn flush(&self) {}
}
fn main() -> Result<()> {
    if std::env::var_os("MOONITOR_RENDER_DIAGNOSTICS").is_some() {
        static DIAGNOSTICS: Diagnostics = Diagnostics;
        let _ = log::set_logger(&DIAGNOSTICS);
        log::set_max_level(log::LevelFilter::Info);
    }
    let mut config_path = None;
    let mut data_dir = None;
    let mut screenshot = None;
    let mut exit_after = None;
    let mut selected_node = None;
    let mut tab = "overview".to_owned();
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        let mut value = || args.next().context(format!("Missing value for {arg}"));
        match arg.as_str() {
            "--config" => config_path = Some(PathBuf::from(value()?)),
            "--data-dir" => data_dir = Some(PathBuf::from(value()?)),
            "--screenshot" => screenshot = Some(PathBuf::from(value()?)),
            "--exit-after" => {
                let seconds = value()?.parse::<f32>()?;
                if !seconds.is_finite() || seconds <= 0.0 {
                    bail!("Exit delay must be positive");
                }
                exit_after = Some(seconds);
            }
            "--node" => selected_node = Some(value()?),
            "--tab" => tab = value()?,
            "--help" | "-h" => {
                println!(
                    "moonitor [--config fleet.toml] [--data-dir PATH]\n         [--node ID] [--tab overview|processes|ai|disks|network|history]\n         [--screenshot PNG] [--exit-after SECONDS]"
                );
                return Ok(());
            }
            _ => bail!("Unknown argument: {arg}"),
        }
    }
    if config_path.is_none() {
        let bundled = std::env::current_exe()?
            .parent()
            .map(|p| p.join("../Resources/fleet.toml"));
        config_path = bundled.filter(|p| p.is_file());
    }
    let config = config::Config::load(config_path.as_deref())?;
    if let Some(id) = &selected_node
        && !config.nodes.iter().any(|n| &n.id == id)
    {
        bail!("Unknown node: {id}");
    }
    let selected_tab = ui::Tab::parse(&tab).context("Unknown tab")?;
    let data_dir = data_dir.unwrap_or_else(default_data_dir);
    let new_data_dir = !data_dir.exists();
    std::fs::create_dir_all(&data_dir)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if new_data_dir {
            std::fs::set_permissions(&data_dir, std::fs::Permissions::from_mode(0o700))?;
        }
    }
    let selected = selected_node
        .as_ref()
        .and_then(|id| config.nodes.iter().position(|n| &n.id == id))
        .unwrap_or(0);
    ui::run(
        config,
        data_dir,
        selected,
        selected_tab,
        screenshot,
        exit_after,
    )
}

fn default_data_dir() -> PathBuf {
    let base = std::env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."));
    if cfg!(target_os = "macos") {
        base.join("Library/Application Support/moonitor")
    } else {
        std::env::var_os("XDG_DATA_HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|| base.join(".local/share"))
            .join("moonitor")
    }
}
