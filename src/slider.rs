//! The slider: an amount between two ends, dragged along a track, nudged
//! with the arrow keys, or typed into the field at the track's end.

use std::ops::RangeInclusive;

use egui::{Align2, FontId, Id, Key, Modifiers, Pos2, Rect, Response, Sense, Stroke, Ui, Vec2, Widget};

use crate::{Ink, Size, TextField};

/// How wide the value's field is.
const FIELD: f32 = 72.0;
/// Room between the track and the field.
const GAP: f32 = 10.0;
/// The label's line above the control, when there is one.
const LABEL_LINE: f32 = 20.0;
/// The track's thickness.
const TRACK: f32 = 6.0;

/// The knob's radius; the track keeps that much clear at both ends.
fn knob(size: Size) -> f32 {
    match size {
        Size::Small => 7.0,
        Size::Regular | Size::Large => 9.0,
    }
}

/// An amount between two ends: a knob on a track, filled from zero when
/// the range crosses zero and from the start otherwise, and a field at the
/// track's end with the value in it. With a label, the label sits on its
/// own line above.
///
/// Dragging anywhere on the track sets it, and gives the field the
/// keyboard. While the field has the keyboard, Up and Down move the value
/// a step, Shift ten steps, Ctrl a hundred; Enter, or leaving the field,
/// takes what was typed, within the range; Esc puts back what the value
/// was when the field took the keyboard. A double click on the track puts
/// the default back. `changed()` whenever the value moved.
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
    id: Option<Id>,
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
            id: None,
        }
    }

    /// What the amount is, on a line above the track.
    pub fn label(mut self, label: impl Into<String>) -> Slider<'a> {
        self.label = label.into();
        self
    }

    /// Where a double click on the track puts it back.
    pub fn default(mut self, default: f32) -> Slider<'a> {
        self.default = Some(default);
        self
    }

    /// How many decimals the value shows, and the step it moves in.
    pub fn decimals(mut self, decimals: usize) -> Slider<'a> {
        self.decimals = decimals;
        self
    }

    /// Text after the value in its field: a unit, a percent sign.
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

    /// Where egui keeps the field's state, when two sliders would share it.
    pub fn id_salt(mut self, salt: impl std::hash::Hash) -> Slider<'a> {
        self.id = Some(Id::new(salt));
        self
    }
}

