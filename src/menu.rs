//! Menus: the menu bar along a window's top, and the menu a right-click
//! opens — the same lines in both.

use egui::accesskit::{Role, Toggled};
use egui::{
    Align2, Area, Context, FontId, Id, Key, LayerId, Modifiers, Order, Pos2, Rect, Response, Sense, Shape, Stroke, Ui,
    Vec2, WidgetInfo, WidgetType,
};

use crate::palette::{MARGIN, popup_frame};
use crate::{Button, Ink, Item, Size};

/// The narrowest a menu is.
const NARROWEST: f32 = 160.0;
/// The column the ticks of a menu's checked lines sit in.
const CHECK: f32 = 18.0;
/// The column a submenu's chevron sits in.
const CHEVRON: f32 = 16.0;
/// Room between a line's label and its key.
const GAP: f32 = 24.0;

/// What a menu's line is, as the keyboard walks it.
#[derive(Clone, Copy, PartialEq, Debug)]
enum Kind {
    Item,
    Submenu,
    /// A separator, or a line that is disabled: the keyboard passes over it.
    Inert,
}

/// One chain of open menus — a menu bar's, or a right-click's.
#[derive(Clone, Default)]
struct Chain {
    /// The bar's open menu; for a right-click menu, 0 while it is open.
    open: Option<usize>,
    /// Where a right-click menu opened.
    anchor: Pos2,
    /// The submenu open at each depth: `subs[d]` is the line of the menu at
    /// depth `d` whose submenu is open.
    subs: Vec<usize>,
    /// The line at each depth the pointer or the keyboard is on.
    keyed: Vec<Option<usize>>,
    /// Each depth's lines as last drawn, for the keyboard to walk.
    lines: Vec<Vec<Kind>>,
    /// The line Enter ran this frame, as (depth, line).
    run: Option<(usize, usize)>,
    /// How many menus the bar had, as last drawn.
    count: usize,
}

impl Chain {
    /// Opens `menu` with nothing under the keyboard yet.
    fn open(&mut self, menu: usize) {
        self.open = Some(menu);
        self.subs.clear();
        self.keyed = vec![None];
    }

    fn close(&mut self) {
        self.open = None;
        self.subs.clear();
        self.keyed.clear();
        self.run = None;
    }
}

fn chain_id(root: Id) -> Id {
    root.with("omarchy-kit-menu-chain")
}

fn load(ctx: &Context, root: Id) -> Chain {
    ctx.data(|d| d.get_temp::<Chain>(chain_id(root))).unwrap_or_default()
}

fn change<R>(ctx: &Context, root: Id, f: impl FnOnce(&mut Chain) -> R) -> R {
    ctx.data_mut(|d| f(d.get_temp_mut_or_default::<Chain>(chain_id(root))))
}

/// The area the menu at `depth` of the chain is drawn in: one per menu, so
/// each keeps its own size.
fn area_id(root: Id, chain: &Chain, depth: usize) -> Id {
    root.with(("omarchy-kit-menu", chain.open, chain.subs[..depth.min(chain.subs.len())].to_vec()))
}

/// The next line from `from` the keyboard can be on, `step` down or up,
/// round the ends.
fn walk(lines: &[Kind], from: Option<usize>, step: isize) -> Option<usize> {
    let n = lines.len() as isize;
    let mut i = match from {
        Some(i) => i as isize,
        None if step > 0 => -1,
        None => n,
    };
    for _ in 0..n {
        i = (i + step).rem_euclid(n);
        if lines[i as usize] != Kind::Inert {
            return Some(i as usize);
        }
    }
    from
}

