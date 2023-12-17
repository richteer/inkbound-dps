use std::collections::HashMap;

use egui::Ui;
use egui_plot::{Plot, BarChart, Bar, Text, PlotPoint};
use inkbound_parser::parser::{PlayerStats, DataLog};
use serde::{Deserialize, Serialize};

use crate::OverlayOptions;

use super::{WindowDisplay, DamageTotalsMode, DiveCombatSplit, DiveCombatSelectionState, PlayerSelection, PlayerDiveCombatOptions};


#[derive(Default, Debug)]
pub struct IndividualDamageState {
    pub dive: usize,
    pub combat: usize,
}

#[derive(Default, Deserialize, Serialize, Debug)]
pub struct IndividualSkillsWindow {
    #[serde(skip)]
    state: DiveCombatSelectionState,
    mode: DamageTotalsMode,
    player: Option<String>,
}

impl PlayerSelection for IndividualSkillsWindow {
    fn player<'a>(&'a mut self) -> &'a mut Option<String> {
        &mut self.player
    }
}

impl DiveCombatSplit for IndividualSkillsWindow {
    fn mode<'a>(&'a mut self) -> &'a mut DamageTotalsMode {
        &mut self.mode
    }

    fn set_mode(&mut self, mode: DamageTotalsMode) {
        self.mode = mode
    }

    fn state<'a>(&'a mut self) -> &'a mut super::DiveCombatSelectionState {
        &mut self.state
    }
}

#[typetag::serde]
impl WindowDisplay for IndividualSkillsWindow {
    fn show(&mut self, ui: &mut egui::Ui, options: &OverlayOptions, data: &DataLog) {
        self.show_options(ui, data);

        let player_stats = self.get_current_player_stat_list(data);
        let player_stats = if let Some(player_stats) = player_stats {
            player_stats
        } else {
            ui.label(super::NO_DATA_MSG.to_string());
            return;
        };

        let player_stats = if let Some(selection) = self.player.as_ref() {
            player_stats.get(selection)
        } else if let Some(pov) = data.pov.as_ref() {
            player_stats.get(pov)
        } else {
            None
        };

        if let Some(player_stats) = player_stats {
            self.draw_individual_damage_plot(ui, player_stats, options);
        }
    }

    fn name(&self) -> String {
        let mode = self.mode.to_string();
        let base = format!("Skill Totals: {mode}");
        match &self.player {
            Some(p) => format!("{base}: {p}"),
            // TODO: consider naming the window different if self-stats
            None => format!("{base}"),
        }
    }
}

#[inline]
fn clean_skill_name<'a>(name: &String) -> String {
    name
        .replace("Damage","")
        .replace("Upgrade","")
        .replace("Legendary", "➡")
        .replace("_"," ")
        .trim()
        .to_string()
}


impl IndividualSkillsWindow {
    /// Draw the bar plot for the individual skills given the player stats data
    #[inline]
    fn draw_individual_damage_plot(&self, ui: &mut Ui, player_stats: &PlayerStats, options: &OverlayOptions) {
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
                        clean_skill_name(name),
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
        Plot::new(format!("{} Plot", self.name()))
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
}
