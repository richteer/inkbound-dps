use serde::{Serialize, Deserialize};

use crate::windows;

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
        }
    }
}

