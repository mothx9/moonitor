# MOONITOR 2.0 — Product & Engineering RFC

**Stato:** proposta di architettura, UX e roadmap, NON implementata.
**Baseline verificata:** `mothx9/moonitor`, branch `main`, commit `0f4d2e02dd5968af35c6bc31f7148fd40f705329` del 23 ottobre 2026; screen recording fornito il 24 ottobre 2026.
**Destinazione:** progetto open source general-purpose, local-first, desktop-native, multi-host e AI-aware.
**Vincolo editoriale:** nessun inventario personale, alias, indirizzo, endpoint, identificativo o screenshot reale deve entrare negli asset di distribuzione.

## 0. Executive decision

Moonitor deve diventare un **system observability product** e non una raccolta di schermate hardware. Rifare integralmente IA/UX e configurazione; conservare la base Rust verificata (collector, letture con provenance, SSH rigoroso, SQLite, capacità native) mentre si separano controller, agenti e UI. Nessun cloud obbligatorio; installazione su una sola macchina sufficiente per funzionare; possibilità di un controller persistente sempre attivo.

### Principi non negoziabili
1. Zero obblighi TOML/YAML/SSH-config: tutto l'onboarding, la modifica e la rimozione degli host deve funzionare attraverso l'app. Config testuale solo come esportazione facoltativa di amministrazione.
2. Network agnostic: LAN, VPN overlay, SSH e aggiunta manuale sono percorsi alternativi verso identità stabili di agenti, non provider da implementare da zero.
3. Discovery != trust != pairing != monitoring. Nessun device scoperto è automaticamente autorizzato.
4. Telemetria esatta e contestualizzata: NULL/unsupported/denied/stale non sono zero; timestamp, scope, source, unità, qualità e capacità dichiarata.
5. Flotta-first: la Overview deve rendere leggibile simultaneamente lo stato di 1, 5, 20+ macchine senza navigazione obbligatoria fra host.
6. Storage e collection non dipendono dal processo grafico quando la modalità Controller persistente è attiva.
7. Nessuna azione remota pericolosa per impostazione predefinita; agent read-only; ogni escalation opt-in e per collector.
8. Non introdurre dati sintetici nelle schermate produzione; dati demo esclusivamente in modalità distinta, esplicita e dichiarata.
9. Design instrument-grade, non dashboard vetrina, non scena 3D decorativa.
10. Il repository diventa pubblico solo dopo scrub di sorgenti, asset, documentazione e storia Git.

## 1. Audit snapshot (evidenze)

- Cargo workspace: `moonitor-core`, `moonitor-agent`, `moonitor-store`, `moonitor-desktop`; Rust 1.96/2024, Iced 0.14 + wgpu.
- `agent/src/main.rs`: JSONL su stdout; servizio e listener non presenti, nessun lifecycle background autonomo.
- `desktop/src/engine.rs`: desktop genera collectors locali e processi SSH, persiste riepiloghi in SQLite ogni ~5s, fino a 7 giorni, e li chiude quando termina.
- `desktop/src/config.rs`: nodi Local/Ssh/Unavailable; un unico local, max 32, alias SSH.
- `config/fleet.example.toml` e `docs/inventory.md`: esempi/inventario riferiti a specifiche macchine private, NON idonei alla pubblicazione.
- `desktop/src/main.rs` default ID di prova riferito a un tipo personale di computer; `scripts/package-macos.sh` copia in supporto applicazione la configurazione esemplificativa e contiene bundle identifier legato all'autore: da generalizzare.
- `agent/src/lib.rs`: CPU/mem/processi/dischi/rete/sensori + NVML o IOKit; Ollama rilevato via `http://127.0.0.1:11434/api/ps`, limite 300 ms. La stringa del localhost di Ollama è un default di integrazione, non una scoperta generica; va resa configurabile.
- `store/src/lib.rs`: schema rows `(node, received_ms, source_ms, cpu, memory_percent, gpu, gpu_board_watts)` perde per-device, eventi e qualità.
- UX nel video: grandi pannelli vuoti, overview focalizzata su un singolo host, mappa isometrica imprecisa, Fleet ridondante, Settings privo di CRUD, selettore temi solo sessione.
- Documenti W0/W1 presenti: preservare metric semantics, regole qualità, WGPU caching e test che hanno già intercettato regressioni; non riportare i test locali precedenti come prova di qualità del redesign non ancora realizzato.

