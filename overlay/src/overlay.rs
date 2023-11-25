use std::sync::{Arc, RwLock};

use egui::{Visuals, Pos2};
use inkbound_parser::parser::DataLog;
use serde::{Serialize, Deserialize};

use crate::windows;

static OPTIONS_STORAGE_KEY: &'static str = "overlayoptions";

#[derive(Default)]
pub struct WindowState {
    pub dive_group_damage: windows::GroupDamageState,
    pub combat_group_damage: windows::GroupDamageState,
    pub dive_individual_damage: windows::IndividualDamageState,
    pub combat_individual_damage: windows::IndividualDamageState,
}

#[derive(Serialize, Deserialize)]
pub struct OverlayOptions {
    pub show_dive_group_damage: bool,
    pub show_combat_group_damage: bool,
    pub show_dive_individual_damage: bool,
    pub show_combat_individual_damage: bool,
    pub default_player_name: String,
    pub plot_font_size: f32,
}

// TODO: consider using a crate to make this whole impl not necessary
impl Default for OverlayOptions {
    fn default() -> Self {
        Self {
            show_dive_group_damage: false,
            show_combat_group_damage: false,
            show_dive_individual_damage: false,
            show_combat_individual_damage: false,
            default_player_name: "".to_string(),
            plot_font_size: 14.0,
        }
    }
}

// #[derive(Default)]
pub struct Overlay {
    pub datalog: Arc<RwLock<DataLog>>,
    pub window_state: WindowState,
    pub options: OverlayOptions,
    pub closing: bool,
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

        let options = if let Some(storage) = _cc.storage {
            let options = storage.get_string(OPTIONS_STORAGE_KEY);
            if let Some(options) = options {
                match ron::from_str(options.as_str()) {
                    Ok(options) => options,
                    Err(e) => {
                        log::warn!("options parse error: {:?}", e);
                        log::warn!("failed to parse stored options, reverting to default options");
                        OverlayOptions::default()
                    }
                }
            } else {
                log::debug!("no options to load, using defaults");
                OverlayOptions::default()
            }
        } else {
            log::warn!("failed to load persistence storage, using default options");
            OverlayOptions::default()
        };

        Self {
            datalog,
            // TODO: probably persist this information
            window_state: WindowState::default(),
            options,
            closing: false,
        }
    }
}

impl eframe::App for Overlay {
    fn clear_color(&self, _visuals: &Visuals) -> [f32; 4] {
        egui::Rgba::TRANSPARENT.to_rgba_unmultiplied()
    }

    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        match ron::to_string(&self.options) {
            Ok(options) => storage.set_string(OPTIONS_STORAGE_KEY, options),
            Err(e) => log::error!("failed to serialize options: {:?}", e),
        }
    }

    // TODO: seriously consider preformatting the information on the parse thread
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {

        // Force maximize the window, for some reason setting in NativeOptions is ignored
        // Don't in debug though, just so it can be tested in a window better
        #[cfg(not(debug_assertions))]
        frame.set_maximized(true);

        // TODO: maybe don't run this every update?
        ctx.style_mut(|style| {
            style.text_styles.insert(egui::style::TextStyle::Small, egui::FontId { size: self.options.plot_font_size, family: egui::FontFamily::Proportional }).unwrap();
        });

        use mouse_position::mouse_position::Mouse;
        match Mouse::get_mouse_position() {
            Mouse::Position { x, y } => {
                let mut pointer_pos: Pos2 = [x as f32, y as f32].into();

                // Convert the absolute pointer position to egui points
                pointer_pos.x /= ctx.pixels_per_point();
                pointer_pos.y /= ctx.pixels_per_point();

                // Get window position in egui points
                // TODO: remove unwrap
                let window_pos = frame.info().window_info.position.unwrap();

                // Adjust absolute pointer pos to egui relative position
                pointer_pos -= window_pos.to_vec2();
                if ctx.is_pos2_over_area(pointer_pos) {
                    log::trace!("is over area, disabling passthrough");
                    frame.set_mouse_passthrough(false);
                } else {
                    log::trace!("enabling passthrough");
                    frame.set_mouse_passthrough(true);
                }
            },
            Mouse::Error => log::error!("error getting mouse position"),
        }

        draw_overlay(self, ctx);

        if self.closing {
            frame.close();
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
    #[cfg(not(debug_assertions))]
    let native_options = eframe::NativeOptions {
        decorated: false,
        transparent: true,
        always_on_top: true,
        maximized: true,
        ..Default::default()
    };

    // Use a different set of settings in debug mode just for convenience
    #[cfg(debug_assertions)]
    let native_options = eframe::NativeOptions {
        decorated: true, // enable in debug mode just for easy testing
        transparent: true,
        always_on_top: true,
        maximized: false,
        ..Default::default()
    };

    // No need to set `mouse_passthrough: true` here, the update() function will take care of that for us

    eframe::run_native("Inkbound Overlay", native_options, Box::new(|c| Box::new(Overlay::new(c, datalog)))).unwrap();
}