impl Widget for Slider<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        let size = self.size;
        let ink = Ink::of(ui);
        let (lo, hi) = (*self.range.start(), *self.range.end());
        let step = 10f32.powi(-(self.decimals as i32));
        let width = self.width.unwrap_or_else(|| ui.available_width());
        let labeled = !self.label.is_empty();
        let height = size.height() + if labeled { LABEL_LINE } else { 0.0 };

        let (rect, mut response) = ui.allocate_exact_size(Vec2::new(width, height), Sense::hover());
        let id = self.id.unwrap_or_else(|| response.id.with("omarchy-kit-slider"));
        let control = Rect::from_min_max(Pos2::new(rect.left(), rect.bottom() - size.height()), rect.max);
        let track_rect = Rect::from_min_max(control.min, Pos2::new(control.right() - FIELD - GAP, control.bottom()));
        let r = knob(size);
        let (left, right) = (track_rect.left() + r, track_rect.right() - r);
        let field_rect = Rect::from_min_max(Pos2::new(control.right() - FIELD, control.top()), control.max);

        // The field's own egui id, as TextField makes it from the salt.
        let field_id = id.with("field");
        let edit_id = Id::new(field_id);
        let orig_id = id.with("orig");
        let nudged_id = id.with("nudged");
        let had_id = id.with("had-focus");
        let focused = ui.memory(|m| m.has_focus(edit_id));
        // egui drops the keyboard on Esc before anything is drawn; what the
        // field had last frame says the Esc was for it.
        let had_focus = ui.data(|d| d.get_temp::<bool>(had_id)).unwrap_or(false);

        let mut value = *self.value;
        let mut nudged = false;

        // The keyboard, while the field has it: Up and Down by a step,
        // more with Shift and Ctrl; Esc back to where it started.
        let mut escaped = false;
        if focused {
            let steps = ui.input_mut(|i| {
                // The fuller chords first: a plain arrow would claim a
                // Shift arrow, since egui matches with at least the modifiers.
                let mut n = 0i32;
                for (mods, mult) in [(Modifiers::CTRL, 100), (Modifiers::SHIFT, 10), (Modifiers::NONE, 1)] {
                    if i.consume_key(mods, Key::ArrowUp) {
                        n += mult;
                    }
                    if i.consume_key(mods, Key::ArrowDown) {
                        n -= mult;
                    }
                }
                n
            });
            // A frame egui runs again, as when the field just took the
            // keyboard, sees the key again: once per frame is enough.
            let frame = ui.ctx().cumulative_frame_nr();
            let done = ui.data(|d| d.get_temp::<u64>(nudged_id)) == Some(frame);
            if steps != 0 && !done {
                value = quantize((value + steps as f32 * step).clamp(lo, hi), self.decimals);
                nudged = true;
                ui.data_mut(|d| d.insert_temp(nudged_id, frame));
            }
        }
        if focused || had_focus {
            escaped = ui.input(|i| i.key_pressed(Key::Escape));
        }

        // The track: dragged, the knob follows the pointer; a click or a
        // drag also hands the keyboard to the field.
        let track = ui.interact(track_rect, id.with("track"), Sense::click_and_drag());
        if track.double_clicked()
            && let Some(default) = self.default
        {
            value = default;
        } else if (track.is_pointer_button_down_on() || track.dragged())
            && let Some(at) = track.interact_pointer_pos()
        {
            value = quantize(value_at(fraction(at.x, left, right), lo, hi), self.decimals);
        }
        if track.clicked() || track.drag_started() {
            ui.memory_mut(|m| m.request_focus(edit_id));
        }

        // The field: what is typed stays while it has the keyboard and is
        // taken when it leaves; a nudge or a drag writes the value into it.
        let mut text = if focused && !nudged && !track.dragged() {
            ui.data(|d| d.get_temp::<String>(field_id)).unwrap_or_else(|| format(value, self.decimals))
        } else {
            format(value, self.decimals)
        };
        let edit = ui
            .scope_builder(egui::UiBuilder::new().max_rect(field_rect), |ui| {
                let mut field = TextField::new(&mut text).width(FIELD).id_salt(field_id);
                if size == Size::Small {
                    field = field.small();
                }
                if !self.suffix.is_empty() {
                    field = field.key(self.suffix.trim());
                }
                ui.add(field)
            })
            .inner;
        if edit.gained_focus() {
            ui.data_mut(|d| d.insert_temp(orig_id, *self.value));
        }
        if edit.has_focus() {
            ui.data_mut(|d| d.insert_temp(field_id, text.clone()));
        }
        let has_focus = edit.has_focus();
        ui.data_mut(|d| d.insert_temp(had_id, has_focus));
        if escaped {
            value = ui.data(|d| d.get_temp::<f32>(orig_id)).unwrap_or(value);
        } else if edit.lost_focus()
            && let Ok(typed) = text.trim().parse::<f32>()
        {
            value = quantize(typed.clamp(lo, hi), self.decimals);
        }

        if value != *self.value {
            *self.value = value;
            response.mark_changed();
        }

        if ui.is_rect_visible(rect) {
            let painter = ui.painter();
            if labeled {
                painter.text(
                    Pos2::new(rect.left(), rect.top() + LABEL_LINE / 2.0),
                    Align2::LEFT_CENTER,
                    &self.label,
                    FontId::proportional(Size::Small.text()),
                    ink.weak,
                );
            }
            let y = control.center().y;
            let bar = Rect::from_min_max(Pos2::new(left, y - TRACK / 2.0), Pos2::new(right, y + TRACK / 2.0));
            painter.rect_filled(bar, TRACK / 2.0, ink.shade(0.18));
            let (a, b) = fill_span(value, lo, hi);
            let span = Rect::from_min_max(Pos2::new(left + a * (right - left), bar.top()), Pos2::new(left + b * (right - left), bar.bottom()));
            if span.width() > 0.0 {
                painter.rect_filled(span, TRACK / 2.0, ink.accent);
            }
            let at = Pos2::new(left + fraction(value, lo, hi) * (right - left), y);
            let lively = track.hovered() || track.dragged() || focused;
            painter.circle(at, if lively { r } else { r - 1.0 }, ink.bg, Stroke::new(if lively { 2.5_f32 } else { 2.0_f32 }, ink.accent));
        }
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
pub(crate) fn quantize(value: f32, decimals: usize) -> f32 {
    let step = 10f32.powi(decimals as i32);
    (value * step).round() / step
}

