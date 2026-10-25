# MOONITOR · DESIGN REBOOT RFC

**Data:** 25 ottobre 2026  
**Repository di riferimento:** `mothx9/moonitor`, `main`, `b7c8e3b13952ed73521712f39b65a030114b57a4`  
**Stato:** proposta di direzione e contratto progettuale, **non** UI implementata.  
**Prove:** registrazione utente `Screen Recording 2026-10-25 at 17.00.16.mov` (26,73 s); review Codex `.local/qa/ux-review-20261025/report.md` riportata nella conversazione, con video di riproduzione separato (64,04 s). Le due registrazioni **non** sono la stessa prova.  
**Stato repository:** fonti correnti controllate (`ui/instrument.rs`, `ui/setup.rs`, commit). Nessun file GitHub modificato.

## 0. Decisione

L'interfaccia attuale non va aggiustata attraverso altri cicli di padding, palette e pannelli. Inizia una **ricostruzione coerente dell'esperienza operativa**, con due workstream coordinati:

1. **Trust & correctness (P0):** metriche valide e attribuibili; selezione e stato live corretti; azioni senza sorprese.
2. **Experience & composition (P0):** nuovi contratti di navigazione, layout, pannelli, interazione e densità per tutte le superfici, senza trascinare i dettagli della vecchia dashboard.

Il motore Rust, gli agenti, i collector, SQLite e SSH restano dove corretti; nessuna proposta obbliga a buttare il backend. Non aggiungere funzioni o schermate apparentemente complete con funzionalità finte.

## 1. Diagnosi strutturale, distinta dai singoli bug

| Problema di fondo | Evidenza | Conseguenza | Intervento strutturale |
|---|---|---|---|
| UI orientata alle pagine, non ai casi d'uso | Fleet/Hosts quasi duplicati; Metrics e host detail scollegati | L'utente deve conoscere dove cercare una misura | Modello `Scope → Entity → Resource → Time` unico |
| Pannelli auto-contenuti e impilati | Setup: host scroll 210px, spazio fuori area non sfruttato; Metric catalog: righe enormi | Layout disomogeneo, vuoti e scroll inutili | Layout con split panes misurati, altezze content-aware |
| Design token insufficienti | titoli 23px, label 9-11px, numeri 12px nel codice | In una finestra reale molti controlli sembrano didascalie | Scala tipografica orientata alla lettura, test ottico/macOS |
| Nessun contratto di chart | inspector, Investigate e storico differiscono per legenda, asse e gap | Impossibile comparare affidabilmente | Un chart engine + tooltip + legend + sorgente/time |
| Selezione globale ambigua | `selected` sopravvive a filtri che nascondono la riga | Azione 'Apri host' può scegliere un altro dispositivo | `selection` contestuale al result set e ID espliciti |
| Tempo live confuso con tempo osservato | freeze dello snapshot ma stale calcolato rispetto a ora | Falso 0/4 dopo Pausa | Distinguere tempo di acquisizione, vista e salute live |
| Sensori senza validazione semantica | temperature PMU ~ -9200 °C trattate come normali | Perdita di fiducia nel software | Quality pipeline e motivo specifico per ogni lettura |
| Azioni lontane dall'oggetto | conferma Rimuovi accodata al fondo della lista | Non si capisce se il click ha funzionato | Overlay contestuale/dialog gestito da focus |
| Build ≠ UX approvata | screenshot deterministici, ma schermate rifiutate | Mancano controlli di qualità percettiva e flussi | Gate visuale umano + test task + accessibilità |

Osservazioni aggiuntive nella clip utente: Fleet ha matrice relativamente bassa e grande vuoto; timeline in microtracce quasi illeggibili; pagina Metric catalog fatta di lunghi link; Setup come modulo/archivio verticale; sidebar icon-only e microtesto; troppe cornici in livelli concorrenti. La review Codex aggiunge il falso stale in pausa, filtro/host sbagliato, temperature e conferme.

## 2. Modello mentale della UX

L'utente non sceglie tra 'pagine' arbitrarie. Parte da una domanda:

