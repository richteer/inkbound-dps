mod settings;

use std::collections::HashMap;

pub use settings::*;

mod individual_damage;
pub use individual_damage::*;

mod group_damage;
pub use group_damage::*;

mod history;
pub use history::*;

use serde::{Deserialize, Serialize};
use inkbound_parser::parser::{DataLog, PlayerStats};

use crate::OverlayOptions;

pub type WindowId = String;

#[typetag::serde(tag = "type")]
pub trait WindowDisplay: std::fmt::Debug {
    fn show(&mut self, ui: &mut egui::Ui, options: &OverlayOptions, data: &DataLog);

    /// The name of the window, to be used in the title bar and probably a window list
    fn name(&self) -> String;
}

#[inline]
pub fn inverted_number_label(current: usize, total: usize) -> String {
    format!("{}{}", total - current, if current == 0 {
        " (current)"
    } else {
        ""
    })
}

#[derive(Serialize, Deserialize, Debug)]
pub struct OverlayWindow {
    pub id: WindowId,
    pub window: Box<dyn WindowDisplay>,
}

impl OverlayWindow {
    pub fn new<T: WindowDisplay + Default + 'static>() -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            window: Box::new(T::default()),
        }
    }

    pub fn from_window(window: Box<dyn WindowDisplay>) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            window,
        }
    }

    pub fn id(&self) -> WindowId {
        self.id.clone()
    }
}

// Convenience passthrough, since most operations will really be on the window anyway
#[typetag::serde]
impl WindowDisplay for OverlayWindow {
    fn show(&mut self,ui: &mut egui::Ui,options: &OverlayOptions,data: &DataLog) {
        self.window.show(ui, options, data);
    }

    fn name(&self) -> String {
        self.window.name()
    }
}

// TODO: Remove this eventually, probably when options are derived almost entirely from traits
pub fn show_dive_selection_box(ui: &mut egui::Ui, dive_state: &mut usize, num_dives: usize) {
    egui::ComboBox::from_label("Select Dive")
        .selected_text(inverted_number_label(*dive_state, num_dives))
        .show_ui(ui, |ui| {
            for dive in 0..num_dives {
                ui.selectable_value(dive_state, dive, inverted_number_label(dive, num_dives));
            }
        });
}


/// Helper Enum for windows that want to select between per-combat and per-dive modes
#[derive(Default, Deserialize, Serialize, Debug, PartialEq)]
pub enum DamageTotalsMode {
    #[default]
    Dive,
    Combat,
}

impl std::fmt::Display for DamageTotalsMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            DamageTotalsMode::Dive => "Dive",
            DamageTotalsMode::Combat => "Combat",
        })
    }
}

#[derive(Debug, Default)]
pub struct DiveCombatSelectionState {
    pub dive: usize,
    pub combat: usize,
}

// TODO: consider making this two seperate traits, and create combiner auto-impl'd traits to define the fancier stuff
// Not really sure why the 'static lifetime is needed here
pub trait DiveCombatSplit: WindowDisplay + Default + 'static {
    /// Return a mutable reference to however the window is storing the mode state
    fn mode<'a>(&'a mut self) -> &'a mut DamageTotalsMode;
    /// Set the mode for the window, this is most likely only used by initialization
    fn set_mode(&mut self, mode: DamageTotalsMode);
    /// Get a mutable reference to the selected dive/combat state
    fn state<'a>(&'a mut self) -> &'a mut DiveCombatSelectionState;

    fn mode_selection(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
           ui.selectable_value(self.mode(), DamageTotalsMode::Dive, DamageTotalsMode::Dive.to_string());
           ui.selectable_value(self.mode(), DamageTotalsMode::Combat, DamageTotalsMode::Combat.to_string());
        });
    }

    fn window_from_mode(mode: DamageTotalsMode) -> OverlayWindow {
        let mut window = Self::default();
        window.set_mode(mode);
        OverlayWindow::from_window(Box::new(window))
    }

    fn show_dive_selection_box(&mut self, ui: &mut egui::Ui, data: &DataLog) {
        let dive_state = &mut self.state().dive;
        let num_dives = data.dives.len();

        egui::ComboBox::from_label("Select Dive")
            .selected_text(inverted_number_label(*dive_state, num_dives))
            .show_ui(ui, |ui| {
                for dive in 0..num_dives {
                    ui.selectable_value(dive_state, dive, inverted_number_label(dive, num_dives));
                }
            });
    }

    fn show_combat_selection_box(&mut self, ui: &mut egui::Ui, data: &DataLog) {
        let dive_num = self.state().dive;
        let combat_state = &mut self.state().combat;
        let num_combats = if let Some(dive) = data.dives.get(dive_num) {
            dive.combats.len()
        } else {
            0
        };

        egui::ComboBox::from_label("Select Combat")
            .selected_text(inverted_number_label(*combat_state, num_combats))
            .show_ui(ui, |ui| {
                for dive in 0..num_combats {
                    ui.selectable_value(combat_state, dive, inverted_number_label(dive, num_combats));
                }
            });
    }

    fn show_selection_boxes(&mut self, ui: &mut egui::Ui, data: &DataLog) {
        match self.mode() {
            DamageTotalsMode::Dive => self.show_dive_selection_box(ui, data),
            DamageTotalsMode::Combat => {
                self.show_dive_selection_box(ui, data);
                self.show_combat_selection_box(ui, data);
            },
        }
    }

    fn get_current_player_stat_list<'a>(&mut self, data: &'a DataLog) -> Option<&'a HashMap<String, PlayerStats>> {
        match self.mode() {
            DamageTotalsMode::Dive => data.dives.get(self.state().dive).and_then(|d| Some(&d.player_stats.player_stats)),
            DamageTotalsMode::Combat => {
                let state = self.state();
                if let Some(dive) = data.dives.get(state.dive) {
                    dive.combats.get(state.combat).and_then(|c| Some(&c.player_stats.player_stats))
                } else {
                    None
                }
            },
        }
    }
}

pub trait PlayerSelection {
    /// Get a mutable reference to how the window is storing the player state
    fn player<'a>(&'a mut self) -> &'a mut Option<String>;

    fn show_player_selection_box(&mut self, ui: &mut egui::Ui, player_stats: &HashMap<String, PlayerStats>) {
        let player = self.player();
        egui::ComboBox::from_label("Select Player")
                    .selected_text(format!("{}", player.as_ref().unwrap_or(&"".to_string())))
                    .show_ui(ui, |ui| {
                        // Assumes None -> pov character. Probably could be improved, especially if POV detection fails
                        ui.selectable_value(player, None, "YOU");
                        for pkey in player_stats.keys() {
                            ui.selectable_value(player, Some(pkey.clone()), pkey);
                        }
                    }
                );
    }
}

pub trait PlayerDiveCombatOptions: DiveCombatSplit + PlayerSelection {
    fn show_options(&mut self, ui: &mut egui::Ui, data: &DataLog) {
        ui.collapsing("⛭", |ui| {
            let player_stats = self.get_current_player_stat_list(data);

            self.mode_selection(ui);
            self.show_selection_boxes(ui, data);

            if let Some(player_stats) = player_stats {
                self.show_player_selection_box(ui, player_stats);
            }
        });
    }
}

// Automatically implement PlayerDiveCombatOptions for windows that implement both dive/combat and player selection
impl<T> PlayerDiveCombatOptions for T where T: DiveCombatSplit + PlayerSelection {}

static NO_DATA_MSG: &'static str = "Waiting for data...";
