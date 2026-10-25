# Roadmap / V3 interface and V2 architecture

The supplied V3 specification replaces the earlier W1 layout. Preserve the earlier
W0/W1 reports as historical evidence; their gate names are not the V2 wave names.

| Wave | Dependency and acceptance |
| --- | --- |
| W1 / UI1–UI2 | Fleet-first shell, host resource workspace, persistent GUI CRUD and first-run local metrics. See the current qualification report. |
| W2 | Controller independent of UI, user service, versioned authenticated protocol, agent lifecycle and revocation. |
| W3 | LAN/VPN/manual discovery candidates followed by explicit identity verification and enrollment. |
| W4 / UI3 | Per-component history, synchronized comparisons, persisted events, services, runtime requests and energy coverage. |
| W5 / UI5 | Privacy and Git history audit, author-selected license, distributable packaging, accessibility, soak, sleep/wake, inference overhead and cross-platform release qualification. |

See [requirements](moonitor-v2/product-requirements.md),
[rollout and test gates](moonitor-v2/rollout-test-plan.md) and
[qualification](moonitor-v2/qualification.md). Discovery is not trust, loaded models
are not active inference, and current-file cleanup is not a public-release audit.
