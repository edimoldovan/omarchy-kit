//! The tag field: what has been entered, as tags, with the text being typed
//! after them — the addresses an email goes to, the labels on a file.

use egui::{
    FontId, Id, Key, Margin, Pos2, Rect, Response, RichText, Sense, Stroke, TextEdit, Ui, Vec2,
    Widget,
};

use crate::field::framed;
use crate::{Ink, Size, theme};

/// Room between the field's edge and the tags in it.
const INSET: f32 = 4.0;

/// The least room the text being typed is given beside the tags; with less,
/// it starts a line of its own.
const TYPING: f32 = 120.0;

/// Tags in a field, with the text being typed after them, one line tall
/// until they need more. What is typed becomes a tag when one of the
/// separators ends it, and on Enter, Tab or leaving the field — whenever
/// `accept` takes it; what it does not take stays typed, to be put right.
/// Backspace with nothing typed takes the last tag back, and each tag's ×
/// takes it off. The same tag twice, in any case, is one.
#[must_use = "a tag field is drawn with ui.add(field)"]
pub struct TagField<'a> {
    tags: &'a mut Vec<String>,
    typing: &'a mut String,
    accept: Box<dyn Fn(&str) -> bool + 'a>,
    separators: Vec<char>,
    hint: String,
    size: Size,
    width: Option<f32>,
    id: Option<Id>,
}

impl<'a> TagField<'a> {
    pub fn new(tags: &'a mut Vec<String>, typing: &'a mut String) -> TagField<'a> {
        TagField {
            tags,
            typing,
            accept: Box::new(|_| true),
            separators: vec![',', ';'],
            hint: String::new(),
            size: Size::default(),
            width: None,
            id: None,
        }
    }

    /// What may become a tag, asked of what was typed, trimmed; anything
    /// when unset.
    pub fn accept(mut self, accept: impl Fn(&str) -> bool + 'a) -> TagField<'a> {
        self.accept = Box::new(accept);
        self
    }

    /// The characters that end a tag as it is typed; a comma and a
    /// semicolon when unset.
    pub fn separators(mut self, separators: &str) -> TagField<'a> {
        self.separators = separators.chars().collect();
        self
    }

    /// What it says while it holds nothing.
    pub fn hint(mut self, hint: impl Into<String>) -> TagField<'a> {
        self.hint = hint.into();
        self
    }

    pub fn size(mut self, size: Size) -> TagField<'a> {
        self.size = size;
        self
    }

    pub fn small(self) -> TagField<'a> {
        self.size(Size::Small)
    }

    /// As wide as `width`, frame and all, rather than the room there is.
    pub fn width(mut self, width: f32) -> TagField<'a> {
        self.width = Some(width);
        self
    }

    /// Where egui keeps its state, when two fields would otherwise share it.
    pub fn id_salt(mut self, salt: impl std::hash::Hash) -> TagField<'a> {
        self.id = Some(Id::new(salt));
        self
    }

    /// What is being typed made a tag, when `accept` takes it — what the
    /// field does on Enter, Tab and leaving it, for an app to do before it
    /// uses the tags: Send, say. True when a tag was added.
    pub fn commit(tags: &mut Vec<String>, typing: &mut String, accept: impl Fn(&str) -> bool) -> bool {
        let typed = typing.trim().to_string();
        if typed.is_empty() || !accept(&typed) {
            return false;
        }
        typing.clear();
        add(tags, &typed)
    }
}

