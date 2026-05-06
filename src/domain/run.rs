use crate::domain::combat::{simulate_combat, CombatOutcome};
use crate::domain::dungeon::{generate_dungeon, RoomKind};
use crate::domain::hero::HeroProfile;
use crate::domain::items::ItemInstance;
use crate::domain::loot::{roll_loot, salvage_value};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RunConfig {
    pub seed: u64,
    pub max_depth: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RunOutcome {
    HeroDied,
    BossDefeated,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RunSummary {
    pub outcome: RunOutcome,
    pub deepest_depth: u32,
    pub gold_earned: u32,
    pub salvage_earned: u32,
    pub loot: Vec<ItemInstance>,
    pub death_reason: Option<String>,
    pub log: Vec<String>,
}

pub fn simulate_run(hero: &HeroProfile, config: RunConfig) -> RunSummary {
    let rooms = generate_dungeon(config.max_depth, config.seed);
    let mut deepest_depth = 0;
    let mut gold_earned = 0;
    let mut salvage_earned = 0;
    let mut loot = Vec::new();
    let mut log = Vec::new();

    for room in rooms {
        deepest_depth = room.depth;
        let is_boss = room.kind == RoomKind::Boss;
        match room.kind {
            RoomKind::Monster | RoomKind::Elite | RoomKind::Boss => {
                let enemy = room.encounter.as_ref().unwrap().enemy.clone();
                let combat = simulate_combat(hero, &enemy, 100);
                match combat.outcome {
                    CombatOutcome::HeroWon => {
                        log.push(format!("Depth {}: defeated {}", room.depth, enemy.name));
                        gold_earned += room.depth * 3;
                        if is_boss {
                            log.push("The Gate Warden falls. The delve is victorious.".to_string());
                            return RunSummary {
                                outcome: RunOutcome::BossDefeated,
                                deepest_depth,
                                gold_earned,
                                salvage_earned,
                                loot,
                                death_reason: None,
                                log,
                            };
                        }
                    }
                    CombatOutcome::EnemyWon | CombatOutcome::TimedOut => {
                        log.push(format!("Depth {}: defeated by {}", room.depth, enemy.name));
                        return RunSummary {
                            outcome: RunOutcome::HeroDied,
                            deepest_depth,
                            gold_earned,
                            salvage_earned,
                            loot,
                            death_reason: Some(format!("Defeated by {}", enemy.name)),
                            log,
                        };
                    }
                }
            }
            RoomKind::Treasure => {
                let item = roll_loot(room.depth, config.seed);
                salvage_earned += salvage_value(&item);
                log.push(format!("Depth {}: found {}", room.depth, item.name));
                loot.push(item);
                gold_earned += room.depth * 2;
            }
            RoomKind::Shrine => {
                log.push(format!(
                    "Depth {}: shrine grants {} gold",
                    room.depth, room.depth
                ));
                gold_earned += room.depth;
            }
        }
    }

    RunSummary {
        outcome: RunOutcome::BossDefeated,
        deepest_depth,
        gold_earned,
        salvage_earned,
        loot,
        death_reason: None,
        log,
    }
}
