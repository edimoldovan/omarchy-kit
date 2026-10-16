//! The dropdown: the choice made, with a chevron after it, and all the
//! choices in a list below it while it is open.

use egui::{Align2, FontId, Id, Popup, Pos2, Rect, Response, Sense, Shape, Stroke, Ui, Vec2, Widget};

use crate::{Ink, Item, Size};

/// A choice among a few that do not all fit on screen at once: the chosen
/// one in a field with a chevron, the list below it when clicked, each line
/// marked like any item. A click on a line chooses it and closes the list;
/// a click elsewhere or Esc closes it. `changed()` when the choice moved.
#[must_use = "a dropdown is drawn with ui.add(dropdown)"]
pub struct Dropdown<'a> {
    selected: &'a mut usize,
    options: &'a [&'a str],
    placeholder: String,
    size: Size,
    width: Option<f32>,
    id: Option<Id>,
}

impl<'a> Dropdown<'a> {
    /// `selected` out of range shows the placeholder: nothing chosen yet.
    pub fn new(selected: &'a mut usize, options: &'a [&'a str]) -> Dropdown<'a> {
        Dropdown { selected, options, placeholder: String::new(), size: Size::default(), width: None, id: None }
    }

    /// What it says while nothing is chosen.
    pub fn placeholder(mut self, placeholder: impl Into<String>) -> Dropdown<'a> {
        self.placeholder = placeholder.into();
        self
    }

    pub fn size(mut self, size: Size) -> Dropdown<'a> {
        self.size = size;
        self
    }

    pub fn small(self) -> Dropdown<'a> {
        self.size(Size::Small)
    }

    /// As wide as `width` rather than the room there is.
    pub fn width(mut self, width: f32) -> Dropdown<'a> {
        self.width = Some(width);
        self
    }

    /// Where egui keeps the list's state, when two dropdowns would share it.
    pub fn id_salt(mut self, salt: impl std::hash::Hash) -> Dropdown<'a> {
        self.id = Some(Id::new(salt));
        self
    }
}

/// The label a dropdown shows: the chosen option, or the placeholder.
fn shown<'a>(selected: usize, options: &[&'a str], placeholder: &'a str) -> (&'a str, bool) {
    match options.get(selected) {
        Some(option) => (option, true),
        None => (placeholder, false),
    }
}

impl Widget for Dropdown<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        let size = self.size;
        let ink = Ink::of(ui);
        let width = self.width.unwrap_or_else(|| ui.available_width());
        let (rect, mut response) = ui.allocate_exact_size(Vec2::new(width, size.height()), Sense::click());
        let id = self.id.unwrap_or_else(|| response.id.with("omarchy-kit-dropdown"));

        if ui.is_rect_visible(rect) {
            let step = if response.is_pointer_button_down_on() { 2.0 } else if response.hovered() { 1.0 } else { 0.0 };
            let painter = ui.painter();
            painter.rect_filled(rect, size.radius(), ink.shade(0.10 + 0.06 * step));
            // The chevron, drawn: no font has to carry it.
            let chevron = Pos2::new(rect.right() - size.pad() - 4.0, rect.center().y);
            painter.add(Shape::line(
                vec![Pos2::new(chevron.x - 4.0, chevron.y - 2.0), Pos2::new(chevron.x, chevron.y + 2.5), Pos2::new(chevron.x + 4.0, chevron.y - 2.0)],
                Stroke::new(1.5_f32, ink.weak),
            ));
            let (label, chosen) = shown(*self.selected, self.options, &self.placeholder);
            let clip = Rect::from_min_max(rect.min, Pos2::new(chevron.x - 12.0, rect.max.y));
            painter.with_clip_rect(clip).text(
                Pos2::new(rect.left() + size.pad(), rect.center().y),
                Align2::LEFT_CENTER,
                label,
                FontId::proportional(size.text()),
                if chosen { ink.fg } else { ink.weak },
            );
        }

        let mut chosen = None;
        let frame = crate::palette::popup_frame(ui.ctx());
        let inner = rect.width() - 2.0 * crate::palette::MARGIN;
        Popup::menu(&response).id(id).width(rect.width()).gap(4.0).frame(frame).show(|ui| {
            ui.set_width(inner);
            ui.spacing_mut().item_spacing.y = 2.0;
            let line = Size::Small;
            for (i, option) in self.options.iter().enumerate() {
                let (row, hit) = ui.allocate_exact_size(Vec2::new(inner, line.height()), Sense::click());
                Item { open: i == *self.selected, keyed: false, hovered: hit.hovered(), size: line }.paint(ui, row);
                ui.painter().text(
                    Pos2::new(row.left() + line.pad(), row.center().y),
                    Align2::LEFT_CENTER,
                    option,
                    FontId::proportional(line.text()),
                    ink.fg,
                );
                if hit.clicked() {
                    chosen = Some(i);
                }
            }
        });
        if let Some(i) = chosen
            && i != *self.selected
        {
            *self.selected = i;
            response.mark_changed();
        }
        response
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use egui::{Context, Event, PointerButton, RawInput};

    #[test]
    fn shows_the_choice_or_the_placeholder() {
        let options = ["One", "Two"];
        assert_eq!(shown(1, &options, "Pick"), ("Two", true));
        assert_eq!(shown(5, &options, "Pick"), ("Pick", false));
    }

    /// A click opens the list, a click on its second line chooses it.
    #[test]
    fn a_click_opens_the_list_and_a_line_chooses() {
        let ctx = Context::default();
        let options = ["One", "Two", "Three"];
        let mut selected = 0usize;
        let mut changed = false;
        let id = Id::new("dropdown-test");
        let mut frame = |events: Vec<Event>, selected: &mut usize| {
            let input = RawInput { events, ..RawInput::default() };
            let _ = ctx.run(input, |ctx| {
                egui::CentralPanel::default().frame(egui::Frame::NONE).show(ctx, |ui| {
                    changed |= ui.add(Dropdown::new(selected, &options).width(200.0).id_salt("dropdown-test")).changed();
                });
            });
        };
        let click = |at: Pos2| {
            vec![
                Event::PointerMoved(at),
                Event::PointerButton { pos: at, button: PointerButton::Primary, pressed: true, modifiers: Default::default() },
                Event::PointerButton { pos: at, button: PointerButton::Primary, pressed: false, modifiers: Default::default() },
            ]
        };
        frame(vec![], &mut selected);
        frame(click(Pos2::new(100.0, 16.0)), &mut selected);
        frame(vec![], &mut selected);
        assert!(Popup::is_id_open(&ctx, id), "the list is open");
        // Below the field by the gap and the frame's margin: line i at its height plus spacing.
        let second = Size::Regular.height() + 4.0 + crate::palette::MARGIN + Size::Small.height() + 2.0 + Size::Small.height() / 2.0;
        frame(click(Pos2::new(100.0, second)), &mut selected);
        frame(vec![], &mut selected);
        assert_eq!(selected, 1, "the second line was chosen");
        assert!(changed);
        assert!(!Popup::is_id_open(&ctx, id), "choosing closes the list");
    }
}
