---
name: omarchy-kit
description: Build or restyle an egui / eframe UI in the user's own Rust apps with omarchy-kit — the Omarchy theme's colours (with defaults off Omarchy) and the shared buttons, text fields, tag fields, checkboxes, number fields, toggle, swatches, dialogs, status line, item marks and command palette drawn from them. Use whenever a Rust desktop UI gets a button, a text field, a search box, a list of tags or email addresses, a checkbox, a number, a colour, a dialog or a status line, a toggle or tabs, a toolbar, a sidebar, a list row, a card, a selection or keyboard highlight, a corner radius, a command palette or a list of keyboard shortcuts, or a colour; when a UI should "look consistent", "follow the theme" or "match Post"; and when starting a new egui app.
---

# omarchy-kit

One crate for every egui app of the user's: the Omarchy theme turned into
egui's visuals, and a small set of widgets drawn from those visuals and nothing
else. Apps that use it look like one product and follow a theme switch live.
Off Omarchy it falls back to a neutral dark theme.

## Set-up

`Cargo.toml` — by git, pinned to a `rev`, so kit changes are taken
deliberately:

```toml
omarchy-kit = { git = "https://github.com/edimoldovan/omarchy-kit.git", rev = "<commit>" }
```

The app's egui must be the kit's version (0.33.3; eframe 0.33.3).

The theme, once at start and again whenever it changes:

```rust
use omarchy_kit::theme::{self, ThemeWatcher};

// when the app starts
ctx.set_visuals(theme::visuals(&theme::load()));

// in update(), every frame — it touches the disk at most once a second
if let Some(palette) = self.theme_watcher.changed(ctx) {
    ctx.set_visuals(theme::visuals(&palette));
}
```

`fonts::install(ctx)`, once at start, adds the system's symbol fonts behind
egui's own, so symbols such as ▾ and ✕ do not render as empty boxes.
`theme::Palette` is `{ bg, fg, accent }`; `theme::mix(a, b, t)` gives every
in-between shade. Colours come from these and from `ui.visuals()` only. The one
exception is a product's own logo, which keeps its own colours.

## Widgets

```rust
use omarchy_kit::{Button, Checkbox, Choice, ColorButton, CommandPalette, Dialog, Dropdown, Item, NumberField, Size, Slider, StatusLine, Swatches, TagField, TextField, Toggle, field};

ui.add(Button::new("Send").icon(glyph("send")).primary());
ui.add(Button::new("Discard"));                                  // secondary is the default
ui.add(Button::new("Reply").icon(glyph("reply")).hint("R").quiet());
ui.add(Button::icon_only(glyph("back")).quiet()).on_hover_text("Back");
ui.add(Button::new("Remove").danger());
ui.add(Button::new("Compose").primary().large().width(ui.available_width()));
ui.add(Button::icon_only(glyph("gear")).quiet().selected(on_settings));

let search = ui.add(TextField::new(&mut query).icon(glyph("find")).hint("Search").key("/"));

// A list entered one item at a time: the items as tags, the typing after
// them. Both Vec<String> and String are the app's; commit what is still
// typed before the list is used.
let to = ui.add(TagField::new(&mut addresses, &mut typing).accept(is_address).separators(" ,;").hint("name@example.com"));
TagField::commit(&mut addresses, &mut typing, is_address);

ui.add(Checkbox::new(&mut match_case, "Match case"));
ui.add(NumberField::new(&mut size, 6.0..=96.0).suffix("pt"));
ui.add(ColorButton::new(&mut fill, glyph("fill-color")).clear("No fill").recent(&recent));
ui.add(Swatches::new(&mut colour).clear("Automatic"));

// A card over the window (modal), or beside the work with .modeless().
let shown = Dialog::new("Go to").show(ctx, |ui| { /* … */ });
if shown.closed { /* … */ }

// The line along the bottom of every window.
StatusLine::new("status").place(where_).message(&status).end(|ui| { /* … */ }).show(ctx);

let mut chosen = usize::from(unread_only);
if ui.add(Toggle::new(&mut chosen, &["All", "Unread"])).changed() { /* … */ }

// A choice among more than fit together: the chosen one with a chevron,
// the list below it when clicked. Out of range shows the placeholder.
let mut mode = 2usize;
if ui.add(Dropdown::new(&mut mode, &["normal", "multiply", "screen"]).width(160.0)).changed() { /* … */ }

// An amount between two ends: its label above, a knob on a track and a
// field at the end to type into; Up and Down nudge it while the field has
// the keyboard; a double click on the track puts the default back.
if ui.add(Slider::new(&mut opacity, 0.0..=100.0).label("Opacity").default(100.0).decimals(0).suffix("%")).changed() { /* … */ }

// Anything else that should look like a field: chips, a multi-line editor.
field(ui, Size::Regular, |ui| {
    let edit = ui.add(TextEdit::multiline(&mut body).frame(false));
    (edit.has_focus(), edit)
});

// A row, a sidebar entry, a card: its marks, painted before its contents.
let (rect, response) = ui.allocate_exact_size(size, Sense::click());
Item { open, keyed, hovered: response.hovered(), size: Size::Regular }.paint(ui, rect);

// Every action, found by typing. The app keeps one in its state, toggles it on
// its key, and hands it every frame the choices that apply where it is.
let choices = vec![Choice { label: "Reply".into(), key: Some("R".into()) } /* , … */];
if let Some(i) = self.command_palette.show(ctx, "Ctrl+K", &choices) { /* run choices[i] */ }
```

