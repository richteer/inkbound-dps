use std::collections::HashMap;

use lazy_regex::*;
use log::*;
use serde::Serialize;

use crate::aspects::Aspect;

use super::{Event, DamageEventData, Entity, PlayerData, DamageDirection};

#[derive(Debug, Serialize)]
pub struct LogParser {
    players: HashMap<i64, String>, // id -> name
    classes: HashMap<i64, Aspect>, // id -> pre-translated Aspect
}

#[derive(Debug)]
enum InternalEvent {
    Damage(String, DamageEventData),
    RegisterName(String, i64, String),
    UnitClass(String, i64, String),
    EndDive(String),
    Unknown(String),
}

#[derive(Debug)]
enum ParseEvent {
    Internal(InternalEvent),
    Parsed(Event),
}

trait IdToPlayer {
    fn to_player(self, players: &HashMap<i64, String>, classes: &HashMap<i64, Aspect>) -> Self;
}

impl IdToPlayer for Entity {
    fn to_player(self, players: &HashMap<i64, String>, classes: &HashMap<i64, Aspect>) -> Self {
        match self {
            Entity::Player(_) => self,
            Entity::Id(id) => {
                // TODO: clean this up, it's gross
                let name = if let Some(name) = players.get(&id) {
                    name.clone()
                } else {
                    // Return early, this isn't a player
                    return Entity::Id(id);
                };
                let class = if let Some(class) = classes.get(&id) {
                    class.clone()
                } else {
                    Aspect::Unknown(id.to_string())
                };
                Entity::Player(
                    PlayerData {
                        name,
                        // TODO: probably Option this if it gets detected later
                        class,
                        id,
                    }
                )
            }
        }
    }
}

impl LogParser {
    pub fn new() -> Self {
        Self {
            players: HashMap::new(),
            classes: HashMap::new(),
        }
    }

    fn do_parse(&mut self, line: &str) -> ParseEvent {
        if let Some(caps) = regex!(r"EventOnUnitDamaged.*?TargetUnitHandle:\(EntityHandle:(?<target>\d+)\).*?SourceEntityHandle:\(EntityHandle:(?<source>\d+)\).*?DamageAmount:(?<damage>\d+).*?IsCriticalHit:(?<crit>True|False)-WasDodged:(?<dodged>True|False)-ActionData:ActionData-(?<ability>\w+)_Action").captures(line) {
            ParseEvent::Internal(InternalEvent::Damage(line.to_string(), DamageEventData {
                source: Entity::Id(caps.name("source").unwrap().as_str().parse().unwrap()),
                target: Entity::Id(caps.name("target").unwrap().as_str().parse().unwrap()),
                ability: caps.name("ability").unwrap().as_str().to_string(),
                amount: caps.name("damage").unwrap().as_str().parse().unwrap(),
                crit:   caps.name("crit").unwrap().as_str().to_lowercase().parse().unwrap(),
                dodged: caps.name("dodged").unwrap().as_str().to_lowercase().parse().unwrap(),
            }))
        }
        // NOTE: this matches a lot of extra entity ids, may or may not be issue
        else if let Some(caps) = regex_captures!(r"Setting unit class.*?UnitEntityHandle:\(EntityHandle:(\d+)\)-classType:(\w+)", line) {
            ParseEvent::Internal(InternalEvent::UnitClass(line.to_string(), caps.1.parse().unwrap(), caps.2.to_string()))
        }
        else if let Some(caps) = regex_captures!(r" I (\w+) \(EntityHandle:(\d+)\) is playing ability", line) {
            ParseEvent::Internal(InternalEvent::RegisterName(line.to_string(), caps.2.parse().unwrap(), caps.1.to_string()))
        }
        else if regex_is_match!(r"Party run start triggered", line) {
            ParseEvent::Parsed(Event::StartDive(line.to_string()))
        }
        // TODO: Event::EndDive
        else if regex_is_match!(r"EventOnCombatStarted", line) {
            ParseEvent::Parsed(Event::StartCombat(line.to_string()))
        }
        else if regex_is_match!(r"EventOnCombatEndSequenceStarted", line) {
            ParseEvent::Parsed(Event::EndCombat(line.to_string()))
        }
        // TODO: This seems to appear once per player per turn, so this might need to be post-processed in the state machine
        else if regex_is_match!(r"QuestObjective_TurnCount", line) {
            ParseEvent::Parsed(Event::NextTurn(line.to_string()))
        }
        else if regex_is_match!(r"broadcasting EventSetGameState-EndRun", line) {
            ParseEvent::Internal(InternalEvent::EndDive(line.to_string()))
        }
        else {
            ParseEvent::Internal(InternalEvent::Unknown(line.to_string()))
        }
        // TODO: Clear player/class mappings after a dive
    }

