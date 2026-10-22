//! Compact history: process lists and full host snapshots are never persisted.

use anyhow::Result;
use moonitor_core::{HostSnapshot, PowerScope};
use rusqlite::{Connection, params};
use std::path::Path;

#[derive(Debug, Clone)]
pub struct HistoryPoint {
    pub received_at_ms: u64,
    pub cpu_percent: Option<f64>,
    pub memory_percent: Option<f64>,
    pub gpu_percent: Option<f64>,
}

pub struct Store {
    db: Connection,
}

impl Store {
    pub fn open(path: &Path) -> Result<Self> {
        if let Some(parent) = path.parent().filter(|p| !p.as_os_str().is_empty()) {
            std::fs::create_dir_all(parent)?;
        }
        let db = Connection::open(path)?;
        db.busy_timeout(std::time::Duration::from_millis(500))?;
        db.execute_batch(
            "PRAGMA journal_mode=WAL; PRAGMA synchronous=NORMAL;
            PRAGMA journal_size_limit=4194304;
            CREATE TABLE IF NOT EXISTS samples (
                node TEXT NOT NULL, received_ms INTEGER NOT NULL, source_ms INTEGER NOT NULL,
                cpu REAL, memory_percent REAL, gpu REAL, gpu_board_watts REAL,
                PRIMARY KEY(node, received_ms)
            ) WITHOUT ROWID;
            CREATE INDEX IF NOT EXISTS samples_received ON samples(received_ms);",
        )?;
        Ok(Self { db })
    }

    pub fn append(&mut self, snapshot: &HostSnapshot, received_ms: u64) -> Result<()> {
        let received_ms = i64::try_from(received_ms)?;
        let source_ms = i64::try_from(snapshot.sampled_at_unix_ms)?;
        let memory = memory_percent(snapshot);
        let gpu = snapshot
            .gpus
            .iter()
            .filter_map(|g| g.utilization_percent.value().copied())
            .reduce(f64::max);
        // Keep only same-scope GPU board power here; never combine CPU/SoC/device sensors.
        let watts: Option<f64> = snapshot
            .power
            .iter()
            .filter(|p| p.scope == PowerScope::GpuBoard)
            .map(|p| p.watts.value().copied())
            .collect::<Option<Vec<_>>>()
            .filter(|v| !v.is_empty())
            .map(|v| v.iter().sum());
        self.db.execute(
            "INSERT OR IGNORE INTO samples VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                snapshot.node_id,
                received_ms,
                source_ms,
                snapshot.cpu_utilization_percent.value().copied(),
                memory,
                gpu,
                watts
            ],
        )?;
        Ok(())
    }

    pub fn history(&self, node: &str, since_ms: u64) -> Result<Vec<HistoryPoint>> {
        let mut statement = self.db.prepare("SELECT received_ms, cpu, memory_percent, gpu FROM
            (SELECT received_ms, cpu, memory_percent, gpu FROM samples WHERE node=?1 AND received_ms>=?2
            ORDER BY received_ms DESC LIMIT 720) ORDER BY received_ms")?;
        Ok(statement
            .query_map(params![node, i64::try_from(since_ms)?], |r| {
                Ok(HistoryPoint {
                    received_at_ms: r.get::<_, i64>(0)? as u64,
                    cpu_percent: r.get(1)?,
                    memory_percent: r.get(2)?,
                    gpu_percent: r.get(3)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?)
    }

    pub fn prune(&mut self, before_ms: u64) -> Result<()> {
        self.db.execute(
            "DELETE FROM samples WHERE received_ms < ?1",
            [i64::try_from(before_ms)?],
        )?;
        self.db.execute_batch("PRAGMA wal_checkpoint(PASSIVE);")?;
        Ok(())
    }
}

pub fn memory_percent(s: &HostSnapshot) -> Option<f64> {
    let total = *s.memory.total_bytes.value()?;
    let available = *s.memory.available_bytes.value()?;
    (total > 0 && available <= total).then(|| (total - available) as f64 / total as f64 * 100.0)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn sqlite_preserves_missing_values_and_bounds_history() -> Result<()> {
        let mut store = Store::open(Path::new(":memory:"))?;
        for ts in 0..800 {
            store.db.execute(
                "INSERT INTO samples VALUES ('node', ?1, ?1, 0.0, NULL, NULL, NULL)",
                [ts],
            )?;
        }
        let points = store.history("node", 0)?;
        assert_eq!(points.len(), 720);
        assert_eq!(points[0].received_at_ms, 80);
        assert_eq!(points[0].cpu_percent, Some(0.0));
        assert_eq!(points[0].memory_percent, None);
        store.prune(790)?;
        assert_eq!(store.history("node", 0)?.len(), 10);
        assert!(store.history("other", 0)?.is_empty());
        Ok(())
    }
}
