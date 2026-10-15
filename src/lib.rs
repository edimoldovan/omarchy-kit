//! egui in the colours of the Omarchy theme.
//!
//! [`theme`] reads the active Omarchy theme — background, foreground and
//! accent — and turns it into egui's visuals; a machine without Omarchy gets a
//! neutral dark set instead. The widgets draw from those visuals and from
//! nothing else, so they follow whatever theme is applied, and every app that
//! uses them looks like one product:
//!
//! - [`Button`]: four kinds — primary, secondary, quiet, danger — in three
//!   sizes, with an optional icon and key hint;
//! - [`TextField`], and [`field`] for a field that holds more than text;
//! - [`Toggle`]: a choice of a few, shown together;
//! - [`Slider`]: an amount between two ends, dragged;
//! - [`Item`]: how a row, a sidebar entry or a card is marked — open, under the
//!   pointer, where the keyboard is;
//! - [`CommandPalette`]: every action there is, found by typing, each with its
//!   key.
//!
//! A size is one height, one corner and one type size, for buttons, fields,
//! toggles and items alike, so everything lines up wherever it meets. Rows,
//! entries and cards take the regular corner; what sits inside one — chips,
//! badges, avatars — the small one.

pub mod theme;

mod button;
mod field;
mod item;
mod palette;
mod slider;
mod toggle;

pub use button::{Button, Kind};
pub use field::{TextField, field};
pub use item::Item;
pub use palette::{Choice, CommandPalette};
pub use slider::Slider;
pub use toggle::Toggle;

use egui::{Color32, Ui};

/// How big a control is.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Size {
    /// Lists, side bars, chips: where a regular control would crowd.
    Small,
    /// Everything else.
    #[default]
    Regular,
    /// The one control a screen leads with, such as Compose.
    Large,
}

impl Size {
    pub fn height(self) -> f32 {
        match self {
            Size::Small => 26.0,
            Size::Regular => 32.0,
            Size::Large => 42.0,
        }
    }

    pub fn radius(self) -> f32 {
        match self {
            Size::Small => 7.0,
            Size::Regular => 8.0,
            Size::Large => 10.0,
        }
    }

    /// The type size of a label.
    pub fn text(self) -> f32 {
        match self {
            Size::Small => 12.5,
            Size::Regular => 14.0,
            Size::Large => 15.0,
        }
    }

    /// Room on either side of what a control says.
    pub fn pad(self) -> f32 {
        match self {
            Size::Small => 9.0,
            Size::Regular => 13.0,
            Size::Large => 16.0,
        }
    }

    /// The square an icon is drawn in.
    pub fn icon(self) -> f32 {
        match self {
            Size::Small => 14.0,
            Size::Regular => 17.0,
            Size::Large => 19.0,
        }
    }
}

/// Text that reads on `fill`: white on a dark colour, near-black on a light
/// one.
pub fn on(fill: Color32) -> Color32 {
    let light = 0.299 * fill.r() as f32 + 0.587 * fill.g() as f32 + 0.114 * fill.b() as f32;
    if light > 160.0 {
        Color32::from_gray(20)
    } else {
        Color32::WHITE
    }
}

/// The colours a control is drawn in, read off the visuals the theme set.
#[derive(Clone, Copy)]
struct Ink {
    bg: Color32,
    fg: Color32,
    weak: Color32,
    accent: Color32,
    danger: Color32,
    selected: Color32,
}

impl Ink {
    fn of(ui: &Ui) -> Ink {
        let v = ui.visuals();
        Ink {
            bg: v.panel_fill,
            fg: v.text_color(),
            weak: v.weak_text_color(),
            accent: v.selection.stroke.color,
            danger: v.error_fg_color,
            selected: v.selection.bg_fill,
        }
    }

    /// The background moved `t` of the way towards the text: the quiet fills
    /// a control is drawn in, darker or lighter as the theme is.
    fn shade(&self, t: f32) -> Color32 {
        theme::mix(self.bg, self.fg, t)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The sizes go one way: a small control is smaller in every respect.
    #[test]
    fn the_sizes_grow_together() {
        for (small, big) in [(Size::Small, Size::Regular), (Size::Regular, Size::Large)] {
            assert!(small.height() < big.height());
            assert!(small.radius() < big.radius());
            assert!(small.text() < big.text());
            assert!(small.icon() < big.icon());
        }
        assert_eq!(on(Color32::from_rgb(0xf0, 0xe0, 0x60)), Color32::from_gray(20));
        assert_eq!(on(Color32::from_rgb(0x1e, 0x71, 0x45)), Color32::WHITE);
    }
}
