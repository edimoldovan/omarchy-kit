//! How a selectable item is marked: a row of a list, an entry of a sidebar,
//! a card.

use egui::{Rect, Stroke, StrokeKind, Ui};

use crate::{Ink, Size};

/// What a selectable item is showing, painted under it: filled while it is
/// open — on screen beside the list, say — a quiet fill under the pointer,
/// and the accent outline where the keyboard is. All three in the size's
/// corner, so an item, its fill and its outline are one shape.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Item {
    pub open: bool,
    pub keyed: bool,
    pub hovered: bool,
    pub size: Size,
}

impl Item {
    /// Paints the marks into `rect`, under whatever the item draws after.
    pub fn paint(self, ui: &Ui, rect: Rect) {
        let ink = Ink::of(ui);
        let radius = self.size.radius();
        if self.open {
            ui.painter().rect_filled(rect, radius, ink.selected);
        } else if self.hovered {
            ui.painter().rect_filled(rect, radius, ink.shade(0.07));
        }
        if self.keyed {
            // Inside the shape, at its own corner: an outline drawn around
            // the outside would need a corner of its own to stay concentric.
            ui.painter().rect_stroke(
                rect,
                radius,
                Stroke::new(1.0_f32, ink.accent),
                StrokeKind::Inside,
            );
        }
    }
}
