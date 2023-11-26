use egui::Window;

use crate::Overlay;


pub fn draw_settings_window(overlay: &mut Overlay, ctx: &egui::Context) {
    Window::new("Settings")
        .show(ctx, |ui| {
            ui.heading("Dive Overlays");
            ui.checkbox(&mut overlay.options.show_dive_group_damage, "Show Group Damage")
                .on_hover_text("Enable a plot showing the overall damage per player in a particular dive.");
            ui.checkbox(&mut overlay.options.show_dive_individual_damage, "Show Individual Damage")
                .on_hover_text("Enable a plot showing the damage per skill for a player in a dive.");
            ui.checkbox(&mut overlay.options.history.show, "Show History")
                .on_hover_text("Enable a plot that shows the history of a dive per each combat");
            ui.separator();
            ui.heading("Combat Overlays");
            ui.checkbox(&mut overlay.options.show_combat_group_damage, "Show Group Damage")
                .on_hover_text("Enable a plot showing the overall damage per player for a single combat encounter.");
            ui.checkbox(&mut overlay.options.show_combat_individual_damage, "Show Individual Damage")
                .on_hover_text("Enable a plot showing the damage per skill for a player in a single combat encounter.");


            ui.separator();
            ui.horizontal(|ui| {
                ui.label("Default Player Name");
                ui.text_edit_singleline(&mut overlay.options.default_player_name);
            }).response.on_hover_text("Enter a player name to set as the default if no player is selected in an individual damage window.\n\nYou probably want to set this to your in-game character name.");
            ui.add(egui::Slider::new(&mut overlay.options.plot_font_size, 4.0..=32.0).text("Plot font size"))
                .on_hover_text("Set the font size for the damage plot labels.");

            ui.separator();
            if ui.button("Close Overlay").clicked() {
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            }
        }
    );
}
