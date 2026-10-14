//! The toggle: a choice of a few, shown together.

use egui::{Align2, FontId, Pos2, Rect, Response, Sense, Ui, Vec2, Widget};

use crate::{Ink, Size};

/// A choice of a few, shown together — All / Unread: the one chosen filled,
/// the others quiet. A click changes `selected` in place, and the response
/// says it changed.
#[must_use = "a toggle is drawn with ui.add(toggle)"]
pub struct Toggle<'a> {
    selected: &'a mut usize,
    options: &'a [&'a str],
    size: Size,
}

impl<'a> Toggle<'a> {
    pub fn new(selected: &'a mut usize, options: &'a [&'a str]) -> Toggle<'a> {
        Toggle {
            selected,
            options,
            size: Size::Small,
        }
    }

    pub fn size(mut self, size: Size) -> Toggle<'a> {
        self.size = size;
        self
    }
}

impl Widget for Toggle<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        let size = self.size;
        let ink = Ink::of(ui);
        let font = FontId::proportional(size.text());
        let inset = 3.0;
        let widths: Vec<f32> = self
            .options
            .iter()
            .map(|option| {
                ui.painter()
                    .layout_no_wrap(option.to_string(), font.clone(), ink.fg)
                    .size()
                    .x
                    + 2.0 * size.pad()
            })
            .collect();
        let total = widths.iter().sum::<f32>() + 2.0 * inset;
        let (rect, mut response) =
            ui.allocate_exact_size(Vec2::new(total, size.height()), Sense::click());
        let painter = ui.painter();
        painter.rect_filled(rect, size.radius(), ink.shade(0.10));

        let mut x = rect.min.x + inset;
        let mut chose = None;
        for (i, (option, width)) in self.options.iter().zip(&widths).enumerate() {
            let segment = Rect::from_min_size(
                Pos2::new(x, rect.min.y + inset),
                Vec2::new(*width, size.height() - 2.0 * inset),
            );
            let chosen = i == *self.selected;
            let over = response.hover_pos().is_some_and(|at| segment.contains(at));
            let corner = size.radius() - inset;
            if chosen {
                painter.rect_filled(segment, corner, ink.selected);
            } else if over {
                painter.rect_filled(segment, corner, ink.shade(0.17));
            }
            painter.text(
                segment.center(),
                Align2::CENTER_CENTER,
                option,
                font.clone(),
                if chosen { ink.fg } else { ink.weak },
            );
            if over && response.clicked() {
                chose = Some(i);
            }
            x += width;
        }
        if let Some(i) = chose
            && i != *self.selected
        {
            *self.selected = i;
            response.mark_changed();
        }
        response
    }
}
