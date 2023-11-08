use std::sync::{RwLock, Arc};

use egui_render_three_d::ThreeDBackend as DefaultGfxBackend;
use inkbound_parser::parser::DataLog;

use crate::*;

impl egui_overlay::EguiOverlay for Overlay {
     fn gui_run(
        &mut self,
        egui_context: &egui::Context,
        _default_gfx_backend: &mut DefaultGfxBackend,
        glfw_backend: &mut egui_window_glfw_passthrough::GlfwBackend,
    ) {

        draw_overlay(self, egui_context);

        // here you decide if you want to be passthrough or not.
        if egui_context.wants_pointer_input() || egui_context.wants_keyboard_input() {
            glfw_backend.window.set_mouse_passthrough(false);
        } else {
            glfw_backend.window.set_mouse_passthrough(true);
        }
        egui_context.request_repaint();
    }
}

pub fn spawn_overlay(datalog: Arc<RwLock<DataLog>>) {
    egui_overlay::start(Overlay::new(datalog));
}
