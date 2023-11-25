use egui::Window;

use crate::Overlay;


pub fn main_menu(overlay: &mut Overlay, ctx: &egui::Context) {
    Window::new("Main Menu")
        // .resizable(false)
        .show(ctx, |ui| {
            ui.heading("Dive Overlays");
            ui.checkbox(&mut overlay.options.show_dive_group_damage, "Show Group Damage");
            ui.checkbox(&mut overlay.options.show_dive_individual_damage, "Show Individual Damage");
            ui.separator();
            ui.heading("Combat Overlays");
            ui.checkbox(&mut overlay.options.show_combat_group_damage, "Show Group Damage");
            ui.checkbox(&mut overlay.options.show_combat_individual_damage, "Show Individual Damage");

            ui.separator();
            ui.horizontal(|ui| {
                ui.label("Default Player Name");
                ui.text_edit_singleline(&mut overlay.options.default_player_name);
            });
            ui.add(egui::Slider::new(&mut overlay.options.plot_font_size, 4.0..=32.0).text("Plot font size"));

            ui.separator();
            if ui.button("Close Overlay").clicked() {
                overlay.closing = true;
            }
            // ui.set_width(ui.available_width());
            // ui.set_height(ui.available_height());
        }
    );
}