- **Sta succedendo qualcosa?** → `Fleet` (global health, matrix, hot spots, attention)
- **Cosa succede su questo dispositivo?** → `Host workspace` (resource cockpit nello stesso shell)
- **Quale risorsa/processo/servizio?** → `Resource inspector` (fonte + letture + timeline)
- **Quando è successo e come si confronta?** → `Investigate` (filtri e tempi persistenti)
- **Perché monitoraggio non funziona?** → `Setup / Diagnostics` (connessioni, agenti e sorveglianza delle sorgenti)

Navigazione primaria proposta: **Fleet · Workloads · Investigate · Events · Setup**. `Host` è una schermata contestuale accessibile da Fleet, ricerca e deep link. `Metrics` è un catalogo ricercabile nell'inspector/Investigate, non un'intera pagina fatta di collegamenti. `Hosts` non duplica Fleet; inventario/gestione nell'area Setup se e quando ha compiti distinti.

Contratto universale di contesto:

```
Scope {fleet | host | resource | process | service | workload}
EntityId {stable typed ID}
ViewTime {live | frozen_at | range}
Filters {explicit, serializable}
Selection {visible result set, stable ID}
Presentation {panes, density, sort, columns}
```

Ogni azione attraversa questi tipi invece di resettare la pagina o di utilizzare un ID selezionato nascosto. URL non necessario per desktop, ma uno stato serializzabile/testabile aiuta deep link, test e ripristino.

## 3. Riferimenti di mestiere

- **btop:** pannelli stabili CPU/memory/network/process, filtraggio/ordinamento, dettagli processi, presets di layout, controlli rapidi: https://github.com/aristocratos/btop
- **Apple Activity Monitor:** tabella con colonne configurabili, ricerca, selezione e ordinamento corretti; vista processi con contesto persistente: https://support.apple.com/en-ie/guide/activity-monitor/actmntr1001/mac
- **Netdata:** metric context/family, dimensioni esplicite, legenda, drill-down di risorse e fonti: https://learn.netdata.cloud/docs/dashboards-and-charts/tabs/dashboards
- **Checkmk:** separazione dei global views da quelli contestuali di host e servizio, Monitor vs Setup: https://docs.checkmk.com/latest/en/views.html
- **Grafana Explore:** pannelli comparabili con intervalli temporali sincronizzabili: https://grafana.com/docs/grafana/latest/visualizations/explore/get-started-with-explore/

Non copiare stili o schermate alla lettera: estrarre **contratti di interazione** e modelli informativi.

## 4. Direzione visiva: Lunar Instrument

**Carattere:** desktop operativo, tecnico, maturo; composizione per strumenti, non per card marketing. Il marchio Moonitor resta (luna, Moonlight, Deep Space), ma i temi devono risultare come due sistemi coerenti, non recolor uniforme.

**Elementi:**

- Navigazione primaria 56–64 px in modalità compatta; possibilità di estendere label e scorciatoie. Destinazione attiva visibile senza affidarsi a una sfumatura.
- Chrome app 44–52 px, contextbar 34–42 px. Un solo titolo per contesto, non tre volte nome host.
- Split panes con confini, drag handles e layout presets salvabili. Mai scroll nested gratuiti.
- Typography: body 13–14 px minimo pratico, metadati 12 px (non 9–10), numeri tabulari 13–15 px, valori di importanza 19–24 px quando necessari. Non usare 23px per una pagina vuota e 9px per dati importanti. Test a pixel reali Retina e Linux scaling.
- Griglia 4px di base, spazi tipici 4/8/12/16, separatori 1px, angoli 4–8: riduzione bordi di card ripetitive, niente pillette ovunque.
- Stati: online, busy, warn, critical, stale, unreachable, unsupported, uninstrumented, invalid, connecting e frozen. Colore **più** testo/icona/fonte; combinazioni ad alto contrasto.
- Una sola famiglia chart con legenda locale, assi/tick coerenti, tooltip con timestamp/qualità/fonte, zoom/brush, gap validati, confronto. Le sparkline micro non sostituiscono una timeline ispezionabile.
- Tabelle vere: tabular numerals, sort che evidenzia direzione, row hover/focus, sticky headings, scelta colonne, row expansion, virtualizzazione reale, no falsi pulsanti o celle disattivate mimetizzate.
- Nav e azioni prevedibili: Tab/focus ring, Enter apre selezione visibile, Esc annulla, menu contestuale vicino all'oggetto, preferenze persistenti.
- Grafica lunare discreta: logo e tonalità del tema; nessuna metafora spaziale non informativa, nessun 3D nella navigazione di base.

