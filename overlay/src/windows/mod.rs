mod settings;

pub use settings::*;

mod individual_damage;
pub use individual_damage::*;

mod group_damage;
pub use group_damage::*;

mod history;
pub use history::*;

use serde::{Deserialize, Serialize};
use inkbound_parser::parser::DataLog;

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

// Not really sure why the 'static lifetime is needed here
pub trait DiveCombatSplit: WindowDisplay + Default + 'static {
    /// Return a mutable reference to however the window is storing the mode state
    fn mode<'a>(&'a mut self) -> &'a mut DamageTotalsMode;
    /// Set the mode for the window, this is most likely only used by initialization
    fn set_mode(&mut self, mode: DamageTotalsMode);

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
}

static NO_DATA_MSG: &'static str = "Waiting for data...";
