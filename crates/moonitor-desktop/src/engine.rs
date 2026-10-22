use crate::config::{Config, NodeConfig, NodeKind};
use anyhow::{Result, bail};
use moonitor_core::{HostSnapshot, MAX_MESSAGE_BYTES, now_ms};
use moonitor_store::{HistoryPoint, Store, memory_percent};
use std::{
    collections::VecDeque,
    io::{BufRead, BufReader, Read},
    path::PathBuf,
    process::{Child, Command, Stdio},
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
        mpsc,
    },
    thread,
    time::{Duration, Instant},
};

const HISTORY_LIMIT: usize = 720;

#[derive(Clone)]
pub struct NodeState {
    pub config: NodeConfig,
    pub sample: Option<Arc<HostSnapshot>>,
    pub last_seen: Option<Instant>,
    pub error: Option<String>,
    pub history: VecDeque<HistoryPoint>,
}

impl NodeState {
    pub fn live(&self) -> bool {
        self.sample.is_some()
            && self
                .last_seen
                .is_some_and(|t| t.elapsed() < Duration::from_secs(4))
            && self.error.is_none()
    }
}

pub struct State {
    pub nodes: Vec<NodeState>,
    pub storage_error: Option<String>,
}
type ChildSlot = Arc<Mutex<Option<Child>>>;
pub struct Engine {
    pub state: Arc<Mutex<State>>,
    stop: Arc<AtomicBool>,
    children: Vec<ChildSlot>,
}
enum Event {
    Snapshot(usize, Arc<HostSnapshot>, u64, Instant),
    Error(usize, String),
}

impl Engine {
    pub fn start(config: Config, db_path: PathBuf) -> Self {
        let state = Arc::new(Mutex::new(State {
            storage_error: None,
            nodes: config
                .nodes
                .iter()
                .map(|c| NodeState {
                    config: c.clone(),
                    sample: None,
                    last_seen: None,
                    error: None,
                    history: VecDeque::new(),
                })
                .collect(),
        }));
        let stop = Arc::new(AtomicBool::new(false));
        let (tx, rx) = mpsc::sync_channel(16);
        let shared = state.clone();
        let stopped = stop.clone();
        thread::spawn(move || {
            let mut store = match Store::open(&db_path) {
                Ok(s) => Some(s),
                Err(e) => {
                    shared.lock().unwrap().storage_error = Some(e.to_string());
                    None
                }
            };
            if let Some(db) = &mut store {
                let mut shared = shared.lock().unwrap();
                for node in &mut shared.nodes {
                    match db.history(&node.config.id, now_ms().saturating_sub(3_600_000)) {
                        Ok(points) => node.history = points.into(),
                        Err(e) => node.error = Some(format!("History unavailable: {e}")),
                    }
                }
            }
            let mut last_saved = vec![0u64; config.nodes.len()];
            let mut last_prune = 0u64;
            while !stopped.load(Ordering::Relaxed) {
                let Ok(event) = rx.recv_timeout(Duration::from_secs(1)) else {
                    continue;
                };
                match event {
                    Event::Error(index, e) => shared.lock().unwrap().nodes[index].error = Some(e),
                    Event::Snapshot(index, snapshot, received, seen) => {
                        if let Err(e) = snapshot.validate(&config.nodes[index].id) {
                            shared.lock().unwrap().nodes[index].error = Some(e);
                            continue;
                        }
                        if received.saturating_sub(last_saved[index]) >= 5000
                            && let Some(db) = &mut store
                        {
                            match db.append(&snapshot, received) {
                                Ok(()) => {
                                    last_saved[index] = received;
                                    shared.lock().unwrap().storage_error = None;
                                }
                                Err(e) => {
                                    shared.lock().unwrap().storage_error = Some(e.to_string())
                                }
                            }
                            if received.saturating_sub(last_prune) >= 3_600_000 {
                                if let Err(e) = db.prune(received.saturating_sub(7 * 86_400_000)) {
                                    shared.lock().unwrap().storage_error = Some(e.to_string());
                                }
                                last_prune = received;
                            }
                        }
                        let point = HistoryPoint {
                            received_at_ms: received,
                            cpu_percent: snapshot.cpu_utilization_percent.value().copied(),
                            memory_percent: memory_percent(&snapshot),
                            gpu_percent: snapshot
                                .gpus
                                .iter()
                                .filter_map(|g| g.utilization_percent.value().copied())
                                .reduce(f64::max),
                        };
                        let mut shared = shared.lock().unwrap();
                        let node = &mut shared.nodes[index];
                        node.sample = Some(snapshot);
                        node.last_seen = Some(seen);
                        node.error = None;
                        node.history.push_back(point);
                        while node.history.len() > HISTORY_LIMIT {
                            node.history.pop_front();
                        }
                    }
                }
                // The UI samples this shared state on its own one-second timer.
                // Worker callbacks must not keep postponing the renderer's timer.
            }
        });
        let mut children = Vec::new();
        // The ingestor owns its configuration; workers use the same validated node list.
        let nodes: Vec<_> = state
            .lock()
            .unwrap()
            .nodes
            .iter()
            .map(|n| n.config.clone())
            .collect();
        for (index, node) in nodes.into_iter().enumerate() {
            let stopped = stop.clone();
            let sender = tx.clone();
            match node.kind {
                NodeKind::Local => {
                    thread::spawn(move || {
                        let mut collector = moonitor_agent::Collector::new(node.id);
                        thread::sleep(Duration::from_millis(300));
                        while !stopped.load(Ordering::Relaxed) {
                            let tick = Instant::now();
                            let sample = Arc::new(collector.sample());
                            let _ = sender.try_send(Event::Snapshot(
                                index,
                                sample,
                                now_ms(),
                                Instant::now(),
                            ));
                            thread::sleep(Duration::from_secs(1).saturating_sub(tick.elapsed()));
                        }
                    });
                }
                NodeKind::Ssh => {
                    let child = Arc::new(Mutex::new(None));
                    children.push(child.clone());
                    thread::spawn(move || {
                        while !stopped.load(Ordering::Relaxed) {
                            let result = remote_stream(&node, index, &sender, &child, &stopped);
                            if !stopped.load(Ordering::Relaxed) {
                                let _ = sender.try_send(Event::Error(
                                    index,
                                    result
                                        .err()
                                        .map(|e| e.to_string())
                                        .unwrap_or_else(|| "SSH stream ended".into()),
                                ));
                                for _ in 0..100 {
                                    if stopped.load(Ordering::Relaxed) {
                                        break;
                                    }
                                    thread::sleep(Duration::from_millis(100));
                                }
                            }
                        }
                    });
                }
                NodeKind::Unavailable => (),
            }
        }
        Self {
            state,
            stop,
            children,
        }
    }
}

