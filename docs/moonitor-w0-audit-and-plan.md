# Moonitor — audit operativo e piano di qualificazione

Data: 23 ottobre 2026. Riferimento progettuale: [blueprint integrale](moonitor-professional-upgrade-blueprint.md). Questo documento descrive evidenze e gate; il blueprint resta un documento di indirizzo, non una attestazione di funzionalità.

## Provenienza e conservazione

- Main locale e remoto verificati con fetch: `ba91ef433e83721ea52aad6fbea0d32a30bccf84`, desktop egui/eframe+glow.
- Migrazione locale preesistente salvata senza modifiche in `f75afc0`, branch `feature/native-workbench`. Tag locale `baseline/egui-2026-10-22` sul main precedente.
- Copia aggiuntiva di diff binario, file sorgenti non tracciati e stato Git in `.local/checkpoints/20261023-124305-before-w0/`. Configurazioni, SQLite, screenshot e dati reali restano ignorati in `.local/`.
- Il bundle avviato sul Mac prima della W0 è già Iced/wgpu: la divergenza è fra main remoto e sorgenti/bundle locali, non fra un bundle egui e sorgenti Iced. Non è una release notarizzata.

## Audit del checkpoint (prima delle correzioni W0)

| Area | Evidenza nel codice | Giudizio / confine |
|---|---|---|
| Workspace | Quattro crate, edition 2024, MSRV 1.96 | Separazione collector/core/store valida; desktop troppo monolitico |
| UI | Iced 0.14, wgpu, Inter locale; `ui.rs` | Migrazione effettiva, non pubblicata su main. Sei tab, selezione per indice, colori chiari fissi |
| Rendering | `scene.rs`, `scene.wgsl`, `composite.wgsl` | Mesh procedurali, MSAA 4, shadow map 1024, cap offscreen 1600; cache evita render 3D continuo |
| Scena | `ANCHORS[5]`, `DEVICE_IDS[5]`, mesh `OnceLock` | Cinque asset anche con altra configurazione; shader ignora bitset live; selezione ignota ricade sul primo asset |
| Metriche | `metric(Option<f64>)`, `pct`, `unwrap_or(0)` | Perdita di motivo/fonte/qualità/timestamp; barra assente equivalente a zero; numeri mancanti negli screenshot reali |
| Stato | `NodeState::live`, ricezione monotona <4s | Base valida; denominatore online include nodi senza agente; campioni conservati devono restare esplicitamente obsoleti |
| Processi | top 64 CPU + 32 candidati AI per nome | Non è un censimento esaustivo; nessuna prova che candidato AI significhi inferenza attiva |
| GPU | NVML, IOKit; memoria unified | Buona distinzione memoria. Aggregato max perde identità nello storico; UI attribuisce la prima misura board a ogni GPU: errato su multi-GPU |
| Runtime | Ollama `/api/ps`, timeout 300ms, polling lento | Modelli residenti, non richieste/throughput/TTFT. YVEX e vLLM non implementati |
| Transport | SSH persistente, known hosts strict, batch mode, frame max 512KiB, reconnect | Riutilizzare; canale bounded 16 scarta silenziosamente su `try_send` |
| Schema | `HostSnapshot::validate` | Valida versione, ID, cardinalità, CPU; non tutte le grandezze, clock o correlazione GPU/power |
| Store | SQLite WAL, campione 5s, retention 7d, query 720 punti | NULL distinto da zero. Solo riepiloghi; nessuna migrazione versionata, nessun hub; si interrompe chiudendo il desktop |
| History | UI 5 minuti, gap >7.5s | Nessuno zoom/assi/crosshair; gap e freschezza hanno soglie diverse; dati stored 5s non equivalenti al live 1s |
| Packaging | script `.app` ad-hoc | Copia `.local/fleet.toml` nel bundle: non adatto a distribuzione. Versione statica; nessuna attestazione commit/binary |
| Test | sette test di unità/protocollo/SQLite | Nessuna prova UI, screenshot o interazioni deterministiche pre-W0 |
| CI | Ubuntu/macOS, Rust stable non pin | Run main Linux passa; Mac cancellato dal runner, non evidenza di errore sorgente. Iced locale non ancora qualificato in CI |
| Benchmark | script 32s CPU/RSS, diagnostics render | Misure esplorative, non confronto paired inference; nessun TTFT, intervallo confidenza, scene/list/background controllato |

## Architettura target e confini

Si conservano i quattro crate. Non si introduce un hub, una nuova UI library o una nuova infrastruttura in W0.

- **core**: contratti di dominio/serializzazione e invarianti. W2 aggiunge entità, observation, metric descriptor, capability, event e identità dei componenti; W3 runtime/workload/energy boundary. Nessun widget.
- **agent**: adattatori read-only per piattaforma, GPU e runtime, frequenze differenziate e limiti di payload. Nessuna raccolta predefinita di prompt/output/env/command line completa. Telemetria runtime soltanto se esposta/strumentata.
- **store**: persistenza locale e query; W2 migrazioni forward-only, sample/status/source per componente, eventi, retention/downsampling. DB fallito deve essere visibile. Hub opzionale solo dopo stabilità dello storage.
- **desktop/engine**: acquisizione bounded e stato ricevuto, shutdown/reconnect; UI non esegue SSH direttamente. W2 separa ingestion e query più nettamente.
- **desktop/view_model** (W0): identità selezionata, classificazione stato, conteggi monitorati/censiti, modello scena e metrica tipizzata. Nessun collector o valore inventato.
- **desktop/scene**: geometria parametrica e rendering derivati da device ID/kind/state. Non decide disponibilità, non deduce topologia. Cache invalidata su geometria/stato/camera; elenco accessibile come alternativa.
- **desktop/ui**: componenti di presentazione; W1 introduce shell, tokens, catalogo locale e device detail dopo i gate P0.

