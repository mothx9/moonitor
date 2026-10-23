# W1 — redesign della shell e delle viste operative

23 ottobre 2026. Dopo la consolidazione W0, l'utente ha chiarito che si aspettava
un redesign completo dell'interfaccia. Questa wave cambia struttura e gerarchia
visiva, mantenendo collector, transport, SQLite e rendering Iced/wgpu.
La qualificazione indipendente della W0 resta un gate esplicito; non viene implicata
dall'avvio del lavoro di design.

## Screen map e contratti

- **Overview**: situazione della flotta. KPI osservabili, andamento del nodo scelto,
  Lab Map compatta, modelli residenti osservati e problemi di raccolta.
- **Fleet**: inventario di tutti i nodi con stato, capacità, contatto e accesso al dettaglio.
  La mappa è una seconda rappresentazione dello stesso modello, con alternativa elenco.
- **Workloads**: runtime osservati, modelli residenti e copertura. Un modello caricato
  non è dichiarato un'inferenza attiva; token/s, TTFT e code restano non disponibili.
- **Energy**: potenza GPU board realmente disponibile e sua copertura; nessuna
  stima whole-device, somma di domini sovrapposti o integrazione Wh implicita.
- **Events**: capacità e diagnostica corrente. Senza persistenza eventi non si
  inventa una timeline. L'empty state spiega il limite e conduce alla Fleet.
- **Explore**: storico recente già disponibile, con selettore nodo e finestre
  realmente supportate. Niente selettore 7d che restituisce sempre cinque minuti.
- **Settings**: stato delle fonti/configurazione e preferenze della sessione.
- **Device detail**: switcher unico, stato e ultimo contatto, tab Summary / Compute /
  Processes / Storage / Network / AI / Sensors / History; le risorse appartengono al nodo.

## Wireframe prima dell'implementazione

```text
┌──────────────────┬─────────────────────────────────────────────────────────────┐
│ moonitor         │ Overview       Live · ultime letture      [selettore nodo]  │
│ Compute lab      ├─────────────────────────────────────────────────────────────┤
│                  │ Monitorati    Modelli residenti     GPU     Potenza GPU     │
│ Overview         ├─────────────────────────────────────┬───────────────────────┤
│ Fleet            │ Resource timeline                   │ Lab Map               │
│ Workloads        │ CPU / Memoria / GPU                 │ 3D compatto o elenco  │
│ Energy           │ percentuali + scala + tempo         │ stato + selezione     │
│ Events           ├─────────────────────────────────────┴───────────────────────┤
│ Explore          │ Modelli osservati: runtime / nodo / modello / memoria      │
│                  ├─────────────────────────────────────────────────────────────┤
│ Settings         │ Stato delle sorgenti e problemi di raccolta                │
│                  │                                                             │
│ Raccolta / errori │                                                             │
└──────────────────┴─────────────────────────────────────────────────────────────┘
```

Dettaglio nodo: header compatto e switcher; tab orizzontali; metriche e grafici
prima dei dati granulari. Fleet: righe comparabili con intera riga selezionabile.
Nessuna lista dispositivi duplicata nella navigazione primaria.

## Design system

- Canvas caldo `#F6F8F5`, superficie bianca, testo `#17251F`; palette scura equivalente.
- Verde salvia come selezione/enfasi; stato accompagnato sempre da testo.
- Inter locale, numeri allineati; heading 24–28, card 13–14, metadati ≥11.
- Sidebar 220, topbar compatta, spaziatura 8/16/24, bordi sottili, raggi 10–14.
- Densità guidata dai dati, non da grandi vuoti. Overview verificata a 1280×800.
- Luna fornita dall’utente come asset originale, con derivati PNG e icona macOS.
- Animazione soltanto su input/stato. Camera 3D, forme e shader della W0 riutilizzati.
- Tooltip di fonte/qualità/timestamp; assenza distinta da zero e da dato obsoleto.

## Accettazione

1. Tutti i percorsi principali navigabili, selezione ID coerente fra mappa, Fleet e dettaglio.
2. Overview mostra KPI, andamento e mappa nel viewport di riferimento senza scroll profondo.
3. Nessun workload, evento, watt totale o numero AI inventato; capacità assente esplicita.
4. Screenshot light/dark e test di click/tastiera/filtri mantenuti; nuove baseline
   sono candidate, non approvate automaticamente.
5. Rendering senza regressione di loop continuo; nuova build e bundle identificabili.
6. Review umana dell'interfaccia separata dai test automatici. W2–W4 non dichiarate implementate.
