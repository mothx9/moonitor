# Audit delta and execution plan

Baseline `main b7c8e3b`, clean and equal to fetched origin at start of 25 October
2026 run. No existing edits discarded. Local Git bundle and SQLite online backups
saved under ignored `.local/checkpoints/design-reboot-20261025/`. Five configured
IDs preserved; no sixth device added. Current source, bundled app and private
review artifacts inspected. The user's 26.73 s recording referenced in the RFC
was not included in this package; it is not the separately generated 64 s review.

## Verified implementation

Four Rust crates remain. Core defines wire readings/snapshots and validation;
agent collects platform/GPU/process/filesystem/network/Ollama data; store is SQLite
WAL with bounded summary retention; desktop owns persistent SSH ingestion, store,
state and Iced presentation. `ui.rs` still mixes reducer and legacy details;
`ui/instrument.rs` adds the V3 resource matrix/cockpit/virtualization; `setup.rs`
owns persistent profile CRUD; `workbench.rs` owns the shell and global sections.
`scene.rs` and shader remain regression surfaces, not current Fleet landing view.

The app is Iced/wgpu, not the blueprint's old egui main. Persistent GUI profile,
add/edit/remove, native resource matrix, individual GPU cockpit, missing-data
chart gaps and top-process wording already exist. Do not rebuild backend features
merely because an older document omitted them. SSH known-host policy, collector
cadence, bounded channels and store retention stay intact.

S0 removes the false outage caused by freezing the desktop copy, hidden-selection
Open and offscreen confirmation. It replaces raw thermal display with a typed
interpretation, preserving field evidence. No schema or fleet migration occurs.
See [S0 qualification](S0-qualification.md) for exact tests and limits.

## Unresolved debt, not quietly marked shipped

| Gap | Impact | Planned phase |
|---|---|---|
| Fleet/Hosts duplication and resource links falling into legacy details | Context loss and two incompatible interaction systems | S2/S3 |
| Largest-traffic interface selected each frame | Fleet column can switch lo/WLAN/VPN silently | Query contract + S3 pinned interface selector |
| PMU source anomaly | Cause unproven; S0 quarantines values only | Collector/source diagnostics after S0 |
| AX only exposes window chrome | Primary assistive workflow fails | ADR gate before S2 |
| 210 px inner host scroll, internal technical copy, weak hierarchy | Host inventory and confirmation hard to use | Dialog S0; full Setup S5 |
| Small independent chart implementations and fixed short window | Poor diagnosis/comparison | Shared ChartController S2, Investigate S4 |
| Process start identity and request instrumentation missing | Cannot claim complete process census or AI request attribution | Schema/runtime work after UI slice |
| History tied to desktop, summary GPU maximum, no persisted event model | Cannot claim always-on hub or full observability | W2/W3 backend roadmap, outside current reboot scope |
| No native Linux optical/AX or paired inference overhead evidence | Cross-platform/performance gate remains open | S6 |

No production page or backend was removed in S0: preserving the working app is
explicit. Legacy frontend paths are to be deleted only once S3 replaces them end
to end. The design prototype is isolated and cannot read production configuration.

## Milestones and dependencies

| Phase | Required input | Concrete output / acceptance |
|---|---|---|
| S0 | Actual repro and protected profile/history | Four independent trust fixes, unit/simulator/native evidence; delivered corrections, documented scope limits |
| S1 | RFC + current ownership + behavioral research | 3 compositions, same content/edge cases, state/component contract, Mac AX probe/ADR; **owner selection and alternative toolkit spikes still open** |
| S2 | Selected composition + justified frontend decision | Context/query boundaries, tokens, grid/chart/pane/dialog/time; accessible primitives with tests |
| S3 | S2 and retained collectors | One live Fleet → exact host → CPU/GPU → time/source → back; no legacy fallback, 5 mixed hosts |
| S4 | Qualified S3, actual storage/runtime capabilities | Reuse primitives across Investigate, workloads and source events; no fabricated request metrics |
| S5 | S2 command/editor contracts, real supported transports | Master/detail Setup; local + remote enrollment/edit/save failure/removal without external config editing |
| S6 | Complete primary tasks | Mac/Linux/scale/AX, visual human review, soak, packaging, CPU/RSS/input and paired inference benchmark |

Reboot phases refine the blueprint's UI work; they do not declare W0–W4 complete.
W0 correctness is strengthened by S0. W1 UI authority is S1–S3/S5. W2 history/query,
W3 request/energy observability, W4 release gates retain separate acceptance.
No additional service, hub or production dependency introduced here.
