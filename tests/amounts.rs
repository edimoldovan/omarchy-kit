//! Sliders and number fields: 64-bit values, rows that line up, dragging,
//! and adjustments that end once.

use egui::accesskit::Role;
use egui::{Event, Key, Modifiers, PointerButton, Pos2, Vec2};
use egui_kittest::Harness;
use egui_kittest::kittest::Queryable;
use omarchy_kit::{Adjust, NumberField, Slider};

fn press(harness: &mut Harness<'_, impl Sized>, at: Pos2, pressed: bool) {
    harness.event(Event::PointerMoved(at));
    harness.event(Event::PointerButton { pos: at, button: PointerButton::Primary, pressed, modifiers: Modifiers::NONE });
    harness.run();
}

/// Typed into, a 64-bit slider and number field keep digits a 32-bit one
/// would lose, and the number field takes an open range.
#[test]
fn sixty_four_bit_values_keep_their_precision() {
    let mut harness = Harness::new_ui_state(
        |ui, (position, bitrate): &mut (f64, f64)| {
            ui.add(Slider::new(position, 0.0..=1e10).decimals(6).name("Position").id_salt("position"));
            ui.add(NumberField::new(bitrate, 0.0..=f64::MAX).decimals(3).name("Bitrate").id_salt("bitrate"));
        },
        (0.0, 0.0),
    );
    harness.get_by_role(Role::TextInput).focus();
    harness.run();
    harness.key_press_modifiers(Modifiers::COMMAND, Key::A);
    harness.get_by_role(Role::TextInput).type_text("1234567890.123456");
    harness.key_press(Key::Enter);
    harness.run();
    assert!((harness.state().0 - 1_234_567_890.123_456).abs() < 1e-6, "{}", harness.state().0);

    harness.get_by_role_and_label(Role::SpinButton, "Bitrate").focus();
    harness.run();
    harness.key_press_modifiers(Modifiers::COMMAND, Key::A);
    harness.get_by_role_and_label(Role::SpinButton, "Bitrate").type_text("98765432101.125");
    harness.key_press(Key::Enter);
    harness.run();
    assert_eq!(harness.state().1, 98_765_432_101.125);
}

/// Sliders with their labels inline are one small row each, their tracks
/// starting at the same place, the label's column wide from the row's start.
#[test]
fn inline_labels_line_the_rows_up() {
    let harness = Harness::new_ui_state(
        |ui, (opacity, scale): &mut (f32, f32)| {
            ui.add(Slider::new(opacity, 0.0..=1.0).label("Opacity").inline_label(74.0).width(300.0).id_salt("o"));
            ui.add(Slider::new(scale, 0.1..=10.0).label("Scale").inline_label(74.0).width(300.0).id_salt("s"));
        },
        (0.5, 1.0),
    );
    let opacity = harness.get_by_role_and_label(Role::Slider, "Opacity").rect();
    let scale = harness.get_by_role_and_label(Role::Slider, "Scale").rect();
    assert_eq!(opacity.left(), scale.left(), "the tracks line up");
    assert_eq!(opacity.height(), 26.0, "a small row");
    // The row less the label's column, the field and the gap before it.
    assert_eq!(opacity.width(), 300.0 - 74.0 - 72.0 - 10.0);
}

/// A drag on the track and the nudges after it, while the field keeps the
/// keyboard, are one adjustment: it starts once and ends once, when the
/// field gives the keyboard up.
#[test]
fn a_drag_and_its_nudges_are_one_adjustment() {
    let mut harness = Harness::new_ui_state(
        |ui, (scale, starts, ends, was): &mut (f32, u32, u32, bool)| {
            let slider = ui.add(Slider::new(scale, 0.0..=1.0).name("Scale").width(300.0).id_salt("scale"));
            if slider.adjusting() && !*was {
                *starts += 1;
            }
            *was = slider.adjusting();
            if slider.adjust_ended() {
                *ends += 1;
            }
        },
        (0.0, 0, 0, false),
    );
    let track = harness.get_by_role_and_label(Role::Slider, "Scale").rect();
    press(&mut harness, track.left_center() + Vec2::new(20.0, 0.0), true);
    harness.event(Event::PointerMoved(track.center()));
    harness.run();
    press(&mut harness, track.center(), false);
    let dragged = harness.state().0;
    assert!(dragged > 0.3, "the drag moved it: {dragged}");
    assert_eq!((harness.state().1, harness.state().2), (1, 0), "still adjusting: the field has the keyboard");

    harness.key_press(Key::ArrowUp);
    harness.run();
    harness.key_press(Key::ArrowUp);
    harness.run();
    assert!((harness.state().0 - (dragged + 0.02)).abs() < 1e-6, "nudged twice");
    assert_eq!((harness.state().1, harness.state().2), (1, 0), "the nudges are the same adjustment");

    harness.key_press(Key::Enter);
    harness.run();
    assert_eq!((harness.state().1, harness.state().2), (1, 1), "it ended once");
}

