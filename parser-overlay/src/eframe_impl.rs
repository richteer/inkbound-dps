use std::sync::{Arc, RwLock};
use eframe::egui;

use egui::{Visuals, Pos2};
use inkbound_parser::parser::DataLog;
use crate::*;

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

impl eframe::App for Overlay {
    fn clear_color(&self, _visuals: &Visuals) -> [f32; 4] {
        egui::Rgba::TRANSPARENT.to_rgba_unmultiplied()
    }

    // TODO: seriously consider preformatting the information on the parse thread
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {

        // Force maximize the window, for some reason setting in NativeOptions is ignored
        // Don't in debug though, just so it can be tested in a window better
        #[cfg(not(debug_assertions))]
        frame.set_maximized(true);

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