impl Drop for Engine {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
        for slot in &self.children {
            if let Some(mut child) = slot.lock().unwrap().take() {
                let _ = child.kill();
                let _ = child.wait();
            }
        }
    }
}

fn remote_stream(
    node: &NodeConfig,
    index: usize,
    tx: &mpsc::SyncSender<Event>,
    slot: &ChildSlot,
    stop: &AtomicBool,
) -> Result<()> {
    let agent = node
        .remote_agent
        .as_deref()
        .unwrap_or(".local/bin/moonitor-agent");
    let command = format!("exec {agent} --node-id {} --interval-ms 1000", node.id);
    let mut child = Command::new("ssh")
        .args([
            "-T",
            "-oBatchMode=yes",
            "-oStrictHostKeyChecking=yes",
            "-oConnectTimeout=6",
            "-oServerAliveInterval=5",
            "-oServerAliveCountMax=2",
            node.ssh_alias.as_deref().unwrap(),
            &command,
        ])
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()?;
    let stdout = child.stdout.take().unwrap();
    {
        // Coordinate registration with Drop: a child must never appear after
        // shutdown has already inspected its slot.
        let mut guard = slot.lock().unwrap();
        if stop.load(Ordering::Relaxed) {
            let _ = child.kill();
            let _ = child.wait();
            return Ok(());
        }
        *guard = Some(child);
    }
    let result = (|| -> Result<()> {
        let mut reader = BufReader::new(stdout);
        while !stop.load(Ordering::Relaxed) {
            let Some(line) = read_frame(&mut reader)? else {
                break;
            };
            let snapshot: HostSnapshot = serde_json::from_slice(&line)?;
            snapshot.validate(&node.id).map_err(anyhow::Error::msg)?;
            let _ = tx.try_send(Event::Snapshot(
                index,
                Arc::new(snapshot),
                now_ms(),
                Instant::now(),
            ));
        }
        Ok(())
    })();
    if let Some(mut child) = slot.lock().unwrap().take() {
        let _ = child.kill();
        let status = child.wait()?;
        if result.is_ok() && !stop.load(Ordering::Relaxed) {
            bail!("SSH stream ended ({status}); check the host alias and agent installation");
        }
    }
    result
}

pub fn read_frame(reader: &mut impl BufRead) -> Result<Option<Vec<u8>>> {
    let mut line = Vec::new();
    reader
        .take((MAX_MESSAGE_BYTES + 1) as u64)
        .read_until(b'\n', &mut line)?;
    if line.len() > MAX_MESSAGE_BYTES {
        bail!("Agent message exceeds size limit");
    }
    Ok((!line.is_empty()).then_some(line))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn streaming_frames_are_bounded_even_without_a_newline() {
        let huge = vec![b'a'; MAX_MESSAGE_BYTES + 10];
        assert!(read_frame(&mut std::io::Cursor::new(huge)).is_err());
        let mut reader = std::io::Cursor::new(b"one\ntwo\n");
        assert_eq!(read_frame(&mut reader).unwrap().unwrap(), b"one\n");
        assert_eq!(read_frame(&mut reader).unwrap().unwrap(), b"two\n");
        assert!(read_frame(&mut reader).unwrap().is_none());
    }
}