## 2. Product information architecture v2

### Navigazione primaria

| Area | Prima domanda | Componenti |
|---|---|---|
| Overview | Tutta la flotta è sana, ora? | summary semantici, fleet grid, eventi, top workload, fonte/freshness |
| Fleet | Quali host e come si connettono? | inventory table/card, filtri/gruppi, stato agent, add/remove, multi-select |
| Services | Quali processi/servizi/container sono vivi? | service inventory, health probe, associati a nodo oppure esterni |
| Workloads | Che cosa gira e dove? | CPU/GPU workloads, runtime AI, modelli, job, richieste se strumentate |
| Explore | Che cosa è successo? | time series, compare nodes, brush, replay, export |
| Events & Alerts | Che cosa richiede attenzione? | timeline, severità, rule engine, ack, mute, origine |
| Settings | Come si configura? | onboarding, agent, networks, sources, storage, UI, privacy, update |

Energy può vivere nella top-level IA quando le letture sufficienti e non ambigue lo giustificano; prima vive come sezione in Explore/Host Detail e card con specifica copertura.

### Overview 1280×800, obiettivo above the fold
- **Topbar 48–56 px:** simbolo e parola Moonitor, titolo corrente, search/command palette, time range, live/freshness, settings/add host.
- **Sidebar collassabile:** 56–64 px icon-only, 176–196 px estesa. Non mostrare due volte l'inventario.
- **KPI strip:** 4–6 indicatori realmente significativi (online/known, alert attivi, GPU osservate, processi/workload noti, potenza board misurata con coverage). Non inventare total watts della flotta.
- **Fleet grid ad alta densità:** un host per riga; stato, CPU, RAM, GPU e memoria, rete, dischi, uptime/freshness; mini-sparkline soltanto quando utile. Almeno cinque host visibili senza scroll su viewport target.
- **Resource trends:** comparazione multi-node, CPU/GPU/RAM selezionabili, scala uniforme per dati omogenei e gaps per mancanti.
- **Events / attention:** eventi e incidenti recenti con causa evidenziata, non 3D.
- **Gerarchie:** clicking host apre Node Detail; Shift/cmd multi-selection alimenta compare; context menu a basso rumore.
- Un device in stato inventory-only NON compare come 'offline' né come 'agent failed'; CPU non supportata è `—` con motivo.

### Node detail
Header sticky: alias utente, nome hardware, OS, connessioni LAN/VPN disponibili, agent/version, ultima lettura e latenza effettiva. Tabs: Summary, Compute, GPU, Processes, Storage, Network, Sensors, Services, AI, History, Diagnostics. Mostrare solo tabs con senso per capabilities; permettere comunque verifiche di fonte/assenza.

### Service detail
Endpoint/protocol, health state vs transport reachability, status history, response time, source probe, uptime osservato, last check, incidenti. Se è collegato a un host, drilldown; se è un endpoint remoto senza agent, non fingere di avere metriche interne.

## 3. Design system (v2 rispetto a tema corrente)

- Preservare l'icona di luna e una wordmark rigorosa; separare la luna dal sistema di navigazione. No slogan invasivi dentro le pagine.
- Theme **Deep Space**: canvas scuro vicino a `#0B0E18`, superficie `#141927`, bordi `#292F42`, tipo primario argento `#EFF1FA`, accento selezione pervinca `#AAA7F5`; CPU/ram/gpu serie percepibili e distinte. Valori indicativi da valutare con WCAG e gamut reale.
- Theme **Moonlight**: canvas lunare freddo, superfici bianche, un solo bordo neutro, tabelle non invase da card. Definire tutti i token semantici in `palette.rs` e token UI (space/radius/typography/z-index).
- Grid 4 px: 4/8/12/16/20/24; table row Compact 36–44, Comfortable 48–56. Raggi 6–10 per controlli/tabelle e 12 per grandi contenitori. Density mode persistita nel profilo.
- Typography: UI 12–14, titoli 18–24, KPI 22–28, tabular numerals, mono piccolo per rete/ID/source. Contrasto minimo 4.5:1 testo e 3:1 grafica significativa; tastiera, focus, contrasto, motion preference.
- Niente ampi blocchi vuoti, fill-effect, ombre decorative, gradienti non semantici, schede con due righe di testo e 200 px di altezza. Allineamento preciso di etichette, unità e colonne.
- Data visualization: scale esplicite, zero-base quando sensato, tooltip con unità/time/source, brushing/zoom, crosshair sincronizzato su metriche, marker per errori e connessioni, data gap non interpolati, retention dichiarata, no percentuali ingannevoli fra hardware diversi.
- Topology: preferire vista **2D semantica** a camera 3D. Archiviare il rendering 3D attuale nel codice storico senza mostrarlo di default; reintrodurlo solo se topologia, network path e dipendenze effettivamente osservate aggiungono valore e se benchmark/a11y passano.
- GUI native Iced + wgpu è baseline provvisoria, non dogma. Spike comparativo solo qualora misure riproducibili evidenzino limiti su qualità tipografica, input, a11y, performance o piattaforma. No riscrittura framework simultanea al data model senza ADR.