- `Button`: `.primary()`, `.quiet()`, `.danger()` (secondary when none);
  `.small()`, `.large()` (regular when none); `.icon(..)`, `.hint(key)`,
  `.selected(bool)`, `.width(w)`, `.bar(Some(colour))` (a colour bar along
  its bottom edge: a sheet tab's colour). It is a `Widget`: `ui.add` /
  `ui.add_enabled` and the usual `Response`.
- `TextField`: `.hint`, `.icon`, `.key`, `.small()`, `.width`, `.id_salt`. Its
  response is the text's own — `changed`, `has_focus`, `lost_focus`.
- `Toggle`: a few choices as one control; `changed()` when the choice moved.
- `Dropdown`: `.placeholder` (shown while `selected` is out of range),
  `.small()`, `.width`, `.id_salt`; `changed()` when the choice moved. The
  list is as wide as the field, its lines marked like items, the chosen one
  filled.
- `Slider`: a track with a knob and a field at its end with the value;
  `.label` puts the name on a line above. Dragging the track sets it and
  hands the keyboard to the field; there, Up and Down move a step, Shift
  ten, Ctrl a hundred, Enter or leaving takes what was typed within the
  range, Esc puts back what it was. `.default` (a double click on the
  track puts it back), `.decimals` (shown, and the step), `.suffix` (after
  the value in its field), `.small()`, `.width`, `.id_salt`; `changed()`
  whenever the value moved. The fill runs from zero when the range crosses
  zero, else from the start.
- `TagField`: tags in a field with the typing after them, a line tall until
  the tags need more. What is typed becomes a tag when a separator ends it,
  or on Enter, Tab or leaving the field, whenever `.accept(fn)` takes it
  (anything when unset); what it does not take stays typed. Backspace with
  nothing typed takes the last tag back, each tag's × takes it off, the same
  tag twice in any case is one. `.separators(chars)` (`",;"` when unset),
  `.hint`, `.small()`, `.width`, `.id_salt`; `TagField::commit(tags, typing,
  accept)` takes what is still typed. Its response is the typed text's,
  `changed` also when a tag came or went. A tag's text starts where a
  field's text does, so it lines up with the fields around it.
- `Checkbox::new(&mut bool, label)`: on or off; a click or Space flips it;
  `.small()`, `.size`.
- `NumberField::new(&mut f32, range)`: one number in the kit's field; Up and
  Down step it (Shift ten, Ctrl a hundred), Enter or leaving takes what was
  typed within the range, Esc puts it back; `.decimals` (0 when unset),
  `.suffix`, `.small()`, `.width`, `.id_salt`.
- `Swatches::new(&mut Option<Color32>)`: the standard colours as a grid, the
  chosen one outlined; `.clear(label)` for no colour, `.recent(&colours)`, a
  `#RRGGBB` field under them. `ColorButton::new(&mut Option<Color32>, icon)`
  is a quiet button with the colour under its icon, opening the swatches.
  `standard_colours()`, `hex(colour)`.
- `Dialog::new(title).show(ctx, |ui| R) -> DialogResponse { inner, closed }`:
  a card with the title and a ×; modal by default (dimmed window behind, Esc,
  × or a click beside it close it); `.modeless()` beside the work, moved by
  dragging, closed by the × only; `.resizable(size)`; `.width` (380 when
  unset); `.id_salt`.
- `StatusLine::new(id).place(..).message(..).after(|ui| ..).end(|ui| ..).show(ctx)`:
  the line along the bottom; `after` follows the message, `end` sits at the
  far end, right to left.
- `field(ui, size, |ui| (focused, R)) -> R`: the field's fill, corner and focus
  outline around whatever the closure draws.
- `Item { open, keyed, hovered, size }.paint(ui, rect)`: how a selectable thing
  is marked — filled while open (on screen beside its list), a quiet fill
  under the pointer, the accent outline where the keyboard is — fill and
  outline in the size's one corner.
- `CommandPalette`: `.open()`, `.close()`, `.toggle()`, `.is_open()`, and
  `.show(ctx, key, &[Choice]) -> Option<usize>` — drawn over the window, at
  the top, while open: a field with `key` at its end, the choices that match
  what is typed (each with its own key at the end of its line), ↑↓ to walk
  them, Enter or a click to run one, Esc or a click beside it to close. It
  closes itself when one is run. `Choice { label, key: Option<String> }`.
- Icons: `icons::draw(painter, rect, name, colour)` draws one of the shared
  glyphs (trash, open, find, save, undo, redo, …) centred in `rect`; an app
  draws its own glyphs with the same `icons::Pen`, so every icon has one
  stroke weight and size. An element takes any `Fn(&Painter, Rect, Color32)`
  as its icon.
- `omarchy_kit::on(fill)` is the text colour that reads on a fill.

## How to use them

- One primary button per screen: the thing the screen is for — Send, Compose.
- Secondary for every other button: Discard, Attach, Cancel, Add account.
- Quiet for tools on a bar, back arrows and section toggles (Cc / Bcc); with
  `.selected(true)` for the one whose screen is on screen.
- Danger only for what cannot be undone, and only once it has been asked about.
- Sizes: regular by default; small inside lists, chips and dense bars; large
  for the one control a screen leads with.
- A list entered one item at a time — addresses, labels — is a `TagField`;
  never a `TextField` with commas in it, never hand-drawn chips.
- Something on or off is a `Checkbox`; a number is a `NumberField` (a `Slider`
  when it is an amount between two ends); a colour is `Swatches`, or a
  `ColorButton` on a bar.
- Anything that asks for something is a `Dialog` — never egui's `Window`.
- Every window has a `StatusLine` along its bottom.
- Every text input is a `TextField`, or sits in `field`. A choice of a few
  (All / Unread) is a `Toggle`, not tabs or selectable labels.
- Every selectable row, entry or card is marked with `Item`: what is open is
  filled; the keyboard's place is an outline, shown only in the column the
  keyboard is in. The keyboard's place is never a fill.
- Corners come from `Size::radius()`, never a number: rows, entries and cards
  take `Size::Regular`; what sits inside one — chips, badges, avatars, a mark
  under a small button — `Size::Small`. An outline drawn outside a shape adds
  its distance to the shape's corner (2 points out from an 8 is a 10), so the
  two stay concentric.
- Every action an app has is in its `CommandPalette`: the commands that do
  something where the app is, each with the key it is on, then what is on
  screen to act on (a folder to go to, a file to open). Its key is
  `Ctrl+K`, and it opens even while a text field has the keyboard — consume
  it before the widgets run, as egui gives text fields every key otherwise.
  The way to it is a quiet keyboard button beside the settings gear, whose
  tooltip names the key; the key is also at the end of the palette's field.
- Tooltips name the key a control is on, read from the app's keymap rather
  than typed into the text: `"Settings  (Home)"`.
- A choice among more than a `Toggle` holds is a `Dropdown`; never a
  `Button` with a popup of buttons, and never egui's `ComboBox`.
- No egui `Button`, `small_button`, `selectable_label`, `checkbox`, `radio`, `DragValue`, `Window`, `ComboBox` or framed
  `TextEdit` in an app that uses the kit, and no hand-painted buttons. What the
  kit lacks gets added to the kit, with a test, so
  every app gets it.
