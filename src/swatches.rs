//! Colour swatches: the standard colours as a grid to pick from, and the
//! button that opens them.

use egui::{Color32, Id, Painter, Popup, Pos2, Rect, Response, RichText, Sense, Stroke, StrokeKind, Ui, Vec2, Widget};

use crate::{Button, Ink, Size, TextField};

/// The standard colours' hues; the rows below the greys are these lightened,
/// plain and darkened.
const HUES: [[u8; 3]; 10] = [
    [192, 0, 0],
    [255, 0, 0],
    [255, 192, 0],
    [255, 255, 0],
    [146, 208, 80],
    [0, 176, 80],
    [0, 176, 240],
    [0, 112, 192],
    [0, 32, 96],
    [112, 48, 160],
];

/// The top row: white through black.
const GREYS: [[u8; 3]; 10] = [
    [255, 255, 255],
    [242, 242, 242],
    [217, 217, 217],
    [191, 191, 191],
    [166, 166, 166],
    [128, 128, 128],
    [89, 89, 89],
    [64, 64, 64],
    [38, 38, 38],
    [0, 0, 0],
];

/// One square's side, and the room between squares.
const SQUARE: f32 = 18.0;
const GAP: f32 = 4.0;

/// The standard colours, row by row: greys, then the hues lightened, plain
/// and darkened.
pub fn standard_colours() -> Vec<[Color32; 10]> {
    let tint = |c: [u8; 3], f: f32| {
        let target = if f > 0.0 { 255.0 } else { 0.0 };
        let mix = |v: u8| (v as f32 + (target - v as f32) * f.abs()).round().clamp(0.0, 255.0) as u8;
        Color32::from_rgb(mix(c[0]), mix(c[1]), mix(c[2]))
    };
    let mut rows = vec![GREYS.map(|c| Color32::from_rgb(c[0], c[1], c[2]))];
    for f in [0.6, 0.4, 0.0, -0.3] {
        rows.push(HUES.map(|c| tint(c, f)));
    }
    rows
}

/// A colour as `#RRGGBB`.
pub fn hex(colour: Color32) -> String {
    format!("#{:02X}{:02X}{:02X}", colour.r(), colour.g(), colour.b())
}

/// `#RRGGBB` or `RRGGBB` as a colour.
fn parse_hex(text: &str) -> Option<Color32> {
    let digits = text.trim().trim_start_matches('#');
    if digits.len() != 6 {
        return None;
    }
    let v = u32::from_str_radix(digits, 16).ok()?;
    Some(Color32::from_rgb((v >> 16) as u8, (v >> 8) as u8, v as u8))
}

/// The standard colours as a grid of squares, with an optional entry for no
/// colour at the top, the colours used last under the grid, and a field for
/// any colour as `#RRGGBB` at the bottom. The chosen one is outlined; a
/// click picks one, and `changed()` says so. `None` is no colour.
#[must_use = "swatches are drawn with ui.add(swatches)"]
pub struct Swatches<'a> {
    colour: &'a mut Option<Color32>,
    clear: Option<String>,
    recent: &'a [Color32],
    id: Option<Id>,
}

impl<'a> Swatches<'a> {
    pub fn new(colour: &'a mut Option<Color32>) -> Swatches<'a> {
        Swatches { colour, clear: None, recent: &[], id: None }
    }

    /// The entry for no colour, and what it says: "No fill", "Automatic".
    pub fn clear(mut self, label: impl Into<String>) -> Swatches<'a> {
        self.clear = Some(label.into());
        self
    }

    /// The colours used last, newest first, on a row of their own.
    pub fn recent(mut self, recent: &'a [Color32]) -> Swatches<'a> {
        self.recent = recent;
        self
    }

    /// Where egui keeps the field's state, when two sets of swatches would
    /// otherwise share it.
    pub fn id_salt(mut self, salt: impl std::hash::Hash) -> Swatches<'a> {
        self.id = Some(Id::new(salt));
        self
    }
}

impl Widget for Swatches<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        let Swatches { colour, clear, recent, id } = self;
        let width = 10.0 * SQUARE + 9.0 * GAP;
        let id = id.unwrap_or_else(|| ui.next_auto_id().with("omarchy-kit-swatches"));
        let mut picked: Option<Option<Color32>> = None;
        let mut inner = ui.scope(|ui| {
            ui.set_width(width);
            ui.spacing_mut().item_spacing = Vec2::splat(GAP);
            if let Some(label) = &clear {
                let none = Button::new(label.as_str()).small().width(width).selected(colour.is_none());
                if ui.add(none).clicked() {
                    picked = Some(None);
                }
                ui.add_space(2.0);
            }
            for row in standard_colours() {
                if let Some(c) = squares(ui, &row, *colour) {
                    picked = Some(Some(c));
                }
            }
            if !recent.is_empty() {
                ui.add_space(2.0);
                ui.label(RichText::new("Recent").size(Size::Small.text()).weak());
                if let Some(c) = squares(ui, &recent[..recent.len().min(10)], *colour) {
                    picked = Some(Some(c));
                }
            }
            ui.add_space(2.0);
            let field_id = id.with("hex");
            let mut text = ui
                .data(|d| d.get_temp::<String>(field_id))
                .unwrap_or_else(|| colour.map(hex).unwrap_or_default());
            let typed = ui.add(TextField::new(&mut text).hint("#RRGGBB").small().width(width).id_salt(field_id));
            if typed.has_focus() {
                ui.data_mut(|d| d.insert_temp(field_id, text.clone()));
            } else {
                ui.data_mut(|d| d.remove::<String>(field_id));
            }
            if typed.lost_focus()
                && let Some(c) = parse_hex(&text)
            {
                picked = Some(Some(c));
            }
        })
        .response;
        if let Some(choice) = picked
            && choice != *colour
        {
            *colour = choice;
            inner.mark_changed();
        }
        inner
    }
}

