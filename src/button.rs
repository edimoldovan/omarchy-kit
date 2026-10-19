//! The button: four kinds, three sizes, one way of being drawn.

use egui::{
    Align2, Color32, CornerRadius, FontId, Painter, Pos2, Rect, Response, Sense, Ui, Vec2, Widget,
    WidgetInfo, WidgetType,
};

use crate::{Ink, Size, on};

/// What a button is for, which decides how loud it is.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Kind {
    /// The one thing the screen is for — Send, Compose: filled with the accent.
    Primary,
    /// A button among others — Discard, Attach, Cancel: a quiet fill.
    #[default]
    Secondary,
    /// A tool on a bar, a back arrow, a section's toggle: no fill until the
    /// pointer is on it.
    Quiet,
    /// What cannot be undone — Remove: filled with the theme's error colour.
    Danger,
}

/// An icon, drawn in the square it is given, in the colour it is given. The
/// kit carries no icon set of its own: each app brings its glyphs.
type Icon<'a> = Box<dyn Fn(&Painter, Rect, Color32) + 'a>;

/// A button: a label, an icon, or both, and a key hint after them.
#[must_use = "a button is drawn with ui.add(button)"]
pub struct Button<'a> {
    label: Option<String>,
    icon: Option<Icon<'a>>,
    hint: Option<String>,
    tip: Option<String>,
    kind: Kind,
    size: Size,
    selected: bool,
    width: Option<f32>,
    bar: Option<Color32>,
}

impl<'a> Button<'a> {
    /// A button that says `label`.
    pub fn new(label: impl Into<String>) -> Button<'a> {
        Button {
            label: Some(label.into()),
            icon: None,
            hint: None,
            tip: None,
            kind: Kind::default(),
            size: Size::default(),
            selected: false,
            width: None,
            bar: None,
        }
    }

    /// A button that is only an icon: square, its height on every side.
    pub fn icon_only(icon: impl Fn(&Painter, Rect, Color32) + 'a) -> Button<'a> {
        Button {
            label: None,
            ..Button::new("")
        }
        .icon(icon)
    }

    /// An icon before the label.
    pub fn icon(mut self, icon: impl Fn(&Painter, Rect, Color32) + 'a) -> Button<'a> {
        self.icon = Some(Box::new(icon));
        self
    }

    /// The key it is also on, quietly after the label. Nothing when empty.
    pub fn hint(mut self, hint: impl Into<String>) -> Button<'a> {
        let hint = hint.into();
        self.hint = (!hint.is_empty()).then_some(hint);
        self
    }

    /// What it says while the pointer is on it, and its name to a screen
    /// reader and a test when it shows no label. On an icon-only button the
    /// key from `hint` goes in the tip, `"Settings  (Home)"`, rather than
    /// beside the icon; its name stays `"Settings"`.
    pub fn tip(mut self, tip: impl Into<String>) -> Button<'a> {
        self.tip = Some(tip.into());
        self
    }

    pub fn kind(mut self, kind: Kind) -> Button<'a> {
        self.kind = kind;
        self
    }

    pub fn primary(self) -> Button<'a> {
        self.kind(Kind::Primary)
    }

    pub fn quiet(self) -> Button<'a> {
        self.kind(Kind::Quiet)
    }

    pub fn danger(self) -> Button<'a> {
        self.kind(Kind::Danger)
    }

    pub fn size(mut self, size: Size) -> Button<'a> {
        self.size = size;
        self
    }

    pub fn small(self) -> Button<'a> {
        self.size(Size::Small)
    }

    pub fn large(self) -> Button<'a> {
        self.size(Size::Large)
    }

    /// Marked as the one in use — the screen a quiet button leads to is on
    /// screen.
    pub fn selected(mut self, selected: bool) -> Button<'a> {
        self.selected = selected;
        self
    }

    /// As wide as `width` rather than as wide as what it says: a button that
    /// fills its column.
    pub fn width(mut self, width: f32) -> Button<'a> {
        self.width = Some(width);
        self
    }

    /// A bar of `colour` along its bottom edge: the colour a sheet's tab was
    /// given. Nothing when `None`.
    pub fn bar(mut self, colour: Option<Color32>) -> Button<'a> {
        self.bar = colour;
        self
    }
}

