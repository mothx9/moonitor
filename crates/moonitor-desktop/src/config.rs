use anyhow::{Context, Result, bail};
use serde::Deserialize;
use std::{collections::HashSet, path::Path};

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NodeKind {
    Local,
    Ssh,
    Unavailable,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NodeConfig {
    pub id: String,
    pub label: String,
    pub kind: NodeKind,
    pub ssh_alias: Option<String>,
    pub remote_agent: Option<String>,
    #[serde(default)]
    pub note: String,
}

#[derive(Clone, Deserialize)]
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

impl Config {
    pub fn load(path: Option<&Path>) -> Result<Self> {
        let config = match path {
            Some(p) => toml::from_str(
                &std::fs::read_to_string(p)
                    .with_context(|| format!("Cannot read {}", p.display()))?,
            )?,
            None => Self {
                nodes: vec![NodeConfig {
                    id: "macbook".into(),
                    label: "This computer".into(),
                    kind: NodeKind::Local,
                    ssh_alias: None,
                    remote_agent: None,
                    note: String::new(),
                }],
            },
        };
        config.validate()?;
        Ok(config)
    }

    fn validate(&self) -> Result<()> {
        if self.nodes.is_empty() || self.nodes.len() > 32 {
            bail!("Configure between 1 and 32 nodes");
        }
        let mut ids = HashSet::new();
        let mut locals = 0;
        for node in &self.nodes {
            if !safe_id(&node.id) || !ids.insert(&node.id) {
                bail!("Invalid or duplicate node ID");
            }
            if node.label.is_empty() || node.label.len() > 128 {
                bail!("Invalid node label");
            }
            match node.kind {
                NodeKind::Local => locals += 1,
                NodeKind::Ssh => {
                    if !node.ssh_alias.as_deref().is_some_and(safe_id) {
                        bail!("SSH node requires a safe host alias");
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
        assert!(safe_id("spark-vpn"));
    }
    #[test]
    fn duplicate_ids_are_rejected() {
        let c = Config {
            nodes: vec![
                NodeConfig {
                    id: "same".into(),
                    label: "One".into(),
                    kind: NodeKind::Local,
                    ssh_alias: None,
                    remote_agent: None,
                    note: String::new()
                };
                2
            ],
        };
        assert!(c.validate().is_err());
    }
}
