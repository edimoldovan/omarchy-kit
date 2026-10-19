//! The section: a titled group that folds, with a switch for the whole group
//! and a reset.

use egui::{Align, Align2, FontId, Id, Layout, Pos2, Rect, Sense, Shape, Stroke, StrokeKind, Ui, UiBuilder, Vec2, WidgetInfo, WidgetType};

use crate::{Button, Ink, Size, Switch, icons};

/// A titled group of settings — an inspector's Transform, Color, Audio. The
/// header folds it: the chevron and the title, a click or Space on them
/// folding and unfolding it. At the header's far end, a switch turning the
/// whole group on and off, and a reset. While the switch is off the group
/// draws disabled, its values still shown and kept. Whether it is open is
/// remembered per id.
#[must_use = "a section is drawn with section.show(ui, …)"]
pub struct Section<'a> {
    id: Id,
    title: String,
    switch: Option<&'a mut bool>,
    reset: Option<bool>,
    default_open: bool,
}

/// What happened to a section this frame.
pub struct SectionResponse<R> {
    /// What the body returned; `None` while it is folded.
    pub inner: Option<R>,
    /// The switch was flipped.
    pub switched: bool,
    /// The reset was clicked.
    pub reset: bool,
    /// It is open.
    pub open: bool,
}

impl<'a> Section<'a> {
    pub fn new(id_salt: impl std::hash::Hash, title: impl Into<String>) -> Section<'a> {
        Section {
            id: Id::new(("omarchy-kit-section", id_salt)),
            title: title.into(),
            switch: None,
            reset: None,
            default_open: true,
        }
    }

    /// A switch for the whole group, named "TITLE section".
    pub fn switch(mut self, on: &'a mut bool) -> Section<'a> {
        self.switch = Some(on);
        self
    }

    /// A reset, named "Reset TITLE"; clickable while `enabled` — while the
    /// group differs from its defaults, say.
    pub fn reset(mut self, enabled: bool) -> Section<'a> {
        self.reset = Some(enabled);
        self
    }

    /// Whether it starts open; open when unset.
    pub fn default_open(mut self, open: bool) -> Section<'a> {
        self.default_open = open;
        self
    }

    /// Draws the header, and the body with `add` while it is open.
    pub fn show<R>(self, ui: &mut Ui, add: impl FnOnce(&mut Ui) -> R) -> SectionResponse<R> {
        let Section { id, title, mut switch, reset, default_open } = self;
        let ink = Ink::of(ui);
        let size = Size::Regular;
        let mut open = ui.data_mut(|d| d.get_persisted::<bool>(id)).unwrap_or(default_open);
        let (row, _) = ui.allocate_exact_size(Vec2::new(ui.available_width(), size.height()), Sense::hover());
        let hover_fill = ui.painter().add(Shape::Noop);

        // The far end: the switch, then the reset at the very end.
        let (mut switched, mut reset_clicked) = (false, false);
        // A child that leaves the header's row as it was allocated.
        let mut far = ui.new_child(UiBuilder::new().max_rect(row).layout(Layout::right_to_left(Align::Center)));
        far.spacing_mut().item_spacing.x = 6.0;
        if let Some(enabled) = reset {
            let undo = |painter: &egui::Painter, rect: Rect, colour| {
                icons::draw(painter, rect, "undo", colour);
            };
            let button = Button::icon_only(undo).quiet().small().tip(format!("Reset {title}"));
            reset_clicked = far.add_enabled(enabled, button).clicked();
        }
        if let Some(on) = switch.as_deref_mut() {
            switched = far.add(Switch::new(on).small().name(format!("{title} section"))).changed();
        }
        let end = far.min_rect();
        let has_end = reset.is_some() || switch.is_some();
        let fold_rect = if has_end { Rect::from_min_max(row.min, Pos2::new(end.left() - 8.0, row.max.y)) } else { row };

        let fold = ui.interact(fold_rect, id.with("fold"), Sense::click());
        if fold.clicked() {
            open = !open;
            ui.data_mut(|d| d.insert_persisted(id, open));
        }
        fold.widget_info(|| WidgetInfo::labeled(WidgetType::CollapsingHeader, ui.is_enabled(), &title));
        fold.ctx.accesskit_node_builder(fold.id, |node| node.set_expanded(open));

        if ui.is_rect_visible(row) {
            let painter = ui.painter();
            if fold.hovered() {
                painter.set(hover_fill, Shape::rect_filled(fold_rect, size.radius(), ink.shade(0.07)));
            }
            // The chevron, drawn: right while folded, turning down as it opens.
            let turn = ui.ctx().animate_bool_responsive(id.with("chevron"), open) * std::f32::consts::FRAC_PI_2;
            let centre = Pos2::new(row.left() + 10.0, row.center().y);
            let point = |x: f32, y: f32| centre + Vec2::new(x * turn.cos() - y * turn.sin(), x * turn.sin() + y * turn.cos());
            painter.add(Shape::line(vec![point(-2.0, -4.0), point(2.5, 0.0), point(-2.0, 4.0)], Stroke::new(1.5_f32, ink.weak)));
            painter.text(
                Pos2::new(row.left() + 22.0, row.center().y),
                Align2::LEFT_CENTER,
                &title,
                FontId::proportional(size.text()),
                ink.fg,
            );
            // The keyboard's place: an outline, as on any item.
            if fold.has_focus() {
                painter.rect_stroke(fold_rect, size.radius(), Stroke::new(1.0_f32, ink.accent), StrokeKind::Inside);
            }
        }

        let enabled = switch.as_deref().copied().unwrap_or(true);
        let inner = open.then(|| {
            let inner = ui.add_enabled_ui(enabled, add).inner;
            ui.add_space(6.0);
            inner
        });
        SectionResponse { inner, switched, reset: reset_clicked, open }
    }
}