/// The keys, while the chain is open: Up and Down walk the deepest menu,
/// Enter runs its line or opens its submenu, Right opens a submenu or moves
/// to the bar's next menu, Left closes a submenu or moves to the bar's
/// previous menu, Esc closes them all.
fn keys(ctx: &Context, root: Id, bar: bool) {
    if load(ctx, root).open.is_none() {
        return;
    }
    let [up, down, left, right, enter, escape] = ctx.input_mut(|i| {
        [Key::ArrowUp, Key::ArrowDown, Key::ArrowLeft, Key::ArrowRight, Key::Enter, Key::Escape]
            .map(|key| i.consume_key(Modifiers::NONE, key))
    });
    change(ctx, root, |c| {
        if escape {
            c.close();
            return;
        }
        let depth = c.subs.len();
        c.keyed.resize(depth + 1, None);
        let lines = c.lines.get(depth).cloned().unwrap_or_default();
        if down {
            c.keyed[depth] = walk(&lines, c.keyed[depth], 1);
        }
        if up {
            c.keyed[depth] = walk(&lines, c.keyed[depth], -1);
        }
        let on = c.keyed[depth];
        let on_submenu = on.is_some_and(|i| lines.get(i) == Some(&Kind::Submenu));
        let count = c.count.max(1);
        if (right || enter) && on_submenu {
            c.subs.push(on.unwrap_or(0));
            c.keyed.push(Some(0));
        } else if right && bar {
            let next = (c.open.unwrap_or(0) + 1) % count;
            c.open(next);
        } else if enter && let Some(line) = on {
            c.run = Some((depth, line));
        }
        if left {
            if depth > 0 {
                c.subs.pop();
                c.keyed.truncate(depth);
            } else if bar {
                let previous = (c.open.unwrap_or(0) + count - 1) % count;
                c.open(previous);
            }
        }
    });
}

/// The pointer moved this frame: what it rests on is not chosen again while
/// the keyboard moves elsewhere.
fn moved(ctx: &Context) -> bool {
    ctx.input(|i| i.pointer.delta() != Vec2::ZERO)
}

/// A press anywhere but on the chain's menus, or on `also`, closes them.
fn close_on_press_elsewhere(ctx: &Context, root: Id, also: &[Rect]) {
    let chain = load(ctx, root);
    if chain.open.is_none() {
        return;
    }
    let Some(at) = ctx.input(|i| if i.pointer.any_pressed() { i.pointer.interact_pos() } else { None }) else {
        return;
    };
    if also.iter().any(|rect| rect.contains(at)) {
        return;
    }
    let inside = ctx.layer_id_at(at).is_some_and(|layer| {
        (0..=chain.subs.len()).any(|depth| layer == LayerId::new(Order::Foreground, area_id(root, &chain, depth)))
    });
    if !inside {
        change(ctx, root, Chain::close);
    }
}

/// The menu at `depth` of the chain, at `at`, with `add` drawing its lines.
fn level(ctx: &Context, root: Id, depth: usize, at: Pos2, add: impl FnOnce(&mut Menu<'_>)) {
    let area = area_id(root, &load(ctx, root), depth);
    let (wide, room) = ctx.data(|d| d.get_temp::<(f32, bool)>(area)).unwrap_or((NARROWEST, false));
    Area::new(area).order(Order::Foreground).fixed_pos(at).constrain(true).show(ctx, |ui| {
        popup_frame(ctx).show(ui, |ui| {
            ui.spacing_mut().item_spacing.y = 0.0;
            ctx.accesskit_node_builder(ui.id(), |node| node.set_role(Role::Menu));
            let mut menu = Menu { ui, root, depth, lines: Vec::new(), width: NARROWEST, checks: false, wide, room };
            add(&mut menu);
            let Menu { lines, width, checks, .. } = menu;
            ctx.data_mut(|d| d.insert_temp(area, (width, checks)));
            change(ctx, root, |c| {
                c.lines.resize(depth + 1, Vec::new());
                c.lines[depth] = lines;
            });
        });
    });
}

/// The menu bar along a window's top: its menus' names, each opening its
/// menu below it when clicked; with one open, the pointer moving onto
/// another name opens that one instead. The keyboard walks an open menu.
#[must_use = "a menu bar is drawn with bar.show(ui, …)"]
pub struct MenuBar {
    id: Id,
}

impl Default for MenuBar {
    fn default() -> Self {
        MenuBar::new()
    }
}

impl MenuBar {
    pub fn new() -> MenuBar {
        MenuBar { id: Id::new("omarchy-kit-menu-bar") }
    }

    /// Where egui keeps its state, when a window has two bars.
    pub fn id_salt(mut self, salt: impl std::hash::Hash) -> MenuBar {
        self.id = Id::new(("omarchy-kit-menu-bar", salt));
        self
    }

    /// Draws the bar, with `add` naming its menus.
    pub fn show(self, ui: &mut Ui, add: impl FnOnce(&mut Bar<'_>)) {
        let root = self.id;
        let ctx = ui.ctx().clone();
        keys(&ctx, root, true);
        let (count, names) = ui
            .horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 2.0;
                ctx.accesskit_node_builder(ui.id(), |node| node.set_role(Role::MenuBar));
                let mut bar = Bar { ui, root, count: 0, names: Vec::new() };
                add(&mut bar);
                (bar.count, bar.names)
            })
            .inner;
        change(&ctx, root, |c| c.count = count);
        close_on_press_elsewhere(&ctx, root, &names);
        change(&ctx, root, |c| c.run = None);
    }
}

