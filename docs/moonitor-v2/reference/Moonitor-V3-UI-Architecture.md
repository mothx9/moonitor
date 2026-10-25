# MOONITOR / SYSTEM MONITOR UI ARCHITECTURE · Revision 3

**Status:** design study, not implemented in the native application.
**Scope:** UI information architecture, navigation, density, comparative fleet monitoring, deep node diagnostics and visual qualification.
**Existing code target:** `mothx9/moonitor` Rust workspace, `moonitor-desktop` (Iced + wgpu), `moonitor-core`, `moonitor-agent`, `moonitor-store`.
**Source evidence:** GitHub `main` observed at `0f4d2e02` (2026-10-23); this document is a proposed replacement for the prior W1 visual direction, not a statement that all target telemetry already exists.

## 1. Product premise

Moonitor is an infrastructure instrument, not a SaaS KPI dashboard. Its primary job is to let an operator move from fleet-level awareness to the responsible hardware component, process, service and period in as few interactions as possible. The product must remain valuable for one local machine as well as many remote machines.

Three real patterns are combined without imitating their styling:

- **btop:** a stable resource workspace with CPU/core, memory, disk, network and sortable processes always close to hand. Reference: https://github.com/aristocratos/btop
- **Netdata:** time-series telemetry, metric families, per-host/per-device breakdown, correlation, chart inspection. Reference: https://learn.netdata.cloud/docs/dashboards-and-charts/tabs/metrics and https://learn.netdata.cloud/docs/dashboards-and-charts/expanded-chart-analysis
- **Checkmk:** fleet/host/service inventory, operational state, table-driven global/contextual views, attention management and setup separated from monitoring. Reference: https://docs.checkmk.com/latest/en/user_interface.html and https://docs.checkmk.com/latest/en/views.html
- **Grafana Explore (secondary):** synchronized multi-node comparisons and raw source inspection. Reference: https://grafana.com/docs/grafana/latest/visualizations/explore/get-started-with-explore/

Do **not** emulate an analytics landing page (giant KPI cards, decoration, sparse 3D map, symmetric cards regardless of hardware, fake trends). Do **not** recreate a terminal UI verbatim: native desktop interactions, typography, accessibility, adaptive density and graphs still matter.

## 2. UI ontology: do not confuse these objects

- **Fleet:** authenticated enrolled nodes, groups, reachability and collected/source health.
- **Node:** stable device ID, host identity, network paths, platform, capabilities and samples.
- **Component:** CPU package/core, GPU, filesystem/device, interface, sensor/meter.
- **Metric:** descriptor, unit, scope, quality, timestamp, availability, current value and history.
- **Process:** PID + process-start identity, observed CPU/memory and optional vendor GPU attribution, never an unproven AI workload.
- **Service/runtime:** health state and capability-discovered facts, distinct from process activity.
- **Event:** a persisted state transition or observation with time, severity, provenance and an inspectable source.
- **Time window:** synchronized and consistent across every selected chart; stale or missing gaps must not be connected.

**Semantic invariants:** absent is not 0; stale is not offline; unified RAM is not dedicated VRAM; GPU board W is not whole-node W; CPU % for a process can exceed 100% on a multicore definition; loaded model is not active generation; service health is not CPU usage; `nvidia-smi`-like GPU aggregate must never hide GPU identities when several devices exist.

## 3. Primary navigation

A compact, collapsible rail (56–68 px), not a 220px fixed marketing sidebar.

1. **Fleet** (landing): dense comparative matrix, operational states, warnings, timeline, context inspector.
2. **Hosts**: searchable device inventory and host diagnostic workspaces.
3. **Metrics**: dynamic catalog of CPU, memory, GPU, storage, network, thermals and custom sources.
4. **Processes & Services**: sortable read-only process/source tables, service health, containers where supported.
5. **Investigate**: synchronized time ranges, graph overlays, cross-host and cross-period comparison, event markers, raw values.
6. **Events**: current/persisted incidents, history, acknowledge, thresholds, diagnostics.
7. **Setup**: onboarding, agent enrollment, networks/overlays, transport selection, sources, security, retention and export.

Do not put every device under primary nav as a separate icon or repeat every device in all views. A single host-switcher handles host context.

## 4. Fleet / global monitor

**Upper chrome:** app title, current context, global search, status of Controller and freshness of observations; a compact toolbar includes time window, filters, group and sort.

**Tactical status line (not KPI cards):** registered/streaming/unreachable/uninstrumented counts, active warnings, agent/Controller health. All quantities must be clickable filters. Never invent fleet-wide power or GPU/CPU aggregates with incompatible denominators.

