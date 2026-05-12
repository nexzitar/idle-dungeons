use crate::domain::combat::{
    combat_playback_frames_from_result, party_strike_damage_white_yellow,
    simulate_party_vs_encounter_foes, CombatOutcome, CombatPlaybackFrame, CombatSimOptions,
};
use crate::domain::combat_timing::encounter_initiative_seed;
use crate::domain::dungeon::{
    generate_dungeon, peak_risk_note, room_risk_hint, room_risk_rank, DungeonRoom, RoomKind,
};
use crate::domain::combat_archetype::hints_for_hero;
use crate::domain::hero::HeroProfile;
use crate::domain::items::ItemInstance;
use crate::domain::loot::{
    roll_loot_with_hints, roll_profile_guided_early_combat_drop, salvage_value,
};
use serde::{Deserialize, Serialize};

/// Default floor cap for a full delve (matches typical [`RunConfig::max_depth`]).
pub const DEFAULT_RUN_MAX_DEPTH: u32 = 100;

/// Fixed seed for the MVP “Start run” control until a seed picker exists.
pub const DEFAULT_RUN_SEED: u64 = 1;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RunConfig {
    pub seed: u64,
    pub max_depth: u32,
    /// Multiplier applied to total run gold (from meta upgrades). Default `1.0`.
    pub gold_gain_multiplier: f32,
    /// Copy of **`MetaProgression::guided_early_combat_drop_count`** at run start — steers weapon→armor pacing for shallow combat salvage.
    pub guided_early_combat_claims_already: u32,
}

