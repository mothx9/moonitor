//! Read-only collectors. No shell commands or privileged helpers run per sample.

use moonitor_core::*;
use serde::Deserialize;
use std::{collections::HashMap, time::Instant};
use sysinfo::{Components, Disks, Networks, ProcessRefreshKind, ProcessesToUpdate, System};

#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "linux")]
mod nvidia;

pub struct Collector {
    id: String,
    system: System,
    networks: Networks,
    disks: Disks,
    components: Components,
    last_tick: Instant,
    slow_tick: Instant,
    process_tick: Instant,
    cached_disks: Vec<DiskSnapshot>,
    cached_temps: Vec<TemperatureSnapshot>,
    runtime: RuntimeSnapshot,
    http: ureq::Agent,
    #[cfg(target_os = "linux")]
    nvidia: Option<nvml_wrapper::Nvml>,
}

impl Collector {
    pub fn new(id: String) -> Self {
        let mut system = System::new();
        system.refresh_cpu_all();
        system.refresh_memory();
        system.refresh_processes_specifics(ProcessesToUpdate::All, true, process_refresh());
        let past = Instant::now() - std::time::Duration::from_secs(30);
        Self {
            id,
            system,
            networks: Networks::new_with_refreshed_list(),
            disks: Disks::new_with_refreshed_list(),
            components: Components::new_with_refreshed_list(),
            last_tick: Instant::now(),
            slow_tick: past,
            process_tick: past,
            cached_disks: Vec::new(),
            cached_temps: Vec::new(),
            runtime: RuntimeSnapshot {
                models: Reading::unavailable(UnavailableReason::NotConfigured),
            },
            http: ureq::Agent::config_builder()
                .timeout_global(Some(std::time::Duration::from_millis(300)))
                .proxy(None)
                .max_redirects(0)
                .build()
                .into(),
            #[cfg(target_os = "linux")]
            nvidia: nvml_wrapper::Nvml::init().ok(),
        }
    }