## 4. Deployment / process boundaries

```
Moonitor Desktop (Iced)
  ⇅ local authenticated IPC
Moonitor Controller (headless user service or local daemon)
  ├── config API / host registry / capability inventory
  ├── Discovery Providers (LAN/VPN/manual)
  ├── Enrollment/CA / auth / authorization
  ├── Scheduler & Ingest / normalization / rules
  ├── Query & Event API
  └── SQLite WAL / rollups / retention / migrations
           ⇅ versioned, authenticated Agent Protocol
       Moonitor Agent on monitored hosts
           ├─ Native OS collectors
           ├─ GPU/energy/sensor adapters
           ├─ processes/services/containers
           └─ AI/runtime adapters
```

### Deployment profiles
1. **Portable:** desktop avvia il controller locale come processo gestito; funziona offline, storico finché attivo. Nessun daemon né ascolto su rete richiesto.
2. **Local persistent (recommended):** controller installato come `launchd` user agent o `systemd --user` con policy di sopravvivenza al logout dichiarata; raccolta attiva con UI chiusa, nei limiti del runtime OS/energia/sospensione.
3. **Hub dedicated:** controller su server headless Linux/ARM64; desktop si connette via rete autenticata. È opzionale, mai obbligatorio.
4. **Remote agent:** Windows service in milestone distinta; prime release agent Linux x86_64/aarch64 e macOS arm64/x86_64 se testate; risorse e permessi dipendono dalla piattaforma.

Eliminare la dipendenza `Engine::start` dalla sola UI, riusare collection e store. Procedura di migrazione from `fleet.toml`: import guidato idempotente e rollback prima di rendere TOML opzionale.

## 5. LAN + VPN discovery: design reale

### Una differenza fondamentale
mDNS/DNS-SD (RFC 6762/6763) consente zero-conf nella stessa rete link-local; non attraversa automaticamente router, VLAN e VPN overlay. I prodotti VPN mettono a disposizione routing e talvolta peer directory ma NON un protocollo uniforme di discovery di applicazioni. Quindi discovery è federato in provider con un percorso di verifica comune.

### Provider e roadmap
| Provider | Inventory candidato | Endpoint | Livello proposto |
|---|---|---|---|
| LAN mDNS/DNS-SD | `_moonitor-agent._tcp.local` (agent che annuncia) | A/AAAA+SRV/TXT | P0 |
| LAN manuale | IP, FQDN, host:port, subnet scelto | TCP/TLS + SSH | P0 |
| Tailscale/Headscale | `tailscale status --json`, API locale del client, MagicDNS | tailnet IPv4/IPv6/FQDN | P0 |
| WireGuard puro | `wg show` indica peer/AllowedIPs, **non** discovery servizi affidabile | IP/routing, manual seeds | P0 transport, P1 hints |
| NetBird | `netbird status -d --json` | overlay peer IP/DNS | P1 |
| ZeroTier | client local API/peer list, con permesso | network-managed IP | P1 |
| OpenVPN | routing locale/route inventory | IP/manual | P1 transport |
| Nebula | overlay IP e Lighthouse/host mapping se esposto | IP/manual | P2 |
| Cloudflare One/WARP/Tunnel | private routes se organizzazione ha permessi | private network IP/DNS | P2 transport |
| Custom VPN/IP network | peer registry esterno facoltativo, manuale | TCP/IP | P0 generic |

