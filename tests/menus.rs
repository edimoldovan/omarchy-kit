//! Menus, driven the way a person and a screen reader drive them.

use egui::accesskit::{Role, Toggled};
use egui::{Event, Key, Modifiers, PointerButton, Pos2, Sense};
use egui_kittest::Harness;
use egui_kittest::kittest::{NodeT, Queryable};
use omarchy_kit::{MenuBar, context_menu};

#[derive(Default)]
struct App {
    dirty: bool,
    saved: u32,
    opened: u32,
    snapping: bool,
    movies: u32,
    cut: u32,
}

fn menus(ui: &mut egui::Ui, app: &mut App) {
    MenuBar::new().show(ui, |bar| {
        bar.menu("File", |m| {
            if m.item("Save").key("Ctrl+S").enabled(app.dirty).clicked() {
                app.saved += 1;
            }
            if m.item("Open").key("Ctrl+O").clicked() {
                app.opened += 1;
            }
            m.separator();
            m.check("Snapping", &mut app.snapping);
            m.submenu("Export", |m| {
                if m.item("Movie").clicked() {
                    app.movies += 1;
                }
                let _ = m.item("Frame").clicked();
            });
        });
        bar.menu("Edit", |m| {
            if m.item("Cut").key("Ctrl+X").clicked() {
                app.cut += 1;
            }
        });
    });
}

fn harness() -> Harness<'static, App> {
    Harness::new_ui_state(menus, App::default())
}

fn open_file(harness: &mut Harness<'_, App>) {
    harness.get_by_role_and_label(Role::MenuItem, "File").click();
    harness.run();
}

fn shows(harness: &Harness<'_, App>, role: Role, label: &str) -> bool {
    harness.query_by_role_and_label(role, label).is_some()
}

/// A click opens a menu; a click on a line runs it and closes the menu. A
/// line is named by its label, its key reported apart.
#[test]
fn a_click_runs_a_line_and_closes_the_menu() {
    let mut harness = harness();
    assert!(!shows(&harness, Role::MenuItem, "Open"));
    open_file(&mut harness);
    let file = harness.get_by_role_and_label(Role::MenuItem, "File");
    assert_eq!(file.accesskit_node().data().is_expanded(), Some(true));
    let open = harness.get_by_role_and_label(Role::MenuItem, "Open");
    assert_eq!(open.accesskit_node().data().keyboard_shortcut(), Some("Ctrl+O"));
    open.click();
    harness.run();
    assert_eq!(harness.state().opened, 1);
    assert!(!shows(&harness, Role::MenuItem, "Open"), "running it closed the menu");
}

/// A disabled line is reported so, does nothing when clicked, and leaves the
/// menu open.
#[test]
fn a_disabled_line_does_nothing() {
    let mut harness = harness();
    open_file(&mut harness);
    let save = harness.get_by_role_and_label(Role::MenuItem, "Save");
    assert!(save.accesskit_node().is_disabled());
    save.click();
    harness.run();
    assert_eq!(harness.state().saved, 0);
    assert!(shows(&harness, Role::MenuItem, "Open"), "still open");
}

/// A line that can be checked reports its state and turns it over.
#[test]
fn a_checked_line_reports_and_turns_over() {
    let mut harness = harness();
    open_file(&mut harness);
    let snapping = harness.get_by_role_and_label(Role::MenuItemCheckBox, "Snapping");
    assert_eq!(snapping.accesskit_node().toggled(), Some(Toggled::False));
    snapping.click();
    harness.run();
    assert!(harness.state().snapping);
    open_file(&mut harness);
    let snapping = harness.get_by_role_and_label(Role::MenuItemCheckBox, "Snapping");
    assert_eq!(snapping.accesskit_node().toggled(), Some(Toggled::True));
}

/// A submenu opens beside its line; running a line in it closes the whole
/// chain.
#[test]
fn a_submenu_opens_and_a_run_closes_the_chain() {
    let mut harness = harness();
    open_file(&mut harness);
    harness.get_by_role_and_label(Role::MenuItem, "Export").click();
    harness.run();
    harness.get_by_role_and_label(Role::MenuItem, "Movie").click();
    harness.run();
    assert_eq!(harness.state().movies, 1);
    assert!(!shows(&harness, Role::MenuItem, "Movie"));
    assert!(!shows(&harness, Role::MenuItem, "Open"), "the menu it was in closed too");
}

