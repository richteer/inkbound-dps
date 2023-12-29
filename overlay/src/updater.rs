use updater::{UpdateStatus, VersionStatus};

use crate::Overlay;

#[derive(Default)]
pub struct UpdateState {
    cache: egui_commonmark::CommonMarkCache,
    full_cache: egui_commonmark::CommonMarkCache,
    show_full: bool,
}

pub fn updater_settings(ui: &mut egui::Ui, overlay: &mut Overlay) {
    ui.checkbox(&mut overlay.options.auto_check_update, "Check for update on start")
        .on_hover_text("Check for an update when the application loads.\nThis will NOT automatically apply, you will still need to click the update button.");

    let mut updater = updater::UPDATER.lock().unwrap();
    let status = updater.status.lock().unwrap().clone();
    match status {
        UpdateStatus::Idle => {
            ui.horizontal(|ui| {
                if ui.button("Check for update").clicked() {
                    updater.fetch_update(false);
                }
                version_label(ui, overlay, format!("v{}", updater.current_version));
            });
        },
        UpdateStatus::Fetching => {
            ui.horizontal(|ui| {
                ui.spinner();
                ui.label("Fetching version information");
            });
        },
        UpdateStatus::Fetched(VersionStatus::Update(version, body)) => {
            ui.horizontal(|ui| {
                if ui.button("Update").clicked() {
                    updater.do_update(false);
                }
                let current_version = updater.current_version.clone();
                // TODO: consider adding full changelog here too?
                let new_version = ui.colored_label(egui::Color32::GREEN, format!("New version available! {current_version} ➡ {version}"));
                if let Some(body) = body {
                    new_version.on_hover_ui(|ui| {
                        egui::ScrollArea::vertical().max_height(400.0).show(ui, |ui| {
                            egui_commonmark::CommonMarkViewer::new("HoverChangelog").show(ui, &mut overlay.window_state.update.cache, &body);
                        });
                    });
                }
            });
        },
        // Basically the same as Idle, with additional text
        UpdateStatus::Fetched(VersionStatus::UpToDate) => {
            ui.horizontal(|ui| {
                if ui.button("Check for update").clicked() {
                    updater.fetch_update(false);
                }
                version_label(ui, overlay, format!("v{} - Up to date.", updater.current_version));
            });
        },
        UpdateStatus::Updating => {
            ui.horizontal(|ui| {
                ui.spinner();
                ui.label("Updating...");
            });
        },
        UpdateStatus::Updated => { ui.colored_label(egui::Color32::GREEN, "Updated! Restart to use new version."); },
        UpdateStatus::Error(error) => { ui.colored_label(egui::Color32::RED, format!("Error updating: {error}")); },
    };

    if overlay.window_state.update.show_full {
        show_full_changelog(ui, overlay);
    }
}

fn version_label(ui: &mut egui::Ui, overlay: &mut Overlay, label: String) {
    let version = egui::Label::new(label).sense(egui::Sense::click());
    let version = ui.add(version).on_hover_text("Click to show changelog");
    if version.clicked() {
        overlay.window_state.update.show_full = true;
    }
}

fn show_full_changelog(ui: &mut egui::Ui, overlay: &mut Overlay) {
    egui::Window::new("Changelog").open(&mut overlay.window_state.update.show_full).show(ui.ctx(), |ui| {
        egui::ScrollArea::vertical().show(ui, |ui| {
            egui_commonmark::CommonMarkViewer::new("FullChangelog").show(ui, &mut overlay.window_state.update.full_cache, include_str!("../../CHANGELOG.md"));
        });
    });
}
