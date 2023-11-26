use std::sync::{Arc, RwLock};

use egui::{Visuals, Pos2, ViewportBuilder, ViewportCommand};
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
    pub history: windows::HistoryState,
}

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
        }
    }
}

// #[derive(Default)]
pub struct Overlay {
    pub datalog: Arc<RwLock<DataLog>>,
    pub window_state: WindowState,
    pub options: OverlayOptions,
}


impl Overlay {
    pub fn new(_cc: &eframe::CreationContext<'_>, datalog: Arc<RwLock<DataLog>>) -> Self {
        // Customize egui here with cc.egui_ctx.set_fonts and cc.egui_ctx.set_visuals.
        // Restore app state using cc.storage (requires the "persistence" feature).
        // Use the cc.gl (a glow::Context) to create graphics shaders and buffers that you can use
        // for e.g. egui::PaintCallback.
        _cc.egui_ctx.set_visuals(egui::style::Visuals {
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
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {

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
                let window_pos = ctx.input(|i| i.viewport().inner_rect.unwrap().min);

                // Adjust absolute pointer pos to egui relative position
                pointer_pos -= window_pos.to_vec2();

                #[cfg(debug_assertions)]
                if let Some(pos) = ctx.input(|i| i.pointer.latest_pos()) {
                    // For debugging when the mouse is slightly offset for some reason
                    log::trace!("pos - pointer_pos = {:?}", pos - pointer_pos);
                };
                if ctx.is_pos2_over_area(pointer_pos) {
                    log::trace!("is over area, disabling passthrough");
                    ctx.send_viewport_cmd(ViewportCommand::MousePassthrough(false));
                } else {
                    log::trace!("enabling passthrough");
                    ctx.send_viewport_cmd(ViewportCommand::MousePassthrough(true));
                }
            },
            Mouse::Error => log::error!("error getting mouse position"),
        }

        draw_overlay(self, ctx);
    }
}

/// Main entrypoint to draw all the egui widgets and things
pub fn draw_overlay(overlay: &mut Overlay, ctx: &egui::Context) {
    let datalog = {
        // TODO: This seems very wasteful of memory, consider doing some kind of cache
        //  update clone only if data changed, etc
        overlay.datalog.read().unwrap().clone()
    };
    windows::draw_settings_window(overlay, ctx);
    windows::draw_dive_damage_window(overlay, ctx, &datalog);
    windows::draw_combat_damage_window(overlay, ctx, &datalog);
    windows::draw_dive_individual_damage_window(overlay, ctx, &datalog);
    windows::draw_combat_individual_damage_window(overlay, ctx, &datalog);
    windows::draw_history_window(overlay, ctx, &datalog);
}


/// Entrypoint for the main application to spawn the actual overlay window and such
pub fn spawn_overlay(datalog: Arc<RwLock<DataLog>>) {
    // Common options to release and debug
    let viewport = ViewportBuilder::default()
        .with_transparent(true)
    ;

    // Release mode options
    #[cfg(not(debug_assertions))]
    let viewport = {
        viewport
            .with_decorations(false)
            .with_transparent(true)
            .with_always_on_top()
            .with_maximized(true)
            // Probably unnecessary, but useful to make note of here
            .with_mouse_passthrough(true)
    };

    // Debug mode options
    //  Run in a non-maximized window so it doesn't take up the whole dang screen
    #[cfg(debug_assertions)]
    let viewport = {
        viewport
            .with_decorations(true)
            .with_transparent(true)
            // .with_always_on_top()
            .with_maximized(false)
    };

    let native_options = eframe::NativeOptions {
        viewport,
        ..Default::default()
    };

    eframe::run_native("Inkbound Overlay", native_options, Box::new(|c| Box::new(Overlay::new(c, datalog)))).unwrap();
}

