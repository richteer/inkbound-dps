use egui::{Window, Align2};
use egui_plot::{Plot, Bar, BarChart, AxisHints, Text, PlotPoint};
use inkbound_parser::parser::{DataLog, PlayerStats, DiveLog};
use serde::{Deserialize, Serialize};

use crate::{Overlay, class_string_to_color};

use super::show_dive_selection_box;

#[derive(Default, PartialEq, Serialize, Deserialize)]
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
}

#[derive(Serialize, Deserialize)]
#[serde(default)]
pub struct HistoryOptions {
    pub show: bool,
    pub mode: HistoryMode,
    pub group_bar_width: f64,
    pub stacked_bar_width: f64,
    pub stacked_show_totals: bool,
}

impl Default for HistoryOptions {
    fn default() -> Self {
        Self {
            show: false,
            mode: HistoryMode::default(),
            group_bar_width: 0.90,
            stacked_bar_width: 0.75,
            stacked_show_totals: false,
        }
    }
}


#[inline]
fn generate_split_bars(dive: &DiveLog, bar_group_width: f64) -> Vec<Bar> {
    dive.combats.iter().rev().enumerate().map(|(combat_index, combat)| {
        let mut players: Vec<PlayerStats> = combat.player_stats.player_stats.values().cloned().collect();
        players.sort_by(|a,b| a.player_data.name.cmp(&b.player_data.name) );
        players.iter().enumerate().map(|(pind, p)| {
            let pind = pind as f64;
            let num_players = combat.player_stats.player_stats.len() as f64;
            let bar_width = bar_group_width / num_players;
            // let x_offset = ((pind + bar_group_width / 2.0) * width) - (bar_group_width / 2.0);
            let x_offset = pind * bar_width - ((bar_group_width - bar_width) / 2.0);
            Bar::new(combat_index as f64 + x_offset + 1.0, p.total_damage_dealt as f64)
                .name(format!("{} {}", p.player_data.name, combat_index + 1))
                .width(bar_width as f64)
                .fill(class_string_to_color(p.player_data.class.as_str()))
        }).collect::<Vec<Bar>>()
    }).flatten().collect()
}

#[inline]
fn generate_stacked_bars(dive: &DiveLog, bar_width: f64, show_stacked_totals: bool) -> (Vec<Bar>, Option<Vec<Text>>) {
    let bars = dive.combats.iter().rev().enumerate().map(|(combat_index, combat)| {
        let mut players: Vec<PlayerStats> = combat.player_stats.player_stats.values().cloned().collect();
        players.sort_by_key(|p| p.total_damage_dealt);
        players.iter().scan(0, |state, p| {
            *state += p.total_damage_dealt;
            Some((*state,  p))
        }).map(|(previous, p)| {
            Bar::new(combat_index as f64 + 1.0, p.total_damage_dealt as f64)
                .name(format!("{} {}", p.player_data.name, combat_index + 1))
                .base_offset((previous - p.total_damage_dealt) as f64)
                .width(bar_width)
                .fill(class_string_to_color(p.player_data.class.as_str()))
        }).collect::<Vec<Bar>>()
    }).flatten().collect();

    // TODO: This totally can be done in one pass with the previous
    let texts = if show_stacked_totals {
        Some(dive.combats.iter().rev().enumerate().map(|(combat_index, combat)| {
            let total_damage_dealt = combat.player_stats.player_stats.values().fold(0, |acc, elem| acc + elem.total_damage_dealt);
            Text::new(
                PlotPoint { x: combat_index as f64 + 1.0, y: total_damage_dealt as f64 },
                format!("{}", total_damage_dealt)
            ).anchor(Align2::CENTER_BOTTOM)
        }).collect())
    } else {
            None
    };

    (bars, texts)
}


pub fn draw_history_window(overlay: &mut Overlay, ctx: &egui::Context, datalog: &DataLog) {
    if !overlay.options.history.show {
        return;
    }
    
    Window::new("History").show(ctx, |ui| {
        show_dive_selection_box(ui, &mut overlay.window_state.history.dive, datalog.dives.len());
        egui::ComboBox::from_label("Mode")
            .selected_text(overlay.options.history.mode.to_string())
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut overlay.options.history.mode, HistoryMode::Split, "Split");
                ui.selectable_value(&mut overlay.options.history.mode, HistoryMode::Stacked, "Stacked");
            })
            .response.on_hover_text("Select which mode to render the history plot.\n\nSplit - Each player has their own vertical bar grouped by combat.\nStacked - Player damage bars are stacked on top of each other, with the total length of the bar representing total group damage.");

        match overlay.options.history.mode {
            HistoryMode::Split => {
                ui.add(egui::Slider::new(&mut overlay.options.history.group_bar_width, 0.25..=1.0)
                    .max_decimals(2)
                    .text("Bar Group Width"));
            },
            HistoryMode::Stacked => {
                ui.add(egui::Slider::new(&mut overlay.options.history.stacked_bar_width, 0.25..=1.0)
                    .max_decimals(2)
                    .text("Bar Width"));
                ui.checkbox(&mut overlay.options.history.stacked_show_totals, "Show Totals");
            },
        }

        ui.separator();
        
        let dive = if let Some(dive) = datalog.dives.get(overlay.window_state.history.dive) {
            dive
        } else {
            return
        };

        let (bars, texts) = match overlay.options.history.mode {
            HistoryMode::Split => (generate_split_bars(dive, overlay.options.history.group_bar_width), None),
            HistoryMode::Stacked => generate_stacked_bars(dive, overlay.options.history.stacked_bar_width, overlay.options.history.stacked_show_totals),
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
            .set_margin_fraction(egui::Vec2 { x: 0.1, y: 0.1 })
            .show_grid(true)
            .show_axes(true)
            .show_background(false)
            .show_x(false)
            .show_y(true)
            .x_axis_label("Combat")
            .y_axis_label("Damage")
            .show(ui, |plot_ui| {
                plot_ui.bar_chart(chart);
                if let Some(texts) = texts {
                    for text in texts {
                        plot_ui.text(text);
                    }
                }
            });
            
    });
}