impl Widget for Button<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        let size = self.size;
        let ink = Ink::of(ui);
        let font = FontId::proportional(size.text());
        let small = FontId::proportional(size.text() - 2.0);
        let measure = |text: &str, font: &FontId| {
            ui.painter()
                .layout_no_wrap(text.to_string(), font.clone(), Color32::PLACEHOLDER)
                .size()
                .x
        };
        let gap = 7.0;
        let label_w = self.label.as_deref().map_or(0.0, |label| measure(label, &font));
        // An icon-only button has no room for its key: the key is in its tip.
        let hint = self.hint.as_deref().filter(|_| self.label.is_some());
        let hint_w = hint.map_or(0.0, |hint| measure(hint, &small) + gap);
        let icon_w = if self.icon.is_some() { size.icon() } else { 0.0 };
        let between = if self.icon.is_some() && self.label.is_some() { gap } else { 0.0 };
        let content = icon_w + between + label_w + hint_w;
        let width = match (self.width, &self.label) {
            (Some(width), _) => width,
            (None, None) => size.height(),
            (None, Some(_)) => content + 2.0 * size.pad(),
        };
        let (rect, mut response) =
            ui.allocate_exact_size(Vec2::new(width, size.height()), Sense::click());
        let enabled = ui.is_enabled();
        let name = self.label.clone().or_else(|| self.tip.clone()).unwrap_or_default();
        response.widget_info(|| WidgetInfo {
            enabled,
            label: (!name.is_empty()).then(|| name.clone()),
            ..WidgetInfo::new(WidgetType::Button)
        });
        if let Some(tip) = &self.tip {
            response = match (&self.label, &self.hint) {
                (None, Some(key)) => response.on_hover_text(format!("{tip}  ({key})")),
                _ => response.on_hover_text(tip),
            };
        }
        if !ui.is_rect_visible(rect) {
            return response;
        }

        let hovered = response.hovered();
        let pressed = response.is_pointer_button_down_on();
        let (fill, text) = colours(self.kind, &ink, hovered, pressed, self.selected);
        let fade = |c: Color32| if enabled { c } else { c.gamma_multiply(0.45) };
        let painter = ui.painter();
        if fill != Color32::TRANSPARENT {
            painter.rect_filled(rect, CornerRadius::from(size.radius()), fade(fill));
        }

        // What it says, centred: the icon, the label, then the hint.
        let mut x = rect.center().x - content / 2.0;
        let y = rect.center().y;
        if let Some(icon) = &self.icon {
            icon(
                painter,
                Rect::from_center_size(Pos2::new(x + icon_w / 2.0, y), Vec2::splat(icon_w)),
                fade(text),
            );
            x += icon_w + between;
        }
        if let Some(label) = &self.label {
            painter.text(Pos2::new(x, y), Align2::LEFT_CENTER, label, font, fade(text));
            x += label_w;
        }
        if let Some(hint) = hint {
            painter.text(
                Pos2::new(x + gap, y),
                Align2::LEFT_CENTER,
                hint,
                small,
                fade(text.gamma_multiply(0.6)),
            );
        }
        // Inset by the corner, so the bar stays inside the button's curve.
        if let Some(colour) = self.bar {
            let inset = size.radius();
            let bar = Rect::from_min_max(
                Pos2::new(rect.left() + inset, rect.bottom() - 3.0),
                Pos2::new(rect.right() - inset, rect.bottom()),
            );
            painter.rect_filled(bar, CornerRadius::from(1.5), fade(colour));
        }
        response
    }
}

/// A kind's fill and text, as the pointer has it.
fn colours(kind: Kind, ink: &Ink, hovered: bool, pressed: bool, selected: bool) -> (Color32, Color32) {
    let step = if pressed { 2.0 } else if hovered { 1.0 } else { 0.0 };
    match kind {
        Kind::Primary => {
            let fill = crate::theme::mix(ink.accent, ink.fg, 0.10 * step);
            (fill, on(ink.accent))
        }
        Kind::Danger => {
            let fill = crate::theme::mix(ink.danger, ink.fg, 0.10 * step);
            (fill, on(ink.danger))
        }
        Kind::Secondary if selected => (ink.selected, ink.fg),
        Kind::Secondary => (ink.shade(0.10 + 0.06 * step), ink.fg),
        Kind::Quiet if selected => (ink.selected, ink.fg),
        Kind::Quiet if step > 0.0 => (ink.shade(0.06 * step), ink.fg),
        Kind::Quiet => (Color32::TRANSPARENT, ink.fg),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ink() -> Ink {
        Ink {
            bg: Color32::from_rgb(18, 20, 24),
            fg: Color32::from_rgb(200, 205, 212),
            weak: Color32::from_rgb(120, 124, 130),
            accent: Color32::from_rgb(95, 135, 190),
            danger: Color32::from_rgb(220, 80, 80),
            selected: Color32::from_rgb(40, 60, 90),
        }
    }

    /// Each kind is its own loudness, and the pointer moves every one of them.
    #[test]
    fn every_kind_answers_the_pointer() {
        let ink = ink();
        for kind in [Kind::Primary, Kind::Secondary, Kind::Quiet, Kind::Danger] {
            let (rest, _) = colours(kind, &ink, false, false, false);
            let (hover, _) = colours(kind, &ink, true, false, false);
            let (press, _) = colours(kind, &ink, true, true, false);
            assert_ne!(rest, hover, "{kind:?} does not answer the pointer");
            assert_ne!(hover, press, "{kind:?} does not answer a press");
        }
        assert_eq!(colours(Kind::Primary, &ink, false, false, false).0, ink.accent);
        assert_eq!(colours(Kind::Quiet, &ink, false, false, false).0, Color32::TRANSPARENT);
        assert_eq!(colours(Kind::Quiet, &ink, false, false, true).0, ink.selected);
        assert_eq!(colours(Kind::Danger, &ink, false, false, false).1, on(ink.danger));
    }
}
