# Rollout e gate V3

Baseline `0f4d2e02`, CI macOS/Linux conclusa con successo. Backup Git/bundle/config
in `.local/checkpoints/v3-baseline`. Lavoro diretto su main per istruzione utente.

| Fase | Dipendenza | Verifica / gate |
| --- | --- | --- |
| UI0 | documenti forniti + audit | contratti Fleet/Host/Setup e dati assenti |
| W1 / UI1–UI2 | UI0 | shell nativa, flotta reale, cockpit, CRUD e persistenza |
| W2 Controller | registry W1 stabile | raccolta con UI chiusa, servizio/revoca/protocollo |
| W3 discovery | identità/trust W2 | LAN/VPN/manuale senza trust implicito |
| W4 / UI3 | storage evoluto | storia per componente, servizi/eventi, confronto |
| W5 / UI5 | gate precedenti | privacy, licenza, packaging, a11y, soak, release |

## Prove W1

- `fmt`, `check`, `clippy -D warnings`, unit/contract tests.
- Registro: primo avvio, import idempotente, rollback su errore, ID stabili,
  creazione/modifica/rimozione, riapertura, preferenze, conflitto revisioni.
- Fleet 0/1/2/5/16/32/500: widget bounded, ordinamento stabile, filtro, selezione,
  rimozione selezionato, scroll fino all'ultimo; 5 stati sopra il fold.
- Metriche: zero/assente/unsupported/denied/stale, GPU multiple, unified memory,
  process CPU >100 valido, malformed/NaN/range rifiutati, gap temporali.
- Renderer macOS: Fleet/Host/Setup/metriche/processi/investigate a tre viewport,
  light/dark, screenshot ripetuti; ispezione visiva e interazioni. Linux distinto.
- Profilo nuovo senza TOML: collector locale reale, save/restart. Config esistente:
  stessi ID e agenti, nessun deploy remoto, archivio storico preservato.
- CPU/RSS prima/dopo con stesso carico, viewport e periodo. Nessun benchmark
  inferenza dichiarato per analogia: se non ripetuto, riportare il limite.
- README con fixture esplicite, nessun screenshot personale pubblicato; scan file
  correnti, evidenza separata del rischio storico. Nessun cambio visibilità GitHub.

Esiti e limiti saranno registrati in `qualification.md` dopo le prove; screenshot
sono candidati di review, non approvazione visiva automatica.
