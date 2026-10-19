//! The checkbox: one thing on or off, with what it is beside it.

use egui::{FontId, Pos2, Rect, Response, Sense, Stroke, StrokeKind, Ui, Vec2, Widget, WidgetInfo, WidgetType};

use crate::{Ink, Size, on};

/// One thing on or off: a box — filled with the accent and ticked while on —
/// and what it is beside it. A click anywhere on it, or Space while it has
/// the keyboard, flips it; `changed()` when it did.
#[must_use = "a checkbox is drawn with ui.add(checkbox)"]
pub struct Checkbox<'a> {
    checked: &'a mut bool,
    label: String,
    size: Size,
}

impl<'a> Checkbox<'a> {
    pub fn new(checked: &'a mut bool, label: impl Into<String>) -> Checkbox<'a> {
        Checkbox {
            checked,
            label: label.into(),
            size: Size::default(),
        }
    }

    pub fn size(mut self, size: Size) -> Checkbox<'a> {
        self.size = size;
        self
    }

    pub fn small(self) -> Checkbox<'a> {
        self.size(Size::Small)
    }
}

/// Room between the box and what it says.
const GAP: f32 = 8.0;

impl Widget for Checkbox<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        let size = self.size;
        let ink = Ink::of(ui);
        let enabled = ui.is_enabled();
        let fade = |c: egui::Color32| if enabled { c } else { c.gamma_multiply(0.45) };
        let side = size.icon();
        let galley = ui
            .painter()
            .layout_no_wrap(self.label.clone(), FontId::proportional(size.text()), fade(ink.fg));
        let width = side + if self.label.is_empty() { 0.0 } else { GAP + galley.size().x };
        let (rect, mut response) = ui.allocate_exact_size(Vec2::new(width, size.height()), Sense::click());
        if response.clicked() {
            *self.checked = !*self.checked;
            response.mark_changed();
        }
        let checked = *self.checked;
        response.widget_info(|| WidgetInfo::selected(WidgetType::Checkbox, enabled, checked, &self.label));
        if !ui.is_rect_visible(rect) {
            return response;
        }
        let painter = ui.painter();
        let tick = Rect::from_center_size(Pos2::new(rect.left() + side / 2.0, rect.center().y), Vec2::splat(side));
        let radius = size.radius() / 2.0;
        if *self.checked {
            painter.rect_filled(tick, radius, fade(ink.accent));
            let at = |x: f32, y: f32| tick.left_top() + Vec2::new(x * side, y * side);
            let stroke = Stroke::new(1.8_f32, fade(on(ink.accent)));
            painter.line_segment([at(0.24, 0.52), at(0.43, 0.70)], stroke);
            painter.line_segment([at(0.43, 0.70), at(0.77, 0.31)], stroke);
        } else {
            let fill = if response.hovered() { ink.shade(0.16) } else { ink.shade(0.10) };
            painter.rect_filled(tick, radius, fade(fill));
            painter.rect_stroke(tick, radius, Stroke::new(1.0_f32, fade(ink.shade(0.32))), StrokeKind::Inside);
        }
        // The keyboard's place: an outline two points out, two points rounder.
        if response.has_focus() {
            painter.rect_stroke(tick.expand(2.0), radius + 2.0, Stroke::new(1.0_f32, ink.accent), StrokeKind::Outside);
        }
        if !self.label.is_empty() {
            let y = rect.center().y - galley.size().y / 2.0;
            painter.galley(Pos2::new(tick.right() + GAP, y), galley, ink.fg);
        }
        response
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use egui::{Context, Event, PointerButton, RawInput};

    /// A click flips it, and says so.
    #[test]
    fn a_click_flips_it() {
        let ctx = Context::default();
        let mut on = false;
        let mut changed = false;
        let frame = |events: Vec<Event>, on: &mut bool, changed: &mut bool| {
            let input = RawInput { events, ..RawInput::default() };
            let _ = ctx.run(input, |ctx| {
                egui::CentralPanel::default().show(ctx, |ui| {
                    *changed |= ui.add(Checkbox::new(on, "Match case")).changed();
                });
            });
        };
        frame(vec![], &mut on, &mut changed);
        let at = Pos2::new(14.0, 20.0);
        let press = |pressed| Event::PointerButton {
            pos: at,
            button: PointerButton::Primary,
            pressed,
            modifiers: egui::Modifiers::NONE,
        };
        frame(vec![Event::PointerMoved(at), press(true)], &mut on, &mut changed);
        frame(vec![press(false)], &mut on, &mut changed);
        assert!(on && changed);
    }
}
