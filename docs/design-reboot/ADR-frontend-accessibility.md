# ADR · Frontend accessibility gate

Status: **provisional retention of Iced; accessibility gate failed; no migration
selected**. 25 October 2026. This is a bounded current-build validation and a
comparison of alternatives, not a completed three-toolkit benchmark.

## Reproducible current result

The S0 bundled macOS application was exercised through actual Fleet filtering,
host selection, sensors and Remove modal. A fresh AX observation exposes only
window, close/fullscreen/minimize controls, title and app menu. No Fleet row,
button, input, sensor label or dialog action is exposed. The captured tree is in
`.local/qa/design-reboot/s0/native/accessibility-tree.txt`; the preceding review
independently produced the same result. Explicit keyboard routing makes S0
usable by keyboard in those cases but does not provide VoiceOver semantics.
The native app is the bounded Iced spike, with actual data and real table/dialog.

Iced's [upstream accessibility issue](https://github.com/iced-rs/iced/issues/552)
remains open at review. This corroborates the local failure, not a claim about
all possible forks or future versions. Do not mark AX done from `iced_test`,
text labels in source, or the HTML study's accessible DOM.

## Alternatives and evidence strength

| Option | Current evidence | Integration / cost | Decision |
|---|---|---|---|
| Retain Iced + wgpu | Actual native table/modal probe fails AX. Existing collection, native packaging and renderer tests work. | Smallest immediate migration cost, but exposing roles/actions requires a verified accessibility implementation, not cosmetic changes. | Keep S0 working; do not lock S2 on an unproved AX promise. |
| Slint frontend, Rust backend | Official docs support explicit accessible roles, labels and actions. No Moonitor Slint binary built in this run. A reported macOS list-item VoiceOver issue still requires explicit validation. | Native Rust integration; UI DSL and custom grid/chart adaptation; license choice and distribution terms require explicit review. | Candidate for equivalent isolated spike, not an approved replacement. |
| Qt 6 frontend, Rust bridge | QAccessible documents roles/names/actions/table interfaces and platform AT bridging. No Moonitor Qt binary built in this run. | Qt runtime/bundling and C++/Rust bridge ownership, model lifetime and build tooling; complex tables are plausible but must be measured. | Candidate for equivalent isolated spike, not automatic accessibility pass. |

Sources: [Slint accessibility practices](https://docs.slint.dev/latest/docs/slint/guide/development/best-practices/),
[Slint license options](https://github.com/slint-ui/slint/blob/master/LICENSE.md),
[reported Slint macOS list VoiceOver issue](https://github.com/slint-ui/slint/issues/12644),
[Qt accessibility API](https://doc.qt.io/qt-6/qaccessible.html).
License references describe choices, not clearance for a release. No new toolkit
was added to Cargo.lock and no production dependency changed.

## Bounded next spike before S2 commitment

Use one shared fixture: five rows, searchable and sortable, one hidden selected ID,
resource drilldown, two GPUs, invalid source and a named confirmation dialog.
Implement only that surface in isolated `.local/spikes/` projects, never replacing
the current app. Budget one working day per alternative; stop on a reproducible
missing capability and retain source/logs. First test standard controls; then the
custom table/chart. Native .app bundle required for a fair Mac AX comparison.

Measure on identical machine/build/fixture: AX roles/names/actions + manual
VoiceOver row/dialog task, Tab/arrow/Enter/Esc focus, 100/200% text scaling, 1280/1440
optical result, 1/5/32/100+ row allocation and virtualization, idle/update CPU/RSS,
input-to-present latency, binary/bundle size, Mac and Linux packaging. Five paired
runs for CPU/latency; preserve raw samples and confidence intervals. No number is
reported here without a run. Qt/Slint install/build, Linux AX and VoiceOver task
results remain **pending**. Framework decision requires these results; visual
composition review can proceed independently.

Retain telemetry and SQLite behind a query facade regardless of outcome. Do not
build a custom Iced AX bridge and a frontend migration simultaneously. Approve a
new ADR only after the same task succeeds and maintenance/packaging costs are
explicit. A browser design prototype is not a WebView migration proposal.
