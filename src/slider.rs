//! The slider: an amount between two ends, dragged.

use std::ops::RangeInclusive;

use egui::{Align2, FontId, Pos2, Rect, Response, Sense, Stroke, StrokeKind, Ui, Vec2, Widget};

use crate::{Ink, Size};

/// An amount between two ends, shown as a field filled up to it — from the
/// left, or from zero when the range crosses zero — with its name inside at
/// the left and its value at the right. Dragging anywhere on it sets it; a
/// double click puts the default back. The response says `changed` whenever
/// the value moved.
#[must_use = "a slider is drawn with ui.add(slider)"]
pub struct Slider<'a> {
    value: &'a mut f32,
    range: RangeInclusive<f32>,
    label: String,
    default: Option<f32>,
    decimals: usize,
    suffix: String,
    size: Size,
    width: Option<f32>,
}

impl<'a> Slider<'a> {
    pub fn new(value: &'a mut f32, range: RangeInclusive<f32>) -> Slider<'a> {
        Slider {
            value,
            range,
            label: String::new(),
            default: None,
            decimals: 2,
            suffix: String::new(),
            size: Size::default(),
            width: None,
        }
    }

    /// What the amount is, inside at the left.
    pub fn label(mut self, label: impl Into<String>) -> Slider<'a> {
        self.label = label.into();
        self
    }

    /// Where a double click puts it back.
    pub fn default(mut self, default: f32) -> Slider<'a> {
        self.default = Some(default);
        self
    }

    /// How many decimals the value shows, and the steps it moves in.
    pub fn decimals(mut self, decimals: usize) -> Slider<'a> {
        self.decimals = decimals;
        self
    }

    /// Text after the value: a unit, a percent sign.
    pub fn suffix(mut self, suffix: impl Into<String>) -> Slider<'a> {
        self.suffix = suffix.into();
        self
    }

    pub fn size(mut self, size: Size) -> Slider<'a> {
        self.size = size;
        self
    }

    pub fn small(self) -> Slider<'a> {
        self.size(Size::Small)
    }

    /// As wide as `width` rather than the room there is.
    pub fn width(mut self, width: f32) -> Slider<'a> {
        self.width = Some(width);
        self
    }
}

impl Widget for Slider<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        let size = self.size;
        let ink = Ink::of(ui);
        let (lo, hi) = (*self.range.start(), *self.range.end());
        let width = self.width.unwrap_or_else(|| ui.available_width());
        let (rect, mut response) =
            ui.allocate_exact_size(Vec2::new(width, size.height()), Sense::click_and_drag());
        // The track runs between the pads, where the label and the value sit.
        let track = Rect::from_min_max(
            Pos2::new(rect.left() + size.pad(), rect.top()),
            Pos2::new(rect.right() - size.pad(), rect.bottom()),
        );

        let mut value = *self.value;
        if response.double_clicked()
            && let Some(default) = self.default
        {
            value = default;
        } else if (response.is_pointer_button_down_on() || response.dragged())
            && let Some(at) = response.interact_pointer_pos()
        {
            let f = fraction(at.x, track.left(), track.right());
            value = quantize(value_at(f, lo, hi), self.decimals);
        }
        if value != *self.value {
            *self.value = value;
            response.mark_changed();
        }

        if !ui.is_rect_visible(rect) {
            return response;
        }
        let painter = ui.painter();
        let radius = size.radius();
        let hovered = response.hovered() || response.dragged();
        painter.rect_filled(rect, radius, ink.shade(if hovered { 0.13 } else { 0.10 }));
        let (a, b) = fill_span(value, lo, hi);
        let span = Rect::from_min_max(
            Pos2::new(track.left() + a * track.width(), rect.top()),
            Pos2::new(track.left() + b * track.width(), rect.bottom()),
        );
        if span.width() > 0.0 {
            painter.with_clip_rect(span).rect_filled(rect, radius, ink.selected);
        }
        if response.dragged() {
            painter.rect_stroke(rect, radius, Stroke::new(1.0_f32, ink.accent), StrokeKind::Inside);
        }
        let font = FontId::proportional(size.text());
        let small = FontId::proportional(size.text() - 2.0);
        let text = format!("{:.*}{}", self.decimals, value, self.suffix);
        painter.text(Pos2::new(track.left(), rect.center().y), Align2::LEFT_CENTER, &self.label, font, ink.fg);
        painter.text(Pos2::new(track.right(), rect.center().y), Align2::RIGHT_CENTER, text, small, ink.weak);
        response
    }
}

