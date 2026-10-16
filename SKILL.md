---
name: omarchy-kit
description: Build or restyle an egui / eframe UI in the user's own Rust apps with omarchy-kit — the Omarchy theme's colours (with defaults off Omarchy) and the shared buttons, text fields, toggle, item marks and command palette drawn from them. Use whenever a Rust desktop UI gets a button, a text field, a search box, a toggle or tabs, a toolbar, a sidebar, a list row, a card, a selection or keyboard highlight, a corner radius, a command palette or a list of keyboard shortcuts, or a colour; when a UI should "look consistent", "follow the theme" or "match Post"; and when starting a new egui app.
---

# omarchy-kit

One crate for every egui app of the user's: the Omarchy theme turned into
egui's visuals, and a small set of widgets drawn from those visuals and nothing
else. Apps that use it look like one product and follow a theme switch live.
Off Omarchy it falls back to a neutral dark theme.

Used by Post (`crates/post-ui` in the fold repo) and by emiga; Post uses all
of it and is the worked example.

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

`theme::Palette` is `{ bg, fg, accent }`; `theme::mix(a, b, t)` gives every
in-between shade. Colours come from these and from `ui.visuals()` only. The one
exception is a product's own logo, which keeps its own colours.

## Widgets

```rust
use omarchy_kit::{Button, Choice, CommandPalette, Item, Size, Slider, TextField, Toggle, field};

ui.add(Button::new("Send").icon(glyph("send")).primary());
ui.add(Button::new("Discard"));                                  // secondary is the default
ui.add(Button::new("Reply").icon(glyph("reply")).hint("R").quiet());
ui.add(Button::icon_only(glyph("back")).quiet()).on_hover_text("Back");
ui.add(Button::new("Remove").danger());
ui.add(Button::new("Compose").primary().large().width(ui.available_width()));
ui.add(Button::icon_only(glyph("gear")).quiet().selected(on_settings));

let search = ui.add(TextField::new(&mut query).icon(glyph("find")).hint("Search").key("/"));

let mut chosen = usize::from(unread_only);
if ui.add(Toggle::new(&mut chosen, &["All", "Unread"])).changed() { /* … */ }

// An amount between two ends: a field filled up to it, dragged anywhere,
// a double click back to the default.
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
  `.selected(bool)`, `.width(w)`. It is a `Widget`: `ui.add` /
  `ui.add_enabled` and the usual `Response`.
- `TextField`: `.hint`, `.icon`, `.key`, `.small()`, `.width`, `.id_salt`. Its
  response is the text's own — `changed`, `has_focus`, `lost_focus`.
- `Toggle`: a few choices as one control; `changed()` when the choice moved.
- `Slider`: `.label`, `.default` (where a double click puts it back),
  `.decimals` (shown, and the steps it moves in), `.suffix`, `.small()`,
  `.width`; `changed()` whenever the value moved. The fill runs from zero
  when the range crosses zero, else from the start.
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
- Icons are the app's own: anything `Fn(&Painter, Rect, Color32)`, drawn in the
  square and colour the widget hands it. Post's `icons::glyph(name)` is one.
- `omarchy_kit::on(fill)` is the text colour that reads on a fill.

## How to use them

- One primary button per screen: the thing the screen is for — Send, Compose.
- Secondary for every other button: Discard, Attach, Cancel, Add account.
- Quiet for tools on a bar, back arrows and section toggles (Cc / Bcc); with
  `.selected(true)` for the one whose screen is on screen.
- Danger only for what cannot be undone, and only once it has been asked about.
- Sizes: regular by default; small inside lists, chips and dense bars; large
  for the one control a screen leads with.
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
- A picker is a `Button` with `.hint("▾")` and a `Popup::menu` of quiet
  buttons, `.selected` on the current one.
- No egui `Button`, `small_button`, `selectable_label`, `ComboBox` or framed
  `TextEdit` in an app that uses the kit, and no hand-painted buttons. What the
  kit lacks gets added to the kit — `crates/omarchy-kit` — with a test, so
  every app gets it.
