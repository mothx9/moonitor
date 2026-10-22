# Implementation sequence

## 0. Repository foundation

- [x] Name the project `moonitor`.
- [x] Record scope, initial fleet and measurement boundaries.
- [x] Start a Rust workspace with shared telemetry types.

## 1. One real collector

- [ ] Refresh live fleet discovery using trusted existing connections.
- [ ] Collect real CPU and memory snapshots on macOS and Linux.
- [ ] Validate unavailable values and timestamps against actual sources.
- [ ] Record collector overhead during idle and a representative AI workload.

## 2. Native local monitor

- [ ] Add the egui/eframe desktop with real local readings and bounded history.
- [ ] Add device details and process drill-down.
- [ ] Measure redraw behavior and desktop overhead.

## 3. Fleet and history

- [ ] Add authenticated persistent transport, enrollment and reconnect handling.
- [ ] Add the four-computer fleet view and explicit staleness states.
- [ ] Add SQLite storage, retention and optional always-on collection on Zima.
- [ ] Verify LAN/VPN changes, disconnects and Mac sleep/wake.

## 4. Local AI and energy

- [ ] Validate NVIDIA and Apple Silicon GPU/memory collection.
- [ ] Integrate the runtimes actually installed on each node.
- [ ] Add model, throughput, latency, queue and cache metrics where exposed.
- [ ] Add supported power sensors with explicit measurement scope.
- [ ] Add external energy meters if available.
- [ ] Add limited iPhone telemetry through an appropriate native integration.

Completion requires real readings, user-visible UI verification and measured
overhead. A successful build alone does not qualify monitoring or efficiency.
