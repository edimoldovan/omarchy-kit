//! Colour swatches: the standard colours as a grid to pick from, and the
//! button that opens them.

use egui::{
    Color32, Id, Painter, Popup, Pos2, Rect, Response, RichText, Sense, Stroke, StrokeKind, Ui, Vec2, Widget, WidgetInfo,
    WidgetType,
};

use crate::{Button, Ink, Size, Slider, TextField};

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

/// A colour as `#RRGGBB`, its opacity left out.
pub fn hex(colour: Color32) -> String {
    let [r, g, b, _] = colour.to_srgba_unmultiplied();
    format!("#{r:02X}{g:02X}{b:02X}")
}

/// A colour as `#AARRGGBB`: its opacity first, then the colour.
pub fn hex_argb(colour: Color32) -> String {
    let [r, g, b, a] = colour.to_srgba_unmultiplied();
    format!("#{a:02X}{r:02X}{g:02X}{b:02X}")
}

/// `#AARRGGBB`, or `#RRGGBB` for an opaque colour, as a colour; the `#` may
/// be left out.
pub fn parse_argb(text: &str) -> Option<Color32> {
    let digits = text.trim().trim_start_matches('#');
    if digits.len() == 6 {
        return parse_hex(digits);
    }
    if digits.len() != 8 {
        return None;
    }
    let v = u32::from_str_radix(digits, 16).ok()?;
    Some(Color32::from_rgba_unmultiplied((v >> 16) as u8, (v >> 8) as u8, v as u8, (v >> 24) as u8))
}

/// How a colour is written: `#RRGGBB` when it is opaque, `#AARRGGBB` when
/// it is see-through.
fn written(colour: Color32) -> String {
    if colour.a() == 255 { hex(colour) } else { hex_argb(colour) }
}

/// `colour` at opacity `alpha`.
fn with_alpha(colour: Color32, alpha: u8) -> Color32 {
    let [r, g, b, _] = colour.to_srgba_unmultiplied();
    Color32::from_rgba_unmultiplied(r, g, b, alpha)
}

/// A checkerboard under `rect`, so what is see-through over it shows as such.
fn checker(painter: &Painter, rect: Rect, ink: &Ink) {
    let cell = 5.0;
    painter.rect_filled(rect, 0.0, ink.shade(0.42));
    let (cols, rows) = ((rect.width() / cell).ceil() as usize, (rect.height() / cell).ceil() as usize);
    for row in 0..rows {
        for col in (row % 2..cols).step_by(2) {
            let min = rect.min + Vec2::new(col as f32 * cell, row as f32 * cell);
            let square = Rect::from_min_size(min, Vec2::splat(cell)).intersect(rect);
            painter.rect_filled(square, 0.0, ink.shade(0.20));
        }
    }
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
    alpha: bool,
    id: Option<Id>,
}

impl<'a> Swatches<'a> {
    pub fn new(colour: &'a mut Option<Color32>) -> Swatches<'a> {
        Swatches { colour, clear: None, recent: &[], alpha: false, id: None }
    }

    /// Colours may be see-through: an opacity slider under the grid, which a
    /// pick keeps; see-through colours over a checkerboard; and the field
    /// taking and showing `#AARRGGBB` as well as `#RRGGBB`.
    pub fn alpha(mut self) -> Swatches<'a> {
        self.alpha = true;
        self
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
        let Swatches { colour, clear, recent, alpha, id } = self;
        // A standard colour picked keeps the opacity there is.
        let opacity = colour.map_or(255, |c| c.a());
        let keep = |c: Color32| if alpha { with_alpha(c, opacity) } else { c };
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
                    picked = Some(Some(keep(c)));
                }
            }
            if !recent.is_empty() {
                ui.add_space(2.0);
                ui.label(RichText::new("Recent").size(Size::Small.text()).weak());
                if let Some(c) = squares(ui, &recent[..recent.len().min(10)], *colour) {
                    picked = Some(Some(c));
                }
            }
            if alpha {
                ui.add_space(2.0);
                let mut percent = f32::from(opacity) / 2.55;
                let slider = Slider::new(&mut percent, 0.0..=100.0)
                    .label("Opacity")
                    .suffix("%")
                    .decimals(0)
                    .small()
                    .width(width)
                    .id_salt(id.with("opacity"));
                if ui.add_enabled(colour.is_some(), slider).changed()
                    && let Some(c) = *colour
                {
                    picked = Some(Some(with_alpha(c, (percent * 2.55).round() as u8)));
                }
            }
            ui.add_space(2.0);
            let field_id = id.with("hex");
            let shown = |c: Color32| if alpha { written(c) } else { hex(c) };
            let mut text = ui
                .data(|d| d.get_temp::<String>(field_id))
                .unwrap_or_else(|| colour.map(shown).unwrap_or_default());
            let hint = if alpha { "#RRGGBB or #AARRGGBB" } else { "#RRGGBB" };
            let typed = ui.add(TextField::new(&mut text).hint(hint).small().width(width).id_salt(field_id));
            if typed.has_focus() {
                ui.data_mut(|d| d.insert_temp(field_id, text.clone()));
            } else {
                ui.data_mut(|d| d.remove::<String>(field_id));
            }
            let parsed = if alpha { parse_argb(&text) } else { parse_hex(&text) };
            if typed.lost_focus()
                && let Some(c) = parsed
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
            let on = chosen.is_some_and(|chosen| hex(chosen) == hex(c));
            response.widget_info(|| WidgetInfo::selected(WidgetType::ColorButton, true, on, written(c)));
            let painter = ui.painter();
            if c.a() < 255 {
                checker(&painter.with_clip_rect(rect), rect, &ink);
            }
            painter.rect_filled(rect, radius, c);
            painter.rect_stroke(rect, radius, Stroke::new(1.0_f32, ink.shade(0.28)), StrokeKind::Inside);
            if on || response.hovered() {
                let colour = if on { ink.accent } else { ink.fg };
                painter.rect_stroke(rect.expand(2.0), radius + 2.0, Stroke::new(1.5_f32, colour), StrokeKind::Outside);
            }
            if response.on_hover_text(written(c)).clicked() {
                clicked = Some(c);
            }
        }
    });
    clicked
}

