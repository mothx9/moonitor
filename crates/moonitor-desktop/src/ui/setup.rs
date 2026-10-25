use super::*;
use crate::{
    config::{NodeConfig, NodeKind, VisualKind},
    profile::Preferences,
};
use iced::widget::column;
use iced::widget::pick_list;

#[derive(Debug, Clone, Copy)]
pub enum Field {
    Label,
    Destination,
    User,
    Port,
    Agent,
    Note,
}
#[derive(Debug, Clone)]
pub struct HostForm {
    pub id: String,
    pub label: String,
    pub kind: NodeKind,
    pub destination: String,
    pub user: String,
    pub port: String,
    pub agent: String,
    pub note: String,
    pub visual_kind: VisualKind,
}
impl HostForm {
    fn from_node(n: &NodeConfig) -> Self {
        Self {
            id: n.id.clone(),
            label: n.label.clone(),
            kind: n.kind,
            destination: n.ssh_alias.clone().unwrap_or_default(),
            user: n.ssh_user.clone().unwrap_or_default(),
            port: n.ssh_port.map(|p| p.to_string()).unwrap_or_default(),
            agent: n
                .remote_agent
                .clone()
                .unwrap_or_else(|| ".local/bin/moonitor-agent".into()),
            note: n.note.clone(),
            visual_kind: n.visual_kind,
        }
    }
    fn node(&self) -> anyhow::Result<NodeConfig> {
        let ssh = self.kind == NodeKind::Ssh;
        Ok(NodeConfig {
            id: self.id.clone(),
            label: self.label.trim().into(),
            kind: self.kind,
            visual_kind: self.visual_kind,
            ssh_alias: ssh.then(|| self.destination.trim().into()),
            ssh_user: (ssh && !self.user.trim().is_empty()).then(|| self.user.trim().into()),
            ssh_port: if ssh && !self.port.trim().is_empty() {
                Some(
                    self.port
                        .parse()
                        .map_err(|_| anyhow::anyhow!("Porta SSH: usa un numero tra 1 e 65535"))?,
                )
            } else {
                None
            },
            remote_agent: ssh.then(|| self.agent.trim().into()),
            note: self.note.trim().into(),
        })
    }
}
impl App {
    pub(super) fn save_preferences(&mut self, preferences: Preferences) -> bool {
        if let Some(profile) = &mut self.profile
            && let Err(e) = profile.save(profile.config.clone(), preferences.clone())
        {
            self.setup_error = Some(e.to_string());
            return false;
        }
        self.preferences = preferences;
        self.setup_error = None;
        true
    }
    pub(super) fn setup_update(&mut self, message: Message) -> Task<Message> {
        if self.applying {
            return Task::none();
        }
        self.setup_error = None;
        self.setup_notice = None;
        match message {
            Message::NewHost => {
                let id = if let Some(profile) = &self.profile {
                    profile.new_id()
                } else {
                    Ok(format!("test-{}", now_ms()))
                };
                match id {
                    Ok(id) => {
                        self.form = Some(HostForm {
                            id,
                            label: String::new(),
                            kind: NodeKind::Ssh,
                            destination: String::new(),
                            user: String::new(),
                            port: String::new(),
                            agent: ".local/bin/moonitor-agent".into(),
                            note: String::new(),
                            visual_kind: VisualKind::Compact,
                        })
                    }
                    Err(e) => self.setup_error = Some(e.to_string()),
                }
                self.section = Section::Settings;
                self.remove_host = None;
            }
            Message::EditHost(id) => {
                self.form = self
                    .nodes
                    .iter()
                    .find(|n| n.config.id == id)
                    .map(|n| HostForm::from_node(&n.config));
                self.remove_host = None;
            }
            Message::HostField(field, value) => {
                if let Some(f) = &mut self.form {
                    *match field {
                        Field::Label => &mut f.label,
                        Field::Destination => &mut f.destination,
                        Field::User => &mut f.user,
                        Field::Port => &mut f.port,
                        Field::Agent => &mut f.agent,
                        Field::Note => &mut f.note,
                    } = value.chars().take(256).collect();
                }
            }
            Message::HostKind(kind) => {
                if let Some(f) = &mut self.form {
                    f.kind = kind
                }
            }
            Message::CancelHost => {
                self.form = None;
                self.remove_host = None
            }
            Message::RequestRemove(id) => {
                if self.nodes.iter().any(|n| n.config.id == id) {
                    self.remove_host = Some(id);
                    self.dialog_confirm_focused = false;
                }
            }
            Message::SaveHost => {
                if let Some(form) = &self.form {
                    let result = form.node().and_then(|node| {
                        let mut config = Config {
                            nodes: self.nodes.iter().map(|n| n.config.clone()).collect(),
                        };
                        if let Some(old) = config.nodes.iter_mut().find(|n| n.id == node.id) {
                            *old = node
                        } else {
                            config.nodes.push(node)
                        }
                        config.validate()?;
                        Ok(config)
                    });
                    match result {
                        Ok(config) => return self.apply_config(config),
                        Err(e) => self.setup_error = Some(e.to_string()),
                    }
                }
            }
            Message::RemoveHost(id) => {
                if self.remove_host.as_deref() != Some(&id) {
                    return Task::none();
                }
                let config = Config {
                    nodes: self
                        .nodes
                        .iter()
                        .filter(|n| n.config.id != id)
                        .map(|n| n.config.clone())
                        .collect(),
                };
                return self.apply_config(config);
            }
            _ => (),
        }
        Task::none()
    }
    fn apply_config(&mut self, config: Config) -> Task<Message> {
        if let Some(profile) = &mut self.profile
            && let Err(e) = profile.save(config.clone(), self.preferences.clone())
        {
            self.setup_error = Some(e.to_string());
            return Task::none();
        }
        self.nodes = config
            .nodes
            .iter()
            .map(|c| {
                self.nodes
                    .iter()
                    .find(|n| n.config == *c)
                    .cloned()
                    .unwrap_or_else(|| NodeState {
                        config: c.clone(),
                        capacity_limited: false,
                        sample: None,
                        last_seen: None,
                        error: None,
                        history: Default::default(),
                    })
            })
            .collect();
        self.selected =
            FleetViewModel::new(&self.nodes, self.selected.as_deref(), Instant::now()).selected_id;
        self.form = None;
        self.remove_host = None;
        self.paused = false;
        self.frozen = None;
        if let Some(old) = self._engine.take() {
            self.applying = true;
            let db = self.data_dir.join("history.sqlite");
            return Task::perform(
                async move {
                    drop(old);
                    std::sync::Arc::new(std::sync::Mutex::new(Some(Engine::start(config, db))))
                },
                Message::EngineReady,
            );
        }
        self.setup_notice = Some("Configurazione salvata.".into());
        Task::none()
    }
    pub(super) fn setup_view(&self) -> Element<'_, Message> {
        let mut hosts = column![row![
            text("Host configurati").size(15).font(STRONG),
            Space::new().width(Fill),
            small_button("+ Aggiungi host", Message::NewHost)
        ]]
        .spacing(8);
        for n in &self.nodes {
            hosts = hosts.push(
                row![
                    column![
                        text(&n.config.label).size(12).font(MEDIUM),
                        text(format!("{} · {}", n.config.kind, n.config.id))
                            .size(10)
                            .tone(MUTED)
                    ]
                    .width(Fill)
                    .spacing(3),
                    small_button("Modifica", Message::EditHost(n.config.id.clone())),
                    small_button("Rimuovi", Message::RequestRemove(n.config.id.clone()))
                ]
                .spacing(6)
                .align_y(alignment::Vertical::Center),
            );
            hosts = hosts.push(widget::rule::horizontal(1));
        }
        if self.nodes.is_empty() {
            hosts = hosts.push(
                text("Nessun host. Aggiungi questo computer per iniziare.")
                    .size(12)
                    .tone(MUTED),
            );
        }
        let mut content = column![
            text("Setup").size(23).font(STRONG),
            text(
                "Configurazione effettiva · salvataggio locale · nessuna credenziale nel registro"
            )
            .size(12)
            .tone(MUTED),
        ]
        .spacing(12);
        if self.applying {
            content = content.push(notice(
                "Configurazione salvata. Aggiornamento della raccolta…",
            ));
        }
        if let Some(error) = &self.setup_error {
            content = content.push(notice(error));
        }
        if let Some(message) = &self.setup_notice {
            content = content.push(text(message).size(12).tone(HEALTHY));
        }
        if let Some(f) = &self.form {
            let mut fields = column![
                text(if self.nodes.iter().any(|n| n.config.id == f.id) {
                    "Modifica host"
                } else {
                    "Nuovo host"
                })
                .size(16)
                .font(STRONG),
                text(format!("ID stabile · {}", f.id)).size(10).tone(MUTED),
                row![
                    field("Nome visualizzato", &f.label, Field::Label),
                    column![
                        text("Metodo").size(11).tone(MUTED),
                        pick_list(
                            [NodeKind::Local, NodeKind::Ssh, NodeKind::Unavailable],
                            Some(f.kind),
                            Message::HostKind
                        )
                        .width(Fill)
                        .text_size(12)
                        .padding(9)
                    ]
                    .spacing(5)
                    .width(Fill)
                ]
                .spacing(12)
            ]
            .spacing(10);
            if f.kind == NodeKind::Ssh {
                fields=fields.push(row![field("Alias SSH / FQDN / IP",&f.destination,Field::Destination),field("Utente · opzionale",&f.user,Field::User),field("Porta · vuota = SSH config",&f.port,Field::Port)].spacing(12))
                    .push(field("Percorso agente già installato",&f.agent,Field::Agent))
                    .push(text("SSH usa chiavi e known_hosts del sistema. Host key sconosciute vengono rifiutate. Nessun deploy automatico.").size(11).tone(MUTED));
            }
            fields = fields
                .push(field("Nota · opzionale", &f.note, Field::Note))
                .push(
                    row![
                        small_button("Salva host", Message::SaveHost),
                        small_button("Annulla modifica", Message::CancelHost)
                    ]
                    .spacing(10),
                );
            content = content.push(instrument::panel("HOST EDITOR", fields));
        }
        content = content.push(instrument::panel(
            "HOSTS & CONNECTIONS",
            scrollable(hosts).height(210),
        ));
        content = content.push(instrument::panel(
            "APPEARANCE",
            column![
                row![
                    small_button(
                        if !self.dark {
                            "Moonlight · attivo"
                        } else {
                            "Moonlight"
                        },
                        Message::Theme(false)
                    ),
                    small_button(
                        if self.dark {
                            "Deep Space · attivo"
                        } else {
                            "Deep Space"
                        },
                        Message::Theme(true)
                    ),
                    small_button(
                        if self.preferences.comfortable {
                            "Densità: comoda"
                        } else {
                            "Densità: compatta"
                        },
                        Message::Density
                    ),
                    small_button(
                        if self.preferences.inspector {
                            "Inspector: aperto"
                        } else {
                            "Inspector: chiuso"
                        },
                        Message::Inspector
                    )
                ]
                .spacing(8),
                text("Preferenze persistenti. Il tema da riga di comando prevale solo all'avvio.")
                    .size(11)
                    .tone(MUTED)
            ]
            .spacing(10),
        ));
        content.push(instrument::panel("DATA & SECURITY",column![
            text("Raccolta nel desktop · SSH verificato · SQLite locale · storico 7 giorni, riepiloghi ogni 5 s").size(12),
            text("Controller persistente, discovery LAN/VPN, pairing e installazione agenti: non ancora disponibili. Il registro importa una sola volta il precedente fleet.toml, conservando ID e file originale.").size(11).tone(MUTED),
            text(format!("Registro: {}",self.data_dir.join("settings.sqlite").display())).size(10).tone(MUTED),
        ].spacing(8))).into()
    }
}
fn field<'a>(label: &'static str, value: &'a str, field: Field) -> Element<'a, Message> {
    column![
        text(label).size(11).tone(MUTED),
        text_input(label, value)
            .id(format!("host-{field:?}"))
            .on_input(move |v| Message::HostField(field, v))
            .size(12)
            .padding(9)
    ]
    .spacing(5)
    .width(Fill)
    .into()
}