**Non implementare Tailscale, WireGuard, OpenVPN, ecc. dentro Moonitor.** Si riutilizzano i client/tunnel esistenti del sistema operativo; i provider sono discovery/diagnostics adapters e suggeritori di endpoint. Moonitor non diventa una VPN e non raccoglie chiavi private di WireGuard.

### Discovery pipeline
1. Per ogni scheda rete: rileva scope LAN/VPN, route, multicast capabilities, no packet scan invasivo in automatico.
2. Ottieni candidati per mDNS, provider VPN locale, endpoint digitato o subnet scan esplicitamente autorizzato e rate-limited.
3. Normalizza a `DiscoveryCandidate {method, claimed identity, addresses, hints, seen_at, ttl, confidence}`; considerali non attendibili.
4. Determina raggiungibilità con probe del protocollo Moonitor a timeout e concurrency limit; non dedurre agent dall'apertura di una qualsiasi porta.
5. Mostra elenco per revisione; pairing esplicito dell'utente; valida certificate fingerprint / code / CA.
6. Post-pairing ricevi `NodeIdentity` stabile, `AgentHello` versionato e capabilities; deduplica host che appaiono via LAN e VPN usando identità crittografica, NON hostname/IP.
7. Persisti diversi endpoint per stesso nodo, stato per path/RTT e fallback automatico con anti-flapping/hysteresis. Una VPN peer discovery non garantisce che Grants/ACL e firewall autorizzino la porta del servizio.

### DNS-SD target
Annuncio facoltativo e minimizzato, esempio service `_moonitor-agent._tcp.local.`; TXT v=1 + schema version + discovery opaque instance hint, niente hostnames sensibili, chiavi, token o dettagli macchina. Nessuna fiducia accordata all'annuncio. IPv4 + IPv6, interfacce multiple e network permission esplicita. TTL, goodbye packet e update supportati; fallback a unicast DNS-SD dove amministrato.

### Manual host/add service
Host wizard: label libero, indirizzo IPv4/IPv6/FQDN, porta, prova connessione, identity/pairing, metodo installazione, test telemetria, salva. `Manual Service`: HTTPS/TCP healthcheck, Prometheus/OpenMetrics endpoint, OTLP, SNMPv3 con credentials in secret store, eventuali exporter; un servizio non-agent è monitorato nei limiti del protocollo scelto.

## 6. Protocollo Moonitor Agent v1

**Proposal:** Protobuf + gRPC (`tonic`) su TLS mutuo e versioning esplicito; trasporti diretti TCP/IP su LAN e VPN; SSH tunnel come alternativa; controller opzionalmente riceve stream outbound su trust validato per reti senza inbound. Il protocollo può essere interno ma la sua specifica e il compatibility contract sono pubblici e testati.

Metodi logici: `GetIdentity`, `GetCapabilities`, `Health`, `SubscribeObservations`, `GetMetricDescriptors`, `ListComponents`, `ListServices`, `GetDiagnostics`, `SetReadOnlyCollectionPolicy` (solo configurazione consentita), `GetAgentVersion`. Stream: `AgentHello`, `ObservationBatch`, `EventBatch`, `Heartbeat`, `Ack` e `Error`; `sample_seq`, `agent_boot_id`, `sampled_at`, `received_at`, `schema_version`, `compress`, `max_message_bytes`, optional resume token.

Separare identità persistente agente, boot ID, process ID, host display name. Schema negotiation con N-1 compatibilità definita; no silent coercions. Limiti sul payload e sulle serie/cardinalità, retry exponential backoff + jitter, queue bounded, failure counters. Opzionale spool cifrato e bounded su disco con replay e dedup, evitando che un laptop saturi la propria memoria in caso di perdita della rete.

