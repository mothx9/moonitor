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

An optional headless Controller is planned in V2 W2. Current history
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

The V3 primary shell has Fleet (landing), Hosts, Metrics, Processes, Investigate,
Events and Setup. A compact 64px rail can expand; Fleet uses a bounded visible row
window, sort/filter and optional inspector. Host Monitor combines compute, individual
GPUs, memory, network, filesystems, processes and diagnostics. The former 3D renderer
is retained for focused regression tests, not the default home.

`ui/workbench.rs` owns routing/chrome, `ui/instrument.rs` the dense data workspaces,
`ui/setup.rs` host forms and settings commands, `profile.rs` typed transactional
SQLite settings, `ui/chart.rs` receipt-time charts and inspection. `ui.rs` owns
application state and reusable typed metric components. `view_model.rs` keeps
availability/identity/metric contracts. There are no synthetic production readings.

Settings import a legacy profile once without changing IDs or history. Changes are
validated before a revision-checked transaction; concurrent windows cannot overwrite
each other silently. Collection replacement runs off the UI event loop, stops and
joins previous workers and reaps SSH children. Ingestion events carry stable node
IDs, never display positions. Inventories have a 1 MiB serialized budget; the first
engine has a conservative 32-stream concurrency budget (local takes priority).
Additional enrolled hosts show a capacity state distinct from a network failure.
That bound is not a benchmark claim for 32/500 concurrent streams; W2 needs a
scheduler and resource qualification. Fleet rendering is tested with 500 fixtures.

The desktop and collectors poll at one second without a decorative render loop.
Full GPU-free startup and assistive-technology accessibility remain unqualified.
Settings are in a separate versioned SQLite database; history remains the existing
summary schema and 5-second persistence cadence. Collection stops with the desktop.

See [W0 audit and target module boundaries](moonitor-w0-audit-and-plan.md).

## References

- [Iced 0.14](https://docs.rs/iced/0.14.0/iced/)
- [NVIDIA Management Library](https://developer.nvidia.com/management-library-nvml)
- [Spark memory reporting](https://nvidia.custhelp.com/app/answers/detail/a_id/5775/)
- [Ollama loaded models](https://docs.ollama.com/api/ps)
- [Ollama request usage](https://docs.ollama.com/api/usage)
- [vLLM metrics](https://docs.vllm.ai/en/latest/design/metrics/)
- [iOS background strategies](https://developer.apple.com/documentation/backgroundtasks/choosing-background-strategies-for-your-app)
