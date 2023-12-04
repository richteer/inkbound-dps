use std::collections::BTreeMap;

use egui::{Window, Color32};
use inkbound_parser::aspects::Aspect;
use strum::IntoEnumIterator;

use crate::{Overlay, DefaultColor};


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
            ui.checkbox(&mut overlay.options.show_crit_bars, "Show Crit Bars")
                .on_hover_text("Show what portion of a skill's damage was done via critical hits.\n\nThis will appear as a darker portion of a skill's damage bar in individual damage windows.");
            if overlay.options.show_crit_bars {
                ui.add(egui::Slider::new(&mut overlay.options.crit_bar_opacity, 1..=255).text("Crit Bar Opacity"))
                    .on_hover_text("Set the opacity of the overlain crit bar.");
            }
            ui.checkbox(&mut overlay.window_state.color_settings.show, "Show Color Editor");

            ui.separator();
            ui.checkbox(&mut overlay.options.auto_check_update, "Check for update on start")
                .on_hover_text("Check for an update when the application loads.\nThis will NOT automatically apply, you will still need to click the update button.");

            {
                use updater::{UpdateStatus, VersionStatus};

                let mut updater = updater::UPDATER.lock().unwrap();
                let status = updater.status.lock().unwrap().clone();
                match status {
                    UpdateStatus::Idle => {
                        ui.horizontal(|ui| {
                            if ui.button("Check for update").clicked() {
                                updater.fetch_update(false);
                            }
                            ui.label(format!("v{}", updater.current_version));
                        });
                    },
                    UpdateStatus::Fetching => {
                        ui.horizontal(|ui| {
                            ui.spinner();
                            ui.label("Fetching version information");
                        });
                    },
                    UpdateStatus::Fetched(VersionStatus::Update(version)) => {
                        ui.horizontal(|ui| {
                            if ui.button("Update").clicked() {
                                updater.do_update(false);
                            }
                            let current_version = updater.current_version.clone();
                            ui.colored_label(egui::Color32::GREEN, format!("New version available! {current_version} ➡ {version}"));
                        });
                    },
                    // Basically the same as Idle, with additional text
                    UpdateStatus::Fetched(VersionStatus::UpToDate) => {
                        ui.horizontal(|ui| {
                            if ui.button("Check for update").clicked() {
                                updater.fetch_update(false);
                            }
                            ui.label(format!("v{} - Up to date.", updater.current_version));
                        });
                    },
                    UpdateStatus::Updating => {
                        ui.horizontal(|ui| {
                            ui.spinner();
                            ui.label("Updating...");
                        });
                    },
                    UpdateStatus::Updated => { ui.colored_label(egui::Color32::GREEN, "Updated! Restart to use new version."); },
                    UpdateStatus::Error(error) => { ui.colored_label(egui::Color32::RED, format!("Error updating: {error}")); },
                };
            }

            ui.separator();
            if ui.button("Close Overlay").clicked() {
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            }

            if overlay.window_state.color_settings.show {
                draw_color_settings_window(overlay, ctx);
            }
        }
    );
}

pub struct ColorSettingsState {
    pub show: bool,
    pub aspects: BTreeMap<Aspect, Color32>,
}

impl Default for ColorSettingsState {
    fn default() -> Self {
        let aspects: BTreeMap<Aspect, Color32> = Aspect::iter().map(|elem| {
            let color = elem.default_color();
            (elem, color)
        }).collect();

        Self {
            show: false,
            aspects,
        }
    }
}

impl ColorSettingsState {
    pub fn sync_from_options(&mut self, options: &crate::OverlayOptions) {
        for (aspect, color) in options.colors.aspects.iter() {
            self.aspects.insert(aspect.clone(), *color);
        }
    }
}


pub fn draw_color_settings_window(overlay: &mut Overlay, ctx: &egui::Context) {
    Window::new("Color Settings")
        .open(&mut overlay.window_state.color_settings.show)
        .show(ctx, |ui| {
            for (aspect, color) in overlay.window_state.color_settings.aspects.iter_mut() {
                ui.horizontal(|ui| {
                    let mut cpicker = ui.color_edit_button_srgba(color);
                    let label = egui::widgets::Label::new("⟲")
                        .sense(egui::Sense::click());
                    let label = ui.add(label);
                    if label.clicked() {
                        *color = aspect.default_color();
                        overlay.options.colors.aspects.remove(&aspect); // Doesn't matter if it's not actually there
                        cpicker.mark_changed();
                    };
                    if cpicker.changed() {
                        overlay.options.colors.aspects.insert(aspect.clone(), *color);
                    }
                    cpicker.labelled_by(
                        ui.label(
                            if *aspect == Aspect::Unknown("".to_string()) {
                                "Unknown".to_string()
                            } else {
                                aspect.to_string()
                            }
                        ).id
                    );
                });
            }
        }
    );
}
