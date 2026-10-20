//! The slider: an amount between two ends, dragged along a track, nudged
//! with the arrow keys, or typed into the field at the track's end.

use std::ops::RangeInclusive;

use egui::{Align2, Color32, FontId, Id, Key, Mesh, Modifiers, Pos2, Rect, Response, Sense, Shape, Stroke, Ui, Vec2, Widget, WidgetInfo};

use crate::{Ink, Size, TextField, adjust};

/// How wide the value's field is.
const FIELD: f32 = 72.0;
/// Room between the track and the field.
const GAP: f32 = 10.0;
/// The label's line above the control, when there is one.
const LABEL_LINE: f32 = 20.0;
/// The track's thickness.
const TRACK: f32 = 6.0;

/// The number a [`Slider`] or a [`crate::NumberField`] holds: `f32` or `f64`.
pub trait Amount: Copy + PartialEq + 'static {
    fn to_f64(self) -> f64;
    fn from_f64(value: f64) -> Self;
}

impl Amount for f32 {
    fn to_f64(self) -> f64 {
        f64::from(self)
    }

    fn from_f64(value: f64) -> f32 {
        value as f32
    }
}

impl Amount for f64 {
    fn to_f64(self) -> f64 {
        self
    }

    fn from_f64(value: f64) -> f64 {
        value
    }
}

/// The knob's radius; the track keeps that much clear at both ends.
fn knob(size: Size) -> f32 {
    match size {
        Size::Mini => 6.0,
        Size::Small => 7.0,
        Size::Regular | Size::Large => 9.0,
    }
}

/// An amount between two ends: a knob on a track, filled from zero when
/// the range crosses zero and from the start otherwise, and a field at the
/// track's end with the value in it. With a label, the label sits on its
/// own line above, or in a column of its own on the track's line.
///
/// Dragging anywhere on the track sets it, and gives the field the
/// keyboard. While the field has the keyboard, Up and Down move the value
/// a step, Shift ten steps, Ctrl a hundred; Enter, or leaving the field,
/// takes what was typed, within the range; Esc puts back what the value
/// was when the field took the keyboard. A double click on the track puts
/// the default back. `changed()` whenever the value moved; the response's
/// [`crate::Adjust`] says when an adjustment ends.
#[must_use = "a slider is drawn with ui.add(slider)"]
pub struct Slider<'a, T: Amount = f32> {
    value: &'a mut T,
    range: RangeInclusive<T>,
    label: String,
    inline: Option<f32>,
    name: String,
    default: Option<T>,
    decimals: usize,
    suffix: String,
    logarithmic: bool,
    stops: &'a [(f32, Color32)],
    size: Size,
    width: Option<f32>,
    id: Option<Id>,
    keep_keyboard: bool,
}

impl<'a, T: Amount> Slider<'a, T> {
    pub fn new(value: &'a mut T, range: RangeInclusive<T>) -> Slider<'a, T> {
        Slider {
            value,
            range,
            label: String::new(),
            inline: None,
            name: String::new(),
            default: None,
            decimals: 2,
            suffix: String::new(),
            logarithmic: false,
            stops: &[],
            size: Size::default(),
            width: None,
            id: None,
            keep_keyboard: false,
        }
    }

    /// A click or a drag on the track leaves the keyboard where it was,
    /// rather than handing it to the field: for apps whose plain keys are
    /// shortcuts (an editor's Space, J, K, L). A click on the field still
    /// takes the keyboard for typing.
    pub fn keep_keyboard(mut self) -> Slider<'a, T> {
        self.keep_keyboard = true;
        self
    }