impl Widget for TagField<'_> {
    /// The response is the typed text's, so `has_focus`, `lost_focus` and
    /// `request_focus` work on the field; `changed` also when a tag came or
    /// went.
    fn ui(self, ui: &mut Ui) -> Response {
        let TagField { tags, typing, accept, separators, hint, size, width, id } = self;
        let ink = Ink::of(ui);
        let tall = tag_height(size);
        let inner = width.unwrap_or_else(|| ui.available_width()) - 2.0 * (INSET + 1.0);
        let base = id.unwrap_or_else(|| ui.next_auto_id());
        let was_empty = typing.is_empty();
        let mut dropped = None;

        let mut edit = framed(ui, size, Margin::same(INSET as i8), tall, |ui| {
            ui.set_width(inner);
            let edit = ui
                .horizontal_wrapped(|ui| {
                    ui.set_row_height(tall);
                    ui.spacing_mut().item_spacing = Vec2::splat(INSET);
                    for (i, text) in tags.iter().enumerate() {
                        if tag(ui, text, size, base.with(i)) {
                            dropped = Some(i);
                        }
                    }
                    // With no tags, the text starts where a field's text does.
                    if tags.is_empty() {
                        ui.add_space(size.pad() - INSET);
                    }
                    if ui.available_size_before_wrap().x < TYPING {
                        ui.end_row();
                    }
                    let shown = if tags.is_empty() { hint.as_str() } else { "" };
                    let mut edit = TextEdit::singleline(typing)
                        .frame(false)
                        .margin(Margin::ZERO)
                        .font(FontId::proportional(size.text()))
                        .hint_text(RichText::new(shown).color(ink.weak))
                        .desired_width(ui.available_size_before_wrap().x);
                    if let Some(id) = id {
                        edit = edit.id(id);
                    }
                    ui.add(edit)
                })
                .inner;
            (edit.has_focus(), edit)
        });

        if edit.has_focus() && was_empty && ui.input(|input| input.key_pressed(Key::Backspace)) {
            dropped = tags.len().checked_sub(1);
        }
        let mut changed = false;
        if let Some(i) = dropped {
            tags.remove(i);
            changed = true;
        }
        changed |= split(tags, typing, &accept, &separators);
        if edit.lost_focus() {
            changed |= TagField::commit(tags, typing, &accept);
        }
        if changed {
            edit.mark_changed();
        }
        edit
    }
}

/// How tall a tag is in a field of `size`: the field's line, less the inset
/// all round.
fn tag_height(size: Size) -> f32 {
    size.height() - 2.0 * (INSET + 1.0)
}

/// One tag: its text, lined up with a field's text, and a × at its end that
/// takes it off. True when the × was clicked.
fn tag(ui: &mut Ui, text: &str, size: Size, id: Id) -> bool {
    let ink = Ink::of(ui);
    let tall = tag_height(size);
    let galley = ui.painter().layout_no_wrap(text.to_string(), FontId::proportional(size.text() - 1.5), ink.fg);
    let pad = size.pad() - INSET;
    let (rect, _) = ui.allocate_exact_size(Vec2::new(pad + galley.size().x + tall, tall), Sense::hover());
    let cross = Rect::from_min_size(Pos2::new(rect.right() - tall, rect.top()), Vec2::splat(tall));
    let x = ui.interact(cross, id, Sense::click()).on_hover_text("Take it off");

    let painter = ui.painter();
    painter.rect_filled(rect, Size::Small.radius(), ink.selected);
    painter.galley(Pos2::new(rect.left() + pad, rect.center().y - galley.size().y / 2.0), galley, ink.fg);
    let c = cross.center();
    if x.hovered() {
        painter.circle_filled(c, tall * 0.34, theme::mix(ink.selected, ink.fg, 0.18));
    }
    let arm = tall * 0.15;
    let mark = if x.hovered() { ink.fg } else { theme::mix(ink.selected, ink.fg, 0.7) };
    let stroke = Stroke::new(1.3_f32, mark);
    painter.line_segment([c + Vec2::new(-arm, -arm), c + Vec2::new(arm, arm)], stroke);
    painter.line_segment([c + Vec2::new(-arm, arm), c + Vec2::new(arm, -arm)], stroke);
    x.clicked()
}

/// Puts a tag on, once: the same tag in another case is the same tag. True
/// when it was not there yet.
fn add(tags: &mut Vec<String>, tag: &str) -> bool {
    if tags.iter().any(|t| t.eq_ignore_ascii_case(tag)) {
        return false;
    }
    tags.push(tag.to_string());
    true
}

