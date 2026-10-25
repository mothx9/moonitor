use crate::config::{Config, NodeConfig, NodeKind};
use anyhow::{Result, bail};
use moonitor_core::{HostSnapshot, MAX_MESSAGE_BYTES, now_ms};
use moonitor_store::{HistoryPoint, Store, memory_percent};
use std::{
    collections::{HashMap, VecDeque},
    io::{BufRead, BufReader, Read},
    path::PathBuf,
    process::{Child, Command, Stdio},
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, AtomicU64, Ordering},
        mpsc,
    },
    thread,
    time::{Duration, Instant},
};

const HISTORY_LIMIT: usize = 720;

#[derive(Clone)]
pub struct NodeState {
    pub config: NodeConfig,
    pub capacity_limited: bool,
    pub sample: Option<Arc<HostSnapshot>>,
    pub last_seen: Option<Instant>,
    pub error: Option<String>,
    pub history: VecDeque<HistoryPoint>,
}

impl NodeState {
    pub fn live(&self) -> bool {
        crate::view_model::NodeStatus::of(self, Instant::now())
            == crate::view_model::NodeStatus::Online
    }
}

pub struct State {
    pub nodes: Vec<NodeState>,
    pub storage_error: Option<String>,
    pub dropped_events: Arc<AtomicU64>,
}
type ChildSlot = Arc<Mutex<Option<Child>>>;
pub struct Engine {
    pub state: Arc<Mutex<State>>,
    stop: Arc<AtomicBool>,
    children: Vec<ChildSlot>,
    workers: Vec<thread::JoinHandle<()>>,
}
impl std::fmt::Debug for Engine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Engine").finish_non_exhaustive()
    }
}
enum Event {
    Snapshot(String, Arc<HostSnapshot>, u64, Instant),
    Error(String, String),
}

