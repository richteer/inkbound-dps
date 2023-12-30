mod settings;

use std::collections::HashMap;

pub use settings::*;

mod skill_totals;
pub use skill_totals::*;

mod group_stats;
pub use group_stats::*;

mod history;
pub use history::*;

mod stat_table;
pub use stat_table::*;

pub mod extractors;

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

/// Divide two numbers. If the result isn't a normal number, return zero instead.
fn div_or_zero(x: f64, y: f64) -> f64 {
    let ret = x / y;
    if ret.is_normal() {
        ret
    } else {
        0.0
    }
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
            window: Box::<T>::default(),
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
pub enum DiveCombatSelection {
    #[default]
    Dive,
    Combat,
}

impl std::fmt::Display for DiveCombatSelection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            DiveCombatSelection::Dive => "Dive",
            DiveCombatSelection::Combat => "Combat",
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
    fn mode(&mut self) -> &mut DiveCombatSelection;
    /// Set the mode for the window, this is most likely only used by initialization
    fn set_mode(&mut self, mode: DiveCombatSelection);
    /// Get a mutable reference to the selected dive/combat state
    fn state(&mut self) -> &mut DiveCombatSelectionState;

    fn mode_selection(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
           ui.selectable_value(self.mode(), DiveCombatSelection::Dive, DiveCombatSelection::Dive.to_string());
           ui.selectable_value(self.mode(), DiveCombatSelection::Combat, DiveCombatSelection::Combat.to_string());
        });
    }

    fn window_from_mode(mode: DiveCombatSelection) -> OverlayWindow {
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
            DiveCombatSelection::Dive => self.show_dive_selection_box(ui, data),
            DiveCombatSelection::Combat => {
                self.show_dive_selection_box(ui, data);
                self.show_combat_selection_box(ui, data);
            },
        }
    }

    fn get_current_player_stat_list<'a>(&mut self, data: &'a DataLog) -> Option<&'a HashMap<String, PlayerStats>> {
        match self.mode() {
            DiveCombatSelection::Dive => data.dives.get(self.state().dive).map(|d| &d.player_stats.player_stats),
            DiveCombatSelection::Combat => {
                let state = self.state();
                if let Some(dive) = data.dives.get(state.dive) {
                    dive.combats.get(state.combat).map(|c| &c.player_stats.player_stats)
                } else {
                    None
                }
            },
        }
    }
}

pub trait PlayerSelection {
    /// Get a mutable reference to how the window is storing the player state
    fn player(&mut self) -> &mut Option<String>;

    fn show_player_selection_box(&mut self, ui: &mut egui::Ui, player_stats: &HashMap<String, PlayerStats>) {
        let player = self.player();
        egui::ComboBox::from_label("Select Player")
                    .selected_text(player.as_ref().unwrap_or(&"".to_string()).to_string())
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

static FORMAT_SELECTION_HELP: &str =
"Enter a text formatter string, and all valid patterns contained in {braces} will be filled with its respective value.
Any other characters will be printed verbatim. See the box below for a list of valid patterns.
    NOTE: entering an invalid {pattern} will cause the whole format string to fail.

Some fmt-like specifications may also be used.
For example, to print a float with only two decimal places, use:
{value:.2}
";
pub trait FormatSelection {
    fn get_format(&mut self) -> &mut String;
    fn default_format() -> &'static str;
    fn hover_text() -> &'static str;

    fn show_format_selection_box(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label("ℹ")
                .on_hover_text(FORMAT_SELECTION_HELP)
                .on_hover_text(Self::hover_text());
            ui.text_edit_singleline(self.get_format());
            let reset = egui::Label::new("⟲").sense(egui::Sense::click());
            if ui.add(reset).on_hover_text("Reset to default.").clicked() {
                *self.get_format() = Self::default_format().to_string();
            }
        });
    }
}

// TODO: Consider just putting this in parser Aspect?
pub trait AspectAbbv {
    fn abbv(&self) -> String;
}

use inkbound_parser::aspects::Aspect;
impl AspectAbbv for Aspect {
    /// Get an abbreviated name for the aspect.
    // TODO: consider allowing users to define these?
    fn abbv(&self) -> String {
        match self {
            Aspect::MagmaMiner => "MGM",
            Aspect::Mosscloak => "MSC",
            Aspect::Weaver => "WVR",
            Aspect::Obelisk => "OBE",
            Aspect::Clairvoyant => "CLV",
            Aspect::StarCaptain => "STC",
            Aspect::Chainbreaker => "CHB",
            Aspect::Godkeeper => "GKR",
            Aspect::Unknown(_) => "UNK",
        }.to_string()
    }
}

static NO_DATA_MSG: &str = "Waiting for data...";
