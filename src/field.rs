//! The text field, and the field's frame around anything else.

use egui::{
    Color32, FontId, Frame, Id, Margin, Painter, Rect, Response, RichText, Sense, Stroke, TextEdit,
    Ui, Vec2, Widget,
};

use crate::{Ink, Size};

/// An icon, drawn in the square it is given, in the colour it is given.
type Icon<'a> = Box<dyn Fn(&Painter, Rect, Color32) + 'a>;

/// A field's frame around anything — chips and the text being typed, a
/// multi-line editor: the field's fill and corner, and the accent around it
/// while what it holds has the keyboard. `add` returns whether it does, and
/// whatever else it likes.
pub fn field<R>(ui: &mut Ui, size: Size, add: impl FnOnce(&mut Ui) -> (bool, R)) -> R {
    let ink = Ink::of(ui);
    let line = size.text() * 1.3;
    let pad_y = ((size.height() - line) / 2.0).max(2.0);
    let mut prepared = Frame::new()
        .fill(ink.shade(0.10))
        .corner_radius(size.radius())
        .inner_margin(Margin::symmetric(size.pad() as i8, pad_y as i8))
        // Always a stroke, so having the keyboard never moves what is inside.
        .stroke(Stroke::new(1.0_f32, Color32::TRANSPARENT))
        .begin(ui);
    prepared.content_ui.set_min_height(line);
    let (focused, inner) = add(&mut prepared.content_ui);
    if focused {
        prepared.frame.stroke = Stroke::new(1.0_f32, ink.accent);
    }
    prepared.end(ui);
    inner
}

/// One line of text in the kit's field, with an icon before it and the key
/// that brings the keyboard to it after.
#[must_use = "a text field is drawn with ui.add(field)"]
pub struct TextField<'a> {
    text: &'a mut String,
    hint: String,
    icon: Option<Icon<'a>>,
    key: Option<String>,
    size: Size,
    width: Option<f32>,
    id: Option<Id>,
}

impl<'a> TextField<'a> {
    pub fn new(text: &'a mut String) -> TextField<'a> {
        TextField {
            text,
            hint: String::new(),
            icon: None,
            key: None,
            size: Size::default(),
            width: None,
            id: None,
        }
    }

    /// What it says while it is empty.
    pub fn hint(mut self, hint: impl Into<String>) -> TextField<'a> {
        self.hint = hint.into();
        self
    }

    pub fn icon(mut self, icon: impl Fn(&Painter, Rect, Color32) + 'a) -> TextField<'a> {
        self.icon = Some(Box::new(icon));
        self
    }

    /// The key that brings the keyboard here, quietly at the end.
    pub fn key(mut self, key: impl Into<String>) -> TextField<'a> {
        self.key = Some(key.into());
        self
    }

    pub fn size(mut self, size: Size) -> TextField<'a> {
        self.size = size;
        self
    }

    pub fn small(self) -> TextField<'a> {
        self.size(Size::Small)
    }

    /// As wide as `width`, frame and all, rather than the room there is.
    pub fn width(mut self, width: f32) -> TextField<'a> {
        self.width = Some(width);
        self
    }

    /// Where egui keeps its state, when two fields would otherwise share it.
    pub fn id_salt(mut self, salt: impl std::hash::Hash) -> TextField<'a> {
        self.id = Some(Id::new(salt));
        self
    }
}

impl Widget for TextField<'_> {
    /// The response is the text's own, so `changed`, `has_focus` and
    /// `lost_focus` say what they always do.
    fn ui(self, ui: &mut Ui) -> Response {
        let size = self.size;
        let ink = Ink::of(ui);
        let outer = self.width.unwrap_or_else(|| ui.available_width());
        let gap = 7.0;
        let key_font = FontId::proportional(size.text() - 2.0);
        let key_w = self.key.as_deref().map_or(0.0, |key| {
            ui.painter()
                .layout_no_wrap(key.to_string(), key_font.clone(), ink.weak)
                .size()
                .x
                + gap
        });
        let icon_w = if self.icon.is_some() { size.icon() + gap } else { 0.0 };
        let text_w = (outer - 2.0 * size.pad() - 2.0 - icon_w - key_w).max(24.0);
        let (icon, key, hint, text) = (self.icon, self.key, self.hint, self.text);
        let id = self.id;
        field(ui, size, |ui| {
            let edit = ui
                .horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = gap;
                    if let Some(icon) = &icon {
                        let (spot, _) = ui.allocate_exact_size(Vec2::splat(size.icon()), Sense::hover());
                        icon(ui.painter(), spot, ink.weak);
                    }
                    let mut edit = TextEdit::singleline(text)
                        .frame(false)
                        .margin(Margin::ZERO)
                        .font(FontId::proportional(size.text()))
                        .hint_text(RichText::new(hint).color(ink.weak))
                        .desired_width(text_w);
                    if let Some(id) = id {
                        edit = edit.id(id);
                    }
                    let edit = ui.add(edit);
                    if let Some(key) = &key {
                        ui.label(RichText::new(key).font(key_font).color(ink.weak));
                    }
                    edit
                })
                .inner;
            (edit.has_focus(), edit)
        })
    }
}