    pub fn sample(&mut self) -> HostSnapshot {
        let start = Instant::now();
        let ts = now_ms();
        let elapsed = self.last_tick.elapsed().as_secs_f64().max(0.001);
        self.last_tick = start;
        self.system.refresh_cpu_usage();
        self.system.refresh_memory();
        self.networks.refresh(true);
        if self.process_tick.elapsed().as_secs() >= 2 {
            self.system.refresh_processes_specifics(
                ProcessesToUpdate::All,
                true,
                process_refresh(),
            );
            self.process_tick = Instant::now();
        }
        if self.slow_tick.elapsed().as_secs() >= 5 {
            self.disks.refresh(true);
            self.cached_disks = self
                .disks
                .list()
                .iter()
                .take(128)
                .map(|d| DiskSnapshot {
                    mount: d.mount_point().to_string_lossy().into(),
                    name: d.name().to_string_lossy().into(),
                    total_bytes: d.total_space(),
                    available_bytes: d.available_space(),
                    sampled_at_unix_ms: ts,
                })
                .collect();
            self.components.refresh(true);
            self.cached_temps = self
                .components
                .list()
                .iter()
                .filter_map(|c| {
                    c.temperature()
                        .filter(|t| t.is_finite())
                        .map(|celsius| TemperatureSnapshot {
                            label: c.label().into(),
                            celsius,
                        })
                })
                .take(128)
                .collect();
            self.runtime = collect_ollama(&self.http, ts);
            self.slow_tick = Instant::now();
        }
        let mut processes: Vec<_> = self
            .system
            .processes()
            .iter()
            .map(|(pid, p)| {
                let name = p.name().to_string_lossy().into_owned();
                ProcessSnapshot {
                    pid: pid.as_u32(),
                    is_ai: is_ai_process(&name),
                    name,
                    cpu_percent: p.cpu_usage(),
                    memory_bytes: p.memory(),
                }
            })
            .collect();
        processes.sort_by(|a, b| {
            b.cpu_percent
                .total_cmp(&a.cpu_percent)
                .then(b.memory_bytes.cmp(&a.memory_bytes))
        });
        // Keep AI candidates even when idle; bound process payloads independently of host size.
        let mut selected: Vec<_> = processes
            .iter()
            .filter(|p| p.is_ai)
            .take(32)
            .cloned()
            .collect();
        for p in processes.iter().take(64) {
            if !selected.iter().any(|x| x.pid == p.pid) {
                selected.push(p.clone());
            }
        }
        selected.sort_by(|a, b| b.cpu_percent.total_cmp(&a.cpu_percent));
        let (gpus, power) = self.gpus(ts);
        let mut snapshot = HostSnapshot {
            schema_version: SCHEMA_VERSION,
            node_id: self.id.clone(),
            sampled_at_unix_ms: ts,
            hostname: System::host_name().unwrap_or_else(|| self.id.clone()),
            os: System::long_os_version().unwrap_or_else(|| std::env::consts::OS.into()),
            arch: std::env::consts::ARCH.into(),
            cpu_name: self
                .system
                .cpus()
                .first()
                .map(|c| c.brand().into())
                .unwrap_or_default(),
            uptime_seconds: System::uptime(),
            cpu_utilization_percent: Reading::reported(
                f64::from(self.system.global_cpu_usage()),
                "sysinfo",
                ts,
            ),
            cpu_cores_percent: self
                .system
                .cpus()
                .iter()
                .take(512)
                .map(|c| c.cpu_usage())
                .collect(),
            memory: MemorySnapshot {
                total_bytes: Reading::reported(self.system.total_memory(), "sysinfo", ts),
                available_bytes: Reading::reported(self.system.available_memory(), "sysinfo", ts),
                swap_used_bytes: Reading::reported(self.system.used_swap(), "sysinfo", ts),
            },
            gpus,
            power,
            processes: selected,
            process_count: processes.len(),
            disks: self.cached_disks.clone(),
            temperatures: self.cached_temps.clone(),
            networks: self
                .networks
                .iter()
                .take(128)
                .map(|(name, n)| NetworkSnapshot {
                    name: name.clone(),
                    received_bytes_per_second: n.received() as f64 / elapsed,
                    transmitted_bytes_per_second: n.transmitted() as f64 / elapsed,
                    errors: n
                        .total_errors_on_received()
                        .saturating_add(n.total_errors_on_transmitted()),
                })
                .collect(),
            pressure: collect_pressure(),
            ollama: self.runtime.clone(),
            battery_percent: battery(ts),
            collector_duration_ms: 0.0,
        };
        snapshot.collector_duration_ms = start.elapsed().as_secs_f64() * 1000.0;
        snapshot
    }

    fn gpus(&self, ts: u64) -> (Vec<GpuSnapshot>, Vec<PowerReading>) {
        #[cfg(target_os = "linux")]
        {
            self.nvidia
                .as_ref()
                .map(|n| nvidia::collect(n, ts))
                .unwrap_or_default()
        }
        #[cfg(target_os = "macos")]
        {
            (
                macos::gpu(self.system.total_memory(), ts)
                    .into_iter()
                    .collect(),
                Vec::new(),
            )
        }
        #[cfg(not(any(target_os = "macos", target_os = "linux")))]
        {
            let _ = ts;
            (Vec::new(), Vec::new())
        }
    }
}

fn process_refresh() -> ProcessRefreshKind {
    ProcessRefreshKind::nothing()
        .with_cpu()
        .with_memory()
        .without_tasks()
}

pub fn is_ai_process(name: &str) -> bool {
    let n = name.to_lowercase();
    [
        "ollama",
        "llama",
        "vllm",
        "mlx",
        "yvex",
        "comfy",
        "lm studio",
        "lm-studio",
        "lmstudio",
        "tensorrt",
    ]
    .iter()
    .any(|x| n.contains(x))
}