**Primary surface:** a resizable/virtualized comparative table, not five unrelated boxes. Default sortable columns:

| Column | Meaning |
|---|---|
| Host | stable label, OS/platform/capabilities, actual monitoring state |
| CPU | total host %, bounded sparkline; on hover source/cores/time |
| Memory | in-use actual/total plus history, with unified accounting respected |
| GPU | each accelerator or clearly labeled hottest device, per-device drilldown; `unsupported` distinct from absent |
| Network | selected interface RX/TX bytes/s or configurable aggregate without duplicate bridges |
| Disk | capacity plus I/O when instrumented, not arbitrary disk % without denominator |
| Thermal/power | only supported sensors, scoped board/package/device measurement |
| Services | counts/health, not inferred from process name |
| Freshness | seconds since last report and data-source quality |

Rows 48–68px in compact mode. Allow sorting, pinning/freeze of Host, quick filtering (`online`, `has GPU`, `agent error`, groups, tags), selectable columns and keyboard selection. For 0/1/5/32/500 hosts, use a proper virtualized grid and never hard-code a five-device layout.

**Context inspector:** a 300–360px resizable *optional* pane. One click previews the selected host/metric, a second interaction opens a full node diagnostic view. Inspector must not steal 1/3 screen width by default on a narrow display. Remember user pane preference per workspace.

**Fleet timeline:** synchronized per-host activity tracks with event markers; linked to metric charts and Alerts. Must not draw an unmeasured causal relationship from simultaneous events. Support meaningful 15min / 1h / 24h only where persistence supports it.

## 5. Host / deep resource monitor

**Important:** This is NOT another copy of Fleet Overview. It is a dedicated, btop-like cockpit for diagnosing a *single* machine.

The top includes stable device ID, name, OS/architecture, current connection path, agent health, sample age, source diagnostics and an integrated host switcher.

Three vertically stacked, resizable functional regions:

- **Region A / COMPUTE (about 37%):** CPU timeseries + per-core tracks; GPU(s) each separately (utilization, VRAM accounting, power scope, temperature, throttling where available); RAM physical + available/cache/swap/pressure. These regions can expand when the user focuses on a resource.
- **Region B / I/O (about 25%):** network per interface with RX/TX and rates; storage per filesystem/device with real I/O and health; services/runtimes with sources and state.
- **Region C / ACTIVITY (about 38%):** process explorer as a sortable/virtualized *table* with PID/start identity, CPU semantics, RSS/private/used memory, executable name (privacy-bound), status, actual time-series when available; adjacent diagnostic and event timeline. No destructive process actions in the default read-only monitor.

A click on any metric opens a detailed time-series inspector; clicking a process opens its scoped stats; clicking a service opens its status/source history. On GPU-less hosts, do not reserve a large dead GPU rectangle: collapse/reallocate the area while leaving discoverability of sensor status.

### Domain-specific view hierarchy

- `Fleet` → `Host` → `CPU` → core → trend/thermal and associated processes
- `Fleet` → `Host` → `GPU` → GPU ID → processes, power, memory, clocks, throttling and time range
- `Fleet` → `Host` → `Network` → interface → RX/TX/errors/path health
- `Fleet` → `Host` → `Storage` → disk/mount → capacity/IOPS/latency/SMART
- `Fleet` → `Host` → `Runtime` → model/requests/queue/metrics **only when instrumented**
- `Events` → host/component/metric and exact time range

## 6. Investigation mode

This is a deliberate third screen rather than packing 14 mini-graphs into Fleet.

- Select 1–N devices or metrics, compare in side-by-side/superimposed synchronized panels.
- Shared cursor and brush selection, crosshair with timestamps/units, clear missing/stale gaps.
- Raw data table; source, reason, quality and received vs sampled timestamps are inspectable.
- Event markers and comparison of equal-length periods; links to processes, services and nodes.
- Operators may bookmark a layout; defaults must remain useful with zero personalization.

## 7. Settings as a management console

**Settings navigation** is substantive: General/Theme/Density, Networks & VPN, Hosts & Agents, Connections & Pairing, Telemetry Sources, AI/Services, Data & Retention, Alerts, Security, Diagnostics. No text editor or TOML/YAML editing required.

Actual flows: local discovery via DNS-SD, Tailscale/Headscale adapter, WireGuard/other IP tunnels through known peers and manual address, secure identity check before enrollment, install agent with compatibility verification and rollback, explain permission failures, persist state. Settings show **effective configuration**, not only desired switches. No claiming a VPN enumerates services by default.

## 8. Visual system constraints

