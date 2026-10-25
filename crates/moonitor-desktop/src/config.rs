use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};
use std::{collections::HashSet, path::Path};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum NodeKind {
    Local,
    Ssh,
    Unavailable,
}

#[derive(Debug, Clone, Copy, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum VisualKind {
    Laptop,
    Tower,
    #[default]
    Compact,
    Board,
    Phone,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct NodeConfig {
    pub id: String,
    pub label: String,
    pub kind: NodeKind,
    #[serde(default)]
    pub visual_kind: VisualKind,
    pub ssh_alias: Option<String>,
    #[serde(default)]
    pub ssh_user: Option<String>,
    #[serde(default)]
    pub ssh_port: Option<u16>,
    pub remote_agent: Option<String>,
    #[serde(default)]
    pub note: String,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub nodes: Vec<NodeConfig>,
}

pub fn safe_id(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 64
        && !value.starts_with('-')
        && value
            .bytes()
            .all(|c| c.is_ascii_alphanumeric() || b"-_.".contains(&c))
}

pub fn safe_destination(value: &str) -> bool {
    safe_id(value) || value.parse::<std::net::IpAddr>().is_ok()
}

impl std::fmt::Display for NodeKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Local => "Questo computer",
            Self::Ssh => "SSH",
            Self::Unavailable => "Solo inventario",
        })
    }
}

impl Config {
    pub fn load(path: Option<&Path>) -> Result<Self> {
        let config = match path {
            Some(p) => toml::from_str(&{
                if std::fs::metadata(p)?.len() > 1024 * 1024 {
                    bail!("Configurazione oltre il budget di 1 MiB");
                }
                std::fs::read_to_string(p)
                    .with_context(|| format!("Cannot read {}", p.display()))?
            })?,
            None => Self {
                nodes: vec![NodeConfig {
                    id: "local".into(),
                    label: "This computer".into(),
                    kind: NodeKind::Local,
                    visual_kind: VisualKind::default(),
                    ssh_alias: None,
                    ssh_user: None,
                    ssh_port: None,
                    remote_agent: None,
                    note: String::new(),
                }],
            },
        };
        config.validate()?;
        Ok(config)
    }

    pub fn validate(&self) -> Result<()> {
        let mut ids = HashSet::new();
        let mut locals = 0;
        for node in &self.nodes {
            if !safe_id(&node.id) || !ids.insert(&node.id) {
                bail!("Invalid or duplicate node ID");
            }
            if node.label.trim().is_empty()
                || node.label.len() > 128
                || node.label.chars().any(char::is_control)
            {
                bail!("Invalid node label");
            }
            match node.kind {
                NodeKind::Local => locals += 1,
                NodeKind::Ssh => {
                    if !node.ssh_alias.as_deref().is_some_and(safe_destination) {
                        bail!("SSH: inserisci un alias, FQDN o indirizzo IP valido");
                    }
                    if node.ssh_user.as_deref().is_some_and(|u| !safe_id(u))
                        || node.ssh_port == Some(0)
                    {
                        bail!("Utente o porta SSH non validi");
                    }
                    let path = node
                        .remote_agent
                        .as_deref()
                        .unwrap_or(".local/bin/moonitor-agent");
                    if path.is_empty()
                        || path.starts_with('-')
                        || !path
                            .bytes()
                            .all(|c| c.is_ascii_alphanumeric() || b"/._-".contains(&c))
                    {
                        bail!("Unsafe remote agent path");
                    }
                }
                NodeKind::Unavailable => (),
            }
        }
        if locals > 1 {
            bail!("Only one local collector is allowed");
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn ssh_alias_cannot_be_an_option_or_shell_expression() {
        for s in ["-oProxyCommand=evil", "host;id", "a b", "$(id)", ""] {
            assert!(!safe_id(s));
        }
        assert!(safe_id("compute-vpn"));
    }
    #[test]
    fn empty_and_large_inventories_are_valid() {
        assert!(Config { nodes: vec![] }.validate().is_ok());
        let nodes = crate::fixtures::fleet(32)
            .into_iter()
            .map(|n| n.config)
            .collect::<Vec<_>>();
        assert!(
            Config {
                nodes: nodes.clone()
            }
            .validate()
            .is_ok()
        );
        let mut too_many = nodes;
        let mut extra = too_many[0].clone();
        extra.id = "extra".into();
        too_many.push(extra);
        assert!(Config { nodes: too_many }.validate().is_ok());
        let nodes = crate::fixtures::fleet(500)
            .into_iter()
            .map(|n| n.config)
            .collect();
        assert!(Config { nodes }.validate().is_ok());
    }
    #[test]
    fn duplicate_ids_are_rejected() {
        let c = Config {
            nodes: vec![
                NodeConfig {
                    id: "same".into(),
                    label: "One".into(),
                    kind: NodeKind::Local,
                    visual_kind: VisualKind::default(),
                    ssh_alias: None,
                    ssh_user: None,
                    ssh_port: None,
                    remote_agent: None,
                    note: String::new()
                };
                2
            ],
        };
        assert!(c.validate().is_err());
    }
}
