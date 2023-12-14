use std::collections::HashMap;
use serde::Serialize;

use super::{DamageReceivedEventData, DamageDealtEventData, PlayerData};

/// Ongoing Statistics for a particular Player
#[derive(Debug, Serialize, Clone)]
pub struct PlayerStats {
    // pub name: String,
    // pub class: Option<String>,
    pub player_data: PlayerData,
    pub total_damage_dealt: i64,
    pub total_damage_received: i64,
    pub skill_totals: HashMap<String, i64>,
    // Subset of skill_totals that only contains crit damage
    pub crit_totals: HashMap<String, i64>,
    pub orb_pickups: i64,
    // TODO: status effects applied, etc
}

impl PlayerStats {
    pub fn new(player_data: PlayerData) -> Self{
        Self {
            // name,
            // class: None,
            player_data,
            total_damage_dealt: 0,
            total_damage_received: 0,
            skill_totals: HashMap::new(),
            crit_totals: HashMap::new(),
            orb_pickups: 0,
        }
    }

    pub fn apply_dealt_damage(&mut self, dmg: DamageDealtEventData) {
        // I don't love the clone here, but it at least prevents the bleh if/else
        self.skill_totals.entry(dmg.ability.clone()).and_modify(|total| *total += dmg.amount).or_insert(dmg.amount);

        if dmg.crit {
            self.crit_totals.entry(dmg.ability).and_modify(|total| *total += dmg.amount).or_insert(dmg.amount);
        }
        
        self.total_damage_dealt += dmg.amount;
    }

    pub fn apply_received_damage(&mut self, dmg: DamageReceivedEventData) {
        self.total_damage_received += dmg.amount;
    }

    pub fn increment_orbs(&mut self) {
        self.orb_pickups += 1;
    }
}

#[derive(Debug, Serialize, Clone)]
pub struct PlayerStatList {
    #[serde(flatten)]
    pub player_stats: HashMap<String, PlayerStats>,
}

impl PlayerStatList {
    pub fn new() -> Self {
        Self {
            player_stats: HashMap::new(),
        }
    }

    // TODO: Remove the clones in these two functions
    pub fn apply_dealt_damage(&mut self, dmg: DamageDealtEventData) {
        let name = &dmg.source.name;
        let player = if let Some(player) = self.player_stats.get_mut(name) {
            player
        } else {
            self.player_stats.insert(name.clone(), PlayerStats::new(dmg.source.clone()));
            self.player_stats.get_mut(name).unwrap()
        };

        player.apply_dealt_damage(dmg);
    }

    pub fn apply_received_damage(&mut self, dmg: DamageReceivedEventData) {
        let name = &dmg.target.name;
        let player = if let Some(player) = self.player_stats.get_mut(name) {
            player
        } else {
            self.player_stats.insert(name.clone(), PlayerStats::new(dmg.target.clone()));
            self.player_stats.get_mut(name).unwrap()
        };

        player.apply_received_damage(dmg);
    }

    pub fn apply_orb_pickup(&mut self, player: PlayerData) {
        self.player_stats.entry(player.name.clone())
            .and_modify(|e| e.increment_orbs())
            .or_insert(PlayerStats::new(player));
    }

    // pub fn set_class(&mut self, name: &String, class: String) {
    //     if let Some(player) = self.player_stats.get_mut(name) {
    //         if player.class.is_none() {
    //             player.class = Some(class);
    //         } else {
    //             trace!("player class already set");
    //         }
    //     } else {
    //         trace!("ignoring update for class");
    //     }
    // }
}
