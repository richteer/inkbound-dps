use std::collections::HashMap;

use egui::{Window, Ui};
use egui_plot::{Plot, BarChart, Bar, Text, PlotPoint};
use inkbound_parser::parser::{PlayerStats, DataLog};

use crate::{Overlay, class_string_to_color};

#[derive(Default)]
pub struct IndividualDamageState {
    pub show: bool,
    pub player: Option<String>,
}

#[inline]
pub fn draw_dive_individual_damage_window(overlay: &mut Overlay, ctx: &egui::Context, datalog: &DataLog) {
    if !overlay.window_state.dive_individual_damage.show {
        return;
    }

    let player_stats = if let Some(dive) = datalog.dives.get(0) {
        dive.player_stats.player_stats.clone()
    } else {
        // Don't bother with the rest if there isn't dive data
        return;
    };
    draw_individual_damage_window(ctx, "Dive Individual Damage", &player_stats, &mut overlay.window_state.dive_individual_damage.player);
}

#[inline]
pub fn draw_combat_individual_damage_window(overlay: &mut Overlay, ctx: &egui::Context, datalog: &DataLog) {
    if !overlay.window_state.combat_individual_damage.show {
        return;
    }
    
    let player_stats = if let Some(dive) = datalog.dives.get(0) {
        if let Some(combat) = dive.combats.get(0) {
            combat.player_stats.player_stats.clone()
        } else {
            return;
        }
    } else {
        // Don't bother with the rest if there isn't dive data
        return;
    };
    draw_individual_damage_window(ctx, "Combat Individual Damage", &player_stats, &mut overlay.window_state.combat_individual_damage.player);
}

/// Draw the window and player selection combo box, chain into plot drawing logic
#[inline]
fn draw_individual_damage_window(ctx: &egui::Context, name: &str, player_stats: &HashMap<String, PlayerStats>, selection: &mut Option<String>) {
    Window::new(name).show(ctx, |ui| {

        Box::new(egui::ComboBox::from_label("Select Player"))
            .selected_text(format!("{}", selection.as_ref().unwrap_or(&"".to_string())))
            .show_ui(ui, |ui| {
                for player in player_stats.keys() {
                    ui.selectable_value(selection, Some(player.clone()), player);
                }
            }
        );
        if let Some(selection) = selection {
            if let Some(player_stats) = player_stats.get(selection) {
                draw_individual_damage_plot(ui, player_stats, name);
            }
        }
    });
}

/// Draw the bar plot for the individual skills given the player stats data
#[inline]
fn draw_individual_damage_plot(ui: &mut Ui, player_stats: &PlayerStats, name: &str) {
    
    let mut skill_totals: Vec<(String, i64)> = player_stats.skill_totals.clone().into_iter().collect();
    skill_totals.sort_by_key(|e| e.1);

    let bars = {
        skill_totals.iter().enumerate().map(|(index, (_, dmg))| {
            Bar::new(index as f64, *dmg as f64)
                .width(1.0)
                .fill(class_string_to_color(player_stats.player_data.class.as_str()))
        }).collect()
    };

    let texts: Vec<Text> = {
        skill_totals.iter().enumerate().map(|(index, (name, dmg))| {
            Text::new(
                PlotPoint { x: 0.0, y: index as f64 },
                format!("  {} - {} ({:.2}%)",
                    name,
                    dmg,
                    *dmg as f64 / player_stats.total_damage_dealt as f64 * 100.0,
                )
            )
            .anchor(egui::Align2::LEFT_CENTER)
            .color(egui::Color32::WHITE)
        }).collect()
    };

    let chart = BarChart::new(bars)
        .horizontal()
    ;
    Plot::new(format!("{} Plot", name))
        .allow_boxed_zoom(false)
        .allow_drag(false)
        .allow_scroll(false)
        .allow_zoom(false)
        .show_grid(false)
        .show_axes(false)
        .show_background(false)
        .show_x(false)
        .show_y(false)
        .show(ui, |plot_ui| {
                plot_ui.bar_chart(chart);
                for text in texts {
                    plot_ui.text(text);
                }
            }
        );
}