    /// What the amount is, on a line above the track.
    pub fn label(mut self, label: impl Into<String>) -> Slider<'a, T> {
        self.label = label.into();
        self
    }

    /// The label in a column `width` wide on the track's line, rather than
    /// above it, at the small size: a stack of these rows lines up, label
    /// under label and field under field.
    pub fn inline_label(mut self, width: f32) -> Slider<'a, T> {
        self.inline = Some(width);
        self.size = Size::Small;
        self
    }

    /// Its name to a screen reader and a test; the label when unset. The
    /// amount is its value.
    pub fn name(mut self, name: impl Into<String>) -> Slider<'a, T> {
        self.name = name.into();
        self
    }

    /// Where a double click on the track puts it back.
    pub fn default(mut self, default: T) -> Slider<'a, T> {
        self.default = Some(default);
        self
    }

    /// How many decimals the value shows, and the step it moves in.
    pub fn decimals(mut self, decimals: usize) -> Slider<'a, T> {
        self.decimals = decimals;
        self
    }

    /// Text after the value in its field: a unit, a percent sign.
    pub fn suffix(mut self, suffix: impl Into<String>) -> Slider<'a, T> {
        self.suffix = suffix.into();
        self
    }

    /// The track maps the range logarithmically: as much of it for 0.1–1 as
    /// for 1–10 — speed, scale, a frequency. A range that does not start
    /// above zero stays linear.
    pub fn logarithmic(mut self) -> Slider<'a, T> {
        self.logarithmic = true;
        self
    }

    /// Colour stops painted along the track in place of its fill — each a
    /// fraction of the track, 0 to 1, and its colour: cool to warm, green to
    /// magenta, the hues.
    pub fn track(mut self, stops: &'a [(f32, Color32)]) -> Slider<'a, T> {
        self.stops = stops;
        self
    }

    pub fn size(mut self, size: Size) -> Slider<'a, T> {
        self.size = size;
        self
    }

    pub fn small(self) -> Slider<'a, T> {
        self.size(Size::Small)
    }

    /// As wide as `width` rather than the room there is.
    pub fn width(mut self, width: f32) -> Slider<'a, T> {
        self.width = Some(width);
        self
    }

    /// Where egui keeps the field's state, when two sliders would share it.
    pub fn id_salt(mut self, salt: impl std::hash::Hash) -> Slider<'a, T> {
        self.id = Some(Id::new(salt));
        self
    }
}

impl<T: Amount> Widget for Slider<'_, T> {
    fn ui(self, ui: &mut Ui) -> Response {
        let size = self.size;
        let ink = Ink::of(ui);
        let (lo, hi) = (self.range.start().to_f64(), self.range.end().to_f64());
        let scale = Scale::new(lo, hi, self.logarithmic);
        let step = 10f64.powi(-(self.decimals as i32));
        let width = self.width.unwrap_or_else(|| ui.available_width());
        let labeled = !self.label.is_empty();
        let above = labeled && self.inline.is_none();
        let height = size.height() + if above { LABEL_LINE } else { 0.0 };

        let (rect, mut response) = ui.allocate_exact_size(Vec2::new(width, height), Sense::hover());
        let id = self.id.unwrap_or_else(|| response.id.with("omarchy-kit-slider"));
        let control = Rect::from_min_max(Pos2::new(rect.left(), rect.bottom() - size.height()), rect.max);
        let column = self.inline.unwrap_or(0.0);
        let track_rect = Rect::from_min_max(
            Pos2::new(control.left() + column, control.top()),
            Pos2::new(control.right() - FIELD - GAP, control.bottom()),
        );
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

        let original = self.value.to_f64();
        let mut value = original;
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
                value = quantize((value + f64::from(steps) * step).clamp(lo, hi), self.decimals);
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
        let pressing = track.is_pointer_button_down_on() || track.dragged();
        if track.double_clicked()
            && let Some(default) = self.default
        {
            value = default.to_f64();
        } else if pressing && let Some(at) = track.interact_pointer_pos() {
            value = quantize(scale.value_at(fraction(at.x, left, right)), self.decimals);
        }
        if !self.keep_keyboard && (track.clicked() || track.drag_started()) {
            ui.memory_mut(|m| m.request_focus(edit_id));
        }

        // The field: what is typed stays while it has the keyboard and is
        // taken when it leaves; a nudge or a drag writes the value into it.
        let mut text = if focused && !nudged && !track.dragged() {
            ui.data(|d| d.get_temp::<String>(field_id)).unwrap_or_else(|| format(value, self.decimals))
        } else {
            format(value, self.decimals)
        };
        // In the room the slider already took, left to right whatever the
        // row it sits in: a right-to-left status line, say.
        let mut room = ui.new_child(
            egui::UiBuilder::new().max_rect(field_rect).layout(egui::Layout::left_to_right(egui::Align::Center)),
        );
        let mut field = TextField::new(&mut text).width(FIELD).id_salt(field_id);
        if size == Size::Small {
            field = field.small();
        }
        if !self.suffix.is_empty() {
            field = field.key(self.suffix.trim());
        }
        let edit = room.add(field);
        if edit.gained_focus() {
            ui.data_mut(|d| d.insert_temp(orig_id, original));
        }
        if edit.has_focus() {
            ui.data_mut(|d| d.insert_temp(field_id, text.clone()));
        }
        let has_focus = edit.has_focus();
        ui.data_mut(|d| d.insert_temp(had_id, has_focus));
        if escaped {
            value = ui.data(|d| d.get_temp::<f64>(orig_id)).unwrap_or(value);
        } else if edit.lost_focus()
            && let Ok(typed) = text.trim().parse::<f64>()
        {
            value = quantize(typed.clamp(lo, hi), self.decimals);
        }

        let taken = T::from_f64(value);
        if taken != *self.value {
            *self.value = taken;
            response.mark_changed();
        }
        adjust::record(ui, response.id, pressing || nudged || value != original, pressing || has_focus);
        let enabled = ui.is_enabled();
        let name = if self.name.is_empty() { &self.label } else { &self.name };
        track.widget_info(|| WidgetInfo::slider(enabled, value, name));

        if ui.is_rect_visible(rect) {
            let painter = ui.painter();
            if labeled {
                let (at, clip) = match self.inline {
                    Some(column) => (
                        Pos2::new(control.left(), control.center().y),
                        Rect::from_min_size(control.min, Vec2::new(column - 6.0, control.height())),
                    ),
                    None => (Pos2::new(rect.left(), rect.top() + LABEL_LINE / 2.0), rect),
                };
                painter.with_clip_rect(clip).text(at, Align2::LEFT_CENTER, &self.label, FontId::proportional(Size::Small.text()), ink.weak);
            }
            let y = control.center().y;
            let bar = Rect::from_min_max(Pos2::new(left, y - TRACK / 2.0), Pos2::new(right, y + TRACK / 2.0));
            if self.stops.is_empty() {
                painter.rect_filled(bar, TRACK / 2.0, ink.shade(0.18));
                let (a, b) = fill_span(value, scale);
                let span = Rect::from_min_max(Pos2::new(left + a * (right - left), bar.top()), Pos2::new(left + b * (right - left), bar.bottom()));
                if span.width() > 0.0 {
                    painter.rect_filled(span, TRACK / 2.0, ink.accent);
                }
            } else {
                painter.extend(gradient(bar, self.stops));
            }
            let at = Pos2::new(left + scale.fraction(value) * (right - left), y);
            let lively = track.hovered() || track.dragged() || focused;
            painter.circle(at, if lively { r } else { r - 1.0 }, ink.bg, Stroke::new(if lively { 2.5_f32 } else { 2.0_f32 }, ink.accent));
        }
        response
    }
}

