use egui::Window;

use crate::Overlay;


pub fn main_menu(overlay: &mut Overlay, ctx: &egui::Context) {
    Window::new("Main Menu")
        // .resizable(false)
        .show(ctx, |ui| {
            ui.heading("Dive Overlays");
            ui.checkbox(&mut overlay.enabled_windows.show_dive_group_damage, "Show Group Damage");
            ui.checkbox(&mut overlay.enabled_windows.show_dive_individual_damage, "Show Individual Damage");
            ui.separator();
            ui.heading("Combat Overlays");
            ui.checkbox(&mut overlay.enabled_windows.show_combat_group_damage, "Show Group Damage");
            ui.checkbox(&mut overlay.enabled_windows.show_combat_individual_damage, "Show Individual Damage");
            ui.separator();
            if ui.button("Close Overlay").clicked() {
                overlay.closing = true;
            }
            // ui.set_width(ui.available_width());
            // ui.set_height(ui.available_height());
        }
    );
}
