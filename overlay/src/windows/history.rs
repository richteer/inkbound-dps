use egui::Align2;
use egui_plot::{Plot, Bar, BarChart, AxisHints, Text, PlotPoint};
use inkbound_parser::parser::{DataLog, PlayerStats, DiveLog};
use serde::{Deserialize, Serialize};
use strum::{IntoEnumIterator, EnumIter};

use crate::{options::ColorOptions, OverlayOptions};

use super::{show_dive_selection_box, WindowDisplay, extractors::{StatSelectionState, StatSelection}};

#[derive(Default, PartialEq, Serialize, Deserialize, Debug, EnumIter, Clone, Copy)]
pub enum HistoryMode {
    #[default]
    Split,
    Stacked,
    Percent,
}

impl std::fmt::Display for HistoryMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let out = match self {
            HistoryMode::Split => "Split",
            HistoryMode::Stacked => "Stacked",
            HistoryMode::Percent => "Percent",
        };
        f.write_str(out)
    }
}

#[derive(Default, Debug)]
pub struct HistoryState {
    pub dive: usize,
}

#[derive(Default, Debug, Serialize, Deserialize, EnumIter, PartialEq, Eq, Clone, Copy)]
pub enum BarOrder {
    AscendingStat,
    #[default]
    DescendingStat,
    AscendingName,
    DescendingName,
}

impl std::fmt::Display for BarOrder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            BarOrder::AscendingStat  => "Stat ⬆",
            BarOrder::DescendingStat => "Stat ⬇",
            BarOrder::AscendingName  => "Name ⬆",
            BarOrder::DescendingName => "Name ⬇",
        })
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(default)]
pub struct HistoryOptions {
    pub show: bool,
    pub mode: HistoryMode,
    pub group_bar_width: f64,
    pub bar_order: BarOrder,
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
            bar_order: BarOrder::default(),
        }
    }
}


#[derive(Default, Deserialize, Serialize, Debug)]
#[serde(default)]
pub struct HistoryWindow {
    pub options: HistoryOptions,
    #[serde(skip)]
    state: HistoryState,
    pub stat_selection: StatSelectionState,
}

#[typetag::serde]
impl WindowDisplay for HistoryWindow {
    fn show(&mut self, ui: &mut egui::Ui, options: &OverlayOptions, data: &DataLog) {
        self.draw_history_window(ui, options, data);
    }

    fn name(&self) -> String {
        format!("History: {}", self.stat_selection)
    }
}

impl StatSelection for HistoryWindow {
    fn get_stat_selection(&self) -> &StatSelectionState {
        &self.stat_selection
    }

    fn get_stat_selection_mut(&mut self) -> &mut StatSelectionState {
        &mut self.stat_selection
    }
}

impl HistoryWindow {