Questi sono token **candidati**; occorre produrre e confrontare tre direzioni visive realmente differenti (strumentale, editoriale tecnica, native macOS/cross-platform), non soltanto tre colori. Selezione con revisione umana prima della produzione.

## 5. Composizione funzionale alle misure

### 5.1 Fleet, 1440×900, cinque host

```
┌──NAV──┬────────── FLEET / all hosts ───── time / connection / command ─────────────┐
│       │ 4 online · 1 uninstrumented · 0 warnings  [group] [filter] [columns]   │
│       ├────────────────────────────────────────────────────────────────────────┤
│       │ HOST         CPU / spark  RAM / cap     GPU / ID  NET / iface    HEALTH│
│       │ node-01      23%          18/48 GiB     61%       eno1  ↑↓      Online│
│       │ node-02      42%          72/128 GiB    91%       wg0   ↑↓      Online│
│       │ node-03       7%           1/2 GiB        –        eth0  ↑↓      Online│
│       │ node-04      12%          14/24 GiB     20%       en0   ↑↓      Online│
│       │ node-05        –               –         –           –        Unmon. │
│       ├─────────────────────────────┬──────────────────────────────────────────┤
│       │ SYNCHRONIZED SERIES        │ ACTIVE ISSUES / SOURCE HEALTH            │
│       │ CPU/GPU/Mem/Net, compare   │ clear reasons and actionable diagnostics │
│       │ hover time and dimensions  │                                          │
│       ├─────────────────────────────┴──────────────────────────────────────────┤
│       │ EVENT / SERVICE ACTIVITY TIMELINE  │ filters to resources / timestamps │
└───────┴────────────────────────────────────────────────────────────────────────┘
```

Numbers above are **illustrative**. This is an information architecture diagram, not a screenshot. No hostnames or metrics from real users belong in fixtures. The five-row table itself must fit without scrolling when editor/dialog is closed. At 1280×800, secondary columns collapse or horizontal scroll is explicit; the table never hides values silently. At 1920 and 4K, use additional columns/inspector or give detail panels space; do not merely stretch one text line 1500 px wide.

**Click semantics:** row click selects visible ID; double click/Enter opens that exact host; metric click opens its resource inspector (host ID + metric descriptor + shared timeframe). Filter can preserve pinned global selection but must not present its action as opening the only visible result. Either clear contextual selection or show explicit 'Selected host is hidden by filter'.

**Fleet isn't a graveyard of giant counters.** Summary is a slim status strip with clickable filters, not four executive KPI cards.

### 5.2 Host cockpit

A single `Host Workspace`, not cockpit + legacy tabs with mismatching components. Information architecture:

```
host / stable id / connection path · last received · [time] [compare] [configure]
┌──────────────────┬──────────────────┬──────────────────┐
│ CPU              │ GPU(s)           │ MEMORY           │
│ whole + per core │ device ID, VRAM  │ physical/cache   │
│ trend and load   │ watts and scope  │ swap and pressure│
├──────────────────┼──────────────────┼──────────────────┤
│ NETWORK          │ STORAGE          │ SERVICES/AI      │
│ iface + RX/TX    │ mount + IO/health│ instrumented only│
├──────────────────┴──────────────────┼──────────────────┤
│ PROCESS TABLE (sortable)           │ EVENTS / INSPECT │
│ PID, proc,start,CPU,RSS,trend      │ source/quality   │
└─────────────────────────────────────┴──────────────────┘
```

This is a *default composition*, not a rigid grid. Resource panels can be resized/expanded as a **focus mode in the same workspace**. `CPU` click expands CPU/core + top attributed processes + synchronized history; it never returns to old `Compute` with full-width bars per core. GPU-less/unsupported nodes reallocate the space. Multiple GPUs use tabs/stacked device headers, NEVER only the maximum. Services and AI are contextual; 'model loaded' ≠ 'active request'.