/// How the track maps the range: evenly, or logarithmically for a range
/// above zero.
#[derive(Clone, Copy)]
struct Scale {
    lo: f64,
    hi: f64,
    log: bool,
}

impl Scale {
    fn new(lo: f64, hi: f64, logarithmic: bool) -> Scale {
        Scale { lo, hi, log: logarithmic && lo > 0.0 && hi > lo }
    }

    /// Where `value` sits on the track, 0..1.
    fn fraction(self, value: f64) -> f32 {
        let Scale { lo, hi, log } = self;
        if hi <= lo {
            return 0.0;
        }
        let f = if log { (value.max(lo) / lo).ln() / (hi / lo).ln() } else { (value - lo) / (hi - lo) };
        f.clamp(0.0, 1.0) as f32
    }

    /// The value `f` of the way along the track.
    fn value_at(self, f: f32) -> f64 {
        let Scale { lo, hi, log } = self;
        let f = f64::from(f);
        if log { lo * (hi / lo).powf(f) } else { lo + (hi - lo) * f }
    }
}

/// Where `x` sits between `left` and `right`, 0..1.
fn fraction(x: f32, left: f32, right: f32) -> f32 {
    if right <= left {
        return 0.0;
    }
    ((x - left) / (right - left)).clamp(0.0, 1.0)
}

/// `value` rounded to `decimals` places.
pub(crate) fn quantize(value: f64, decimals: usize) -> f64 {
    let step = 10f64.powi(decimals as i32);
    (value * step).round() / step
}

/// The value as the field shows it.
pub(crate) fn format(value: f64, decimals: usize) -> String {
    format!("{value:.decimals$}")
}

/// The filled part of the track, as fractions of it from left to right: from
/// zero to the value when the range crosses zero, else from the start.
fn fill_span(value: f64, scale: Scale) -> (f32, f32) {
    let v = scale.fraction(value);
    let zero = if !scale.log && scale.lo < 0.0 && scale.hi > 0.0 { scale.fraction(0.0) } else { 0.0 };
    (zero.min(v), zero.max(v))
}