/// The value as the field shows it.
pub(crate) fn format(value: f32, decimals: usize) -> String {
    format!("{value:.decimals$}")
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
        assert_eq!(format(0.5, 2), "0.50");
        assert_eq!(format(30.0, 0), "30");
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

    fn run(ctx: &Context, events: Vec<Event>, value: &mut f32, changed: &mut bool) {
        let input = RawInput { events, ..RawInput::default() };
        let _ = ctx.run(input, |ctx| {
            egui::CentralPanel::default().frame(egui::Frame::NONE).show(ctx, |ui| {
                *changed |= ui.add(Slider::new(value, 0.0..=1.0).width(300.0).id_salt("slider-test")).changed();
            });
        });
    }

    fn click(at: Pos2) -> Vec<Event> {
        vec![
            Event::PointerMoved(at),
            Event::PointerButton { pos: at, button: PointerButton::Primary, pressed: true, modifiers: Default::default() },
            Event::PointerButton { pos: at, button: PointerButton::Primary, pressed: false, modifiers: Default::default() },
        ]
    }

    fn key(key: Key, modifiers: Modifiers) -> Event {
        Event::Key { key, physical_key: None, pressed: true, repeat: false, modifiers }
    }

    /// Pressed on the middle of its track, the slider takes the middle value
    /// and says it changed.
    #[test]
    fn a_press_sets_the_value_under_the_pointer() {
        let ctx = Context::default();
        let mut value = 0.0f32;
        let mut changed = false;
        run(&ctx, vec![], &mut value, &mut changed);
        let r = knob(Size::Regular);
        let (left, right) = (r, 300.0 - FIELD - GAP - r);
        let at = Pos2::new((left + right) / 2.0, Size::Regular.height() / 2.0);
        run(
            &ctx,
            vec![
                Event::PointerMoved(at),
                Event::PointerButton { pos: at, button: PointerButton::Primary, pressed: true, modifiers: Default::default() },
            ],
            &mut value,
            &mut changed,
        );
        run(&ctx, vec![], &mut value, &mut changed);
        assert!((value - 0.5).abs() < 0.02, "value {value}");
        assert!(changed);
    }

    /// Typed into its field and entered, the slider takes the number, kept
    /// within its range.
    #[test]
    fn a_typed_value_is_taken_on_enter() {
        let ctx = Context::default();
        let mut value = 0.25f32;
        let mut changed = false;
        run(&ctx, vec![], &mut value, &mut changed);
        let field = Pos2::new(300.0 - FIELD / 2.0, Size::Regular.height() / 2.0);
        run(&ctx, click(field), &mut value, &mut changed);
        run(&ctx, vec![], &mut value, &mut changed);
        run(&ctx, vec![key(Key::A, Modifiers::COMMAND)], &mut value, &mut changed);
        run(&ctx, vec![Event::Text("0.8".into())], &mut value, &mut changed);
        assert_eq!(value, 0.25, "nothing taken while typing");
        run(&ctx, vec![key(Key::Enter, Modifiers::NONE)], &mut value, &mut changed);
        run(&ctx, vec![], &mut value, &mut changed);
        assert!((value - 0.8).abs() < 1e-6, "value {value}");
        assert!(changed);
        // Out of range comes back clamped.
        run(&ctx, click(field), &mut value, &mut changed);
        run(&ctx, vec![], &mut value, &mut changed);
        run(&ctx, vec![key(Key::A, Modifiers::COMMAND)], &mut value, &mut changed);
        run(&ctx, vec![Event::Text("7".into())], &mut value, &mut changed);
        run(&ctx, vec![key(Key::Enter, Modifiers::NONE)], &mut value, &mut changed);
        run(&ctx, vec![], &mut value, &mut changed);
        assert_eq!(value, 1.0);
    }

    /// With the keyboard in its field, the arrows nudge the value by a step,
    /// ten with Shift, and Esc puts back what it was.
    #[test]
    fn arrows_nudge_and_escape_restores() {
        let ctx = Context::default();
        let mut value = 0.5f32;
        let mut changed = false;
        run(&ctx, vec![], &mut value, &mut changed);
        let field = Pos2::new(300.0 - FIELD / 2.0, Size::Regular.height() / 2.0);
        run(&ctx, click(field), &mut value, &mut changed);
        run(&ctx, vec![], &mut value, &mut changed);
        run(&ctx, vec![key(Key::ArrowUp, Modifiers::NONE)], &mut value, &mut changed);
        assert!((value - 0.51).abs() < 1e-6, "{value}");
        run(&ctx, vec![key(Key::ArrowUp, Modifiers::SHIFT)], &mut value, &mut changed);
        assert!((value - 0.61).abs() < 1e-6, "{value}");
        run(&ctx, vec![key(Key::ArrowDown, Modifiers::CTRL)], &mut value, &mut changed);
        assert_eq!(value, 0.0, "clamped at the end");
        run(&ctx, vec![key(Key::Escape, Modifiers::NONE)], &mut value, &mut changed);
        run(&ctx, vec![], &mut value, &mut changed);
        assert_eq!(value, 0.5, "back to where the field took the keyboard");
    }
}
