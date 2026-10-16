//! The command palette: every action there is, found by typing.

use egui::{
    Align2, Context, FontId, Frame, Id, Key, Modal, Modifiers, Pos2, Rect, RichText, ScrollArea,
    Sense, Stroke, Ui, Vec2,
};

use crate::{Ink, Item, Size, TextField};

/// One action a palette offers: what it is called, and the key it is also on.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Choice {
    pub label: String,
    pub key: Option<String>,
}

/// A field over the actions that match what is typed in it, each with its
/// key at the end of its line. ↑↓ walk them, Enter runs one, Esc or a click
/// beside it closes it.
///
/// It keeps only what is typed and where the keyboard is; the app hands it
/// its choices every frame, so they are always what applies where the app
/// is.
#[derive(Debug, Default)]
pub struct CommandPalette {
    open: bool,
    query: String,
    at: usize,
}

/// Room between a palette's or a list's edge and what is in it.
pub(crate) const MARGIN: f32 = 8.0;

impl CommandPalette {
    pub fn is_open(&self) -> bool {
        self.open
    }

    /// Opens it empty, with the keyboard on the first choice.
    pub fn open(&mut self) {
        *self = CommandPalette { open: true, ..CommandPalette::default() };
    }

    pub fn close(&mut self) {
        self.open = false;
    }

    pub fn toggle(&mut self) {
        if self.open {
            self.close();
        } else {
            self.open();
        }
    }

    /// Draws it over the window while it is open, at the top, with `key` —
    /// the key that opens it — at the end of its field. Returns the index of
    /// the choice run this frame; running one closes it.
    pub fn show(&mut self, ctx: &Context, key: &str, choices: &[Choice]) -> Option<usize> {
        if !self.open {
            return None;
        }
        // Before the field is drawn: a one-line field gives the keyboard up
        // on Enter and Esc, and these are the palette's.
        let (down, up, enter, escape) = ctx.input_mut(|input| {
            (
                input.consume_key(Modifiers::NONE, Key::ArrowDown),
                input.consume_key(Modifiers::NONE, Key::ArrowUp),
                input.consume_key(Modifiers::NONE, Key::Enter),
                input.consume_key(Modifiers::NONE, Key::Escape),
            )
        });
        let screen = ctx.content_rect();
        let width = (screen.width() - 2.0 * (16.0 + MARGIN)).min(560.0);
        let list = (screen.height() * 0.6).min(10.0 * Size::Regular.height());
        let id = Id::new("omarchy-kit-command-palette");
        let area = Modal::default_area(id).anchor(Align2::CENTER_TOP, Vec2::new(0.0, screen.height().min(800.0) * 0.12));
        let mut chosen = None;

        let shown = Modal::new(id).area(area).frame(popup_frame(ctx)).show(ctx, |ui| {
            ui.set_width(width);
            let field = ui.add(TextField::new(&mut self.query).hint("Type what to do").key(key).id_salt(id));
            field.request_focus();
            if field.changed() {
                self.at = 0;
            }
            let hits = matching(&self.query, choices.iter().map(|c| c.label.as_str()));
            let last = hits.len().saturating_sub(1);
            if down {
                self.at = (self.at + 1).min(last);
            }
            if up {
                self.at = self.at.saturating_sub(1);
            }
            self.at = self.at.min(last);
            if enter {
                chosen = hits.get(self.at).copied();
            }
            ui.add_space(MARGIN);
            if hits.is_empty() {
                ui.label(RichText::new("Nothing matches.").color(Ink::of(ui).weak));
                return;
            }
            ScrollArea::vertical().max_height(list).show(ui, |ui| {
                ui.spacing_mut().item_spacing.y = 2.0;
                for (row, &i) in hits.iter().enumerate() {
                    let keyed = row == self.at;
                    if line(ui, &choices[i], keyed, keyed && (down || up)) {
                        chosen = Some(i);
                    }
                }
            });
        });
        if chosen.is_some() || escape || shown.should_close() {
            self.close();
        }
        chosen
    }
}

/// The card a palette or a dropdown's list sits on: the window's fill, a
/// hairline, and a corner that keeps the lines inside it concentric.
pub(crate) fn popup_frame(ctx: &Context) -> Frame {
    let style = ctx.style();
    let v = &style.visuals;
    Frame::popup(&style)
        .fill(v.panel_fill)
        .stroke(Stroke::new(1.0_f32, crate::theme::mix(v.panel_fill, v.text_color(), 0.14)))
        .corner_radius(Size::Regular.radius() + MARGIN)
        .inner_margin(MARGIN)
}

