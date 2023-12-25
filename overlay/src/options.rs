use inkbound_parser::aspects::Aspect;
use serde::{Serialize, Deserialize};

use crate::DefaultColor;

#[derive(Serialize, Deserialize)]
#[serde(default)]
pub struct OverlayOptions {
    pub plot_font_size: f32,
    pub show_crit_bars: bool,
    pub crit_bar_opacity: u8,
    pub colors: ColorOptions,
    pub auto_check_update: bool,
}

// TODO: consider using a crate to make this whole impl not necessary
impl Default for OverlayOptions {
    fn default() -> Self {
        Self {
            plot_font_size: 14.0,
            show_crit_bars: false,
            crit_bar_opacity: 128,
            colors: ColorOptions::default(),
            auto_check_update: false,
        }
    }
}

#[derive(Serialize, Deserialize, Default)]
#[serde(default)]
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
