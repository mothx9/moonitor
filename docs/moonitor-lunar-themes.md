# Moonlight e Deep Space

> Historical revision. The [V3 contract](moonitor-v2/product-requirements.md) replaces the former layout and session-only preferences. Measurements below refer to that earlier build.

23 ottobre 2026. Cambio di direzione visiva richiesto dall'utente: l'identità lunare
del marchio sostituisce i temi verdi precedenti. Questa decisione aggiorna la
proposta cromatica del blueprint; non cambia framework, roadmap o modello dati.

## Palette e applicazione

| Ruolo | Moonlight | Deep Space |
| --- | --- | --- |
| Canvas | `#F2F3F8` | `#0B0E18` |
| Superficie | `#FFFFFF` | `#141927` |
| Testo | `#24293A` | `#EDF0FA` |
| Testo secondario | `#5B667A` | `#A5B0C7` |
| Accento | `#5757BF` | `#B6B2FF` |
| CPU / RAM / GPU | pervinca / ametista / blu | pervinca / ametista / azzurro |

`crates/moonitor-desktop/src/palette.rs` centralizza i token semantici. Widget,
grafici, barre, tooltip, etichette e terreno della Lab Map condividono la palette.
Lo shader riceve terreno e selezione come uniform, senza colori di tema duplicati
nel WGSL. Nessuna nuova dipendenza, texture, animazione o subscription periodica.

Online usa azzurro, stale ambra, unreachable corallo, connecting blu e
uninstrumented grigio; le etichette di stato restano esplicite. La selezione usa
pervinca. I materiali fisici dei dispositivi mantengono le differenze hardware.

Settings presenta due opzioni con indicazione **Attivo**. Il cambio conserva nodo,
tab risorse, camera e modalità 3D/elenco. Le preferenze restano di sessione, come
prima; `--theme light|dark` resta compatibile. L'originale della luna non cambia.

## Verifica

```sh
scripts/check.sh
MOONITOR_QA_DIR="$PWD/.local/qa/lunar/visual" scripts/qa-ui.sh
cargo build --release --locked -p moonitor-desktop -p moonitor-agent
```

- Contrasto automatico: almeno 4,5:1 per testo/stati su canvas, card, selezione e
  hover; almeno 3:1 per le tre serie sui grafici; contrasto dei testi della sidebar
  e degli avvisi verificato separatamente. Non equivale ad accessibilità completa.
- Test di interazione: Deep Space → Moonlight → Deep Space senza perdere contesto,
  con catture della selezione attiva e ritorno alla mappa.
- Matrice esistente: 10 viste × 3 viewport × 2 temi; rendering ripetuto con clock
  e osservazioni congelati. Test di glyph, picking, filtri, tastiera, scroll e flotta
  variabile conservati. I nuovi colori hanno baseline separate dalle precedenti.
- Artifact e log ignorati: `.local/qa/lunar/`. I file `fixture-*` contengono dati
  sintetici dichiarati di test; le preview nel README provengono dalla build live.

## Esito su macOS / Metal

- `fmt`, `check`, `clippy -D warnings` e **19 test Rust** passati.
- **66 invocazioni grafiche passate**: 60 casi della matrice e sei test grafici,
  incluso il nuovo cambio tema durante la sessione. Ogni caso della matrice viene
  renderizzato due volte per verificarne la stabilità.
- Overview live ispezionata in entrambi i temi, con quattro nodi monitorati e un
  dispositivo censito senza agente. Preview aggiornate in `docs/assets/` e README.
- README: rendering GitHub, 33 link locali, cinque badge e struttura HTML verificati.
- Build release, SHA-256 dell'eseguibile prima della firma:
  `6575c33db74be6a8e2e99cfd0cdc2fc372c378864077ddf4984e4b1b03233060`.

Misura esplorativa Deep Space, Overview 1440×900, cinque nodi reali, collector locale
incluso, logging normale e vecchio desktop chiuso: **1,74% di un core logico** e
**112,08 MiB RSS massimo** in 30,38 secondi (`performance-normal-dark.json`).
La prima osservazione con diagnostica verbosa e un altro desktop ancora aperto
misurava 3,06% e 133,89 MiB: le condizioni differiscono, quindi non è un confronto
causale né un benchmark A/B. Nessuna delle due finestre è una garanzia generale.

Nella sessione diagnostica di 45 secondi: **6 render della scena, 1 passaggio ombre,
119 composizioni**. La geometria resta di 29.130 vertici; nessuna animazione continua
è stata aggiunta. Log e misure sono conservati in `.local/qa/lunar/`.

Questa qualifica grafica è macOS/Metal. La matrice Linux/Vulkan non è stata ripetuta
per il cambio palette; le evidenze W1 precedenti restano distinte. Restano aperti
approvazione visiva dell'utente, accessibilità assistiva completa e prove di lunga
durata. Collector e workload AI non cambiano; i benchmark di inferenza precedenti
non vengono presentati come una nuova misura di questa build.