### ADR W0-001 — conservare Iced 0.14 + wgpu

Decisione: conservare framework e geometrie valide. Bloccare toolchain e lockfile; aggiungere `iced_test` solo come dipendenza di sviluppo per rendering e interazioni headless. Test unitari puri non dipendono da GPU. Test visuali separati richiedono backend grafico e font dichiarati. Un cambio framework richiede un limite riproducibile con evidenze e ADR separato.

### ADR W0-002 — identità e assenza

Selezione per ID, mai per posizione; fallback al primo nodo soltanto se quello selezionato è stato rimosso. Flotta vuota è configurazione valida e produce empty state; >32, ID duplicati o alias non sicuri restano invalidi. Forma opzionale in config (`visual_kind`) con default generico, nessuna associazione fra ID privato e geometria nel codice. Un nodo `unavailable` è censito ma escluso dai monitorati e dagli offline. Stale conserva ultimo valore e motivo, non implica spegnimento fisico.

### ADR W0-003 — disponibilità e potenza

Text/bar/tooltip derivano dalla stessa metrica; zero ha barra zero e testo `0%`; assenza ha `N/D`, motivo e nessuna barra numerica. Aggregati GPU etichettati come massimo disponibile. La potenza per GPU non è attribuibile su nodi multi-GPU con schema 1: mostrare assenza anziché replicare il primo sensore. Nessun totale whole-device o Wh inventato.

## Milestone e dipendenze

| Milestone | Dipende da | Interventi | Gate verificabile |
|---|---|---|---|
| W0 | checkpoint immutabile | Build pin, correctness metriche/stati, scena 0–32, selezione ID, test e bench | Tutti i gate sotto; test Rust da soli non completano W0 |
| W1 | P0 W0 + review umana screenshot | Wireframe, screen map, tokens/componenti, Overview data-first, device detail | Schermate rispondono a quesito operativo, no clipping, tastiera/contrasto, screenshot approvati |
| W2 | W0 stabile, contratti W1 | Observation/capability/source, migrazioni SQLite, serie componenti/eventi, query/time ranges | Gap non collegati, migrazioni/recovery/clock verificati; desktop chiuso solo se hub esplicitamente attivo |
| W3 | W2 + runtime reale disponibile | Taglio verticale run→request→stop, token/latency reali, energy scopes/coverage, alert read-only | Chi/dove/quanto/da quando/cambiamenti con campioni originali; metriche non esposte restano assenti |
| W4 | W0–W3 qualificati | Package pulito, agent negotiation/rollback, CI multi-platform, soak≥24h, backup/privacy/export | Clean install/upgrade senza perdita, nessuna config privata nel bundle, reconnect/sleep-wake |

## Matrice di accettazione W0

Stati: **PASS** solo con evidenza registrata, **FAIL** difetto riprodotto, **PENDING** non misurato. Il report di esecuzione separato aggiorna gli esiti, senza riscrivere la baseline.

1. Checkpoint/tag e verifica nessun file `.local` tracciato; build `--locked` con toolchain pin; manifest sorgente/binary/bundle.
2. Valori 0/100/None e quattro UnavailableReason distinti; text/bar/tooltip concordi. Campioni stale dichiarati. Screenshot reale senza numeri scomparsi.
3. Flotte 0/1/2/5/16/32; >32/duplicate invalidi; 5→1 e reorder conservano identità e numero asset; click stesso ID in scena/inventario/dettaglio. Nessun live finto per iPhone.
4. Overview/AI/History/Processi a 1280×800, 1440×900 e 3840×2160 logici, light/dark, Mac/Linux; dichiarare scale factor fisico, renderer/font. Click, keyboard, scroll, tooltip, empty, stale. Nessuna auto-approvazione di un nuovo golden: baseline nuova richiede review.
5. Confronto idle scena visibile/elenco e foreground/background, CPU di un core, RSS endpoint, contatori pass render. La soglia preliminare <2%/<150MiB non è una promessa. Latenza p95 ricezione→presentazione <1s va strumentata, non dedotta dal timer 1s.
6. GPU workstation e Spark: paired inference collector off/on, ≥5 repliche, input/model/runtime/version/hash, warmup, throughput e TTFT/p95, potenza con scope, statistiche e intervalli confidenza. Nessun carico concorrente modificato; se non isolabile dichiarare confondenti.
7. Persona diversa dallo sviluppatore identifica CPU/GPU/stato dagli screenshot e dalla build. Gate umano esplicitamente aperto finché non c'è riscontro.

## Vincoli e rischi residui

- Linux GUI e dark theme non diventano qualificati perché Mac compila. Il tema attuale contiene colori fissi; la matrice dark è un requisito aperto finché non implementato e ispezionato.
- Il canvas 3D richiede wgpu; un elenco senza scena riduce lavoro grafico, ma non prova fallback su macchina priva di backend GPU. Accessibilità assistiva e native behavior richiedono test reali.
- GPU workstation è un host condiviso con carico elevato e swap già utilizzato al controllo iniziale. Build e benchmark devono avere limiti di memoria/concorrenza; non arrestare workload altrui. Spark non aveva runtime in PATH né modello piccolo già qualificato alla prima ricognizione; trovare/qualificare un runtime è una dipendenza del benchmark, non una licenza per inventarne i risultati.
- Le misure di ottobre 22 in `.local/qa` sono baseline esplorative pre-W0; non sono risultati delle nuove modifiche.
- Screenshot con dati reali, config, IP/alias e processi restano locali. Fixture QA hanno nomi generici e vengono compilate solo nei test.
- Non si promette un process manager completo, autonomia always-on, throughput AI, energia whole-device, release distribuita o approvazione visiva prima dei rispettivi gate.
