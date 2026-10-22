# moonitor

A native system monitor for a small LAN/VPN homelab, with a dedicated view of
local AI workloads and their resource and energy use.

## Working first version

- Rust collectors and a native egui/eframe desktop, rendered with glow/OpenGL.
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

The app is created at `.local/moonitor.app`. Packaging copies the ignored
`.local/fleet.toml` into the app; edit that file and repackage for a different
fleet. The first packaging run copies `config/fleet.example.toml` as a starting
point. The native executable also accepts configuration explicitly:

```sh
cargo run --release -p moonitor-desktop -- --config config/fleet.example.toml
```

Without configuration, an unbundled executable monitors only its local computer.
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

## Development

```sh
scripts/check.sh
cargo build --release --locked -p moonitor-agent -p moonitor-desktop
```

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
