use std::collections::BTreeMap;

use egui::{Window, Color32};
use inkbound_parser::aspects::Aspect;
use strum::{IntoEnumIterator, EnumIter};

use crate::{Overlay, DefaultColor};

use super::{WindowDisplay, OverlayWindow, WindowId};

#[derive(Debug, Default)]
pub enum HighlightWindow {
    #[default]
    None,
    Toggle(WindowId),
    Delete(WindowId),
}

impl HighlightWindow {
    pub fn inner_eq(&self, other: &WindowId) -> bool {
        match self {
            HighlightWindow::None => false,
            HighlightWindow::Toggle(wid) => wid == other,
            HighlightWindow::Delete(wid) => wid == other,
        }
    }
}

#[derive(Default)]
pub struct SettingsState {
    add_window: Option<AddWindowChoice>,
    pub highlight_window: HighlightWindow,
}

#[derive(EnumIter, Debug, PartialEq, Eq)]
enum AddWindowChoice {
    GroupStats,
    SkillTotals,
    History,
    StatTable,
}

impl std::fmt::Display for AddWindowChoice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            // TODO: Consider defining these in the mod so they can be reused by the window title too or something
            AddWindowChoice::GroupStats => "Group Stats",
            AddWindowChoice::SkillTotals => "Skill Totals",
            AddWindowChoice::History => "History",
            AddWindowChoice::StatTable => "Table",
        })
    }
}

impl From<&AddWindowChoice> for OverlayWindow {
    fn from(value: &AddWindowChoice) -> Self {
        use super::*;
        match value {
            AddWindowChoice::GroupStats => OverlayWindow::new::<GroupStatsWindow>(),
            AddWindowChoice::SkillTotals => OverlayWindow::new::<SkillTotalsWindow>(),
            AddWindowChoice::History => OverlayWindow::new::<HistoryWindow>(),
            AddWindowChoice::StatTable => OverlayWindow::new::<StatTableWindow>(),
        }
    }
}

pub fn draw_settings_window(overlay: &mut Overlay, ctx: &egui::Context) {
    Window::new("Settings")
        .show(ctx, |ui| {
            ui.heading("Windows");

            overlay.window_state.settings.highlight_window = HighlightWindow::None;
            // Prep the queue for deleting windows
            let mut delete_windows = Vec::new();
            // Pre-sort by name, purely for aesthetic reasons
            let mut sorted_windows = overlay.windows.iter().collect::<Vec<(&String, &OverlayWindow)>>();
            sorted_windows.sort_by_key(|f| f.1.name());
            for (id, window) in sorted_windows.into_iter() {
                ui.horizontal(|ui| {
                    let delbutton = egui::Label::new("♻").sense(egui::Sense::click());
                    let delbutton = ui.add(delbutton);
                    if delbutton.clicked() {
                        delete_windows.push(id.clone());
                    }
                    if delbutton.hovered() {
                        overlay.window_state.settings.highlight_window = HighlightWindow::Delete(id.clone());
                    }
                    let mut open = overlay.enabled_windows.contains(id);
                    let cbox = ui.checkbox(&mut open, window.name());
                    if cbox.clicked() {
                        match open {
                            true => overlay.enabled_windows.insert(id.clone()),
                            false => overlay.enabled_windows.remove(id),
                        };
                    };
                    if cbox.hovered() {
                        overlay.window_state.settings.highlight_window = HighlightWindow::Toggle(id.clone());
                    }
                });
            }

            // Consider having the overlay update method do this if there are more delayed commands to implement
            for wid in delete_windows.into_iter() {
                overlay.windows.remove(&wid);
                overlay.enabled_windows.remove(&wid);
            }

            ui.horizontal(|ui| {
                if ui.button("➕").clicked() {
                    if let Some(win) = &overlay.window_state.settings.add_window {
                        let empty: OverlayWindow = win.into();
                        overlay.enabled_windows.insert(empty.id());
                        overlay.windows.insert(empty.id(), empty);
                    }
                }

                let selected_text = match &overlay.window_state.settings.add_window {
                    Some(win) => win.to_string(),
                    None => String::new(),
                };
                egui::ComboBox::from_label("Add Window")
                    .selected_text(selected_text)
                    .show_ui(ui, |ui| {
                        for win in AddWindowChoice::iter() {
                            let text = win.to_string();
                            ui.selectable_value(&mut overlay.window_state.settings.add_window, Some(win), text);
                        }
                    });
            });
            ui.separator();
            if ui.button("Reset Default Windows").clicked() {
                overlay.windows = crate::overlay::default_windows();
            }

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

            #[cfg(feature = "auto_update")]
            {
                ui.separator();
                crate::updater::updater_settings(ui, overlay);
            }

            ui.separator();
            ui.horizontal(|ui| {
                if ui.button("Close Overlay").clicked() {
                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                }
                if ui.button("Restart Parser").clicked() {
                    overlay.logreader.reset();
                }
                ui.separator();
                let status = overlay.logreader.get_status();
                match status {
                    logreader::LogReaderStatus::Initializing => {
                        ui.label(format!("{status}"));
                        ui.spinner()
                    },
                    logreader::LogReaderStatus::Errored => ui.colored_label(egui::Rgba::RED, format!("{status}")),
                    _ => ui.label(format!("{status}")),
                };
            });

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
