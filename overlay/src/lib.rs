mod windows;
mod overlay;
pub use overlay::*;
mod options;
pub use options::OverlayOptions;

use inkbound_parser::aspects::Aspect;

trait DefaultColor {
    fn default_color(&self) -> egui::Color32;
}

// Out of scope for the parser, so plopping it here
impl DefaultColor for Aspect {
    /// Get the default color for a given aspect
    fn default_color(&self) -> egui::Color32 {
        use egui::Color32;
        match self {
            Aspect::MagmaMiner   => Color32::from_rgb(184, 67, 0),
            Aspect::Mosscloak    => Color32::from_rgb(76, 142, 33),
            Aspect::Clairvoyant  => Color32::from_rgb(194, 66, 66),
            Aspect::Weaver       => Color32::from_rgb(151, 30, 167),
            Aspect::Obelisk      => Color32::from_rgb(55, 147, 147),
            Aspect::StarCaptain  => Color32::from_rgb(188, 150, 53),
            Aspect::Chainbreaker => Color32::from_rgb(137, 26, 37),
            Aspect::Godkeeper    => Color32::from_rgb(213, 123, 22),
            Aspect::Unknown(_)   => Color32::DARK_GRAY,
        }
    }
}
