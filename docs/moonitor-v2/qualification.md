# V3 / primo percorso verticale — qualifica

25 ottobre 2026. Baseline conservata: `0f4d2e02`. Questo lavoro implementa W1 del
kickoff V2 e la base UI1–UI2 della specifica V3; i gate del vecchio W1 non vengono
trasferiti alla nuova interfaccia. Il pacchetto HTML resta uno studio di progetto.

## Risultato verificato

- Fleet è la home: rail compatto, matrice CPU/RAM/GPU/rete/stato/età, filtro,
  ordinamento, selezione per ID, anteprima opzionale, timeline CPU ricevuta.
- Host Monitor: CPU/core, GPU individuali, memoria/swap, interfacce, filesystem,
  processi e diagnostica nello stesso workspace. Nessuna metrica I/O o servizio
  ricostruita da dati assenti. I dettagli precedenti restano raggiungibili.
- Investigate usa assi 0–100%, tempo di ricezione, cursore sincronizzato e tabella
  degli ultimi dieci campioni. Il range resta cinque minuti realmente disponibili.
- Setup: CRUD locale/SSH/inventario, conferma rimozione, errori inline, ID immutabili;
  tema/densità/inspector/rail persistenti. Nessuna configurazione testuale obbligatoria.
- Registro SQLite versionato, commit atomici e controllo revisione contro la perdita
  di modifiche fra finestre. Import legacy idempotente, file originale e storico
  preservati; nuove installazioni generano l'ID locale.
- Eventi engine per ID, sostituzione dei collector fuori dal ciclo UI, arresto/join
  dei worker e reap dei figli SSH. Flotta vuota/solo inventario non gira a vuoto su
  un canale disconnesso: difetto riprodotto e coperto da regressione.
- Nessun cambio framework; SQLite è già parte dello stack e viene riusata per il
  registro. Nessun nuovo servizio, listener, VPN o deploy remoto.

## Prove automatiche

`scripts/check.sh`: **25 test Rust** (21 desktop, 3 agent, 1 store), fmt/check e
clippy con warning trattati come errori. Resta il precedente avviso upstream di
compatibilità futura di `block 0.1.6`.

**72 casi grafici**: 12 viste × 1280×800 / 1440×900 / 3840×2160 × light/dark.
Ogni caso viene renderizzato due volte con dati e orologio di serie congelati.
**9 scenari grafici mirati** verificano metriche/glifi, picking del renderer
conservato, tastiera/filtro processi, selezione, cambio tema, form e virtualizzazione.
Il nuovo test di scroll mantiene la cache reale dei widget: arriva all'ultimo dei
500 host, poi applica un filtro e verifica che il risultato sia visibile. Cambiare
solo l'ID del widget non resettava lo scroll; ora si usa un'operazione esplicita.

Le popolazioni 0/1/2/5/16/32/500 hanno fixture dichiarate e numero di righe costruite
proporzionale al viewport. Le prove non qualificano 500 stream SSH simultanei.
Contrasto di testo/stati/serie verificato in entrambi i temi. Candidati e contatti
visivi ispezionati, con controlli mirati a dimensione piena per Fleet/Host/Setup.
Sono evidenza di regressione, non approvazione visiva indipendente dell'utente.

Artifact ignorati: `.local/qa/v3/`. `delivery-visual.log` contiene la matrice e gli
otto scenari precedenti; `virtual-scroll.log` il nono, aggiunto separatamente.
`final-interactions.log` ripete i percorsi interessati dalle ultime correzioni.
`checks-complete.log` registra i check finali e la regressione flotta vuota.

```sh
scripts/check.sh
MOONITOR_QA_DIR="$PWD/.local/qa/v3/reproduce" scripts/qa-ui.sh
scripts/package-macos.sh
```

## Dati reali e pacchetto

Profilo nuovo isolato, due avvii senza TOML: CPU/RAM reali in SQLite, un solo nodo
locale e identità identica dopo il riavvio. Le prove CRUD verificano aggiunta,
rinomina senza cambio ID, preferenze, rimozione confermata e file storico intatto.

La build release con il profilo esistente importa **5 nodi**, conserva tutti gli
ID legacy e riceve campioni reali da **4 collector**; il nodo solo inventario resta
non strumentato. Nessuna installazione o modifica dei servizi remoti. La finestra
reale macOS è stata ispezionata; catture private restano fuori Git.

Eseguibile di build prima della firma, SHA-256:
`c568edb4918e23da020b689054c56ad7e078e5665f976c641dcc9270abf780a4`.
Il pacchetto ad-hoc viene verificato con `codesign`; manifest e ricevuta distinguono
hash di build e binario firmato. Namespace progetto `org.moonitor.desktop`; nessuna
fleet privata nel bundle né copia della fleet sviluppatore nelle nuove installazioni.

## Prestazioni: misure e limiti

Build finale, Fleet Deep Space 1440×900, cinque nodi reali, logging normale e cattura
automatica all'avvio: **1,29% di un core logico**, **132,91 MiB RSS massimo**, finestra
30,26 s (`performance-delivery.json`). La memoria include la cattura PNG. È una
finestra esplorativa, non la qualifica del budget né un benchmark di inferenza.

Il confronto preliminare sulla candidata `a87f0590` registra baseline 2,38% / 1,39%
e candidata 8,57% / 1,63% / 4,71%, con variabilità e attività concorrente non isolata.
Una prima baseline è **invalida**: il processo non era più residente e `ps` aveva
azzerato RSS/CPU. I risultati grezzi sono conservati, non mediati selettivamente.
Il misuratore ora rifiuta processi terminati e contatori regressivi; una prova con
processo di breve durata conferma che non produce un risultato valido.

Non si conclude una regressione/miglioria causale da queste finestre. Restano aperti
profili foreground/background separati, soak e confronto paired durante inferenza
su macOS/Linux. Non trasferire le vecchie misure AI alla nuova build.

## Scostamenti ancora aperti

- W1 è il primo percorso funzionante, non tutta la RFC. Setup usa sezioni operative
  nella stessa pagina; indice di gestione completo, discovery e agent lifecycle
  dipendono da W2/W3. Nessun interruttore decorativo per backend assenti.
- Matrice Fleet virtualizzata; inspector opzionale fisso 300 px. Ridimensionamento,
  colonne configurabili/pinned, gruppi e layout salvati richiedono ulteriori lavori.
- Budget di raccolta conservativo: 32 stream simultanei, locale prioritario; nodi
  aggiuntivi mostrano capacità esaurita, distinta da errore rete. Il registro ha
  budget serializzato 1 MiB. Occorre qualificare uno scheduler prima di alzare il
  budget dei collector; i test grafici 500 host non lo sostituiscono.
- Storico resta schema riepiloghi: niente serie per-core/GPU individuale/eventi,
  request traces, servizi OS/container o energia integrata. Processi senza start
  identity e sottoinsieme limitato esplicitamente dichiarato.
- Questa matrice grafica è macOS/Metal. CI statica/unit macOS/Linux separata;
  nuova qualifica grafica Linux, a11y assistiva, localizzazione completa, 1920×1080
  dedicata, sleep/wake e installazione pulita distribuita restano aperti.
- Esempi, inventario e screenshot correnti resi generici; etichette private nei
  report storici generalizzate, originali preservati nel checkpoint. Questo non
  cancella la storia Git. Nessun cambio visibilità, riscrittura storia o licenza.

Prossima dipendenza: **W2 Controller / protocollo / lifecycle agent**, dopo review
di questo percorso. La comparazione storica completa resta UI3/W4.
