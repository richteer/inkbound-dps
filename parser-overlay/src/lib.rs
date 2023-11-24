mod windows;
mod overlay;
pub use overlay::*;

mod eframe_impl;

// TODO: Make this configurable and save to layout state
fn class_string_to_color(class_name: &str) -> egui::Color32 {
    // TODO: use a complete lookup table so this only needs to be updated in one place
    match class_name {
        "Magma Miner"  => egui::Color32::from_rgb(196, 75, 0),
        "Mosscloak"    => egui::Color32::from_rgb(0, 130, 13),
        "Clairvoyant"  => egui::Color32::from_rgb(130, 0, 0),
        "Weaver"       => egui::Color32::from_rgb(173, 148, 0),
        "Obelisk"      => egui::Color32::from_rgb(128, 128, 128),
        "Star Captain" => egui::Color32::from_rgb(0, 119, 140),
        "Chainbreaker" => egui::Color32::from_rgb(107, 0, 52),
        "Godseeker" => egui::Color32::from_rgb(150, 131, 2),
        _ => egui::Color32::DARK_GRAY,
    }
}


