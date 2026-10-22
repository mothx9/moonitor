# Requirements

## Product

moonitor is a complete homelab system monitor with a local desktop UI. Local AI
is a primary use case. The initial desktop target is the MacBook; Linux desktop
support should share the same Rust application.

## Updated visual direction

After reviewing the first functional UI, the user requested a much more refined,
modern, minimal interface with 3D elements. The existing desktop is a technical
baseline rather than the accepted final visual design. Iced + wgpu is the current
selected stack for the upcoming custom UI and integrated 3D; GPUI and egui + wgpu were also
evaluated. No framework migration or 3D implementation is claimed yet.

The user selected a restrained 3D scene containing all five devices. The design
should use clear typography, restrained materials and lighting, device-focused
interaction, and demand-driven animation. It must look and behave as a professional
desktop product.

## Monitoring coverage

1. Fleet: node status, last successful observation, LAN/VPN connectivity and stale data.
2. CPU: aggregate/per-core utilization, frequency and pressure where available.
3. Memory: available memory, pressure, swap and out-of-memory events where available.
4. GPU: utilization, memory, temperature, power and throttling where supported.
5. Storage: capacity, throughput, I/O pressure and available health sensors.
6. Network: interface throughput and errors; distinguish the active transport path.
7. Processes, services and containers: resource attribution and selected drill-down views.
8. Energy: available component power and external whole-device measurements.
9. AI: runtime, loaded model, model memory, active work, queue, tokens/s, latency,
   context and cache statistics when exposed by that runtime.
10. History: aligned time series, configurable thresholds and bounded storage.

These are requirements, not implemented capabilities.

## Measurement semantics

- Missing, unsupported, denied and temporarily unavailable data must never become zero.
- Keep the collection time, source and reported/estimated quality of each reading.
- Show stale observations explicitly; failed connectivity is not proof of shutdown.
- Preserve CPU, GPU, SoC and whole-device power boundaries. Never sum overlapping sensors.
- Unified memory is shared physical memory. GPU allocations are not extra host RAM.
- Energy integration requires a documented power source and sampling coverage.
- Whole-device energy and per-request AI energy need suitable instrumentation;
  component readings alone do not establish either quantity.
- Inference throughput and latency come from runtime APIs or explicitly instrumented
  requests, not inferred from GPU utilization or process names.
- No collection of prompts, generated text, credentials, environment variables or
  full process command lines by default.

## Efficiency

- Start with one-second host samples, slower expensive probes, and demand-driven
  process details. Adjust only after measuring overhead and data usefulness.
- Use direct OS/vendor interfaces where possible; avoid spawning a command per metric per tick.
- Bound queues, in-memory history and disk retention; batch persistence work.
- Repaint the desktop when observations or user interaction require it.
- Measure collector CPU/RSS, desktop CPU/RSS, network bytes, disk writes and effects
  on AI throughput/latency against a baseline on the same hardware.
- Set concrete performance budgets after the first measured collector, not by assumption.

## Platform boundaries

- macOS and Linux require separate platform adapters and capability discovery.
- GPU support must distinguish NVIDIA dedicated memory from Spark unified memory.
- Permissions and sensor availability vary; privileged collection is an optional,
  narrowly scoped integration, not the default for the whole application.
- iPhone is a separate, limited telemetry source. Public iOS APIs and background
  execution rules do not provide desktop-equivalent continuous monitoring.