### 5.3 Investigate

Pick host(s), metric(s), time window → split or overlay charts. One cursor and shared timeline. Event markers, source inspector and numeric/raw sample table. Graph legend shows series identity, unit and availability. Missing/invalid data produces gaps; no connecting line through unknown intervals. Do not pretend request latencies, runtime queue or GPU process attribution exist without sources.

### 5.4 Setup, reflow not vertical landfill

Two-column information architecture or tree/list + contextual detail:

```
SETUP / Networks & agents
┌───────────────────────────────────┬─────────────────────────────────┐
│ HOST INVENTORY                    │ SELECTED HOST                   │
│ Search + Add                      │ Name, source, agent, transport  │
│ 01 node status ...                │ [Test] [Edit] [Update] [Remove]│
│ 02 node status ...                │ connection and diagnostics     │
│ 03 node status ...                │                                 │
│ 04 node status ...                │                                 │
│ 05 node status ...                │                                 │
├───────────────────────────────────┴─────────────────────────────────┤
│ Networks & VPN | Agents | Sources | Storage | Privacy | Appearance  │
└─────────────────────────────────────────────────────────────────────┘
```

New-host wizard = staged discovery/connection/auth/install/verify/result, not an exposed internal agent path by default. The present implementation has only SSH/manual host save. Do not show LAN/VPN discovery as operational until it really is. Any command that changes the fleet must report applied/failed/pending state and never silently stop unrelated collectors.

Contextual removal confirmation/dialog contains host label, impact, Cancel/Esc, foreground focus, no auto-scroll hunt. A failed save must keep entered fields and show field-level errors.

## 6. Data trust contracts

### 6.1 Thermal validity

- A raw reading and its interpreted state are separate. Readings around **−9200 °C** cannot become ordinary values.
- Apply typed unit/range/sensor-kind/source-specific validation at ingestion/normalization; possible sentinel, corrupted source, invalid unit or driver anomaly investigated rather than guessed.
- Quarantine suspect field as `Invalid` with reason and source, preserve other node metrics and retain safe diagnostics for audit; do not silently coerce to 0, clamp, substitute a fictional average or reject the full host snapshot.
- Edge cases: legitimate below-zero readings from some sensors, unsupported keys, missing temperature, stale timestamp. Define plausible range by sensor class; strict bounds are not universally interchangeable.
- Test real captured anomalies + synthetic boundary tests, per-platform.

### 6.2 Live health vs paused visual snapshot

Three distinct notions:

1. `Controller/agent live health` derived from current transport and most recent valid arrival.
2. `Displayed snapshot state` frozen at a declared timestamp; readings from that snapshot are historical **by design**.
3. `Source freshness` computed for live or anchored to the frozen snapshot reference as appropriate, never recalculated as a new outage caused by pausing the UI.

If view is frozen show `VIEW FROZEN · 17:03:42`; keep a small independently live cluster-health indicator. On unpause reconcile immediately to latest live values, respecting stored ingest timestamps. SQLite continuing to collect must not cause 0/4 and warning spikes. Test pause for >=60s with ingestion verified for all healthy agents.

### 6.3 Network and process semantics

- A fleet network cell has a stable, labeled interface (selected route, pinned interface or unambiguous aggregation). Never pick 'max traffic interface' afresh each frame, switching `lo`, WLAN and VPN under the same column. Avoid double counting tunnels/bridges.
- Process identity `host ID + PID + start time` (not PID alone). Display % CPU with documented denominator. Processes sampled by top-N are not exhaustive; UI may not imply complete process registry.
- Invalid, absent, unsupported, denied, stale and zero require distinct data states; maintain sample timestamp, receive timestamp and source quality.

### 6.4 Interaction contracts

- `Open host` receives *the exact host ID of the clicked visible row*. If no visible selection, disable with explanation, or label pinned selection explicitly.
- Filter/sort affects table order and keyboard focus consistently. Return from detail restores filter, scroll, selection and time.
- `Remove` opens local inline dialog/popover within viewport; Esc and Cancel undo pending action without side effects. Confirm sends scoped ID and shows result; destructive actions explicit.
- State transitions (connect/disconnect, agent restart, permissions) visibly update only relevant components; don't replace a card with an unrelated page or reset historical context.

