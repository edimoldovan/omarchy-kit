//! Where a modeless dialog sits as the window changes size.

use egui::accesskit::Role;
use egui::{Event, Modifiers, PointerButton, Pos2, Rect, Vec2, vec2};
use egui_kittest::Harness;
use egui_kittest::kittest::Queryable;
use omarchy_kit::Dialog;

fn harness(width: f32) -> Harness<'static> {
    Harness::builder().with_size(vec2(width, 900.0)).build(|ctx| {
        Dialog::new("Find").modeless().width(320.0).show(ctx, |ui| {
            ui.label("What to find");
        });
    })
}

fn dialog(harness: &Harness<'_>) -> Rect {
    harness.get_by_role_and_label(Role::Dialog, "Find").rect()
}

fn resize(harness: &mut Harness<'_>, width: f32) {
    harness.set_size(vec2(width, 900.0));
    harness.run();
}

/// Until it is dragged, it stays centred under the window's top as the
/// window changes size.
#[test]
fn a_modeless_dialog_follows_the_window_until_dragged() {
    let mut harness = harness(1600.0);
    harness.run();
    assert!((dialog(&harness).center().x - 800.0).abs() < 1.0, "{:?}", dialog(&harness));
    resize(&mut harness, 1000.0);
    assert!((dialog(&harness).center().x - 500.0).abs() < 1.0, "{:?}", dialog(&harness));
    let top = dialog(&harness).top();
    resize(&mut harness, 1300.0);
    assert!((dialog(&harness).center().x - 650.0).abs() < 1.0);
    assert_eq!(dialog(&harness).top(), top, "96 points down, whatever the width");
}

/// Dragged, it keeps where it was left as the window changes size, held
/// inside the window when the window shrinks past it.
#[test]
fn a_dragged_dialog_keeps_its_place_inside_the_window() {
    let mut harness = harness(1600.0);
    harness.run();
    // The card's margin, outside what is in it: the dialog's own background.
    let from = dialog(&harness).left_top() - Vec2::splat(4.0);
    let to = from + vec2(-500.0, 120.0);
    let button = |pos: Pos2, pressed| Event::PointerButton { pos, button: PointerButton::Primary, pressed, modifiers: Modifiers::NONE };
    // egui repaints while a window is held, so the drag goes a frame at a time.
    harness.event(Event::PointerMoved(from));
    harness.event(button(from, true));
    harness.step();
    harness.event(Event::PointerMoved(from + vec2(-250.0, 60.0)));
    harness.step();
    harness.event(Event::PointerMoved(to));
    harness.step();
    harness.event(button(to, false));
    harness.run();
    let left = dialog(&harness);
    assert!(left.center().x < 500.0, "dragged left: {left:?}");

    resize(&mut harness, 1400.0);
    assert_eq!(dialog(&harness), left, "a wider window leaves it where it was dragged");
    resize(&mut harness, 360.0);
    let held = dialog(&harness);
    assert!(held.left() >= 0.0 && held.right() <= 360.0, "held inside the window: {held:?}");
}
