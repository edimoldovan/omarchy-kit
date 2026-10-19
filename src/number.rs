//! The number field: one number, typed, nudged or dragged, within its range.

use std::ops::RangeInclusive;

use egui::{CursorIcon, Id, Key, Modifiers, Response, Sense, Ui, Widget};

use crate::slider::{Amount, format, quantize};
use crate::{Size, TextField, adjust};

/// One number in the kit's field, kept within its range. Dragged sideways it
/// moves by its speed per point; a click without a drag puts the cursor in
/// to type. While the field has the keyboard, Up and Down move it a step,
/// Shift ten steps, Ctrl a hundred; Enter, or leaving the field, takes what
/// was typed, within the range; Esc puts back what it was when the field
/// took the keyboard. `changed()` whenever the number moved; the response's
/// [`crate::Adjust`] says when an adjustment ends. A spin button to a screen
/// reader and a test, with the number as its value.
#[must_use = "a number field is drawn with ui.add(field)"]
pub struct NumberField<'a, T: Amount = f32> {
    value: &'a mut T,
    range: RangeInclusive<T>,
    decimals: usize,
    speed: Option<f64>,
    prefix: String,
    suffix: String,
    name: String,
    size: Size,
    width: Option<f32>,
    id: Option<Id>,
}

impl<'a, T: Amount> NumberField<'a, T> {
    /// `range` may be open at an end: `0.0..=f64::MAX`.
    pub fn new(value: &'a mut T, range: RangeInclusive<T>) -> NumberField<'a, T> {
        NumberField {
            value,
            range,
            decimals: 0,
            speed: None,
            prefix: String::new(),
            suffix: String::new(),
            name: String::new(),
            size: Size::default(),
            width: None,
            id: None,
        }
    }

    /// How many decimals it shows, and the step it moves in; none when unset,
    /// which makes it a field of whole numbers.
    pub fn decimals(mut self, decimals: usize) -> NumberField<'a, T> {
        self.decimals = decimals;
        self
    }

    /// How much a drag moves it per point; a step per point when unset.
    pub fn speed(mut self, speed: f64) -> NumberField<'a, T> {
        self.speed = Some(speed);
        self
    }

    /// Text before the number in its field: a currency, a sign.
    pub fn prefix(mut self, prefix: impl Into<String>) -> NumberField<'a, T> {
        self.prefix = prefix.into();
        self
    }

    /// Text after the number in its field: a unit, a percent sign.
    pub fn suffix(mut self, suffix: impl Into<String>) -> NumberField<'a, T> {
        self.suffix = suffix.into();
        self
    }

    /// Its name to a screen reader and a test.
    pub fn name(mut self, name: impl Into<String>) -> NumberField<'a, T> {
        self.name = name.into();
        self
    }

    pub fn size(mut self, size: Size) -> NumberField<'a, T> {
        self.size = size;
        self
    }

    pub fn small(self) -> NumberField<'a, T> {
        self.size(Size::Small)
    }

    /// As wide as `width`, frame and all; 88 points and the prefix when
    /// unset.
    pub fn width(mut self, width: f32) -> NumberField<'a, T> {
        self.width = Some(width);
        self
    }

    /// Where egui keeps its state, when two fields would otherwise share it.
    pub fn id_salt(mut self, salt: impl std::hash::Hash) -> NumberField<'a, T> {
        self.id = Some(Id::new(salt));
        self
    }
}

