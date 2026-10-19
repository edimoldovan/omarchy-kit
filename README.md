# omarchy-kit

The elements an egui app is built from, drawn in the colours of the Omarchy
theme. The theme is read from the system and turned into egui's
visuals; every element is drawn from those visuals and nothing else. Apps built
with it look like one product and follow a theme switch live. Off Omarchy they
fall back to a neutral dark theme.

Post, Grido and omarchy-kit are made together:

- [**Post**](https://github.com/edimoldovan/post) — Gmail as a native window
  on Linux, a command in the terminal, and tools for Claude.
- [**Grido**](https://github.com/edimoldovan/grido) — a fast native
  spreadsheet for `.xlsx` files, and tools for Claude.
- [**omarchy-kit**](https://github.com/edimoldovan/omarchy-kit) — the egui
  elements both apps are built from, in the Omarchy theme's colours.

## The elements

| Element | What it is | Use it for |
|---|---|---|
| [`Button`](#button) | A button in four kinds and three sizes, with an optional icon and key | Every button |
| [`TextField`](#textfield) | One line of text in a field | Every one-line text input |
| [`TagField`](#tagfield) | Tags in a field, with the text being typed after them | A list entered one item at a time: the addresses an email goes to, labels |
| [`Checkbox`](#checkbox) | One thing on or off, with what it is beside it | A setting that is on or off: Match case |
| [`Switch`](#switch) | A pill with a sliding knob | A feature or a group of settings on or off, its settings kept |
| [`Section`](#section) | A titled group that folds, with a switch and a reset | An inspector's groups: Transform, Color, Audio |
| [`NumberField`](#numberfield) | One number, typed or nudged, within its range | A count or a size: decimal places, a font size |
| [`Toggle`](#toggle) | A few choices, shown side by side as one control | A choice among two to four: All / Unread |
| [`Dropdown`](#dropdown) | The chosen one of a list, with the list under it | A choice among more than fit side by side |
| [`Slider`](#slider) | An amount on a track, with a field to type it into | A number between two ends: opacity, volume |
| [`field`](#field) | The field's frame around anything | An input the others do not cover: a multi-line editor |
| [`Item`](#item) | The marks of a selectable thing: open, under the pointer, where the keyboard is | List rows, sidebar entries, cards |
| [`Swatches`](#swatches) · [`ColorButton`](#swatches) | The standard colours to pick from, and the button that opens them | A colour: a fill, a font colour |
| [`Dialog`](#dialog) | A card over the window, or a panel beside the work | Asking for something: Go to, Find, Format cells |
| [`StatusLine`](#statusline) | The line along the bottom of a window | Where the app is, and what just happened |
| [`CommandPalette`](#commandpalette) | Every action, found by typing, each with its key | An app's actions and the keys they are on |
| [`MenuBar`](#menus) · [`context_menu`](#menus) | Menus of lines, each with its key, from a bar or a right-click | A window's menu bar; what a right-click on something offers |

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

- `fonts::install(ctx)`, once when the app starts, puts the system's symbol
  fonts — Linux's, and Apple Symbols on macOS — and then egui's own Hack
  behind egui's fonts.
- Text draws only the symbols the bundled fonts cover, so they draw where no
  system font is installed:
  × ✔ … — – · • ← → ↑ ↓ ↖ ↗ ↘ ↙ ⇆ ⇄ ⇔ ⇥ ⇩ ⇧ ↵ ▾ ▴ ▸ ◂ ▲ ▼ ◀ ▶ ◆ ◇ ● ○ ■ □ ⌘ ⏸ ⏮ ⏭ ⏪ ⏩ ✂ ⊢ ⊣ ° ± µ ½ ∩ ∞ ≈ ≠ ≤ ≥ ✚ ➕.
  Not ✓, ✕, ⏎, ⌥, ⌫, ⏯ or ✎ — use ✔, × and ↵. The kit draws its own chevrons,
  crosses and ticks with lines.
- `theme::Palette` is `{ bg, fg, accent, ansi }`; `theme::mix(a, b, t)` gives
  every shade between two of them.
- `ansi` is the theme's sixteen terminal colours, 0–7 normal and 8–15 bright,
  read from `colors.toml` or alacritty's normal and bright colours; `red()`,
  `green()`, `yellow()`, `blue()`, `magenta()` and `cyan()` are the normal
  ones. They are for what an app colours by kind: clips, titles, meters,
  errors.
- An app with a look of its own off Omarchy passes it:
  `theme::load_or(&fallback)` and `ThemeWatcher::with_fallback(fallback)` use
  that palette when there is no theme, and its colours where a theme leaves
  one out.
- Colours come from the palette and from `ui.visuals()` only, with two
  exceptions: a product's own logo, and colours that show a colour itself —
  hue and saturation rails, red/green/blue curves and parade traces, colour
  wheels, vectorscope targets, the picture.
- `omarchy_kit::on(fill)` is the text colour that reads on a fill.

## Sizes and corners

| `Size` | Height | Corner | Text | For |
|---|---|---|---|---|
| `Mini` | 20 | 5 | 11.5 | Icon buttons on a bar denser still, such as a timeline's track headers |
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
ui.add(Button::icon_only(glyph("back")).quiet().tip("Back"));
ui.add(Button::new("Compose").primary().large().width(ui.available_width()));
ui.add(Button::icon_only(glyph("gear")).quiet().selected(on_settings));
```

- `.icon(glyph)` puts the app's icon before the label; `Button::icon_only`
  is the icon alone — give it `.tip(name)`.
- `.tip(text)` is what it says while the pointer is on it. On an icon-only
  button it is also the button's name, and `.hint(key)` goes in the tip —
  `"Settings  (Home)"` — rather than beside the icon.
- `.hint(key)` shows the key it is also on, quietly after the label.
- `.selected(true)` marks the quiet tool whose screen is showing.
- `.bar(Some(colour))` puts a bar of that colour along its bottom edge: the
  colour a sheet's tab was given.
- `.width(w)`, `.small()`, `.large()`, `.mini()` (20 points, for icon buttons
  on a dense bar such as a timeline's track headers), `.kind(Kind)`.
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

## Checkbox

One thing on or off: a box, filled with the accent and ticked while on, and
what it is beside it.

```rust
use omarchy_kit::Checkbox;

if ui.add(Checkbox::new(&mut match_case, "Match case")).changed() { /* … */ }
```

- A click anywhere on it, or Space while it has the keyboard, flips it;
  `changed()` when it did.
- `.small()`, `.size(Size::…)`.
- A single option on or off is a `Checkbox`; a feature or a group of settings
  on or off, its settings kept, is a `Switch`; a choice between two named
  things is a `Toggle`.

## Switch

A feature, or a whole group of settings, on or off — its settings kept
either way: a pill whose knob slides across, the accent while on.

```rust
use omarchy_kit::Switch;

if ui.add(Switch::new(&mut snapping).name("Snapping")).changed() { /* … */ }
```

- A click, or Space while it has the keyboard, flips it; `changed()` when it
  did.
- `.name(…)` is its name to a screen reader and a test; it shows no text of
  its own.
- `.small()` fits a header row; `.size(Size::…)`.

## Section

A titled group of settings that folds — an inspector's Transform, Color,
Audio — with a switch for the whole group and a reset at the header's far
end.

```rust
use omarchy_kit::Section;

let shown = Section::new("color", "Color").switch(&mut grade.on).reset(grade != Grade::default()).show(ui, |ui| {
    ui.add(Slider::new(&mut grade.saturation, 0.0..=2.0).label("Saturation"));
});
if shown.reset { grade = Grade::default(); }
```

- The chevron and the title fold it, by a click or Space; whether it is open
  is remembered per id. It starts open; `.default_open(false)` starts it
  folded.
- `.switch(&mut on)` turns the whole group on and off. While it is off, the
  body draws disabled, its values still shown and kept.
- `.reset(enabled)` is a quiet reset button, clickable while `enabled`.
- `show` returns `SectionResponse { inner, switched, reset, open }`; `inner`
  is what the body returned, `None` while folded.
- Names: the fold is the title, with its open state; the switch is
  "TITLE section"; the reset is "Reset TITLE".

## NumberField

One number in the kit's field, kept within its range: typed, nudged or
dragged.

```rust
use omarchy_kit::NumberField;

if ui.add(NumberField::new(&mut size, 6.0..=96.0).suffix("pt")).changed() { /* … */ }
ui.add(NumberField::new(&mut bitrate, 0.0..=f64::MAX).speed(50.0).suffix("kb/s").name("Bitrate"));
```

- Dragged sideways it moves by `.speed(s)` per point — a step per point when
  unset — with the sideways-resize pointer; a click without a drag puts the
  cursor in to type.
- While it has the keyboard, Up and Down move a step, Shift ten, Ctrl a
  hundred; Enter or leaving takes what was typed, within the range; Esc puts
  back what it was.
- It holds an `f32` or an `f64`; the range may be open at an end,
  `0.0..=f64::MAX`.
- `.decimals(n)` sets what is shown and the step — none when unset, a field
  of whole numbers; `.prefix(text)` comes before the number, `.suffix(text)`
  after it.
- `.name(…)`, `.small()`, `.width(w)`, `.id_salt(salt)`; `changed()`
  whenever the number moved; `adjust_ended()` — see [Adjustments](#adjustments).

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

// An inspector's rows: the label in a column, a small row each.
ui.add(Slider::new(&mut clip.speed, 0.1..=10.0).label("Speed").inline_label(74.0).logarithmic().suffix("×"));
ui.add(Slider::new(&mut grade.temperature, 2000.0..=10000.0).label("Temperature").inline_label(74.0).track(&COOL_TO_WARM));
```

- Dragging the track sets the value and hands the keyboard to the field.
- In the field, Up and Down move a step, Shift ten, Ctrl a hundred; Enter or
  leaving takes what was typed, within the range; Esc puts back what it was.
- A double click on the track puts back `.default(value)`.
- `.decimals(n)` sets what is shown and the step; `.suffix(text)` follows the
  value in its field.
- The fill runs from zero when the range crosses zero, else from the start.
- `.inline_label(width)` puts the label in a column `width` wide on the
  track's line, at the small size, so a stack of rows lines up.
- `.logarithmic()` maps the track logarithmically — speed, scale, a
  frequency; a range that does not start above zero stays linear.
- `.track(&[(f32, Color32)])` paints colour stops along the track in place of
  the fill, each a fraction of the track and its colour.
- `.keep_keyboard()`: a click or a drag on the track leaves the keyboard
  where it was rather than handing it to the field, for apps whose plain
  keys are shortcuts (an editor's Space, J, K, L); a click on the field
  still takes it for typing.
- It holds an `f32` or an `f64`.
- `.name(…)`, `.small()`, `.width(w)`, `.id_salt(salt)`; `changed()` whenever
  the value moved; `adjust_ended()` — see [Adjustments](#adjustments).

## Adjustments

A drag on a slider or a number field, and the nudges after it while its field
keeps the keyboard, are one adjustment: the one undo step an app records.

```rust
use omarchy_kit::Adjust;

let scale = ui.add(Slider::new(&mut clip.scale, 0.1..=10.0).label("Scale").logarithmic());
if scale.adjust_ended() {
    history.record("Scale");
}
```

- `adjusting()` is true from the first press on the track, the first drag on
  the field or the first keyboard nudge, until the pointer is up and the field
  has given up the keyboard.
- `adjust_ended()` is true on the one frame it ended. A value typed and
  entered is an adjustment of its own.

## Swatches

The standard colours — greys, then ten hues lightened, plain and darkened —
as a grid to pick from, with the colours used last under it and a field for
any colour as `#RRGGBB`. `ColorButton` is the quiet button that opens them,
with the colour it applies under its icon.

```rust
use omarchy_kit::{ColorButton, Swatches};

// On a bar: the button, and the swatches below it when clicked.
if ui.add(ColorButton::new(&mut fill, glyph("fill-color")).clear("No fill").recent(&recent)).changed() { /* … */ }

// In a dialog: the swatches themselves.
if ui.add(Swatches::new(&mut colour).clear("Automatic")).changed() { /* … */ }
```

- The colour is an `Option<Color32>`; `None` is no colour, the entry
  `.clear(label)` adds on top.
- The chosen colour is outlined; a pick closes a `ColorButton`'s swatches.
- `.recent(&colours)` is a row of the colours used last, newest first.
- `.alpha()`, on either, lets a colour be see-through: an opacity slider under
  the grid, which picking another colour keeps; see-through colours over a
  checkerboard; and the field taking and showing `#AARRGGBB` as well as
  `#RRGGBB`.
- `standard_colours()`, `hex(colour)` (`#RRGGBB`), `hex_argb(colour)` and
  `parse_argb(text)` (`#AARRGGBB`, opacity first) are there for an app that
  keeps colours of its own.

## Dialog

A card with a title and a × that closes it, around whatever the app puts in
it.

```rust
use omarchy_kit::Dialog;

let shown = Dialog::new("Go to").show(ctx, |ui| {
    ui.add(TextField::new(&mut reference).hint("B12"));
    ui.add(Button::new("Go").primary()).clicked()
});
if shown.closed { /* … */ }
if shown.inner == Some(true) { /* go */ }
```

- Modal unless told otherwise: the window behind it is dimmed and takes no
  clicks; Esc, the × or a click beside it closes it.
- `.modeless()` keeps it beside the work: the window behind keeps working,
  it moves by dragging, and only the × closes it. `.resizable(size)` is a
  modeless panel the user can resize, such as a chart.
- `.width(w)` — 380 points when unset; `.id_salt(salt)` when two share a
  title.
- `show` gives back what the closure returned, and `closed` when it was
  asked to close; the app decides what closing means.

## StatusLine

The line along the bottom of a window: where the app is, quietly, then what
just happened; at the far end, what the app keeps there.

```rust
use omarchy_kit::StatusLine;

StatusLine::new("status")
    .place("ed@acme.com — 3 unread")
    .message(&self.status)
    .end(|ui| { ui.label("Sum: 12"); })
    .show(ctx);
```

- `.after(|ui| …)` follows the message: a button that puts something right.
- `.end(|ui| …)` sits at the far end, drawn right to left: counts, a zoom.
- Every window of an app that uses the kit has one.

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
let choices = vec![
    Choice { label: "Reply".into(), key: Some("R".into()), keywords: "answer respond".into() },
    /* , … */
];
if let Some(i) = self.command_palette.show(ctx, "Ctrl+K", &choices) { /* run choices[i] */ }
```

- Up and Down walk the choices, Enter or a click runs one, Esc or a click
  beside it closes it. It closes itself when one is run.
- A choice matches when every word typed is in its label or its `keywords`,
  in any case; label matches come first. Keywords are never shown: other
  editors' words for the action, so "razor" finds Split.
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

## Menus

The menu bar along a window's top, and the menu a right-click opens: the
same lines in both, each with its label on the left and its key, dim, on the
right.

```rust
use omarchy_kit::{MenuBar, context_menu};

MenuBar::new().show(ui, |bar| {
    bar.menu("File", |m| {
        if m.item("Save").key("Ctrl+S").enabled(dirty).clicked() { /* … */ }
        m.separator();
        m.check("Snapping", &mut snapping);
        m.submenu("Export", |m| {
            if m.item("Movie…").key("Ctrl+M").clicked() { /* … */ }
        });
    });
    bar.menu("Edit", |m| { /* … */ });
});

let clip = ui.add(/* … */);
context_menu(&clip, |m| {
    if m.item("Split").key("S").clicked() { /* … */ }
});
```

- `m.item(label)` runs something: `.key(k)`, `.enabled(b)`, and
  `.clicked()`, which draws it and is true when it was run.
- `m.check(label, &mut bool)` has a tick while on, and turns it over when
  run. A menu with such a line keeps a column for the ticks.
- `m.separator()` divides groups of lines; `m.submenu(label, |m| …)` opens a
  menu of its own beside its line.
- A click on a menu's name opens it; with one open, moving onto another name
  opens that one. A click on a line runs it and closes every open menu; a
  press anywhere else closes them.
- The keyboard: Up and Down walk the lines, past separators and disabled
  lines; Enter runs one; Right and Left go into and out of a submenu, and
  across the bar; Esc closes them all.
- A line is named by its label; its key is its keyboard shortcut, apart.
- `menu_open(ctx)` says whether a menu was open when the last frame drew
  it: an app that reads its own shortcuts before drawing leaves the keys to
  the menu then, as the menu walks on Up and Down and closes on Esc.

## Names

Every element reports its role, its name and its value to AccessKit, so a
screen reader reads it and an [egui_kittest](https://crates.io/crates/egui_kittest)
test finds it by name:

```rust
harness.get_by_role_and_label(Role::Slider, "Brightness");
harness.get_by_role_and_label(Role::Button, "Settings").click();
```

| Element | Role | Name | Value |
|---|---|---|---|
| Button | Button | its label; its `.tip()` when icon-only | — |
| Checkbox | CheckBox | its label | on or off |
| Switch | Switch | `.name()` | on or off |
| Section | the fold a Button, the switch a Switch, the reset a Button | the title; `"TITLE section"`; `"Reset TITLE"` | open or folded |
| Toggle | a RadioButton per option | the option | the chosen one on |
| Slider | Slider | `.name()`, else its label | the amount |
| Dropdown | ComboBox; each line of the open list a Button | `.name()`, else the placeholder | the chosen option |
| TextField, TagField | TextInput | `.name()`, else the hint | the text |
| A TagField's tags | a Label, and a Button for its × | the tag; `"Take ale@acme.com off"` | — |
| NumberField | SpinButton | `.name()` | the number |
| Swatches | a ColorWell per swatch | its hex, `"#FF0000"` | the chosen one on |
| ColorButton | Button | its `.tip()` | the colour's hex |
| Dialog | Dialog, modal unless `.modeless()`; its × a Button | its title; `"Close"` | — |
| CommandPalette | its field a TextInput; each action a Button | `"Type what to do"`; the action's label | the keyboard's one on |
| MenuBar | a MenuBar; each menu's name a MenuItem | the menu's name | open or not |
| A menu's lines | MenuItem, or MenuItemCheckBox for `check` | the label; the key is its keyboard shortcut | checked or not; a submenu open or not |

- What shows no text of its own is named with `.name(…)`: a slider without a
  label, a field the app labels beside it. An icon-only button is named by
  its `.tip(…)`.
- The key in a tip is not part of the name: `.tip("Settings").hint("Home")`
  shows `"Settings  (Home)"` and is named `"Settings"`.
- The kit turns on egui's `accesskit` feature; eframe has it on by default.

## Across the kit

- Icons: `icons::draw(painter, rect, name, colour)` draws one of the shared
  glyphs, centred in `rect`:
  - editing: cut, copy, paste, undo, redo, clear, duplicate, rename, trash,
    save, open, new-doc, find, filter, sort, sort-asc, sort-desc, chart,
    zoom-in, zoom-out, zoom-reset, align-left, align-center, align-right,
    wrap, fill-color, bold, italic;
  - moving: back, prev, next, up, down, plus, close, check, gear, keyboard, help;
  - editing tools: select, razor, slip, slide, stretch, split, render,
    snapping;
  - tracks: lock, unlock, eye, eye-off, mute, solo, effects;
  - keyframes: keyframe, keyframe-empty, keyframe-prev, keyframe-next;
  - transport: play, pause, step-back, step-forward, start, end,
    shuttle-back, shuttle-forward;
  - source monitor: mark-in, mark-out, insert, overwrite.

  An app draws its own glyphs with the same `icons::Pen`, so every icon has
  one stroke weight and one size. An element takes any
  `Fn(&Painter, Rect, Color32)` as its icon.
- Tooltips name the key a control is on, read from the app's keymap rather
  than typed into the text: `"Settings  (Home)"` — on an icon-only button,
  `.tip("Settings").hint(key)`.
- No egui `Button`, `small_button`, `selectable_label`, `ComboBox`,
  `menu_button`, `context_menu` or framed `TextEdit` in an app that uses the
  kit, and no hand-painted controls. What
  the kit lacks is added here, with a test, so every app gets it.
