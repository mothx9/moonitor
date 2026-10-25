# Canonical state and component contract · proposal 1

This document plus the versioned tokens, compositions and RFC form the reviewable
reference. The HTML is a design instrument, not a production frontend or framework
choice. No composition is approved yet. Acceptance requires the real native slice.

## Navigation and ownership

`Fleet → Host(stable ID) → Resource(component ID / descriptor) → Time → Source`.
Fleet is the single inventory entry. Workloads, Investigate, Events and Setup reuse
this context; no separate Hosts page, decorative Metrics catalog or legacy Compute
fallback in the replacement. A breadcrumb names scope and object. Esc/back returns
to the originating result set, preserving filter, sort, scroll, selected ID and time.
The host switcher changes only host scope and validates the selected component.

The root `UiContext` owns scope, `ViewSelection`, `TimeCursor`, focus return target
and contextual commands. `FleetQuery` produces ordered visible IDs; row actions
carry IDs, never indices. A hidden pinned selection remains distinguishable from
contextual selection and cannot enable “Open”. Resizing/reordering does not change
identity. One click selects; name-link/double-click/Enter opens that exact row;
metric activation opens the resource with host ID + component ID + descriptor.
Arrow traversal follows the visible sorted result. Filter Enter opens only visible
selection. On zero results, explain the filter and offer clear, not another host.

## Observation state is not node state

| Dimension | States | Rendering / invariant |
|---|---|---|
| Transport health | Connecting, Online, Stale, Unreachable, Uninstrumented | Text plus symbol; never color alone. Stale does not prove powered off. Uninstrumented excluded from monitored denominator. |
| Reading | Available(0 included), Absent, Unsupported, Denied, NotConfigured, TemporarilyUnavailable, Invalid | Numeric zero with unit; other states have a short specific reason and source action. No silent zero. |
| Freshness | sampled_at, received_at, age, clock anomaly | Source time and receipt time separate. Historical/frozen readings anchored to reference time. |
| Visual time | Live / Frozen(at) / Historical(range) | Freeze copies observations, not controller state. Independent live health and ingestion continue. Resume reads latest immediately. |
| Command | Idle, Draft, Pending, Applied, Failed | Failed edit keeps draft and field errors. Only successful apply updates effective state. Remove scoped to ID with preserved history. |
| Capability | Supported, Unsupported, Unknown, NotConfigured | Unknown does not claim absent hardware. GPU-less layout reclaims panel width. |

Never sum unified GPU memory into host RAM. Keep all GPU IDs and energy boundaries.
A fleet maximum must say max and expose contributing device. Loaded model is
residency, not a running request. Top process sample is explicitly bounded;
process identity target is host + PID + start time (not yet collected in current
schema). Never invent tokens/s, TTFT, latency, I/O or attribution from utilization.

## Shared components

| Component | Inputs and commands | Required behavior / AX contract |
|---|---|---|
| DataGrid | ordered row IDs, selection, typed column descriptors, filter/sort | Bounded virtualization; named rows/cells, sort direction exposed; visible focus; keyboard follows rows. Compact row target 48–52 px, comfortable 60–64. |
| ResourcePanel | host/component/metric IDs, RenderReading, provenance | One consistent numeric/bar/source interpretation. Focus expands in place; no legacy navigation. Label, unit, definition, timestamp inspectable. |
| MetricChart / ChartController | bounded series, descriptor, common TimeCursor, event markers | Time and unit axes, identity legend, missing-data gaps, crosshair with exact timestamp/value/source; keyboard inspection, zoom/reset; no new per-page chart. |
| ContextInspector | exact selection + query result | Distinguish selected vs hidden; source failure does not erase other metrics. May collapse to reserve primary workspace. |
| TimeControl | range supported by store, visual time, receipt freshness | Only real supported ranges enabled. Live health separate from historical cursor. Charts share cursor and range. |
| AvailabilityState | reason, capability, last sample, viable action | Specific compact content, useful next action; no full-window blank unsupported page. |
| DialogHost | scoped command, object label, impact, return-focus ID | In viewport, blocks background, named dialog/description, initial Cancel, focus trap, Esc cancel; return focus to trigger. |
| HostEditor | stable ID, draft, effective config, validation, apply result | No base onboarding TOML/path editing; advanced agent path disclosed only if needed. Discovery/install controls absent or disabled with factual reason. |
| StatusBadge | semantic state, age | 12 px minimum, text + shape, contrast independent of accent color. |

