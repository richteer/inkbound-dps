use inkbound_parser::aspects::Aspect;
use serde::{Serialize, Deserialize};

use crate::{windows, DefaultColor};

#[derive(Serialize, Deserialize)]
#[serde(default)]
pub struct OverlayOptions {
    pub show_dive_group_damage: bool,
    pub show_combat_group_damage: bool,
    pub show_dive_individual_damage: bool,
    pub show_combat_individual_damage: bool,
    pub history: windows::HistoryOptions,
    pub default_player_name: String,
    pub plot_font_size: f32,
    pub show_crit_bars: bool,
    pub crit_bar_opacity: u8,
    pub colors: ColorOptions,
}

// TODO: consider using a crate to make this whole impl not necessary
impl Default for OverlayOptions {
    fn default() -> Self {
        Self {
            show_dive_group_damage: false,
            show_combat_group_damage: false,
            show_dive_individual_damage: false,
            show_combat_individual_damage: false,
            history: windows::HistoryOptions::default(),
            default_player_name: "".to_string(),
            plot_font_size: 14.0,
            show_crit_bars: false,
            crit_bar_opacity: 128,
            colors: ColorOptions::default(),
        }
    }
}

#[derive(Serialize, Deserialize, Default)]
pub struct ColorOptions {
    pub aspects: std::collections::BTreeMap<Aspect, egui::Color32>,
}

impl ColorOptions {
    pub fn get_aspect_color(&self, aspect: &Aspect) -> egui::Color32 {
        if let Some(color) = self.aspects.get(aspect) {
            *color
        } else {
            aspect.default_color()
        }
    }
}