impl Engine {
    pub fn start(config: Config, db_path: PathBuf) -> Self {
        let state = Arc::new(Mutex::new(State {
            storage_error: None,
            dropped_events: Arc::new(AtomicU64::new(0)),
            nodes: config
                .nodes
                .iter()
                .map(|c| NodeState {
                    config: c.clone(),
                    capacity_limited: false,
                    sample: None,
                    last_seen: None,
                    error: None,
                    history: VecDeque::new(),
                })
                .collect(),
        }));
        let dropped = state.lock().unwrap().dropped_events.clone();
        let stop = Arc::new(AtomicBool::new(false));
        let (tx, rx) = mpsc::sync_channel(16);
        let shared = state.clone();
        let stopped = stop.clone();
        let ingestor = thread::spawn(move || {
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
            let mut last_saved = HashMap::<String, u64>::new();
            let mut last_prune = 0u64;
            while !stopped.load(Ordering::Relaxed) {
                let event = match rx.recv_timeout(Duration::from_secs(1)) {
                    Ok(event) => event,
                    Err(mpsc::RecvTimeoutError::Timeout) => continue,
                    Err(mpsc::RecvTimeoutError::Disconnected) => break,
                };
                match event {
                    Event::Error(id, e) => {
                        if let Some(node) = shared
                            .lock()
                            .unwrap()
                            .nodes
                            .iter_mut()
                            .find(|n| n.config.id == id)
                        {
                            node.error = Some(e);
                        }
                    }
                    Event::Snapshot(id, snapshot, received, seen) => {
                        let Some(index) = config.nodes.iter().position(|n| n.id == id) else {
                            continue;
                        };
                        if let Err(e) = snapshot.validate(&config.nodes[index].id) {
                            shared.lock().unwrap().nodes[index].error = Some(e);
                            continue;
                        }
                        if received.saturating_sub(*last_saved.get(&id).unwrap_or(&0)) >= 5000
                            && let Some(db) = &mut store
                        {
                            match db.append(&snapshot, received) {
                                Ok(()) => {
                                    last_saved.insert(id.clone(), received);
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
        let mut workers = vec![ingestor];
        // The ingestor owns its configuration; workers use the same validated node list.
        let mut nodes: Vec<_> = state
            .lock()
            .unwrap()
            .nodes
            .iter()
            .map(|n| n.config.clone())
            .collect();
        nodes.sort_by_key(|n| match n.kind {
            NodeKind::Local => 0,
            NodeKind::Ssh => 1,
            NodeKind::Unavailable => 2,
        });
        let mut active = 0;
        for node in nodes {
            if node.kind != NodeKind::Unavailable {
                if active >= 32 {
                    if let Some(n) = state
                        .lock()
                        .unwrap()
                        .nodes
                        .iter_mut()
                        .find(|n| n.config.id == node.id)
                    {
                        n.capacity_limited = true;
                    }
                    continue;
                }
                active += 1;
            }
            let stopped = stop.clone();
            let sender = tx.clone();
            let dropped = dropped.clone();
            match node.kind {
                NodeKind::Local => {
                    workers.push(thread::spawn(move || {
                        let mut collector = moonitor_agent::Collector::new(node.id.clone());
                        thread::sleep(Duration::from_millis(300));
                        while !stopped.load(Ordering::Relaxed) {
                            let tick = Instant::now();
                            let sample = Arc::new(collector.sample());
                            enqueue(
                                &sender,
                                &dropped,
                                Event::Snapshot(node.id.clone(), sample, now_ms(), Instant::now()),
                            );
                            thread::sleep(Duration::from_secs(1).saturating_sub(tick.elapsed()));
                        }
                    }));
                }
                NodeKind::Ssh => {
                    let child = Arc::new(Mutex::new(None));
                    children.push(child.clone());
                    workers.push(thread::spawn(move || {
                        while !stopped.load(Ordering::Relaxed) {
                            let result = remote_stream(&node, &sender, &child, &stopped, &dropped);
                            if !stopped.load(Ordering::Relaxed) {
                                enqueue(
                                    &sender,
                                    &dropped,
                                    Event::Error(
                                        node.id.clone(),
                                        result
                                            .err()
                                            .map(|e| e.to_string())
                                            .unwrap_or_else(|| "SSH stream ended".into()),
                                    ),
                                );
                                for _ in 0..100 {
                                    if stopped.load(Ordering::Relaxed) {
                                        break;
                                    }
                                    thread::sleep(Duration::from_millis(100));
                                }
                            }
                        }
                    }));
                }
                NodeKind::Unavailable => (),
            }
        }
        Self {
            state,
            stop,
            children,
            workers,
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
        for worker in self.workers.drain(..) {
            let _ = worker.join();
        }
    }
}

fn remote_stream(
    node: &NodeConfig,
    tx: &mpsc::SyncSender<Event>,
    slot: &ChildSlot,
    stop: &AtomicBool,
    dropped: &AtomicU64,
) -> Result<()> {
    let agent = node
        .remote_agent
        .as_deref()
        .unwrap_or(".local/bin/moonitor-agent");
    let command = format!("exec {agent} --node-id {} --interval-ms 1000", node.id);
    let mut ssh = Command::new("ssh");
    if let Some(user) = &node.ssh_user {
        ssh.args(["-l", user]);
    }
    if let Some(port) = node.ssh_port {
        ssh.args(["-p", &port.to_string()]);
    }
    let mut child = ssh
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
            enqueue(
                tx,
                dropped,
                Event::Snapshot(
                    node.id.clone(),
                    Arc::new(snapshot),
                    now_ms(),
                    Instant::now(),
                ),
            );
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

fn enqueue(tx: &mpsc::SyncSender<Event>, dropped: &AtomicU64, event: Event) {
    if matches!(tx.try_send(event), Err(mpsc::TrySendError::Full(_))) {
        dropped.fetch_add(1, Ordering::Relaxed);
    }
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
    fn empty_fleet_ingestion_exits_instead_of_spinning_on_disconnected_channel() {
        let path = std::env::temp_dir().join(format!(
            "moonitor-empty-{}-{}.sqlite",
            std::process::id(),
            now_ms()
        ));
        let engine = Engine::start(Config { nodes: vec![] }, path.clone());
        for _ in 0..100 {
            if engine.workers[0].is_finished() {
                break;
            }
            thread::sleep(Duration::from_millis(10));
        }
        assert!(
            engine.workers[0].is_finished(),
            "Empty fleets must not busy-loop"
        );
        drop(engine);
        let _ = std::fs::remove_file(path);
    }
    #[test]
    fn bounded_ingress_accounts_for_dropped_events() {
        let (tx, _rx) = mpsc::sync_channel(1);
        let dropped = AtomicU64::new(0);
        enqueue(
            &tx,
            &dropped,
            Event::Error("fixture".into(), "first".into()),
        );
        enqueue(
            &tx,
            &dropped,
            Event::Error("fixture".into(), "second".into()),
        );
        assert_eq!(dropped.load(Ordering::Relaxed), 1);
    }
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