/// The track as colour stops, round at both ends and no longer than the
/// plain track: the first stop's colour before the first stop, the last's
/// after the last.
fn gradient(bar: Rect, stops: &[(f32, Color32)]) -> Vec<Shape> {
    let (Some(first), Some(last)) = (stops.first(), stops.last()) else {
        return Vec::new();
    };
    let mut points = vec![(0.0, first.1)];
    points.extend(stops.iter().map(|&(f, c)| (f.clamp(0.0, 1.0), c)));
    points.push((1.0, last.1));
    let r = bar.height() / 2.0;
    let x = |f: f32| (bar.left() + f * bar.width()).clamp(bar.left() + r, bar.right() - r);
    let mut mesh = Mesh::default();
    for pair in points.windows(2) {
        let ((a, ca), (b, cb)) = (pair[0], pair[1]);
        let base = mesh.vertices.len() as u32;
        mesh.colored_vertex(Pos2::new(x(a), bar.top()), ca);
        mesh.colored_vertex(Pos2::new(x(b), bar.top()), cb);
        mesh.colored_vertex(Pos2::new(x(b), bar.bottom()), cb);
        mesh.colored_vertex(Pos2::new(x(a), bar.bottom()), ca);
        mesh.add_triangle(base, base + 1, base + 2);
        mesh.add_triangle(base, base + 2, base + 3);
    }
    vec![
        Shape::circle_filled(Pos2::new(bar.left() + r, bar.center().y), r, first.1),
        Shape::circle_filled(Pos2::new(bar.right() - r, bar.center().y), r, last.1),
        Shape::mesh(mesh),
    ]
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
        assert_eq!(Scale::new(-1.0, 1.0, false).value_at(0.25), -0.5);
        assert_eq!(quantize(0.123_456, 2), 0.12);
        assert_eq!(quantize(2.5, 0), 3.0);
        assert_eq!(format(0.5, 2), "0.50");
        assert_eq!(format(30.0, 0), "30");
    }

    /// A range that crosses zero fills out from zero; one that does not fills
    /// from its start.
    #[test]
    fn the_fill_starts_at_zero_when_there_is_one() {
        let even = |lo, hi| Scale::new(lo, hi, false);
        assert_eq!(fill_span(0.5, even(0.0, 1.0)), (0.0, 0.5));
        assert_eq!(fill_span(0.5, even(-1.0, 1.0)), (0.5, 0.75));
        assert_eq!(fill_span(-0.5, even(-1.0, 1.0)), (0.25, 0.5));
        assert_eq!(fill_span(0.0, even(-1.0, 1.0)), (0.5, 0.5));
        let (a, b) = fill_span(1.0, even(0.2, 5.0));
        assert_eq!(a, 0.0);
        assert!((b - even(0.2, 5.0).fraction(1.0)).abs() < 1e-6);
    }

    /// Logarithmically, each tenfold step takes the same length of track,
    /// and the mapping comes back to the value it started from.
    #[test]
    fn the_logarithmic_scale_maps_both_ways() {
        let log = Scale::new(0.1, 10.0, true);
        assert!((log.fraction(1.0) - 0.5).abs() < 1e-6, "1 sits midway between 0.1 and 10");
        assert!((log.fraction(0.1)).abs() < 1e-6);
        assert!((log.fraction(10.0) - 1.0).abs() < 1e-6);
        for value in [0.1, 0.25, 1.0, 3.3, 10.0] {
            assert!((log.value_at(log.fraction(value)) - value).abs() < 1e-5, "{value}");
        }
        let eq = Scale::new(20.0, 20_000.0, true);
        assert!((eq.value_at(0.5) - 632.455).abs() < 0.01, "a frequency's middle");
        assert!(!Scale::new(0.0, 10.0, true).log, "a range from zero stays even");
    }

    /// The gradient runs from the first stop's colour to the last's, through
    /// those between, across the whole track.
    #[test]
    fn the_gradient_paints_its_stops() {
        let bar = Rect::from_min_max(Pos2::new(0.0, 0.0), Pos2::new(100.0, 6.0));
        let (cool, white, warm) = (Color32::from_rgb(80, 140, 255), Color32::WHITE, Color32::from_rgb(255, 160, 60));
        let shapes = gradient(bar, &[(0.0, cool), (0.5, white), (1.0, warm)]);
        let mesh = shapes
            .iter()
            .find_map(|s| if let Shape::Mesh(mesh) = s { Some(mesh.clone()) } else { None })
            .expect("a mesh");
        let at = |x: f32| mesh.vertices.iter().find(|v| (v.pos.x - x).abs() < 0.01).map(|v| v.color);
        // The ends round off inside the track, half its thickness in.
        assert_eq!(at(3.0), Some(cool));
        assert_eq!(at(50.0), Some(white));
        assert_eq!(at(97.0), Some(warm));
        assert!(mesh.vertices.iter().all(|v| (3.0..=97.0).contains(&v.pos.x)));
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
