mod logparser;

pub use logparser::LogParser;

mod playerstats;
pub use playerstats::{PlayerStats, PlayerStatList};
mod loggers;
pub use loggers::*;
use serde::Serialize;

use crate::aspects::Aspect;

#[derive(PartialEq, Debug, Clone)]
pub enum Entity {
    Id(i64),
    Player(PlayerData),
}

#[derive(PartialEq, Debug, Clone, Serialize)]
pub struct PlayerData {
    pub name: String,
    pub class: Aspect,
    pub id: i64,
}

#[derive(PartialEq, Debug, Clone)]
pub struct DamageEventData {
    source: Entity,
    target: Entity,
    amount: i64,
    ability: String,
    crit: bool,
    dodged: bool,
}

#[derive(PartialEq, Debug, Clone)]
pub struct DamageDealtEventData {
    source: PlayerData,
    target: Entity,
    amount: i64,
    ability: String,
    crit: bool,
    dodged: bool,
}

#[derive(PartialEq, Debug, Clone)]
pub struct DamageReceivedEventData {
    source: Entity,
    target: PlayerData,
    amount: i64,
    ability: String,
    crit: bool,
    dodged: bool,
}

pub enum DamageDirection {
    Dealt(DamageDealtEventData),
    Received(DamageReceivedEventData),
    EnemyToEnemy(DamageEventData),
    PlayerToPlayer(DamageEventData),
}

// TODO: figure out how to remove the clones here
impl From<DamageEventData> for DamageDirection {
    fn from(dmg: DamageEventData) -> Self {
        match (&dmg.source, &dmg.target) {
            (Entity::Id(_), Entity::Id(_)) => DamageDirection::EnemyToEnemy(dmg),
            (Entity::Player(_), Entity::Player(_)) => DamageDirection::PlayerToPlayer(dmg),
            (Entity::Player(player), Entity::Id(id)) => DamageDirection::Dealt(DamageDealtEventData {
                source: player.clone(),
                target: Entity::Id(*id),
                amount: dmg.amount,
                ability: dmg.ability,
                crit: dmg.crit,
                dodged: dmg.dodged,
            }),
            (Entity::Id(id), Entity::Player(player)) => DamageDirection::Received(DamageReceivedEventData {
                source: Entity::Id(*id),
                target: player.clone(),
                amount: dmg.amount,
                ability: dmg.ability,
                crit: dmg.crit,
                dodged: dmg.dodged,
            }),
        }
    }
}

#[derive(PartialEq, Debug, Clone)]
pub enum TargetUnitTeam {
    Enemy,
    Friendly,
    Unknown(String),
}

#[derive(PartialEq, Debug, Clone)]
pub struct AddStatusEffectData {
    source: Entity,
    target: Entity,
    target_team: TargetUnitTeam,
    effectname: String,
    added: i64,
    newvalue: i64,
}

#[derive(PartialEq, Debug, Clone)]
pub enum Event {
    StartDive(String),
    EndDive(String),
    StartCombat(String),
    EndCombat(String),
    /// line, name, data
    DamageDealt(String, DamageDealtEventData),
    /// line, name, data
    DamageReceived(String, DamageReceivedEventData),
    DamageOther(String, DamageEventData),
    AddStatusEffect(String, AddStatusEffectData),
    // RegisterPlayer(String, String, String),
    NextTurn(String),
    OrbPickup(String, PlayerData),
    SetSelf(String, String),
    // Unknown(String),
}
