# Plan: what Viode needs from the kit

Twelve additions, in six steps. Every new element reports
its role, name and state to AccessKit, with a test in `tests/access.rs`.
Each step updates `README.md` and `SKILL.md` (and the installed copy at
`~/.claude/skills/omarchy-kit/SKILL.md`).

A kit bump in Post and Grido needs one change there: each `Choice { .. }`
they build gains `keywords: String::new()`.

## Step 1 — Theme

- `Palette.ansi: [Color32; 16]` (0–7 normal, 8–15 bright), with `red()`,
  `green()`, `yellow()`, `blue()`, `magenta()`, `cyan()`.
- Read from `color0`–`color15` in `colors.toml`, else from alacritty's
  `[colors.normal]` and `[colors.bright]`. A missing entry takes the
  fallback's.
- `theme::load_or(&fallback)` and `ThemeWatcher::with_fallback(fallback)` use
  the app's palette when there is no Omarchy theme or a file can't be read.
  `load()` keeps the kit's own fallback, which carries sixteen colours too.
- The colour rule: colours come from the palette, except a product's logo and
  colours that show a colour itself — hue and saturation rails, red/green/blue
  curves and parade traces, colour wheels, vectorscope targets, the picture.
- Done when: tests read both file formats, a missing colour falls back, a
  theme switch changes the colours, and with no theme folder `load_or`
  returns the given palette exactly (Viode's: background 18,20,24; text
  190,196,205; accent 120,160,220; yellow 196,160,66; blue 100,130,180; green
  110,160,100; cyan 90,160,170; magenta 170,110,170).

## Step 2 — Fonts

- `fonts::install` loads, in order: the Linux system paths; on macOS
  `/System/Library/Fonts/Apple Symbols.ttf`; last, egui's bundled Hack behind
  the proportional face.
- The rule: apps draw as text only glyphs the bundled fonts cover — `×` and
  `✔`, not `✕`, `✓` or `⏎`. The kit draws its own chevron, cross and tick
  with lines.
- Done when: a test with no system fonts finds every glyph of a written list
  covered after `install`.

## Step 3 — Icons, palette keywords, transparency

- Icons, drawn with `icons::Pen`. Already in the kit: `plus` (add), `prev`,
  `next`, `keyboard`, `trash`. New: select, razor, slip, slide, stretch;
  split, render, snapping; lock, unlock, eye, eye-off, mute, solo, effects;
  keyframe, keyframe-empty, keyframe-prev, keyframe-next; play, pause,
  step-back, step-forward, start, end, shuttle-back, shuttle-forward;
  mark-in, mark-out, insert, overwrite; help.
- `Choice` gains `keywords: String`. An entry matches when every typed word
  is in its label or its keywords, ignoring case; label matches rank first.
- `Swatches::alpha()` and `ColorButton::alpha()` add an opacity slider under
  the grid; see-through swatches draw over a checkerboard. The hex field
  takes `#RRGGBB` and `#AARRGGBB`; `hex_argb` and `parse_argb` write and read
  `#AARRGGBB`.
- Done when: the icon test draws each icon; a keyword not in the label finds
  its entry; `#80FF0000` round-trips through the field and the functions.

## Step 4 — Switch and Section

- `Switch::new(&mut bool)`, `.small()`, `.name(..)`: a pill with a sliding
  knob — accent when on, a background/text mix when off, the knob in
  `on(fill)`. A click or Space flips it. Reports the switch role, its name
  and on/off state.
- The rule: a single option is a `Checkbox`; turning a feature or a group on
  and off, its settings kept, is a `Switch`.
- `Section::new(id, title).switch(&mut on).reset(enabled).default_open(..)
  .show(ui, |ui| ..) -> SectionResponse { switched, reset, open }`. The
  header: the fold chevron and the title; at the far end the switch and a
  quiet icon-only reset button. Folding is remembered per id; sections start
  open. While the switch is off the body draws disabled, its values kept.
- Names: the fold toggle is the title, with its expanded state; the switch is
  "TITLE section"; the reset is "Reset TITLE".
- Done when: tests flip the switch by click and Space, fold and unfold by
  click and keyboard, find the body disabled while off, and see the reset.

## Step 5 — Slider and NumberField

- Both take `&mut f32` or `&mut f64`; existing f32 calls stay as they are.
- Slider: `.inline_label(width)` puts the label in a fixed column on the
  track's line, at the kit's Small size (26 points); `.logarithmic()`;
  `.track(&[(f32, Color32)])` paints colour stops along the track in place
  of the fill.
- NumberField: dragging sideways changes the value by `.speed(s)` per point,
  with the sideways-resize cursor; a click without a drag starts typing;
  `.prefix(..)` joins `.suffix(..)`; open ranges such as `0.0..=f64::MAX`.
- The adjustment: the trait `omarchy_kit::Adjust` gives `adjusting()` and
  `adjust_ended()` on the response. An adjustment runs from the first press
  on the track, or the first keyboard nudge, until the pointer is up and the
  field has lost the keyboard; `adjust_ended()` is true on that frame.
- Done when: tests cover f64 precision, the logarithmic mapping both ways,
  the inline layout widths, the gradient painted, one start and one end
  across a drag followed by nudges; and on the number field dragging, typing,
  the prefix and suffix, and clamping.

## Step 6 — Menus

- One item builder for `MenuBar::new().show(ui, |bar| bar.menu(..))` and
  `context_menu(&response, |m| ..)`: `m.item(label).key(k).enabled(b)
  .clicked()`, `m.check(label, &mut bool)`, `m.separator()`,
  `m.submenu(label, |m| ..)`.
- A row: the label left, the key dim at the right; a check column when any
  item of the menu can be checked; disabled items dimmed and inert; the hover
  mark is `Item`.
- Keyboard: Up and Down walk the items, Enter runs one, Esc closes; Left and
  Right move across the bar and into and out of submenus. A click runs the
  item and closes the whole chain.
- Names: an item is named by its label; its key is its keyboard shortcut.
  Items report the menu item role, or the checkbox menu item role with the
  checked state.
- Done when: tests cover clicking, the keyboard, disabled and checked items,
  submenus and closing.
