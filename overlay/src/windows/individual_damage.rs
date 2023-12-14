use std::collections::HashMap;

use egui::{Window, Ui};
use egui_plot::{Plot, BarChart, Bar, Text, PlotPoint};
use inkbound_parser::parser::{PlayerStats, DataLog, CombatLog};

use crate::Overlay;

use super::{show_dive_selection_box, show_combat_selection_box};

#[derive(Default)]
pub struct IndividualDamageState {
    pub player: Option<String>,
    pub dive: usize,
}

#[inline]
pub fn draw_dive_individual_damage_window(overlay: &mut Overlay, ctx: &egui::Context, datalog: &DataLog) {
    if !overlay.options.show_dive_individual_damage {
        return;
    }

    let name = "Dive Individual Damage";

    Window::new(name).show(ctx, |ui| {
        let selection = &mut overlay.window_state.dive_individual_damage.player;
        show_dive_selection_box(ui, &mut overlay.window_state.dive_individual_damage.dive, datalog.dives.len());

        let player_stats = if let Some(dive) = datalog.dives.get(overlay.window_state.dive_individual_damage.dive) {
            dive.player_stats.player_stats.clone()
        } else {
            // Don't bother with the rest if there isn't dive data
            return;
        };

        egui::ComboBox::from_label("Select Player")
            .selected_text(format!("{}", selection.as_ref().unwrap_or(&"".to_string())))
            .show_ui(ui, |ui| {
                for player in player_stats.keys() {
                    ui.selectable_value(selection, Some(player.clone()), player);
                }
            }
        );
        if let Some(selection) = selection {
            if let Some(player_stats) = player_stats.get(selection) {
                draw_individual_damage_plot(ui, player_stats, name, overlay);
            }
        } else if let Some(player_stats) = player_stats.get(&overlay.options.default_player_name) {
            draw_individual_damage_plot(ui, player_stats, name, overlay);
        }
    });
}

#[inline]
pub fn draw_combat_individual_damage_window(overlay: &mut Overlay, ctx: &egui::Context, datalog: &DataLog) {
    if !overlay.options.show_combat_individual_damage {
        return;
    }

    let name = "Combat Individual Damage";
    Window::new(name).show(ctx, |ui| {
        show_dive_selection_box(ui, &mut overlay.window_state.combat_group_damage.dive, datalog.dives.len());

        let combats: &Vec<CombatLog> = {
            if let Some(dive) = datalog.dives.get(overlay.window_state.combat_group_damage.dive) {
                &dive.combats
            } else {
                return // Dive doesn't exist, don't bother continuning
            }
        };

        show_combat_selection_box(ui, &mut overlay.window_state.combat_group_damage.combat, combats.len());

        let player_stats = if let Some(combat) = combats.get(overlay.window_state.combat_group_damage.combat) {
            &combat.player_stats.player_stats
        } else {
            return // No combat, no need to select a player or draw a plot
        };

        let selection = &mut overlay.window_state.combat_individual_damage.player;

        egui::ComboBox::from_label("Select Player")
            .selected_text(format!("{}", selection.as_ref().unwrap_or(&"".to_string())))
            .show_ui(ui, |ui| {
                for player in player_stats.keys() {
                    ui.selectable_value(selection, Some(player.clone()), player);
                }
            }
        );

        if let Some(selection) = selection {
            if let Some(player_stats) = player_stats.get(selection) {
                draw_individual_damage_plot(ui, player_stats, name, overlay);
            }
        } else if let Some(player_stats) = player_stats.get(&overlay.options.default_player_name) {
            draw_individual_damage_plot(ui, player_stats, name, overlay);
        }
    });
}

/// Draw the bar plot for the individual skills given the player stats data
#[inline]
fn draw_individual_damage_plot(ui: &mut Ui, player_stats: &PlayerStats, name: &str, overlay: &Overlay) {
    let mut skill_totals: HashMap<String, (i64, i64)> = HashMap::new();
    player_stats.skill_totals.iter().for_each(|(k,v)| { skill_totals.insert(k.clone(), (*v, 0)); });

    // Skip if not showing crit bars for performance I guess
    if overlay.options.show_crit_bars {
        player_stats.crit_totals.iter().for_each(|(k, crit_dmg)| { skill_totals.entry(k.clone())
            .and_modify(|elem| elem.1 += crit_dmg)
            .or_insert((0, *crit_dmg)); } );
    }
    
    // let mut skill_totals: Vec<(String, i64)> = player_stats.skill_totals.clone().into_iter().collect();
    let mut skill_totals: Vec<(String, (i64, i64))> = skill_totals.into_iter().collect();
    skill_totals.sort_by(|a,b| {
        let res = a.1.0.cmp(&b.1.0);
        match res {
            std::cmp::Ordering::Equal => a.0.cmp(&b.0),
            _ => res,
        }
    });

    let bar_color = overlay.options.colors.get_aspect_color(&player_stats.player_data.class);
        let bars = if overlay.options.show_crit_bars {
        skill_totals.iter().enumerate().map(|(index, (_, (dmg, crit)))| {
            [
                Bar::new(index as f64, *dmg as f64)
                    .width(1.0)
                    .fill(bar_color)
                ,
                Bar::new(index as f64, *crit as f64)
                    .width(1.0)
                    .fill(egui::Color32::from_rgba_unmultiplied(0, 0, 0, overlay.options.crit_bar_opacity))
            ]
        }).flatten().collect()
    } else {
        skill_totals.iter().enumerate().map(|(index, (_, (dmg, _crit)))| {
            Bar::new(index as f64, *dmg as f64)
                .width(1.0)
                .fill(bar_color)
        }).collect()
    };

    let texts: Vec<Text> = {
        skill_totals.iter().enumerate().map(|(index, (name, (dmg, _crit)))| {
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
        .auto_bounds_x()
        .auto_bounds_y()
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