/// One choice: its name, its key at the end, marked like any item. True when
/// it was clicked.
fn line(ui: &mut Ui, choice: &Choice, keyed: bool, follow: bool) -> bool {
    let size = Size::Regular;
    let ink = Ink::of(ui);
    let (rect, response) =
        ui.allocate_exact_size(Vec2::new(ui.available_width(), size.height()), Sense::click());
    if follow {
        response.scroll_to_me(None);
    }
    Item { open: false, keyed, hovered: response.hovered(), size }.paint(ui, rect);
    let painter = ui.painter();
    let mut end = rect.right() - size.pad();
    if let Some(key) = &choice.key {
        let galley = painter.layout_no_wrap(key.clone(), FontId::proportional(size.text() - 2.0), ink.weak);
        end -= galley.size().x;
        painter.galley(Pos2::new(end, rect.center().y - galley.size().y / 2.0), galley, ink.weak);
        end -= 12.0;
    }
    let galley = painter.layout_no_wrap(choice.label.clone(), FontId::proportional(size.text()), ink.fg);
    let at = Pos2::new(rect.left() + size.pad(), rect.center().y - galley.size().y / 2.0);
    painter
        .with_clip_rect(Rect::from_min_max(rect.min, Pos2::new(end, rect.max.y)))
        .galley(at, galley, ink.fg);
    response.clicked()
}

/// The labels that match `query`, best first, as indexes into them. Every
/// word typed has to be in a label, in any case. Labels that start with the
/// first word come first, then those with a word starting with it, then the
/// rest — each group in the order given.
fn matching<'a>(query: &str, labels: impl Iterator<Item = &'a str>) -> Vec<usize> {
    let words: Vec<String> = query.split_whitespace().map(str::to_lowercase).collect();
    let mut hits: Vec<(u8, usize)> = labels
        .enumerate()
        .filter_map(|(i, label)| {
            let label = label.to_lowercase();
            if !words.iter().all(|w| label.contains(w.as_str())) {
                return None;
            }
            let rank = match words.first() {
                None => 0,
                Some(first) if label.starts_with(first.as_str()) => 0,
                Some(first) if label.split(|c: char| !c.is_alphanumeric()).any(|w| w.starts_with(first.as_str())) => 1,
                Some(_) => 2,
            };
            Some((rank, i))
        })
        .collect();
    hits.sort_by_key(|&(rank, _)| rank);
    hits.into_iter().map(|(_, i)| i).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use egui::{Event, RawInput};

    const LABELS: [&str; 5] = ["Reply", "Forward", "Discard the draft", "Go to Drafts", "Refresh"];

    /// Typing narrows the choices; what starts with it comes first, and
    /// every word typed has to be there.
    #[test]
    fn typing_narrows_and_orders_the_choices() {
        let found = |query: &str| matching(query, LABELS.iter().copied());
        assert_eq!(found(""), vec![0, 1, 2, 3, 4]);
        assert_eq!(found("re"), vec![0, 4]);
        assert_eq!(found("DRA"), vec![2, 3]);
        assert_eq!(found("go dra"), vec![3]);
        assert_eq!(found("ward"), vec![1]);
        assert!(found("teleport").is_empty());
    }

    /// Typed into, walked with the arrows and run with Enter, the palette
    /// answers with the choice and closes.
    #[test]
    fn enter_runs_the_choice_the_keyboard_is_on() {
        let ctx = Context::default();
        let choices: Vec<Choice> = LABELS
            .iter()
            .map(|label| Choice { label: label.to_string(), key: None })
            .collect();
        let mut palette = CommandPalette::default();
        palette.open();
        let mut frame = |events: Vec<Event>| {
            let mut chosen = None;
            let input = RawInput { events, ..RawInput::default() };
            let _ = ctx.run(input, |ctx| chosen = palette.show(ctx, "Ctrl+K", &choices));
            chosen
        };
        let key = |key| Event::Key {
            key,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: Modifiers::NONE,
        };
        assert_eq!(frame(vec![]), None, "the field takes the keyboard");
        assert_eq!(frame(vec![Event::Text("dra".into())]), None);
        assert_eq!(frame(vec![key(Key::ArrowDown)]), None);
        assert_eq!(frame(vec![key(Key::Enter)]), Some(3));
        assert!(!palette.is_open());
    }
}
