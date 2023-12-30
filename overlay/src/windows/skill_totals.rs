use std::collections::HashMap;

use derivative::Derivative;
use egui::Ui;
use egui_plot::{Plot, BarChart, Bar, Text, PlotPoint};
use inkbound_parser::parser::{PlayerStats, DataLog};
use interpolator::Formattable;
use serde::{Deserialize, Serialize};

use crate::OverlayOptions;

use super::{WindowDisplay, DiveCombatSelection, DiveCombatSplit, DiveCombatSelectionState, PlayerSelection, FormatSelection, div_or_zero};

static DEFAULT_FORMAT: &str = "  {fancy} - {dmg} ({dmg_percent:.2}%)";

#[derive(Default, Debug)]
pub struct SkillTotalsState {
    pub dive: usize,
    pub combat: usize,
}

#[derive(Derivative, Deserialize, Serialize, Debug)]
#[serde(default)]
#[derivative(Default)]
pub struct SkillTotalsWindow {
    #[serde(skip)]
    state: DiveCombatSelectionState,
    mode: DiveCombatSelection,
    player: Option<String>,
    #[derivative(Default(value = "DEFAULT_FORMAT.to_string()"))]
    format: String,
    merge_upgrades: bool,
}

impl PlayerSelection for SkillTotalsWindow {
    fn player(&mut self) -> &mut Option<String> {
        &mut self.player
    }
}

impl DiveCombatSplit for SkillTotalsWindow {
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

#[typetag::serde]
impl WindowDisplay for SkillTotalsWindow {
    fn show(&mut self, ui: &mut egui::Ui, options: &OverlayOptions, data: &DataLog) {
        ui.collapsing("⛭", |ui| {
            let player_stats = self.get_current_player_stat_list(data);

            self.mode_selection(ui);
            self.show_selection_boxes(ui, data);

            if let Some(player_stats) = player_stats {
                self.show_player_selection_box(ui, player_stats);
            }
            ui.checkbox(&mut self.merge_upgrades, "Merge Upgraded Skills")
                .on_hover_text("Merge base and upgraded skills into one bar.\n\nNOTE: This may not work with all skills, those with inconsistent naming may not merge properly.");
            self.show_format_selection_box(ui);
        });

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
            None => base.to_string(),
        }
    }
}

// TODO: probably optimize this, it's probably slow
#[inline]
fn clean_skill_name(name: &str) -> String {
    name
        .replace("_BaseDamage", "")
        .replace("_DamageBase", "")
        .replace("Damage","")
        .replace("_StatusEffect", "")
        .replace("_Legendary","")
        .replace('_'," ")
        .trim()
        .to_string()
}

#[inline]
fn fancy_skill_name(name: &str) -> String {
    clean_skill_name(&name.replace("Upgrade", " ➡"))
}

#[inline]
fn split_skill_name(name: &str) -> (String, Option<String>) {
    if let Some(name) = name.split_once("Upgrade") {
        (clean_skill_name(name.0), Some(clean_skill_name(name.1)))
    } else {
        (clean_skill_name(name), None)
    }
}

impl FormatSelection for SkillTotalsWindow {
    fn get_format(&mut self) -> &mut String {
        &mut self.format
    }

    fn default_format() -> &'static str {
        DEFAULT_FORMAT
    }

    fn hover_text() -> &'static str {
        "Valid options:
{fancy}: Name of the skill + Upgrade name if upgraded
  e.g. Flurry ➡ Barrage
{name}: Name of the skill. Base name if base skill, upgraded name if upgraded.
  e.g. Barrage
{base}: Base, non-upgraded name of the skill
  e.g. Flurry
{label}: Raw name of the skill, you probably don't want this

{dmg}: Damage dealt by the skill
{dmg_percent}: Percentage of overall damage dealt by the skill
{crit}: Damage dealt by the skill as a crit
{crit_dmg_percent}: Percentage damage dealt by this skill as a crit
{crit_total_percent}: Percentage of overall damage dealt by this skill as a crit
"
    }
}