    pub fn draw_history_window(&mut self, ui: &mut egui::Ui, options: &OverlayOptions, datalog: &DataLog) {
        ui.collapsing("⛭", |ui| {
            show_dive_selection_box(ui, &mut self.state.dive, datalog.dives.len());

            self.show_stat_selection_box(ui);

            ui.horizontal(|ui| {
                ui.selectable_value(&mut self.options.mode, HistoryMode::Split, HistoryMode::Split.to_string())
                    .on_hover_text("Each player has their own vertical bar, grouped horizontally by combat.");
                ui.selectable_value(&mut self.options.mode, HistoryMode::Stacked, HistoryMode::Stacked.to_string())
                    .on_hover_text("Player stat bars are stacked on top of each other, with the total length of the bar repesenting the total group stat value.");
                ui.selectable_value(&mut self.options.mode, HistoryMode::Percent, HistoryMode::Percent.to_string())
                    .on_hover_text("A single full-height (100%) bar is segmented by each player's percentage contribution to the total stat value.");
                ui.label("Mode")
                    .on_hover_text("Select which mode to use to render the history plot.\nMouse over each option for more details.");
            });

            egui::ComboBox::from_label("Bar Order")
                .selected_text(self.options.bar_order.to_string())
                .show_ui(ui, |ui|{
                    BarOrder::iter().for_each(|e| { ui.selectable_value(&mut self.options.bar_order, e, e.to_string()); })
                }).response.on_hover_text("The order that the bars will be rendered.\n\nIn split mode, ascending(⬆) is left to right.\nIn stacked/percent mode, ascending(⬆) is bottom-up.");

            match self.options.mode {
                HistoryMode::Split => {
                    ui.add(egui::Slider::new(&mut self.options.group_bar_width, 0.25..=1.0)
                        .max_decimals(2)
                        .text("Bar Group Width"));
                },
                HistoryMode::Stacked | HistoryMode::Percent => {
                    ui.add(egui::Slider::new(&mut self.options.stacked_bar_width, 0.25..=1.0)
                        .max_decimals(2)
                        .text("Bar Width"));
                    ui.checkbox(&mut self.options.stacked_show_totals, "Show Totals");
                },
            }
        });

        ui.separator();

        let dive = if let Some(dive) = datalog.dives.get(self.state.dive) {
            dive
        } else {
            return
        };

        let (bars, texts) = match self.options.mode {
            HistoryMode::Split => (self.generate_split_bars(dive, &options.colors), None),
            HistoryMode::Stacked => self.generate_stacked_bars(dive, &options.colors, false),
            HistoryMode::Percent => self.generate_stacked_bars(dive, &options.colors, true),
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
    }

    #[inline]
    fn generate_split_bars(&self, dive: &DiveLog, colors: &ColorOptions) -> Vec<Bar> {
        let bar_group_width = self.options.group_bar_width;
        dive.combats.iter().rev().enumerate().flat_map(|(combat_index, combat)| {
            let players: Vec<PlayerStats> = combat.player_stats.player_stats.values().cloned().collect();
            let players = self.sort_players(players, self.options.bar_order);
            players.iter().enumerate().map(|(pind, p)| {
                let pind = pind as f64;
                let num_players = combat.player_stats.player_stats.len() as f64;
                let bar_width = bar_group_width / num_players;
                // let x_offset = ((pind + bar_group_width / 2.0) * width) - (bar_group_width / 2.0);
                let x_offset = pind * bar_width - ((bar_group_width - bar_width) / 2.0);
                Bar::new(combat_index as f64 + x_offset + 1.0, self.extract_stat(p))
                    .name(format!("{} {}", p.player_data.name, combat_index + 1))
                    .width(bar_width)
                    .fill(colors.get_aspect_color(&p.player_data.class))
            }).collect::<Vec<Bar>>()
        }).collect()
    }

    #[inline]
    fn generate_stacked_bars(&self, dive: &DiveLog, colors: &ColorOptions, percent: bool) -> (Vec<Bar>, Option<Vec<Text>>) {
        let bars = dive.combats.iter().rev().enumerate().flat_map(|(combat_index, combat)| {
            let players: Vec<PlayerStats> = combat.player_stats.player_stats.values().cloned().collect();
            // Skip sum calculation if not in percent mode, save a bit of effort
            let total: f64 = if percent { players.iter().map(|e| self.extract_stat(e)).sum() } else { 0.0 };
            let players = self.sort_players(players, self.options.bar_order);
            players.iter().scan(0.0, |state, p| {
                *state += if percent {
                    self.extract_stat(p) / total * 100.0
                } else {
                    self.extract_stat(p)
                };
                Some((*state,  p))
            }).map(|(previous, p)| {
                let value = if percent {
                    self.extract_stat(p) / total * 100.0
                } else {
                    self.extract_stat(p)
                };
                Bar::new(combat_index as f64 + 1.0, value)
                    .name(format!("{} {}", p.player_data.name, combat_index + 1))
                    .base_offset(previous - value)
                    .width(self.options.stacked_bar_width)
                    .fill(colors.get_aspect_color(&p.player_data.class))
            }).collect::<Vec<Bar>>()
        }).collect();

        // TODO: This totally can be done in one pass with the previous
        let texts = if self.options.stacked_show_totals {
            Some(dive.combats.iter().rev().enumerate().map(|(combat_index, combat)| {
                let stat = combat.player_stats.player_stats.values().fold(0.0, |acc, elem| acc + self.extract_stat(elem));
                Text::new(
                    PlotPoint { x: combat_index as f64 + 1.0, y: if percent { 100.0 } else { stat } },
                    format!("{}", stat)
                ).anchor(Align2::CENTER_BOTTOM)
            }).collect())
        } else {
            None
        };

        (bars, texts)
    }

    // TODO: Consider passing in a sort function to the generate functions, or just passing in pre-sorted data
    fn sort_players(&self, mut players: Vec<PlayerStats>, sorting: BarOrder) -> Vec<PlayerStats> {
        match sorting {
            // TODO: Probably shouldn't just cast these to ints, but it probably doesn't matter?
            BarOrder::AscendingStat  => players.sort_by_key(|p| self.extract_stat(p) as i64),
            BarOrder::DescendingStat => players.sort_by_key(|p| std::cmp::Reverse(self.extract_stat(p) as i64)),
            BarOrder::AscendingName  => players.sort_by(|a,b| a.player_data.name.to_lowercase().cmp(&b.player_data.name.to_lowercase())),
            BarOrder::DescendingName => players.sort_by(|a,b| b.player_data.name.to_lowercase().cmp(&a.player_data.name.to_lowercase())),
        };

        players
    }
}
