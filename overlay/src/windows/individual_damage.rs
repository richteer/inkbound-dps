use std::collections::HashMap;

use egui::Ui;
use egui_plot::{Plot, BarChart, Bar, Text, PlotPoint};
use inkbound_parser::parser::{PlayerStats, DataLog, CombatLog};
use serde::{Deserialize, Serialize};

use crate::OverlayOptions;

use super::{show_dive_selection_box, show_combat_selection_box, WindowDisplay, DamageTotalsMode, DiveCombatSplit};


/// Render a selection box for selecting a player, and return the player stats of the selected player
///  Defaults to the POV character, if the POV is set and exists in the log
///  Candidate to be moved into window/mod.rs if any other window needs this
fn get_player_selection<'a>(ui: &mut egui::Ui, selection: &mut Option<String>, players: &'a HashMap<String, PlayerStats>, pov: &Option<String>) -> Option<&'a PlayerStats> {
    egui::ComboBox::from_label("Select Player")
            .selected_text(format!("{}", selection.as_ref().unwrap_or(&"".to_string())))
            .show_ui(ui, |ui| {
                // Assumes None -> pov character. Probably could be improved, especially if POV detection fails
                ui.selectable_value(selection, None, "YOU");
                for player in players.keys() {
                    ui.selectable_value(selection, Some(player.clone()), player);
                }
            }
        );

    if let Some(selection) = selection {
        players.get(selection)
    } else if let Some(pov) = pov {
        players.get(pov)
    } else {
        None
    }
}

#[derive(Default, Debug)]
pub struct IndividualDamageState {
    pub dive: usize,
    pub combat: usize,
}

#[derive(Default, Deserialize, Serialize, Debug)]
pub struct IndividualSkillsWindow {
    #[serde(skip)]
    state: IndividualDamageState,
    mode: DamageTotalsMode,
    player: Option<String>,
}

#[typetag::serde]
impl WindowDisplay for IndividualSkillsWindow {
    fn show(&mut self, ui: &mut egui::Ui, options: &OverlayOptions, data: &DataLog) {
        self.mode_selection(ui);

        let player_stats = match self.mode {
            DamageTotalsMode::Dive => self.handle_per_dive(ui, data),
            DamageTotalsMode::Combat => self.handle_per_combat(ui, data),
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

impl DiveCombatSplit for IndividualSkillsWindow {
    fn mode<'a>(&'a mut self) -> &'a mut DamageTotalsMode {
        &mut self.mode
    }

    fn set_mode(&mut self, mode: DamageTotalsMode) {
        self.mode = mode
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
    fn handle_per_dive<'a>(&mut self, ui: &mut Ui, datalog: &'a DataLog) -> Option<&'a PlayerStats> {
        let player_stats = if let Some(dive) = datalog.dives.get(self.state.dive) {
            &dive.player_stats.player_stats
        } else {
            ui.label(crate::windows::NO_DATA_MSG);
            return None;
        };

        show_dive_selection_box(ui, &mut self.state.dive, datalog.dives.len());

        get_player_selection(ui, &mut self.player, player_stats, &datalog.pov)
    }

    fn handle_per_combat<'a>(&mut self, ui: &mut egui::Ui, datalog: &'a DataLog) -> Option<&'a PlayerStats> {
        let combats: &Vec<CombatLog> = {
            if let Some(dive) = datalog.dives.get(self.state.dive) {
                &dive.combats
            } else {
                ui.label(crate::windows::NO_DATA_MSG);
                return None;
            }
        };

        show_dive_selection_box(ui, &mut self.state.dive, datalog.dives.len());
        show_combat_selection_box(ui, &mut self.state.combat, combats.len());

        let player_stats = if let Some(combat) = combats.get(self.state.combat) {
            &combat.player_stats.player_stats
        } else {
            return None;
        };

        get_player_selection(ui, &mut self.player, player_stats, &datalog.pov)
    }

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
