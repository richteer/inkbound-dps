use egui::Window;
use egui_plot::{Plot, Bar, BarChart, AxisHints};
use inkbound_parser::parser::{DataLog, PlayerStats, DiveLog};

use crate::{Overlay, class_string_to_color};

use super::show_dive_selection_box;

#[derive(Default, PartialEq)]
pub enum HistoryMode {
    #[default]
    Split,
    Stacked,
}

impl std::fmt::Display for HistoryMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let out = match self {
            HistoryMode::Split => "Split",
            HistoryMode::Stacked => "Stacked",
        };
        f.write_str(out)
    }
}

#[derive(Default)]
pub struct HistoryState {
    pub dive: usize,
    pub mode: HistoryMode,
}


#[inline]
fn generate_split_bars(dive: &DiveLog) -> Vec<Bar> {
    dive.combats.iter().rev().enumerate().map(|(combat_index, combat)| {
        let mut players: Vec<PlayerStats> = combat.player_stats.player_stats.values().cloned().collect();
        players.sort_by(|a,b| a.player_data.name.cmp(&b.player_data.name) );
        players.iter().enumerate().map(|(pind, p)| {
            let num_players = combat.player_stats.player_stats.len() as f64;
            let width = 1.0 / num_players as f64;
            let x_offset = ((pind as f64 + 0.5) as f64 * width) - 0.5;
            Bar::new(combat_index as f64 + x_offset + 1.0, p.total_damage_dealt as f64)
                .name(format!("{} {}", p.player_data.name, combat_index + 1))
                .width(width as f64)
                .fill(class_string_to_color(p.player_data.class.as_str()))
        }).collect::<Vec<Bar>>()
    }).flatten().collect()
}

#[inline]
fn generate_stacked_bars(dive: &DiveLog) -> Vec<Bar> {
    dive.combats.iter().rev().enumerate().map(|(combat_index, combat)| {
        let mut players: Vec<PlayerStats> = combat.player_stats.player_stats.values().cloned().collect();
        players.sort_by_key(|p| p.total_damage_dealt);
        players.iter().scan(0, |state, p| {
            *state += p.total_damage_dealt;
            Some((*state,  p))
        }).map(|(previous, p)| {
            Bar::new(combat_index as f64 + 1.0, p.total_damage_dealt as f64)
                .name(format!("{} {}", p.player_data.name, combat_index + 1))
                .base_offset((previous - p.total_damage_dealt) as f64)
                .width(1.0)
                .fill(class_string_to_color(p.player_data.class.as_str()))
        }).collect::<Vec<Bar>>()
    }).flatten().collect()
}


pub fn draw_history_window(overlay: &mut Overlay, ctx: &egui::Context, datalog: &DataLog) {
    if !overlay.options.show_history {
        return;
    }
    
    Window::new("History").show(ctx, |ui| {
        show_dive_selection_box(ui, &mut overlay.window_state.history.dive, datalog.dives.len());
        egui::ComboBox::from_label("Mode")
            .selected_text(overlay.window_state.history.mode.to_string())
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut overlay.window_state.history.mode, HistoryMode::Split, "Split");
                ui.selectable_value(&mut overlay.window_state.history.mode, HistoryMode::Stacked, "Stacked");
            })
            .response.on_hover_text("Select which mode to render the history plot\n\nSplit - Each player has their own vertical bar per combat\nStacked - Player damage bars are stacked on top of each other, with the total length of the bar representing total group damage.");

        ui.separator();
        
        let dive = if let Some(dive) = datalog.dives.get(overlay.window_state.history.dive) {
            dive
        } else {
            return
        };

        let bars = match overlay.window_state.history.mode {
            HistoryMode::Split => generate_split_bars(dive),
            HistoryMode::Stacked => generate_stacked_bars(dive),
        };

        let chart = BarChart::new(bars);

        Plot::new("History Plot")
            .allow_boxed_zoom(false)
            .allow_drag(false)
            .allow_scroll(false)
            .allow_zoom(false)
            .auto_bounds_x()
            .auto_bounds_y()
            // .clamp_grid(true)
            .custom_y_axes(vec![AxisHints::default().formatter(|value, _, _| {
                if value > 1000.0 {
                    format!("{:.0}k", value / 1000.0)
                } else {
                    format!("{value}")
                }
            })])
            .set_margin_fraction(egui::Vec2 { x: 0.0, y: 0.1 })
            .show_grid(true)
            .show_axes(true)
            .show_background(false)
            .show_x(false)
            .show_y(true)
            .x_axis_label("Combat")
            .y_axis_label("Damage")
            .show(ui, |plot_ui| {
                plot_ui.bar_chart(chart);
            });
            
    });
}
