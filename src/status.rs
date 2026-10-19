//! The status line: along the bottom of a window, where the app is and what
//! just happened.

use egui::{Align, Context, Id, Layout, RichText, TopBottomPanel, Ui};

/// What goes on the line besides the place and the message.
type Part<'a> = Box<dyn FnOnce(&mut Ui) + 'a>;

/// The line along the bottom of a window: where the app is, quietly, then
/// what just happened, then whatever the app adds after it; and at the far
/// end, what the app keeps there — a count, a zoom.
#[must_use = "a status line is drawn with line.show(ctx)"]
pub struct StatusLine<'a> {
    id: Id,
    place: String,
    message: String,
    after: Option<Part<'a>>,
    end: Option<Part<'a>>,
}

impl<'a> StatusLine<'a> {
    pub fn new(id_salt: impl std::hash::Hash) -> StatusLine<'a> {
        StatusLine { id: Id::new(id_salt), place: String::new(), message: String::new(), after: None, end: None }
    }

    /// Where the app is: an account and its count, a workbook and its sheet.
    pub fn place(mut self, place: impl Into<String>) -> StatusLine<'a> {
        self.place = place.into();
        self
    }

    /// What just happened, or what went wrong.
    pub fn message(mut self, message: impl Into<String>) -> StatusLine<'a> {
        self.message = message.into();
        self
    }

    /// What follows the message: a button that puts something right.
    pub fn after(mut self, after: impl FnOnce(&mut Ui) + 'a) -> StatusLine<'a> {
        self.after = Some(Box::new(after));
        self
    }

    /// What sits at the far end, drawn right to left.
    pub fn end(mut self, end: impl FnOnce(&mut Ui) + 'a) -> StatusLine<'a> {
        self.end = Some(Box::new(end));
        self
    }

    pub fn show(self, ctx: &Context) {
        let StatusLine { id, place, message, after, end } = self;
        TopBottomPanel::bottom(id).show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new(place).weak());
                if !message.is_empty() {
                    ui.separator();
                    ui.label(message);
                }
                if let Some(after) = after {
                    ui.separator();
                    after(ui);
                }
                if let Some(end) = end {
                    ui.with_layout(Layout::right_to_left(Align::Center), end);
                }
            });
        });
    }
}
