use egui::Ui;
use egui_plot::{Text, PlotPoint, BarChart, Plot, Bar};
use inkbound_parser::parser::{PlayerStats, DataLog};
use serde::{Deserialize, Serialize};
use super::extractors::{StatSelection, StatSelectionState};

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
    stat_selection: StatSelectionState,
}

#[typetag::serde]
impl WindowDisplay for GroupDamageWindow {
    fn show(&mut self, ui: &mut egui::Ui, options: &OverlayOptions, data: &DataLog) {
        ui.collapsing("⛭", |ui| {
            self.mode_selection(ui);
            self.show_selection_boxes(ui, data);
            self.show_stat_selection_box(ui);
        });

        if let Some(stats) = self.get_current_player_stat_list(data) {
            self.draw_group_damage_plot(ui, options, stats.values().collect());
        } else {
            ui.label(super::NO_DATA_MSG.to_string());
        };
    }

    fn name(&self) -> String {
        format!("{}: {}", self.stat_selection, self.mode)
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

impl StatSelection for GroupDamageWindow {
    fn get_stat_selection<'a>(&'a self) -> &'a super::extractors::StatSelectionState {
        &self.stat_selection
    }

    fn get_stat_selection_mut<'a>(&'a mut self) -> &'a mut super::extractors::StatSelectionState {
        &mut self.stat_selection
    }
}

impl GroupDamageWindow {
    /// Helper to draw the plot for group damage stats
    #[inline]
    fn draw_group_damage_plot(&self, ui: &mut Ui, options: &OverlayOptions, mut statlist: Vec<&PlayerStats>) {
        let total_stat = statlist.iter().fold(0.0, |acc, player| acc + self.extract_stat(player));

        statlist.sort_by_key(|e| self.extract_stat(e) as i64);
        let bars = {
            statlist.iter().enumerate().map(|(index, stats)|
                Bar::new(index as f64, self.extract_stat(stats))
                    .width(1.0)
                    .fill(options.colors.get_aspect_color(&stats.player_data.class))
            ).collect()
        };
        let texts: Vec<Text> = {
            statlist.iter().enumerate().map(|(index, stats)| {
                let stat = self.extract_stat(stats);
                Text::new(
                    PlotPoint { x: 0.0, y: index as f64 },
                    format!("  {} - {:.*} ({:.2}%)",
                        stats.player_data.name,
                        // TODO: temporary measure, this should probably be handled by user-formatting
                        if stat.trunc() == stat { 0 } else { 2 },
                        stat,
                        if total_stat != 0.0 { stat / total_stat * 100.0 } else { 0.0 },
                    )
                )}
                .anchor(egui::Align2::LEFT_CENTER)
                .color(egui::Color32::WHITE)
            ).collect()
        };

        let chart = BarChart::new(bars)
            .horizontal()
        ;
        Plot::new(format!("{} Plot", self.name()))
            .allow_boxed_zoom(false)
            .allow_drag(false)
            .allow_scroll(false)
            .allow_zoom(false)
            // Hack to fix text offset getting screwed up when there are no bars to render
            .include_x(1.0)
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
