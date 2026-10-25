# Moonitor visual and UX acceptance matrix

The matrix is a required review harness; ownership and status must be tracked per build/commit. `Pass` is not allowed from source compilation or a static screenshot alone.

| ID | Scenario / task | Context | Pass evidence | Status |
|---|---|---|---|---|
| T01 | Mac PMU -9200 °C not valid | Mac actual capture | Screenshot + invalid reason + raw source trace | TODO |
| T02 | freeze 60 seconds while ingestion continues | 4 live agents | UI video + SQLite arrivals unchanged + health stable | TODO |
| T03 | filter iPhone while Mac previously selected | Fleet | Open action never opens hidden Mac | TODO |
| T04 | remove first host | Setup | Confirmation visible at click, Esc cancels, no commit | TODO |
| T05 | select host→CPU core→history→return | Fleet/Host | Filter, ID, time and scroll preserved | TODO |
| T06 | select GPU #2 on multi-GPU node | Host | Device identity, memory scope and power valid | TODO |
| T07 | GPU absent | Host | Reclaimed layout, capability reason visible | TODO |
| T08 | process sampling uses bounded top list | Host | UI does not claim exhaustive processes | TODO |
| T09 | network interface change | Fleet | Pinned interface name and scope, no silent auto-switch | TODO |
| T10 | chart missing/invalid intervals | Investigate | Gap, no interpolation, source inspectable | TODO |
| T11 | time range compare across hosts | Investigate | Shared time/cursor, series and units visible | TODO |
| T12 | host list 5 entries | Setup 1440x900 | All visible when form closed without arbitrary nested scroll | TODO |
| T13 | 5 mixed hosts | Fleet 1440x900 | All visible, key metrics aligned and readably sized | TODO |
| T14 | narrower viewport | Fleet/Host 1280x800 | No clipped primary control or invisible CTA | TODO |
| T15 | large viewport | Fleet/Host 1920/4K | Space used for diagnostic information, not stretched emptiness | TODO |
| T16 | keyboard only | primary task flows | Tab/Enter/Esc/arrow, focus ring and shortcuts | TODO |
| T17 | assistive tree | macOS AX/VoiceOver | Buttons, list items, labels/actions exposed and navigable | TODO |
| T18 | dark/light visual review | 1280/1440/1920/4K | Human readability/contrast and intentional hierarchy | TODO |
| T19 | real app under load | local + remote | CPU/RSS/render/input measures, AI workload impact | TODO |
| T20 | no connection / unsupported platform | Mac/Linux/iPhone inventory | Clear truthful specific empty state, viable action | TODO |
| T21 | save/edit host failed | Setup | Form preserved, reason shown, active monitoring unaffected | TODO |
| T22 | migration old profile | production upgrade | Data/preferences/history preserved; rollback plan | TODO |
