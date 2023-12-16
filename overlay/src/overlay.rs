use std::{sync::{Arc, RwLock}, collections::{BTreeSet, BTreeMap}};

use egui::{Visuals, Pos2, ViewportBuilder, ViewportCommand};
use inkbound_parser::parser::DataLog;
use serde::de::DeserializeOwned;

use crate::{windows::{self, WindowDisplay, WindowId, OverlayWindow}, options::OverlayOptions};

static OPTIONS_STORAGE_KEY: &'static str = "overlayoptions";
static WINDOWS_STORAGE_KEY: &'static str = "overlaywindows";
static ENABLED_WINDOWS_STORAGE_KEY: &'static str = "overlayenabledwindows";

#[derive(Default)]
pub struct WindowState {
    pub settings: windows::SettingsState,
    pub color_settings: windows::ColorSettingsState,
    #[cfg(feature = "auto_update")]
    pub update: crate::updater::UpdateState,
}

// #[derive(Default)]
pub struct Overlay {
    pub datalog: Arc<RwLock<DataLog>>,
    pub window_state: WindowState,
    pub options: OverlayOptions,
    pub windows: BTreeMap<WindowId, OverlayWindow>,
    pub enabled_windows: BTreeSet<WindowId>,
}

pub fn default_windows() -> BTreeMap<WindowId, OverlayWindow> {
    use crate::windows::*;
    let windows = vec![
        GroupDamageWindow::window_from_mode(DamageTotalsMode::Dive),
        GroupDamageWindow::window_from_mode(DamageTotalsMode::Combat),
        IndividualSkillsWindow::window_from_mode(DamageTotalsMode::Dive),
        IndividualSkillsWindow::window_from_mode(DamageTotalsMode::Combat),
        OverlayWindow::new::<HistoryWindow>(),
    ];

    windows.into_iter().map(|e| (e.id(), e)).collect()
}

fn load_from_storage<T: Default + DeserializeOwned>(storage: Option<&dyn eframe::Storage>, key: &str) -> Option<T> {
    if let Some(storage) = storage {
        let data = storage.get_string(key);
        if let Some(data) = data {
            match ron::from_str::<T>(data.as_str()) {
                Ok(d) => Some(d),
                Err(e) => {
                    log::warn!("options parse error: {:?}", e);
                    log::warn!("failed to parse stored options, reverting to default options");
                    None
                }
            }
        } else {
            log::debug!("no options to load, using defaults");
            None
        }
    } else {
        log::warn!("failed to load persistence storage, using default options");
        None
    }
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

        let options = load_from_storage(_cc.storage, OPTIONS_STORAGE_KEY);

        // If options could not be loaded, either there is an error, or this is the first run.
        let (windows, enabled_windows) = if options.is_none() {
            // Load defaults in this case
            (default_windows(), BTreeSet::new())
        } else {
            // Otherwise handle the rest of the loads
            let windows: BTreeMap<WindowId, OverlayWindow> = load_from_storage(_cc.storage, WINDOWS_STORAGE_KEY).unwrap_or_default();
            let enabled_windows = load_from_storage(_cc.storage, ENABLED_WINDOWS_STORAGE_KEY).unwrap_or_default();
            (windows, enabled_windows)
        };

        let options = options.unwrap_or_default();

        let mut window_state = WindowState::default();
        window_state.color_settings.sync_from_options(&options);

        #[cfg(feature = "auto_update")]
        {
            let mut updater = updater::UPDATER.lock().unwrap();
            // Confirmation is handled by the UI
            updater.set_options(updater::UpdaterOptions::default()
                .no_confirm(true)
                .show_download_progress(false)
            );

            if options.auto_check_update {
                updater.fetch_update(false);
            }
        }

        Self {
            datalog,
            window_state,
            options,
            windows,
            enabled_windows,
        }
    }
}

impl eframe::App for Overlay {
    fn clear_color(&self, _visuals: &Visuals) -> [f32; 4] {
        egui::Rgba::TRANSPARENT.to_rgba_unmultiplied()
    }

    // TODO: factor out a helper
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        // Filter out any "enabled" window IDs that no longer exist in the definition table
        //  This is mostly to clean up potentially stale IDs after a version update
        self.enabled_windows.retain(|w| self.windows.contains_key(w));

        match ron::to_string(&self.options) {
            Ok(options) => storage.set_string(OPTIONS_STORAGE_KEY, options),
            Err(e) => log::error!("failed to serialize options: {:?}", e),
        }
        match ron::to_string(&self.windows) {
            Ok(windows) => storage.set_string(WINDOWS_STORAGE_KEY, windows),
            Err(e) => log::error!("failed to serialize windows: {:?}", e),
        }
        match ron::to_string(&self.enabled_windows) {
            Ok(enabled_windows) => storage.set_string(ENABLED_WINDOWS_STORAGE_KEY, enabled_windows),
            Err(e) => log::error!("failed to serialize enabled_windows: {:?}", e),
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

    let style = ctx.style();
    for (id, window) in overlay.windows.iter_mut() {
        if overlay.enabled_windows.contains(id) {
            let mut open = true;

            // If hovering over this window in the settings menu, highlight the frame with a shadow
            let frame = match (overlay.window_state.settings.highlight_window.inner_eq(id), &overlay.window_state.settings.highlight_window) {
                (true, windows::HighlightWindow::Toggle(_)) =>
                    egui::Frame::window(&style)
                        .shadow(egui::epaint::Shadow { extrusion: 5.0, color: egui::Color32::WHITE}),
                (true, windows::HighlightWindow::Delete(_)) =>
                    egui::Frame::window(&style)
                        .shadow(egui::epaint::Shadow { extrusion: 5.0, color: egui::Color32::RED}),
                (true, windows::HighlightWindow::None) => {
                    log::error!("somehow inner_eq returned true with this set to None");
                    egui::Frame::window(&style)
                },
                (false, _) => egui::Frame::window(&style),
            };

            egui::Window::new(window.name())
                .id(egui::Id::new(window.id()))
                .open(&mut open)
                .frame(frame)
                .show(ctx, |ui| {
                    window.show(ui, &overlay.options, &datalog);
                });

            if !open {
                overlay.enabled_windows.remove(id);
            }
        }
    }
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