    pub fn parse_line(&mut self, line: &str) -> Option<Event> {
        match self.do_parse(line) {
            ParseEvent::Parsed(event) => Some(event),
            ParseEvent::Internal(InternalEvent::Unknown(line)) => {
                trace!("ignoring line: {}", line);
                None
            },
            ParseEvent::Internal(InternalEvent::RegisterName(_, id, name)) => {
                // debug!("mapping id {} to player {}", id, name);
                self.players.insert(id, name);
                None
            },
            ParseEvent::Internal(InternalEvent::Damage(line, dmg)) => {
                Some(self.convert_damage(line, dmg))
            }
            // Register the EntityId -> Class mapping first, return the information RegisterPlayer when name is received
            ParseEvent::Internal(InternalEvent::UnitClass(_, id, class_id)) => {
                self.classes.insert(id, Aspect::from_id(&class_id));
                None
            },
            ParseEvent::Internal(InternalEvent::EndDive(line)) => {
                self.players.clear();
                self.classes.clear();
                Some(Event::EndDive(line))
            }
        }
    }

    /// Parse multiple lines, convert to a list of Events
    /// May return an empty vector if no lines are useful
    pub fn parse_lines(&mut self, lines: &[&str]) -> Vec<Event> {
        lines.iter().filter_map(|l| self.parse_line(l)).collect()
    }

    fn convert_damage(&self, line: String, mut dmg: DamageEventData) -> Event {
        // debug!("self.players = {:?}", self.players);
        // debug!("self.classes = {:?}", self.classes);
        dmg.source = dmg.source.to_player(&self.players, &self.classes);
        dmg.target = dmg.target.to_player(&self.players, &self.classes);

        // debug!("converting damage: {:?}", dmg);
        match dmg.into() {
            DamageDirection::Dealt(dmg) => Event::DamageDealt(line, dmg),
            DamageDirection::Received(dmg) => Event::DamageReceived(line, dmg),
            DamageDirection::EnemyToEnemy(dmg) => Event::DamageOther(line, dmg),
            DamageDirection::PlayerToPlayer(dmg) => Event::DamageOther(line, dmg),
        }
    }
}


#[cfg(test)]
mod tests {
    use crate::parser::*;
    use super::{ParseEvent, InternalEvent};

    static L_DAMAGE_NORMAL: &'static str = "0T23:17:51 70 I [EventSystem] broadcasting EventOnUnitDamaged-WorldStateChangeDamageUnit-TargetUnitHandle:(EntityHandle:78)-SourceEntityHandle:(EntityHandle:22)-TargetUnitTeam:Enemy-IsInActiveCombat:True-DamageAmount:25-IsCriticalHit:False-WasDodged:False-ActionData:ActionData-Flurry_BaseDamage_Action (UPNE5APs)-AbilityData:AbilityData-Flurry_AbilityData (Flurry my7gMbFo)-StatusEffectData:(none)-LootableData:(none)";
    static L_DAMAGE_CRIT: &'static str = "0T23:17:51 70 I [EventSystem] broadcasting EventOnUnitDamaged-WorldStateChangeDamageUnit-TargetUnitHandle:(EntityHandle:78)-SourceEntityHandle:(EntityHandle:22)-TargetUnitTeam:Enemy-IsInActiveCombat:True-DamageAmount:25-IsCriticalHit:True-WasDodged:False-ActionData:ActionData-Flurry_BaseDamage_Action (UPNE5APs)-AbilityData:AbilityData-Flurry_AbilityData (Flurry my7gMbFo)-StatusEffectData:(none)-LootableData:(none)";
    static L_DAMAGE_DODGED: &'static str = "0T23:17:51 70 I [EventSystem] broadcasting EventOnUnitDamaged-WorldStateChangeDamageUnit-TargetUnitHandle:(EntityHandle:78)-SourceEntityHandle:(EntityHandle:22)-TargetUnitTeam:Enemy-IsInActiveCombat:True-DamageAmount:25-IsCriticalHit:False-WasDodged:True-ActionData:ActionData-Flurry_BaseDamage_Action (UPNE5APs)-AbilityData:AbilityData-Flurry_AbilityData (Flurry my7gMbFo)-StatusEffectData:(none)-LootableData:(none)";
    static L_UNIT_CLASS: &'static str = "0T23:24:03 57 I Setting unit class for animation-UnitEntityHandle:(EntityHandle:22)-classType:C02";
    static L_START_DIVE: &'static str = "0T23:24:45 80 I Party run start triggered - solo party: False";
    static L_START_COMBAT: &'static str = "0T23:26:31 50 I [EventSystem] broadcasting EventOnCombatStarted-WorldStateChangeCombatStarted-CombatZoneHandle:(EntityHandle:68)-TriggeringInteractableHandle:(EntityHandle:69)";
    static L_END_COMBAT: &'static str = "0T23:47:19 32 I [EventSystem] broadcasting EventOnCombatEndSequenceStarted-WorldStateChangeCombatFinishedStartSequence";
    static L_NEXT_TURN: &'static str = "0T23:45:57 21 I Evaluating quest progress for (EntityHandle:16) with 101 active quests. Record variable: QuestObjective_TurnCount";
    static L_REGISTER_NAME: &'static str = "0T23:17:51 66 I TestPlayer (EntityHandle:22) is playing ability AbilityData-Flurry_AbilityData (Flurry my7gMbFo)";

