# W1 — redesign nativo e identità dell'app

> Historical revision. The [V3 contract](moonitor-v2/product-requirements.md) replaces the former layout and session-only preferences. Measurements below refer to that earlier build.

23 ottobre 2026. Segue la [specifica W1](moonitor-w1-interface-design.md) e il
[report W0](moonitor-w0-qualification.md). L'avvio del redesign è autorizzato dalla
correzione esplicita dell'utente dopo la W0; non implica il superamento automatico
dei gate esterni W0. Il layout precedente è stato rifiutato dall'utente.

Aggiornamento successivo: [Moonlight e Deep Space](moonitor-lunar-themes.md)
sostituiscono la palette verde su richiesta del proprietario. Le evidenze di questa
pagina restano riferite alla prima build W1; la qualifica dei nuovi temi è separata.

## Cambiamento verificabile

- Shell nuova con Overview, Fleet, Workloads, Energy, Events, Explore e Settings;
  sidebar senza seconda lista dei dispositivi, top bar persistente e switcher per ID.
- Overview con quattro indicatori derivati da osservazioni correnti, andamento CPU /
  RAM / GPU sopra il fold, mappa compatta, modelli osservati e stato delle sorgenti.
- Grafico condiviso con scala percentuale, tempo di ricezione, cursore e campione
  puntuale. I gap, valori assenti, timestamp duplicati o inversi non generano segmenti.
  L'ispezione non sceglie campioni lontani attraverso una lacuna.
- Fleet confrontabile con intestazioni allineate, righe selezionabili e disponibilità
  esplicita. Click su nodo/mappa apre il dettaglio; lo switcher conserva la vista.
- Dettaglio con Summary, Compute, Sensori, AI locale, Processi, Archiviazione, Rete e
  Storico. Senza campione viene mostrata una spiegazione e una via di ritorno; non si
  etichetta più un dispositivo mai strumentato come ultima lettura obsoleta.
- Lab Map alimentata dal modello W0: etichette con fondo leggibile, separazione delle
  etichette vicine, selezione e stati; controlli camera e alternativa elenco.
- Palette chiara/scura, componenti metrici condivisi, barre e avvisi leggibili anche
  nel tema scuro. Nessun cambio di framework o di collector.
- Luna fornita dall'utente nel README, nella sidebar, nell'icona finestra e nel bundle
  macOS. Originale conservato identico: SHA-256
  `29d498e83a19c74fc1713f4bf2ad93c76bd27828c29881cb21f64bf14667766d`.
  [Procedura icone](../assets/brand/README.md), conversione standard 16–1024 px.

## Semantica e limiti delle viste

Workloads espone residenza dei modelli e disponibilità dell'adapter Ollama; non
rappresenta richieste, throughput o TTFT inventati. Energy espone potenza GPU board
con copertura, non potenza totale del laboratorio né Wh. Events mostra diagnostica
corrente e spiega l'assenza di una timeline persistita. Explore usa i cinque minuti
realmente disponibili in memoria; non simula selettori 24h/7d. Processi rimane un
campione limitato, non un censimento completo. UI in italiano con nomi delle sezioni
in inglese: un catalogo di localizzazione completo resta da implementare.

`ui/workbench.rs` compone le viste; `ui/chart.rs` gestisce il grafico; `ui.rs` mantiene
stato, input, metriche e dettagli delle risorse; `view_model.rs` conserva le invarianti
W0. Nessuna nuova sorgente, hub, servizio, porta o modifica di protocollo/storage.
L'unica feature UI aggiunta è il rendering immagini di Iced per l'icona fornita.

## Verifica riproducibile

```sh
scripts/check.sh
MOONITOR_QA_DIR="$PWD/.local/qa/w1/visual-current" scripts/qa-ui.sh
scripts/package-macos.sh
```

La prima esecuzione registra candidati, non un'approvazione umana. Le successive
confrontano i PNG. Ogni caso viene anche renderizzato due volte e deve restituire
pixel identici con osservazioni e clock congelati. Le fixture sono `cfg(test)`, con
nomi QA espliciti, stati misti e lacune nelle serie; non sono una modalità del prodotto.

- 18 test Rust workspace: 3 agent, 14 desktop, 1 store; fmt/check/clippy con warning
  trattati come errori. Cinque test grafici sono eseguiti separatamente dal runner.
- 60 casi per piattaforma: dieci viste × 1280×800 / 1440×900 / 3840×2160 × light/dark.
  Overview/Fleet/Workloads/Energy/Events/Explore/Settings e AI/History/Processes del nodo.
- Elenco alternativo: due catture light/dark aggiuntive, selezione del dispositivo
  e contrasto delle righe; verifica aggiunta dopo la prova della finestra reale.
