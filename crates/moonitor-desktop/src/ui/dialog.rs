//! Scoped confirmation lives outside every scrollable. Default keyboard action is Cancel.
use super::*;
use iced::widget::column;

impl App {
    pub(super) fn dialog_key(&mut self, key: iced::keyboard::Key) -> Task<Message> {
        use iced::keyboard::{Key, key::Named};
        if self.remove_host.is_none() {
            return Task::none();
        }
        match key {
            Key::Named(Named::Escape) => self.setup_update(Message::CancelHost),
            Key::Named(Named::Tab | Named::ArrowLeft | Named::ArrowRight) => {
                self.dialog_confirm_focused = !self.dialog_confirm_focused;
                Task::none()
            }
            Key::Named(Named::Enter | Named::Space) => {
                if self.dialog_confirm_focused {
                    self.setup_update(Message::RemoveHost(self.remove_host.clone().unwrap()))
                } else {
                    self.setup_update(Message::CancelHost)
                }
            }
            _ => Task::none(),
        }
    }
    pub(super) fn with_dialog<'a>(&'a self, content: Element<'a, Message>) -> Element<'a, Message> {
        let Some(id) = &self.remove_host else {
            return content;
        };
        let Some(n) = self.nodes.iter().find(|n| &n.config.id == id) else {
            return content;
        };
        let focus = |selected: bool, destructive: bool| {
            move |theme: &Theme, state| {
                let mut style = if destructive {
                    button::danger(theme, state)
                } else {
                    button::secondary(theme, state)
                };
                if selected {
                    style.border = Border {
                        color: ACCENT,
                        width: 2.0,
                        radius: 6.into(),
                    };
                }
                style
            }
        };
        let dialog = container(
            column![
                text(format!("Rimuovere {}?", n.config.label))
                    .size(22)
                    .font(STRONG),
                text("La raccolta di questo host si fermerà. Lo storico sarà conservato.").size(14),
                text("Tab cambia azione · Invio attiva · Esc annulla")
                    .size(12)
                    .tone(MUTED),
                row![
                    button(text("Annulla").size(14))
                        .padding([10, 18])
                        .style(focus(!self.dialog_confirm_focused, false))
                        .on_press(Message::CancelHost),
                    button(text("Rimuovi host").size(14))
                        .padding([10, 18])
                        .style(focus(self.dialog_confirm_focused, true))
                        .on_press(Message::RemoveHost(id.clone()))
                ]
                .spacing(12)
            ]
            .spacing(18),
        )
        .padding(24)
        .width(520)
        .style(super::panel);
        widget::stack![
            content,
            widget::opaque(container(dialog).center_x(Fill).center_y(Fill).style(|_| {
                container::Style {
                    background: Some(Color::from_rgba(0.02, 0.03, 0.06, 0.64).into()),
                    ..Default::default()
                }
            }))
        ]
        .into()
    }
}
