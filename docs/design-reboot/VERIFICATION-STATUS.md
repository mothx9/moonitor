# Verification status · 25 October 2026

Owner: Codex for engineering evidence, product owner for visual approval. S0 commit
`d034b0c`; design study is a separate commit. PASS below is scoped to the stated
scenario/platform, never extended to an untested redesign or platform.

| IDs from supplied matrix | Current result | Evidence / next gate |
|---|---|---|
| T01 | **PASS, real Mac anomaly + typed boundaries** | Raw PMU −9201.143 °C retained, reason below absolute zero, invalid label, valid other fields. Screenshot/tooltip; source cause unresolved. |
| T02 | **PASS, real four-agent replay** | >90 s freeze still 4/4 online; 103.082 s SQLite verification, 17–18 arrivals each, unchanged profile. Native captured interval; not an overhead test. |
| T03 | **PASS, real app + simulator** | Hidden Mac cannot open through iPhone filter/Enter; explicit row opens inventory host. Visible sorted keyboard order tested. |
| T04 | **PASS, cancellation in real app + simulator** | First-host dialog in viewport, Esc cancels, scoped ID and default Enter cancel, no profile mutation. Actual host removal deliberately unnecessary for this regression. |
| T05–T11 | **PENDING new native slice** | Browser fixtures demonstrate CPU/GPU focus, multiple GPU identities, missing sources/gaps and unavailable state. Chart hover is illustrative, no native synchronized comparison claim. |
| T12–T14 | **DESIGN EVIDENCE ONLY** | A/B/C × Fleet/Host/Setup × 1280/1440 × light/dark; fixed CSS viewports, explicit fixtures, layout report. Final native acceptance and optical owner review pending. |
| T15 | **PENDING** | 1920 and 4K/scale qualification not performed for the replacement. |
| T16 | **PARTIAL** | S0 dialog and visible-node keyboard tests; native filter/Enter/Esc replay. Full primary-flow focus/return/keyboard acceptance still open. |
| T17 | **FAIL on current macOS Iced** | Fresh AX tree only window chrome. ADR records evidence; Qt/Slint equivalent runtime spikes pending. No VoiceOver pass. |
| T18 | **REVIEW REQUIRED** | Both-theme design captures produced; not a human approval record, native rendering differs. |
| T19 | **PENDING** | No controlled AI load or paired inference benchmark run. UI capture CPU cannot be used as idle overhead. |
| T20 | **PARTIAL / fixtures** | Inventory-only real path replayed; CPU-only/empty/stale prototype fixtures. Linux native and network failure task matrix pending. |
| T21 | **DESIGN EVIDENCE + existing editor tests** | Failed-save prototype retains draft; existing persistent editor simulator passed. Redesigned native editor not built. |
| T22 | **NO MIGRATION INTRODUCED** | Git + DB backups, real profile unchanged, existing CRUD restart/history test passes. This does not qualify a future schema/installer migration. |

## Recorded design review limits

A favors simultaneous resources but retains a dense panel grid. B spends more
height on narrative/context and needs evaluation of below-fold chart metadata at
1280. C inspector reserves substantial width and should become collapsible before
native acceptance. The owner has not selected one. All five Setup inventory
entries are visible in the candidates; native legacy Setup's artificial inner
scroll remains until S5. Primary overflow is measured by the in-page QA report;
optical clipping and text legibility still require review. Missing capabilities,
filter state and source identity must be checked interactively, not inferred from
a hero image.

## Repeat the S0 checks

```sh
scripts/check.sh
MOONITOR_QA_DIR="$PWD/.local/qa/design-reboot/recheck" cargo test --locked \
  -p moonitor-desktop ui::qa::trust_regressions_have_visible_feedback_and_scoped_actions \
  -- --ignored --exact --test-threads=1
```

A new image directory creates candidates. Reusing it compares pixels; changed
images require optical inspection before accepting a new baseline. The inset
change intentionally differed from the first candidate set; both were retained.
The targeted test is also registered in `scripts/qa-ui.sh`.

Recheck native live-health separately: wait for healthy fleet; read max received
sample for every monitored agent; freeze for ≥60 s; capture independent health
and fixed visual timestamp; count new DB rows read-only; resume and confirm
latest values immediately. Do not count inventory-only nodes or restore an old
SQLite backup over live history. Backups are a rollback resource, not test input.

No Linux native visual/task result, human visual approval, alternate-toolkit
benchmark or AI-performance result is implied by green Rust checks.

## Design study checks at handoff

84 fixed-viewport captures: 36 principal views (3 compositions × 3 views × 2 sizes
× 2 themes) and 48 edge compositions (8 cases × 3 compositions × 2 sizes, dark).
Per-capture URL/scenario/theme assertions passed; all 84 layout reports have no
primary action outside the viewport. This limited geometry check does not detect
all text clipping, density or below-fold chart metadata. Captures were cropped
using the actual pink viewport boundary; Safari reported DPR separately from the
physical screenshot scale, so DPR alone was not used to crop. Browser chrome and
personal bookmarks are excluded from tracked previews.

Developer interaction replay: hidden selection → explicit inventory row → exact
host → Esc retains filter; GPU #1 shows its own ID/value/memory/board watts and
source; Setup Remove opens centered with default Cancel, Esc leaves five fixtures.
The independent native S0 replay is the production correctness evidence; browser
AX does not qualify Iced. Human task timings and visual preference remain unmeasured.

The preview builder requires Pillow only for optional QA artifact processing:
`uv run --with pillow python design/reboot/build-previews.py`. It consumes ignored
CUA window captures, checks crop aspect and creates the versioned generic previews.
It is not a native render runner. Use the supplied viewport URLs to repeat design
review; `scripts/qa-ui.sh` remains the native renderer harness.