- Keep Moonitor wordmark and crescent. Deep Space/Moonlight may remain; replace excessive padding, pill badges and fixed card chrome with instrument panels and row dividers.
- Numeric content is dominant: tabular numerals, visible units, 11–13px metadata, 12–14px working text, system/Inter UI with readable contrast, compact dense mode as default.
- A single moderate accent for selection, independent colors for data series and severity. Colors convey data when necessary, never rely on color alone.
- Charts show axes/ticks, exact units, 0 vs missing, time zone/time window and source quality; guard against duplicate/clock-backwards points.
- Minimize automatic animation and render only on new visible data or interaction. User can pause/freeze live mode and inspect history.
- Responsive layouts for 1280x800, 1440x900, 1920x1080, 3840x2160; narrow layout collapses context inspector and moves optional columns to horizontal scroll/column chooser rather than clipping text.
- All primary actions keyboard navigable, no icon-only essential action without accessible label and tooltip; reduced motion and focus visibility.

## 9. Implementation boundaries in Moonitor's repo

Do not treat the HTML study as production code. Production must remain Rust/Iced until justified by reproducible rendering/accessibility/performance evidence. Keep `moonitor-agent` independent of graphics.

Refactor `moonitor-desktop/src/ui.rs`, `ui/workbench.rs`, `ui/chart.rs`, `view_model.rs`, `palette.rs` into explicit, tested view-models/components. Keep state/data semantics in core; UI should never parse arbitrary sample JSON or guess the meaning of missing metrics. Provide shared service interfaces for fleet snapshots, normalized metrics, metric history, diagnostics, events and setup commands with clear async/error states.

No personal hostnames, devices, addresses or VPN credentials belong in Git. Replace actual demo fixtures with generic identities and documentation-reserved addresses. Synthetic data only in explicitly marked demos, screenshots and tests; never in production code paths.

## 10. Acceptance gates for Codex and review

A. **Fleet:** 5 mixed states in a 1280x800 window; all five identifiable and comparable; no horizontal clipping; CPU, memory, GPU, network, freshness and state discernible above the fold. Run 0/1/2/5/16/32/500 virtualized-data tests without unbounded widgets.

B. **Node:** CPU + per-core, GPU + provenance, memory, network, storage, process table, events in a real single-node diagnostic workspace; no false metrics where source unavailable. At 1440x900, essential information visible before scrolling; at 1280x800 smaller panels must be scrollable, not cropped.

C. **Interactions:** fleet selection → context preview → node diagnostics → metric history → fleet; sorting, filtering, host switching, time pickers, keyboard, focus, empty/error/reconnecting and source-unsupported states work. A control that appears enabled must not be a placebo.

D. **Truthfulness:** all data mapped to measured `Reading<T>` or explicit unavailable/source states. Unit tests for null vs zero, stale vs unreachable, process CPU multicore, GPU memory scopes and power boundaries.

E. **Rendering:** visual regression matrix: each view at 1280x800 / 1440x900 / 3840x2160, both themes, Linux and macOS, 0–32 host populations, random host ordering, resource variations, timestamps frozen. Inspect every screenshot, text truncation and color contrast. Tests compiling successfully are not visual approval.

F. **Efficiency:** compare idle and active CPU/RSS and time-to-render against pinned current build; include 5 nodes and active inference on both macOS and Linux. Avoid background rendering for invisible panels, bound time-series objects and keep collectors unaffected by UI.

G. **Release readiness:** Settings/Discovery are real; prior private TOML workflows migrate without data loss; public README screenshot has no personal metadata; no obsolete docs describing old 3D home as the current approved design.

## 11. Deliverables and wave order

**UI0, design-only:** feature and info audit of the current repo, new FLEET+HOST+INVESTIGATE UX contracts, mock screenshots, plan, no wholesale code rewrite yet.

**UI1, foundation:** reusable instrument panel, virtualized matrix, node context, chart primitives, time synchronization, accessibility and theme tokens. No feature bluff.

**UI2, real data:** connect current collectors and SQLite with truthful provenance to new Fleet/Host views; detailed per-GPU, disk/network/process panels; effective no-data states.

**UI3, investigate:** full metric/event engine, interactive historical charts, split compare, source inspector.

**UI4, setup:** production onboarding, VPN/LAN discovery adapters, agent enrollment and persistence. Coordinate with Controller/Agent architecture work, do not hide unavailable backend functions with decorative switches.

**UI5, qualification:** performance, cross-platform screenshot QA, keyboard/accessibility review, packaging and migration.

The visual study is available as `moonitor-system-monitor-v3.html`. It uses fictitious data and exists only to evaluate navigation, density, hierarchy and visual grammar. It is **not** the native app and does **not** read Moonitor's collectors.
