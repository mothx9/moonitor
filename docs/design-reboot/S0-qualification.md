# S0 · Trust and interaction corrections

25 October 2026. Baseline: `main` at `b7c8e3b`. Scope is the four P1 bugs,
not the rejected interface's visual redesign. macOS results only.

## What changed

| Failure | Correction | Evidence |
|---|---|---|
| PMU around −9200 °C displayed as valid | Typed thermal interpretation in core, reused by both sensor views. Invalid field retains raw value, source, sample timestamp and specific reason. No clamp, invented replacement or host rejection. | Real Mac `PMU tdev7`: raw −9201.143 °C; tooltip reports below absolute zero. Other sensors remain valid. Captured anomaly fixture and boundary tests. |
| Pause makes every monitored host stale | Desktop continues copying live controller health. A separate frozen observation set has its own clock. Resume reconciles immediately. | Live four-agent run: freeze remained 4/4 online for over 90 s; SQLite read-only verification window 103.082 s, 18/17/17/18 new stored samples by agent; settings profile unchanged. |
| Filter opens hidden selection | Open and filter Enter require a selected ID in the visible result set. Alt arrows follow visible sorted order. | Mac selected → `iphone` filter → disabled “Seleziona una riga”; Enter leaves Fleet; select the visible row → exact inventory host opens. Deterministic simulator also verifies no hidden Select message. |
| Remove confirmation below viewport | Scoped modal outside all scrollables, with named host, impact, foreground pointer capture and default Cancel keyboard action. | Real first-host Remove opens centered confirmation immediately; Esc cancels. Tests cover Enter default cancel, Tab/action selection, stale/unconfirmed remove IDs and configuration preservation. |

Thermal policy: finite values, physical lower limit −273.15 °C, separate conservative
PMU review range −40…150 °C. General component subzero temperatures remain valid.
The PMU range is an application quarantine policy, not a vendor sensor specification.
Root cause of the macOS raw anomaly is **not established**. Wire schema is unchanged;
other consumers of raw agent JSON must use the interpretation policy themselves.

## Qualification

- `scripts/check.sh`: fmt, workspace check, clippy `-D warnings`, workspace tests.
  **31 tests passed; 10 graphical tests intentionally ignored by this command.**
- Graphical targeted test `ui::qa::trust_regressions_have_visible_feedback_and_scoped_actions`:
  passed; 1280×800 and 1440×900, Moonlight/Deep Space, eight renderer images.
- Existing `fleet_preview_sort_filter_open_and_empty_host` and
  `setup_form_keyboard_and_save`: passed with a graphical simulator.
  The former's outdated config-order expectation was corrected to visible state-sort order.
- Release desktop and agent build with `--locked`: passed. macOS bundle ad-hoc signing
  and `codesign --verify --deep --strict`: passed. No remote agent update or deployment.
- Native interactions replayed against the bundled release; SQLite inspected read-only.
  No host removed, profile write, DB migration or collector change.
- A sensor row right inset was added after optical inspection found the legacy
  scrollbar covering the last characters. The expected visual difference was
  retained separately; a fresh candidate set was generated and inspected, rather
  than silently replacing a baseline.

Private artifacts: `.local/qa/design-reboot/s0/`, `s0-refined/`, and
`s0/native/`. These include screenshots, AX tree, the SQLite count report and
an interval-timed window capture. Captures contain real telemetry and remain ignored.
The video repeats actual sequential window captures at their recorded timestamps;
it is state evidence, **not a smooth screen recording or a latency benchmark**.
An initial capture lost its application binding before 60 s and was excluded from
qualification; the run with a logged process supplied the result above.

## Remaining limits

The current Iced window exposes no actionable controls in the macOS AX tree.
Keyboard handling does not solve VoiceOver. Legacy detail pages, small charts,
network auto-selection by largest traffic, duplicate Fleet/Hosts and the 210 px
Setup scroll remain design debt. Linux optical/task qualification, AI overhead,
24-hour reliability and new-workspace acceptance are not claimed by S0.

Before S2, select a composition and resolve the frontend accessibility ADR.
