use egui::Align2;
use egui_plot::{Plot, Bar, BarChart, AxisHints, Text, PlotPoint};
use inkbound_parser::parser::{DataLog, PlayerStats, DiveLog};
use serde::{Deserialize, Serialize};
use strum::{IntoEnumIterator, EnumIter};

use crate::{options::ColorOptions, OverlayOptions};

use super::{show_dive_selection_box, WindowDisplay, WindowId};

#[derive(Default, PartialEq, Serialize, Deserialize)]
pub enum HistoryMode {
    #[default]
    Split,
    Stacked,
}

impl std::fmt::Display for HistoryMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let out = match self {
            HistoryMode::Split => "Split",
            HistoryMode::Stacked => "Stacked",
        };
        f.write_str(out)
    }
}

#[derive(Default)]
pub struct HistoryState {
    pub dive: usize,
}

#[derive(Default, Serialize, Deserialize, EnumIter, PartialEq, Eq, Clone, Copy)]
pub enum BarOrder {
    AscendingDamage,
    #[default]
    DescendingDamage,
    AscendingName,
    DescendingName,
}

impl std::fmt::Display for BarOrder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            BarOrder::AscendingDamage  => "Damage ⬆",
            BarOrder::DescendingDamage => "Damage ⬇",
            BarOrder::AscendingName    => "Name ⬆",
            BarOrder::DescendingName   => "Name ⬇",
        })
    }
}

#[derive(Serialize, Deserialize)]
#[serde(default)]
// TODO: Re-persist these
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

#[derive(Default, Deserialize, Serialize)]
pub struct HistoryWindow {
    pub options: HistoryOptions,
    #[serde(skip)]
    state: HistoryState,
}

#[typetag::serde]
impl WindowDisplay for HistoryWindow {
    fn show(&mut self, ui: &mut egui::Ui, options: &OverlayOptions, data: &DataLog) {
        self.draw_history_window_new(ui, options, data);
    }

    fn id(&self) -> WindowId {
        self.name()
    }

    fn name(&self) -> String {
        "History".to_string()
    }
}

// TODO: Consider passing in a sort function to the generate functions, or just passing in pre-sorted data
fn sort_players(mut players: Vec<PlayerStats>, sorting: BarOrder) -> Vec<PlayerStats> {
    match sorting {
        BarOrder::AscendingDamage  => players.sort_by_key(|p| p.total_damage_dealt),
        BarOrder::DescendingDamage => players.sort_by_key(|p| std::cmp::Reverse(p.total_damage_dealt)),
        BarOrder::AscendingName    => players.sort_by(|a,b| a.player_data.name.to_lowercase().cmp(&b.player_data.name.to_lowercase())),
        BarOrder::DescendingName   => players.sort_by(|a,b| b.player_data.name.to_lowercase().cmp(&a.player_data.name.to_lowercase())),
    };

    players
}

#[inline]
fn generate_split_bars(dive: &DiveLog, bar_group_width: f64, colors: &ColorOptions, sorting: BarOrder) -> Vec<Bar> {
    dive.combats.iter().rev().enumerate().map(|(combat_index, combat)| {
        let players: Vec<PlayerStats> = combat.player_stats.player_stats.values().cloned().collect();
        let players = sort_players(players, sorting);
        players.iter().enumerate().map(|(pind, p)| {
            let pind = pind as f64;
            let num_players = combat.player_stats.player_stats.len() as f64;
            let bar_width = bar_group_width / num_players;
            // let x_offset = ((pind + bar_group_width / 2.0) * width) - (bar_group_width / 2.0);
            let x_offset = pind * bar_width - ((bar_group_width - bar_width) / 2.0);
            Bar::new(combat_index as f64 + x_offset + 1.0, p.total_damage_dealt as f64)
                .name(format!("{} {}", p.player_data.name, combat_index + 1))
                .width(bar_width as f64)
                .fill(colors.get_aspect_color(&p.player_data.class))
        }).collect::<Vec<Bar>>()
    }).flatten().collect()
}