/// An icon, drawn in the square it is given, in the colour it is given.
type Icon<'a> = Box<dyn Fn(&Painter, Rect, Color32) + 'a>;

/// A quiet button with the colour it applies under its icon, opening the
/// swatches below it; a pick closes them, and `changed()` says so. To a
/// screen reader and a test it is named by its tip, with the colour's hex as
/// its value; each swatch is named by its hex.
#[must_use = "a colour button is drawn with ui.add(button)"]
pub struct ColorButton<'a> {
    colour: &'a mut Option<Color32>,
    icon: Icon<'a>,
    tip: String,
    clear: Option<String>,
    recent: &'a [Color32],
    alpha: bool,
    size: Size,
}

impl<'a> ColorButton<'a> {
    pub fn new(colour: &'a mut Option<Color32>, icon: impl Fn(&Painter, Rect, Color32) + 'a) -> ColorButton<'a> {
        ColorButton { colour, icon: Box::new(icon), tip: String::new(), clear: None, recent: &[], alpha: false, size: Size::default() }
    }

    /// What it says while the pointer is on it, and its name.
    pub fn tip(mut self, tip: impl Into<String>) -> ColorButton<'a> {
        self.tip = tip.into();
        self
    }

    /// The swatches' entry for no colour, and what it says.
    pub fn clear(mut self, label: impl Into<String>) -> ColorButton<'a> {
        self.clear = Some(label.into());
        self
    }

    /// Colours may be see-through, as [`Swatches::alpha`].
    pub fn alpha(mut self) -> ColorButton<'a> {
        self.alpha = true;
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
        let ColorButton { colour, icon, tip, clear, recent, alpha, size } = self;
        let ink = Ink::of(ui);
        let mut button = Button::icon_only(icon).quiet().size(size);
        if !tip.is_empty() {
            button = button.tip(tip);
        }
        let mut response = ui.add(button);
        // The colour, as a bar under the icon; an outline when there is none.
        let bar = Rect::from_center_size(
            Pos2::new(response.rect.center().x, response.rect.center().y + size.icon() / 2.0 + 1.5),
            Vec2::new(size.icon(), 3.0),
        );
        match *colour {
            Some(c) => {
                if c.a() < 255 {
                    checker(&ui.painter().with_clip_rect(bar), bar, &ink);
                }
                ui.painter().rect_filled(bar, 1.0, c);
            }
            None => {
                ui.painter().rect_stroke(bar, 1.0, Stroke::new(1.0_f32, ink.weak), StrokeKind::Inside);
            }
        }
        let mut picked = false;
        Popup::menu(&response).show(|ui| {
            let mut swatches = Swatches::new(colour).recent(recent).id_salt(response.id.with("swatches"));
            if alpha {
                swatches = swatches.alpha();
            }
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
        let value = colour.map(written);
        response.ctx.accesskit_node_builder(response.id, |node| match value {
            Some(value) => node.set_value(value),
            None => node.clear_value(),
        });
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
        let see_through = parse_argb("#80FF0000").expect("eight digits");
        assert_eq!(see_through.to_srgba_unmultiplied(), [255, 0, 0, 0x80]);
        assert_eq!(hex_argb(see_through), "#80FF0000");
        assert_eq!(written(see_through), "#80FF0000");
        assert_eq!(hex(see_through), "#FF0000", "the colour without its opacity");
        assert_eq!(parse_argb("#12ABEF"), Some(c), "six digits are opaque");
        assert_eq!(written(c), "#12ABEF");
        assert_eq!(parse_argb("#80FF000"), None);
        assert_eq!(parse_hex("#12abef"), Some(c));
        assert_eq!(parse_hex("12ABEF"), Some(c));
        assert_eq!(parse_hex("#12AB"), None);
    }
}