struct SkillInfo {
    label: String,
    name: String,
    base: String,
    fancy: String,
    dmg: i64,
    dmg_percent: f64,
    crit: i64,
    crit_dmg_percent: f64,
    crit_total_percent: f64,
}

impl SkillInfo {
    pub fn new(name: &str, dmg: i64, crit: i64, total_dmg: i64) -> Self {
        let (base, upgrade) = split_skill_name(name);

        Self {
            label: name.to_string(),
            name: upgrade.unwrap_or_else(|| base.clone()),
            base,
            fancy: fancy_skill_name(name),
            dmg,
            dmg_percent: div_or_zero(dmg as f64, total_dmg as f64) * 100.0,
            crit,
            crit_dmg_percent: div_or_zero(crit as f64, dmg as f64) * 100.0,
            crit_total_percent: div_or_zero(crit as f64, total_dmg as f64) * 100.0,
        }
    }

    pub fn to_map(&self) -> HashMap<&str, Formattable> {
        [
            ("label",              Formattable::display(&self.label)),
            ("name",               Formattable::display(&self.name)),
            ("base",               Formattable::display(&self.base)),
            ("fancy",              Formattable::display(&self.fancy)),
            ("dmg",                Formattable::integer(&self.dmg)),
            ("dmg_percent",        Formattable::float(&self.dmg_percent)),
            ("crit",               Formattable::integer(&self.crit)),
            ("crit_dmg_percent",   Formattable::float(&self.crit_dmg_percent)),
            ("crit_total_percent", Formattable::float(&self.crit_total_percent)),
        ].into_iter().collect()
    }
}


impl SkillTotalsWindow {
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

        if self.merge_upgrades {
            // Create a "super" map, of "base skill name" -> "full skill name" -> totals

            // First add only skills that have an upgrade
            let mut name_map: HashMap<String, (String, (i64, i64))> = skill_totals.keys()
                .filter_map(|k|
                    if let (base, Some(_upgrade)) = split_skill_name(k) {
                        Some((base, (k.clone(), (0,0))))
                    } else {
                        None
                    }
                ).collect();

            // Now fold in all the other skills. Non-upgraded skills will be added,
            //  and base skills will be folded into the upgraded variant
            for (label, &(dmg, crit)) in skill_totals.iter() {
                name_map.entry(split_skill_name(label).0)
                    .and_modify(|(_key, (vdmg, vcrit))| {
                        *vdmg += dmg;
                        *vcrit += crit;
                    })
                    .or_insert((label.clone(), (dmg, crit)));
            }

            skill_totals = name_map.into_values().collect();
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

        let total_damage = player_stats.total_damage_dealt;
        let bar_color = options.colors.get_aspect_color(&player_stats.player_data.class);
        let (bars, texts): (Vec<[Bar; 2]>, Vec<SkillInfo>) =
            skill_totals.iter().enumerate().map(|(index, (name, (dmg, crit)))| {
                ([
                    Bar::new(index as f64, *dmg as f64)
                        .width(1.0)
                        .fill(bar_color)
                    ,
                    Bar::new(index as f64, *crit as f64)
                        .width(1.0)
                        .fill(egui::Color32::from_rgba_unmultiplied(0, 0, 0, options.crit_bar_opacity))
                ],
                    SkillInfo::new(name, *dmg, *crit, total_damage)
                )
            }).collect::<Vec<([Bar; 2], SkillInfo)>>().into_iter().unzip();

        let texts: Vec<Text> = {
            texts.into_iter().enumerate().map(|(index, info)| {
                let args = info.to_map();
                Text::new(
                    PlotPoint { x: 0.0, y: index as f64 },
                    interpolator::format(&self.format, &args).unwrap_or(self.format.clone())
                )
                .anchor(egui::Align2::LEFT_CENTER)
                .color(egui::Color32::WHITE)
            }).collect()
        };

        let bars = bars.into_iter().flatten().collect();
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