#[inline]
fn generate_stacked_bars(dive: &DiveLog, bar_width: f64, show_stacked_totals: bool, colors: &ColorOptions, sorting: BarOrder) -> (Vec<Bar>, Option<Vec<Text>>) {
    let bars = dive.combats.iter().rev().enumerate().map(|(combat_index, combat)| {
        let players: Vec<PlayerStats> = combat.player_stats.player_stats.values().cloned().collect();
        let players = sort_players(players, sorting);
        players.iter().scan(0, |state, p| {
            *state += p.total_damage_dealt;
            Some((*state,  p))
        }).map(|(previous, p)| {
            Bar::new(combat_index as f64 + 1.0, p.total_damage_dealt as f64)
                .name(format!("{} {}", p.player_data.name, combat_index + 1))
                .base_offset((previous - p.total_damage_dealt) as f64)
                .width(bar_width)
                .fill(colors.get_aspect_color(&p.player_data.class))
        }).collect::<Vec<Bar>>()
    }).flatten().collect();

    // TODO: This totally can be done in one pass with the previous
    let texts = if show_stacked_totals {
        Some(dive.combats.iter().rev().enumerate().map(|(combat_index, combat)| {
            let total_damage_dealt = combat.player_stats.player_stats.values().fold(0, |acc, elem| acc + elem.total_damage_dealt);
            Text::new(
                PlotPoint { x: combat_index as f64 + 1.0, y: total_damage_dealt as f64 },
                format!("{}", total_damage_dealt)
            ).anchor(Align2::CENTER_BOTTOM)
        }).collect())
    } else {
            None
    };

    (bars, texts)
}


impl HistoryWindow {

    pub fn draw_history_window_new(&mut self, ui: &mut egui::Ui, options: &OverlayOptions, datalog: &DataLog) {
        show_dive_selection_box(ui, &mut self.state.dive, datalog.dives.len());
        egui::ComboBox::from_label("Mode")
            .selected_text(self.options.mode.to_string())
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut self.options.mode, HistoryMode::Split, "Split");
                ui.selectable_value(&mut self.options.mode, HistoryMode::Stacked, "Stacked");
            })
            .response.on_hover_text("Select which mode to render the history plot.\n\nSplit - Each player has their own vertical bar grouped by combat.\nStacked - Player damage bars are stacked on top of each other, with the total length of the bar representing total group damage.");

        egui::ComboBox::from_label("Bar Order")
            .selected_text(self.options.bar_order.to_string())
            .show_ui(ui, |ui|{
                BarOrder::iter().for_each(|e| { ui.selectable_value(&mut self.options.bar_order, e, e.to_string()); })
            }).response.on_hover_text("The order that the bars will be rendered.\n\nIn split mode, ascending(⬆) is left to right.\nIn stacked mode, ascending(⬆) is bottom-up.");

        match self.options.mode {
            HistoryMode::Split => {
                ui.add(egui::Slider::new(&mut self.options.group_bar_width, 0.25..=1.0)
                    .max_decimals(2)
                    .text("Bar Group Width"));
            },
            HistoryMode::Stacked => {
                ui.add(egui::Slider::new(&mut self.options.stacked_bar_width, 0.25..=1.0)
                    .max_decimals(2)
                    .text("Bar Width"));
                ui.checkbox(&mut self.options.stacked_show_totals, "Show Totals");
            },
        }

        ui.separator();

        let dive = if let Some(dive) = datalog.dives.get(self.state.dive) {
            dive
        } else {
            return
        };

        let (bars, texts) = match self.options.mode {
            HistoryMode::Split => (generate_split_bars(dive, self.options.group_bar_width, &options.colors, self.options.bar_order), None),
            HistoryMode::Stacked => generate_stacked_bars(dive, self.options.stacked_bar_width, self.options.stacked_show_totals, &options.colors, self.options.bar_order),
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
}
