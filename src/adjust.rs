//! Adjustments: a drag, and the nudges after it while the keyboard stays on
//! the field, are one adjustment — the one undo step an app records.

use egui::{Id, Response, Ui};

#[derive(Clone, Copy, Default)]
struct State {
    adjusting: bool,
    ended: bool,
}

fn key(id: Id) -> Id {
    id.with("omarchy-kit-adjust")
}

/// Whether a [`crate::Slider`] or a [`crate::NumberField`] is being adjusted,
/// read off the response it returned:
///
/// ```ignore
/// use omarchy_kit::Adjust;
///
/// let slider = ui.add(Slider::new(&mut clip.scale, 0.1..=10.0).label("Scale"));
/// if slider.adjust_ended() {
///     history.record("Scale");
/// }
/// ```
pub trait Adjust {
    /// From the first press on the track, the first drag on the field or the
    /// first keyboard nudge, until the pointer is up and the field has given
    /// up the keyboard.
    fn adjusting(&self) -> bool;

    /// True on the one frame an adjustment ended.
    fn adjust_ended(&self) -> bool;
}

impl Adjust for Response {
    fn adjusting(&self) -> bool {
        state(self).adjusting
    }

    fn adjust_ended(&self) -> bool {
        state(self).ended
    }
}

fn state(response: &Response) -> State {
    response.ctx.data(|d| d.get_temp::<State>(key(response.id))).unwrap_or_default()
}

/// Moves on the adjustment of the element whose response has `id`, once a
/// frame: `began` when something moved it this frame, `holding` while the
/// pointer is down on it or its field has the keyboard.
pub(crate) fn record(ui: &Ui, id: Id, began: bool, holding: bool) {
    let was = ui.data(|d| d.get_temp::<State>(key(id))).unwrap_or_default().adjusting;
    let active = was || began;
    let state = State { adjusting: active && holding, ended: active && !holding };
    ui.data_mut(|d| d.insert_temp(key(id), state));
}