## 7. Frontend architecture: remove widget-driven spaghetti

Proposed boundaries:

```
moonitor-core  / reading, semantics, capability descriptor
        ↓
controller/query services / live state + history + metric registry
        ↓
UI ViewModel / selected scope, resource ID, time, filter, visible IDs
        ↓
UI components / data table, metric panel, chart, inspector, dialog
        ↓
workspaces / Fleet, Host, Workloads, Investigate, Events, Setup
```

Provide a data/query facade so presentation is not bound to `NodeState` field-by-field. Implement `UiContext`, `ViewSelection`, `TimeCursor`, `MetricDescriptor`, `RenderReading` and a contextual command model. A single `MetricChart`/`ChartController`, one `DataGrid`, one `ResourcePanel`, one `ContextInspector`, one `DialogHost`, shared typography/tokens. Parent owns transitions; children are pure/rendering or typed command producers. Accessibility roles and focus order included in widget contracts.

Do **not** make one huge Rust `ui.rs` plus hundreds of `small_button` calls. Move testable components and standardized use cases into modules with independent contracts, while keeping source collectors untouched.

## 8. Framework: evidence gate, no dogma

Keep Iced+wgpu **conditionally**, but the reported macOS accessibility tree contained only window chrome. Accessibility is a product gate, not a low-priority detail. A short, bounded technical spike compares:

- Iced 0.14/available development track: a real sortable row table, focus, modal, VoiceOver/AX, screen scaling, text shaping, 1440×900 runtime.
- Slint: equivalent small spike and license suitability for intended open-source distribution.
- Qt 6 (Rust binding/bridge): equivalent spike, a11y support and packaging/maintenance costs.

Choose only after test results: real AX tree, keyboard use, complex virtualized table, user font scaling, macOS/Linux rendering, binary size, CPU/RSS, maintenance. Qt provides macOS Accessibility and AT-SPI integration; Iced accessibility support has a longstanding public issue. **No migration recommendation based on taste alone.** Keep backend Rust regardless; isolate frontend so swap doesn't affect telemetry.

## 9. Design workflow that prevents another failed iteration

**Research → design authority → prototype → production slice → qualification.**

1. **Benchmark tasks**, not screenshot imitation: trace 8–12 operator tasks across btop/Activity Monitor/Netdata/Checkmk/Grafana, record information location, clicks, time, feedback and state retention.
2. **Three genuinely different high-fidelity compositions:** compact operational, workstation-centric, analytical/investigative, all the same real content and edge states. Evaluate at 1280×800, 1440×900, 1920×1080, 4K / scaling and both themes. Avoid showing only hero screenshots.
3. **Canonical design artifact**: Figma or versioned design reference with tokens, states, spacing, typography, diagrams, scroll regions, density presets, chart primitives, keyboard/AX semantics. A lone static HTML demo is not an authoritative component specification.
4. **Prototype workflow**, not a collection of screenshots: fleet select/filter → host → CPU/GPU → chart → back; Setup add/edit/remove; pause; no data. User visually approves the composition before Rust implementation, independently of Codex's own review.
5. **One complete vertical slice** wired to actual collector data and typed query contract. No hidden legacy fallbacks. Expand only when first slice passes real macOS + Linux checks.
6. **Visual / task quality gate:** automated layout asserts and visual diffs plus a human approval record. Passing 60 screenshots means determinism, not design quality. Codex must surface failing screenshots as evidence, not declare success from CI alone.

## 10. Measurable acceptance gates

**Trust gates, zero tolerance:**
- Negative absurd sensor values do not display as trusted observations and are inspectable.
- Freeze for ≥60s with samples still ingesting leaves independent live fleet health unchanged.
- Filtering `iphone` and selecting/opening works only with a visible ID; no hidden Mac opens accidentally.
- Removing first host shows confirmation in-view within the same interaction; Esc/Cancel leaves configuration unchanged.
- All chart legends/units/timestamps/source quality are self-describing, no unsupported metrics invented.

