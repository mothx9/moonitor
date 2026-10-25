//! Transactional, versioned user settings. No credentials and no telemetry here.
use crate::config::Config;
use anyhow::{Context, Result, bail};
use rusqlite::{Connection, OptionalExtension, params};
use serde::{Deserialize, Serialize};
use std::path::Path;

const MAX_PROFILE_BYTES: usize = 1024 * 1024;
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(default, deny_unknown_fields)]
pub struct Preferences {
    pub dark: bool,
    pub comfortable: bool,
    pub inspector: bool,
    pub expanded_rail: bool,
}
pub struct Profile {
    connection: Connection,
    pub revision: i64,
    pub config: Config,
    pub preferences: Preferences,
}
impl Profile {
    pub fn open(path: &Path, legacy: Option<&Path>) -> Result<Self> {
        let connection = Connection::open(path)?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600))?;
        }
        connection.busy_timeout(std::time::Duration::from_millis(250))?;
        let version: i64 = connection.query_row("PRAGMA user_version", [], |r| r.get(0))?;
        if version > 1 {
            bail!("Registro creato da una versione più recente di Moonitor");
        }
        connection.execute_batch("PRAGMA journal_mode=WAL;
            BEGIN IMMEDIATE;
            CREATE TABLE IF NOT EXISTS profile (singleton INTEGER PRIMARY KEY CHECK(singleton=1), revision INTEGER NOT NULL, config TEXT NOT NULL, preferences TEXT NOT NULL, imported_from TEXT);
            PRAGMA user_version=1;
            COMMIT;")?;
        let row: Option<(i64, String, String)> = connection
            .query_row(
                "SELECT revision,config,preferences FROM profile WHERE singleton=1",
                [],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
            )
            .optional()?;
        let (revision, config, preferences) = if let Some((revision, config, preferences)) = row {
            if config.len() + preferences.len() > MAX_PROFILE_BYTES {
                bail!("Registro oltre il budget di 1 MiB");
            }
            (
                revision,
                serde_json::from_str::<Config>(&config)
                    .context("Registro host non leggibile; il file viene preservato")?,
                serde_json::from_str(&preferences)?,
            )
        } else {
            let mut config = Config::load(legacy)?;
            if legacy.is_none() {
                config.nodes[0].id = new_id(&connection)?;
            }
            let preferences = Preferences::default();
            connection.execute(
                "INSERT INTO profile VALUES(1,1,?1,?2,?3)",
                params![
                    serde_json::to_string(&config)?,
                    serde_json::to_string(&preferences)?,
                    legacy.map(|p| p.to_string_lossy().into_owned())
                ],
            )?;
            (1, config, preferences)
        };
        config.validate()?;
        Ok(Self {
            connection,
            revision,
            config,
            preferences,
        })
    }
    pub fn new_id(&self) -> Result<String> {
        new_id(&self.connection)
    }
    pub fn save(&mut self, config: Config, preferences: Preferences) -> Result<()> {
        config.validate()?;
        let encoded = serde_json::to_string(&config)?;
        let prefs = serde_json::to_string(&preferences)?;
        if encoded.len() + prefs.len() > MAX_PROFILE_BYTES {
            bail!("Registro oltre il budget di 1 MiB");
        }
        let tx = self.connection.transaction()?;
        if tx.execute("UPDATE profile SET revision=revision+1, config=?1, preferences=?2 WHERE singleton=1 AND revision=?3", params![encoded,prefs,self.revision])? != 1 {
            bail!("Configurazione modificata da un'altra finestra. Riapri Moonitor prima di salvare.");
        }
        tx.commit()?;
        self.revision += 1;
        self.config = config;
        self.preferences = preferences;
        Ok(())
    }
}
fn new_id(db: &Connection) -> Result<String> {
    Ok(
        db.query_row("SELECT 'node-' || lower(hex(randomblob(16)))", [], |r| {
            r.get(0)
        })?,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    fn path() -> std::path::PathBuf {
        static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
        std::env::temp_dir().join(format!(
            "moonitor-profile-{}-{}-{}.sqlite",
            NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed),
            std::process::id(),
            moonitor_core::now_ms()
        ))
    }
    #[test]
    fn onboarding_crud_preferences_restart_and_conflict_are_transactional() {
        let path = path();
        let mut p = Profile::open(&path, None).unwrap();
        let local = p.config.nodes[0].id.clone();
        assert!(local.starts_with("node-"));
        let mut stale = Profile::open(&path, None).unwrap();
        let mut config = p.config.clone();
        let mut remote = config.nodes[0].clone();
        remote.id = p.new_id().unwrap();
        remote.kind = crate::config::NodeKind::Ssh;
        remote.ssh_alias = Some("node.example".into());
        remote.label = "Compute".into();
        config.nodes.push(remote.clone());
        let prefs = Preferences {
            dark: true,
            inspector: true,
            ..Default::default()
        };
        p.save(config.clone(), prefs.clone()).unwrap();
        assert!(
            stale
                .save(stale.config.clone(), Default::default())
                .is_err()
        );
        let mut invalid = config.clone();
        invalid.nodes[1].id = local.clone();
        assert!(p.save(invalid, Default::default()).is_err());
        drop(p);
        let mut p = Profile::open(&path, None).unwrap();
        assert_eq!(p.config, config);
        assert_eq!(p.preferences, prefs);
        p.config.nodes[1].label = "Renamed".into();
        p.save(p.config.clone(), p.preferences.clone()).unwrap();
        assert_eq!(p.config.nodes[1].id, remote.id);
        p.config.nodes.remove(1);
        p.save(p.config.clone(), p.preferences.clone()).unwrap();
        drop(p);
        drop(stale);
        assert_eq!(
            Profile::open(&path, None).unwrap().config.nodes[0].id,
            local
        );
        let _ = std::fs::remove_file(path);
    }
    #[test]
    fn legacy_import_is_idempotent_and_preserves_file_and_ids() {
        let db = path();
        let legacy = db.with_extension("toml");
        let text = "[[nodes]]\nid='preserved'\nlabel='Old local'\nkind='local'\n";
        std::fs::write(&legacy, text).unwrap();
        let p = Profile::open(&db, Some(&legacy)).unwrap();
        assert_eq!(p.config.nodes[0].id, "preserved");
        drop(p);
        std::fs::write(&legacy, "broken config").unwrap();
        assert_eq!(
            Profile::open(&db, Some(&legacy)).unwrap().config.nodes[0].id,
            "preserved"
        );
        assert_eq!(std::fs::read_to_string(&legacy).unwrap(), "broken config");
        let _ = std::fs::remove_file(legacy);
        let _ = std::fs::remove_file(db);
    }
}
