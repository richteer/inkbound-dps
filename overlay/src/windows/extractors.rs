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

#[derive(Default, Debug, Deserialize, Serialize, EnumIter, PartialEq, Clone, Copy)]
pub enum StatExtractionFunc {
    #[default]
    TotalDamageDealt,
    TotalCritDamageDealt,
    TotalDamageReceived,
    StatusEffectApplied,
    OrbCount,
    DamagePerOrb,
}

impl std::fmt::Display for StatExtractionFunc {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            StatExtractionFunc::TotalDamageDealt => "Damage Dealt",
            StatExtractionFunc::TotalCritDamageDealt => "Crit Damage Dealt",
            StatExtractionFunc::TotalDamageReceived => "Damage Received",
            StatExtractionFunc::StatusEffectApplied => "Status Effect Applied",
            StatExtractionFunc::OrbCount => "Orbs Consumed",
            StatExtractionFunc::DamagePerOrb => "Damage Per Orb",
        })
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct StatSelectionState {
    pub selection: StatExtractionFunc,
    pub status_selection: String,
}

impl std::fmt::Display for StatSelectionState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Override the display of specific extractors, mostly those with additional parameters
        match self.selection {
            StatExtractionFunc::StatusEffectApplied => if !self.status_selection.is_empty() {
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

    /// Apply the configured extraction function to a PlayerStats.
    /// Handles any internal options under the hood.
    fn extract_stat(&self, player: &PlayerStats) -> ExtractType {
        let stat_selection = self.get_stat_selection();
        match stat_selection.selection {
            StatExtractionFunc::TotalDamageDealt => extract_total_damage_dealt(player),
            StatExtractionFunc::TotalCritDamageDealt => extract_total_crit_damage_dealt(player),
            StatExtractionFunc::TotalDamageReceived => extract_total_damage_received(player),
            StatExtractionFunc::StatusEffectApplied => extract_status_effect_applied(player, &stat_selection.status_selection),
            StatExtractionFunc::OrbCount => extract_orb_count(player),
            StatExtractionFunc::DamagePerOrb => extract_damage_per_orb(player),
        }
    }

    fn show_stat_selection_box(&mut self, ui: &mut egui::Ui) {
        let stat_selection = self.get_stat_selection_mut();
        egui::ComboBox::from_label("Stat")
            .selected_text(stat_selection.selection.to_string())
            .show_ui(ui, |ui| {
                for statfunc in StatExtractionFunc::iter() {
                    ui.selectable_value(&mut stat_selection.selection, statfunc, statfunc.to_string());
                }
            });
        if stat_selection.selection == StatExtractionFunc::StatusEffectApplied {
            egui::ComboBox::from_label("Status Effect")
                .selected_text(stat_selection.status_selection.to_string())
                .show_ui(ui, |ui| {
                    // TODO: configure this, perhaps allow custom overrides
                    for status in ["Poison", "Burn", "Bleed", "Frostbite"].into_iter() {
                        ui.selectable_value(&mut stat_selection.status_selection, status.to_string(), status);
                    }
                });
        }
    }
}