/// The bar a [`MenuBar`] hands its content: each menu's name, and its lines.
pub struct Bar<'u> {
    ui: &'u mut Ui,
    root: Id,
    count: usize,
    names: Vec<Rect>,
}

impl Bar<'_> {
    /// A menu called `name`, with `add` drawing its lines while it is open.
    pub fn menu(&mut self, name: &str, add: impl FnOnce(&mut Menu<'_>)) {
        let index = self.count;
        self.count += 1;
        let ctx = self.ui.ctx().clone();
        let open = load(&ctx, self.root).open;
        let response = self.ui.add(Button::new(name).quiet().small().selected(open == Some(index)));
        self.names.push(response.rect);
        if response.clicked() {
            change(&ctx, self.root, |c| if open == Some(index) { c.close() } else { c.open(index) });
        } else if open.is_some_and(|open| open != index) && response.hovered() && moved(&ctx) {
            change(&ctx, self.root, |c| c.open(index));
        }
        let is_open = load(&ctx, self.root).open == Some(index);
        ctx.accesskit_node_builder(response.id, |node| {
            node.set_role(Role::MenuItem);
            node.set_expanded(is_open);
        });
        if is_open {
            level(&ctx, self.root, 0, response.rect.left_bottom() + Vec2::new(0.0, 4.0), add);
        }
    }
}

/// The menu a right-click on `response` opens, at the pointer, with `add`
/// drawing its lines — the same lines as a menu bar's.
pub fn context_menu(response: &Response, add: impl FnOnce(&mut Menu<'_>)) {
    let ctx = &response.ctx;
    let root = response.id.with("omarchy-kit-context-menu");
    if response.secondary_clicked() {
        let at = response.interact_pointer_pos().unwrap_or(response.rect.center());
        change(ctx, root, |c| {
            c.open(0);
            c.anchor = at;
        });
    }
    let chain = load(ctx, root);
    if chain.open.is_none() {
        return;
    }
    keys(ctx, root, false);
    if load(ctx, root).open.is_some() {
        level(ctx, root, 0, chain.anchor, add);
    }
    close_on_press_elsewhere(ctx, root, &[]);
    change(ctx, root, |c| c.run = None);
}

/// One open menu: its lines, drawn one under another, each as wide as the
/// widest. A line's label is on its left and its key, dim, on its right;
/// lines that can be checked have their tick in a column of its own.
pub struct Menu<'u> {
    ui: &'u mut Ui,
    root: Id,
    depth: usize,
    lines: Vec<Kind>,
    /// The widest line wanted this frame.
    width: f32,
    /// A line here can be checked, this frame.
    checks: bool,
    /// The width the lines take: the widest, as last drawn.
    wide: f32,
    /// The tick's column is there: a line could be checked, last drawn.
    room: bool,
}

