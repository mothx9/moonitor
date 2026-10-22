# Initial fleet

Inventory checkpoint: 2026-10-22. Four known computers plus one iPhone. This is a
project scope inventory, not a completed scan of every device on the LAN.

| Node | Platform/hardware | Evidence | Current reachability |
| --- | --- | --- | --- |
| MacBook Pro | macOS 26.6.2, Apple M5 Pro, 24 GB unified memory, 15 CPU cores, 16 GPU cores | Local `system_profiler` and `sw_vers` on 2026-10-22 | Local hardware verified |
| Exon | Previous inventory: Arch Linux, NVIDIA RTX 5060 Ti, about 46 GiB RAM | Existing SSH configuration; hardware from 2026-10-07 diagnostics, needs refresh | Not established |
| DGX Spark | Linux ARM64; exact memory and current GPU capabilities need collection | Existing LAN/VPN and NVIDIA Sync configuration; previous project documentation | Not established |
| Zima | Previous documentation: Debian 12, x86_64; hardware needs collection | Existing SSH configuration and project deployment documentation | Not established |
| iPhone | Model, OS version and available telemetry need verification | User-confirmed fleet member | Not established |

During the initial check, SSH was blocked by the session's execution permissions
with `Operation not permitted`. Tailscale status could not be obtained, and the
device listing timed out. These outcomes do not establish that any node is offline.

Next discovery must obtain actual CPU/RAM/GPU details, active runtimes, available
sensors and permissions from each reachable computer. Keep addresses and machine
identifiers in ignored local configuration; do not commit them here.