/// Every tag a separator has finished moves out of what is being typed. The
/// unfinished one stays, and so does a finished one `accept` does not take.
/// True when a tag was added.
fn split(tags: &mut Vec<String>, typing: &mut String, accept: &dyn Fn(&str) -> bool, separators: &[char]) -> bool {
    let Some(end) = typing.rfind(separators) else {
        return false;
    };
    let after = end + typing[end..].chars().next().map_or(1, char::len_utf8);
    let rest = typing[after..].trim_start().to_string();
    let mut kept: Vec<String> = Vec::new();
    let mut added = false;
    for word in typing[..end].split(separators).map(str::trim).filter(|w| !w.is_empty()) {
        if accept(word) {
            added |= add(tags, word);
        } else {
            kept.push(word.to_string());
        }
    }
    if !rest.is_empty() {
        kept.push(rest);
    }
    *typing = kept.join(" ");
    added
}

#[cfg(test)]
mod tests {
    use super::*;
    use egui::{Context, Event, Modifiers, RawInput};

    fn address(text: &str) -> bool {
        text.contains('@') && !text.contains(' ')
    }

    /// A separator finishes a tag; the one still being typed stays, and so
    /// does a word `accept` does not take. Committing adds the last one —
    /// once, in any case.
    #[test]
    fn typing_becomes_tags_where_a_separator_ends_them() {
        let (mut tags, mut typing) = (Vec::new(), "ale@acme.com cm@ey.com, fre".to_string());
        assert!(split(&mut tags, &mut typing, &address, &[' ', ',', ';']));
        assert_eq!(tags, vec!["ale@acme.com", "cm@ey.com"]);
        assert_eq!(typing, "fre");

        typing = "fredrik ".to_string();
        assert!(!split(&mut tags, &mut typing, &address, &[' ', ',', ';']));
        assert_eq!(typing, "fredrik", "not taken: it stays, to be put right");

        typing = "ALE@acme.com".to_string();
        assert!(!TagField::commit(&mut tags, &mut typing, address), "the same tag twice is one");
        assert_eq!(typing, "");
        assert_eq!(tags.len(), 2);

        // Without a space among the separators, a space is part of a tag.
        let (mut tags, mut typing) = (Vec::new(), "New York, Oslo".to_string());
        split(&mut tags, &mut typing, &|_| true, &[',', ';']);
        assert_eq!((tags, typing), (vec!["New York".to_string()], "Oslo".to_string()));
    }

    /// One frame of a window holding a tag field, with these events in it.
    fn frame(ctx: &Context, tags: &mut Vec<String>, typing: &mut String, events: Vec<Event>) {
        let input = RawInput { events, ..RawInput::default() };
        let _ = ctx.run(input, |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                ui.add(TagField::new(tags, typing).accept(address).id_salt("to"));
            });
        });
    }

    /// Typed into, the field makes a tag where a comma ends one, and
    /// Backspace with nothing typed takes it back.
    #[test]
    fn the_field_makes_and_takes_back_tags() {
        let ctx = Context::default();
        let (mut tags, mut typing) = (Vec::<String>::new(), String::new());
        ctx.memory_mut(|memory| memory.request_focus(Id::new("to")));
        frame(&ctx, &mut tags, &mut typing, vec![Event::Text("ale@acme.com, cm".into())]);
        assert_eq!(tags, vec!["ale@acme.com"]);
        assert_eq!(typing, "cm", "the one still being typed stays");
        typing.clear();
        frame(&ctx, &mut tags, &mut typing, vec![]);
        let backspace = Event::Key {
            key: Key::Backspace,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: Modifiers::NONE,
        };
        frame(&ctx, &mut tags, &mut typing, vec![backspace]);
        assert!(tags.is_empty(), "Backspace on nothing typed takes the tag back");
    }
}
