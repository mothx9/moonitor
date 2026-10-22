# Architecture

## Proposed components

| Component | Role | Status |
| --- | --- | --- |
| `moonitor-core` | Shared telemetry types and measurement semantics | Initial types |
| `moonitor-agent` | Local platform collection and runtime adapters | Planned |
| `moonitor-store` | SQLite ingestion, retention and historical queries | Planned |
| `moonitor-desktop` | Native Rust egui/eframe fleet and device UI | Planned |

The headless agent must not depend on the desktop rendering stack.

## Data flow

An agent on each computer collects platform and runtime measurements. The native
desktop consumes recent snapshots and history. An optional always-on collector
and SQLite store on Zima can preserve history when the MacBook sleeps. The
desktop should also support local collection without that hub.

Reuse existing LAN/VPN reachability without changing network settings. Transport,
enrollment and authentication are implementation decisions still to validate.
Prefer persistent authenticated connections, bounded messages, reconnect backoff
and a small bounded buffer when the receiver is unavailable.

This project has its own state and lifecycle. Existing homelab services, including
Codex Relay, are not repurposed or modified by repository initialization.

## Collection

- Linux: `/proc`, `/sys` and narrowly selected OS APIs.
- NVIDIA: NVML where supported, with explicit unsupported-field handling.
- macOS: system APIs and supported sensor interfaces; validate Apple Silicon
  memory/GPU/power coverage before promising a particular field.
- AI: separate adapters for the runtimes actually found on each node. Start with
  loaded-model and resource observations, then add request-level performance.
- External meters: optional adapters for whole-device power and energy.

## Storage and UI

Use SQLite with batched writes, explicit retention and aggregated older history.
Keep high-frequency data bounded, and keep per-process details out of the default
fleet snapshot. Store receipt times alongside source collection times to support
staleness and clock-skew diagnosis.

The UI has fleet, device, AI and history views. Do not add a browser runtime or
web UI to the desktop. Rendering should wake for new data and user input; detailed
queries and collection must remain outside the UI thread.

## References

- [egui repaint scheduling](https://docs.rs/egui/latest/egui/struct.Context.html)
- [NVIDIA Management Library](https://developer.nvidia.com/management-library-nvml)
- [Spark memory reporting](https://nvidia.custhelp.com/app/answers/detail/a_id/5775/)
- [Ollama loaded models](https://docs.ollama.com/api/ps)
- [Ollama request usage](https://docs.ollama.com/api/usage)
- [vLLM metrics](https://docs.vllm.ai/en/latest/design/metrics/)
- [iOS background strategies](https://developer.apple.com/documentation/backgroundtasks/choosing-background-strategies-for-your-app)