impl<T: Amount> Widget for NumberField<'_, T> {
    fn ui(self, ui: &mut Ui) -> Response {
        let (lo, hi) = (self.range.start().to_f64(), self.range.end().to_f64());
        let step = 10f64.powi(-(self.decimals as i32));
        let id = self.id.unwrap_or_else(|| ui.next_auto_id().with("omarchy-kit-number"));
        // The field's own egui id, as TextField makes it from the salt.
        let field_id = id.with("field");
        let edit_id = Id::new(field_id);
        let orig_id = id.with("orig");
        let from_id = id.with("drag-from");
        let nudged_id = id.with("nudged");
        let had_id = id.with("had-focus");
        let focused = ui.memory(|m| m.has_focus(edit_id));
        // egui drops the keyboard on Esc before anything is drawn; what the
        // field had last frame says the Esc was for it.
        let had_focus = ui.data(|d| d.get_temp::<bool>(had_id)).unwrap_or(false);
        let original = self.value.to_f64();
        // What Esc puts back: the number as the field took the keyboard,
        // however it came to have it.
        if focused && !had_focus {
            ui.data_mut(|d| d.insert_temp(orig_id, original));
        }
        let mut value = original;
        let mut nudged = false;

        if focused {
            let steps = ui.input_mut(|i| {
                // The fuller chords first: a plain arrow would claim a Shift
                // arrow, since egui matches with at least the modifiers.
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
            // A frame egui runs again sees the key again: once per frame.
            let frame = ui.ctx().cumulative_frame_nr();
            let done = ui.data(|d| d.get_temp::<u64>(nudged_id)) == Some(frame);
            if steps != 0 && !done {
                value = quantize((value + f64::from(steps) * step).clamp(lo, hi), self.decimals);
                nudged = true;
                ui.data_mut(|d| d.insert_temp(nudged_id, frame));
            }
        }
        let escaped = (focused || had_focus) && ui.input(|i| i.key_pressed(Key::Escape));

        // What is typed stays while the field has the keyboard and is taken
        // when it leaves; a nudge or a drag writes the number into it.
        let mut text = if focused && !nudged {
            ui.data(|d| d.get_temp::<String>(field_id)).unwrap_or_else(|| format(value, self.decimals))
        } else {
            format(value, self.decimals)
        };
        let prefix = self.prefix.trim();
        let width = self.width.unwrap_or_else(|| {
            let font = egui::FontId::proportional(self.size.text());
            let measured = ui.painter().layout_no_wrap(prefix.to_string(), font, egui::Color32::PLACEHOLDER).size().x;
            88.0 + if prefix.is_empty() { 0.0 } else { measured + 7.0 }
        });
        let field = TextField::new(&mut text)
            .name(&self.name)
            .prefix(prefix)
            .width(width)
            .size(self.size)
            .id_salt(field_id);
        let field = if self.suffix.is_empty() { field } else { field.key(self.suffix.trim()) };
        let mut edit = ui.add(field);

        // Dragged sideways while it does not have the keyboard, the number
        // follows the pointer; a click without a drag hands the keyboard to
        // the text.
        let mut dragging = false;
        if !edit.has_focus() {
            let drag = ui.interact(edit.rect, id.with("drag"), Sense::click_and_drag()).on_hover_cursor(CursorIcon::ResizeHorizontal);
            if drag.clicked() {
                ui.memory_mut(|m| m.request_focus(edit_id));
            }
            if drag.drag_started() {
                ui.data_mut(|d| d.insert_temp(from_id, original));
            }
            if drag.dragged()
                && let (Some(at), Some(origin)) = (drag.interact_pointer_pos(), ui.input(|i| i.pointer.press_origin()))
            {
                let from = ui.data(|d| d.get_temp::<f64>(from_id)).unwrap_or(original);
                let speed = self.speed.unwrap_or(step);
                value = quantize((from + f64::from(at.x - origin.x) * speed).clamp(lo, hi), self.decimals);
                dragging = true;
            }
        }

        // Or taken it just now, by a click.
        if edit.has_focus() && !had_focus && !focused {
            ui.data_mut(|d| d.insert_temp(orig_id, original));
        }
        if edit.has_focus() {
            ui.data_mut(|d| d.insert_temp(field_id, text.clone()));
        }
        let has_focus = edit.has_focus();
        ui.data_mut(|d| d.insert_temp(had_id, has_focus));
        if escaped {
            value = ui.data(|d| d.get_temp::<f64>(orig_id)).unwrap_or(value);
        } else if edit.lost_focus() {
            let typed = text.trim().trim_start_matches(self.prefix.trim()).trim_end_matches(self.suffix.trim()).trim();
            if let Ok(typed) = typed.parse::<f64>() {
                value = quantize(typed.clamp(lo, hi), self.decimals);
            }
        }

        let taken = T::from_f64(value);
        if taken != *self.value {
            *self.value = taken;
            edit.mark_changed();
        }
        adjust::record(ui, edit.id, dragging || nudged || value != original, dragging || has_focus);
        edit.ctx.accesskit_node_builder(edit.id, |node| {
            node.set_role(egui::accesskit::Role::SpinButton);
            node.set_numeric_value(value);
        });
        edit
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use egui::{Context, Event, RawInput};

    fn run(ctx: &Context, events: Vec<Event>, value: &mut f32) {
        let input = RawInput { events, ..RawInput::default() };
        let _ = ctx.run(input, |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                ui.add(NumberField::new(value, 6.0..=96.0).id_salt("size"));
            });
        });
    }

    fn key(key: Key, modifiers: Modifiers) -> Event {
        Event::Key { key, physical_key: None, pressed: true, repeat: false, modifiers }
    }

    /// Typed and taken on Enter, within the range; nudged by the arrows; put
    /// back by Esc.
    #[test]
    fn typed_nudged_and_put_back() {
        let ctx = Context::default();
        let mut value = 11.0;
        let edit = Id::new(Id::new("size").with("field"));
        ctx.memory_mut(|m| m.request_focus(edit));
        run(&ctx, vec![], &mut value);
        run(&ctx, vec![key(Key::A, Modifiers::CTRL), Event::Text("200".into())], &mut value);
        run(&ctx, vec![key(Key::Enter, Modifiers::NONE)], &mut value);
        assert_eq!(value, 96.0, "kept within the range");

        ctx.memory_mut(|m| m.request_focus(edit));
        run(&ctx, vec![], &mut value);
        run(&ctx, vec![key(Key::ArrowDown, Modifiers::SHIFT)], &mut value);
        assert_eq!(value, 86.0, "Shift moves ten steps");
        run(&ctx, vec![key(Key::Escape, Modifiers::NONE)], &mut value);
        run(&ctx, vec![], &mut value);
        assert_eq!(value, 96.0, "Esc puts back what it was");
    }
}