### Trust & enrollment
- Alla prima installazione ogni agent crea una coppia di chiavi e prova possesso. Il controller possiede un'identità/CA locale dedicata (o alternativa trust-list) e autorizza via code/token monouso breve + fingerprint confermato, oppure bootstrap SSH verificato.
- mTLS dopo enrollment, device certificate rotation, revocation, revoke controller, recovery procedure, audit event. Un peer VPN presente non è automaticamente trusted come agente.
- CLI-only option per server headless: l'utente vede fingerprint/one-time token nel terminale o tramite canale SSH verificato; non aprire endpoint di enrollment sulla rete pubblica per default.
- Agent di base read-only e non-root; `sudo` solo per install/uninstall service o collector separati realmente necessari. Accesso al socket Docker è altamente privilegiato, richiede consenso e hardening.
- Controller local IPC autenticata/accessibile solo all'utente; GUI non espone Web API non autenticata. No una porta globale bind `0.0.0.0` senza scelta consapevole.

## 7. Onboarding: UX sequenziale, autosufficiente

1. **Welcome & mode:** Monitora questo computer / Monitora una rete / Apri un controller remoto / Importa configurazione precedente.
2. **Local storage:** percorso, retention default, cifratura/segreti, modalità background; default ragionevole senza richiedere parametri.
3. **Local agent:** test automatico OS/GPU/sensors e capability table; abilita servizio persistente solo se utente lo vuole.
4. **Networks:** schede rilevate, LAN discovery toggle, Tailscale rilevato / altro provider / nessuna VPN; presentare cosa verrà letto e quali permessi servono.
5. **Discover:** visualizza candidati raggruppati per source, online reachability, installazione agent rilevata, sistema operativo quando disponibile; permesso per eventuale scan subnet.
6. **Enroll:** scegli peer, pairing, fingerprint trust, procedura guidata per deploy agent: SSH assistito (preferito), installazione manuale, package, QR/setup code solo dove sensato; preview dei comandi e dei privilegi.
7. **Validation:** handshake, remote version, CPU/RAM/GPU, timings, capability result, security status. Errori actionable (ACL, SSH, firewall, OS permission, certificate, wrong arch) e riprova.
8. **Finish:** overview live, 'Add host' sempre disponibile, revisione privacy/retention, eventuali aggiornamenti.

**Non imporre tutti i passi:** saltabili i nodi remoti, configurazione esportabile/importabile, welcome non riappare dopo completamento, ripristino progressivo se chiusura improvvisa. Qualsiasi voce dei wizard resta modificabile in Settings. Supportare prima installazione anche con zero host remoti e completamente offline.

## 8. Settings: struttura concreta

- General: language, locale, updates, startup, system tray, minimize/close behavior.
- Appearance: Moonlight/Deep Space, Auto, scale, compact/comfortable, reduced motion, chart units.
- Fleet: CRUD, tags/groups, reorder, aliases, select-by-capabilities, pause/resume, removal/revocation.
- Discovery & Networks: LAN interfaces, mDNS, VPN provider detection, peer list permission, subnet manual scan, route/path priority, reconnect/failover.
- Connections: endpoint per agent, transport (direct/ssh/outbound), test handshake, latency, proxy, last failure, certificate chain.
- Agents: installs, per-host version/health/capabilities, updates/rollback, rotation/revoke, runtime permissions.
- Collectors: per-source enablement and frequency, permissions, data scope, detection, unsupported reasons and collector diagnostics.
- Services & Integrations: endpoint check, Docker/Podman, systemd/launchd, exporter, OTLP, Prometheus, optional SNMPv3/Redfish/NUT, AI runtimes.
- Monitoring & History: sampling, rollups, retention by tier, storage quotas, alerts, export, wipe/delete history, backlog/spool.
- Alerts & Notifications: rule builder, channels, quiet hours, dedup, severity, ack/mute, notification permission.
- Security & Privacy: trusted devices, key renewal, data collection scope, local API, remote exposure, telemetry export/redaction.
- Diagnostics: health of controller/database/agents/discovery, collectors, errors, support bundle sanitized.
- About & Release: version/commit, open-source licenses, build signature, update checks, changelog.

Persist preferences/settings in **versioned SQLite** (or equivalent typed transactional local persistence), not scattered hand-edited YAML. Network keys/passwords in OS keyring/Secret Service with controlled file fallback, never plaintext DB. Atomic saves with immediate UI feedback and explicit restart only when truly required.

## 9. Telemetry coverage matrix

