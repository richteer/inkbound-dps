use log::*;
use serde::Serialize;

use super::{Event, playerstats::PlayerStatList};

// // TODO: Probably fold this into PlayerStats
// fn apply_damage(devent: &Event, player_stats: &mut HashMap<String, PlayerStats>) {
//     match devent {
//         Event::DamageDealt(_, name, data) => {
//             if let Some(player) = player_stats.get_mut(name) {
//                 player.total_damage_dealt += data.amount;
//             } else {
//                 player_stats.insert(name.clone(), PlayerStats { name: name.clone(), total_damage_dealt: data.amount, total_damage_received: 0 });
//             }
//         }
//         Event::DamageReceived(_, name, data) => {
//             if let Some(player) = player_stats.get_mut(name) {
//                 player.total_damage_received += data.amount;
//             } else {
//                 player_stats.insert(name.clone(), PlayerStats { name: name.clone(), total_damage_dealt: 0, total_damage_received: data.amount });
//             }
//         }
//         _ => panic!("apply damage called with a non-damage event")
//     }
// }


#[derive(Debug, Serialize, Clone)]
pub struct CombatLog {
    pub player_stats: PlayerStatList,
    // player_stats: HashMap<String, PlayerStats>, // EntityHandle/id -> PlayerStats
    // events: Vec<DamageEvent>,
}

impl CombatLog {
    fn new() -> Self {
        Self {
            player_stats: PlayerStatList::new(),
            // player_stats: HashMap::new(),
            // events: Vec::new(),
        }
    }
    
    fn handle_event(&mut self, event: Event) {
        match event {
            Event::DamageDealt(_, dmg) => self.player_stats.apply_dealt_damage(dmg),
            Event::DamageReceived(_, dmg) => self.player_stats.apply_received_damage(dmg),
            Event::StartCombat(_) => (),
            Event::EndCombat(_) => (),
            // Event::UnitClass(_, name, class) => self.player_stats.set_class(&name, class),
            _ => (),
        }
    }
}

#[derive(Debug, Serialize, Clone)]
pub struct DiveLog {
    pub player_stats: PlayerStatList,
    // player_stats: HashMap<String, PlayerStats>,
    pub combats: Vec<CombatLog>, // Reverse order list of combats, current is always first
}

impl DiveLog {
    pub fn new() -> Self {
        Self {
            player_stats: PlayerStatList::new(),
            combats: Vec::new(),
        }
    }

    fn handle_event(&mut self, event: Event) {
        match event.clone() {
            Event::StartCombat(_) => self.combats.insert(0, CombatLog::new()),
            Event::DamageDealt(_, dmg) => self.player_stats.apply_dealt_damage(dmg),
            Event::DamageReceived(_, dmg) => self.player_stats.apply_received_damage(dmg),
            // Event::UnitClass(_, name, class) => self.player_stats.set_class(&name, class),
            // _ => debug!("received other event: {:?}", event),
            _ => (),
        };

        if let Some(combat) = self.combats.get_mut(0) {
            combat.handle_event(event);
        }
    }

}

#[derive(Serialize, Debug, Clone)]
pub struct DataLog {
    pub dives: Vec<DiveLog>,
}

impl DataLog {
    pub fn new() -> Self {
        Self {
            dives: Vec::new()
        }
    }

    pub fn handle_event(&mut self, event: Event) {
        match event {
            Event::StartDive(_) => {
                debug!("starting new dive");
                self.dives.insert(0, DiveLog::new())
            },
            _ => {
                // debug!("propogating event: {:?}", event);
                if let Some(dive) = self.dives.get_mut(0) {
                    dive.handle_event(event);
                }
            }
            // _ => debug!("{:?}", event),
        }
    }

    pub fn handle_events(&mut self, events: Vec<Event>) {
        events.into_iter().for_each(|e| self.handle_event(e));
    }
}
