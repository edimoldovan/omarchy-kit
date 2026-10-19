//! The dialog: a card over the window that asks for something, or a panel
//! that stays beside the work.

use egui::{Align, Align2, Context, Frame, Id, Layout, Modal, RichText, Stroke, Ui, Vec2};

use crate::{Button, Size};

/// Room between the card's edge and what is in it.
const MARGIN: f32 = 16.0;

/// A card with a title, a × that closes it, and whatever the app puts in it.
///
/// Modal unless told otherwise: it sits over the window, which is dimmed
/// behind it and takes no clicks; Esc, the × or a click beside it closes
/// it. A modeless one stays beside the work — the window behind it keeps
/// working, it can be moved by dragging, and only the × closes it.
#[must_use = "a dialog is drawn with dialog.show(ctx, …)"]
pub struct Dialog {
    title: String,
    id: Id,
    modal: bool,
    width: f32,
    resizable: Option<Vec2>,
}

/// What a dialog's frame gave back: what the app's closure returned, and
/// whether the dialog was asked to close.
pub struct DialogResponse<R> {
    pub inner: Option<R>,
    pub closed: bool,
}

impl Dialog {
    pub fn new(title: impl Into<String>) -> Dialog {
        let title = title.into();
        Dialog {
            id: Id::new(("omarchy-kit-dialog", title.clone())),
            title,
            modal: true,
            width: 380.0,
            resizable: None,
        }
    }

    /// Where egui keeps its state, when two dialogs share a title.
    pub fn id_salt(mut self, salt: impl std::hash::Hash) -> Dialog {
        self.id = Id::new(("omarchy-kit-dialog", salt));
        self
    }

    /// Beside the work instead of over it.
    pub fn modeless(mut self) -> Dialog {
        self.modal = false;
        self
    }

    /// How wide what is in it is; 380 points when unset.
    pub fn width(mut self, width: f32) -> Dialog {
        self.width = width;
        self
    }

    /// A modeless panel the user can resize, starting at `size`.
    pub fn resizable(mut self, size: Vec2) -> Dialog {
        self.modal = false;
        self.resizable = Some(size);
        self
    }

    /// Draws it, with `add` drawing what is in it.
    pub fn show<R>(self, ctx: &Context, add: impl FnOnce(&mut Ui) -> R) -> DialogResponse<R> {
        let mut closed = false;
        let width = self.width.min(ctx.content_rect().width() - 2.0 * (16.0 + MARGIN));
        let resizable = self.resizable;
        let title = self.title.clone();
        let modal = self.modal;
        let body = |ui: &mut Ui| -> R {
            // To a screen reader and a test, a dialog named by its title.
            ui.ctx().accesskit_node_builder(ui.id(), |node| {
                node.set_role(egui::accesskit::Role::Dialog);
                node.set_label(title.as_str());
                if modal {
                    node.set_modal();
                }
            });
            if resizable.is_none() {
                ui.set_width(width);
            }
            ui.horizontal(|ui| {
                ui.label(RichText::new(&title).size(16.0).strong());
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    let close = Button::icon_only(|painter: &egui::Painter, rect: egui::Rect, colour| {
                        crate::cross(painter, rect.center(), rect.width() * 0.32, colour);
                    })
                    .quiet()
                    .small()
                    .tip("Close");
                    if ui.add(close).clicked() {
                        closed = true;
                    }
                });
            });
            ui.add_space(8.0);
            add(ui)
        };
        if self.modal {
            let shown = Modal::new(self.id).frame(card(ctx)).show(ctx, body);
            let asked = shown.should_close();
            return DialogResponse { inner: Some(shown.inner), closed: closed || asked };
        }
        let screen = ctx.content_rect();
        let mut window = egui::Window::new(&self.title)
            .id(self.id)
            .title_bar(false)
            .collapsible(false)
            .frame(card(ctx))
            .pivot(Align2::CENTER_TOP)
            .default_pos(screen.center_top() + Vec2::new(0.0, 96.0));
        window = match resizable {
            Some(size) => window.resizable(true).default_size(size),
            None => window.resizable(false),
        };
        let shown = window.show(ctx, body);
        DialogResponse { inner: shown.and_then(|s| s.inner), closed }
    }
}

/// The dialog's card: the window's fill, a hairline, and a corner that keeps
/// the buttons inside it concentric.
fn card(ctx: &Context) -> Frame {
    let style = ctx.style();
    let v = &style.visuals;
    Frame::popup(&style)
        .fill(v.panel_fill)
        .stroke(Stroke::new(1.0_f32, crate::theme::mix(v.panel_fill, v.text_color(), 0.14)))
        .corner_radius(Size::Regular.radius() + MARGIN)
        .inner_margin(MARGIN)
}

#[cfg(test)]
mod tests {
    use super::*;
    use egui::{Event, Key, Modifiers, RawInput};

    /// A modal dialog draws what it is given and closes on Esc.
    #[test]
    fn a_modal_dialog_draws_its_inside_and_closes_on_escape() {
        let ctx = Context::default();
        let frame = |events: Vec<Event>| {
            let mut out = None;
            let input = RawInput { events, ..RawInput::default() };
            let _ = ctx.run(input, |ctx| {
                out = Some(Dialog::new("Go to").show(ctx, |_ui| 7));
            });
            out.expect("drawn")
        };
        let shown = frame(vec![]);
        assert_eq!(shown.inner, Some(7));
        assert!(!shown.closed);
        let escape = Event::Key {
            key: Key::Escape,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: Modifiers::NONE,
        };
        assert!(frame(vec![escape]).closed);
    }
}