/// A slider that keeps the keyboard leaves it where it was after a drag:
/// the adjustment ends as the pointer comes up, and keys go on reaching
/// the app rather than the field.
#[test]
fn a_slider_that_keeps_the_keyboard_ends_its_adjustment_on_release() {
    let mut harness = Harness::new_ui_state(
        |ui, (scale, ends, focused): &mut (f32, u32, bool)| {
            let slider = ui.add(Slider::new(scale, 0.0..=1.0).name("Scale").width(300.0).keep_keyboard().id_salt("scale"));
            if slider.adjust_ended() {
                *ends += 1;
            }
            *focused = ui.ctx().wants_keyboard_input();
        },
        (0.0, 0, false),
    );
    let track = harness.get_by_role_and_label(Role::Slider, "Scale").rect();
    press(&mut harness, track.left_center() + Vec2::new(20.0, 0.0), true);
    harness.event(Event::PointerMoved(track.center()));
    harness.run();
    press(&mut harness, track.center(), false);
    harness.run();
    assert!(harness.state().0 > 0.3, "the drag moved it");
    assert_eq!(harness.state().1, 1, "the adjustment ended with the drag");
    assert!(!harness.state().2, "no field took the keyboard");
}

/// Dragged sideways, a number field moves by its speed per point and the
/// adjustment ends as the pointer comes up; a click without a drag puts the
/// cursor in to type.
#[test]
fn a_number_field_drags_and_still_types() {
    let mut harness = Harness::new_ui_state(
        |ui, (strength, ends): &mut (f64, u32)| {
            let field = ui.add(NumberField::new(strength, 0.0..=100.0).speed(0.5).name("Strength").id_salt("strength"));
            if field.adjust_ended() {
                *ends += 1;
            }
        },
        (10.0, 0),
    );
    let at = harness.get_by_role_and_label(Role::SpinButton, "Strength").rect().center();
    press(&mut harness, at, true);
    harness.event(Event::PointerMoved(at + Vec2::new(40.0, 0.0)));
    harness.run();
    press(&mut harness, at + Vec2::new(40.0, 0.0), false);
    assert_eq!(harness.state().0, 30.0, "40 points at half a unit each");
    assert_eq!(harness.state().1, 1, "the drag was one adjustment");
    assert!(!harness.get_by_role_and_label(Role::SpinButton, "Strength").is_focused(), "a drag does not take the keyboard");

    harness.get_by_role_and_label(Role::SpinButton, "Strength").click();
    harness.run();
    assert!(harness.get_by_role_and_label(Role::SpinButton, "Strength").is_focused(), "a click does");
}

/// The prefix and suffix sit around the number, typed or not, and what is
/// typed is kept within the range.
#[test]
fn a_number_field_takes_a_prefix_a_suffix_and_its_range() {
    let mut harness = Harness::new_ui_state(
        |ui, price: &mut f64| {
            ui.add(NumberField::new(price, 0.0..=100.0).decimals(2).prefix("€").suffix("%").name("Discount").id_salt("discount"));
        },
        5.0,
    );
    harness.get_by_role_and_label(Role::Label, "€");
    harness.get_by_role_and_label(Role::Label, "%");
    let typed = |harness: &mut Harness<'_, f64>, text: &str| {
        harness.get_by_role_and_label(Role::SpinButton, "Discount").focus();
        harness.run();
        harness.key_press_modifiers(Modifiers::COMMAND, Key::A);
        harness.get_by_role_and_label(Role::SpinButton, "Discount").type_text(text);
        harness.key_press(Key::Enter);
        harness.run();
        *harness.state()
    };
    assert_eq!(typed(&mut harness, "€ 12.5 %"), 12.5, "typed with its prefix and suffix");
    assert_eq!(typed(&mut harness, "250"), 100.0, "kept within the range");
    assert_eq!(typed(&mut harness, "-3"), 0.0);
}

/// In a right-to-left row, as a status line's far end is, what comes after
/// a slider sits beside it, not over its track.
#[test]
fn a_slider_keeps_its_room_in_a_right_to_left_row() {
    let harness = Harness::new_ui_state(
        |ui, zoom: &mut f32| {
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.add(Slider::new(zoom, 40.0..=400.0).decimals(0).small().width(190.0).name("Zoom").id_salt("zoom"));
                ui.label("Count 42");
            });
        },
        100.0,
    );
    let track = harness.get_by_role_and_label(Role::Slider, "Zoom").rect();
    let count = harness.get_by_role_and_label(Role::Label, "Count 42").rect();
    assert!(count.right() <= track.left(), "the label beside the slider: {count:?} {track:?}");
}
