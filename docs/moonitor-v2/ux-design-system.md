# UX V3 — griglia e contratti

Riferimento: [specifica V3](reference/Moonitor-V3-UI-Architecture.md) e prototipo
fornito, preservato negli artifact locali. Il prototipo HTML non è codice prodotto.
Luna originale e Moonlight/Deep Space conservati, Iced/wgpu invariati.

## Griglia

| Viewport | Chrome | Fleet | Host | Setup |
| --- | --- | --- | --- | --- |
| 1280×800 | rail 64, top 52, margini 16 | righe 56; inspector chiuso inizialmente; CPU/RAM/GPU/rete/stato/età | compute 222, I/O 162, processi/diagnostica 216; pannelli scorrono | editor prioritario, sezioni operative; scroll verticale |
| 1440×900 | rail 64, top 52, margini 16 | inspector opzionale 300; tabella occupa spazio restante | compute/I/O/attività nello stesso viewport; dettagli scrollabili | modulo e inventario nello stesso contesto |
| 1920×1080 | stessi token, più spazio dati | più colonne e righe visibili | regioni estese senza titoli giganti | larghezza contenuto limitata |
| 3840×2160 | stessa scala logica | righe virtualizzate, più dati visibili | pannelli si espandono; niente upscaling di screenshot | moduli leggibili, niente righe lunghissime |

Rail espandibile con nomi; tooltip sulle azioni compatte. Navigazione: Fleet,
Hosts, Metrics, Processes, Investigate, Events, Setup. Host switcher unico.
Fleet è la home, senza scena 3D. Il renderer precedente resta verificabile nel
codice per regressione, non è la rappresentazione della rete né la home.

## Componenti

InstrumentPanel: titolo 11–12, separatore, contenuto 12–14, metadati 10–11,
padding 12; raggio 7. Matrice con intestazione stabile, righe 56/68, selezione
per ID, ordinamento deterministico e finestra di widget proporzionale al viewport.
Valori, barre e tooltip usano `Metric` e `Reading<T>`. Colore non sostituisce testo.

CPU/per-core, RAM/swap e ogni GPU sono pannelli distinti. GPU memory unified viene
esplicitata; la serie storica GPU massima non viene spacciata per una singola GPU.
Interfacce non sommate; mount non sommati (APFS può condividere capacità).

## Interazioni

- Fleet: click seleziona e apre anteprima; Apri host porta al cockpit. Filtro e
  ordinamento mantengono ID; Alt+frecce percorre gli host.
- Host: pulsanti risorsa aprono dettagli e storia; scelta host conserva contesto.
- Investigate: finestra reale di cinque minuti; pausa congela i campioni presentati (età e cursore restano ispezionabili), raccolta continua. Nessun selettore di range non interrogabile.
- Setup: bozze separate da configurazione effettiva; salva valida/commit/applica,
  annulla scarta solo la bozza. Conferma rimozione esplicita; errori inline.
- Empty, connecting, stale, unreachable e non strumentato hanno spiegazione e
  percorso verso Setup. Zero resta `0%`.

## Verifica

Screenshot non equivalgono ad approvazione umana. Confrontare fixture generiche e
finestre reali separatamente; dark/light, 1280/1440/3840, tastiera, filtri, ritorno,
CRUD e persistenza. L'accessibilità Iced su macOS resta un limite da misurare.
