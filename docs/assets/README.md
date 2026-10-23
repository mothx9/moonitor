# README previews

`overview-light.png` and `overview-dark.png` are direct screenshots from the
native Moonitor application on macOS, captured on 23 October 2026 at a logical
window size of 1440×900. The application source is commit `60fdf36`.

They show live telemetry from the configured fleet, with recent history seeded
from a read-only SQLite backup into an isolated, ignored QA data directory.
The two themes were captured sequentially, so their readings can differ. No
synthetic metrics or retouching were used. Only the Overview is included; it
contains node display names and resource readings, without connection addresses,
process command lines or runtime prompts.

Capture procedure, from the repository root after preparing the isolated history:

```sh
./target/release/moonitor --config .local/fleet.toml \
  --data-dir .local/qa/readme/data --theme light --size 1440x900 \
  --screenshot docs/assets/overview-light.png --exit-after 12
```

Repeat with `--theme dark` and `overview-dark.png`. QA databases and logs remain in
`.local/qa/readme/` and are not committed. These previews document appearance;
they are not performance benchmarks or independent visual acceptance.
