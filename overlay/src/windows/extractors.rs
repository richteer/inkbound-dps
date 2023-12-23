use inkbound_parser::parser::PlayerStats;
use serde::{Deserialize, Serialize};
use strum::EnumIter;

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

// TODO: I don't like the hardcoding here. Consider user-parameterizing, or defining an allowlist
fn extract_poison_applied(player: &PlayerStats) -> ExtractType {
    *player.status_applied.get("Poison").unwrap_or(&0) as ExtractType
}

fn extract_burn_applied(player: &PlayerStats) -> ExtractType {
    *player.status_applied.get("Burn").unwrap_or(&0) as ExtractType
}

fn extract_bleed_applied(player: &PlayerStats) -> ExtractType {
    *player.status_applied.get("Bleed").unwrap_or(&0) as ExtractType
}

fn extract_frostbite_applied(player: &PlayerStats) -> ExtractType {
    *player.status_applied.get("Frostbite").unwrap_or(&0) as ExtractType
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
    PoisonApplied,
    BurnApplied,
    BleedApplied,
    FrostbiteApplied,
    OrbCount,
    DamagePerOrb,
}

impl std::fmt::Display for StatExtractionFunc {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            StatExtractionFunc::TotalDamageDealt => "Damage Dealt",
            StatExtractionFunc::TotalCritDamageDealt => "Crit Damage Dealt",
            StatExtractionFunc::TotalDamageReceived => "Damage Received",
            StatExtractionFunc::PoisonApplied => "Poison Stacks",
            StatExtractionFunc::BurnApplied => "Burn Stacks",
            StatExtractionFunc::BleedApplied => "Bleed Stacks",
            StatExtractionFunc::FrostbiteApplied => "Frostbite Stacks",
            StatExtractionFunc::OrbCount => "Orbs Consumed",
            StatExtractionFunc::DamagePerOrb => "Damage Per Orb",
        })
    }
}

impl StatExtractionFunc {
    pub fn to_func(&self) -> fn(&PlayerStats) -> ExtractType {
        match self {
            StatExtractionFunc::TotalDamageDealt => extract_total_damage_dealt,
            StatExtractionFunc::TotalCritDamageDealt => extract_total_crit_damage_dealt,
            StatExtractionFunc::TotalDamageReceived => extract_total_damage_received,
            StatExtractionFunc::PoisonApplied => extract_poison_applied,
            StatExtractionFunc::BurnApplied => extract_burn_applied,
            StatExtractionFunc::BleedApplied => extract_bleed_applied,
            StatExtractionFunc::FrostbiteApplied => extract_frostbite_applied,
            StatExtractionFunc::OrbCount => extract_orb_count,
            StatExtractionFunc::DamagePerOrb => extract_damage_per_orb,
        }
    }
}
