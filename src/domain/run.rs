use crate::domain::combat::{
    combat_playback_frames_from_result, simulate_combat_party, CombatOutcome, CombatPlaybackFrame,
};
use crate::domain::party::tank_ally_starting_stats;
use crate::domain::dungeon::{
    generate_dungeon, peak_risk_note, room_risk_hint, room_risk_rank, DungeonRoom, RoomKind,
};
use crate::domain::hero::HeroProfile;
use crate::domain::items::ItemInstance;
use crate::domain::loot::{roll_loot, salvage_value};
use serde::{Deserialize, Serialize};

/// Default floor cap for a full delve (matches typical [`RunConfig::max_depth`]).
pub const DEFAULT_RUN_MAX_DEPTH: u32 = 25;

/// Fixed seed for the MVP “Start run” control until a seed picker exists.
pub const DEFAULT_RUN_SEED: u64 = 1;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RunConfig {
    pub seed: u64,
    pub max_depth: u32,
    /// Multiplier applied to total run gold (from meta upgrades). Default `1.0`.
    pub gold_gain_multiplier: f32,
}

impl RunConfig {
    pub fn new(seed: u64, max_depth: u32) -> Self {
        Self {
            seed,
            max_depth,
            gold_gain_multiplier: 1.0,
        }
    }
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
    #[serde(default)]
    pub floors_cleared: u32,
    #[serde(default = "default_summary_dungeon_cap")]
    pub dungeon_depth_cap: u32,
    pub gold_earned: u32,
    pub salvage_earned: u32,
    pub loot: Vec<ItemInstance>,
    pub death_reason: Option<String>,
    pub log: Vec<String>,
    #[serde(default)]
    pub peak_risk_note: String,
}

fn default_summary_dungeon_cap() -> u32 {
    DEFAULT_RUN_MAX_DEPTH
}

#[derive(Debug, Clone)]
pub enum RunPlaybackFrameKind {
    Narration { text: String },
    Combat(CombatPlaybackFrame),
}

#[derive(Debug, Clone)]
pub struct RunPlaybackFrame {
    pub depth: u32,
    pub room_kind: RoomKind,
    /// Hero HP for the status bar (carries across rooms).
    pub hero_snapshot_hp: i32,
    pub hero_snapshot_max_hp: i32,
    /// Tank ally HP when the delve includes the fixed tank NPC (`None` reserved for solo playback).
    pub ally_snapshot_hp: Option<i32>,
    pub ally_snapshot_max_hp: Option<i32>,
    /// Floors fully cleared before this frame (`0` until the first room is done).
    pub delve_floors_cleared: u32,
    /// Same as run max depth (`RunConfig.max_depth`).
    pub delve_floors_cap: u32,
    /// Encounter pressure label for this step (matches [`crate::domain::dungeon::room_risk_hint`]).
    pub risk_hint: String,
    pub kind: RunPlaybackFrameKind,
}

#[derive(Debug, Clone, Default)]
pub struct RunPlayback {
    pub frames: Vec<RunPlaybackFrame>,
}

#[derive(Debug, Clone)]
pub struct RunSimulation {
    pub summary: RunSummary,
    pub playback: RunPlayback,
}

fn playback_frame(
    depth: u32,
    room_kind: RoomKind,
    hero_hp: i32,
    hero_max: i32,
    ally_hp: Option<i32>,
    ally_max: Option<i32>,
    floors_cleared: u32,
    cap: u32,
    kind: RunPlaybackFrameKind,
) -> RunPlaybackFrame {
    RunPlaybackFrame {
        depth,
        room_kind,
        hero_snapshot_hp: hero_hp,
        hero_snapshot_max_hp: hero_max,
        ally_snapshot_hp: ally_hp,
        ally_snapshot_max_hp: ally_max,
        delve_floors_cleared: floors_cleared,
        delve_floors_cap: cap,
        risk_hint: room_risk_hint(room_kind).to_string(),
        kind,
    }
}

