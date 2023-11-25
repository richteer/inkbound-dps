mod main_menu;
pub use main_menu::*;

mod individual_damage;
pub use individual_damage::*;

mod group_damage;
pub use group_damage::*;

#[inline]
pub fn inverted_number_label(current: usize, total: usize) -> String {
    format!("{}{}", total - current, if current == 0 {
        " (current)"
    } else {
        ""
    })
}

pub fn show_dive_selection_box(ui: &mut egui::Ui, dive_state: &mut usize, num_dives: usize) {
    egui::ComboBox::from_label("Select Dive")
        .selected_text(inverted_number_label(*dive_state, num_dives))
        .show_ui(ui, |ui| {
            for dive in 0..num_dives {
                ui.selectable_value(dive_state, dive, inverted_number_label(dive, num_dives));
            }
        });
}

pub fn show_combat_selection_box(ui: &mut egui::Ui, combat_state: &mut usize, num_combats: usize) {
    egui::ComboBox::from_label("Select Combat")
        .selected_text(inverted_number_label(*combat_state, num_combats))
        .show_ui(ui, |ui| {
            for dive in 0..num_combats {
                ui.selectable_value(combat_state, dive, inverted_number_label(dive, num_combats));
            }
        });
}