/// What a line came to.
struct Line {
    index: usize,
    rect: Rect,
    ran: bool,
}

impl<'u> Menu<'u> {
    /// A line that runs something: `.key(..)`, `.enabled(..)`, and
    /// `.clicked()`, which draws it.
    pub fn item(&mut self, label: impl Into<String>) -> MenuItem<'_, 'u> {
        MenuItem { menu: self, label: label.into(), key: String::new(), enabled: true }
    }

    /// A line with a tick while `on`; running it turns `on` over. True when
    /// it did.
    pub fn check(&mut self, label: impl Into<String>, on: &mut bool) -> bool {
        self.checks = true;
        let ran = self.line(&label.into(), "", true, Kind::Item, Some(*on)).ran;
        if ran {
            *on = !*on;
        }
        ran
    }

    /// A line between groups of lines.
    pub fn separator(&mut self) {
        self.lines.push(Kind::Inert);
        let ink = Ink::of(self.ui);
        let (rect, _) = self.ui.allocate_exact_size(Vec2::new(self.wide, 9.0), Sense::hover());
        let y = rect.center().y;
        let pad = Size::Small.pad();
        self.ui.painter().hline(rect.left() + pad..=rect.right() - pad, y, Stroke::new(1.0_f32, ink.shade(0.16)));
    }

