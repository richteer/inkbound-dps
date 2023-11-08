use std::sync::{Arc, RwLock};

use inkbound_parser::parser::DataLog;

use crate::windows;


#[derive(Default)]
pub struct EnabledWindows {
    pub show_dive_group_damage: bool,
    pub show_dive_individual_damage: bool,
    pub show_combat_group_damage: bool,
    pub show_combat_individual_damage: bool,
}


// #[derive(Default)]
pub struct Overlay {
    pub datalog: Arc<RwLock<DataLog>>,
    pub dive_individual_selection: Option<String>,
    pub combat_individual_selection: Option<String>,
    pub enabled_windows: EnabledWindows,
    pub closing: bool,
}

impl Overlay {
    #[cfg(feature = "use_eframe")]
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

        Self::do_new(datalog)
    }

    #[cfg(feature = "use_egui_overlay")]
    pub fn new(datalog: Arc<RwLock<DataLog>>) -> Self {
        Self::do_new(datalog)
    }

    fn do_new(datalog: Arc<RwLock<DataLog>>) -> Self {
        Self {
            datalog,
            dive_individual_selection: None,
            combat_individual_selection: None,
            // TODO: probably persist this information
            enabled_windows: EnabledWindows::default(),
            closing: false,
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
    #[cfg(feature = "use_eframe")]
    {
        log::debug!("using eframe-based backend");
        crate::eframe_impl::spawn_overlay(datalog);
    }
    #[cfg(feature = "use_egui_overlay")]
    {
        log::debug!("using egui_overlay-based backend");
        crate::egui_overlay_impl::spawn_overlay(datalog);
    }
}

