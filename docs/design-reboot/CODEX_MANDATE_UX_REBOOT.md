# CODEX · MOONITOR UX REBOOT · Principal Engineer / Product Design Mandate

**Scope:** `mothx9/moonitor`, `main`, current local worktree. Read `AGENTS.md`, repository contracts and `MOONITOR_DESIGN_REBOOT_RFC.md` first. This is an outcome-first task; choose implementation decomposition independently while protecting invariants. The spec is a target and hypothesis collection, not a substitute for checking reality.

## Outcome

Transform Moonitor's native desktop interface from fragmented, card/page-oriented prototype into a **unified, professional, data-trustworthy system observability workstation**. The fundamental operational path is `Fleet → Host → Resource/Process/Service → Time → Source/Evidence`, sharing selection, charts, state, density and navigation contracts. Preserve Rust telemetry and its working collection/storage while replacing obsolete visual structure. A successful build and deterministic screenshots are **not** proof of UI/UX quality.

## Authority

- Current repo/worktree, current running bundle and actual telemetry; inspect before making changes.
- `.local/qa/ux-review-20261025/report.md`, evidence JPG, 64-second reproduction video and verification JSON where present. Treat user-provided 26.73-second screen recording as a separate visual evidence source.
- `MOONITOR_DESIGN_REBOOT_RFC.md` (place it in `docs/` or another suitable tracked location), prior specs only where compatible; user rejection of previous UI overrides obsolete layout decisions.
- btop, macOS Activity Monitor, Checkmk, Netdata and Grafana Explore for *behavioral contracts*, not visual copying.
- Actual rendered native window at 1280×800 / 1440×900 / 1920×1080, two themes, macOS and Linux, with screenshots and keyboard/AX evidence.

## Freedom

Perform repository archaeology, trace UI ownership and current data paths, identify what to remove vs preserve, design the component and state architecture, and stage work without superficial patches. Propose a radically stronger visual language informed by professional monitors, and articulate why each spatial allocation serves diagnosis. Compare three high-fidelity but genuinely different composition directions before locking the new design. You may change widget structure completely while preserving core/agent/store contracts. An alternate UI toolkit is permitted only after an evidence-backed ADR and a thin technical spike; do not migrate libraries because they look fashionable.

## Invariants

1. Never invent a measurement, process activity, hardware support or network capability. Zero/absent/unsupported/stale/invalid/denied/unreachable/paused are not interchangeable. Maintain provenance, time, scope and source.
2. Fix immediately the four verified P1 interactions: absurd macOS thermal values, pause causing false fleet outage, hidden filtered selection opening wrong host, remove confirmation outside visible viewport. Reproduce and regression-test independently of redesign.
3. `Pause` freezes inspected visual time only, not real health and ingestion. Live health and frozen snapshot must remain separately observable.
4. Keep host IDs stable and avoid data loss in fleet registry, preferences and historical SQLite. Never silently drop agents or interfere with inference workloads.
5. Remove old page fallbacks from the finished diagnostic path. Fleet/Hosts duplication, Metrics catalog as decorative links, legacy Compute screens and tiny HeatTrack are not permanent architecture.
6. Settings/Setup must represent effective vs planned capabilities honestly. Do not fabricate discovery, pairing or update controls before backend support exists. No TOML/YAML required for the supported base setup.
7. Preserve Moonitor name, crescent identity, Moonlight/Deep Space and local-first/read-only safety posture. Do not include personal hostnames, private IPs, tokens or raw captured personal paths in tracked fixtures/README.
8. Respect current repository workflow including main/worktree/reconciliation rules; avoid force pushes or deleting unrelated work.
9. Use only `vim` if interactive terminal text editing is needed, not nano/other terminal editors.

## Completion standard

Deliver:
- A short **audit delta** stating exactly what already exists, what was removed, what remains unsupported and why.
- Corrective commits/tests/evidence for the four P1 regressions; show real results for the pause/SQLite scenario.
- A comprehensive product/UX state contract, shared components and design tokens with 3 distinct high-fidelity layout candidates at 1440×900 and 1280×800, each including fleet, host, setup and edge cases. Explicitly mark illustrative sample values as fixtures.
- A bounded tech validation for Iced accessibility on Mac. If AX tree exposes no actionable controls, provide spike/ADR comparing feasible frontends; don't mark accessibility done.
- After review/selection of visual direction, a complete first vertical slice using actual live fleet data: fleet row → exact host → focused resource → time/source and back, without old UI fallback. Make this a cohesive, visually qualified experience, not a sequence of legacy pages.
- Later unify Investigate, Workloads/AI, Events and Setup using the same navigation/time/chart/selection/confirmation primitives; no screenshot-only placeholders.
- Deterministic unit/integration/visual tests plus *real user-task testing* with evidence; measure optical quality, task success, text readability, table density, AX and render performance. A failure must be reported explicitly, not waived by passing CI.

## Execution boundaries

- This instruction authorizes **analysis, immediate P1 bug repair and design/prototype deliverables**. A sweeping frontend reimplementation must be preceded by a credible, user-reviewable design gate. Preserve current working app while designing replacement.
- Keep accurate progress in the repository under a discoverable `docs/` path; don't rely on `.local/` files that are ignored for the design authority.
- Be explicit on which results are observed on macOS vs assumed/planned for Linux.
- Do not terminate after producing generic high-level recommendations; create concrete design/state artifacts and testable vertical slice acceptance requirements.

**First task to start now:** audit and fix the four P1 trust/interaction issues without breaking telemetry; in parallel author the information architecture and three competing high-fidelity compositions for the full reboot, with measurable acceptance and a clear implementation architecture. Keep design evaluation separate from 'just ship the UI'.