/// Up and Down walk past what cannot run, Enter runs, Right and Left go into
/// and out of a submenu and across the bar, Esc closes.
#[test]
fn the_keyboard_walks_runs_and_closes() {
    let mut harness = harness();
    let press = |harness: &mut Harness<'_, App>, key: Key| {
        harness.key_press(key);
        harness.run();
    };
    open_file(&mut harness);
    press(&mut harness, Key::ArrowDown);
    press(&mut harness, Key::Enter);
    assert_eq!(harness.state().opened, 1, "Down passed the disabled Save to Open");
    assert!(!shows(&harness, Role::MenuItem, "Open"));

    open_file(&mut harness);
    for _ in 0..3 {
        press(&mut harness, Key::ArrowDown);
    }
    press(&mut harness, Key::ArrowRight);
    assert!(shows(&harness, Role::MenuItem, "Movie"), "past the separator to Export, and into it");
    press(&mut harness, Key::ArrowLeft);
    assert!(!shows(&harness, Role::MenuItem, "Movie"), "back out");
    press(&mut harness, Key::ArrowRight);
    press(&mut harness, Key::Enter);
    assert_eq!(harness.state().movies, 1);
    assert!(!shows(&harness, Role::MenuItem, "Open"), "the chain closed");

    open_file(&mut harness);
    press(&mut harness, Key::ArrowRight);
    assert!(shows(&harness, Role::MenuItem, "Cut"), "Right moved across the bar to Edit");
    press(&mut harness, Key::ArrowLeft);
    assert!(shows(&harness, Role::MenuItem, "Open"), "and Left back to File");
    press(&mut harness, Key::Escape);
    assert!(!shows(&harness, Role::MenuItem, "Open"), "Esc closed it");
}

/// With one menu open, the pointer on another name opens that one instead.
#[test]
fn the_pointer_moves_across_the_bar() {
    let mut harness = harness();
    open_file(&mut harness);
    harness.get_by_role_and_label(Role::MenuItem, "Edit").hover();
    harness.run();
    assert!(shows(&harness, Role::MenuItem, "Cut"));
    assert!(!shows(&harness, Role::MenuItem, "Open"));
}

/// A right-click opens the same lines at the pointer; Esc, or a press
/// anywhere else, closes them.
#[test]
fn a_right_click_opens_a_menu_of_the_same_lines() {
    let mut harness = Harness::new_ui_state(
        |ui, app: &mut App| {
            let clip = ui.add(egui::Label::new("Clip").sense(Sense::click()));
            context_menu(&clip, |m| {
                if m.item("Cut").key("Ctrl+X").clicked() {
                    app.cut += 1;
                }
                m.check("Snapping", &mut app.snapping);
            });
        },
        App::default(),
    );
    let open = |harness: &mut Harness<'_, App>| {
        harness.get_by_role_and_label(Role::Label, "Clip").click_secondary();
        harness.run();
    };
    open(&mut harness);
    harness.get_by_role_and_label(Role::MenuItem, "Cut").click();
    harness.run();
    assert_eq!(harness.state().cut, 1);
    assert!(!shows(&harness, Role::MenuItem, "Cut"));

    open(&mut harness);
    harness.key_press(Key::Escape);
    harness.run();
    assert!(!shows(&harness, Role::MenuItem, "Cut"), "Esc closed it");

    open(&mut harness);
    let away = Pos2::new(400.0, 500.0);
    harness.event(Event::PointerMoved(away));
    harness.event(Event::PointerButton { pos: away, button: PointerButton::Primary, pressed: true, modifiers: Modifiers::NONE });
    harness.event(Event::PointerButton { pos: away, button: PointerButton::Primary, pressed: false, modifiers: Modifiers::NONE });
    harness.run();
    assert!(!shows(&harness, Role::MenuItem, "Cut"), "a press elsewhere closed it");
    assert_eq!(harness.state().cut, 1);
}