/// One row of squares; the colour clicked, if one was.
fn squares(ui: &mut Ui, row: &[Color32], chosen: Option<Color32>) -> Option<Color32> {
    let ink = Ink::of(ui);
    let radius = Size::Small.radius() / 2.0;
    let mut clicked = None;
    ui.horizontal(|ui| {
        for &c in row {
            let (rect, response) = ui.allocate_exact_size(Vec2::splat(SQUARE), Sense::click());
            let painter = ui.painter();
            painter.rect_filled(rect, radius, c);
            painter.rect_stroke(rect, radius, Stroke::new(1.0_f32, ink.shade(0.28)), StrokeKind::Inside);
            if chosen == Some(c) || response.hovered() {
                let colour = if chosen == Some(c) { ink.accent } else { ink.fg };
                painter.rect_stroke(rect.expand(2.0), radius + 2.0, Stroke::new(1.5_f32, colour), StrokeKind::Outside);
            }
            if response.on_hover_text(hex(c)).clicked() {
                clicked = Some(c);
            }
        }
    });
    clicked
}

/// An icon, drawn in the square it is given, in the colour it is given.
type Icon<'a> = Box<dyn Fn(&Painter, Rect, Color32) + 'a>;

/// A quiet button with the colour it applies under its icon, opening the
/// swatches below it; a pick closes them, and `changed()` says so.
#[must_use = "a colour button is drawn with ui.add(button)"]
pub struct ColorButton<'a> {
    colour: &'a mut Option<Color32>,
    icon: Icon<'a>,
    clear: Option<String>,
    recent: &'a [Color32],
    size: Size,
}

impl<'a> ColorButton<'a> {
    pub fn new(colour: &'a mut Option<Color32>, icon: impl Fn(&Painter, Rect, Color32) + 'a) -> ColorButton<'a> {
        ColorButton { colour, icon: Box::new(icon), clear: None, recent: &[], size: Size::default() }
    }

    /// The swatches' entry for no colour, and what it says.
    pub fn clear(mut self, label: impl Into<String>) -> ColorButton<'a> {
        self.clear = Some(label.into());
        self
    }

    /// The colours used last, newest first.
    pub fn recent(mut self, recent: &'a [Color32]) -> ColorButton<'a> {
        self.recent = recent;
        self
    }

    pub fn size(mut self, size: Size) -> ColorButton<'a> {
        self.size = size;
        self
    }

    pub fn small(self) -> ColorButton<'a> {
        self.size(Size::Small)
    }
}

impl Widget for ColorButton<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        let ColorButton { colour, icon, clear, recent, size } = self;
        let ink = Ink::of(ui);
        let mut response = ui.add(Button::icon_only(icon).quiet().size(size));
        // The colour, as a bar under the icon; an outline when there is none.
        let bar = Rect::from_center_size(
            Pos2::new(response.rect.center().x, response.rect.center().y + size.icon() / 2.0 + 1.5),
            Vec2::new(size.icon(), 3.0),
        );
        match *colour {
            Some(c) => {
                ui.painter().rect_filled(bar, 1.0, c);
            }
            None => {
                ui.painter().rect_stroke(bar, 1.0, Stroke::new(1.0_f32, ink.weak), StrokeKind::Inside);
            }
        }
        let mut picked = false;
        Popup::menu(&response).show(|ui| {
            let mut swatches = Swatches::new(colour).recent(recent).id_salt(response.id.with("swatches"));
            if let Some(label) = &clear {
                swatches = swatches.clear(label.as_str());
            }
            if ui.add(swatches).changed() {
                picked = true;
                ui.close();
            }
        });
        if picked {
            response.mark_changed();
        }
        response
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Five rows of ten: greys first, white to black, then the hues.
    #[test]
    fn the_standard_colours_are_five_rows_of_ten() {
        let rows = standard_colours();
        assert_eq!(rows.len(), 5);
        assert_eq!(rows[0][0], Color32::WHITE);
        assert_eq!(rows[0][9], Color32::BLACK);
        assert_eq!(rows[3][1], Color32::from_rgb(255, 0, 0), "the plain row is the hues as they are");
    }

    /// A colour reads back from the way it is written.
    #[test]
    fn hex_round_trips() {
        let c = Color32::from_rgb(0x12, 0xAB, 0xEF);
        assert_eq!(hex(c), "#12ABEF");
        assert_eq!(parse_hex("#12abef"), Some(c));
        assert_eq!(parse_hex("12ABEF"), Some(c));
        assert_eq!(parse_hex("#12AB"), None);
    }
}