    /// A line opening a menu of its own beside it, with `add` drawing that
    /// menu's lines.
    pub fn submenu(&mut self, label: impl Into<String>, add: impl FnOnce(&mut Menu<'_>)) {
        let line = self.line(&label.into(), "", true, Kind::Submenu, None);
        let ctx = self.ui.ctx().clone();
        if load(&ctx, self.root).subs.get(self.depth) == Some(&line.index) {
            let at = line.rect.right_top() + Vec2::new(MARGIN + 2.0, -MARGIN);
            level(&ctx, self.root, self.depth + 1, at, add);
        }
    }

    /// Draws one line and answers for the pointer and the keyboard on it.
    fn line(&mut self, label: &str, key: &str, enabled: bool, kind: Kind, checked: Option<bool>) -> Line {
        let ctx = self.ui.ctx().clone();
        let ink = Ink::of(self.ui);
        let size = Size::Small;
        let (root, depth) = (self.root, self.depth);
        let index = self.lines.len();
        self.lines.push(if enabled { kind } else { Kind::Inert });

        let font = FontId::proportional(size.text());
        let key_font = FontId::proportional(size.text() - 2.0);
        let measure = |text: &str, font: &FontId| {
            ctx.fonts_mut(|f| f.layout_no_wrap(text.to_string(), font.clone(), ink.fg).size().x)
        };
        let check = if self.room { CHECK } else { 0.0 };
        let chevron = if kind == Kind::Submenu { CHEVRON } else { 0.0 };
        let key_w = if key.is_empty() { 0.0 } else { GAP + measure(key, &key_font) };
        let want = 2.0 * size.pad() + check + measure(label, &font) + key_w + chevron;
        self.width = self.width.max(want);
        let sense = if enabled { Sense::click() } else { Sense::hover() };
        let (rect, response) = self.ui.allocate_exact_size(Vec2::new(self.wide.max(want), size.height()), sense);

        // The pointer moving onto a line puts the keyboard there too, and
        // opens a submenu or closes the one open beside it.
        if response.hovered() && moved(&ctx) {
            change(&ctx, root, |c| {
                c.keyed.resize(depth + 1, None);
                c.keyed[depth] = enabled.then_some(index);
                c.subs.truncate(depth);
                c.keyed.truncate(depth + 1);
                if enabled && kind == Kind::Submenu {
                    c.subs.push(index);
                }
            });
        }
        let chain = load(&ctx, root);
        let mut ran = enabled && (response.clicked() || chain.run == Some((depth, index)));
        if ran && kind == Kind::Submenu {
            change(&ctx, root, |c| {
                c.subs.truncate(depth);
                c.subs.push(index);
                c.keyed.truncate(depth + 1);
                c.keyed.push(Some(0));
            });
            ran = false;
        } else if ran {
            change(&ctx, root, Chain::close);
        }

        let chain = load(&ctx, root);
        let open = kind == Kind::Submenu && chain.subs.get(depth) == Some(&index);
        let on = enabled && chain.keyed.get(depth).copied().flatten() == Some(index);
        Item { open, keyed: false, hovered: on, size }.paint(self.ui, rect);
        let painter = self.ui.painter();
        let text = if enabled { ink.fg } else { ink.weak.gamma_multiply(0.7) };
        let y = rect.center().y;
        if checked == Some(true) {
            let at = |x: f32, dy: f32| Pos2::new(rect.left() + size.pad() + x, y + dy);
            let tick = Stroke::new(1.6_f32, text);
            painter.line_segment([at(1.0, 0.5), at(4.5, 4.0)], tick);
            painter.line_segment([at(4.5, 4.0), at(11.0, -4.0)], tick);
        }
        painter.text(Pos2::new(rect.left() + size.pad() + check, y), Align2::LEFT_CENTER, label, font, text);
        if !key.is_empty() {
            let end = Pos2::new(rect.right() - size.pad() - chevron, y);
            painter.text(end, Align2::RIGHT_CENTER, key, key_font, ink.weak);
        }
        if kind == Kind::Submenu {
            let c = Pos2::new(rect.right() - size.pad() - 3.0, y);
            painter.add(Shape::line(vec![c + Vec2::new(-2.5, -4.0), c + Vec2::new(1.5, 0.0), c + Vec2::new(-2.5, 4.0)], Stroke::new(1.5_f32, ink.weak)));
        }

        // To a screen reader and a test: named by its label, its key its
        // keyboard shortcut.
        response.widget_info(|| WidgetInfo::labeled(WidgetType::Button, enabled, label));
        ctx.accesskit_node_builder(response.id, |node| {
            node.set_role(if checked.is_some() { Role::MenuItemCheckBox } else { Role::MenuItem });
            if !key.is_empty() {
                node.set_keyboard_shortcut(key);
            }
            if let Some(checked) = checked {
                node.set_toggled(if checked { Toggled::True } else { Toggled::False });
            }
            if kind == Kind::Submenu {
                node.set_expanded(open);
            }
        });
        Line { index, rect, ran }
    }
}

/// A line of a [`Menu`] that runs something, drawn by [`MenuItem::clicked`].
#[must_use = "a menu item is drawn by .clicked()"]
pub struct MenuItem<'m, 'u> {
    menu: &'m mut Menu<'u>,
    label: String,
    key: String,
    enabled: bool,
}

impl MenuItem<'_, '_> {
    /// The key it is also on, dim at the line's end.
    pub fn key(mut self, key: impl Into<String>) -> Self {
        self.key = key.into();
        self
    }

    /// Whether it can be run; dimmed, and passed over by the keyboard, when
    /// not.
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// Draws it; true when it was run — a click, or Enter on it — which
    /// closes the menus it is in.
    pub fn clicked(self) -> bool {
        self.menu.line(&self.label, &self.key, self.enabled, Kind::Item, None).ran
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The keyboard walks round the ends and passes over separators and
    /// disabled lines.
    #[test]
    fn the_keyboard_passes_over_what_cannot_run() {
        let lines = [Kind::Item, Kind::Inert, Kind::Submenu, Kind::Inert];
        assert_eq!(walk(&lines, None, 1), Some(0));
        assert_eq!(walk(&lines, Some(0), 1), Some(2));
        assert_eq!(walk(&lines, Some(2), 1), Some(0), "round the end");
        assert_eq!(walk(&lines, None, -1), Some(2));
        assert_eq!(walk(&[Kind::Inert], None, 1), None);
    }
}
