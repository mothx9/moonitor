# Initial fleet

Inventory checkpoint: 2026-10-22. Four known computers plus one iPhone. This is a
project scope inventory, not a completed scan of every device on the LAN.

| Node | Platform/hardware | Evidence | Current reachability |
| --- | --- | --- | --- |
| MacBook Pro | macOS 26.6.2, Apple M5 Pro, 24 GB unified memory, 15 CPU cores, 16 GPU cores | Local `system_profiler` and `sw_vers` on 2026-10-22 | Local hardware verified |
| Exon | Arch Linux, Ryzen 9 7900X, 24 logical cores, 46.6 GiB RAM, RTX 5060 Ti with 16 GiB VRAM | Fresh SSH inventory and actual Rust agent snapshots | Live over existing VPN SSH |
| DGX Spark | Ubuntu 24.04.4, ARM64, 20 cores, 121.7 GiB shared memory, NVIDIA GB10 | Fresh SSH inventory and actual Rust agent snapshots | Live over existing VPN SSH |
| Zima | Debian 12, x86_64, Celeron N3350, 2 cores, 1.8 GiB RAM | Fresh SSH inventory and actual Rust agent snapshots | Live over existing VPN SSH |
| iPhone | Physical paired device reports model identifier iPhone17,3 | Fresh `devicectl` listing; distinct from simulated devices | Paired/available; telemetry not connected |

The initial restricted session could not establish connectivity. The later live
check found that the Mac's existing Tailscale backend was stopped. Resuming it
with its existing non-default route preference restored the three VPN SSH paths.
Old LAN addresses were unreachable and were not rewritten.

Ollama answers on the Mac with an empty loaded-model list. Its loopback API was
not available on the three Linux hosts during qualification. No inference
request, model download or workload interruption was used for discovery.

The tailnet also listed `hp-erika`; the user explicitly excluded it from this
project's five-device fleet. Keep addresses and machine identifiers in ignored
local configuration; do not commit them here.
