use egui::Ui;
use egui_plot::{Text, PlotPoint, BarChart, Plot, Bar};
use inkbound_parser::parser::{PlayerStats, DataLog};
use serde::{Deserialize, Serialize};

use crate::OverlayOptions;

use super::{WindowDisplay, DamageTotalsMode, DiveCombatSplit, DiveCombatSelectionState};

#[derive(Default, Debug)]
pub struct GroupDamageState {
    pub dive: usize,
    pub combat: usize,
}

#[derive(Default, Deserialize, Serialize, Debug)]
pub struct GroupDamageWindow {
    #[serde(skip)]
    state: DiveCombatSelectionState,
    mode: DamageTotalsMode,
}

#[typetag::serde]
impl WindowDisplay for GroupDamageWindow {
    fn show(&mut self, ui: &mut egui::Ui, options: &OverlayOptions, data: &DataLog) {
        ui.collapsing("⛭", |ui| {
            self.mode_selection(ui);
            self.show_selection_boxes(ui, data);
        });

        if let Some(stats) = self.get_current_player_stat_list(data) {
            self.draw_group_damage_plot(ui, options, stats.values().collect());
        } else {
            ui.label(super::NO_DATA_MSG.to_string());
        };
    }

    fn name(&self) -> String {
        format!("Group Damage: {}", self.mode.to_string())
    }
}

impl DiveCombatSplit for GroupDamageWindow {
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

impl GroupDamageWindow {
    /// Helper to draw the plot for group damage stats
    #[inline]
    fn draw_group_damage_plot(&self, ui: &mut Ui, options: &OverlayOptions, mut statlist: Vec<&PlayerStats>) {
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
                    // plot_ui.text(Text::new(PlotPoint{ x:0.0, y: 1.0}, "  meep").anchor(egui::Align2::LEFT_CENTER))
                }
            );
    }
}