| Domain | Baseline oggi | Moonitor 2 target | Care points |
|---|---|---|---|
| CPU | total/per-core % | freq, load, package, scheduler pressure/PSI, throttling if known | percent definition and core normalization |
| Memory | used/available/swap | pressure, caches, per-process resident, OOM events | unified GPU memory not extra RAM |
| GPU | NVML + best-effort IOKit | per-GPU, GPU mem, power, clocks, temp, enc/dec, throttle, ECC, PCIe/NVLink if supported | capability matrix NVIDIA/AMD/Intel/Apple |
| Disk | capacity | IO throughput, ops, latency, mount/device map, SMART/NVMe health with permission | never falsely infer SMART from capacity |
| Network | per-interface B/s, errors | packets, errors/drops, link speed, RTT, VPN path, reconnect history | differentiate peer state and app reachability |
| Processes | bounded top candidates | sortable bounded list, CPU, RSS, lifecycle, selected process history | process is not automatically AI workload |
| Services/containers | not instrumented | systemd/launchd, Docker/Podman opt-in, generic probes, status transitions | least privilege; no default access to Docker socket |
| Sensors | temperatures/battery | fan, voltage, battery/UPS, optional NUT/IPMI/Redfish | explicit permissions and vendor limits |
| Power | GPU board W | scoped W, validated Wh intervals, device meter opt-in | no summation overlapping boundaries |
| Local AI | Ollama loaded models | Ollama, vLLM, llama.cpp, MLX, optional YVEX, request metrics if runtime exposes them | loaded != in use; no inferred TTFT |
| History/events | 5s summaries/7d | per-component series, events, query windows, alerts, provenance | cardinality quotas, clock skew, gaps |

**Prioritizzazione:** P0 cross-platform CPU/RAM/GPU/disk/network/health/history; P1 richer Linux GPU/NVML and disk I/O, service health, Docker opt-in; P2 specialized AI, energy, advanced vendor collectors and mobile. No one-click promise of 'tutti i sensori' non supportati da OS.

### Canonical model
- `Node`: `uuid`, `agent_fingerprint`, `display_label`, platform, OS, lifecycle, grouping.
- `Endpoint`: node_id, URI/IP/port, transport, interface, `observed_by`, last_probe, RTT, tunnel classification.
- `Component`: per-node stable component ID / category / parent (CPU, GPU, NVMe, disk, NIC, power meter).
- `MetricDescriptor`: metric name, type (gauge/counter/hist), unit, aggregation semantics, sample interval, labels budget.
- `Observation`: node/component/workload IDs, value OR unavailability reason, source, source version, sampled_at, received_at, quality, collector_error reference, seq.
- `RuntimeInstance`, `ModelInstance`, `WorkloadRun`, `Service`, `ServiceProbe`, `Event`, `AlertRule`, `AlertInstance`, `SourceHealth`.
- `Status`: distinct discovered, pairing, connected, telemetry-online, stale, uninstrumented, unreachable, paused, unsupported, revoked.

No aggregation of board/CPU/SoC/whole device Watt when overlapping. Explicit uncertainty for derived measurements. Multi-GPU aggregations must specify max, weighted or sum and when valid; never use a maximum as if it were a total.

## 10. Services / non-agent integrations

- **Native agent service inventory:** systemd (Linux), launchd (macOS), Docker/Podman (opt-in runtime API), later K8s.
- **Generic remote endpoint:** HTTP(S) probe with status/body/latency, TCP connect with timing, TLS expiry, DNS resolution; allowlist of target domains/networks and bounded probes.
- **OpenMetrics/Prometheus:** user-provided scrape target, auth/TLS controls and label mapping with cardinality limits. OpenMetrics is a wire format, not unauthenticated access by default.
- **OTLP:** optional receiver for metrics/events/traces with authentication, ingestion quotas, namespace isolation.
- **Infrastructure devices:** SNMPv3 (not insecure default v1/v2c), Redfish, NUT for UPS; user credentials in secret storage, permissions documented.
- **AI:** adapter SPI (capabilities + metrics), versioned adapters: Ollama first, vLLM Prom metrics, MLX/llama.cpp where instrumentable, YVEX optional. No prompt/output collection by default.

## 11. Storage, history and diagnostics

