# Native V3 interface previews

`overview-light.png` / `overview-dark.png` and `host-light.png` / `host-dark.png`
are direct screenshots from the native Iced/wgpu test renderer on macOS.
They contain **explicit synthetic QA fixtures**, not telemetry from a personal lab.
Capture: 25 October 2026, logical 1440×900, both Moonlight and Deep Space.

The fixture module is `cfg(test)` and is never compiled into the production binary.
No production demo mode or synthetic readings were added. The public preview files
are copied unchanged from the V3 visual matrix; personal live captures remain ignored.

Run `MOONITOR_QA_DIR="$PWD/.local/qa/v3/visual-v3" scripts/qa-ui.sh` for the matrix.
See [qualification](../moonitor-v2/qualification.md). Candidates are reproducible
visual evidence, not an independent design approval. Current-file cleanup does not
remove earlier images from Git history; public release requires a separate audit.