/// Where `x` sits between `left` and `right`, 0..1.
fn fraction(x: f32, left: f32, right: f32) -> f32 {
    if right <= left {
        return 0.0;
    }
    ((x - left) / (right - left)).clamp(0.0, 1.0)
}

/// The value `f` of the way from `lo` to `hi`.
fn value_at(f: f32, lo: f32, hi: f32) -> f32 {
    lo + (hi - lo) * f
}

/// `value` rounded to `decimals` places.
fn quantize(value: f32, decimals: usize) -> f32 {
    let step = 10f32.powi(decimals as i32);
    (value * step).round() / step
}

/// The filled part of the track, as fractions of it from left to right: from
/// zero to the value when the range crosses zero, else from the start.
fn fill_span(value: f32, lo: f32, hi: f32) -> (f32, f32) {
    let v = fraction(value, lo, hi);
    let zero = if lo < 0.0 && hi > 0.0 { fraction(0.0, lo, hi) } else { 0.0 };
    (zero.min(v), zero.max(v))
}

#[cfg(test)]
mod tests {
    use super::*;
    use egui::{Context, Event, PointerButton, RawInput};

    #[test]
    fn fractions_values_and_steps() {
        assert_eq!(fraction(50.0, 0.0, 100.0), 0.5);
        assert_eq!(fraction(-10.0, 0.0, 100.0), 0.0);
        assert_eq!(fraction(10.0, 10.0, 10.0), 0.0);
        assert_eq!(value_at(0.25, -1.0, 1.0), -0.5);
        assert_eq!(quantize(0.123_456, 2), 0.12);
        assert_eq!(quantize(2.5, 0), 3.0);
    }

    /// A range that crosses zero fills out from zero; one that does not fills
    /// from its start.
    #[test]
    fn the_fill_starts_at_zero_when_there_is_one() {
        assert_eq!(fill_span(0.5, 0.0, 1.0), (0.0, 0.5));
        assert_eq!(fill_span(0.5, -1.0, 1.0), (0.5, 0.75));
        assert_eq!(fill_span(-0.5, -1.0, 1.0), (0.25, 0.5));
        assert_eq!(fill_span(0.0, -1.0, 1.0), (0.5, 0.5));
        let (a, b) = fill_span(1.0, 0.2, 5.0);
        assert_eq!(a, 0.0);
        assert!((b - fraction(1.0, 0.2, 5.0)).abs() < 1e-6);
    }

    /// Pressed on the middle of its track, the slider takes the middle value
    /// and says it changed.
    #[test]
    fn a_press_sets_the_value_under_the_pointer() {
        let ctx = Context::default();
        let mut value = 0.0f32;
        let mut changed = false;
        let mut frame = |events: Vec<Event>, value: &mut f32| {
            let input = RawInput { events, ..RawInput::default() };
            let _ = ctx.run(input, |ctx| {
                // A frameless panel: the slider sits at the origin with no
                // widget of the panel's own over it.
                egui::CentralPanel::default().frame(egui::Frame::NONE).show(ctx, |ui| {
                    changed |= ui.add(Slider::new(value, 0.0..=1.0).width(200.0)).changed();
                });
            });
        };
        // Laid out once, so the next frame's press finds it.
        frame(vec![], &mut value);
        let pad = Size::Regular.pad();
        let at = Pos2::new(pad + (200.0 - 2.0 * pad) / 2.0, 16.0);
        frame(
            vec![
                Event::PointerMoved(at),
                Event::PointerButton { pos: at, button: PointerButton::Primary, pressed: true, modifiers: Default::default() },
            ],
            &mut value,
        );
        frame(vec![], &mut value);
        assert!((value - 0.5).abs() < 0.02, "value {value}");
        assert!(changed);
    }
}