#[derive(Deserialize)]
struct OllamaResponse {
    models: Vec<OllamaModel>,
}
#[derive(Deserialize)]
struct OllamaModel {
    name: String,
    size: u64,
    size_vram: Option<u64>,
    context_length: Option<u64>,
    #[serde(default)]
    details: HashMap<String, serde_json::Value>,
}

fn collect_ollama(http: &ureq::Agent, ts: u64) -> RuntimeSnapshot {
    let response = http
        .get("http://127.0.0.1:11434/api/ps")
        .call()
        .and_then(|mut r| {
            r.body_mut()
                .with_config()
                .limit(256 * 1024)
                .read_json::<OllamaResponse>()
        });
    let models = match response {
        Ok(r) if r.models.len() <= 128 => Reading::reported(
            r.models
                .into_iter()
                .map(|m| LoadedModel {
                    name: m.name,
                    size_bytes: m.size,
                    gpu_allocation_bytes: m.size_vram,
                    context_length: m.context_length,
                    quantization: m
                        .details
                        .get("quantization_level")
                        .and_then(|s| s.as_str())
                        .map(String::from),
                })
                .collect(),
            "ollama /api/ps",
            ts,
        ),
        _ => Reading::unavailable(UnavailableReason::TemporarilyUnavailable),
    };
    RuntimeSnapshot { models }
}

pub fn parse_psi(text: &str) -> Option<f64> {
    let line = text.lines().find(|l| l.starts_with("some "))?;
    let value = line
        .split_whitespace()
        .find_map(|p| p.strip_prefix("avg10="))?
        .parse::<f64>()
        .ok()?;
    (value.is_finite() && (0.0..=100.0).contains(&value)).then_some(value)
}

fn collect_pressure() -> Vec<PressureSnapshot> {
    #[cfg(target_os = "linux")]
    {
        ["cpu", "memory", "io"]
            .iter()
            .filter_map(|resource| {
                let text = std::fs::read_to_string(format!("/proc/pressure/{resource}")).ok()?;
                Some(PressureSnapshot {
                    resource: (*resource).into(),
                    some_avg10_percent: parse_psi(&text)?,
                })
            })
            .collect()
    }
    #[cfg(not(target_os = "linux"))]
    {
        Vec::new()
    }
}

fn battery(ts: u64) -> Reading<f64> {
    #[cfg(target_os = "macos")]
    {
        macos::battery(ts)
    }
    #[cfg(not(target_os = "macos"))]
    {
        let _ = ts;
        Reading::unavailable(UnavailableReason::Unsupported)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn real_platform_snapshot_obeys_the_protocol() {
        let mut collector = Collector::new("qualification".into());
        std::thread::sleep(
            sysinfo::MINIMUM_CPU_UPDATE_INTERVAL.max(std::time::Duration::from_millis(250)),
        );
        let snapshot = collector.sample();
        snapshot.validate("qualification").unwrap();
        assert!(!snapshot.cpu_cores_percent.is_empty());
        assert!(snapshot.memory.total_bytes.value().is_some_and(|m| *m > 0));
        assert!(snapshot.validate("wrong-node").is_err());
        let serialized = serde_json::to_vec(&snapshot).unwrap();
        assert!(serialized.len() < MAX_MESSAGE_BYTES);
        let decoded: HostSnapshot = serde_json::from_slice(&serialized).unwrap();
        assert_eq!(decoded.node_id, snapshot.node_id);
    }
    #[test]
    fn psi_uses_some_avg10_and_rejects_invalid_values() {
        assert_eq!(
            parse_psi("some avg10=12.34 avg60=4.50 total=100\nfull avg10=1.0"),
            Some(12.34)
        );
        for input in [
            "full avg10=1.0",
            "some avg10=NaN",
            "some avg10=-1",
            "some avg10=101",
            "",
        ] {
            assert_eq!(parse_psi(input), None);
        }
    }
    #[test]
    fn generic_python_is_not_evidence_of_an_ai_runtime() {
        assert!(!is_ai_process("python3"));
        assert!(is_ai_process("ollama runner"));
    }
}
