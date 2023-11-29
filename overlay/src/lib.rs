mod windows;
mod overlay;
pub use overlay::*;
mod options;
pub use options::OverlayOptions;

// TODO: Make this configurable and save to layout state
fn class_string_to_color(class_name: &str) -> egui::Color32 {
    // TODO: use a complete lookup table so this only needs to be updated in one place
    match class_name {
        "Magma Miner"  => egui::Color32::from_rgb(184, 67, 0),
        "Mosscloak"    => egui::Color32::from_rgb(76, 142, 33),
        "Clairvoyant"  => egui::Color32::from_rgb(194, 66, 66),
        "Weaver"       => egui::Color32::from_rgb(151, 30, 167),
        "Obelisk"      => egui::Color32::from_rgb(55, 147, 147),
        "Star Captain" => egui::Color32::from_rgb(188, 150, 53),
        "Chainbreaker" => egui::Color32::from_rgb(137, 26, 37),
        "Godkeeper"    => egui::Color32::from_rgb(213, 123, 22),
        _ => egui::Color32::DARK_GRAY,
    }
}


