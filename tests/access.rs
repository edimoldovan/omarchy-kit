//! Every element reports its role, its name and its value, so a screen
//! reader reads it and a test finds it by name.

use egui::Color32;
use egui::accesskit::{Role, Toggled};
use egui_kittest::Harness;
use egui_kittest::kittest::{NodeT, Queryable};
use omarchy_kit::{
    Button, Checkbox, Choice, ColorButton, CommandPalette, Dialog, Dropdown, NumberField, Slider, Swatches, TagField,
    TextField, Toggle,
};

fn toggled<'t>(node: &impl NodeT<'t>) -> Option<Toggled> {
    node.accesskit_node().toggled()
}

/// A button is named by its label; an icon-only one by its tip, without the
/// key the tip shows.
#[test]
fn buttons_are_named_by_their_label_or_their_tip() {
    let harness = Harness::new_ui(|ui| {
        ui.add(Button::new("Send"));
        ui.add(Button::icon_only(|_, _, _| {}).tip("Settings").hint("Home"));
    });
    harness.get_by_role_and_label(Role::Button, "Send");
    harness.get_by_role_and_label(Role::Button, "Settings");
}

/// A checkbox says whether it is on, and a click turns it.
#[test]
fn a_checkbox_reports_its_state() {
    let mut harness = Harness::new_ui_state(
        |ui, on: &mut bool| {
            ui.add(Checkbox::new(on, "Wrap lines"));
        },
        false,
    );
    let node = harness.get_by_role_and_label(Role::CheckBox, "Wrap lines");
    assert_eq!(toggled(&node), Some(Toggled::False));
    node.click();
    harness.run();
    assert!(*harness.state());
    assert_eq!(toggled(&harness.get_by_role_and_label(Role::CheckBox, "Wrap lines")), Some(Toggled::True));
}

/// Each option of a toggle is a radio button of its own, the chosen one on.
#[test]
fn each_segment_of_a_toggle_is_a_radio_button() {
    let mut harness = Harness::new_ui_state(
        |ui, chosen: &mut usize| {
            ui.add(Toggle::new(chosen, &["All", "Unread"]));
        },
        0,
    );
    assert_eq!(toggled(&harness.get_by_role_and_label(Role::RadioButton, "All")), Some(Toggled::True));
    harness.get_by_role_and_label(Role::RadioButton, "Unread").click();
    harness.run();
    assert_eq!(*harness.state(), 1);
    assert_eq!(toggled(&harness.get_by_role_and_label(Role::RadioButton, "Unread")), Some(Toggled::True));
    assert_eq!(toggled(&harness.get_by_role_and_label(Role::RadioButton, "All")), Some(Toggled::False));
}

/// A slider is named by its label, or by its name when it shows none, with
/// the amount as its value.
#[test]
fn a_slider_reports_its_name_and_amount() {
    let harness = Harness::new_ui_state(
        |ui, (a, b): &mut (f32, f32)| {
            ui.add(Slider::new(a, 0.0..=1.0).label("Brightness").id_salt("a"));
            ui.add(Slider::new(b, -100.0..=100.0).name("Contrast").id_salt("b"));
        },
        (0.5, 20.0),
    );
    let brightness = harness.get_by_role_and_label(Role::Slider, "Brightness");
    assert_eq!(brightness.accesskit_node().numeric_value(), Some(0.5));
    let contrast = harness.get_by_role_and_label(Role::Slider, "Contrast");
    assert_eq!(contrast.accesskit_node().numeric_value(), Some(20.0));
}

/// A dropdown is named, with the chosen item as its value; open, each line
/// is named, the chosen one on.
#[test]
fn a_dropdown_reports_its_chosen_item() {
    let mut harness = Harness::new_ui_state(
        |ui, chosen: &mut usize| {
            ui.add(Dropdown::new(chosen, &["PNG", "JPEG"]).name("Format").width(200.0));
        },
        0,
    );
    let node = harness.get_by_role_and_label(Role::ComboBox, "Format");
    assert_eq!(node.accesskit_node().value().as_deref(), Some("PNG"));
    node.click();
    harness.run();
    assert_eq!(toggled(&harness.get_by_role_and_label(Role::Button, "PNG")), Some(Toggled::True));
    harness.get_by_role_and_label(Role::Button, "JPEG").click();
    harness.run();
    assert_eq!(*harness.state(), 1);
    let node = harness.get_by_role_and_label(Role::ComboBox, "Format");
    assert_eq!(node.accesskit_node().value().as_deref(), Some("JPEG"));
}

