use inkbound_parser::parser::PlayerStats;
use serde::{Deserialize, Serialize};
use strum::{EnumIter, IntoEnumIterator};

// In case I change my mind...
type ExtractType = f64;

fn extract_total_damage_dealt(player: &PlayerStats) -> ExtractType {
    player.total_damage_dealt as ExtractType
}

fn extract_total_crit_damage_dealt(player: &PlayerStats) -> ExtractType {
    player.crit_totals.values().sum::<i64>() as f64
}

fn extract_total_damage_received(player: &PlayerStats) -> ExtractType {
    player.total_damage_received as ExtractType
}

fn extract_percent_crit_damage(player: &PlayerStats) -> ExtractType {
    extract_total_crit_damage_dealt(player) / player.total_damage_dealt as ExtractType * 100.0
}

fn extract_status_effect_applied(player: &PlayerStats, status: &str) -> ExtractType {
    *player.status_applied.get(status).unwrap_or(&0) as ExtractType
}

fn extract_orb_count(player: &PlayerStats) -> ExtractType {
    player.orb_pickups as ExtractType
}

fn extract_damage_per_orb(player: &PlayerStats) -> ExtractType {
    if player.orb_pickups != 0 {
        player.total_damage_dealt as f64 / player.orb_pickups as f64
    } else {
        0.0
    }
}

#[derive(Default, Debug, Deserialize, Serialize, EnumIter, PartialEq, Clone)]
pub enum StatExtractionFunc {
    #[default]
    TotalDamageDealt,
    TotalCritDamageDealt,
    TotalDamageReceived,
    PercentCritDamage,
    StatusEffectApplied(String),
    OrbCount,
    DamagePerOrb,
}

impl std::fmt::Display for StatExtractionFunc {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&match self {
            StatExtractionFunc::TotalDamageDealt => "Damage Dealt".to_string(),
            StatExtractionFunc::TotalCritDamageDealt => "Crit Damage Dealt".to_string(),
            StatExtractionFunc::TotalDamageReceived => "Damage Received".to_string(),
            StatExtractionFunc::PercentCritDamage => "Percent Crit Damage".to_string(),
            StatExtractionFunc::StatusEffectApplied(status) => 
                if status.is_empty() {
                    "Status Effect Applied".to_string()
                } else {
                    format!("{status} Stacks")
                },
            StatExtractionFunc::OrbCount => "Orbs Consumed".to_string(),
            StatExtractionFunc::DamagePerOrb => "Damage Per Orb".to_string(),
        })
    }
}

impl StatExtractionFunc {
    /// Apply the configured extraction function to a PlayerStats.
    /// Handles any internal options under the hood.
    pub fn extract_stat(&self, player: &PlayerStats) -> ExtractType {
        match &self {
            StatExtractionFunc::TotalDamageDealt => extract_total_damage_dealt(player),
            StatExtractionFunc::TotalCritDamageDealt => extract_total_crit_damage_dealt(player),
            StatExtractionFunc::TotalDamageReceived => extract_total_damage_received(player),
            StatExtractionFunc::PercentCritDamage => extract_percent_crit_damage(player),
            StatExtractionFunc::StatusEffectApplied(status) => extract_status_effect_applied(player, &status),
            StatExtractionFunc::OrbCount => extract_orb_count(player),
            StatExtractionFunc::DamagePerOrb => extract_damage_per_orb(player),
        }
    }

    pub fn extract_formatted_stat(&self, player: &PlayerStats) -> String {
        let stat = self.extract_stat(player);
        format!("{:.*}",
            if stat.trunc() == stat {
                0
            } else {
                2
            },
            stat
        )
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct StatSelectionState {
    pub selection: StatExtractionFunc,
    pub status_selection: String,
}

impl std::fmt::Display for StatSelectionState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Override the display of specific extractors, mostly those with additional parameters
        match &self.selection {
            StatExtractionFunc::StatusEffectApplied(status) => if !status.is_empty() {
                f.write_str(&format!("{} Stacks", self.status_selection))
            } else {
                self.selection.fmt(f)
            }
            _ => self.selection.fmt(f)
        }
    }
}

pub trait StatSelection {
    // TODO: figure out if there's a way to avoid needing both
    fn get_stat_selection<'a>(&'a self) -> &'a StatSelectionState;
    fn get_stat_selection_mut<'a>(&'a mut self) -> &'a mut StatSelectionState;

    fn extract_stat(&self, player: &PlayerStats) -> ExtractType {
        self.get_stat_selection().selection.extract_stat(player)
    }

    fn show_stat_selection_box(&mut self, ui: &mut egui::Ui) {
        let stat_selection = self.get_stat_selection_mut();
        egui::ComboBox::from_label("Stat")
            .selected_text(stat_selection.selection.to_string())
            .show_ui(ui, |ui| {
                for statfunc in StatExtractionFunc::iter() {
                    ui.selectable_value(&mut stat_selection.selection, statfunc.clone(), statfunc.to_string());
                }
            });
        if let StatExtractionFunc::StatusEffectApplied(_) = &stat_selection.selection {
            egui::ComboBox::from_label("Status Effect")
                .selected_text(stat_selection.status_selection.to_string())
                .show_ui(ui, |ui| {
                    // TODO: configure this, perhaps allow custom overrides
                    for status in ["Poison", "Burn", "Bleed", "Frostbite"].into_iter() {
                        ui.selectable_value(&mut stat_selection.status_selection, status.to_string(), status);
                    }
                });
            stat_selection.selection = StatExtractionFunc::StatusEffectApplied(stat_selection.status_selection.clone());
        }
    }
}
