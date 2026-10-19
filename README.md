# omarchy-kit

The elements an egui app is built from, drawn in the colours of the Omarchy
theme. The theme is read from the system and turned into egui's
visuals; every element is drawn from those visuals and nothing else. Apps built
with it look like one product and follow a theme switch live. Off Omarchy they
fall back to a neutral dark theme.

## The elements

| Element | What it is | Use it for |
|---|---|---|
| [`Button`](#button) | A button in four kinds and three sizes, with an optional icon and key | Every button |
| [`TextField`](#textfield) | One line of text in a field | Every one-line text input |
| [`TagField`](#tagfield) | Tags in a field, with the text being typed after them | A list entered one item at a time: the addresses an email goes to, labels |
| [`Toggle`](#toggle) | A few choices, shown side by side as one control | A choice among two to four: All / Unread |
| [`Dropdown`](#dropdown) | The chosen one of a list, with the list under it | A choice among more than fit side by side |
| [`Slider`](#slider) | An amount on a track, with a field to type it into | A number between two ends: opacity, volume |
| [`field`](#field) | The field's frame around anything | An input the others do not cover: a multi-line editor |
| [`Item`](#item) | The marks of a selectable thing: open, under the pointer, where the keyboard is | List rows, sidebar entries, cards |
| [`CommandPalette`](#commandpalette) | Every action, found by typing, each with its key | An app's actions and the keys they are on |

Under all of them: [the theme](#the-theme), and [`Size`](#sizes-and-corners) —
one height, one corner and one type size, shared by every element so they
line up wherever they meet.

## Set-up

By git, pinned to a `rev`, so kit changes are taken deliberately:

```toml
omarchy-kit = { git = "https://github.com/edimoldovan/omarchy-kit.git", rev = "<commit>" }
```

The app's egui must be the kit's version: egui 0.33.3, eframe 0.33.3.

## The theme

Set once when the app starts, and again whenever the theme changes:

```rust
use omarchy_kit::theme::{self, ThemeWatcher};

// when the app starts
ctx.set_visuals(theme::visuals(&theme::load()));

// in update(), every frame — it touches the disk at most once a second
if let Some(palette) = self.theme_watcher.changed(ctx) {
    ctx.set_visuals(theme::visuals(&palette));
}
```

- `theme::Palette` is `{ bg, fg, accent }`; `theme::mix(a, b, t)` gives every
  shade between two of them.
- Colours come from the palette and from `ui.visuals()` only. The one exception
  is a product's own logo, which keeps its own colours.
- `omarchy_kit::on(fill)` is the text colour that reads on a fill.

## Sizes and corners

| `Size` | Height | Corner | Text | For |
|---|---|---|---|---|
| `Small` | 26 | 7 | 12.5 | Lists, dense bars, and what sits inside another element |
| `Regular` — the default | 32 | 8 | 14 | Everything else |
| `Large` | 42 | 10 | 15 | The one control a screen leads with, such as Compose |

- An element is regular unless told otherwise: `.small()`, `.large()`, or
  `.size(Size::…)`.
- Corners come from `Size::radius()`, never a number. Rows, entries, cards and
  fields take the regular corner; what sits inside one — tags, chips, badges,
  avatars — the small one.
- An outline drawn outside a shape adds its distance to the shape's corner: 2
  points out from an 8 is a 10, so the two stay concentric.

## Button

Every button is a `Button`. Its kind says how much it asks for attention:

| Kind | Looks | For |
|---|---|---|
| Primary — `.primary()` | Filled with the accent | The one thing a screen is for: Send, Compose. One per screen |
| Secondary — the default | A quiet fill | Every other button: Discard, Attach, Cancel, Add account |
| Quiet — `.quiet()` | No fill until hovered | Tools on a bar, back arrows, section toggles such as Cc / Bcc |
| Danger — `.danger()` | The error colour | What cannot be undone, once it has been asked about |

```rust
use omarchy_kit::Button;

ui.add(Button::new("Send").icon(glyph("send")).primary());
ui.add(Button::new("Discard"));
ui.add(Button::new("Reply").icon(glyph("reply")).hint("R").quiet());
ui.add(Button::icon_only(glyph("back")).quiet()).on_hover_text("Back");
ui.add(Button::new("Compose").primary().large().width(ui.available_width()));
ui.add(Button::icon_only(glyph("gear")).quiet().selected(on_settings));
```

- `.icon(glyph)` puts the app's icon before the label; `Button::icon_only`
  is the icon alone — give it a tooltip with its name.
- `.hint(key)` shows the key it is also on, quietly after the label.
- `.selected(true)` marks the quiet tool whose screen is showing.
- `.width(w)`, `.small()`, `.large()`, `.kind(Kind)`.
- It is a `Widget`: `ui.add`, `ui.add_enabled`, and the usual `Response`.

## TextField

Every one-line text input is a `TextField`: a quiet fill, and the accent
outline while it has the keyboard.

```rust
use omarchy_kit::TextField;

let search = ui.add(TextField::new(&mut query).icon(glyph("find")).hint("Search").key("/"));
```

- `.hint(text)` is what it says while empty.
- `.icon(glyph)` sits before the text; `.key(key)` names, at its end, the key
  that brings the keyboard to it.
- `.small()`, `.width(w)`, `.id_salt(salt)`.
- The response is the text's own: `changed`, `has_focus`, `lost_focus`,
  `request_focus`.

## TagField

A list entered one item at a time is a `TagField`: the items as tags, with the
text being typed after them, in one field that stays a line tall until the
tags need more.

```rust
use omarchy_kit::TagField;

// addresses: Vec<String>, typing: String — both the app's
let to = ui.add(
    TagField::new(&mut addresses, &mut typing)
        .accept(is_address)
        .separators(" ,;")
        .hint("name@example.com"),
);

// Before the list is used — Send, say — what is still typed is taken too.
TagField::commit(&mut addresses, &mut typing, is_address);
```

- What is typed becomes a tag when one of the separators ends it, or on Enter,
  Tab or leaving the field — whenever `accept` takes it. What it does not take
  stays typed, to be put right.
- Backspace with nothing typed takes the last tag back; each tag's × takes it
  off. The same tag twice, in any case, is one.
- A tag's text starts where a field's text does, so a tag field lines up with
  the text fields above and below it.
- `.accept(fn)` — anything when unset; `.separators(chars)` — a comma and a
  semicolon when unset; `.hint(text)`, `.small()`, `.width(w)`,
  `.id_salt(salt)`.
- The response is the typed text's: `has_focus`, `lost_focus` and
  `request_focus` work on the field, and `changed` also says a tag came or
  went.

## Toggle

A choice among two to four, shown side by side as one control.

```rust
use omarchy_kit::Toggle;

let mut chosen = usize::from(unread_only);
if ui.add(Toggle::new(&mut chosen, &["All", "Unread"])).changed() { /* … */ }
```

- `changed()` when the choice moved; `.size(Size::…)`.
- A choice of a few is a `Toggle`, not tabs or selectable labels.

## Dropdown

A choice among more than fit side by side: the chosen one with a chevron, and
the list under it when clicked.

```rust
use omarchy_kit::Dropdown;

let mut mode = 2usize;
if ui.add(Dropdown::new(&mut mode, &["normal", "multiply", "screen"]).width(160.0)).changed() { /* … */ }
```

- The list is as wide as the field; its lines are marked like items, the
  chosen one filled.
- `.placeholder(text)` is shown while the chosen index is out of range.
- `.small()`, `.width(w)`, `.id_salt(salt)`; `changed()` when the choice moved.
- Never a `Button` with a popup of buttons, and never egui's `ComboBox`.

## Slider

An amount between two ends: its label above, a knob on a track, and a field at
the end with the value, to type into.

```rust
use omarchy_kit::Slider;

if ui.add(Slider::new(&mut opacity, 0.0..=100.0).label("Opacity").default(100.0).decimals(0).suffix("%")).changed() { /* … */ }
```

- Dragging the track sets the value and hands the keyboard to the field.
- In the field, Up and Down move a step, Shift ten, Ctrl a hundred; Enter or
  leaving takes what was typed, within the range; Esc puts back what it was.
- A double click on the track puts back `.default(value)`.
- `.decimals(n)` sets what is shown and the step; `.suffix(text)` follows the
  value in its field.
- The fill runs from zero when the range crosses zero, else from the start.
- `.small()`, `.width(w)`, `.id_salt(salt)`; `changed()` whenever the value
  moved.

## field

The field's frame — its fill, its corner, the accent outline while what it
holds has the keyboard — around anything the other elements do not cover.

```rust
use omarchy_kit::{Size, field};

field(ui, Size::Regular, |ui| {
    let edit = ui.add(TextEdit::multiline(&mut body).frame(false));
    (edit.has_focus(), edit)
});
```

- The closure returns whether what it drew has the keyboard, and whatever else
  it likes, which `field` hands back.
- What goes inside draws no frame of its own: a `TextEdit` with
  `.frame(false)`.

## Item

How a selectable thing is marked — a row of a list, an entry of a sidebar, a
card — painted before its contents:

```rust
use omarchy_kit::{Item, Size};

let (rect, response) = ui.allocate_exact_size(size, Sense::click());
Item { open, keyed, hovered: response.hovered(), size: Size::Regular }.paint(ui, rect);
```

- `open`: what is on screen beside its list — filled.
- `hovered`: under the pointer — a quiet fill.
- `keyed`: where the keyboard is — the accent outline, shown only in the
  column the keyboard is in. The keyboard's place is never a fill.
- Fill and outline share the size's corner.

## CommandPalette

Every action an app has, found by typing: a field at the top of the window,
over the actions that match what is typed, each with its key at the end of
its line.

```rust
use omarchy_kit::{Choice, CommandPalette};

// in the app's state
command_palette: CommandPalette,

// on its key: Ctrl+K
self.command_palette.toggle();

// every frame: the choices that apply where the app is
let choices = vec![Choice { label: "Reply".into(), key: Some("R".into()) } /* , … */];
if let Some(i) = self.command_palette.show(ctx, "Ctrl+K", &choices) { /* run choices[i] */ }
```

- Up and Down walk the choices, Enter or a click runs one, Esc or a click
  beside it closes it. It closes itself when one is run.
- `show` draws it while it is open and returns the index of the choice run;
  `.open()`, `.close()`, `.toggle()`, `.is_open()`.
- It lists the commands that do something where the app is, each with the key
  it is on, then what is on screen to act on: a folder to go to, a file to
  open.
- Its key is Ctrl+K, and it opens even while a text field has the keyboard:
  consume the key before the widgets run, as egui hands text fields every key
  otherwise.
- The way to it is a quiet keyboard button beside the settings gear, whose
  tooltip names the key; the key is also at the end of the palette's field.

## Across the kit

- Icons are the app's own: anything `Fn(&Painter, Rect, Color32)`, drawn in
  the square and the colour the element hands it. The kit draws only its own
  marks, such as a tag's × and a dropdown's chevron.
- Tooltips name the key a control is on, read from the app's keymap rather
  than typed into the text: `"Settings  (Home)"`.
- No egui `Button`, `small_button`, `selectable_label`, `ComboBox` or framed
  `TextEdit` in an app that uses the kit, and no hand-painted controls. What
  the kit lacks is added here, with a test, so every app gets it.
