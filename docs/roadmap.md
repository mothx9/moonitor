# Implementation sequence

## 0. Repository foundation

- [x] Name the project `moonitor`.
- [x] Record scope, initial fleet and measurement boundaries.
- [x] Start a Rust workspace with shared telemetry types.

## 1. One real collector

- [x] Refresh live fleet discovery using trusted existing connections.
- [x] Collect real CPU and memory snapshots on macOS and Linux.
- [x] Validate unavailable values and timestamps against actual sources.
- [ ] Record collector overhead during idle and a representative AI workload.

## 2. Native local monitor

- [x] Add the egui/eframe desktop with real local readings and bounded history.
- [x] Add device details and bounded process drill-down.
- [x] Measure desktop overhead in a short observation window.

## 3. Fleet and history

- [x] Add authenticated persistent SSH transport and reconnect handling, reusing existing SSH identities.
- [x] Add the four-computer fleet view and explicit staleness states.
- [x] Add SQLite storage and retention in the desktop.
- [ ] Add an optional always-on collector/store on Zima.
- [ ] Verify LAN/VPN changes, disconnects and Mac sleep/wake.

## 4. Local AI and energy

- [x] Validate NVIDIA and Apple Silicon GPU/memory collection, including explicit unsupported Spark memory fields.
- [x] Add the local Ollama loaded-model adapter.
- [ ] Integrate additional runtimes actually installed on each node.
- [ ] Add model, throughput, latency, queue and cache metrics where exposed.
- [x] Add NVIDIA GPU power with explicit component scope.
- [ ] Add other supported power sensors and energy integration.
- [ ] Add external energy meters if available.
- [ ] Add limited iPhone telemetry through an appropriate native integration.

Completion requires real readings, user-visible UI verification and measured
overhead. A successful build alone does not qualify monitoring or efficiency.

## 5. Refined native UI and 3D

- [ ] Translate the requested minimal, premium visual direction into a concrete design.
- [ ] Choose between a device scene and a mostly 2D UI with smaller 3D elements.
- [ ] Validate Iced + wgpu as the proposed rendering/UI stack.
- [ ] Implement 3D and motion with idle rendering suspended.
- [ ] Re-qualify GPU/CPU use and memory alongside the current technical baseline.