All of these contracts are targets for a native implementation, not proof that
current Iced widgets expose the required roles. Native S0 dialog uses explicit
keyboard routing as a stopgap; AX remains open.

## Visual foundations

Executable candidate tokens live in `design/reboot/tokens.css`, structural rules in
`compositions.css`. Preserve crescent mark and Moonlight/Deep Space names. Inter
14 px body, 12 px metadata/status, 17 px section, 22–28 px page, 22–28 px numerals;
tabular numeric alignment. Spacing 4/8/12/16/24. Controls 30–36 px; focus 2 px with
3 px offset. Strong surfaces require information grouping, not decoration. No
blur/glass, automatic orbit, pulse or continuous animation. Respect reduced motion.

Deep Space: canvas #10131c, surface #171c29, ink #e8ebf3, muted #a5adbf,
accent #b3a5f7. Moonlight: canvas #f3f4f8, surface #ffffff, ink #232b3e,
muted #5f6b80, accent #6350ac. CPU violet, GPU cyan, memory mauve; warning amber,
danger rose; health cyan with explicit state text. Every native mapping must use
the shared palette, not page-local colors. Contrast must be measured on final
text sizes and selection/hover/focus states, not assumed from this table.

At 1280×800 preserve five fleet rows and all primary controls. Tables may wrap
state/reason, not truncate a measurement invisibly. A uses rail + grid; B top nav
+ evidence column; C labeled sidebar + persistent inspector. Setup shows all five
inventory entries without arbitrary inner height. Main workspace may scroll;
primary header/time/selection remain pinned. Inspectors collapse before the grid
loses readability. At 1920/4K add useful diagnostics, not giant whitespace; final
scaling/font tests remain mandatory. No final pixel specification is locked before
owner selection.

## Native module plan

Keep core/agent/store. Add boundaries inside desktop before adding crates:

- `context`: typed scope, selection, time and command reducer.
- `query`: adapters from engine/history to presentation, availability and source.
- `components/{grid,chart,panel,inspector,availability,dialog,time}`: pure views and
  typed outputs. ChartController owns gestures/cursor; parent owns navigation.
- `workspaces/{fleet,host,setup,investigate,workloads,events}`: composition only.
- `design_system`: palette, typography, spacing, density and focus rules.

Current S0 `trust.rs` separates live/frozen observation data without changing
transport/store. Its source-specific temperature interpretation belongs to core.
Do not move collection into widgets or spread live `NodeState` field access
through the replacement. A query facade makes any justified frontend replacement
independent of SSH/SQLite. No hub, network listener or runtime instrumenter in S2.

## First production slice acceptance

1. One real Fleet of five → visible row → exact host → CPU/core or individual GPU
   → common history/source → back. No legacy detail call anywhere in this path.
2. At most three deliberate activations from fleet to specific GPU/source;
   filter, sort, scroll, ID and time survive return.
3. Fake adapter fixture and actual collector qualification are separate; screenshots
   always identify which. Cases: zero, invalid, missing, stale, denied, unsupported,
   inventory-only, two GPUs, transport drop during freeze, failed save.
4. Keyboard and actual AX primary task pass on Mac, equivalent Linux pass; Iced
   gate cannot be waived by a browser prototype's HTML accessibility.
5. Per-platform 1280/1440/1920/4K optical review, visual diff, shared cursor/gap
   tests, render/CPU/RSS measurements and inference A/B before release claims.
