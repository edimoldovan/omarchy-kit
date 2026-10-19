//! The switch: a feature, or a group of settings, on or off.

use egui::accesskit::Role;
use egui::{Pos2, Rect, Response, Sense, Stroke, StrokeKind, Ui, Vec2, Widget, WidgetInfo, WidgetType};

use crate::{Ink, Size, on};

/// A feature or a group of settings on or off, its settings kept either way:
/// a pill with a knob — the accent while on — that slides across when a
/// click, or Space while it has the keyboard, flips it; `changed()` when it
/// did. A single option is a [`crate::Checkbox`].
#[must_use = "a switch is drawn with ui.add(switch)"]
pub struct Switch<'a> {
    on: &'a mut bool,
    name: String,
    size: Size,
}

impl<'a> Switch<'a> {
    pub fn new(on: &'a mut bool) -> Switch<'a> {
        Switch { on, name: String::new(), size: Size::default() }
    }

    /// Its name to a screen reader and a test.
    pub fn name(mut self, name: impl Into<String>) -> Switch<'a> {
        self.name = name.into();
        self
    }

    pub fn size(mut self, size: Size) -> Switch<'a> {
        self.size = size;
        self
    }

    /// Sized to sit in a header row.
    pub fn small(self) -> Switch<'a> {
        self.size(Size::Small)
    }
}

impl Widget for Switch<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        let ink = Ink::of(ui);
        let enabled = ui.is_enabled();
        let tall = (self.size.height() * 0.6).round();
        let (rect, mut response) = ui.allocate_exact_size(Vec2::new(tall * 1.8, self.size.height()), Sense::click());
        if response.clicked() {
            *self.on = !*self.on;
            response.mark_changed();
        }
        let state = *self.on;
        response.widget_info(|| WidgetInfo::selected(WidgetType::Checkbox, enabled, state, &self.name));
        response.ctx.accesskit_node_builder(response.id, |node| node.set_role(Role::Switch));
        if !ui.is_rect_visible(rect) {
            return response;
        }

        let fade = |c: egui::Color32| if enabled { c } else { c.gamma_multiply(0.45) };
        let pill = Rect::from_center_size(rect.center(), Vec2::new(tall * 1.8, tall));
        let t = ui.ctx().animate_bool_responsive(response.id, state);
        let off = if response.hovered() { ink.shade(0.32) } else { ink.shade(0.24) };
        let fill = crate::theme::mix(off, ink.accent, t);
        let painter = ui.painter();
        painter.rect_filled(pill, tall / 2.0, fade(fill));
        let r = tall / 2.0 - 2.5;
        let x = pill.left() + tall / 2.0 + t * (pill.width() - tall);
        painter.circle_filled(Pos2::new(x, pill.center().y), r, fade(on(fill)));
        // The keyboard's place: an outline two points out, two points rounder.
        if response.has_focus() {
            painter.rect_stroke(pill.expand(2.0), tall / 2.0 + 2.0, Stroke::new(1.0_f32, ink.accent), StrokeKind::Outside);
        }
        response
    }
}