Core tables: `schema_migrations`, `settings`, `nodes`, `node_endpoints`, `identity_trust`, `collector_policies`, `components`, `metric_descriptors`, `samples`, `rollups`, `service_targets`, `service_checks`, `events`, `alert_rules`, `alert_instances`, `agent_status`, `diagnostics`.

- SQLite WAL, bulk insert transaction, indexes on (entity,metric,time), retention configurable, rollups e.g. raw 1s short, 10s intermediate, 1m long; actual profiles user-configurable with disk quota.
- Backfill/replay semantics with sample sequence + boot ID, idempotent insert, query filter by source/time/quality, case of clock drift.
- History finite and audited: default 7 days can be configurable, no infinite raw storage. Report measured DB footprint per #hosts × active series.
- Alert engine read-only with threshold+duration, hysteresis, quiet/maintenance, dedup, ownership/ack, event timeline. Never fire `GPU >90` without verifying availability and period.
- Observability del monitor: each collector reports scan_duration, attempted/errors/unsupported, last_good, dropped_batches, retry, queue watermarks; diagnostics export sanitized with user approval.

## 12. Data collection efficiency and QA

- Separate host counters fast (~1s), process metrics (~2–5s), devices/sensors (~5–15s), hardware inventory (~60s/on change), AI request metrics event-driven where possible. Actual values profiled per platform.
- Never spawn commands per metric per tick. Use native APIs, shared pools, cache slow probes. Background reduced refresh UI; agent collection cadence need not match redraw rate.
- Quantitative acceptance **targets to measure, not achieved claims**: no busy render when idle; small bounded memory and network use per agent; no measurable statistically significant regression in inference workloads above a budget to be fixed from paired benchmarks; UI interactions low latency; zero unbounded queue growth.
- Test 0/1/2/5/16/32/100 host fixture environments; smoke 5 real mixed hosts where authorized; chaos on VPN off/on, DNS changes, multiple addresses, sleep/wake, agent restart/reinstall, clocks skewed, disk full, malformed messages, policy denial and IPv6-only.
- Use pipeline: fmt/clippy/check/unit -> contract -> integration/SSH/service-> visual snapshot + accessibility -> soak -> Mac/Linux packaged build. Real data are captured only in ignored local test assets; public screenshots synthetic and clearly labeled.
- Security scanning: dependency audit, codeQL/equivalent, signature verification, fuzz protocol parsers and candidate discovery; test fake mDNS announcements and replay/wrong-cert.
- Performance cross-run paired measurement for idle and GPU/AI workloads; record version/device/baselines and confidence, not a single best result.

## 13. Public repository readiness (blocker P0)

### Clean-room public content
- Delete/replace personal inventory in `docs/inventory.md`, `config/fleet.example.toml`, README images, `docs/assets/overview-*.png` and any screenshots/QA assets containing home host names or network details.
- Change default local ID to stable generic UUID or local hostname evaluated only on user machine; do not ship `example-host`, `example-host`, etc. as defaults.
- Replace packaging metadata personal identifiers with project namespace, generic filenames; verify branding/release download URLs.
- Audit every tracked file, tags/branches, binary blobs, releases, Actions logs/artifacts and **Git history** for sensitive content. Current branch cleanup alone does not remove content from older commits on GitHub. Decide separately whether to sanitize history with controlled repository migration / rewrite before changing visibility, after backup and explicit signoff.
- No tokens, hosts/IP, full names, emails, private SSH aliases, tailnet names, screenshot metadata. Dependencies/licenses and font licenses checked.
- Add explicit dual MIT/Apache-2.0 or another license after author decision; `LICENSE`, `CONTRIBUTING.md`, `CODE_OF_CONDUCT.md`, `SECURITY.md`, `PRIVACY.md`, `CHANGELOG.md`, releases policy, reproducible binaries/checksums and provenance, SBOM where possible.
- macOS signed/notarized binaries require appropriate account and release process; source build remains the fallback. Linux packages/appimage/.deb/.rpm and user services need packaging qualification.

## 14. Sequencing / deliverables / acceptance gates

### W0 — Project sanitation and UX/design contract
- Freeze baseline, inspect code/docs/screens, replace wireframes with measured density/interaction prototypes.
- Build design tokens and UI component contracts (navigation, tables, mini charts, states, forms/wizard), define event flows.
- Write ADR controller vs desktop, protocol/discovery security, import migration and private-to-public strategy.
**Done when:** all primary screens specified, empty/unsupported/stale handled, 1280×800 5-host overview above fold, prototype approved; tests exist for navigation/state.

