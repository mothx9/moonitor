# moonitor

A native system monitor for a small LAN/VPN homelab, with a dedicated view of
local AI workloads and their resource and energy use.

## Direction

- Rust collectors and a native Rust desktop application using egui/eframe.
- CPU, memory pressure, swap, GPU, storage, network, processes and available sensors.
- AI runtime adapters for loaded models, inference activity and available performance metrics.
- Explicit support for both dedicated GPU memory and unified memory.
- Historical telemetry in SQLite, with bounded retention and aggregated older samples.
- Low overhead verified during real AI workloads, including the monitor's own resource use.

## Current state

Repository foundation only. The workspace currently contains `moonitor-core`,
which defines the initial telemetry types. The desktop UI, platform collectors,
runtime integrations, transport and persistence are planned; no live monitoring
or measured overhead is claimed yet.

```sh
cargo fmt --all -- --check
cargo check --workspace --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
```

## Design

- [Requirements](docs/requirements.md)
- [Architecture](docs/architecture.md)
- [Initial fleet](docs/inventory.md)
- [Implementation sequence](docs/roadmap.md)

Host addresses, credentials, telemetry databases and local configuration stay
outside tracked source files.
