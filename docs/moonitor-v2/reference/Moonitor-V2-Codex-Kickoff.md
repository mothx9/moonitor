# Codex kickoff — Moonitor 2.0

Repository: `mothx9/moonitor`. Read `AGENTS.md` and `docs/moonitor-professional-upgrade-blueprint.md` first, and read attached `moonitor-v2-product-architecture-rfc.md`. The RFC is a TARGET DESIGN, not a claim that any feature exists. Source-of-truth is repository code and current behavior.

## Mandate

Moonitor must be transformed into a public-ready, local-first, multi-host native system observability product. The CURRENT UI is rejected. The lunar name/logo direction is approved, but the application layout, spacing, information hierarchy, host setup and Settings must be redesigned completely. Existing reliable Rust collectors, metric semantics, known-host SSH guarantees and SQLite foundations should be preserved or migrated safely rather than indiscriminately deleted. No personal fleet data or hardcoded host aliases in production code/public assets.

## Start NOW with one vertical slice, not a giant rewrite

1. Inspect current branch/worktree, remotes and divergence (follow `AGENTS.md`: owner currently requested direct `main`; do not create branch/PR unless changed). Back up safely; do not force-push, overwrite unrelated changes, or commit local configs. Document baseline SHA.
2. Audit actual current files, feature implementations, screenshots and recorded UX; record exact gaps vs RFC. Validate the new requirements rather than repeating an old W1 blueprint. In particular identify every reference to private host aliases/default examples, demo screenshot assets and personal package identifiers.
3. Author `docs/moonitor-v2/{product-requirements,ux-design-system,settings-onboarding,discovery-security,protocol-adr,rollout-test-plan}.md`. Include: screen-by-screen layout grids at 1280×800 / 1440×900 / 4K, density/typography/state contract, and diagrams for agent/controller/discovery/enrollment. Clearly mark decisions, assumptions, unresolved measured constraints.
4. Implement **W1 first working product slice** safely: native Overview fleet-first (all real observed hosts, compact comparative table); new shell/navigation; Settings with persistent state, editable local/SSH hosts via UI and no required TOML; first-run local auto-onboarding with genuine metrics. Use Iced/wgpu unless a reproducible limitation proves a framework change is necessary. Ensure no synthetic production telemetry.
5. Make state/configuration independent of a fragile list index; stable persisted IDs and transactional migration from current fleet config. No hardcoded maximum node count without a measured engineering limit. Update tests accordingly.
6. Verify on real renderer and in CI: Rust fmt/check/clippy/test, screenshots both themes and 3 viewports, navigation and keyboard, correct 0/1/5/32 host states, fake malformed metrics, persistence across restart, performance during idle. Screenshot snapshots are candidates, NOT automatic design approval.
7. Stop and report the finished vertical slice, exact commits, tests run and failures, screenshots/visual evidence and next dependency W2. Do not claim LAN/VPN discovery, mTLS agent service, OTLP, etc. are complete unless truly implemented and tested.

## Prioritize
- P0 UI density, onboarding/settings CRUD, no private hardcodes, current metrics correctness.
- P0 architecture boundaries to allow background controller & agent service and secure pairing.
- W2 agent/protocol/controller, W3 mDNS + Tailscale + generic transport/manual, W4 telemetry/services/history, W5 open-source release gates.

## Guardrails
- No arbitrary privileged commands to remote hosts; SSH deployment requires explicit user approval and key fingerprint verification. Read-only agent.
- Discovery is NOT authentication. VPNs are transports. Do not implement VPN protocols; use installed client/peer APIs; WireGuard lacks universal host/service discovery.
- Never conflate zero / unavailable / stale; preserve timestamps and source; don't sum overlapping Watt sensor scopes or Apple unified memory twice.
- Keep exposed endpoints private/authenticated by default. Secrets only in protected OS secret stores. No `curl | sh` silent installation.
- No unbounded threads, JSON payloads, queues, metrics labels, storage or scanning. no busy animation loop.
- For any terminal text editing, use vim, never nano.

## Final acceptance for this slice
A new user launches Moonitor, sees real local CPU/RAM on redesigned fleet-first Overview, adds/edits/removes a remote SSH host entirely from Settings, saves/relaunches successfully, can see state+source/freshness, and never touches TOML/YAML. No personal machine names are used as bundled defaults. Existing SSH-based telemetry remains functional as fallback.
