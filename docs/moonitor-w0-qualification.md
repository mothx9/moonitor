# W0 — implementazione e qualificazione

23 ottobre 2026. Riferimento: [audit e gate](moonitor-w0-audit-and-plan.md).
La W0 consolida la migrazione; non equivale al redesign completo W1.

## Interventi

- Checkpoint della migrazione Iced `f75afc0`, branch separato e tag baseline egui.
- Blueprint integrale versionato; Rust 1.96.0 e Cargo.lock riproducibili.
- `FleetViewModel`, `DeviceSceneModel`, metriche tipizzate e selezione per ID.
- Flotta 0–32, forme configurabili, mesh/picking/stato/shader sincronizzati; nessun
  array di cinque nodi nel renderer. Conteggio online esclude nodi non strumentati.
- `0%` distinto da `N/D`; barra assente quando non c'è misura; motivi e provenance
  nei tooltip. Stato obsoleto esplicito; aggregato GPU etichettato massimo disponibile.
- Nessuna attribuzione della prima potenza board a tutte le GPU di un nodo.
- Drop di ingresso contati e visibili; separazione dati reali/fixture compilate solo nei test.
- Tema scuro, elenco alternativo, scorciatoie, screenshot e viewport da CLI.
- Bundle senza fleet privata: config esterna in Application Support, copia precedente
  preservata, manifest con hash sorgenti e binari, firma ad-hoc verificabile.

## Difetto di cattura riprodotto

Lo screenshot richiesto nello stesso `Tick` che sostituisce i paragrafi catturava
un batch del renderer precedente con riferimenti deboli a testi ormai sostituiti.
La prova locale `live/dark-before-final.png` perde CPU/GPU mentre conserva una
memoria rimasta stabile. La cattura differita su un altro turno dell'event loop
(`live/dark-deferred.png`) contiene tutti i valori. La finestra reale è stata
ispezionata tramite Computer Use: numeri presenti e click su AI locale funzionante.
Il test `changing_metrics_keep_visible_glyphs` controlla i pixel dei numeri dopo
successivi aggiornamenti, oltre alla presenza del widget nel tree. Questa evidenza
spiega le catture riprodotte; non riclassifica automaticamente ogni screenshot o
video storico dell'utente come semplice problema di acquisizione.

## Test eseguiti

- macOS: fmt, check, clippy `-D warnings`, unit/protocol/storage tests.
- Fixture 0/1/2/5/16/32, ID invalidi/duplicati/>32, reorder e rimozione della selezione,
  stati online/stale/unreachable/connecting/uninstrumented, zero e quattro motivi
  unavailable, RAM unified e massimo GPU esplicito, mesh finite/identità esatta.
- UI: Overview/AI/History/Processes × 1280×800/1440×900/3840×2160 × light/dark,
  snapshot doppi identici; click navigazione, picking con ID generici, input tastiera,
  filtro, scroll, flotta vuota e nodo senza agente. Scala headless 2: 4K logico
  produce 7680×4320 fisici. I PNG sono specifici del renderer/piattaforma.
- Linux Arch/Exon: test Rust e matrice grafica headless wgpu. Il primo run con
  molti renderer nel medesimo processo falliva creando un device dopo due
  screenshot 4K; isolando ogni caso in un processo, la matrice termina. Cgroup
  massimo 4 GiB e due core, nessun workload preesistente arrestato.
- Mac nativo: cattura CLI e finestra reale; AX espone solo la finestra e i controlli
  di sistema, non i widget. Questo è un limite di accessibilità ancora aperto.

## Misure

Artefatti privati in `.local/qa/w0/`. I risultati del 22 ottobre restano storici,
non vengono riutilizzati come attestazioni di questa build.

- Exon, Qwen2.5-Coder-7B Q4_K_M già presente, llama.cpp 7156, 166 token di prompt
  senza cache e 512 generati, warmup, cinque coppie off/on con ordine alternato:
  mediana variazione throughput **+0,865%**, bootstrap CI95 **[−0,942%; +2,465%]**.
  TTFT p95 esplorativo: **80,35 ms off / 81,87 ms on**. Nessuna regressione
  mediana >2% in questo carico. Cinque richieste per condizione non qualificano
  la coda di latenza sotto traffico reale. Potenza GPU board campionata a 200 ms;
  CPU package e whole-device restano non misurati.
- Mac, scena visibile con diagnostica di latenza attiva: **2,40% di un core**,
  RSS osservato massimo **116,83 MiB** su 30,4 s. Elenco con stessa diagnostica:
  **2,11%**, **116,97 MiB**. Il budget CPU preliminare 2% non è rispettato da queste
  misure strumentate; la modalità normale viene misurata separatamente.
- Contatori render: la sessione diagnostica di 240 s produce 2 passaggi scena e
  1 shadow pass; le composizioni UI continuano sui dati/interazioni, non la geometria.
- Ricezione→richiesta di redraw strumentata separatamente; non è una misura della
  presentazione fisica sul display. Risultati dettagliati nel JSON di latenza.
- Spark GB10, stesso modello (entrambi gli shard SHA-256 verificati), llama.cpp 9529,
  166 token di prompt senza cache e 512 generati, cinque coppie: mediana throughput
  **−0,077%**, CI95 **[−0,482%; +0,774%]**; TTFT p95 esplorativo **91,57 / 91,62 ms**.
  Entrambi i benchmark raggiungono il budget mediano preliminare per questo carico;
  non sono un confronto diretto fra host, avendo versioni runtime diverse.
- Ricezione→richiesta di redraw, 231 osservazioni: mediana **378,89 ms**, p95
  **962,25 ms**, massimo **1001,32 ms**. La misura non include il tempo fino ai pixel
  effettivamente presentati dal display.
- Modalità normale, cinque nodi, finestra visibile, 30,36 s: **3,39% di un core**,
  RSS massimo osservato **124,28 MiB**. Budget memoria rispettato, CPU **non rispettato**.
  Una misura precedente è stata invalidata dall'uscita programmata del processo;
  non viene utilizzata. L'ottimizzazione del desktop resta da qualificare con la W1.

## Gate ancora aperti

- Riconoscimento indipendente di stato/CPU/GPU e approvazione visiva da parte dell'utente.
- Accessibilità assistiva, fallback su macchina priva di backend GPU, presentazione
  display p95 strumentata end-to-end e registrazione video della build.
- Soak 24 h, sleep/wake, packaging notarizzato, migrazioni/clock/history 7d e
  osservabilità delle richieste in produzione appartengono alle wave successive.
- I benchmark sono su host condivisi; nessuna dichiarazione generale di impatto
  inferenza <2% per tutti i runtime/modelli. Sorgenti, hash e campioni sono negli artifact.

La nuova richiesta dell'utente di vedere il redesign avvia il [lavoro W1](moonitor-w1-interface-design.md),
con i gate esterni sopra ancora dichiarati aperti. Non si dichiara W0 universalmente qualificata.
