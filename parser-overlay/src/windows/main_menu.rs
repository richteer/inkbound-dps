use egui::Window;

use crate::Overlay;


pub fn main_menu(overlay: &mut Overlay, ctx: &egui::Context) {
    Window::new("Main Menu")
        // .resizable(false)
        .show(ctx, |ui| {
            ui.heading("Dive Overlays");
            ui.checkbox(&mut overlay.window_state.dive_group_damage.show, "Show Group Damage");
            ui.checkbox(&mut overlay.window_state.dive_individual_damage.show, "Show Individual Damage");
            ui.separator();
            ui.heading("Combat Overlays");
            ui.checkbox(&mut overlay.window_state.combat_group_damage.show, "Show Group Damage");
            ui.checkbox(&mut overlay.window_state.combat_individual_damage.show, "Show Individual Damage");
            ui.separator();
            if ui.button("Close Overlay").clicked() {
                overlay.closing = true;
            }
            // ui.set_width(ui.available_width());
            // ui.set_height(ui.available_height());
        }
    );
}