impl RunConfig {
    pub fn new(seed: u64, max_depth: u32) -> Self {
        Self {
            seed,
            max_depth,
            gold_gain_multiplier: 1.0,
            guided_early_combat_claims_already: 0,
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
    /// This run awarded the once-per-delve salvage before depth **10** (`weapon`/`armor` pacing).
    #[serde(default)]
    pub guided_early_combat_drop_granted: bool,
    /// Weighted progression score from encounters cleared (Wave 4 experiment; not yet spendable currency).
    #[serde(default)]
    pub encounter_score: u64,
    /// Party **strike** damage to foes: weapon/basic (white) from [`crate::domain::combat::CombatEvent::HeroAttacked`].
    #[serde(default)]
    pub party_strike_damage_white: u64,
    /// Party **strike** damage to foes: ability (yellow) from [`crate::domain::combat::CombatEvent::HeroAttacked`].
    #[serde(default)]
    pub party_strike_damage_yellow: u64,
}

impl RunSummary {
    /// Percent of strike damage that was ability (yellow), **0–100**. `None` if there was no strike damage.
    pub fn strike_ability_share_percent(&self) -> Option<u32> {
        let den = self
            .party_strike_damage_white
            .saturating_add(self.party_strike_damage_yellow);
        if den == 0 {
            None
        } else {
            Some(
                (self
                    .party_strike_damage_yellow
                    .saturating_mul(100)
                    / den)
                .min(100) as u32,
            )
        }
    }
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
    /// Second party hero snapshot for playback (`None` when running solo).
    pub partner_snapshot_hp: Option<i32>,
    pub partner_snapshot_max_hp: Option<i32>,
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
    partner_hp: Option<i32>,
    partner_max: Option<i32>,
    floors_cleared: u32,
    cap: u32,
    kind: RunPlaybackFrameKind,
) -> RunPlaybackFrame {
    RunPlaybackFrame {
        depth,
        room_kind,
        hero_snapshot_hp: hero_hp,
        hero_snapshot_max_hp: hero_max,
        partner_snapshot_hp: partner_hp,
        partner_snapshot_max_hp: partner_max,
        delve_floors_cleared: floors_cleared,
        delve_floors_cap: cap,
        risk_hint: room_risk_hint(room_kind).to_string(),
        kind,
    }
}

pub fn simulate_run_with_playback(
    lead: &HeroProfile,
    party_partner: Option<&HeroProfile>,
    config: RunConfig,
) -> RunSimulation {
    let mut sim = simulate_run_with_playback_inner(lead, party_partner, config);
    let m = config.gold_gain_multiplier.max(0.0);
    if m != 1.0 {
        sim.summary.gold_earned = (sim.summary.gold_earned as f32 * m).round() as u32;
    }
    sim
}

fn simulate_run_with_playback_inner(
    lead: &HeroProfile,
    party_partner: Option<&HeroProfile>,
    config: RunConfig,
) -> RunSimulation {
    let rooms = generate_dungeon(config.max_depth, config.seed);
    simulate_run_with_playback_for_rooms(lead, party_partner, config, rooms)
}

fn simulate_run_with_playback_for_rooms(
    lead: &HeroProfile,
    party_partner: Option<&HeroProfile>,
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
    let hero_max_hp = lead.derived_stats().max_health;
    let mut hero_current_hp = hero_max_hp;
    let partner_max_hp = party_partner.map(|p| p.derived_stats().max_health);
    let mut partner_current_hp = partner_max_hp.unwrap_or(0);
    let mut floors_cleared = 0u32;
    let mut granted_pre10_combat_loot = false;
    let mut guided_early_combat_drop_granted_this_run = false;
    let mut encounter_score = 0u64;
    let mut peak_risk_rank = 0u8;
    let mut run_dmg_meter_0 = 0u32;
    let mut run_dmg_meter_1 = 0u32;
    let mut run_dmg_meter_foe = 0u32;
    let mut run_sim_ticks_total = 0u32;
    let mut run_strike_white = 0u64;
    let mut run_strike_yellow = 0u64;
    let lead_archetype_hints = hints_for_hero(lead);

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
                            guided_early_combat_drop_granted:
                                guided_early_combat_drop_granted_this_run,
                            encounter_score,
                            party_strike_damage_white: run_strike_white,
                            party_strike_damage_yellow: run_strike_yellow,
                        },
                        playback,
                    };
                };
                let at_start = hero_current_hp;
                let initiative_salt =
                    encounter_initiative_seed(config.seed, u64::from(room.depth).rotate_left(13));
                let foes: Vec<crate::domain::dungeon::Enemy> = {
                    let mut v = vec![encounter.enemy.clone()];
                    if let Some(b) = encounter.enemy_b.as_ref() {
                        v.push(b.clone());
                    }
                    v
                };
                let foe_display = foes.iter().map(|e| e.name.as_str()).collect::<Vec<_>>().join(" · ");
                let enemy_max_hp_bar: i32 = foes.iter().map(|e| e.max_health).sum();
                let combat = simulate_party_vs_encounter_foes(
                    lead,
                    &foes,
                    420,
                    at_start,
                    party_partner.map(|p| (p, partner_current_hp)),
                    initiative_salt,
                    &[],
                    CombatSimOptions::default(),
                );
                let (w_add, y_add) = party_strike_damage_white_yellow(&combat.events);
                run_strike_white = run_strike_white.saturating_add(w_add);
                run_strike_yellow = run_strike_yellow.saturating_add(y_add);
                for frame in combat_playback_frames_from_result(
                    lead,
                    party_partner,
                    &combat,
                    &foe_display,
                    hero_max_hp,
                    enemy_max_hp_bar,
                    at_start,
                    partner_max_hp.map(|_| partner_current_hp),
                    partner_max_hp,
                    run_dmg_meter_0,
                    run_dmg_meter_1,
                    run_dmg_meter_foe,
                    run_sim_ticks_total,
                ) {
                    run_dmg_meter_0 = frame.damage_meter_party_0;
                    run_dmg_meter_1 = frame.damage_meter_party_1;
                    run_dmg_meter_foe = frame.damage_meter_foe;
                    run_sim_ticks_total = frame.run_sim_ticks;
                    playback.frames.push(playback_frame(
                        room.depth,
                        room.kind,
                        frame.hero_hp,
                        frame.hero_max_hp,
                        frame.partner_hp,
                        frame.partner_max_hp,
                        floors_cleared,
                        cap,
                        RunPlaybackFrameKind::Combat(frame),
                    ));
                }
                match combat.outcome {
                    CombatOutcome::HeroWon => {
                        let room_mult: u32 = match room.kind {
                            RoomKind::Elite => 3,
                            RoomKind::Boss => 12,
                            _ => 1,
                        };
                        encounter_score +=
                            u64::from(room.depth.saturating_mul(15).saturating_mul(room_mult))
                                + u64::from(combat.clock_ticks / 8);

                        hero_current_hp = combat.hero_health;
                        if let Some(h) = combat.partner_health {
                            partner_current_hp = h;
                        }
                        log.push(format!("Depth {}: defeated {}", room.depth, foe_display));
                        gold_earned += room.depth * 3;
                        floors_cleared += 1;
                        if room.depth < 10
                            && matches!(room.kind, RoomKind::Monster | RoomKind::Elite)
                            && !granted_pre10_combat_loot
                        {
                            granted_pre10_combat_loot = true;
                            guided_early_combat_drop_granted_this_run = true;
                            let item = roll_profile_guided_early_combat_drop(
                                room.depth,
                                config.seed,
                                room.depth,
                                config.guided_early_combat_claims_already,
                                &lead_archetype_hints,
                            );
                            let note = match config.guided_early_combat_claims_already {
                                0 => "main-hand salvage",
                                1 => "chest salvage",
                                _ => "skirmish salvage",
                            };
                            log.push(format!(
                                "Depth {}: scavenged {} ({}).",
                                room.depth, item.name, note
                            ));
                            loot.push(item);
                        }
                        if is_boss {
                            log.push("The Gate Warden falls. The delve is victorious.".to_string());
                            playback.frames.push(playback_frame(
                                room.depth,
                                room.kind,
                                hero_current_hp,
                                hero_max_hp,
                                partner_max_hp.map(|_| partner_current_hp),
                                partner_max_hp,
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
                                    guided_early_combat_drop_granted:
                                        guided_early_combat_drop_granted_this_run,
                                    encounter_score,
                                    party_strike_damage_white: run_strike_white,
                                    party_strike_damage_yellow: run_strike_yellow,
                                },
                                playback,
                            };
                        }
                    }
                    CombatOutcome::EnemyWon | CombatOutcome::TimedOut => {
                        log.push(format!("Depth {}: defeated by {}", room.depth, foe_display));
                        return RunSimulation {
                            summary: RunSummary {
                                outcome: RunOutcome::HeroDied,
                                deepest_depth,
                                floors_cleared,
                                dungeon_depth_cap: cap,
                                gold_earned,
                                salvage_earned,
                                loot,
                                death_reason: Some(format!("Defeated by {}", foe_display)),
                                log,
                                peak_risk_note: peak_risk_note(peak_risk_rank),
                                guided_early_combat_drop_granted:
                                    guided_early_combat_drop_granted_this_run,
                                encounter_score,
                                party_strike_damage_white: run_strike_white,
                                party_strike_damage_yellow: run_strike_yellow,
                            },
                            playback,
                        };
                    }
                }
            }
            RoomKind::Treasure => {
                encounter_score += u64::from(room.depth.saturating_mul(6));
                let item = roll_loot_with_hints(room.depth, config.seed, &lead_archetype_hints);
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
                    partner_max_hp.map(|_| partner_current_hp),
                    partner_max_hp,
                    floors_cleared,
                    cap,
                    RunPlaybackFrameKind::Narration { text: line },
                ));
            }
            RoomKind::Shrine => {
                encounter_score += u64::from(room.depth.saturating_mul(4));
                let line = format!("Depth {}: shrine grants {} gold", room.depth, room.depth);
                log.push(line.clone());
                gold_earned += room.depth;
                floors_cleared += 1;
                playback.frames.push(playback_frame(
                    room.depth,
                    room.kind,
                    hero_current_hp,
                    hero_max_hp,
                    partner_max_hp.map(|_| partner_current_hp),
                    partner_max_hp,
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
            guided_early_combat_drop_granted: guided_early_combat_drop_granted_this_run,
            encounter_score,
            party_strike_damage_white: run_strike_white,
            party_strike_damage_yellow: run_strike_yellow,
        },
        playback,
    }
}

pub fn simulate_run(
    lead: &HeroProfile,
    party_partner: Option<&HeroProfile>,
    config: RunConfig,
) -> RunSummary {
    simulate_run_with_playback(lead, party_partner, config).summary
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
        let sim = simulate_run_with_playback_for_rooms(&hero, None, config, rooms);
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

    #[test]
    fn strike_damage_totals_accumulate_and_are_deterministic() {
        let hero = HeroProfile::default();
        let config = RunConfig::new(5, 12);
        let a = simulate_run_with_playback(&hero, None, config);
        let b = simulate_run_with_playback(&hero, None, config);
        assert_eq!(
            (
                a.summary.party_strike_damage_white,
                a.summary.party_strike_damage_yellow
            ),
            (
                b.summary.party_strike_damage_white,
                b.summary.party_strike_damage_yellow
            )
        );
        assert!(
            a.summary.party_strike_damage_white > 0
                || a.summary.party_strike_damage_yellow > 0,
            "expected some strike damage over a multi-room run"
        );
        let pct = a.summary.strike_ability_share_percent().expect("share");
        assert!(pct <= 100);
    }
}
