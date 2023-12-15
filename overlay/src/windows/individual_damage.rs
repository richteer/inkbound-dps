use std::collections::HashMap;

use egui::Ui;
use egui_plot::{Plot, BarChart, Bar, Text, PlotPoint};
use inkbound_parser::parser::{PlayerStats, DataLog, CombatLog};
use serde::{Deserialize, Serialize};

use crate::OverlayOptions;

use super::{show_dive_selection_box, show_combat_selection_box, WindowDisplay};

#[derive(Default)]
pub struct IndividualDamageState {
    pub player: Option<String>,
    pub dive: usize,
    pub combat: usize,
}

#[derive(Default, Deserialize, Serialize)]
pub struct IndividualDiveWindow {
    #[serde(skip)]
    state: IndividualDamageState,
}

#[typetag::serde]
impl WindowDisplay for IndividualDiveWindow {
    fn show(&mut self, ui: &mut egui::Ui, options: &OverlayOptions, data: &DataLog) {
        self.draw_dive_individual_damage_window(ui, options, data);
    }

    fn id(&self) -> super::WindowId {
        self.name()
    }

    fn name(&self) -> String {
        "Dive Individual Damage".to_string()
    }
}

impl IndividualDiveWindow {
    pub fn draw_dive_individual_damage_window(&mut self, ui: &mut Ui, options: &OverlayOptions, datalog: &DataLog) {
        let name = "Dive Individual Damage";

        let player_stats = if let Some(dive) = datalog.dives.get(self.state.dive) {
            dive.player_stats.player_stats.clone()
        } else {
            ui.label(crate::windows::NO_DATA_MSG);
            // Don't bother with the rest if there isn't dive data
            return;
        };

        show_dive_selection_box(ui, &mut self.state.dive, datalog.dives.len());

        let selection = &mut self.state.player;
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
                draw_individual_damage_plot(ui, player_stats, name, options);
            }
        } else if let Some(player_stats) = player_stats.get(&options.default_player_name) {
            draw_individual_damage_plot(ui, player_stats, name, options);
        }
    }
}

#[derive(Default, Deserialize, Serialize)]
pub struct IndividualCombatWindow {
    #[serde(skip)]
    state: IndividualDamageState,
}

#[typetag::serde]
impl WindowDisplay for IndividualCombatWindow {
    fn show(&mut self, ui: &mut egui::Ui, options: &OverlayOptions, data: &DataLog) {
        self.draw_combat_individual_damage_window(ui, options, data);
    }

    fn id(&self) -> super::WindowId {
        self.name()
    }

    fn name(&self) -> String {
        "Combat Individual Damage".to_string()
    }
}

impl IndividualCombatWindow {
    pub fn draw_combat_individual_damage_window(&mut self, ui: &mut egui::Ui, options: &OverlayOptions, datalog: &DataLog) {
        let name = "Combat Individual Damage";

        let combats: &Vec<CombatLog> = {
            if let Some(dive) = datalog.dives.get(self.state.dive) {
                &dive.combats
            } else {
                ui.label(crate::windows::NO_DATA_MSG);
                return // Dive doesn't exist, don't bother continuning
            }
        };

        show_dive_selection_box(ui, &mut self.state.dive, datalog.dives.len());
        show_combat_selection_box(ui, &mut self.state.combat, combats.len());

        let player_stats = if let Some(combat) = combats.get(self.state.combat) {
            &combat.player_stats.player_stats
        } else {
            return // No combat, no need to select a player or draw a plot
        };

        let selection = &mut self.state.player;

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
                draw_individual_damage_plot(ui, player_stats, name, options);
            }
        } else if let Some(player_stats) = player_stats.get(&options.default_player_name) {
            draw_individual_damage_plot(ui, player_stats, name, options);
        }
    }
}

/// Draw the bar plot for the individual skills given the player stats data
#[inline]
fn draw_individual_damage_plot(ui: &mut Ui, player_stats: &PlayerStats, name: &str, options: &OverlayOptions) {
    let mut skill_totals: HashMap<String, (i64, i64)> = HashMap::new();
    player_stats.skill_totals.iter().for_each(|(k,v)| { skill_totals.insert(k.clone(), (*v, 0)); });

    // Skip if not showing crit bars for performance I guess
    if options.show_crit_bars {
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

    let bar_color = options.colors.get_aspect_color(&player_stats.player_data.class);
        let bars = if options.show_crit_bars {
        skill_totals.iter().enumerate().map(|(index, (_, (dmg, crit)))| {
            [
                Bar::new(index as f64, *dmg as f64)
                    .width(1.0)
                    .fill(bar_color)
                ,
                Bar::new(index as f64, *crit as f64)
                    .width(1.0)
                    .fill(egui::Color32::from_rgba_unmultiplied(0, 0, 0, options.crit_bar_opacity))
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