    #[test]
    fn parse_damage_line() {
        let damage_lines = vec![
            L_DAMAGE_NORMAL,
            L_DAMAGE_CRIT,
            L_DAMAGE_DODGED,
        ];

        let mut parser = LogParser::new();
        let damage_lines: Vec<ParseEvent> = damage_lines.iter().map(|elem| parser.do_parse(elem)).collect();
        let mut line = damage_lines.iter();
        
        match line.next() {
            Some(ParseEvent::Internal(InternalEvent::Damage(_, dmg))) => {
                assert_eq!(dmg.source, Entity::Id(22));
                assert_eq!(dmg.target, Entity::Id(78));
                assert_eq!(dmg.ability, "Flurry_BaseDamage".to_string());
                assert_eq!(dmg.amount, 25);
                assert_eq!(dmg.crit, false);
                assert_eq!(dmg.dodged, false);
            },
            Some(_) => assert!(false),
            None => panic!("Update the test case"),
        }
        match line.next() {
            Some(ParseEvent::Internal(InternalEvent::Damage(_, dmg))) => {
                assert_eq!(dmg.source, Entity::Id(22));
                assert_eq!(dmg.target, Entity::Id(78));
                assert_eq!(dmg.ability, "Flurry_BaseDamage".to_string());
                assert_eq!(dmg.amount, 25);
                assert_eq!(dmg.crit, true);
                assert_eq!(dmg.dodged, false);
            },
            Some(_) => assert!(false),
            None => panic!("Update the test case"),
        }
        match line.next() {
            Some(ParseEvent::Internal(InternalEvent::Damage(_, dmg))) => {
                assert_eq!(dmg.source, Entity::Id(22));
                assert_eq!(dmg.target, Entity::Id(78));
                assert_eq!(dmg.ability, "Flurry_BaseDamage".to_string());
                assert_eq!(dmg.amount, 25);
                assert_eq!(dmg.crit, false);
                assert_eq!(dmg.dodged, true);
            },
            Some(_) => assert!(false),
            None => panic!("Update the test case"),
        }

        match line.next() {
            Some(_) => panic!("Update the test case"),
            None => (),
        }
    }

    #[test]
    fn parse_unit_class() {
        let mut parser = LogParser::new();
        let line = parser.do_parse(L_UNIT_CLASS);

        match &line {
            ParseEvent::Internal(InternalEvent::UnitClass(_, id, class)) => {
                assert_eq!(*id, 22);
                assert_eq!(*class, "C02".to_string());
            },
            _ => {
                println!("received {:?}", line);
                assert!(false);
            }
        }
    }

    #[test]
    fn parse_start_dive() {
        let mut parser = LogParser::new();
        let line = parser.do_parse(L_START_DIVE);

        match &line {
            ParseEvent::Parsed(Event::StartDive(_)) => (),
            _ => {
                println!("received {:?}", line);
                assert!(false);
            }
        }
    }

    #[ignore]
    #[test]
    fn parse_end_dive() {
        todo!()
    }

    #[test]
    fn parse_start_combat() {
        let mut parser = LogParser::new();
        let line = parser.do_parse(L_START_COMBAT);

        match &line {
            ParseEvent::Parsed(Event::StartCombat(_)) => (),
            _ => {
                println!("received {:?}", line);
                assert!(false);
            }
        }
    }