- Controlli di visibilità dei valori CPU/GPU, timeline, mappa e modelli nel viewport;
  allineamento delle intestazioni Fleet; stabilità della selezione; filtro e tastiera;
  picking della scena; scroll; nodo non strumentato; flotta 0/1/2/5/16/32.
- Snapshot dei numeri dopo aggiornamenti successivi con verifica dei pixel dei glifi,
  non solo presenza del testo nel tree. Test di regressione su gap/assenza/clock inverso.
- macOS: finestra reale ispezionata in entrambi i temi, letture dei cinque nodi,
  cursore del grafico, Fleet, dettaglio, Workloads e scorciatoie Cmd+1…7.
- Linux/GPU workstation: renderer headless wgpu/NVIDIA; lavoro confinato a due core e 4 GiB,
  senza fermare servizi o workload esistenti. Non equivale a una sessione interattiva
  X11/Wayland completa.

L'accumulo di molti device headless nello stesso processo falliva su NVIDIA con
`Create new headless renderer`. Il runner ora isola sia ogni test d'interazione sia
ogni viewport in un processo, così le allocazioni vengono rilasciate. Il fallimento
iniziale e i log successivi restano negli artifact; non è stato ignorato un test.

Il test elenco ha inoltre intercettato il caricamento asincrono del logo PNG: il
primo frame poteva precedere l'upload e differire dal secondo. Il piccolo asset viene
ora decodificato una sola volta in RGBA; Iced carica questo handle sincronicamente.
Il doppio screenshot elenco e il confronto Overview con la baseline esistente
verificano la correzione senza introdurre attese arbitrarie nella UI.

Artifact locali ignorati: `.local/qa/w1/`. Le matrici anteriori `visual-v2`,
`visual-final` e `visual-brand` documentano le iterazioni; la matrice della sorgente
corrente è `visual-current`. Log: `checks-current.log`, `visual-current-macos.log`,
`linux/qualification-current.log`. Catture live: `brand-dark-live.png` e
`overview-final-1440.png` e `current-dark-live.png`; i nomi `fixture-*` indicano dati sintetici di test.

## Prestazioni e gate aperti

Build con icona raster finale, cinque nodi, Overview dark 1440×900 attivata con
Cmd+1, 30,36 s: CPU **1,61% di un core logico**, RSS massimo **138,94 MiB**.
Artifact: `performance-current-visible.json`. In questa finestra i due budget
preliminari sono rispettati; non è una qualificazione statistica o di lunga durata.
La modalità elenco, stessa finestra e cinque nodi in una sessione separata di
30,35 s, misura **1,65% di un core** e **134,84 MiB RSS massimo**
(`performance-current-list.json`). La differenza CPU è compatibile con il rumore
di queste brevi osservazioni; non prova una differenza causale tra 3D ed elenco.

La prima misura W1 (prima dell'icona raster), cinque nodi, Overview visibile a
1440×900, 30,35 s: CPU **2,17% di un core logico**, RSS massimo **143,02 MiB**.
È una finestra esplorativa, non una distribuzione statistica né un confronto paired
con W0; il target CPU <2% non è attestato. I collector non sono cambiati: i benchmark
paired di inferenza GPU workstation/Spark restano descritti nella W0, senza estendere il risultato
alla UI nuova o a runtime/modelli non misurati.

Restano aperti: approvazione visiva dell'utente sul nuovo layout; accessibilità dei
widget alle tecnologie assistive (AX espone solo la finestra); avvio senza backend
GPU; profilo rigoroso foreground/background e latenza fino alla presentazione fisica;
video di sessione; soak e sleep/wake. La UI ha tastiera e modalità senza scena, ma
questi due fatti non dimostrano accessibilità completa o assenza di dipendenza GPU.

Prossimo lavoro: chiudere la review W1 e i budget misurati, poi W2 — modello delle
osservazioni/serie/eventi, migrazioni SQLite e query storiche. Le metriche di richiesta
AI e l'energia integrata appartengono alla W3. Nessuna di queste capacità è dichiarata
realizzata dal redesign.

## Esito delle verifiche

macOS e Linux: 18 test Rust passati, matrice delle 60 viste passata, più i test
grafici di metriche aggiornate, picking/interazioni e le due catture elenco.
Dopo il caricamento RGBA del logo, ripetuti i controlli mirati elenco e il confronto
Overview con la baseline già registrata; entrambi passano. Nessun gate umano è
considerato approvato da questi test.

## Identità del pacchetto

Il manifest nel bundle registra commit, stato dirty, hash delle sorgenti e del binario
di build. La firma macOS cambia i byte Mach-O: confrontare direttamente il suo SHA
con il binario non impacchettato sarebbe scorretto. La ricevuta esterna
`.local/moonitor-build-receipt.json` registra separatamente hash di build, eseguibile
firmato, manifest e icona. Il packaging verifica la firma prima di scrivere la ricevuta.
La ricevuta resta esterna per evitare un riferimento circolare dentro la firma.