pub fn simulate_run_with_playback(hero: &HeroProfile, config: RunConfig) -> RunSimulation {
    let mut sim = simulate_run_with_playback_inner(hero, config);
    let m = config.gold_gain_multiplier.max(0.0);
    if m != 1.0 {
        sim.summary.gold_earned = (sim.summary.gold_earned as f32 * m).round() as u32;
    }
    sim
}

fn simulate_run_with_playback_inner(hero: &HeroProfile, config: RunConfig) -> RunSimulation {
    let rooms = generate_dungeon(config.max_depth, config.seed);
    simulate_run_with_playback_for_rooms(hero, config, rooms)
}

fn simulate_run_with_playback_for_rooms(
    hero: &HeroProfile,
    config: RunConfig,
    rooms: Vec<DungeonRoom>,
) -> RunSimulation {
    let cap = config.max_depth.max(1);
    let mut deepest_depth = 0;
    let mut gold_earned = 0;
    let mut salvage_earned = 0;
    let mut loot = Vec::new();
    let mut log = Vec::new();
    let mut playback = RunPlayback::default();
    let hero_max_hp = hero.derived_stats().max_health;
    let mut hero_current_hp = hero_max_hp;
    let tank_stats = tank_ally_starting_stats();
    let ally_max_hp = tank_stats.max_health;
    let mut ally_current_hp = ally_max_hp;
    let mut floors_cleared = 0u32;
    let mut peak_risk_rank = 0u8;

    for room in rooms {
        deepest_depth = room.depth;
        peak_risk_rank = peak_risk_rank.max(room_risk_rank(room.kind));
        let is_boss = room.kind == RoomKind::Boss;
        match room.kind {
            RoomKind::Monster | RoomKind::Elite | RoomKind::Boss => {
                let Some(encounter) = room.encounter.as_ref() else {
                    log.push(format!(
                        "Depth {}: delve aborted — {:?} room has no encounter.",
                        room.depth, room.kind
                    ));
                    return RunSimulation {
                        summary: RunSummary {
                            outcome: RunOutcome::HeroDied,
                            deepest_depth,
                            floors_cleared,
                            dungeon_depth_cap: cap,
                            gold_earned,
                            salvage_earned,
                            loot,
                            death_reason: Some(
                                "Delve aborted: invalid dungeon data (missing encounter).".into(),
                            ),
                            log,
                            peak_risk_note: peak_risk_note(peak_risk_rank),
                        },
                        playback,
                    };
                };
                let enemy = encounter.enemy.clone();
                let at_start = hero_current_hp;
                let combat = simulate_combat_party(
                    hero,
                    &enemy,
                    240,
                    at_start,
                    Some((tank_stats, ally_current_hp)),
                );
                for frame in combat_playback_frames_from_result(
                    hero,
                    &combat,
                    &enemy.name,
                    hero_max_hp,
                    enemy.max_health,
                    at_start,
                    Some(ally_current_hp),
                    Some(ally_max_hp),
                ) {
                    playback.frames.push(playback_frame(
                        room.depth,
                        room.kind,
                        frame.hero_hp,
                        frame.hero_max_hp,
                        frame.ally_hp,
                        frame.ally_max_hp,
                        floors_cleared,
                        cap,
                        RunPlaybackFrameKind::Combat(frame),
                    ));
                }
                match combat.outcome {
                    CombatOutcome::HeroWon => {
                        hero_current_hp = combat.hero_health;
                        if let Some(h) = combat.ally_health {
                            ally_current_hp = h;
                        }
                        log.push(format!("Depth {}: defeated {}", room.depth, enemy.name));
                        gold_earned += room.depth * 3;
                        floors_cleared += 1;
                        if is_boss {
                            log.push("The Gate Warden falls. The delve is victorious.".to_string());
                            playback.frames.push(playback_frame(
                                room.depth,
                                room.kind,
                                hero_current_hp,
                                hero_max_hp,
                                Some(ally_current_hp),
                                Some(ally_max_hp),
                                floors_cleared,
                                cap,
                                RunPlaybackFrameKind::Narration {
                                    text: "The Gate Warden falls. The delve is victorious.".into(),
                                },
                            ));
                            return RunSimulation {
                                summary: RunSummary {
                                    outcome: RunOutcome::BossDefeated,
                                    deepest_depth,
                                    floors_cleared,
                                    dungeon_depth_cap: cap,
                                    gold_earned,
                                    salvage_earned,
                                    loot,
                                    death_reason: None,
                                    log,
                                    peak_risk_note: peak_risk_note(peak_risk_rank),
                                },
                                playback,
                            };
                        }
                    }
                    CombatOutcome::EnemyWon | CombatOutcome::TimedOut => {
                        log.push(format!("Depth {}: defeated by {}", room.depth, enemy.name));
                        return RunSimulation {
                            summary: RunSummary {
                                outcome: RunOutcome::HeroDied,
                                deepest_depth,
                                floors_cleared,
                                dungeon_depth_cap: cap,
                                gold_earned,
                                salvage_earned,
                                loot,
                                death_reason: Some(format!("Defeated by {}", enemy.name)),
                                log,
                                peak_risk_note: peak_risk_note(peak_risk_rank),
                            },
                            playback,
                        };
                    }
                }
            }
            RoomKind::Treasure => {
                let item = roll_loot(room.depth, config.seed);
                salvage_earned += salvage_value(&item);
                let line = format!("Depth {}: found {}", room.depth, item.name);
                log.push(line.clone());
                loot.push(item);
                gold_earned += room.depth * 2;
                floors_cleared += 1;
                playback.frames.push(playback_frame(
                    room.depth,
                    room.kind,
                    hero_current_hp,
                    hero_max_hp,
                    Some(ally_current_hp),
                    Some(ally_max_hp),
                    floors_cleared,
                    cap,
                    RunPlaybackFrameKind::Narration { text: line },
                ));
            }
            RoomKind::Shrine => {
                let line = format!("Depth {}: shrine grants {} gold", room.depth, room.depth);
                log.push(line.clone());
                gold_earned += room.depth;
                floors_cleared += 1;
                playback.frames.push(playback_frame(
                    room.depth,
                    room.kind,
                    hero_current_hp,
                    hero_max_hp,
                    Some(ally_current_hp),
                    Some(ally_max_hp),
                    floors_cleared,
                    cap,
                    RunPlaybackFrameKind::Narration { text: line },
                ));
            }
        }
    }

    RunSimulation {
        summary: RunSummary {
            outcome: RunOutcome::BossDefeated,
            deepest_depth,
            floors_cleared,
            dungeon_depth_cap: cap,
            gold_earned,
            salvage_earned,
            loot,
            death_reason: None,
            log,
            peak_risk_note: peak_risk_note(peak_risk_rank),
        },
        playback,
    }
}

pub fn simulate_run(hero: &HeroProfile, config: RunConfig) -> RunSummary {
    simulate_run_with_playback(hero, config).summary
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::hero::HeroProfile;

    #[test]
    fn missing_encounter_aborts_run_without_panic() {
        let hero = HeroProfile::default();
        let config = RunConfig::new(1, 5);
        let rooms = vec![DungeonRoom {
            depth: 1,
            kind: RoomKind::Monster,
            encounter: None,
        }];
        let sim = simulate_run_with_playback_for_rooms(&hero, config, rooms);
        assert_eq!(sim.summary.outcome, RunOutcome::HeroDied);
        assert!(
            sim.summary
                .death_reason
                .as_ref()
                .is_some_and(|s| s.contains("invalid dungeon")),
            "{:?}",
            sim.summary.death_reason
        );
        assert!(sim
            .summary
            .log
            .iter()
            .any(|line| line.contains("aborted") && line.contains("encounter")));
    }
}