    #[test]
    fn parse_end_combat() {
        let mut parser = LogParser::new();
        let line = parser.do_parse(L_END_COMBAT);

        match &line {
            ParseEvent::Parsed(Event::EndCombat(_)) => (),
            _ => {
                println!("received {:?}", line);
                assert!(false);
            }
        }
    }

    #[test]
    fn parse_next_turn() {
        let mut parser = LogParser::new();
        let line = parser.do_parse(L_NEXT_TURN);

        match &line {
            ParseEvent::Parsed(Event::NextTurn(_)) => (),
            _ => {
                println!("received {:?}", line);
                assert!(false);
            }
        }
    }

    #[test]
    #[ignore] // TODO: skip until there's a sanitized log to actually parse
    fn test_logfile() {
        // TODO: use parse_log_to_json
        // env_logger::Builder::new()
        //     .filter_level(log::LevelFilter::Debug)
        //     .init();

        // debug!("parsing log...");

        let mut log_parser = crate::parser::LogParser::new();
        let mut data_log = crate::parser::DataLog::new();

        let file = std::fs::read_to_string("logfile.log").unwrap();
        let file: Vec<&str> = file.split('\n').collect();

        let events = log_parser.parse_lines(file.as_slice());
        data_log.handle_events(events);

        println!("{}", serde_json::to_string(&data_log).unwrap());
    }

    #[test]
    fn test_logfile_append() {
        use std::io::*;
        use std::fs::*;

        const LOGFILE_NAME: &'static str = "testing_logfile.log";

        let waiter = std::sync::Arc::new(std::sync::Barrier::new(2));

        {
            let waiter = waiter.clone();
            std::thread::spawn(move || {
                let mut file = std::fs::File::create(LOGFILE_NAME).unwrap();
                // TODO: consider putting the newlines into the strings, or doing this cleaner
                file.write(L_START_DIVE.as_bytes()).unwrap();
                file.write(b"\n").unwrap();
                file.write(L_START_COMBAT.as_bytes()).unwrap();
                file.write(b"\n").unwrap();
                file.write(L_REGISTER_NAME.as_bytes()).unwrap();
                file.write(b"\n").unwrap();
                file.write(L_UNIT_CLASS.as_bytes()).unwrap();
                file.write(b"\n").unwrap();
                file.write(L_DAMAGE_NORMAL.as_bytes()).unwrap();
                file.write(b"\n").unwrap();
                file.flush().unwrap();

                waiter.wait(); // Let the reading thread parse the initial lines
                waiter.wait(); // Wait on the reading thread to be done parsing initial lines

                file.write(L_DAMAGE_NORMAL.as_bytes()).unwrap();
                file.write(b"\n").unwrap();
                file.flush().unwrap();

                waiter.wait(); // Let the reading thread parse the last line
            });
        };

        let mut parser = crate::parser::LogParser::new();
        let mut datalog = crate::parser::DataLog::new();

        // Wait for the writing thread to write the initial lines
        waiter.wait();

        let file = File::open(LOGFILE_NAME).unwrap();
        let mut reader = BufReader::new(file);

        // TODO: really consider just using the actual code here
        let mut cache_string = String::new();
        while reader.read_line(&mut cache_string).unwrap() != 0 {
            println!("read: {}", cache_string);
            if let Some(event) = parser.parse_line(&cache_string.as_str()) {
                datalog.handle_event(event);
            }
        }

        let dive = datalog.dives.get(0);
        assert!(dive.is_some());
        let dive = dive.unwrap();
        // TODO: consider asserting combat info too
        let testplayer = dive.player_stats.player_stats.get("TestPlayer");
        assert!(testplayer.is_some());
        let testplayer = testplayer.unwrap();
        assert!(testplayer.total_damage_dealt == 25);

        waiter.wait(); // Let the writing thread append a new line
        waiter.wait(); // Writing thread is done appending a new line

        let mut cache_string = String::new();
        while reader.read_line(&mut cache_string).unwrap() != 0 {
            println!("read: {}", cache_string);
            if let Some(event) = parser.parse_line(&cache_string.as_str()) {
                datalog.handle_event(event);
            }
        }

        let dive = datalog.dives.get(0);
        assert!(dive.is_some());
        let dive = dive.unwrap();
        // TODO: consider asserting combat info too
        let testplayer = dive.player_stats.player_stats.get("TestPlayer");
        assert!(testplayer.is_some());
        let testplayer = testplayer.unwrap();
        assert!(testplayer.total_damage_dealt == 50);

    }
}