### W1 — Vertical slice of new user experience
- Rebuild shell/Overview/Fleet/Node Detail & Settings route tree in Iced.
- Introduce settings persistence layer and CRUD host model backed by migrations; do NOT require manual TOML. Preserve current SSH transport as compatibility adapter.
- First run detects local computer and displays real metrics with no configuration; onboarded remote via existing SSH import/manual host.
**Done when:** install->first run->local real telemetry->add SSH host->save/restart->same host remains; no graphics filler.

### W2 — Agent service, protocol, remote lifecycle
- Headless controller split; agent persistent user service; RPC/transport with TLS/pairing, signed binaries, install/status/update/revoke flow.
- SSH bootstrap assisted from GUI with strict host key confirmation, safe privileges and rollback.
**Done when:** controller collects with UI closed (persistent mode), agents boot/recover, invalid identity rejected, cannot get remote arbitrary command execution.

### W3 — Discovery LAN, Tailscale/Headscale, manual
- mDNS/DNS-SD, peer provider, route selection, dedup verified node identities, failover; manual IP/service and explicit subnet scan.
**Done when:** local agent auto-discovery in same link, candidate via Tailscale, manual WireGuard IP works, LAN->VPN failover with same node_id, unsanctioned peer not automatically trusted.

### W4 — Telemetry + services + storage + alert quality
- Per-component history, richer sensors, process and service inventory, OpenMetrics/OTLP adapters, AI/energy capabilities with provenance, alerts/events.
**Done when:** demonstrate non-zero/zero/unavailable correctness and source quality; historical queries exact, alerts validated; compare multiple nodes and explicit service probes.

### W5 — Public release qualification
- scrub public history/assets, licensing, packaging, code signing, update/rollback, documentation, first-run QA, internationalization, accessibility, soak and hardware benchmark.
**Done when:** fresh user can download, install, pair, monitor, uninstall, export/delete data; no private data remain in public artifacts.

## 15. Non-goals in first public release

Non implementare un altro servizio VPN; non promettere VPN peer-to-peer automatic discovery ovunque; non promettere iOS come agent always-on, né telemetria dell'intero iPhone; no remote shell executor/process killer; no inferred AI token/s; no whole-lab power senza meter coverage; no hosted account, Kubernetes cluster orchestrator, AI chat assistant o plugin marketplace obbligatori.

## 16. First usable end-to-end story

Installazione su computer A -> Wizard raccoglie CPU/RAM locali -> app rileva Tailscale già installato -> elenca peer candidati (senza assumere trust) -> utente seleziona computer B -> deploy via SSH verificato oppure usa agent già installato -> conferma identità -> telemetria B + GPU compare nella stessa Overview -> l'utente aggiunge un endpoint servizio HTTPS -> lo storico resta consultabile dopo aver chiuso/riaperto la UI se Controller persistente è attivato -> l'utente modifica alias, campionamento e retention via Settings -> revoca B e vede esattamente cosa succede allo storico locale.

L'implementazione deve essere progettata per questa storia completa prima di promettere il supporto a ogni provider e ogni sensore.

## 17. Reference documents (external, checked 24 Oct 2026)

- mDNS: https://www.rfc-editor.org/info/rfc6762/ ; DNS-SD: https://www.rfc-editor.org/info/rfc6763/
- Tailscale CLI JSON: https://tailscale.com/kb/1080/cli ; MagicDNS: https://tailscale.com/docs/features/magicdns
- WireGuard quickstart: https://www.wireguard.com/quickstart/
- NetBird CLI: https://docs.netbird.io/get-started/cli ; ZeroTier APIs: https://docs.zerotier.com/api/
- OTLP: https://opentelemetry.io/docs/specs/otlp/ ; Prometheus exposition/OpenMetrics: https://prometheus.io/docs/instrumenting/exposition_formats/
- NVIDIA DCGM exporter: https://docs.nvidia.com/datacenter/dcgm/latest/gpu-telemetry/dcgm-exporter.html
- Beszel installation architecture: https://beszel.dev/guide/agent-installation
