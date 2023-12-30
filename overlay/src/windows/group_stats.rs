use std::collections::HashMap;

use egui::Ui;
use egui_plot::{Text, PlotPoint, BarChart, Plot, Bar};
use inkbound_parser::parser::{PlayerStats, DataLog};
use serde::{Deserialize, Serialize};
use interpolator::*;
use derivative::Derivative;

use super::{extractors::{StatSelection, StatSelectionState}, FormatSelection, AspectAbbv, div_or_zero};

use crate::OverlayOptions;

use super::{WindowDisplay, DiveCombatSelection, DiveCombatSplit, DiveCombatSelectionState};

#[derive(Default, Debug)]
pub struct GroupStatsState {
    pub dive: usize,
    pub combat: usize,
}

#[derive(Deserialize, Serialize, Debug, Derivative)]
#[serde(default)]
#[derivative(Default)]
pub struct GroupStatsWindow {
    #[serde(skip)]
    state: DiveCombatSelectionState,
    mode: DiveCombatSelection,
    stat_selection: StatSelectionState,
    #[derivative(Default(value = "DEFAULT_FORMAT.to_string()"))]
    format: String,
}

#[typetag::serde]
impl WindowDisplay for GroupStatsWindow {
    fn show(&mut self, ui: &mut egui::Ui, options: &OverlayOptions, data: &DataLog) {
        ui.collapsing("⛭", |ui| {
            self.mode_selection(ui);
            self.show_selection_boxes(ui, data);
            self.show_stat_selection_box(ui);
            self.show_format_selection_box(ui);
        });

        if let Some(stats) = self.get_current_player_stat_list(data) {
            self.draw_group_stats_plot(ui, options, stats.values().collect());
        } else {
            ui.label(super::NO_DATA_MSG.to_string());
        };
    }

    fn name(&self) -> String {
        format!("{}: {}", self.stat_selection, self.mode)
    }
}

impl DiveCombatSplit for GroupStatsWindow {
    fn mode(&mut self) -> &mut DiveCombatSelection {
        &mut self.mode
    }

    fn set_mode(&mut self, mode: DiveCombatSelection) {
        self.mode = mode
    }

    fn state(&mut self) -> &mut super::DiveCombatSelectionState {
        &mut self.state
    }
}

impl StatSelection for GroupStatsWindow {
    fn get_stat_selection(&self) -> &super::extractors::StatSelectionState {
        &self.stat_selection
    }

    fn get_stat_selection_mut(&mut self) -> &mut super::extractors::StatSelectionState {
        &mut self.stat_selection
    }
}

impl FormatSelection for GroupStatsWindow {
    fn get_format(&mut self) -> &mut String {
        &mut self.format
    }

    fn default_format() -> &'static str {
        DEFAULT_FORMAT
    }

    fn hover_text() -> &'static str {
        "Valid options:
{name}: Name of the player
{stat}: Value of the selected stat
{percent}: Percent of the group's total stat
{class}: Player's class
{cls}: Abbreviated player's class
"
    }
}

static DEFAULT_FORMAT: &str = "  {name} - {stat} ({percent:.2}%)";

struct StatInfo {
    name: String,
    stat: f64,
    percent: f64,
    class: String,
    cls: String,
}

impl StatInfo {
    pub fn new(stats: &PlayerStats, stat: f64, total: f64) -> Self {
        Self {
            name: stats.player_data.name.clone(),
            stat,
            percent: div_or_zero(stat, total) * 100.0,
            class: stats.player_data.class.to_string(),
            cls: stats.player_data.class.abbv(),
        }
    }

    pub fn to_map(&self) -> HashMap<&str, Formattable<'_>> {
        [
            ("name", Formattable::display(&self.name)),
            ("stat", Formattable::float(&self.stat)),
            ("percent", Formattable::float(&self.percent)),
            ("class", Formattable::display(&self.class)),
            ("cls", Formattable::display(&self.cls)),
        ].into_iter().collect()
    }
}


impl GroupStatsWindow {
    /// Helper to draw the plot for group damage stats
    #[inline]
    fn draw_group_stats_plot(&self, ui: &mut Ui, options: &OverlayOptions, mut statlist: Vec<&PlayerStats>) {
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
                let info = StatInfo::new(stats, self.extract_stat(stats), total_stat);
                let args = info.to_map();

                Text::new(
                    PlotPoint { x: 0.0, y: index as f64 },
                    interpolator::format(&self.format, &args).unwrap_or(self.format.clone())
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
