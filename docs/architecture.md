# Architecture

## Components

| Component | Role | Status |
| --- | --- | --- |
| `moonitor-core` | Shared telemetry types and measurement semantics | Implemented |
| `moonitor-agent` | Local platform collection and Ollama adapter | Implemented |
| `moonitor-store` | SQLite summaries, seven-day retention and recent history | Implemented |
| `moonitor-desktop` | Native Rust Iced/wgpu fleet and device UI | Implemented |

The headless agent must not depend on the desktop rendering stack.

## Data flow

The desktop runs one local collector and starts a headless agent on each remote
computer through an existing trusted SSH alias. Newline-delimited JSON snapshots
flow through those persistent SSH sessions. Remote agents listen on no ports and
exit when their output pipe closes. The desktop closes SSH children on normal exit.

An optional always-on collector/store on Zima is still planned. Current history
collection requires the desktop to remain running.

SSH uses batch authentication, strict known-host checking, bounded connection
timeouts, keepalives and a ten-second retry delay. Node IDs and remote command
paths are validated before process creation. No new credentials or enrollment
service are introduced. Messages are limited to 512 KiB before parsing, and a
bounded channel prevents receiver backlogs. Schema and node identity are checked.

This project has its own state and lifecycle. Existing homelab services, including
Codex Relay, are not repurposed or modified by repository initialization.

## Collection

- Linux: `/proc`, `/sys` and narrowly selected OS APIs.
- NVIDIA: NVML where supported, with explicit unsupported-field handling.
- macOS: sysinfo plus IOKit accelerator statistics and power-source APIs.
  Accelerator dictionary keys are best-effort platform counters, not a stable
  cross-version public telemetry contract. No privileged helper is used.
- AI: Ollama `/api/ps` on node loopback, with a timeout and response size bound.
  Process name matches identify candidates only; Python alone is not classified as AI.
- External meters: optional adapters for whole-device power and energy.

## Storage and UI

SQLite uses WAL, a five-second persistence cadence, seven-day deletion and hourly
checkpoint/pruning work. Each row holds compact CPU/memory/GPU/power summaries;
process lists and full snapshots are not persisted. Each node keeps at most 720
in-memory chart points. Source and receipt timestamps are separate; liveness uses
a monotonic receipt clock. Older history aggregation remains planned.

Processes refresh every two seconds; disks, temperatures and Ollama every five
seconds. Snapshot transmission and basic counters run at one-second intervals.
The initial process list contains bounded CPU-heavy processes and AI name matches,
not an exhaustive process census.

The primary shell has Overview, Fleet, Workloads, Energy, Events, Explore and Settings.
Processes, compute, sensors, AI, storage, network and history live in node detail.
`ui/workbench.rs` owns the screen composition; `ui/chart.rs` renders receipt-time
series with explicit gaps and nearest-sample inspection; `ui.rs` retains application
state, reusable metric components and resource views. The chart window uses the
UI polling timestamp, frozen in tests for reproducibility. W1 introduces no new
collector API. The existing Iced image feature is enabled for the supplied brand
asset; its two additional transitive packages are documented in assets/brand/README.md.
Rendering uses Iced 0.14 and wgpu, with one-second state polling. Collection, SSH
and SQLite run outside the UI thread. `view_model.rs` defines availability,
ID selection, scene entities and typed percent metrics. `scene.rs` caches uploaded
meshes until fleet identity/geometry changes; camera, selection and state invalidate
the color pass, geometry invalidates shadows. There is no continuous animation.
The list alternative removes the shader widget. GPU-free startup is not yet qualified.
Ingress drops are counted and shown; schema-1 history still only stores summaries.

See [W0 audit and target module boundaries](moonitor-w0-audit-and-plan.md).

## References

- [Iced 0.14](https://docs.rs/iced/0.14.0/iced/)
- [NVIDIA Management Library](https://developer.nvidia.com/management-library-nvml)
- [Spark memory reporting](https://nvidia.custhelp.com/app/answers/detail/a_id/5775/)
- [Ollama loaded models](https://docs.ollama.com/api/ps)
- [Ollama request usage](https://docs.ollama.com/api/usage)
- [vLLM metrics](https://docs.vllm.ai/en/latest/design/metrics/)
- [iOS background strategies](https://developer.apple.com/documentation/backgroundtasks/choosing-background-strategies-for-your-app)
