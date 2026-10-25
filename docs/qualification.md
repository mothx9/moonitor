> Historical qualification of the initial desktop. For the current Iced/wgpu branch use [W0 qualification](moonitor-w0-qualification.md); results below are not evidence for the new changes.

# First working version — 2026-10-22

## Verified

- Formatting, workspace compilation and Clippy with warnings denied pass locally.
- Seven tests pass: real platform collection/protocol round-trip, PSI parsing,
  conservative AI process classification, safe SSH aliases, duplicate node IDs,
  bounded streaming frames, and SQLite null/zero/retention/history behavior.
- Release agents compile on Linux x86_64 and ARM64. The x86 binary runs on both
  Arch GPU workstation and Debian 12 headless node; its highest imported GLIBC version is 2.34.
- The native macOS desktop receives simultaneous live snapshots from the macOS target,
  GPU workstation, Spark and headless node. Actual CPU/core counts, RAM, network and disk observations
  are available; NVIDIA and Apple GPU fields are populated where supported.
- The Spark's unsupported NVML memory fields remain unavailable, not zero. Its
  GPU is explicitly identified as using unified memory.
- Actual four-node summaries appear in SQLite, and `PRAGMA quick_check` returns `ok`.
- Fleet and GPU workstation overview screenshots were inspected at the native viewport.
- The Spark AI view was inspected during a controlled interruption of the
  desktop's own GPU workstation SSH child. GPU workstation retained its last readings with an obsolete
  status, and later samples resumed after a 15.9-second gap. Other nodes continued
  producing samples, SQLite remained healthy, and the desktop closed on schedule.
- The local `.app` is packaged and ad-hoc signed. This is a local development
  application, not a notarized/distributed release.

Local raw snapshots, screenshots, logs and benchmark JSON are under the ignored
`.local/qa/` directory. They are not committed as telemetry fixtures.

## Short resource observation

The observation window was approximately 32 seconds after startup, with the
four-computer fleet connected. Normal background activity was uncontrolled. CPU
percentages below refer to **one logical CPU core**, not total machine capacity.
RSS values are the larger of two endpoint observations, not a measured peak or PSS.

| Process | CPU, one core | Endpoint RSS |
| --- | ---: | ---: |
| Desktop plus local collector | 9.73% | 128.2 MiB |
| GPU workstation agent | 0.68% | 29.3 MiB |
| Spark agent | 0.31% | 21.7 MiB |
| headless node agent | 1.62% | 3.0 MiB |

The desktop's three local SSH children each used approximately 5.8–6.0 MiB RSS;
their CPU increments were below the display resolution in this short window.
NVIDIA agent RSS includes mapped driver libraries. These process RSS values do
not establish unique physical memory consumption and should not be summed as PSS.

Uncompressed Linux JSON snapshots were approximately 9–11 kB each at one sample
per second; that excludes SSH framing and transport overhead. Release executable
sizes were approximately 8.4 MiB for the desktop, 1.2 MiB for the macOS headless
agent and 1.7 MiB for each Linux agent.

Reproduce the observation with a built release and reachable configured nodes:

```sh
python3 scripts/benchmark.py --seconds 30
```

This is not an AI inference impact benchmark, a long-running memory stability
test or a proof of unchanged token throughput/latency. No model was loaded and no
inference request was initiated for qualification.

## Current limits

- iPhone pairing and inventory are verified; phone telemetry is not connected.
- Ollama answers on the Mac and reports no loaded models. The adapter's active
  model path is not qualified against an actual loaded model in this session.
- Additional runtimes, per-request token throughput/latency/queues and model
  energy attribution require integrations beyond system counters.
- NVIDIA board watts are component measurements. Whole-device watts, kWh and
  energy integration are not provided by this version.
- Process details are a bounded subset. Service/container views, disk I/O and
  SMART remain planned.
- SQLite retains seven days of summaries, but the current UI charts only the
  last five minutes. No seven-day soak or old-history aggregation is claimed.
- The desktop must run to collect history. Always-on headless node collection and Mac
  sleep/wake/LAN roaming qualification remain follow-up work.
- IOKit accelerator statistics are best-effort counters tied to the platform.
  No privileged sensor helper, public endpoint or background launch service was added.
- The current UI is a functional baseline. The user's requested refined 3D
  interface is a new visual target and has not been implemented yet.
