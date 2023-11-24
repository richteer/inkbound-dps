use std::sync::{Arc, RwLock};

use inkbound_parser::parser::DataLog;

use crate::windows;

#[derive(Default)]
pub struct WindowState {
    pub dive_group_damage: windows::GroupDamageState,
    pub combat_group_damage: windows::GroupDamageState,
    pub dive_individual_damage: windows::IndividualDamageState,
    pub combat_individual_damage: windows::IndividualDamageState,
}

// #[derive(Default)]
pub struct Overlay {
    pub datalog: Arc<RwLock<DataLog>>,
    pub window_state: WindowState,
    pub closing: bool,
    pub plot_font_size: f32,
}

impl Overlay {
    pub fn new(_cc: &eframe::CreationContext<'_>, datalog: Arc<RwLock<DataLog>>) -> Self {
        // Customize egui here with cc.egui_ctx.set_fonts and cc.egui_ctx.set_visuals.
        // Restore app state using cc.storage (requires the "persistence" feature).
        // Use the cc.gl (a glow::Context) to create graphics shaders and buffers that you can use
        // for e.g. egui::PaintCallback.
        _cc.egui_ctx.set_visuals(egui::style::Visuals {
            // panel_fill: egui::Color32::TRANSPARENT,
            // window_fill: egui::Color32::TRANSPARENT,
            ..Default::default()
        });

        Self {
            datalog,
            // TODO: probably persist this information
            window_state: WindowState::default(),
            closing: false,
            plot_font_size: 14.0,
        }
    }
}

/// Main entrypoint to draw all the egui widgets and things
pub fn draw_overlay(overlay: &mut Overlay, ctx: &egui::Context) {
    let datalog = {
        // TODO: This seems very wasteful of memory, consider doing some kind of cache
        //  update clone only if data changed, etc
        overlay.datalog.read().unwrap().clone()
    };
    windows::main_menu(overlay, ctx);
    windows::draw_dive_damage_window(overlay, ctx, &datalog);
    windows::draw_combat_damage_window(overlay, ctx, &datalog);
    windows::draw_dive_individual_damage_window(overlay, ctx, &datalog);
    windows::draw_combat_individual_damage_window(overlay, ctx, &datalog);
}


/// Entrypoint for the main application to spawn the actual overlay window and such
pub fn spawn_overlay(datalog: Arc<RwLock<DataLog>>) {
    crate::eframe_impl::spawn_overlay(datalog);
}