/// A text field is named by its name, or by its hint when it has none; a
/// number field is a spin button with its number.
#[test]
fn fields_are_named() {
    let harness = Harness::new_ui_state(
        |ui, (a, b, n): &mut (String, String, f32)| {
            ui.add(TextField::new(a).hint("Search").id_salt("a"));
            ui.add(TextField::new(b).hint("you@example.com").name("Address").id_salt("b"));
            ui.add(NumberField::new(n, 6.0..=96.0).name("Size").id_salt("n"));
        },
        (String::new(), "ed@acme.com".to_string(), 11.0),
    );
    harness.get_by_role_and_label(Role::TextInput, "Search");
    let address = harness.get_by_role_and_label(Role::TextInput, "Address");
    assert_eq!(address.accesskit_node().value().as_deref(), Some("ed@acme.com"));
    let size = harness.get_by_role_and_label(Role::SpinButton, "Size");
    assert_eq!(size.accesskit_node().numeric_value(), Some(11.0));
}

/// A tag field is named; each tag is a label, and its × a button that takes
/// it off.
#[test]
fn a_tag_field_names_its_tags() {
    let mut harness = Harness::new_ui_state(
        |ui, (tags, typing): &mut (Vec<String>, String)| {
            ui.add(TagField::new(tags, typing).name("To").id_salt("to"));
        },
        (vec!["ale@acme.com".to_string()], String::new()),
    );
    harness.get_by_role_and_label(Role::TextInput, "To");
    harness.get_by_role_and_label(Role::Label, "ale@acme.com");
    harness.get_by_role_and_label(Role::Button, "Take ale@acme.com off").click();
    harness.run();
    assert!(harness.state().0.is_empty());
}

/// Each swatch is named by its hex, the chosen one on; a colour button by
/// its tip, with the colour's hex as its value.
#[test]
fn colours_are_named_by_their_hex() {
    let harness = Harness::new_ui_state(
        |ui, (a, b): &mut (Option<Color32>, Option<Color32>)| {
            ui.add(Swatches::new(a).clear("No colour").id_salt("swatches"));
            ui.add(ColorButton::new(b, |_, _, _| {}).tip("Fill colour"));
        },
        (Some(Color32::BLACK), Some(Color32::from_rgb(255, 0, 0))),
    );
    assert_eq!(toggled(&harness.get_by_role_and_label(Role::ColorWell, "#000000")), Some(Toggled::True));
    assert_eq!(toggled(&harness.get_by_role_and_label(Role::ColorWell, "#FFFFFF")), Some(Toggled::False));
    harness.get_by_role_and_label(Role::Button, "No colour");
    let fill = harness.get_by_role_and_label(Role::Button, "Fill colour");
    assert_eq!(fill.accesskit_node().value().as_deref(), Some("#FF0000"));
}

/// A dialog is named by its title, and its × is Close.
#[test]
fn a_dialog_is_named_by_its_title() {
    let harness = Harness::new(|ctx| {
        Dialog::new("Go to").show(ctx, |ui| {
            ui.label("Inside");
        });
    });
    let dialog = harness.get_by_role_and_label(Role::Dialog, "Go to");
    assert!(dialog.accesskit_node().data().is_modal());
    harness.get_by_role_and_label(Role::Button, "Close");
}

/// The palette's field is named by its hint, and each action by its label.
#[test]
fn the_palette_names_its_field_and_its_actions() {
    let choices = [
        Choice { label: "New email".into(), key: Some("Ctrl+N".into()) },
        Choice { label: "Settings".into(), key: None },
    ];
    let mut palette = CommandPalette::default();
    palette.open();
    let harness = Harness::new(|ctx| {
        palette.show(ctx, "Ctrl+K", &choices);
    });
    harness.get_by_role_and_label(Role::TextInput, "Type what to do");
    assert_eq!(toggled(&harness.get_by_role_and_label(Role::Button, "New email")), Some(Toggled::True));
    assert_eq!(toggled(&harness.get_by_role_and_label(Role::Button, "Settings")), Some(Toggled::False));
}