**Visual/density gates:**
- At 1440×900, 5 hosts all visible with status, CPU, RAM, GPU availability, selected network interface and freshness without table scrolling; optional columns may be controlled.
- At 1280×800, no silently clipped primary controls; missing capabilities reclaim space. Status labels ≥12px under normal scale; content body 13–14px; no core metric hidden behind unrelated panel scroll.
- At 1440×900, Setup displays inventory of 5 hosts without artificial 210px internal scroll when no editor is open; resize behaves predictably when editor is open.
- No legacy detail view; one host header and one coherent content layout. CPU core grids compact and readable; GPU reachable without kilometers of core bars.
- Fleet graphs have meaningful time scale, readable hover values and legend. Eliminate the tiny timeline that cannot be interpreted.
- Pinned contextual controls/selection and active tab/focus recognizably distinct in both themes.

**Task gates:**
- From Fleet, reach exact host's GPU device or temperature source with ≤3 deliberate activations; return preserves host/filter/time and scroll.
- Two host metric comparison has synchronized time and source without search for a second historical view.
- Show error reason for a source failure without losing other active measurements.
- Keyboard-only successful select/filter/open/return and edit/cancel/remove; VoiceOver names roles and actions at least in the primary workflow.
- User never has to hand-edit TOML/YAML in supported base onboarding scenarios; unsupported discovery features are clearly labeled planned/unsupported, not simulated.

**Engineering gates:**
- No unbounded channels/table builds; sample/render workload benchmark with 1, 5, 32 and 100+ simulated nodes where feasible.
- Baseline and candidate CPU/RSS/latency measured on identical host/workload, separate idle vs active inference. No performance claim from a single noisy 30 s sample.
- Mac/Linux screenshot matrix has screenshot diff + semantic checks + manual optical inspection.
- Selection semantics, freeze clock, thermal validation and dialogs have direct deterministic tests.
- Existing monitored hosts/preferences/history preserved; no personal info enters public fixtures, assets, log output or docs.

## 11. Sequencing, no unfinished overhaul

| Phase | Scope | Exit criterion |
|---|---|---|
| **S0 — Guardrails** | Fix thermal invalid, pause/stale, selected host filtering, confirmation; evidence based | Trust checks + replay of reported repros all pass |
| **S1 — UX contract & design study** | Task analysis, UI map, 3 directions, high-fidelity component/state set; framework a11y spike | One approved composition + explicit technology ADR |
| **S2 — New shell & shared infrastructure** | Nav model, design tokens, DataGrid, chart, pane, modal, selection, time | All foundation components accessible, documented/tested |
| **S3 — Integrated Fleet → Host** | Build new Fleet, Host, resource focus, remove legacy drill-down | Live end-to-end 5-host workflow with no fallback pages |
| **S4 — Investigate, services, AI, Events** | Reuse same charts, context, filters & source semantics | Diagnostic paths complete where the data exists |
| **S5 — Setup / onboarding** | Usable inventory + new-host flow, availability/diagnostics, separate discovery/backend feature delivery | Local and one remote host enrolled/configured using UI |
| **S6 — Release qualification** | Cross-platform, scaling, accessibility, soak, inference impact, sanitation | Human design approval + measurable QA + release artifacts |

S0 fixes may be committed independently and quickly. S1 should not be collapsed into another uncontrolled full application rewrite. Parallel backend work is allowed if API and telemetry contracts stay stable. Prefer a user-approved product surface over a broad, unreviewed set of new features.

## 12. Sources inspected (25 Oct 2026)

- `https://github.com/aristocratos/btop`
- `https://support.apple.com/en-ie/guide/activity-monitor/actmntr1001/mac`
- `https://docs.checkmk.com/latest/en/views.html`
- `https://docs.checkmk.com/latest/en/user_interface.html`
- `https://learn.netdata.cloud/docs/dashboards-and-charts/tabs/dashboards`
- `https://grafana.com/docs/grafana/latest/visualizations/explore/get-started-with-explore/`
- `https://github.com/iced-rs/iced/issues/552`
- `https://doc.qt.io/qt-6/qaccessible.html`

---
**Approval rule:** Codex/autotests can prove regressions absent; cannot self-approve the visual language on behalf of the product owner. A design candidate becomes authoritative only after visual review of the real running window and task-based evidence.
