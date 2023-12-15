use egui::Ui;
use egui_plot::{Text, PlotPoint, BarChart, Plot, Bar};
use inkbound_parser::parser::{PlayerStats, DataLog, CombatLog};

use crate::OverlayOptions;

use super::{show_dive_selection_box, show_combat_selection_box, WindowDisplay};

#[derive(Default)]
pub struct GroupDamageState {
    pub dive: usize,
    pub combat: usize,
}

#[derive(Default)]
pub struct GroupCombatWindow {
    state: GroupDamageState,
}

impl WindowDisplay for GroupCombatWindow {
    fn show(&mut self, ui: &mut egui::Ui, options: &OverlayOptions, data: &DataLog) {
        self.draw_combat_damage_window(ui, options, data);
    }

    fn id(&self) -> super::WindowId {
        self.name()
    }

    fn name(&self) -> String {
        "Combat Group Damage".to_string()
    }
}

impl GroupCombatWindow {
    pub fn draw_combat_damage_window(&mut self, ui: &mut Ui, options: &OverlayOptions, datalog: &DataLog) {
        let name = "Combat Group Damage";

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

        let statlist = if let Some(combat) = combats.get(self.state.combat) {
            combat.player_stats.player_stats.values().cloned().collect()
        } else {
            Vec::new()
        };

        draw_group_damage_plot(ui, options, statlist, name);
    }
}

// TODO: Consider merging these somehow
#[derive(Default)]
pub struct GroupDiveWindow {
    state: GroupDamageState,
}

impl WindowDisplay for GroupDiveWindow {
    fn show(&mut self, ui: &mut egui::Ui, options: &OverlayOptions, data: &DataLog) {
        self.draw_dive_damage_window(ui, options, data);
    }

    fn id(&self) -> super::WindowId {
        self.name()
    }

    fn name(&self) -> String {
        "Dive Group Damage".to_string()
    }
}

impl GroupDiveWindow {
    pub fn draw_dive_damage_window(&mut self, ui: &mut Ui, options: &OverlayOptions, datalog: &DataLog) {
        let name = "Dive Group Damage";

        let statlist: Vec<PlayerStats> = {
            if let Some(dive) = datalog.dives.get(self.state.dive) {
                dive.player_stats.player_stats.values().cloned().collect()
            } else {
                ui.label(crate::windows::NO_DATA_MSG);
                return
            }
        };

        show_dive_selection_box(ui, &mut self.state.dive, datalog.dives.len());

        draw_group_damage_plot(ui, options, statlist, name);
    }
}

/// Helper to draw the plot for group damage stats
#[inline]
fn draw_group_damage_plot(ui: &mut Ui, options: &OverlayOptions, mut statlist: Vec<PlayerStats>, name: &str) {
    // TODO: Precalculate this in the DiveLog probably
    let party_damage = statlist.iter().fold(0, |acc, player| acc + player.total_damage_dealt) as f64;
    
    statlist.sort_by_key(|e| e.total_damage_dealt);
    let bars = {
        statlist.iter().enumerate().map(|(index, stats)| 
            Bar::new(index as f64, stats.total_damage_dealt as f64)
                .width(1.0)
                .fill(options.colors.get_aspect_color(&stats.player_data.class))
        ).collect()
    };
    let texts: Vec<Text> = {
        statlist.iter().enumerate().map(|(index, stats)| 
            Text::new(
                PlotPoint { x: 0.0, y: index as f64 },
                // TODO: consider number seperatoring
                format!("  {} - {} ({:.2}%)",
                    stats.player_data.name,
                    stats.total_damage_dealt,
                    stats.total_damage_dealt as f64 / party_damage * 100.0,
                )
            )
            .anchor(egui::Align2::LEFT_CENTER)
            .color(egui::Color32::WHITE)
        ).collect()
    };
    // let bars: Vec<Bar> = values.iter().enumerate().map(|(c, e)| 
    //     Bar::new(c as f64, *e as f64).width(1.0).name("foo")
    // ).collect();
    
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
                // plot_ui.text(Text::new(PlotPoint{ x:0.0, y: 1.0}, "  meep").anchor(egui::Align2::LEFT_CENTER))
            }
        );
}
