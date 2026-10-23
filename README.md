# moonitor

A native system monitor for a small LAN/VPN homelab, with a dedicated view of
local AI workloads and their resource and energy use.

## Working first version

- Rust collectors and a native Iced 0.14 desktop, rendered with wgpu/Metal or Vulkan.
- CPU/per-core use, available RAM, swap, top processes, disk capacity and interface traffic.
- Linux PSI and available temperature sensors; macOS battery state.
- NVIDIA GPU utilization, memory, temperature, clock, power and throttle reasons via NVML.
- Apple Silicon GPU activity and shared-memory counters through read-only IOKit access.
- Ollama loaded models, allocation, quantization and context when its local API is available.
- Local collection plus one persistent SSH connection per remote computer; no agent listener.
- SQLite summaries every five seconds, seven-day retention and a five-minute chart view.

## Current state

The MacBook, Exon, Spark and Zima have produced real live samples together in the
desktop. The iPhone is inventoried and explicitly marked as not instrumented.
See [qualification](docs/qualification.md) for evidence and limitations.

## Run

On macOS, build a local `.app` and open it:

```sh
scripts/run.sh
```

The app is created at `.local/moonitor.app`. On first packaging, the ignored
`.local/fleet.toml` is copied to `~/Library/Application Support/moonitor/fleet.toml`.
Existing user configuration is preserved. Edit that installed file to change the
fleet; private configuration is never embedded in the bundle. The first packaging
run uses `config/fleet.example.toml` if there is no local file. The executable also
accepts configuration explicitly:

```sh
cargo run --release --locked -p moonitor-desktop -- --config config/fleet.example.toml
```

Without an explicit or installed `fleet.toml`, the executable monitors only its local computer.
Data defaults to `~/Library/Application Support/moonitor` on macOS and
`$XDG_DATA_HOME/moonitor` or `~/.local/share/moonitor` on Linux.

Remote nodes require an appropriate Linux agent binary and existing trusted SSH
access. Install it without adding services or listeners:

```sh
scripts/install-agent.sh YOUR_SSH_ALIAS /path/to/moonitor-agent
```

The installer keeps binaries by content hash and refuses to replace an unmanaged
existing agent. The desktop starts remote agents on connection and closes its SSH
children on exit. Missing agents and broken connections appear in the UI.

## W0 consolidation

The design reference is the [professional upgrade blueprint](docs/moonitor-professional-upgrade-blueprint.md).
See the [verified audit, decisions and W0–W4 gates](docs/moonitor-w0-audit-and-plan.md)
and [W0 execution report](docs/moonitor-w0-qualification.md). W1–W4 are not implemented by this change.

The Lab Map uses exactly the configured fleet (0–32 nodes), stable IDs and actual
availability. Optional `visual_kind = "laptop" | "tower" | "compact" | "board" | "phone"`
selects reusable geometry; its default is generic. Uninstrumented nodes do not
count as monitored/offline. `--no-scene` or **Solo elenco** suspends the 3D scene.
Use `--theme light|dark` and `--size 1280x800` for repeatable UI checks.
Keyboard: Cmd/Ctrl+1…6 selects a tab, Alt+Left/Right changes node, Tab/Shift+Tab moves focus.

## Development

```sh
scripts/check.sh
scripts/qa-ui.sh # graphical backend required; local renderer-specific PNG baselines
cargo build --release --locked -p moonitor-agent -p moonitor-desktop
```

Toolchain is pinned in `rust-toolchain.toml`; use the committed `Cargo.lock`.
The first visual run records candidate baselines in `.local/qa/w0/visual`; it is not
human approval. Subsequent runs compare pixels. Synthetic fixtures exist only in
`#[cfg(test)]` modules. Real screenshots use `--screenshot FILE.png --exit-after 12`.

## Design

- [Requirements](docs/requirements.md)
- [Architecture](docs/architecture.md)
- [Initial fleet](docs/inventory.md)
- [Implementation sequence](docs/roadmap.md)

Host addresses, credentials, telemetry databases and local configuration stay
outside tracked source files.

Request-level throughput, latency and queues, additional AI runtimes, whole-device
energy meters, disk throughput/SMART, service/container details, long-range
history browsing and an always-on store remain follow-up work. The desktop must
be running to collect history in this version.
