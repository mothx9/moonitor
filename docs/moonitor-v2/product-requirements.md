# Moonitor V3 — contratto del primo rilascio verticale

24 ottobre 2026. Baseline: `0f4d2e02`. La specifica UI V3 fornita dal proprietario
prevale sulla precedente composizione W1. I nomi delle wave nella RFC V2 non sono
i gate del vecchio blueprint: qui **W1 V2 / UI1–UI2 V3** indica il primo percorso
Fleet → Host → Setup → riavvio. Non implica controller, discovery o nuova telemetria.

## Audit verificato

| Area | Codice alla baseline | Intervento |
| --- | --- | --- |
| Desktop | Iced 0.14, wgpu, quattro crate; sidebar 220 px e Overview a card | Rail compatto, matrice Fleet, cockpit Host |
| Configurazione | `config.rs`: TOML, ID locale personale, massimo 32 nodi | Registro persistente, ID stabili, CRUD grafico, import conservativo |
| Preferenze | tema solo RAM | SQLite versionata; tema, densità e inspector persistenti |
| Engine | uno stream SSH per nodo, eventi indicizzati, desktop proprietario | ID negli eventi, riconfigurazione con arresto dei worker precedenti |
| Fonti | sysinfo, NVML, IOKit, Ollama `/api/ps` | riuso; nessun runtime o metrica simulata |
| Storico | riepiloghi 5 s / 7 giorni, massimo GPU | invariato; 5 minuti visualizzati, aggregazione dichiarata |
| Processi | sottoinsieme limitato, CPU per core, niente start identity | etichetta esplicita; nessuna falsa identità/lifecycle |
| Sensori | GPU distinte; potenza senza component ID | niente attribuzione board a GPU specifica se più GPU |
| Pubblicazione | inventario, esempi e immagini personali; bundle namespace personale | contenuto corrente generico; storia e visibilità restano private |
| QA | 19 unit test, 66 invocazioni grafiche; CI baseline completata con successo | nuove prove sui flussi V3; vecchie evidenze non trasferite |

Backup del repository e bundle: `.local/checkpoints/v3-baseline/`. Nessun file
privato viene eliminato. L'archivio storico conserva le chiavi nodo importate.

## Accettazione del percorso

1. Profilo nuovo: un solo computer locale, CPU/RAM reali, nessun file da scrivere.
2. Fleet rende confrontabili cinque stati a 1280×800; filtro, ordinamento,
   anteprima, apertura Host e ritorno mantengono selezione per ID.
3. Host mostra CPU/core, memoria, GPU individuali, interfacce, filesystem e processi;
   capacità assenti spiegate. Nessun grafico inventato per I/O o singole GPU.
4. Setup aggiunge/modifica/rimuove host locali, SSH o censiti. Salva atomicamente;
   riapertura preserva ID e preferenze. Rimozione non cancella lo storico.
5. L'import TOML è idempotente, conserva gli ID e non modifica il file originale.
6. Credenziali e host key restano nel client SSH di sistema. Nessuna installazione
   remota automatica; sconosciuti/rifiutati restano errori leggibili.

## Non ancora implementato da questo percorso

Controller autonomo, agent service/mTLS, discovery, pairing, install/update agent,
servizi/container, richieste AI, timeline eventi persistita, serie per componente,
7d interrogabili nell'interfaccia, localizzazione completa e accessibilità assistiva.
Le schermate mostrano questi limiti senza controlli finti.
