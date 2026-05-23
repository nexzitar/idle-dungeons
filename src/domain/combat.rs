use crate::domain::buff::{buff_display_name, buff_expires_at_clock, BuffApplication, BuffId};
use crate::domain::combat_meter::{meter_add_attack_speed, meter_try_consume_swing};
use crate::domain::dungeon::Enemy;
use crate::domain::hero::HeroProfile;
use crate::domain::items::ItemAffix;
use crate::domain::skills::{
    skill_definition, skill_timings, SkillCombatStyle, SkillId, SkillKind, SkillTrigger,
};
use crate::domain::stats::Stats;
use std::collections::HashMap;

/// Max foe HP pools (and foe initiative actors) in one engagement.
pub const MAX_COMBAT_FOES: usize = 16;
/// Max party heroes in one engagement (future API: only 1–2 wired today).
pub const MAX_COMBAT_PARTY: usize = 4;

/// Split between auto-attack ("white") and ability ("yellow") contribution on one combat hit.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HeroStrikeDamage {
    pub white: i32,
    pub yellow: i32,
    /// Primary ability credited for yellow damage (display / telemetry).
    pub yellow_source_skill: Option<SkillId>,
}

impl HeroStrikeDamage {
    pub fn total(self) -> i32 {
        self.white.saturating_add(self.yellow)
    }
}

/// Sums white (weapon/basic) vs yellow (ability) damage from party [`CombatEvent::HeroAttacked`]
/// primary hits and cleave splashes. Ignores DoT, thorns, and other non-strike events.
pub fn party_strike_damage_white_yellow(events: &[CombatEvent]) -> (u64, u64) {
    let mut white = 0u64;
    let mut yellow = 0u64;
    for e in events {
        let CombatEvent::HeroAttacked {
            strike,
            cleave_strikes,
            ..
        } = e
        else {
            continue;
        };
        let w = strike.white.max(0) as u64;
        let y = strike.yellow.max(0) as u64;
        white = white.saturating_add(w);
        yellow = yellow.saturating_add(y);
        for (_, c) in cleave_strikes {
            white = white.saturating_add(c.white.max(0) as u64);
            yellow = yellow.saturating_add(c.yellow.max(0) as u64);
        }
    }
    (white, yellow)
}

/// Early exit from multi-pass swing resolution inside `simulate_combat_party`.
#[derive(Debug, Clone, Copy)]
enum CombatBreak {
    HeroWin,
    EnemyWin,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CombatOutcome {
    HeroWon,
    EnemyWon,
    TimedOut,
}

#[derive(Debug, Clone, PartialEq)]
pub enum CombatEvent {
    /// `attacker` 0 = player 1, 1 = player 2 (party roster index).
    HeroAttacked {
        attacker: u8,
        strike: HeroStrikeDamage,
        /// Foe index (`0` = primary pack member) that received [`Self::HeroAttacked::strike`].
        foe_primary: u8,
        /// [`SkillId::Cleave`] (and similar) splash damage per additional living foe: `(foe_index, strike)`.
        cleave_strikes: Vec<(u8, HeroStrikeDamage)>,
    },
    /// Poison at end of clock iteration. `stacks` is potency **before** this tick (and before decrement).
    PoisonTick {
        damage: i32,
        stacks: u32,
        /// Which enemy HP pool the DoT tick consumes.
        foe_index: u8,
    },
    /// `target` 0 = player 1, 1 = player 2 ([`HeroProfile`]).
    EnemyAttacked {
        target: u8,
        damage: i32,
    },
    ThornsReflect {
        damage: i32,
        /// Foe that took thorns damage (the one that struck).
        foe_index: u8,
    },
    /// `target` 0 = player 1, 1 = player 2.
    HeroHealed {
        target: u8,
        amount: i32,
    },
    /// Threat totals per party slot (WoW-style aggro telemetry).
    ThreatSnapshot {
        slot0: i32,
        slot1: i32,
    },
    /// Cast/cooldown meter snapshot for playback bars (`0..=1` each).
    TimingPulse {
        player0_cast: f32,
        player0_cd: f32,
        player1_cast: f32,
        player1_cd: f32,
        foe_cast: f32,
        foe_cd: f32,
        /// When **≥2** living foes, cast/CD for a **non-primary** pack member (off-focus timeline).
        foe_alt_cast: f32,
        foe_alt_cd: f32,
        /// Shared ability GCD (Victory Rush, Empowered Blow queue): fill **during** lockout (`0` when ready).
        player0_skill_gcd: f32,
        /// Per-charge Victory Rush recharge progress (`0` when idle at max charges or skill absent).
        player0_instant_recharge: f32,
        player0_instant_charges: u8,
        player0_instant_max_charges: u8,
        player1_skill_gcd: f32,
        player1_instant_recharge: f32,
        player1_instant_charges: u8,
        player1_instant_max_charges: u8,
    },
    EnemyDefeated {
        /// `0` = first foe, `1` = second foe when applicable.
        foe_index: u8,
    },
    /// Party member defeated (roster index `1+`). Player 1 defeat uses [`HeroDefeated`].
    PartyMemberDown {
        party_index: u8,
    },
    HeroDefeated,
    /// Phase 3: duration / stack buff applied (`combat_clock` at grant is encoded via tick order).
    BuffApplied {
        target: u8,
        buff_id: BuffId,
        stacks: u32,
        duration_ticks: Option<u32>,
    },
    BuffExpired {
        target: u8,
        buff_id: BuffId,
    },
    /// Periodic buff pulse (HoT DoT hooks); reserved until content uses it.
    BuffTick {
        target: u8,
        buff_id: BuffId,
        stacks: u32,
    },
    BuffChargeConsumed {
        target: u8,
        buff_id: BuffId,
        charges_remaining: u32,
    },
}

fn party_player_label(slot: u8, custom_name: Option<&str>) -> String {
    match slot {
        0 => "Player 1".to_string(),
        1 => custom_name
            .map(str::to_string)
            .unwrap_or_else(|| "Player 2".to_string()),
        n => format!("Player {}", n as usize + 1),
    }
}

fn combat_event_caption(event: &CombatEvent, partner_name: Option<&str>) -> String {
    match event {
        CombatEvent::HeroAttacked {
            attacker,
            strike,
            cleave_strikes,
            ..
        } => {
            let total = strike.total();
            let who = party_player_label(*attacker, partner_name);
            let mut base = if strike.yellow == 0 {
                format!("{who} strikes for {total} damage.")
            } else if strike.white == 0 {
                format!("{who} hits for {total} ability damage.")
            } else {
                format!(
                    "{who} strikes for {} white and {} ability ({} total).",
                    strike.white, strike.yellow, total
                )
            };
            for (fi, c) in cleave_strikes {
                let ct = c.total();
                base.push_str(&format!(" Cleave hits foe {} for {ct}.", *fi + 1));
            }
            base
        }
        CombatEvent::PoisonTick {
            damage,
            stacks,
            foe_index,
        } => {
            let tag = if *foe_index == 1 { " (flank)" } else { "" };
            format!("Poison deals {} damage ({} stacks){tag}.", damage, stacks)
        }
        CombatEvent::EnemyAttacked { target, damage } => {
            let who = party_player_label(*target, partner_name);
            format!("The foe hits {who} for {damage} damage.")
        }
        CombatEvent::ThornsReflect { damage, .. } => {
            format!("Thorns bite back for {} damage.", damage)
        }
        CombatEvent::HeroHealed { target, amount } => {
            let who = party_player_label(*target, partner_name);
            format!("{who} recovers {amount} health.")
        }
        CombatEvent::ThreatSnapshot { slot0, slot1 } => {
            format!("Threat · Player 1 {slot0} · Player 2 {slot1}")
        }
        CombatEvent::TimingPulse { .. } => String::new(),
        CombatEvent::EnemyDefeated { foe_index } => {
            if *foe_index == 1 {
                "A second foe falls.".to_string()
            } else {
                "Enemy defeated.".to_string()
            }
        }
        CombatEvent::PartyMemberDown { party_index } => {
            format!(
                "{} is down.",
                party_player_label(*party_index, partner_name)
            )
        }
        CombatEvent::HeroDefeated => "Player 1 is down.".to_string(),
        CombatEvent::BuffApplied {
            target,
            buff_id,
            stacks,
            duration_ticks,
        } => {
            let name = buff_display_name(*buff_id);
            let dur = match duration_ticks {
                None => " (no expiry)".to_string(),
                Some(t) => format!(" ({t} ticks)"),
            };
            let who = party_player_label(*target, partner_name);
            format!("{who} gains {name} ×{stacks}{dur}.")
        }
        CombatEvent::BuffExpired { target, buff_id } => {
            let name = buff_display_name(*buff_id);
            let who = party_player_label(*target, partner_name);
            format!("{name} fades from {who}.")
        }
        CombatEvent::BuffTick {
            target,
            buff_id,
            stacks,
        } => {
            if *buff_id == BuffId::PoisonVenom {
                return format!("Poison corrodes the foe (potency ×{stacks}).");
            }
            let name = buff_display_name(*buff_id);
            let who = party_player_label(*target, partner_name);
            format!("{name} ticks on {who} (×{stacks}).")
        }
        CombatEvent::BuffChargeConsumed {
            target,
            buff_id,
            charges_remaining,
        } => {
            let name = buff_display_name(*buff_id);
            let who = party_player_label(*target, partner_name);
            format!("{name}: {who} consumes a charge ({charges_remaining} left).")
        }
    }
}

/// Where floating combat text should appear relative to the theater layout.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CombatSfxAnchor {
    /// Narration / engage / threat telemetry — skip floating spam.
    #[default]
    Neutral,
    Player0,
    Player1,
    Enemy,
}

fn sfx_anchor_for_event(event: &CombatEvent) -> CombatSfxAnchor {
    match event {
        CombatEvent::HeroAttacked { .. } => CombatSfxAnchor::Enemy,
        CombatEvent::PoisonTick { .. } => CombatSfxAnchor::Enemy,
        CombatEvent::EnemyAttacked { target: 0, .. } => CombatSfxAnchor::Player0,
        CombatEvent::EnemyAttacked { target: 1, .. } => CombatSfxAnchor::Player1,
        CombatEvent::EnemyAttacked { .. } => CombatSfxAnchor::Player0,
        CombatEvent::ThornsReflect { .. } => CombatSfxAnchor::Enemy,
        CombatEvent::HeroHealed { target: 0, .. } => CombatSfxAnchor::Player0,
        CombatEvent::HeroHealed { target: 1, .. } => CombatSfxAnchor::Player1,
        CombatEvent::HeroHealed { .. } => CombatSfxAnchor::Player0,
        CombatEvent::ThreatSnapshot { .. } => CombatSfxAnchor::Neutral,
        CombatEvent::TimingPulse { .. } => CombatSfxAnchor::Neutral,
        CombatEvent::EnemyDefeated { .. } => CombatSfxAnchor::Enemy,
        CombatEvent::PartyMemberDown { .. } => CombatSfxAnchor::Player1,
        CombatEvent::HeroDefeated => CombatSfxAnchor::Player0,
        CombatEvent::BuffApplied { target: 0, .. } => CombatSfxAnchor::Player0,
        CombatEvent::BuffApplied { .. } => CombatSfxAnchor::Player1,
        CombatEvent::BuffExpired { target: 0, .. } => CombatSfxAnchor::Player0,
        CombatEvent::BuffExpired { .. } => CombatSfxAnchor::Player1,
        CombatEvent::BuffTick {
            buff_id: BuffId::PoisonVenom,
            ..
        } => CombatSfxAnchor::Enemy,
        CombatEvent::BuffTick { target: 0, .. } => CombatSfxAnchor::Player0,
        CombatEvent::BuffTick { .. } => CombatSfxAnchor::Player1,
        CombatEvent::BuffChargeConsumed { target: 0, .. } => CombatSfxAnchor::Player0,
        CombatEvent::BuffChargeConsumed { .. } => CombatSfxAnchor::Player1,
    }
}

/// Label for aggro arrow (who the foe is focusing by threat rules).
pub fn aggro_arrow_target_label(
    threat: [i32; 2],
    h0: i32,
    h1: i32,
    has_partner: bool,
    last_foe_target: Option<u8>,
) -> &'static str {
    let slot = pick_party_enemy_target(0, threat, h0, h1, has_partner, last_foe_target);
    if slot == 0 {
        "Player 1"
    } else {
        "Player 2"
    }
}

enum PlaybackStep {
    Event(CombatEvent),
    MergedPoison {
        total_damage: i32,
        tick_count: u32,
        stacks_before_last: u32,
        #[allow(dead_code)]
        foe_index: u8,
    },
}

fn merged_poison_caption(total: i32, count: u32) -> String {
    if count <= 1 {
        format!("Poison deals {total} damage.")
    } else {
        format!("Poison deals {total} damage (×{count}).")
    }
}

fn ability_gcd_bar_frac(left: u32, denom: u32) -> f32 {
    if left == 0 || denom == 0 {
        0.0
    } else {
        (1.0 - (left as f32 / denom as f32)).clamp(0.0, 1.0)
    }
}

fn instant_recharge_bar_frac(recharge_left: u32, period: u8, charges: u8, max_charges: u8) -> f32 {
    if max_charges == 0 || charges >= max_charges || period == 0 || recharge_left == 0 {
        0.0
    } else {
        (1.0 - (recharge_left as f32 / period.max(1) as f32)).clamp(0.0, 1.0)
    }
}

/// Lead-in events before real combat clock work: second-wind heals plus encounter buff grants.
fn opening_playback_prefix_len(events: &[CombatEvent]) -> usize {
    events
        .iter()
        .position(|e| {
            !matches!(
                e,
                CombatEvent::HeroHealed { .. }
                    | CombatEvent::BuffApplied { .. }
                    | CombatEvent::BuffExpired { .. }
            )
        })
        .unwrap_or(events.len())
}

fn playback_apply_opening_event(
    event: &CombatEvent,
    hero_max_hp: i32,
    partner_max_hp: Option<i32>,
    hero_hp: &mut i32,
    partner_hp: &mut Option<i32>,
    player0_buffs: &mut HashMap<BuffId, u32>,
    player1_buffs: &mut HashMap<BuffId, u32>,
) {
    match event {
        CombatEvent::HeroHealed { target, amount } => {
            if *target == 0 {
                *hero_hp = (*hero_hp + *amount).min(hero_max_hp);
            } else if let (Some(a), Some(m)) = (partner_hp.as_mut(), partner_max_hp) {
                *a = (*a + *amount).min(m);
            }
        }
        CombatEvent::BuffApplied {
            target,
            buff_id,
            stacks,
            ..
        } => {
            let map = if *target == 0 {
                player0_buffs
            } else {
                player1_buffs
            };
            map.insert(*buff_id, (*stacks).max(1));
        }
        CombatEvent::BuffExpired { target, buff_id } => {
            let map = if *target == 0 {
                player0_buffs
            } else {
                player1_buffs
            };
            map.remove(buff_id);
        }
        _ => {}
    }
}

fn flatten_playback_steps(events: &[CombatEvent]) -> Vec<PlaybackStep> {
    let mut out = Vec::new();
    let mut i = 0usize;
    while i < events.len() {
        if let CombatEvent::PoisonTick {
            damage,
            stacks,
            foe_index,
        } = events[i]
        {
            let mut total_damage = damage;
            let mut tick_count = 1u32;
            let mut stacks_before_last = stacks;
            let merge_foe = foe_index;
            i += 1;
            while i < events.len() {
                if let CombatEvent::PoisonTick {
                    damage: d2,
                    stacks: s2,
                    foe_index: fi2,
                } = events[i]
                {
                    if fi2 != merge_foe {
                        break;
                    }
                    total_damage += d2;
                    tick_count += 1;
                    stacks_before_last = s2;
                    i += 1;
                } else {
                    break;
                }
            }
            out.push(PlaybackStep::MergedPoison {
                total_damage,
                tick_count,
                stacks_before_last,
                foe_index: merge_foe,
            });
        } else {
            out.push(PlaybackStep::Event(events[i].clone()));
            i += 1;
        }
    }
    out
}

/// Seconds assumed per combat-sim clock tick when the UI prints DPS alongside run damage meters.
pub const COMBAT_TICK_DISPLAY_SECS: f32 = 0.48;

/// One row of combat UI: HP totals after a combat event (plus an opening "engage" row).
#[derive(Debug, Clone, PartialEq)]
pub struct CombatPlaybackFrame {
    pub enemy_name: String,
    pub hero_hp: i32,
    pub hero_max_hp: i32,
    /// Second party hero HP when a partner is in the fight (`None` in solo).
    pub partner_hp: Option<i32>,
    pub partner_max_hp: Option<i32>,
    pub enemy_hp: i32,
    pub enemy_max_hp: i32,
    pub caption: String,
    /// Up to four debuff / status chips per side (e.g. `Poison ×4`, or `—` for empty).
    pub hero_debuff_slots: [String; 4],
    pub enemy_debuff_slots: [String; 4],
    /// Last party slot the foe attacked (`0` = player 1, `1` = player 2), when known.
    pub foe_last_target: Option<u8>,
    /// Latest threat totals from combat telemetry (`None` until a snapshot exists).
    pub threat_slot0: Option<i32>,
    pub threat_slot1: Option<i32>,
    /// Cumulative **run** damage the party dealt to enemies (meter display; includes prior fights).
    pub damage_meter_party_0: u32,
    /// Cumulative **run** damage from partner hero attacks (`0` when solo).
    pub damage_meter_party_1: u32,
    /// Cumulative **run** damage the foe dealt to the party.
    pub damage_meter_foe: u32,
    /// Sim-progress through the delve for DPS (~ combat clock ticks accumulated through this frame).
    pub run_sim_ticks: u32,
    pub sfx_anchor: CombatSfxAnchor,
    /// 0–1 cast bar fill for player 1 (weapon swing wind-up).
    pub player0_cast: f32,
    pub player0_cd: f32,
    pub player1_cast: f32,
    pub player1_cd: f32,
    pub foe_cast: f32,
    pub foe_cd: f32,
    /// Pack off-target foe cast/CD fills (zeros in solo or when only one foe lives).
    pub foe_alt_cast: f32,
    pub foe_alt_cd: f32,
    /// Ability GCD bar (`0` = ready), same semantics as [`CombatEvent::TimingPulse::player0_skill_gcd`].
    pub player0_skill_gcd: f32,
    pub player0_instant_recharge: f32,
    pub player0_instant_charges: u8,
    pub player0_instant_max_charges: u8,
    pub player1_skill_gcd: f32,
    pub player1_instant_recharge: f32,
    pub player1_instant_charges: u8,
    pub player1_instant_max_charges: u8,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CombatResult {
    pub outcome: CombatOutcome,
    pub hero_health: i32,
    /// Second party hero (slot 1) when present.
    pub partner_health: Option<i32>,
    pub partner_max_health: Option<i32>,
    /// Remaining HP per foe index (`0..` during combat), same length as the encounter's foe list.
    pub foe_healths: Vec<i32>,
    /// Combat-simulation clock steps executed (upper bound slice length in [`simulate_combat_party`]).
    pub clock_ticks: u32,
    pub events: Vec<CombatEvent>,
}

pub(crate) struct PreparedHero {
    stats: Stats,
    max_h: i32,
    has_lifesteal: bool,
    has_guard: bool,
    has_heavy: bool,
    has_poison: bool,
    has_thorns: bool,
    has_second_wind: bool,
    has_toxic_mastery: bool,
    has_vampiric_aura: bool,
    affix_heavy: bool,
    affix_vamp: bool,
    affix_spiked: bool,
    affix_shattering: bool,
    affix_virulent: bool,
    affix_titans: bool,
    barrier: i32,
    guard_flat: i32,
    attack_speed: f32,
    /// Max ticks from equipped OnAttack actives (0 = use legacy meter spam).
    attack_cast_total: u32,
    attack_cd_total: u32,
    /// Cleave wins over Heavy when both equipped (yellow damage credit). See [`crate::domain::skill_layering`] for player-facing loadout warnings.
    heavy_skill: Option<SkillId>,
    has_empowered_blow: bool,
    has_victory_rush: bool,
    empower_gcd_ticks: u8,
    empower_icd_ticks: u8,
    vr_gcd_ticks: u8,
    /// Per-charge recharge interval in ticks (from VR [`SkillDefinition::ability_icd_ticks`]).
    vr_icd_ticks: u8,
    /// [`SkillDefinition::max_charges`] for Victory Rush when equipped (`1` otherwise).
    vr_max_charges: u8,
}

#[cfg(test)]
impl PreparedHero {
    fn test_attack_cd_total(&self) -> u32 {
        self.attack_cd_total
    }
}

fn attack_cadence_ticks(hero: &HeroProfile) -> (u32, u32) {
    let mut max_cast = 0u32;
    let mut max_cd = 0u32;
    let mut any_on_attack = false;
    for sid in hero.equipped_skill_ids() {
        let def = skill_definition(sid);
        if def.kind != SkillKind::Active {
            continue;
        }
        if def.trigger != SkillTrigger::OnAttack {
            continue;
        }
        if def.combat_style != SkillCombatStyle::SwingWeave {
            continue;
        }
        any_on_attack = true;
        let (c, d) = skill_timings(sid);
        max_cast = max_cast.max(c as u32);
        max_cd = max_cd.max(d as u32);
    }
    if !any_on_attack {
        return (0, 0);
    }
    (max_cast, max_cd.max(1))
}

fn prepare_hero_combat(hero: &HeroProfile) -> PreparedHero {
    let skills: Vec<SkillId> = hero.equipped_skill_ids().collect();
    let has = |id: SkillId| skills.iter().any(|&s| s == id);

    let has_heavy = has(SkillId::HeavyStrike) || has(SkillId::Cleave);
    let has_barrier = has(SkillId::BarrierPulse);
    let stats = hero.derived_stats();
    let max_h = stats.max_health;
    let affix_cursed = hero.has_affix(ItemAffix::Cursed);

    let mut attack_speed = stats.attack_speed.max(0.12);
    if has_heavy {
        attack_speed *= 0.75;
        attack_speed = attack_speed.max(0.12);
    }

    let barrier = if has_barrier {
        let mut b = (10 + stats.healing_power.saturating_mul(2)).clamp(4, max_h / 2);
        if affix_cursed {
            b = (b * 3 / 4).max(2);
        }
        b
    } else {
        0
    };

    let mut guard_flat = (3 + stats.healing_power.max(0) / 2).clamp(3, 25);
    if has(SkillId::Guard) && hero.has_affix(ItemAffix::Bastion) {
        guard_flat += 2;
    }

    let (attack_cast_total, mut attack_cd_total) = attack_cadence_ticks(hero);
    if hero.has_affix(ItemAffix::Rhythm) && attack_cd_total > 1 {
        attack_cd_total -= 1;
    }

    let heavy_skill = if has(SkillId::Cleave) {
        Some(SkillId::Cleave)
    } else if has(SkillId::HeavyStrike) {
        Some(SkillId::HeavyStrike)
    } else {
        None
    };
    let ed = skill_definition(SkillId::EmpoweredBlow);
    let vr_def = skill_definition(SkillId::VictoryRush);

    PreparedHero {
        stats,
        max_h,
        has_lifesteal: has(SkillId::LifestealStrike),
        has_guard: has(SkillId::Guard),
        has_heavy,
        has_poison: has(SkillId::PoisonEdge),
        has_thorns: has(SkillId::ThornSkin),
        has_second_wind: has(SkillId::SecondWind),
        has_toxic_mastery: has(SkillId::ToxicMastery),
        has_vampiric_aura: has(SkillId::VampiricAura),
        affix_heavy: hero.has_affix(ItemAffix::Heavy),
        affix_vamp: hero.has_affix(ItemAffix::Vampiric),
        affix_spiked: hero.has_affix(ItemAffix::Spiked),
        affix_shattering: hero.has_affix(ItemAffix::Shattering),
        affix_virulent: hero.has_affix(ItemAffix::Virulent),
        affix_titans: hero.has_affix(ItemAffix::TitansFury),
        barrier,
        guard_flat,
        attack_speed,
        attack_cast_total,
        attack_cd_total,
        heavy_skill,
        has_empowered_blow: has(SkillId::EmpoweredBlow),
        has_victory_rush: has(SkillId::VictoryRush),
        empower_gcd_ticks: ed.gcd_ticks,
        empower_icd_ticks: ed.ability_icd_ticks,
        vr_gcd_ticks: if has(SkillId::VictoryRush) {
            vr_def.gcd_ticks
        } else {
            0
        },
        vr_icd_ticks: if has(SkillId::VictoryRush) {
            vr_def.ability_icd_ticks
        } else {
            0
        },
        vr_max_charges: if has(SkillId::VictoryRush) {
            vr_def.max_charges.max(1)
        } else {
            1
        },
    }
}

fn weapon_base_after_armor(p: &PreparedHero, enemy: &Enemy) -> i32 {
    let effective_armor = if p.affix_shattering {
        (enemy.armor - 4).max(0)
    } else {
        enemy.armor
    };
    (p.stats.damage - effective_armor).max(1)
}

fn hero_strike_damage(
    p: &PreparedHero,
    enemy: &Enemy,
    cur_hp: i32,
    max_h: i32,
    consume_empower: bool,
) -> HeroStrikeDamage {
    let base = weapon_base_after_armor(p, enemy);
    let mut yellow_sub = 0i32;
    if p.has_heavy {
        yellow_sub += base / 2;
        let mid = base + base / 2;
        if p.affix_heavy {
            yellow_sub += mid / 5;
        }
    }
    if consume_empower {
        yellow_sub += (base / 2).max(2);
    }
    let white_sub = base;
    let raw_total = white_sub.saturating_add(yellow_sub);
    let scaled_total = if p.affix_titans && cur_hp * 2 <= max_h {
        ((raw_total as i64 * 5 / 4).max(1)) as i32
    } else {
        raw_total.max(1)
    };
    let (white, yellow) = if raw_total > 0 {
        let w = ((scaled_total as i64 * white_sub as i64) / raw_total as i64).max(1) as i32;
        let y = scaled_total.saturating_sub(w);
        (w, y)
    } else {
        (scaled_total.max(1), 0)
    };
    let yellow_source_skill = if consume_empower {
        Some(SkillId::EmpoweredBlow)
    } else if p.has_heavy {
        p.heavy_skill
    } else {
        None
    };
    HeroStrikeDamage {
        white,
        yellow,
        yellow_source_skill,
    }
}

fn victory_rush_strike(p: &PreparedHero, enemy: &Enemy) -> HeroStrikeDamage {
    let base = weapon_base_after_armor(p, enemy);
    let y = ((base as i64 * 3) / 5).max(3) as i32;
    HeroStrikeDamage {
        white: 0,
        yellow: y,
        yellow_source_skill: Some(SkillId::VictoryRush),
    }
}

fn apply_lifesteal(
    p: &PreparedHero,
    hero_damage: i32,
    hp: &mut i32,
    max_h: i32,
    target_slot: u8,
    events: &mut Vec<CombatEvent>,
) {
    if !p.has_lifesteal {
        return;
    }
    let mut amount = (hero_damage / 4).max(1);
    if p.affix_vamp {
        amount = (hero_damage / 3).max(1);
    }
    if p.has_vampiric_aura {
        amount = ((amount as i64 * 6 / 5).max(1)) as i32;
    }
    *hp = (*hp + amount).min(max_h);
    events.push(CombatEvent::HeroHealed {
        target: target_slot,
        amount,
    });
}

fn push_empowered_blow_buff_expired(events: &mut Vec<CombatEvent>, max_events: usize, target: u8) {
    if events.len() < max_events {
        events.push(CombatEvent::BuffExpired {
            target,
            buff_id: BuffId::EmpoweredBlow,
        });
    }
}

/// While below max charges, counts down [`PreparedHero::vr_icd_ticks`] and restores one charge per pulse.
/// Multiple missing charges recover sequentially (one pulse each).
fn tick_instant_strike_charge_recharge(
    has_skill: bool,
    max_charges: u8,
    charges: &mut u8,
    recharge_period: u8,
    recharge_left: &mut u32,
) {
    if !has_skill || *charges >= max_charges || *recharge_left == 0 {
        return;
    }
    *recharge_left = recharge_left.saturating_sub(1);
    if *recharge_left == 0 {
        *charges = (*charges + 1).min(max_charges);
        if *charges < max_charges {
            *recharge_left = recharge_period.max(1) as u32;
        }
    }
}

/// After spending a charge: start the per-charge recharge clock only if none is already running.
fn note_instant_strike_charge_spent(
    max_charges: u8,
    charges_after_use: u8,
    recharge_period: u8,
    recharge_left: &mut u32,
) {
    if charges_after_use < max_charges && *recharge_left == 0 {
        *recharge_left = recharge_period.max(1) as u32;
    }
}

/// Party enemy target: highest threat wins; on a tie, reuse [`last_enemy_target`] when still valid,
/// otherwise [`deterministic_tie_break_target`].
pub fn pick_party_enemy_target(
    tick: u32,
    threat: [i32; 2],
    h0: i32,
    h1: i32,
    has_partner: bool,
    last_enemy_target: Option<u8>,
) -> u8 {
    let h0_alive = h0 > 0;
    let h1_alive = h1 > 0;
    if !has_partner || !h1_alive {
        return 0;
    }
    if threat[0] > threat[1] {
        0
    } else if threat[1] > threat[0] {
        1
    } else if let Some(last) = last_enemy_target {
        let last_valid = (last == 0 && h0_alive) || (last == 1 && h1_alive);
        if last_valid {
            last
        } else {
            deterministic_tie_break_target(tick, threat, h0_alive, h1_alive)
        }
    } else {
        deterministic_tie_break_target(tick, threat, h0_alive, h1_alive)
    }
}

fn deterministic_tie_break_target(
    tick: u32,
    threat: [i32; 2],
    h0_alive: bool,
    h1_alive: bool,
) -> u8 {
    match (h0_alive, h1_alive) {
        (true, false) => 0,
        (false, true) => 1,
        (false, false) => 0,
        (true, true) => {
            let mut h = tick as u64;
            h ^= (threat[0] as u64).wrapping_mul(0x85eb_ca6b);
            h = h.rotate_left(13) ^ (threat[1] as u64);
            if h % 2 == 0 {
                0
            } else {
                1
            }
        }
    }
}

/// One initiative pass for the lead hero: CD tick, cast tick, or at most one weapon swing / cast start.
fn lead_weapon_pass(
    p0: &PreparedHero,
    lead: &HeroProfile,
    enemy: &Enemy,
    meters: &mut [u64; 2],
    h0_cd_left: &mut u32,
    h0_cast_left: &mut u32,
    as_applied: &mut [bool; 3],
    h0: &mut i32,
    enemy_health: &mut i32,
    has_partner: bool,
    poison_stacks: &mut u32,
    threat: &mut [i32; 2],
    events: &mut Vec<CombatEvent>,
    max_events: usize,
    h0_skill_gcd_left: &mut u32,
    h0_skill_gcd_denom: &mut u32,
    h0_vr_charges: &mut u8,
    h0_vr_recharge_left: &mut u32,
    h0_empower_queued: &mut bool,
    h0_empower_icd_left: &mut u32,
) -> (Option<CombatBreak>, bool) {
    if *h0 > 0 && *enemy_health > 0 && p0.has_victory_rush {
        if *h0_skill_gcd_left == 0 && *h0_vr_charges > 0 {
            if events.len() >= max_events {
                return (None, true);
            }
            let strike = victory_rush_strike(p0, enemy);
            let hero_damage = strike.total();
            *enemy_health -= hero_damage;
            events.push(CombatEvent::HeroAttacked {
                attacker: 0,
                strike,
                foe_primary: 0,
                cleave_strikes: vec![],
            });
            *h0_vr_charges = h0_vr_charges.saturating_sub(1);
            if events.len() < max_events {
                events.push(CombatEvent::BuffChargeConsumed {
                    target: 0,
                    buff_id: BuffId::VictoryRush,
                    charges_remaining: *h0_vr_charges as u32,
                });
            }
            if has_partner {
                threat[0] = threat[0].saturating_add(hero_damage);
            }
            apply_lifesteal(p0, hero_damage, h0, p0.max_h, 0, events);
            *h0_skill_gcd_left = p0.vr_gcd_ticks.max(1) as u32;
            *h0_skill_gcd_denom = *h0_skill_gcd_left;
            note_instant_strike_charge_spent(
                p0.vr_max_charges,
                *h0_vr_charges,
                p0.vr_icd_ticks,
                h0_vr_recharge_left,
            );
            if *enemy_health <= 0 {
                events.push(CombatEvent::EnemyDefeated { foe_index: 0 });
                maybe_devourer_heal_on_kill(lead, h0, p0.max_h, events);
                return (Some(CombatBreak::HeroWin), true);
            }
            return (None, true);
        }
    }
    if p0.attack_cast_total == 0 && p0.attack_cd_total == 0 {
        if *h0 <= 0 || *enemy_health <= 0 {
            return (None, false);
        }
        if !as_applied[0] {
            meter_add_attack_speed(&mut meters[0], p0.attack_speed);
            as_applied[0] = true;
        }
        if *h0 > 0 && *enemy_health > 0 && meter_try_consume_swing(&mut meters[0]) {
            if events.len() >= max_events {
                return (None, true);
            }
            let consume = *h0_empower_queued;
            if consume {
                *h0_empower_queued = false;
                *h0_empower_icd_left = p0.empower_icd_ticks.max(1) as u32;
                push_empowered_blow_buff_expired(events, max_events, 0);
            }
            let strike = hero_strike_damage(p0, enemy, *h0, p0.max_h, consume);
            let hero_damage = strike.total();
            *enemy_health -= hero_damage;
            events.push(CombatEvent::HeroAttacked {
                attacker: 0,
                strike,
                foe_primary: 0,
                cleave_strikes: vec![],
            });
            if has_partner {
                threat[0] = threat[0].saturating_add(hero_damage);
            }
            apply_lifesteal(p0, hero_damage, h0, p0.max_h, 0, events);
            if p0.has_poison && strike.white > 0 {
                let inc = if p0.affix_virulent { 3 } else { 2 };
                *poison_stacks = (*poison_stacks + inc).min(40);
            }
            if *enemy_health <= 0 {
                events.push(CombatEvent::EnemyDefeated { foe_index: 0 });
                maybe_devourer_heal_on_kill(lead, h0, p0.max_h, events);
                return (Some(CombatBreak::HeroWin), true);
            }
            return (None, true);
        }
        (None, false)
    } else if *h0 > 0 && *enemy_health > 0 {
        if *h0_cd_left > 0 {
            *h0_cd_left -= 1;
            return (None, true);
        }
        if *h0_cast_left > 0 {
            *h0_cast_left -= 1;
            if *h0_cast_left == 0 {
                if events.len() >= max_events {
                    return (None, true);
                }
                let consume = *h0_empower_queued;
                if consume {
                    *h0_empower_queued = false;
                    *h0_empower_icd_left = p0.empower_icd_ticks.max(1) as u32;
                    push_empowered_blow_buff_expired(events, max_events, 0);
                }
                let strike = hero_strike_damage(p0, enemy, *h0, p0.max_h, consume);
                let hero_damage = strike.total();
                *enemy_health -= hero_damage;
                events.push(CombatEvent::HeroAttacked {
                    attacker: 0,
                    strike,
                    foe_primary: 0,
                    cleave_strikes: vec![],
                });
                if has_partner {
                    threat[0] = threat[0].saturating_add(hero_damage);
                }
                apply_lifesteal(p0, hero_damage, h0, p0.max_h, 0, events);
                if p0.has_poison && strike.white > 0 {
                    let inc = if p0.affix_virulent { 3 } else { 2 };
                    *poison_stacks = (*poison_stacks + inc).min(40);
                }
                *h0_cd_left = p0.attack_cd_total;
                if *enemy_health <= 0 {
                    events.push(CombatEvent::EnemyDefeated { foe_index: 0 });
                    maybe_devourer_heal_on_kill(lead, h0, p0.max_h, events);
                    return (Some(CombatBreak::HeroWin), true);
                }
                return (None, true);
            }
            return (None, true);
        }
        if !as_applied[0] {
            meter_add_attack_speed(&mut meters[0], p0.attack_speed);
            as_applied[0] = true;
        }
        if *h0 > 0 && *enemy_health > 0 && meter_try_consume_swing(&mut meters[0]) {
            if p0.attack_cast_total > 0 {
                *h0_cast_left = p0.attack_cast_total;
                return (None, true);
            }
            if events.len() < max_events {
                let consume = *h0_empower_queued;
                if consume {
                    *h0_empower_queued = false;
                    *h0_empower_icd_left = p0.empower_icd_ticks.max(1) as u32;
                    push_empowered_blow_buff_expired(events, max_events, 0);
                }
                let strike = hero_strike_damage(p0, enemy, *h0, p0.max_h, consume);
                let hero_damage = strike.total();
                *enemy_health -= hero_damage;
                events.push(CombatEvent::HeroAttacked {
                    attacker: 0,
                    strike,
                    foe_primary: 0,
                    cleave_strikes: vec![],
                });
                if has_partner {
                    threat[0] = threat[0].saturating_add(hero_damage);
                }
                apply_lifesteal(p0, hero_damage, h0, p0.max_h, 0, events);
                if p0.has_poison && strike.white > 0 {
                    let inc = if p0.affix_virulent { 3 } else { 2 };
                    *poison_stacks = (*poison_stacks + inc).min(40);
                }
                *h0_cd_left = p0.attack_cd_total;
                if *enemy_health <= 0 {
                    events.push(CombatEvent::EnemyDefeated { foe_index: 0 });
                    maybe_devourer_heal_on_kill(lead, h0, p0.max_h, events);
                    return (Some(CombatBreak::HeroWin), true);
                }
                return (None, true);
            }
        }
        (None, false)
    } else {
        (None, false)
    }
}

fn partner_weapon_pass(
    p1prep: &PreparedHero,
    partner: &HeroProfile,
    enemy: &Enemy,
    meters: &mut [u64; 2],
    h1_cd_left: &mut u32,
    h1_cast_left: &mut u32,
    as_applied: &mut [bool; 3],
    h1: &mut i32,
    enemy_health: &mut i32,
    poison_stacks: &mut u32,
    threat: &mut [i32; 2],
    events: &mut Vec<CombatEvent>,
    max_events: usize,
    h1_skill_gcd_left: &mut u32,
    h1_skill_gcd_denom: &mut u32,
    h1_vr_charges: &mut u8,
    h1_vr_recharge_left: &mut u32,
    h1_empower_queued: &mut bool,
    h1_empower_icd_left: &mut u32,
) -> (Option<CombatBreak>, bool) {
    if *h1 > 0 && *enemy_health > 0 && p1prep.has_victory_rush {
        if *h1_skill_gcd_left == 0 && *h1_vr_charges > 0 {
            if events.len() >= max_events {
                return (None, true);
            }
            let strike = victory_rush_strike(p1prep, enemy);
            let hero_damage = strike.total();
            *enemy_health -= hero_damage;
            events.push(CombatEvent::HeroAttacked {
                attacker: 1,
                strike,
                foe_primary: 0,
                cleave_strikes: vec![],
            });
            *h1_vr_charges = h1_vr_charges.saturating_sub(1);
            if events.len() < max_events {
                events.push(CombatEvent::BuffChargeConsumed {
                    target: 1,
                    buff_id: BuffId::VictoryRush,
                    charges_remaining: *h1_vr_charges as u32,
                });
            }
            threat[1] = threat[1].saturating_add(hero_damage);
            apply_lifesteal(p1prep, hero_damage, h1, p1prep.max_h, 1, events);
            *h1_skill_gcd_left = p1prep.vr_gcd_ticks.max(1) as u32;
            *h1_skill_gcd_denom = *h1_skill_gcd_left;
            note_instant_strike_charge_spent(
                p1prep.vr_max_charges,
                *h1_vr_charges,
                p1prep.vr_icd_ticks,
                h1_vr_recharge_left,
            );
            if *enemy_health <= 0 {
                events.push(CombatEvent::EnemyDefeated { foe_index: 0 });
                maybe_devourer_heal_on_kill(partner, h1, p1prep.max_h, events);
                return (Some(CombatBreak::HeroWin), true);
            }
            return (None, true);
        }
    }
    if p1prep.attack_cast_total == 0 && p1prep.attack_cd_total == 0 {
        if *h1 <= 0 || *enemy_health <= 0 {
            return (None, false);
        }
        if !as_applied[1] {
            meter_add_attack_speed(&mut meters[1], p1prep.attack_speed);
            as_applied[1] = true;
        }
        if *h1 > 0 && *enemy_health > 0 && meter_try_consume_swing(&mut meters[1]) {
            if events.len() >= max_events {
                return (None, true);
            }
            let consume = *h1_empower_queued;
            if consume {
                *h1_empower_queued = false;
                *h1_empower_icd_left = p1prep.empower_icd_ticks.max(1) as u32;
                push_empowered_blow_buff_expired(events, max_events, 1);
            }
            let strike = hero_strike_damage(p1prep, enemy, *h1, p1prep.max_h, consume);
            let hero_damage = strike.total();
            *enemy_health -= hero_damage;
            events.push(CombatEvent::HeroAttacked {
                attacker: 1,
                strike,
                foe_primary: 0,
                cleave_strikes: vec![],
            });
            threat[1] = threat[1].saturating_add(hero_damage);
            apply_lifesteal(p1prep, hero_damage, h1, p1prep.max_h, 1, events);
            if p1prep.has_poison && strike.white > 0 {
                let inc = if p1prep.affix_virulent { 3 } else { 2 };
                *poison_stacks = (*poison_stacks + inc).min(40);
            }
            if *enemy_health <= 0 {
                events.push(CombatEvent::EnemyDefeated { foe_index: 0 });
                maybe_devourer_heal_on_kill(partner, h1, p1prep.max_h, events);
                return (Some(CombatBreak::HeroWin), true);
            }
            return (None, true);
        }
        (None, false)
    } else if *h1 > 0 && *enemy_health > 0 {
        if *h1_cd_left > 0 {
            *h1_cd_left -= 1;
            return (None, true);
        }
        if *h1_cast_left > 0 {
            *h1_cast_left -= 1;
            if *h1_cast_left == 0 {
                if events.len() >= max_events {
                    return (None, true);
                }
                let consume = *h1_empower_queued;
                if consume {
                    *h1_empower_queued = false;
                    *h1_empower_icd_left = p1prep.empower_icd_ticks.max(1) as u32;
                    push_empowered_blow_buff_expired(events, max_events, 1);
                }
                let strike = hero_strike_damage(p1prep, enemy, *h1, p1prep.max_h, consume);
                let hero_damage = strike.total();
                *enemy_health -= hero_damage;
                events.push(CombatEvent::HeroAttacked {
                    attacker: 1,
                    strike,
                    foe_primary: 0,
                    cleave_strikes: vec![],
                });
                threat[1] = threat[1].saturating_add(hero_damage);
                apply_lifesteal(p1prep, hero_damage, h1, p1prep.max_h, 1, events);
                if p1prep.has_poison && strike.white > 0 {
                    let inc = if p1prep.affix_virulent { 3 } else { 2 };
                    *poison_stacks = (*poison_stacks + inc).min(40);
                }
                *h1_cd_left = p1prep.attack_cd_total;
                if *enemy_health <= 0 {
                    events.push(CombatEvent::EnemyDefeated { foe_index: 0 });
                    maybe_devourer_heal_on_kill(partner, h1, p1prep.max_h, events);
                    return (Some(CombatBreak::HeroWin), true);
                }
                return (None, true);
            }
            return (None, true);
        }
        if !as_applied[1] {
            meter_add_attack_speed(&mut meters[1], p1prep.attack_speed);
            as_applied[1] = true;
        }
        if *h1 > 0 && *enemy_health > 0 && meter_try_consume_swing(&mut meters[1]) {
            if p1prep.attack_cast_total > 0 {
                *h1_cast_left = p1prep.attack_cast_total;
                return (None, true);
            }
            if events.len() < max_events {
                let consume = *h1_empower_queued;
                if consume {
                    *h1_empower_queued = false;
                    *h1_empower_icd_left = p1prep.empower_icd_ticks.max(1) as u32;
                    push_empowered_blow_buff_expired(events, max_events, 1);
                }
                let strike = hero_strike_damage(p1prep, enemy, *h1, p1prep.max_h, consume);
                let hero_damage = strike.total();
                *enemy_health -= hero_damage;
                events.push(CombatEvent::HeroAttacked {
                    attacker: 1,
                    strike,
                    foe_primary: 0,
                    cleave_strikes: vec![],
                });
                threat[1] = threat[1].saturating_add(hero_damage);
                apply_lifesteal(p1prep, hero_damage, h1, p1prep.max_h, 1, events);
                if p1prep.has_poison && strike.white > 0 {
                    let inc = if p1prep.affix_virulent { 3 } else { 2 };
                    *poison_stacks = (*poison_stacks + inc).min(40);
                }
                *h1_cd_left = p1prep.attack_cd_total;
                if *enemy_health <= 0 {
                    events.push(CombatEvent::EnemyDefeated { foe_index: 0 });
                    maybe_devourer_heal_on_kill(partner, h1, p1prep.max_h, events);
                    return (Some(CombatBreak::HeroWin), true);
                }
                return (None, true);
            }
        }
        (None, false)
    } else {
        (None, false)
    }
}

/// Apply one foe hit (damage to party, threat, thorns). Returns early combat end if hero/partner dies
/// or thorns kill the enemy.
fn resolve_foe_melee_hit(
    enemy: &Enemy,
    target: u8,
    p0: &PreparedHero,
    p1prep: Option<&PreparedHero>,
    h0: &mut i32,
    h1: &mut i32,
    enemy_health: &mut i32,
    threat: &mut [i32; 2],
    barrier: &mut [i32; 2],
    last_enemy_target: &mut Option<u8>,
    has_partner: bool,
    events: &mut Vec<CombatEvent>,
    max_events: usize,
    lead: &HeroProfile,
) -> Option<CombatBreak> {
    *last_enemy_target = Some(target);
    let prep_t = match target {
        0 => p0,
        _ => p1prep.expect("foe targeted partner without partner"),
    };
    let mut enemy_damage = (enemy.damage - prep_t.stats.armor).max(1);
    if prep_t.has_guard {
        enemy_damage = (enemy_damage - prep_t.guard_flat).max(1);
    }
    let bi = target as usize;
    let absorbed = enemy_damage.min(barrier[bi]);
    barrier[bi] -= absorbed;
    let hp_loss = enemy_damage - absorbed;
    if target == 0 {
        *h0 -= hp_loss;
    } else {
        *h1 -= hp_loss;
    }
    threat[target as usize] = threat[target as usize].saturating_add(hp_loss);
    events.push(CombatEvent::EnemyAttacked {
        target,
        damage: hp_loss,
    });
    if has_partner && *h0 > 0 && *h1 > 0 && events.len() < max_events {
        events.push(CombatEvent::ThreatSnapshot {
            slot0: threat[0],
            slot1: threat[1],
        });
    }
    if prep_t.has_thorns && hp_loss > 0 {
        let mut reflect = (hp_loss / 3).max(1);
        if prep_t.affix_spiked {
            reflect = (hp_loss / 2).max(1);
        }
        *enemy_health -= reflect;
        events.push(CombatEvent::ThornsReflect {
            damage: reflect,
            foe_index: 0,
        });
        if *enemy_health <= 0 {
            events.push(CombatEvent::EnemyDefeated { foe_index: 0 });
            maybe_devourer_heal_on_kill(lead, h0, p0.max_h, events);
            return Some(CombatBreak::HeroWin);
        }
    }
    if target == 0 && *h0 <= 0 {
        events.push(CombatEvent::HeroDefeated);
        return Some(CombatBreak::EnemyWin);
    }
    if target == 1 && *h1 <= 0 {
        events.push(CombatEvent::PartyMemberDown { party_index: 1 });
        return Some(CombatBreak::EnemyWin);
    }
    None
}

#[inline]
fn all_foes_dead(h: &[i32]) -> bool {
    h.iter().all(|&x| x <= 0)
}

#[inline]
fn any_foe_alive_slice(h: &[i32]) -> bool {
    h.iter().any(|&x| x > 0)
}

/// Focus fire: lowest index with HP > 0 while any foe lives.
#[inline]
fn focus_fire_primary_idx_slice(h: &[i32]) -> usize {
    h.iter().position(|&x| x > 0).unwrap_or(0)
}

fn scan_new_foe_deaths(
    enemy_health: &[i32],
    foe_slain_logged: &mut [bool],
    events: &mut Vec<CombatEvent>,
    max_events: usize,
    heal_profile: &HeroProfile,
    heal_hp: &mut i32,
    heal_max: i32,
) -> Option<CombatBreak> {
    for idx in 0..enemy_health.len() {
        if enemy_health[idx] <= 0 && idx < foe_slain_logged.len() && !foe_slain_logged[idx] {
            foe_slain_logged[idx] = true;
            if events.len() < max_events {
                events.push(CombatEvent::EnemyDefeated {
                    foe_index: idx as u8,
                });
            }
            maybe_devourer_heal_on_kill(heal_profile, heal_hp, heal_max, events);
            if all_foes_dead(enemy_health) {
                return Some(CombatBreak::HeroWin);
            }
        }
    }
    None
}

fn resolve_foe_melee_hit_dual(
    enemy: &Enemy,
    foe_index: u8,
    target: u8,
    p0: &PreparedHero,
    p1prep: Option<&PreparedHero>,
    h0: &mut i32,
    h1: &mut i32,
    enemy_health: &mut [i32],
    threat: &mut [i32; 2],
    barrier: &mut [i32; 2],
    last_enemy_target: &mut Option<u8>,
    has_partner: bool,
    events: &mut Vec<CombatEvent>,
    max_events: usize,
    lead: &HeroProfile,
    foe_slain_logged: &mut [bool],
) -> Option<CombatBreak> {
    *last_enemy_target = Some(target);
    let prep_t = match target {
        0 => p0,
        _ => p1prep.expect("foe targeted partner without partner"),
    };
    let mut enemy_damage = (enemy.damage - prep_t.stats.armor).max(1);
    if prep_t.has_guard {
        enemy_damage = (enemy_damage - prep_t.guard_flat).max(1);
    }
    let bi = target as usize;
    let absorbed = enemy_damage.min(barrier[bi]);
    barrier[bi] -= absorbed;
    let hp_loss = enemy_damage - absorbed;
    if target == 0 {
        *h0 -= hp_loss;
    } else {
        *h1 -= hp_loss;
    }
    threat[target as usize] = threat[target as usize].saturating_add(hp_loss);
    events.push(CombatEvent::EnemyAttacked {
        target,
        damage: hp_loss,
    });
    if has_partner && *h0 > 0 && *h1 > 0 && events.len() < max_events {
        events.push(CombatEvent::ThreatSnapshot {
            slot0: threat[0],
            slot1: threat[1],
        });
    }
    let fi = foe_index as usize;
    if prep_t.has_thorns && hp_loss > 0 {
        let mut reflect = (hp_loss / 3).max(1);
        if prep_t.affix_spiked {
            reflect = (hp_loss / 2).max(1);
        }
        enemy_health[fi] -= reflect;
        events.push(CombatEvent::ThornsReflect {
            damage: reflect,
            foe_index,
        });
        if enemy_health[fi] <= 0 {
            if !foe_slain_logged[fi] {
                foe_slain_logged[fi] = true;
                if events.len() < max_events {
                    events.push(CombatEvent::EnemyDefeated { foe_index });
                }
                maybe_devourer_heal_on_kill(lead, h0, p0.max_h, events);
            }
            if all_foes_dead(enemy_health) {
                return Some(CombatBreak::HeroWin);
            }
        }
    }
    if target == 0 && *h0 <= 0 {
        events.push(CombatEvent::HeroDefeated);
        return Some(CombatBreak::EnemyWin);
    }
    if target == 1 && *h1 <= 0 {
        events.push(CombatEvent::PartyMemberDown { party_index: 1 });
        return Some(CombatBreak::EnemyWin);
    }
    None
}

fn pack_apply_lead_victory_rush(
    p0: &PreparedHero,
    lead: &HeroProfile,
    foes: &[Enemy],
    enemy_health: &mut [i32],
    h0: &mut i32,
    has_partner: bool,
    threat: &mut [i32; 2],
    events: &mut Vec<CombatEvent>,
    max_events: usize,
    h0_vr_charges: &mut u8,
    h0_skill_gcd_left: &mut u32,
    h0_skill_gcd_denom: &mut u32,
    h0_vr_recharge_left: &mut u32,
    foe_slain_logged: &mut [bool],
) -> Option<CombatBreak> {
    let pri = focus_fire_primary_idx_slice(enemy_health);
    let strike = victory_rush_strike(p0, &foes[pri]);
    let hero_damage = strike.total();
    enemy_health[pri] -= hero_damage;
    events.push(CombatEvent::HeroAttacked {
        attacker: 0,
        strike,
        foe_primary: pri as u8,
        cleave_strikes: vec![],
    });
    *h0_vr_charges = h0_vr_charges.saturating_sub(1);
    if events.len() < max_events {
        events.push(CombatEvent::BuffChargeConsumed {
            target: 0,
            buff_id: BuffId::VictoryRush,
            charges_remaining: *h0_vr_charges as u32,
        });
    }
    if has_partner {
        threat[0] = threat[0].saturating_add(hero_damage);
    }
    apply_lifesteal(p0, hero_damage, h0, p0.max_h, 0, events);
    *h0_skill_gcd_left = p0.vr_gcd_ticks.max(1) as u32;
    *h0_skill_gcd_denom = *h0_skill_gcd_left;
    note_instant_strike_charge_spent(
        p0.vr_max_charges,
        *h0_vr_charges,
        p0.vr_icd_ticks,
        h0_vr_recharge_left,
    );
    scan_new_foe_deaths(
        enemy_health,
        foe_slain_logged,
        events,
        max_events,
        lead,
        h0,
        p0.max_h,
    )
}

fn pack_apply_lead_weapon_melee(
    p0: &PreparedHero,
    lead: &HeroProfile,
    foes: &[Enemy],
    enemy_health: &mut [i32],
    h0: &mut i32,
    consume: bool,
    has_partner: bool,
    threat: &mut [i32; 2],
    poison_stacks: &mut [u32],
    events: &mut Vec<CombatEvent>,
    max_events: usize,
    foe_slain_logged: &mut [bool],
) -> Option<CombatBreak> {
    let pri = focus_fire_primary_idx_slice(enemy_health);
    let strike = hero_strike_damage(p0, &foes[pri], *h0, p0.max_h, consume);
    let hero_damage = strike.total();
    enemy_health[pri] -= hero_damage;

    let mut cleave_strikes: Vec<(u8, HeroStrikeDamage)> = Vec::new();
    if p0.heavy_skill == Some(SkillId::Cleave) {
        for oth in 0..enemy_health.len() {
            if oth != pri && enemy_health[oth] > 0 {
                let cs = hero_strike_damage(p0, &foes[oth], *h0, p0.max_h, consume);
                let cd = cs.total();
                enemy_health[oth] -= cd;
                cleave_strikes.push((oth as u8, cs));
            }
        }
    }

    let cleave_dmg: i32 = cleave_strikes.iter().map(|(_, c)| c.total()).sum();
    if has_partner {
        threat[0] = threat[0].saturating_add(hero_damage.saturating_add(cleave_dmg));
    }
    apply_lifesteal(
        p0,
        hero_damage.saturating_add(cleave_dmg),
        h0,
        p0.max_h,
        0,
        events,
    );
    if p0.has_poison && strike.white > 0 && pri < poison_stacks.len() {
        let inc = if p0.affix_virulent { 3 } else { 2 };
        poison_stacks[pri] = (poison_stacks[pri] + inc).min(40);
    }
    if events.len() < max_events {
        events.push(CombatEvent::HeroAttacked {
            attacker: 0,
            strike,
            foe_primary: pri as u8,
            cleave_strikes,
        });
    }
    scan_new_foe_deaths(
        enemy_health,
        foe_slain_logged,
        events,
        max_events,
        lead,
        h0,
        p0.max_h,
    )
}

fn pack_apply_partner_victory_rush(
    p1prep: &PreparedHero,
    partner: &HeroProfile,
    foes: &[Enemy],
    enemy_health: &mut [i32],
    h1: &mut i32,
    threat: &mut [i32; 2],
    events: &mut Vec<CombatEvent>,
    max_events: usize,
    h1_vr_charges: &mut u8,
    h1_skill_gcd_left: &mut u32,
    h1_skill_gcd_denom: &mut u32,
    h1_vr_recharge_left: &mut u32,
    foe_slain_logged: &mut [bool],
) -> Option<CombatBreak> {
    let pri = focus_fire_primary_idx_slice(enemy_health);
    let strike = victory_rush_strike(p1prep, &foes[pri]);
    let hero_damage = strike.total();
    enemy_health[pri] -= hero_damage;
    events.push(CombatEvent::HeroAttacked {
        attacker: 1,
        strike,
        foe_primary: pri as u8,
        cleave_strikes: vec![],
    });
    *h1_vr_charges = h1_vr_charges.saturating_sub(1);
    if events.len() < max_events {
        events.push(CombatEvent::BuffChargeConsumed {
            target: 1,
            buff_id: BuffId::VictoryRush,
            charges_remaining: *h1_vr_charges as u32,
        });
    }
    threat[1] = threat[1].saturating_add(hero_damage);
    apply_lifesteal(p1prep, hero_damage, h1, p1prep.max_h, 1, events);
    *h1_skill_gcd_left = p1prep.vr_gcd_ticks.max(1) as u32;
    *h1_skill_gcd_denom = *h1_skill_gcd_left;
    note_instant_strike_charge_spent(
        p1prep.vr_max_charges,
        *h1_vr_charges,
        p1prep.vr_icd_ticks,
        h1_vr_recharge_left,
    );
    scan_new_foe_deaths(
        enemy_health,
        foe_slain_logged,
        events,
        max_events,
        partner,
        h1,
        p1prep.max_h,
    )
}

fn pack_apply_partner_weapon_melee(
    p1prep: &PreparedHero,
    partner: &HeroProfile,
    foes: &[Enemy],
    enemy_health: &mut [i32],
    h1: &mut i32,
    consume: bool,
    threat: &mut [i32; 2],
    poison_stacks: &mut [u32],
    events: &mut Vec<CombatEvent>,
    max_events: usize,
    foe_slain_logged: &mut [bool],
) -> Option<CombatBreak> {
    let pri = focus_fire_primary_idx_slice(enemy_health);
    let strike = hero_strike_damage(p1prep, &foes[pri], *h1, p1prep.max_h, consume);
    let hero_damage = strike.total();
    enemy_health[pri] -= hero_damage;

    let mut cleave_strikes: Vec<(u8, HeroStrikeDamage)> = Vec::new();
    if p1prep.heavy_skill == Some(SkillId::Cleave) {
        for oth in 0..enemy_health.len() {
            if oth != pri && enemy_health[oth] > 0 {
                let cs = hero_strike_damage(p1prep, &foes[oth], *h1, p1prep.max_h, consume);
                let cd = cs.total();
                enemy_health[oth] -= cd;
                cleave_strikes.push((oth as u8, cs));
            }
        }
    }

    let cleave_dmg: i32 = cleave_strikes.iter().map(|(_, c)| c.total()).sum();
    threat[1] = threat[1].saturating_add(hero_damage.saturating_add(cleave_dmg));
    apply_lifesteal(
        p1prep,
        hero_damage.saturating_add(cleave_dmg),
        h1,
        p1prep.max_h,
        1,
        events,
    );
    if p1prep.has_poison && strike.white > 0 && pri < poison_stacks.len() {
        let inc = if p1prep.affix_virulent { 3 } else { 2 };
        poison_stacks[pri] = (poison_stacks[pri] + inc).min(40);
    }
    if events.len() < max_events {
        events.push(CombatEvent::HeroAttacked {
            attacker: 1,
            strike,
            foe_primary: pri as u8,
            cleave_strikes,
        });
    }
    scan_new_foe_deaths(
        enemy_health,
        foe_slain_logged,
        events,
        max_events,
        partner,
        h1,
        p1prep.max_h,
    )
}

fn dual_lead_weapon_pass(
    p0: &PreparedHero,
    lead: &HeroProfile,
    foes: &[Enemy],
    meters: &mut [u64; 2],
    h0_cd_left: &mut u32,
    h0_cast_left: &mut u32,
    as_applied: &mut [bool],
    h0: &mut i32,
    enemy_health: &mut [i32],
    has_partner: bool,
    poison_stacks: &mut [u32],
    threat: &mut [i32; 2],
    events: &mut Vec<CombatEvent>,
    max_events: usize,
    h0_skill_gcd_left: &mut u32,
    h0_skill_gcd_denom: &mut u32,
    h0_vr_charges: &mut u8,
    h0_vr_recharge_left: &mut u32,
    h0_empower_queued: &mut bool,
    h0_empower_icd_left: &mut u32,
    foe_slain_logged: &mut [bool],
) -> (Option<CombatBreak>, bool) {
    if *h0 > 0 && any_foe_alive_slice(enemy_health) && p0.has_victory_rush {
        if *h0_skill_gcd_left == 0 && *h0_vr_charges > 0 {
            if events.len() >= max_events {
                return (None, true);
            }
            if let Some(br) = pack_apply_lead_victory_rush(
                p0,
                lead,
                foes,
                enemy_health,
                h0,
                has_partner,
                threat,
                events,
                max_events,
                h0_vr_charges,
                h0_skill_gcd_left,
                h0_skill_gcd_denom,
                h0_vr_recharge_left,
                foe_slain_logged,
            ) {
                return (Some(br), true);
            }
            return (None, true);
        }
    }
    if p0.attack_cast_total == 0 && p0.attack_cd_total == 0 {
        if *h0 <= 0 || !any_foe_alive_slice(enemy_health) {
            return (None, false);
        }
        if !as_applied[0] {
            meter_add_attack_speed(&mut meters[0], p0.attack_speed);
            as_applied[0] = true;
        }
        if *h0 > 0 && any_foe_alive_slice(enemy_health) && meter_try_consume_swing(&mut meters[0]) {
            if events.len() >= max_events {
                return (None, true);
            }
            let consume = *h0_empower_queued;
            if consume {
                *h0_empower_queued = false;
                *h0_empower_icd_left = p0.empower_icd_ticks.max(1) as u32;
                push_empowered_blow_buff_expired(events, max_events, 0);
            }
            if let Some(br) = pack_apply_lead_weapon_melee(
                p0,
                lead,
                foes,
                enemy_health,
                h0,
                consume,
                has_partner,
                threat,
                poison_stacks,
                events,
                max_events,
                foe_slain_logged,
            ) {
                return (Some(br), true);
            }
            return (None, true);
        }
        (None, false)
    } else if *h0 > 0 && any_foe_alive_slice(enemy_health) {
        if *h0_cd_left > 0 {
            *h0_cd_left -= 1;
            return (None, true);
        }
        if *h0_cast_left > 0 {
            *h0_cast_left -= 1;
            if *h0_cast_left == 0 {
                if events.len() >= max_events {
                    return (None, true);
                }
                let consume = *h0_empower_queued;
                if consume {
                    *h0_empower_queued = false;
                    *h0_empower_icd_left = p0.empower_icd_ticks.max(1) as u32;
                    push_empowered_blow_buff_expired(events, max_events, 0);
                }
                if let Some(br) = pack_apply_lead_weapon_melee(
                    p0,
                    lead,
                    foes,
                    enemy_health,
                    h0,
                    consume,
                    has_partner,
                    threat,
                    poison_stacks,
                    events,
                    max_events,
                    foe_slain_logged,
                ) {
                    return (Some(br), true);
                }
                *h0_cd_left = p0.attack_cd_total;
                return (None, true);
            }
            return (None, true);
        }
        if !as_applied[0] {
            meter_add_attack_speed(&mut meters[0], p0.attack_speed);
            as_applied[0] = true;
        }
        if *h0 > 0 && any_foe_alive_slice(enemy_health) && meter_try_consume_swing(&mut meters[0]) {
            if p0.attack_cast_total > 0 {
                *h0_cast_left = p0.attack_cast_total;
                return (None, true);
            }
            if events.len() < max_events {
                let consume = *h0_empower_queued;
                if consume {
                    *h0_empower_queued = false;
                    *h0_empower_icd_left = p0.empower_icd_ticks.max(1) as u32;
                    push_empowered_blow_buff_expired(events, max_events, 0);
                }
                if let Some(br) = pack_apply_lead_weapon_melee(
                    p0,
                    lead,
                    foes,
                    enemy_health,
                    h0,
                    consume,
                    has_partner,
                    threat,
                    poison_stacks,
                    events,
                    max_events,
                    foe_slain_logged,
                ) {
                    return (Some(br), true);
                }
                *h0_cd_left = p0.attack_cd_total;
                return (None, true);
            }
        }
        (None, false)
    } else {
        (None, false)
    }
}

fn dual_partner_weapon_pass(
    p1prep: &PreparedHero,
    partner: &HeroProfile,
    foes: &[Enemy],
    meters: &mut [u64; 2],
    h1_cd_left: &mut u32,
    h1_cast_left: &mut u32,
    as_applied: &mut [bool],
    h1: &mut i32,
    enemy_health: &mut [i32],
    poison_stacks: &mut [u32],
    threat: &mut [i32; 2],
    events: &mut Vec<CombatEvent>,
    max_events: usize,
    h1_skill_gcd_left: &mut u32,
    h1_skill_gcd_denom: &mut u32,
    h1_vr_charges: &mut u8,
    h1_vr_recharge_left: &mut u32,
    h1_empower_queued: &mut bool,
    h1_empower_icd_left: &mut u32,
    foe_slain_logged: &mut [bool],
) -> (Option<CombatBreak>, bool) {
    if *h1 > 0 && any_foe_alive_slice(enemy_health) && p1prep.has_victory_rush {
        if *h1_skill_gcd_left == 0 && *h1_vr_charges > 0 {
            if events.len() >= max_events {
                return (None, true);
            }
            if let Some(br) = pack_apply_partner_victory_rush(
                p1prep,
                partner,
                foes,
                enemy_health,
                h1,
                threat,
                events,
                max_events,
                h1_vr_charges,
                h1_skill_gcd_left,
                h1_skill_gcd_denom,
                h1_vr_recharge_left,
                foe_slain_logged,
            ) {
                return (Some(br), true);
            }
            return (None, true);
        }
    }
    if p1prep.attack_cast_total == 0 && p1prep.attack_cd_total == 0 {
        if *h1 <= 0 || !any_foe_alive_slice(enemy_health) {
            return (None, false);
        }
        if !as_applied[1] {
            meter_add_attack_speed(&mut meters[1], p1prep.attack_speed);
            as_applied[1] = true;
        }
        if *h1 > 0 && any_foe_alive_slice(enemy_health) && meter_try_consume_swing(&mut meters[1]) {
            if events.len() >= max_events {
                return (None, true);
            }
            let consume = *h1_empower_queued;
            if consume {
                *h1_empower_queued = false;
                *h1_empower_icd_left = p1prep.empower_icd_ticks.max(1) as u32;
                push_empowered_blow_buff_expired(events, max_events, 1);
            }
            if let Some(br) = pack_apply_partner_weapon_melee(
                p1prep,
                partner,
                foes,
                enemy_health,
                h1,
                consume,
                threat,
                poison_stacks,
                events,
                max_events,
                foe_slain_logged,
            ) {
                return (Some(br), true);
            }
            return (None, true);
        }
        (None, false)
    } else if *h1 > 0 && any_foe_alive_slice(enemy_health) {
        if *h1_cd_left > 0 {
            *h1_cd_left -= 1;
            return (None, true);
        }
        if *h1_cast_left > 0 {
            *h1_cast_left -= 1;
            if *h1_cast_left == 0 {
                if events.len() >= max_events {
                    return (None, true);
                }
                let consume = *h1_empower_queued;
                if consume {
                    *h1_empower_queued = false;
                    *h1_empower_icd_left = p1prep.empower_icd_ticks.max(1) as u32;
                    push_empowered_blow_buff_expired(events, max_events, 1);
                }
                if let Some(br) = pack_apply_partner_weapon_melee(
                    p1prep,
                    partner,
                    foes,
                    enemy_health,
                    h1,
                    consume,
                    threat,
                    poison_stacks,
                    events,
                    max_events,
                    foe_slain_logged,
                ) {
                    return (Some(br), true);
                }
                *h1_cd_left = p1prep.attack_cd_total;
                return (None, true);
            }
            return (None, true);
        }
        if !as_applied[1] {
            meter_add_attack_speed(&mut meters[1], p1prep.attack_speed);
            as_applied[1] = true;
        }
        if *h1 > 0 && any_foe_alive_slice(enemy_health) && meter_try_consume_swing(&mut meters[1]) {
            if p1prep.attack_cast_total > 0 {
                *h1_cast_left = p1prep.attack_cast_total;
                return (None, true);
            }
            if events.len() < max_events {
                let consume = *h1_empower_queued;
                if consume {
                    *h1_empower_queued = false;
                    *h1_empower_icd_left = p1prep.empower_icd_ticks.max(1) as u32;
                    push_empowered_blow_buff_expired(events, max_events, 1);
                }
                if let Some(br) = pack_apply_partner_weapon_melee(
                    p1prep,
                    partner,
                    foes,
                    enemy_health,
                    h1,
                    consume,
                    threat,
                    poison_stacks,
                    events,
                    max_events,
                    foe_slain_logged,
                ) {
                    return (Some(br), true);
                }
                *h1_cd_left = p1prep.attack_cd_total;
                return (None, true);
            }
        }
        (None, false)
    } else {
        (None, false)
    }
}

fn foe_weapon_pass_pack_slot(
    foe_slot: u8,
    enemy: &Enemy,
    p0: &PreparedHero,
    p1prep: Option<&PreparedHero>,
    enemy_meter: &mut [u64],
    enemy_as: f32,
    foe_cast_left: &mut [u32],
    foe_cd_left: &mut [u32],
    foe_ct: u32,
    foe_dt: u32,
    h0: &mut i32,
    h1: &mut i32,
    has_partner: bool,
    enemy_health: &mut [i32],
    threat: &mut [i32; 2],
    barrier: &mut [i32; 2],
    last_enemy_target: &mut Option<u8>,
    as_applied: &mut [bool],
    as_idx: usize,
    tick: u32,
    events: &mut Vec<CombatEvent>,
    max_events: usize,
    lead: &HeroProfile,
    foe_slain_logged: &mut [bool],
) -> (Option<CombatBreak>, bool) {
    let i = foe_slot as usize;
    if enemy_health[i] <= 0 {
        return (None, false);
    }
    if foe_ct == 0 && foe_dt == 0 {
        if !(*h0 > 0 && (!has_partner || *h1 > 0)) {
            return (None, false);
        }
        if !as_applied[as_idx] {
            meter_add_attack_speed(&mut enemy_meter[i], enemy_as);
            as_applied[as_idx] = true;
        }
        if enemy_health[i] > 0
            && *h0 > 0
            && (!has_partner || *h1 > 0)
            && meter_try_consume_swing(&mut enemy_meter[i])
        {
            if events.len() >= max_events {
                return (None, true);
            }
            let target: u8 =
                pick_party_enemy_target(tick, *threat, *h0, *h1, has_partner, *last_enemy_target);
            if let Some(br) = resolve_foe_melee_hit_dual(
                enemy,
                foe_slot,
                target,
                p0,
                p1prep,
                h0,
                h1,
                enemy_health,
                threat,
                barrier,
                last_enemy_target,
                has_partner,
                events,
                max_events,
                lead,
                foe_slain_logged,
            ) {
                return (Some(br), true);
            }
            return (None, true);
        }
        (None, false)
    } else if enemy_health[i] > 0 && *h0 > 0 && (!has_partner || *h1 > 0) {
        if foe_cd_left[i] > 0 {
            foe_cd_left[i] -= 1;
            return (None, true);
        }
        if foe_cast_left[i] > 0 {
            foe_cast_left[i] -= 1;
            if foe_cast_left[i] == 0 {
                if events.len() >= max_events {
                    return (None, true);
                }
                let target: u8 = pick_party_enemy_target(
                    tick,
                    *threat,
                    *h0,
                    *h1,
                    has_partner,
                    *last_enemy_target,
                );
                if let Some(br) = resolve_foe_melee_hit_dual(
                    enemy,
                    foe_slot,
                    target,
                    p0,
                    p1prep,
                    h0,
                    h1,
                    enemy_health,
                    threat,
                    barrier,
                    last_enemy_target,
                    has_partner,
                    events,
                    max_events,
                    lead,
                    foe_slain_logged,
                ) {
                    return (Some(br), true);
                }
                foe_cd_left[i] = foe_dt;
                return (None, true);
            }
            return (None, true);
        }
        if !as_applied[as_idx] {
            meter_add_attack_speed(&mut enemy_meter[i], enemy_as);
            as_applied[as_idx] = true;
        }
        if enemy_health[i] > 0
            && *h0 > 0
            && (!has_partner || *h1 > 0)
            && meter_try_consume_swing(&mut enemy_meter[i])
        {
            if foe_ct > 0 {
                foe_cast_left[i] = foe_ct;
                return (None, true);
            }
            if events.len() < max_events {
                let target: u8 = pick_party_enemy_target(
                    tick,
                    *threat,
                    *h0,
                    *h1,
                    has_partner,
                    *last_enemy_target,
                );
                if let Some(br) = resolve_foe_melee_hit_dual(
                    enemy,
                    foe_slot,
                    target,
                    p0,
                    p1prep,
                    h0,
                    h1,
                    enemy_health,
                    threat,
                    barrier,
                    last_enemy_target,
                    has_partner,
                    events,
                    max_events,
                    lead,
                    foe_slain_logged,
                ) {
                    return (Some(br), true);
                }
                foe_cd_left[i] = foe_dt;
                return (None, true);
            }
        }
        (None, false)
    } else {
        (None, false)
    }
}

/// One foe swing / cast tick / GCD tick (matches legacy `simulate_combat_party` branch ordering).
fn foe_weapon_pass(
    enemy: &Enemy,
    p0: &PreparedHero,
    p1prep: Option<&PreparedHero>,
    enemy_meter: &mut u64,
    enemy_as: f32,
    foe_cast_left: &mut u32,
    foe_cd_left: &mut u32,
    foe_ct: u32,
    foe_dt: u32,
    h0: &mut i32,
    h1: &mut i32,
    has_partner: bool,
    enemy_health: &mut i32,
    threat: &mut [i32; 2],
    barrier: &mut [i32; 2],
    last_enemy_target: &mut Option<u8>,
    as_applied: &mut [bool; 3],
    tick: u32,
    events: &mut Vec<CombatEvent>,
    max_events: usize,
    lead: &HeroProfile,
) -> (Option<CombatBreak>, bool) {
    if foe_ct == 0 && foe_dt == 0 {
        if !(*enemy_health > 0 && *h0 > 0 && (!has_partner || *h1 > 0)) {
            return (None, false);
        }
        if !as_applied[2] {
            meter_add_attack_speed(enemy_meter, enemy_as);
            as_applied[2] = true;
        }
        if *enemy_health > 0
            && *h0 > 0
            && (!has_partner || *h1 > 0)
            && meter_try_consume_swing(enemy_meter)
        {
            if events.len() >= max_events {
                return (None, true);
            }
            let target: u8 =
                pick_party_enemy_target(tick, *threat, *h0, *h1, has_partner, *last_enemy_target);
            if let Some(br) = resolve_foe_melee_hit(
                enemy,
                target,
                p0,
                p1prep,
                h0,
                h1,
                enemy_health,
                threat,
                barrier,
                last_enemy_target,
                has_partner,
                events,
                max_events,
                lead,
            ) {
                return (Some(br), true);
            }
            if *enemy_health <= 0 {
                events.push(CombatEvent::EnemyDefeated { foe_index: 0 });
                maybe_devourer_heal_on_kill(lead, h0, p0.max_h, events);
                return (Some(CombatBreak::HeroWin), true);
            }
            return (None, true);
        }
        (None, false)
    } else if *enemy_health > 0 && *h0 > 0 && (!has_partner || *h1 > 0) {
        if *foe_cd_left > 0 {
            *foe_cd_left -= 1;
            return (None, true);
        }
        if *foe_cast_left > 0 {
            *foe_cast_left -= 1;
            if *foe_cast_left == 0 {
                if events.len() >= max_events {
                    return (None, true);
                }
                let target: u8 = pick_party_enemy_target(
                    tick,
                    *threat,
                    *h0,
                    *h1,
                    has_partner,
                    *last_enemy_target,
                );
                if let Some(br) = resolve_foe_melee_hit(
                    enemy,
                    target,
                    p0,
                    p1prep,
                    h0,
                    h1,
                    enemy_health,
                    threat,
                    barrier,
                    last_enemy_target,
                    has_partner,
                    events,
                    max_events,
                    lead,
                ) {
                    return (Some(br), true);
                }
                if *enemy_health <= 0 {
                    events.push(CombatEvent::EnemyDefeated { foe_index: 0 });
                    maybe_devourer_heal_on_kill(lead, h0, p0.max_h, events);
                    return (Some(CombatBreak::HeroWin), true);
                }
                *foe_cd_left = foe_dt;
                return (None, true);
            }
            return (None, true);
        }
        if !as_applied[2] {
            meter_add_attack_speed(enemy_meter, enemy_as);
            as_applied[2] = true;
        }
        if *enemy_health > 0
            && *h0 > 0
            && (!has_partner || *h1 > 0)
            && meter_try_consume_swing(enemy_meter)
        {
            if foe_ct > 0 {
                *foe_cast_left = foe_ct;
                return (None, true);
            }
            if events.len() < max_events {
                let target: u8 = pick_party_enemy_target(
                    tick,
                    *threat,
                    *h0,
                    *h1,
                    has_partner,
                    *last_enemy_target,
                );
                if let Some(br) = resolve_foe_melee_hit(
                    enemy,
                    target,
                    p0,
                    p1prep,
                    h0,
                    h1,
                    enemy_health,
                    threat,
                    barrier,
                    last_enemy_target,
                    has_partner,
                    events,
                    max_events,
                    lead,
                ) {
                    return (Some(br), true);
                }
                if *enemy_health <= 0 {
                    events.push(CombatEvent::EnemyDefeated { foe_index: 0 });
                    maybe_devourer_heal_on_kill(lead, h0, p0.max_h, events);
                    return (Some(CombatBreak::HeroWin), true);
                }
                *foe_cd_left = foe_dt;
                return (None, true);
            }
        }
        (None, false)
    } else {
        (None, false)
    }
}

#[derive(Debug, Clone, Default)]
struct PartyBuffState {
    slot0: Vec<ActivePartyBuff>,
    slot1: Vec<ActivePartyBuff>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ActivePartyBuff {
    id: BuffId,
    stacks: u32,
    expires_at: Option<u32>,
    charges: Option<u32>,
}

impl PartyBuffState {
    fn apply_encounter_start(
        &mut self,
        items: &[(u8, BuffApplication)],
        combat_clock: u32,
        events: &mut Vec<CombatEvent>,
        max_events: usize,
    ) {
        for &(target, app) in items {
            if events.len() >= max_events {
                break;
            }
            self.apply_with_refresh(target, app, combat_clock, events, max_events);
        }
    }

    /// Merge by [`BuffId`] on the same party slot: add stacks, extend expiry to the later tick, refresh charges if set.
    fn apply_with_refresh(
        &mut self,
        target: u8,
        app: BuffApplication,
        combat_clock: u32,
        events: &mut Vec<CombatEvent>,
        max_events: usize,
    ) {
        if events.len() >= max_events {
            return;
        }
        let new_exp = buff_expires_at_clock(combat_clock, app.duration_ticks);
        let slot = match target {
            0 => &mut self.slot0,
            1 => &mut self.slot1,
            _ => return,
        };
        let emit_stacks = if let Some(idx) = slot.iter().position(|b| b.id == app.buff_id) {
            let b = &mut slot[idx];
            b.stacks = b.stacks.saturating_add(app.stacks);
            if let Some(ne) = new_exp {
                b.expires_at = Some(match b.expires_at {
                    Some(oe) => oe.max(ne),
                    None => ne,
                });
            }
            if app.charges.is_some() {
                b.charges = app.charges;
            }
            b.stacks
        } else {
            slot.push(ActivePartyBuff {
                id: app.buff_id,
                stacks: app.stacks,
                expires_at: new_exp,
                charges: app.charges,
            });
            app.stacks
        };
        events.push(CombatEvent::BuffApplied {
            target,
            buff_id: app.buff_id,
            stacks: emit_stacks,
            duration_ticks: app.duration_ticks,
        });
    }

    fn tick_end(&mut self, combat_clock: u32, events: &mut Vec<CombatEvent>, max_events: usize) {
        Self::expire_slot(&mut self.slot0, 0, combat_clock, events, max_events);
        Self::expire_slot(&mut self.slot1, 1, combat_clock, events, max_events);
    }

    fn expire_slot(
        slot: &mut Vec<ActivePartyBuff>,
        target: u8,
        combat_clock: u32,
        events: &mut Vec<CombatEvent>,
        max_events: usize,
    ) {
        let mut i = 0usize;
        while i < slot.len() {
            let drop = slot[i].expires_at.is_some_and(|ex| combat_clock >= ex);
            if drop {
                let id = slot[i].id;
                slot.remove(i);
                if events.len() < max_events {
                    events.push(CombatEvent::BuffExpired {
                        target,
                        buff_id: id,
                    });
                }
            } else {
                i += 1;
            }
        }
    }
}

/// `max_clock_ticks` — upper bound on combat time steps (each step adds attack speed to both
/// sides' action meters; extra hero or enemy swings in one step when speed is higher).
///
/// `partner`: optional second [`HeroProfile`] (party slot 1) and their current HP. Threat is
/// WoW-style: damage generates threat; tank-stance passives add baseline / drip aggro.
///
/// `initiative_run_salt`: mixed into per-encounter initiative (e.g. delve `seed` and room depth).
/// Pass **`0`** for stable ordering that depends only on enemy stats (tests / isolated calls).
///
/// **Phase 4 — shared ability GCD:** [`crate::domain::skills::skill_triggers_shared_ability_gcd`]
/// matches styles that advance **`h0_skill_gcd_left`** (instant strikes and next-melee buff queues).
/// **`SwingWeave`** (Heavy / Cleave cadence) uses the weapon cast/CD timers instead, not this GCD.
///
/// **Tick & ordering overview:** [`crate::domain::combat_timing`] (`COMBAT_TICK_MS`, initiative ranks).
pub fn simulate_combat_party(
    lead: &HeroProfile,
    enemy: &Enemy,
    max_clock_ticks: u32,
    lead_health_start: i32,
    partner: Option<(&HeroProfile, i32)>,
    initiative_run_salt: u64,
) -> CombatResult {
    simulate_combat_party_with_initial_buffs(
        lead,
        enemy,
        max_clock_ticks,
        lead_health_start,
        partner,
        initiative_run_salt,
        &[],
    )
}

/// Optional knobs for [`simulate_combat_party_with_options`]. Prefer [`Default`] for gameplay paths.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct CombatSimOptions {
    /// If set and player 1 has [`SkillId::VictoryRush`], overrides [`SkillDefinition::max_charges`] for
    /// this run (tests / tooling). Long-term gear should adjust [`PreparedHero::vr_max_charges`] in
    /// [`prepare_hero_combat`] instead.
    pub player0_instant_strike_max_charges: Option<u8>,
    /// Same as [`Self::player0_instant_strike_max_charges`] for party slot 1 when present.
    pub player1_instant_strike_max_charges: Option<u8>,
}

/// Like [`simulate_combat_party_with_initial_buffs`], with optional simulation overrides.
pub fn simulate_combat_party_with_options(
    lead: &HeroProfile,
    enemy: &Enemy,
    max_clock_ticks: u32,
    lead_health_start: i32,
    partner: Option<(&HeroProfile, i32)>,
    initiative_run_salt: u64,
    initial_party_buffs: &[(u8, BuffApplication)],
    options: CombatSimOptions,
) -> CombatResult {
    let mut p0 = prepare_hero_combat(lead);
    if let Some(n) = options.player0_instant_strike_max_charges {
        if p0.has_victory_rush {
            p0.vr_max_charges = n.max(1);
        }
    }
    let mut p1 = partner.map(|(h, hp)| (prepare_hero_combat(h), h, hp));
    if let Some(n) = options.player1_instant_strike_max_charges {
        if let Some((ref mut prep, _, _)) = p1 {
            if prep.has_victory_rush {
                prep.vr_max_charges = n.max(1);
            }
        }
    }
    let has_partner = p1.is_some();

    let partner_max = p1.as_ref().map(|(p, _, _)| p.max_h).unwrap_or(0);
    let mut h0 = lead_health_start.clamp(0, p0.max_h);
    let mut h1 = p1
        .as_ref()
        .map(|(prep, _, hp)| (*hp).clamp(0, prep.max_h))
        .unwrap_or(0);

    if h0 <= 0 {
        return CombatResult {
            outcome: CombatOutcome::EnemyWon,
            hero_health: 0,
            partner_health: if has_partner { Some(h1) } else { None },
            partner_max_health: if has_partner { Some(partner_max) } else { None },
            foe_healths: vec![enemy.max_health],
            clock_ticks: 0,
            events: vec![CombatEvent::HeroDefeated],
        };
    }
    if has_partner && h1 <= 0 {
        return CombatResult {
            outcome: CombatOutcome::EnemyWon,
            hero_health: h0,
            partner_health: Some(h1),
            partner_max_health: Some(partner_max),
            foe_healths: vec![enemy.max_health],
            clock_ticks: 0,
            events: vec![CombatEvent::PartyMemberDown { party_index: 1 }],
        };
    }

    let mut enemy_health = enemy.max_health;
    let poison_heal = |p: &PreparedHero| {
        if p.has_poison {
            p.stats.healing_power.max(0)
        } else {
            0
        }
    };
    let poison_tick = (3 + poison_heal(&p0).max(
        p1.as_ref()
            .map(|(prep, _, _)| poison_heal(prep))
            .unwrap_or(0),
    ) / 2)
        .clamp(1, 25);
    let has_poison_any = p0.has_poison || p1.as_ref().is_some_and(|(prep, _, _)| prep.has_poison);
    let has_toxic_any = p0.has_toxic_mastery
        || p1
            .as_ref()
            .is_some_and(|(prep, _, _)| prep.has_toxic_mastery);

    let mut barrier = [
        p0.barrier,
        p1.as_ref().map(|(p, _, _)| p.barrier).unwrap_or(0),
    ];

    let enemy_as = enemy.attack_speed.max(0.12);
    let mut meters = [0u64, 0u64];
    let mut enemy_meter = 0u64;

    let mut poison_stacks: u32 = 0;
    const POISON_DAMAGE_STACK_CAP: u32 = 12;

    let mut threat = [0i32; 2];
    if let Some((_, h1hero, _)) = &p1 {
        threat[1] = crate::domain::party::threat_stance_seed(h1hero);
    }

    let mut events = Vec::new();

    if p0.has_second_wind && h0 > 0 {
        let h = (p0.max_h / 20).max(1).min(8);
        if h > 0 && h0 < p0.max_h {
            h0 = (h0 + h).min(p0.max_h);
            events.push(CombatEvent::HeroHealed {
                target: 0,
                amount: h,
            });
        }
    }
    if let Some((p1prep, _, _)) = &p1 {
        if p1prep.has_second_wind && h1 > 0 {
            let h = (p1prep.max_h / 20).max(1).min(8);
            if h > 0 && h1 < p1prep.max_h {
                h1 = (h1 + h).min(p1prep.max_h);
                events.push(CombatEvent::HeroHealed {
                    target: 1,
                    amount: h,
                });
            }
        }
    }

    const MAX_EVENTS: usize = 1200;

    let mut party_buffs = PartyBuffState::default();
    party_buffs.apply_encounter_start(initial_party_buffs, 0, &mut events, MAX_EVENTS);

    macro_rules! hero_win {
        ($cc:expr) => {
            return CombatResult {
                outcome: CombatOutcome::HeroWon,
                hero_health: h0,
                partner_health: if has_partner { Some(h1) } else { None },
                partner_max_health: if has_partner { Some(partner_max) } else { None },
                foe_healths: vec![enemy_health],
                clock_ticks: $cc,
                events,
            };
        };
    }
    macro_rules! enemy_win {
        ($cc:expr) => {
            return CombatResult {
                outcome: CombatOutcome::EnemyWon,
                hero_health: h0,
                partner_health: if has_partner { Some(h1) } else { None },
                partner_max_health: if has_partner { Some(partner_max) } else { None },
                foe_healths: vec![enemy_health],
                clock_ticks: $cc,
                events,
            };
        };
    }

    let mut last_enemy_target: Option<u8> = None;

    let mut h0_cast_left = 0u32;
    let mut h0_cd_left = 0u32;
    let mut h0_skill_gcd_left = 0u32;
    let mut h0_skill_gcd_denom = 0u32;
    let mut h0_vr_recharge_left = 0u32;
    let mut h0_vr_charges = p0.vr_max_charges;
    let mut h0_empower_queued = false;
    let mut h0_empower_icd_left = 0u32;
    let mut h1_cast_left = 0u32;
    let mut h1_cd_left = 0u32;
    let mut h1_skill_gcd_left = 0u32;
    let mut h1_skill_gcd_denom = 0u32;
    let mut h1_vr_recharge_left = 0u32;
    let mut h1_vr_charges = p1.as_ref().map(|(p, _, _)| p.vr_max_charges).unwrap_or(0);
    let mut h1_empower_queued = false;
    let mut h1_empower_icd_left = 0u32;
    let mut foe_cast_left = 0u32;
    let mut foe_cd_left = 0u32;
    let foe_ct = enemy.cast_ticks;
    let foe_dt = enemy.cooldown_ticks;

    let mut combat_clock = 0u32;
    for tick in 0..max_clock_ticks {
        combat_clock = tick.saturating_add(1);
        if h0 <= 0 || enemy_health <= 0 || (has_partner && h1 <= 0) {
            break;
        }
        if events.len() >= MAX_EVENTS {
            break;
        }

        if let Some((_, h1hero, _)) = &p1 {
            if h0 > 0 && h1 > 0 && enemy_health > 0 {
                let partner_tank = crate::domain::party::threat_stance_tick_drip(h1hero);
                if crate::domain::party::tick_party_threat_routing(
                    &mut threat,
                    partner_tank,
                    combat_clock,
                ) && events.len() < MAX_EVENTS
                {
                    events.push(CombatEvent::ThreatSnapshot {
                        slot0: threat[0],
                        slot1: threat[1],
                    });
                }
                if partner_tank {
                    threat[1] = threat[1].saturating_add(1);
                }
            }
        }

        tick_instant_strike_charge_recharge(
            p0.has_victory_rush,
            p0.vr_max_charges,
            &mut h0_vr_charges,
            p0.vr_icd_ticks,
            &mut h0_vr_recharge_left,
        );
        if p0.has_empowered_blow && h0_empower_icd_left > 0 {
            h0_empower_icd_left -= 1;
        }
        if has_partner {
            let p1p = &p1.as_ref().unwrap().0;
            tick_instant_strike_charge_recharge(
                p1p.has_victory_rush,
                p1p.vr_max_charges,
                &mut h1_vr_charges,
                p1p.vr_icd_ticks,
                &mut h1_vr_recharge_left,
            );
            if p1p.has_empowered_blow && h1_empower_icd_left > 0 {
                h1_empower_icd_left -= 1;
            }
        }
        if p0.has_empowered_blow || p0.has_victory_rush {
            if h0_skill_gcd_left > 0 {
                h0_skill_gcd_left -= 1;
                if h0_skill_gcd_left == 0 {
                    h0_skill_gcd_denom = 0;
                }
            }
        }
        if has_partner {
            let p1p = &p1.as_ref().unwrap().0;
            if p1p.has_empowered_blow || p1p.has_victory_rush {
                if h1_skill_gcd_left > 0 {
                    h1_skill_gcd_left -= 1;
                    if h1_skill_gcd_left == 0 {
                        h1_skill_gcd_denom = 0;
                    }
                }
            }
        }
        let vr_ready = p0.has_victory_rush && h0_skill_gcd_left == 0 && h0_vr_charges > 0;
        let vr1_ready = has_partner && {
            let p1p = &p1.as_ref().unwrap().0;
            p1p.has_victory_rush && h1_skill_gcd_left == 0 && h1_vr_charges > 0
        };
        if p0.has_empowered_blow
            && !h0_empower_queued
            && h0_skill_gcd_left == 0
            && h0_empower_icd_left == 0
            && !vr_ready
        {
            h0_empower_queued = true;
            h0_skill_gcd_left = p0.empower_gcd_ticks.max(1) as u32;
            h0_skill_gcd_denom = h0_skill_gcd_left;
            if events.len() < MAX_EVENTS {
                events.push(CombatEvent::BuffApplied {
                    target: 0,
                    buff_id: BuffId::EmpoweredBlow,
                    stacks: 1,
                    duration_ticks: None,
                });
            }
        }

        if has_partner {
            let p1p = &p1.as_ref().unwrap().0;
            if p1p.has_empowered_blow
                && !h1_empower_queued
                && h1_skill_gcd_left == 0
                && h1_empower_icd_left == 0
                && !vr1_ready
            {
                h1_empower_queued = true;
                h1_skill_gcd_left = p1p.empower_gcd_ticks.max(1) as u32;
                h1_skill_gcd_denom = h1_skill_gcd_left;
                if events.len() < MAX_EVENTS {
                    events.push(CombatEvent::BuffApplied {
                        target: 1,
                        buff_id: BuffId::EmpoweredBlow,
                        stacks: 1,
                        duration_ticks: None,
                    });
                }
            }
        }

        let enc_seed = crate::domain::combat_timing::encounter_initiative_seed(
            initiative_run_salt,
            (enemy.max_health as u64) ^ ((enemy.damage as u64).rotate_left(17)),
        );
        let party_slots = if has_partner { 2u8 } else { 1u8 };
        let init_ranks = crate::domain::combat_timing::initiative_ranks(enc_seed, party_slots);
        let strike_order =
            crate::domain::combat_round::sorted_strike_actors(has_partner, init_ranks);

        let p1prep_ref = p1.as_ref().map(|(prep, _, _)| prep);

        let mut as_applied_tick = [false, false, false];
        loop {
            if h0 <= 0 || enemy_health <= 0 || (has_partner && h1 <= 0) {
                break;
            }
            if events.len() >= MAX_EVENTS {
                break;
            }
            let mut progressed = false;
            for &slot in &strike_order {
                if h0 <= 0 || enemy_health <= 0 || (has_partner && h1 <= 0) {
                    break;
                }
                if events.len() >= MAX_EVENTS {
                    break;
                }
                let (brk, prog) = match slot {
                    0 => lead_weapon_pass(
                        &p0,
                        lead,
                        enemy,
                        &mut meters,
                        &mut h0_cd_left,
                        &mut h0_cast_left,
                        &mut as_applied_tick,
                        &mut h0,
                        &mut enemy_health,
                        has_partner,
                        &mut poison_stacks,
                        &mut threat,
                        &mut events,
                        MAX_EVENTS,
                        &mut h0_skill_gcd_left,
                        &mut h0_skill_gcd_denom,
                        &mut h0_vr_charges,
                        &mut h0_vr_recharge_left,
                        &mut h0_empower_queued,
                        &mut h0_empower_icd_left,
                    ),
                    1 => {
                        if let Some((ref p1prep, partner_hero, _)) = p1 {
                            partner_weapon_pass(
                                p1prep,
                                partner_hero,
                                enemy,
                                &mut meters,
                                &mut h1_cd_left,
                                &mut h1_cast_left,
                                &mut as_applied_tick,
                                &mut h1,
                                &mut enemy_health,
                                &mut poison_stacks,
                                &mut threat,
                                &mut events,
                                MAX_EVENTS,
                                &mut h1_skill_gcd_left,
                                &mut h1_skill_gcd_denom,
                                &mut h1_vr_charges,
                                &mut h1_vr_recharge_left,
                                &mut h1_empower_queued,
                                &mut h1_empower_icd_left,
                            )
                        } else {
                            (None, false)
                        }
                    }
                    2 => foe_weapon_pass(
                        enemy,
                        &p0,
                        p1prep_ref,
                        &mut enemy_meter,
                        enemy_as,
                        &mut foe_cast_left,
                        &mut foe_cd_left,
                        foe_ct,
                        foe_dt,
                        &mut h0,
                        &mut h1,
                        has_partner,
                        &mut enemy_health,
                        &mut threat,
                        &mut barrier,
                        &mut last_enemy_target,
                        &mut as_applied_tick,
                        tick,
                        &mut events,
                        MAX_EVENTS,
                        lead,
                    ),
                    _ => (None, false),
                };
                progressed |= prog;
                match brk {
                    Some(CombatBreak::HeroWin) => {
                        hero_win!(combat_clock);
                    }
                    Some(CombatBreak::EnemyWin) => {
                        enemy_win!(combat_clock);
                    }
                    None => {}
                }
            }
            if !progressed {
                break;
            }
        }

        let player0_cast = if p0.attack_cast_total == 0 {
            0.0
        } else if h0_cast_left > 0 {
            1.0 - (h0_cast_left as f32 / p0.attack_cast_total as f32)
        } else {
            0.0
        };
        let player0_cd = if p0.attack_cd_total == 0 {
            0.0
        } else if h0_cd_left > 0 {
            1.0 - (h0_cd_left as f32 / p0.attack_cd_total as f32)
        } else {
            0.0
        };
        let player1_cast = if has_partner {
            if p1.as_ref().unwrap().0.attack_cast_total == 0 {
                0.0
            } else if h1_cast_left > 0 {
                1.0 - (h1_cast_left as f32 / p1.as_ref().unwrap().0.attack_cast_total as f32)
            } else {
                0.0
            }
        } else {
            0.0
        };
        let player1_cd = if has_partner {
            if p1.as_ref().unwrap().0.attack_cd_total == 0 {
                0.0
            } else if h1_cd_left > 0 {
                1.0 - (h1_cd_left as f32 / p1.as_ref().unwrap().0.attack_cd_total as f32)
            } else {
                0.0
            }
        } else {
            0.0
        };
        let foe_cast = if foe_ct == 0 {
            0.0
        } else if foe_cast_left > 0 {
            1.0 - (foe_cast_left as f32 / foe_ct as f32)
        } else {
            0.0
        };
        let foe_cd_b = if foe_dt == 0 {
            0.0
        } else if foe_cd_left > 0 {
            1.0 - (foe_cd_left as f32 / foe_dt as f32)
        } else {
            0.0
        };
        let player0_skill_gcd = ability_gcd_bar_frac(h0_skill_gcd_left, h0_skill_gcd_denom);
        let player0_instant_recharge = if p0.has_victory_rush {
            instant_recharge_bar_frac(
                h0_vr_recharge_left,
                p0.vr_icd_ticks,
                h0_vr_charges,
                p0.vr_max_charges,
            )
        } else {
            0.0
        };
        let (player0_instant_charges, player0_instant_max_charges) = if p0.has_victory_rush {
            (h0_vr_charges, p0.vr_max_charges)
        } else {
            (0, 0)
        };
        let (
            player1_skill_gcd,
            player1_instant_recharge,
            player1_instant_charges,
            player1_instant_max_charges,
        ) = if has_partner {
            let p1p = &p1.as_ref().unwrap().0;
            let g = ability_gcd_bar_frac(h1_skill_gcd_left, h1_skill_gcd_denom);
            let ir = if p1p.has_victory_rush {
                instant_recharge_bar_frac(
                    h1_vr_recharge_left,
                    p1p.vr_icd_ticks,
                    h1_vr_charges,
                    p1p.vr_max_charges,
                )
            } else {
                0.0
            };
            let (c, m) = if p1p.has_victory_rush {
                (h1_vr_charges, p1p.vr_max_charges)
            } else {
                (0, 0)
            };
            (g, ir, c, m)
        } else {
            (0.0, 0.0, 0, 0)
        };
        if events.len() < MAX_EVENTS {
            events.push(CombatEvent::TimingPulse {
                player0_cast,
                player0_cd,
                player1_cast,
                player1_cd,
                foe_cast,
                foe_cd: foe_cd_b,
                foe_alt_cast: 0.0,
                foe_alt_cd: 0.0,
                player0_skill_gcd,
                player0_instant_recharge,
                player0_instant_charges,
                player0_instant_max_charges,
                player1_skill_gcd,
                player1_instant_recharge,
                player1_instant_charges,
                player1_instant_max_charges,
            });
        }

        // **Poison scheduling:** one batched [`CombatEvent::PoisonTick`] per outer tick, after the
        // proactive strike-order loop and [`CombatEvent::TimingPulse`] for that tick. We intentionally
        // do **not** model DoT as independently scheduled sub-tick slices (see
        // [`ActionLane::Dot`](crate::domain::combat_timing::ActionLane) vs proactive initiative passes).
        if has_poison_any && poison_stacks > 0 && h0 > 0 && enemy_health > 0 {
            if events.len() >= MAX_EVENTS {
                break;
            }
            let potency = poison_stacks.min(POISON_DAMAGE_STACK_CAP);
            let mult = potency.max(1) as i32;
            let mut d = poison_tick * mult;
            if has_toxic_any {
                d = (d as i64 * 5 / 4).max(1) as i32;
            }
            d = d.min(enemy_health);
            events.push(CombatEvent::PoisonTick {
                damage: d,
                stacks: poison_stacks,
                foe_index: 0,
            });
            if events.len() < MAX_EVENTS {
                events.push(CombatEvent::BuffTick {
                    target: 0,
                    buff_id: BuffId::PoisonVenom,
                    stacks: potency,
                });
            }
            poison_stacks -= 1;
            enemy_health -= d;
            if enemy_health <= 0 {
                events.push(CombatEvent::EnemyDefeated { foe_index: 0 });
                maybe_devourer_heal_on_kill(lead, &mut h0, p0.max_h, &mut events);
                hero_win!(combat_clock);
            }
        }
        party_buffs.tick_end(combat_clock, &mut events, MAX_EVENTS);
    }

    let outcome = if h0 <= 0 {
        CombatOutcome::EnemyWon
    } else if enemy_health <= 0 {
        CombatOutcome::HeroWon
    } else {
        CombatOutcome::TimedOut
    };

    CombatResult {
        outcome,
        hero_health: h0,
        partner_health: if has_partner { Some(h1) } else { None },
        partner_max_health: if has_partner { Some(partner_max) } else { None },
        foe_healths: vec![enemy_health],
        clock_ticks: combat_clock,
        events,
    }
}

/// Multi-foe engagement: shared clock, per-foe meters/poison, [`SkillId::Cleave`] splashes every
/// **other** living foe on the same swing. Initiative uses [`initiative_ranks_pack`] (slots `0..party-1`
/// heroes, then foes in encounter order).
fn simulate_combat_party_foes(
    lead: &HeroProfile,
    foes: &[Enemy],
    max_clock_ticks: u32,
    lead_health_start: i32,
    partner: Option<(&HeroProfile, i32)>,
    initiative_run_salt: u64,
    initial_party_buffs: &[(u8, BuffApplication)],
    options: CombatSimOptions,
) -> CombatResult {
    assert!(
        !foes.is_empty() && foes.len() <= MAX_COMBAT_FOES,
        "simulate_combat_party_foes: foe count must be 1..={MAX_COMBAT_FOES}"
    );
    let foe_hp_start: Vec<i32> = foes.iter().map(|e| e.max_health).collect();

    let mut p0 = prepare_hero_combat(lead);
    if let Some(n) = options.player0_instant_strike_max_charges {
        if p0.has_victory_rush {
            p0.vr_max_charges = n.max(1);
        }
    }
    let mut p1 = partner.map(|(h, hp)| (prepare_hero_combat(h), h, hp));
    if let Some(n) = options.player1_instant_strike_max_charges {
        if let Some((ref mut prep, _, _)) = p1 {
            if prep.has_victory_rush {
                prep.vr_max_charges = n.max(1);
            }
        }
    }
    let has_partner = p1.is_some();
    let partner_max = p1.as_ref().map(|(p, _, _)| p.max_h).unwrap_or(0);
    let mut h0 = lead_health_start.clamp(0, p0.max_h);
    let mut h1 = p1
        .as_ref()
        .map(|(prep, _, hp)| (*hp).clamp(0, prep.max_h))
        .unwrap_or(0);

    if h0 <= 0 {
        return CombatResult {
            outcome: CombatOutcome::EnemyWon,
            hero_health: 0,
            partner_health: if has_partner { Some(h1) } else { None },
            partner_max_health: if has_partner { Some(partner_max) } else { None },
            foe_healths: foe_hp_start.clone(),
            clock_ticks: 0,
            events: vec![CombatEvent::HeroDefeated],
        };
    }
    if has_partner && h1 <= 0 {
        return CombatResult {
            outcome: CombatOutcome::EnemyWon,
            hero_health: h0,
            partner_health: Some(h1),
            partner_max_health: Some(partner_max),
            foe_healths: foe_hp_start.clone(),
            clock_ticks: 0,
            events: vec![CombatEvent::PartyMemberDown { party_index: 1 }],
        };
    }

    let party_count: u8 = if has_partner { 2 } else { 1 };
    let foe_count: u8 = foes.len() as u8;

    let mut enemy_health = foe_hp_start;
    let mut foe_slain_logged: Vec<bool> = vec![false; foes.len()];
    let poison_heal = |p: &PreparedHero| {
        if p.has_poison {
            p.stats.healing_power.max(0)
        } else {
            0
        }
    };
    let poison_tick = (3 + poison_heal(&p0).max(
        p1.as_ref()
            .map(|(prep, _, _)| poison_heal(prep))
            .unwrap_or(0),
    ) / 2)
        .clamp(1, 25);
    let has_poison_any = p0.has_poison || p1.as_ref().is_some_and(|(prep, _, _)| prep.has_poison);
    let has_toxic_any = p0.has_toxic_mastery
        || p1
            .as_ref()
            .is_some_and(|(prep, _, _)| prep.has_toxic_mastery);

    let mut barrier = [
        p0.barrier,
        p1.as_ref().map(|(p, _, _)| p.barrier).unwrap_or(0),
    ];

    let enemy_as: Vec<f32> = foes.iter().map(|e| e.attack_speed.max(0.12)).collect();
    let foe_ct: Vec<u32> = foes.iter().map(|e| e.cast_ticks).collect();
    let foe_dt: Vec<u32> = foes.iter().map(|e| e.cooldown_ticks).collect();

    let mut meters = [0u64, 0u64];
    let mut enemy_meter: Vec<u64> = vec![0u64; foes.len()];

    let mut poison_stacks: Vec<u32> = vec![0u32; foes.len()];
    const POISON_DAMAGE_STACK_CAP: u32 = 12;

    let mut threat = [0i32; 2];
    if let Some((_, h1hero, _)) = &p1 {
        threat[1] = crate::domain::party::threat_stance_seed(h1hero);
    }

    let mut events = Vec::new();

    if p0.has_second_wind && h0 > 0 {
        let h = (p0.max_h / 20).max(1).min(8);
        if h > 0 && h0 < p0.max_h {
            h0 = (h0 + h).min(p0.max_h);
            events.push(CombatEvent::HeroHealed {
                target: 0,
                amount: h,
            });
        }
    }
    if let Some((p1prep, _, _)) = &p1 {
        if p1prep.has_second_wind && h1 > 0 {
            let h = (p1prep.max_h / 20).max(1).min(8);
            if h > 0 && h1 < p1prep.max_h {
                h1 = (h1 + h).min(p1prep.max_h);
                events.push(CombatEvent::HeroHealed {
                    target: 1,
                    amount: h,
                });
            }
        }
    }

    const MAX_EVENTS: usize = 1200;

    let mut party_buffs = PartyBuffState::default();
    party_buffs.apply_encounter_start(initial_party_buffs, 0, &mut events, MAX_EVENTS);

    macro_rules! hero_win_pack {
        ($cc:expr) => {
            return CombatResult {
                outcome: CombatOutcome::HeroWon,
                hero_health: h0,
                partner_health: if has_partner { Some(h1) } else { None },
                partner_max_health: if has_partner { Some(partner_max) } else { None },
                foe_healths: vec![0i32; enemy_health.len()],
                clock_ticks: $cc,
                events,
            };
        };
    }
    macro_rules! enemy_win_pack {
        ($cc:expr) => {
            return CombatResult {
                outcome: CombatOutcome::EnemyWon,
                hero_health: h0,
                partner_health: if has_partner { Some(h1) } else { None },
                partner_max_health: if has_partner { Some(partner_max) } else { None },
                foe_healths: enemy_health.clone(),
                clock_ticks: $cc,
                events,
            };
        };
    }

    let mut last_enemy_target: Option<u8> = None;

    let mut h0_cast_left = 0u32;
    let mut h0_cd_left = 0u32;
    let mut h0_skill_gcd_left = 0u32;
    let mut h0_skill_gcd_denom = 0u32;
    let mut h0_vr_recharge_left = 0u32;
    let mut h0_vr_charges = p0.vr_max_charges;
    let mut h0_empower_queued = false;
    let mut h0_empower_icd_left = 0u32;
    let mut h1_cast_left = 0u32;
    let mut h1_cd_left = 0u32;
    let mut h1_skill_gcd_left = 0u32;
    let mut h1_skill_gcd_denom = 0u32;
    let mut h1_vr_recharge_left = 0u32;
    let mut h1_vr_charges = p1.as_ref().map(|(p, _, _)| p.vr_max_charges).unwrap_or(0);
    let mut h1_empower_queued = false;
    let mut h1_empower_icd_left = 0u32;
    let mut foe_cast_left: Vec<u32> = vec![0u32; foes.len()];
    let mut foe_cd_left: Vec<u32> = vec![0u32; foes.len()];

    let enc_mix = foes.iter().enumerate().fold(0u64, |acc, (i, e)| {
        acc ^ (e.max_health as u64).rotate_left((i * 19 + 3) as u32)
            ^ ((e.damage as u64).rotate_left((i * 11 + 47) as u32))
    });

    let mut combat_clock = 0u32;
    for tick in 0..max_clock_ticks {
        combat_clock = tick.saturating_add(1);
        if h0 <= 0 || !any_foe_alive_slice(&enemy_health) || (has_partner && h1 <= 0) {
            break;
        }
        if events.len() >= MAX_EVENTS {
            break;
        }

        if let Some((_, h1hero, _)) = &p1 {
            if h0 > 0 && h1 > 0 && any_foe_alive_slice(&enemy_health) {
                let partner_tank = crate::domain::party::threat_stance_tick_drip(h1hero);
                if crate::domain::party::tick_party_threat_routing(
                    &mut threat,
                    partner_tank,
                    combat_clock,
                ) && events.len() < MAX_EVENTS
                {
                    events.push(CombatEvent::ThreatSnapshot {
                        slot0: threat[0],
                        slot1: threat[1],
                    });
                }
                if partner_tank {
                    threat[1] = threat[1].saturating_add(1);
                }
            }
        }

        tick_instant_strike_charge_recharge(
            p0.has_victory_rush,
            p0.vr_max_charges,
            &mut h0_vr_charges,
            p0.vr_icd_ticks,
            &mut h0_vr_recharge_left,
        );
        if p0.has_empowered_blow && h0_empower_icd_left > 0 {
            h0_empower_icd_left -= 1;
        }
        if has_partner {
            let p1p = &p1.as_ref().unwrap().0;
            tick_instant_strike_charge_recharge(
                p1p.has_victory_rush,
                p1p.vr_max_charges,
                &mut h1_vr_charges,
                p1p.vr_icd_ticks,
                &mut h1_vr_recharge_left,
            );
            if p1p.has_empowered_blow && h1_empower_icd_left > 0 {
                h1_empower_icd_left -= 1;
            }
        }
        if p0.has_empowered_blow || p0.has_victory_rush {
            if h0_skill_gcd_left > 0 {
                h0_skill_gcd_left -= 1;
                if h0_skill_gcd_left == 0 {
                    h0_skill_gcd_denom = 0;
                }
            }
        }
        if has_partner {
            let p1p = &p1.as_ref().unwrap().0;
            if p1p.has_empowered_blow || p1p.has_victory_rush {
                if h1_skill_gcd_left > 0 {
                    h1_skill_gcd_left -= 1;
                    if h1_skill_gcd_left == 0 {
                        h1_skill_gcd_denom = 0;
                    }
                }
            }
        }
        let vr_ready = p0.has_victory_rush && h0_skill_gcd_left == 0 && h0_vr_charges > 0;
        let vr1_ready = has_partner && {
            let p1p = &p1.as_ref().unwrap().0;
            p1p.has_victory_rush && h1_skill_gcd_left == 0 && h1_vr_charges > 0
        };
        if p0.has_empowered_blow
            && !h0_empower_queued
            && h0_skill_gcd_left == 0
            && h0_empower_icd_left == 0
            && !vr_ready
        {
            h0_empower_queued = true;
            h0_skill_gcd_left = p0.empower_gcd_ticks.max(1) as u32;
            h0_skill_gcd_denom = h0_skill_gcd_left;
            if events.len() < MAX_EVENTS {
                events.push(CombatEvent::BuffApplied {
                    target: 0,
                    buff_id: BuffId::EmpoweredBlow,
                    stacks: 1,
                    duration_ticks: None,
                });
            }
        }

        if has_partner {
            let p1p = &p1.as_ref().unwrap().0;
            if p1p.has_empowered_blow
                && !h1_empower_queued
                && h1_skill_gcd_left == 0
                && h1_empower_icd_left == 0
                && !vr1_ready
            {
                h1_empower_queued = true;
                h1_skill_gcd_left = p1p.empower_gcd_ticks.max(1) as u32;
                h1_skill_gcd_denom = h1_skill_gcd_left;
                if events.len() < MAX_EVENTS {
                    events.push(CombatEvent::BuffApplied {
                        target: 1,
                        buff_id: BuffId::EmpoweredBlow,
                        stacks: 1,
                        duration_ticks: None,
                    });
                }
            }
        }

        let enc_seed =
            crate::domain::combat_timing::encounter_initiative_seed(initiative_run_salt, enc_mix);
        let init_ranks =
            crate::domain::combat_timing::initiative_ranks_pack(enc_seed, party_count, foe_count);
        let strike_order = crate::domain::combat_round::sorted_strike_pack_order(
            party_count,
            foe_count,
            &init_ranks,
        );

        let p1prep_ref = p1.as_ref().map(|(prep, _, _)| prep);
        let mut as_applied_tick: Vec<bool> = vec![false; party_count as usize + foes.len()];
        loop {
            if h0 <= 0 || !any_foe_alive_slice(&enemy_health) || (has_partner && h1 <= 0) {
                break;
            }
            if events.len() >= MAX_EVENTS {
                break;
            }
            let mut progressed = false;
            for &slot in &strike_order {
                if h0 <= 0 || !any_foe_alive_slice(&enemy_health) || (has_partner && h1 <= 0) {
                    break;
                }
                if events.len() >= MAX_EVENTS {
                    break;
                }
                let (brk, prog) = if slot == 0 {
                    dual_lead_weapon_pass(
                        &p0,
                        lead,
                        foes,
                        &mut meters,
                        &mut h0_cd_left,
                        &mut h0_cast_left,
                        &mut as_applied_tick,
                        &mut h0,
                        &mut enemy_health,
                        has_partner,
                        &mut poison_stacks,
                        &mut threat,
                        &mut events,
                        MAX_EVENTS,
                        &mut h0_skill_gcd_left,
                        &mut h0_skill_gcd_denom,
                        &mut h0_vr_charges,
                        &mut h0_vr_recharge_left,
                        &mut h0_empower_queued,
                        &mut h0_empower_icd_left,
                        &mut foe_slain_logged,
                    )
                } else if slot == 1 && has_partner {
                    if let Some((ref p1prep, partner_hero, _)) = p1 {
                        dual_partner_weapon_pass(
                            p1prep,
                            partner_hero,
                            foes,
                            &mut meters,
                            &mut h1_cd_left,
                            &mut h1_cast_left,
                            &mut as_applied_tick,
                            &mut h1,
                            &mut enemy_health,
                            &mut poison_stacks,
                            &mut threat,
                            &mut events,
                            MAX_EVENTS,
                            &mut h1_skill_gcd_left,
                            &mut h1_skill_gcd_denom,
                            &mut h1_vr_charges,
                            &mut h1_vr_recharge_left,
                            &mut h1_empower_queued,
                            &mut h1_empower_icd_left,
                            &mut foe_slain_logged,
                        )
                    } else {
                        (None, false)
                    }
                } else if slot >= party_count {
                    let fi = (slot - party_count) as usize;
                    if fi < foes.len() {
                        foe_weapon_pass_pack_slot(
                            fi as u8,
                            &foes[fi],
                            &p0,
                            p1prep_ref,
                            &mut enemy_meter,
                            enemy_as[fi],
                            &mut foe_cast_left,
                            &mut foe_cd_left,
                            foe_ct[fi],
                            foe_dt[fi],
                            &mut h0,
                            &mut h1,
                            has_partner,
                            &mut enemy_health,
                            &mut threat,
                            &mut barrier,
                            &mut last_enemy_target,
                            &mut as_applied_tick,
                            party_count as usize + fi,
                            tick,
                            &mut events,
                            MAX_EVENTS,
                            lead,
                            &mut foe_slain_logged,
                        )
                    } else {
                        (None, false)
                    }
                } else {
                    (None, false)
                };
                progressed |= prog;
                match brk {
                    Some(CombatBreak::HeroWin) => {
                        hero_win_pack!(combat_clock);
                    }
                    Some(CombatBreak::EnemyWin) => {
                        enemy_win_pack!(combat_clock);
                    }
                    None => {}
                }
            }
            if !progressed {
                break;
            }
        }

        let player0_cast = if p0.attack_cast_total == 0 {
            0.0
        } else if h0_cast_left > 0 {
            1.0 - (h0_cast_left as f32 / p0.attack_cast_total as f32)
        } else {
            0.0
        };
        let player0_cd = if p0.attack_cd_total == 0 {
            0.0
        } else if h0_cd_left > 0 {
            1.0 - (h0_cd_left as f32 / p0.attack_cd_total as f32)
        } else {
            0.0
        };
        let player1_cast = if has_partner {
            if p1.as_ref().unwrap().0.attack_cast_total == 0 {
                0.0
            } else if h1_cast_left > 0 {
                1.0 - (h1_cast_left as f32 / p1.as_ref().unwrap().0.attack_cast_total as f32)
            } else {
                0.0
            }
        } else {
            0.0
        };
        let player1_cd = if has_partner {
            if p1.as_ref().unwrap().0.attack_cd_total == 0 {
                0.0
            } else if h1_cd_left > 0 {
                1.0 - (h1_cd_left as f32 / p1.as_ref().unwrap().0.attack_cd_total as f32)
            } else {
                0.0
            }
        } else {
            0.0
        };

        let fd = focus_fire_primary_idx_slice(&enemy_health);
        let fd = fd.min(foes.len().saturating_sub(1));
        let fct = foe_ct[fd];
        let fdt = foe_dt[fd];
        let foe_cast = if fct == 0 {
            0.0
        } else if foe_cast_left[fd] > 0 {
            1.0 - (foe_cast_left[fd] as f32 / fct as f32)
        } else {
            0.0
        };
        let foe_cd_b = if fdt == 0 {
            0.0
        } else if foe_cd_left[fd] > 0 {
            1.0 - (foe_cd_left[fd] as f32 / fdt as f32)
        } else {
            0.0
        };
        let player0_skill_gcd = ability_gcd_bar_frac(h0_skill_gcd_left, h0_skill_gcd_denom);
        let player0_instant_recharge = if p0.has_victory_rush {
            instant_recharge_bar_frac(
                h0_vr_recharge_left,
                p0.vr_icd_ticks,
                h0_vr_charges,
                p0.vr_max_charges,
            )
        } else {
            0.0
        };
        let (player0_instant_charges, player0_instant_max_charges) = if p0.has_victory_rush {
            (h0_vr_charges, p0.vr_max_charges)
        } else {
            (0, 0)
        };
        let (
            player1_skill_gcd,
            player1_instant_recharge,
            player1_instant_charges,
            player1_instant_max_charges,
        ) = if has_partner {
            let p1p = &p1.as_ref().unwrap().0;
            let g = ability_gcd_bar_frac(h1_skill_gcd_left, h1_skill_gcd_denom);
            let ir = if p1p.has_victory_rush {
                instant_recharge_bar_frac(
                    h1_vr_recharge_left,
                    p1p.vr_icd_ticks,
                    h1_vr_charges,
                    p1p.vr_max_charges,
                )
            } else {
                0.0
            };
            let (c, m) = if p1p.has_victory_rush {
                (h1_vr_charges, p1p.vr_max_charges)
            } else {
                (0, 0)
            };
            (g, ir, c, m)
        } else {
            (0.0, 0.0, 0, 0)
        };
        let mut foe_alt_idx = None;
        for ai in 0..foes.len() {
            if ai != fd && enemy_health[ai] > 0 {
                foe_alt_idx = Some(ai);
                break;
            }
        }
        let (foe_alt_cast, foe_alt_cd) = if let Some(ai) = foe_alt_idx {
            let fct_a = foe_ct[ai];
            let fdt_a = foe_dt[ai];
            let fc_a = if fct_a == 0 {
                0.0
            } else if foe_cast_left[ai] > 0 {
                1.0 - (foe_cast_left[ai] as f32 / fct_a as f32)
            } else {
                0.0
            };
            let cd_a = if fdt_a == 0 {
                0.0
            } else if foe_cd_left[ai] > 0 {
                1.0 - (foe_cd_left[ai] as f32 / fdt_a as f32)
            } else {
                0.0
            };
            (fc_a, cd_a)
        } else {
            (0.0, 0.0)
        };
        if events.len() < MAX_EVENTS {
            events.push(CombatEvent::TimingPulse {
                player0_cast,
                player0_cd,
                player1_cast,
                player1_cd,
                foe_cast,
                foe_cd: foe_cd_b,
                foe_alt_cast,
                foe_alt_cd,
                player0_skill_gcd,
                player0_instant_recharge,
                player0_instant_charges,
                player0_instant_max_charges,
                player1_skill_gcd,
                player1_instant_recharge,
                player1_instant_charges,
                player1_instant_max_charges,
            });
        }

        if has_poison_any && h0 > 0 && any_foe_alive_slice(&enemy_health) {
            for foe_i in 0..foes.len() {
                if events.len() >= MAX_EVENTS {
                    break;
                }
                if poison_stacks[foe_i] == 0 || enemy_health[foe_i] <= 0 {
                    continue;
                }
                let potency = poison_stacks[foe_i].min(POISON_DAMAGE_STACK_CAP);
                let mult = potency.max(1) as i32;
                let mut d = poison_tick * mult;
                if has_toxic_any {
                    d = (d as i64 * 5 / 4).max(1) as i32;
                }
                d = d.min(enemy_health[foe_i]);
                events.push(CombatEvent::PoisonTick {
                    damage: d,
                    stacks: poison_stacks[foe_i],
                    foe_index: foe_i as u8,
                });
                if events.len() < MAX_EVENTS {
                    events.push(CombatEvent::BuffTick {
                        target: 0,
                        buff_id: BuffId::PoisonVenom,
                        stacks: potency,
                    });
                }
                poison_stacks[foe_i] -= 1;
                enemy_health[foe_i] -= d;
                if enemy_health[foe_i] <= 0 && !foe_slain_logged[foe_i] {
                    foe_slain_logged[foe_i] = true;
                    if events.len() < MAX_EVENTS {
                        events.push(CombatEvent::EnemyDefeated {
                            foe_index: foe_i as u8,
                        });
                    }
                    maybe_devourer_heal_on_kill(lead, &mut h0, p0.max_h, &mut events);
                    if all_foes_dead(&enemy_health) {
                        hero_win_pack!(combat_clock);
                    }
                }
            }
        }
        party_buffs.tick_end(combat_clock, &mut events, MAX_EVENTS);
    }

    let outcome = if h0 <= 0 {
        CombatOutcome::EnemyWon
    } else if all_foes_dead(&enemy_health) {
        CombatOutcome::HeroWon
    } else {
        CombatOutcome::TimedOut
    };

    CombatResult {
        outcome,
        hero_health: h0,
        partner_health: if has_partner { Some(h1) } else { None },
        partner_max_health: if has_partner { Some(partner_max) } else { None },
        foe_healths: enemy_health,
        clock_ticks: combat_clock,
        events,
    }
}

/// Multi-foe encounters (`2`..=[`MAX_COMBAT_FOES`]) use [`simulate_combat_party_foes`].
pub fn simulate_party_vs_encounter_foes(
    lead: &HeroProfile,
    foes: &[Enemy],
    max_clock_ticks: u32,
    lead_health_start: i32,
    partner: Option<(&HeroProfile, i32)>,
    initiative_run_salt: u64,
    initial_party_buffs: &[(u8, BuffApplication)],
    options: CombatSimOptions,
) -> CombatResult {
    assert!(!foes.is_empty() && foes.len() <= MAX_COMBAT_FOES);
    if foes.len() == 1 {
        return simulate_combat_party_with_options(
            lead,
            &foes[0],
            max_clock_ticks,
            lead_health_start,
            partner,
            initiative_run_salt,
            initial_party_buffs,
            options,
        );
    }
    simulate_combat_party_foes(
        lead,
        foes,
        max_clock_ticks,
        lead_health_start,
        partner,
        initiative_run_salt,
        initial_party_buffs,
        options,
    )
}

/// Like [`simulate_combat_party`], but applies [`BuffApplication`] entries at encounter clock **0**.
pub fn simulate_combat_party_with_initial_buffs(
    lead: &HeroProfile,
    enemy: &Enemy,
    max_clock_ticks: u32,
    lead_health_start: i32,
    partner: Option<(&HeroProfile, i32)>,
    initiative_run_salt: u64,
    initial_party_buffs: &[(u8, BuffApplication)],
) -> CombatResult {
    simulate_combat_party_with_options(
        lead,
        enemy,
        max_clock_ticks,
        lead_health_start,
        partner,
        initiative_run_salt,
        initial_party_buffs,
        CombatSimOptions::default(),
    )
}

/// Solo combat: no party partner (tests and legacy call sites).
pub fn simulate_combat(
    hero: &HeroProfile,
    enemy: &Enemy,
    max_clock_ticks: u32,
    hero_health_start: i32,
) -> CombatResult {
    simulate_combat_party(hero, enemy, max_clock_ticks, hero_health_start, None, 0)
}

/// Heal after the foe is marked defeated (quest / UI ordering: defeat line, then sustain).
fn maybe_devourer_heal_on_kill(
    hero: &HeroProfile,
    hero_health: &mut i32,
    max_h: i32,
    events: &mut Vec<CombatEvent>,
) {
    if !hero.has_affix(ItemAffix::Devourer) {
        return;
    }
    let h = (max_h / 12).max(2).min(14);
    if h > 0 && *hero_health < max_h {
        *hero_health = (*hero_health + h).min(max_h);
        events.push(CombatEvent::HeroHealed {
            target: 0,
            amount: h,
        });
    }
}

fn empty_debuff_slots() -> [String; 4] {
    [
        "—".to_string(),
        "—".to_string(),
        "—".to_string(),
        "—".to_string(),
    ]
}

fn enemy_status_slots_from_poison(enemy_poison: u32) -> [String; 4] {
    let mut enemy = empty_debuff_slots();
    if enemy_poison > 0 {
        enemy[0] = format!("Poison ×{}", enemy_poison);
    }
    enemy
}

fn buff_id_playback_order(id: BuffId) -> u8 {
    match id {
        BuffId::InnerStrength => 0,
        BuffId::EmpoweredBlow => 1,
        BuffId::PoisonVenom => 2,
        BuffId::VictoryRush => 3,
    }
}

fn buff_chip_label(buff_id: BuffId, stacks: u32) -> String {
    let name = buff_display_name(buff_id);
    let st = stacks.max(1);
    if st > 1 {
        format!("{name} ×{st}")
    } else {
        name.to_string()
    }
}

/// Party strip: self-poison (unused today in replay) plus player 1 buffs, then player 2 (max 4 cells).
fn hero_party_status_slots_from_buff_maps(
    hero_self_poison: u32,
    player0: &HashMap<BuffId, u32>,
    player1: &HashMap<BuffId, u32>,
) -> [String; 4] {
    let mut slots = empty_debuff_slots();
    let mut i = 0usize;
    if hero_self_poison > 0 && i < 4 {
        slots[i] = format!("Poison ×{}", hero_self_poison);
        i += 1;
    }
    let mut p0_entries: Vec<(BuffId, u32)> = player0.iter().map(|(&k, &v)| (k, v.max(1))).collect();
    p0_entries.sort_by_key(|(k, _)| buff_id_playback_order(*k));
    for (bid, st) in p0_entries {
        if i >= 4 {
            break;
        }
        slots[i] = buff_chip_label(bid, st);
        i += 1;
    }
    let mut p1_entries: Vec<(BuffId, u32)> = player1.iter().map(|(&k, &v)| (k, v.max(1))).collect();
    p1_entries.sort_by_key(|(k, _)| buff_id_playback_order(*k));
    for (bid, st) in p1_entries {
        if i >= 4 {
            break;
        }
        slots[i] = format!("P2 {}", buff_chip_label(bid, st));
        i += 1;
    }
    slots
}

pub fn combat_playback_frames_from_result(
    lead: &HeroProfile,
    partner: Option<&HeroProfile>,
    result: &CombatResult,
    enemy_name: &str,
    hero_max_hp: i32,
    enemy_max_hp: i32,
    hero_hp_at_start: i32,
    partner_hp_at_start: Option<i32>,
    partner_max_hp: Option<i32>,
    run_meter_party_0: u32,
    run_meter_party_1: u32,
    run_meter_foe: u32,
    run_sim_ticks_base: u32,
) -> Vec<CombatPlaybackFrame> {
    let partner_name = partner.map(|p| p.name.as_str());
    let has_poison = lead.equipped_skill_ids().any(|s| s == SkillId::PoisonEdge)
        || partner.is_some_and(|p| p.equipped_skill_ids().any(|s| s == SkillId::PoisonEdge));

    let mut hero_hp = hero_hp_at_start.clamp(0, hero_max_hp);
    let mut partner_hp = match (partner_hp_at_start, partner_max_hp) {
        (Some(h), Some(m)) => Some(h.clamp(0, m)),
        _ => None,
    };
    let mut enemy_hp = enemy_max_hp;
    let mut enemy_poison_stacks = 0u32;
    let hero_poison_stacks = 0u32;

    let mut foe_last_target: Option<u8> = None;
    let mut threat_slot0: Option<i32> = None;
    let mut threat_slot1: Option<i32> = None;

    let mut player0_buff_chips: HashMap<BuffId, u32> = HashMap::new();
    let mut player1_buff_chips: HashMap<BuffId, u32> = HashMap::new();

    let open_pre = opening_playback_prefix_len(&result.events);
    for ev in result.events.iter().take(open_pre) {
        playback_apply_opening_event(
            ev,
            hero_max_hp,
            partner_max_hp,
            &mut hero_hp,
            &mut partner_hp,
            &mut player0_buff_chips,
            &mut player1_buff_chips,
        );
    }

    let h0 = hero_party_status_slots_from_buff_maps(
        hero_poison_stacks,
        &player0_buff_chips,
        &player1_buff_chips,
    );
    let e0 = enemy_status_slots_from_poison(enemy_poison_stacks);

    let mut d0 = 0u32;
    let mut d1 = 0u32;
    let mut foe_meter = 0u32;

    let mut last_caption = format!("Engaging {enemy_name}.");

    let steps = flatten_playback_steps(&result.events[open_pre..]);
    let total_frames = (1 + steps.len()).max(1) as u32;
    let combat_ticks = result.clock_ticks.max(1);

    let run_sim_tick_for = |frame_index: u32| -> u32 {
        if total_frames <= 1 {
            return run_sim_ticks_base.saturating_add(combat_ticks);
        }
        let last_ix = total_frames - 1;
        run_sim_ticks_base.saturating_add(combat_ticks.saturating_mul(frame_index) / last_ix)
    };

    let mut frames = vec![CombatPlaybackFrame {
        enemy_name: enemy_name.to_string(),
        hero_hp,
        hero_max_hp,
        partner_hp,
        partner_max_hp,
        enemy_hp,
        enemy_max_hp,
        caption: last_caption.clone(),
        hero_debuff_slots: h0,
        enemy_debuff_slots: e0,
        foe_last_target: None,
        threat_slot0: None,
        threat_slot1: None,
        damage_meter_party_0: run_meter_party_0,
        damage_meter_party_1: run_meter_party_1,
        damage_meter_foe: run_meter_foe,
        run_sim_ticks: run_sim_tick_for(0),
        sfx_anchor: CombatSfxAnchor::Neutral,
        player0_cast: 0.0,
        player0_cd: 0.0,
        player1_cast: 0.0,
        player1_cd: 0.0,
        foe_cast: 0.0,
        foe_cd: 0.0,
        foe_alt_cast: 0.0,
        foe_alt_cd: 0.0,
        player0_skill_gcd: 0.0,
        player0_instant_recharge: 0.0,
        player0_instant_charges: 0,
        player0_instant_max_charges: 0,
        player1_skill_gcd: 0.0,
        player1_instant_recharge: 0.0,
        player1_instant_charges: 0,
        player1_instant_max_charges: 0,
    }];

    let mut player0_cast = 0.0f32;
    let mut player0_cd = 0.0f32;
    let mut player1_cast = 0.0f32;
    let mut player1_cd = 0.0f32;
    let mut foe_cast_b = 0.0f32;
    let mut foe_cd_bar = 0.0f32;
    let mut foe_alt_cast_b = 0.0f32;
    let mut foe_alt_cd_bar = 0.0f32;
    let mut player0_skill_gcd = 0.0f32;
    let mut player0_instant_recharge = 0.0f32;
    let mut player0_instant_charges = 0u8;
    let mut player0_instant_max_charges = 0u8;
    let mut player1_skill_gcd = 0.0f32;
    let mut player1_instant_recharge = 0.0f32;
    let mut player1_instant_charges = 0u8;
    let mut player1_instant_max_charges = 0u8;

    for step in steps {
        match &step {
            PlaybackStep::Event(event) => match event {
                CombatEvent::HeroAttacked {
                    attacker,
                    strike,
                    cleave_strikes,
                    ..
                } => {
                    let total =
                        strike.total() + cleave_strikes.iter().map(|(_, c)| c.total()).sum::<i32>();
                    if *attacker == 0 {
                        d0 = d0.saturating_add(total as u32);
                    } else {
                        d1 = d1.saturating_add(total as u32);
                    }
                    enemy_hp -= total;
                    if has_poison && strike.white > 0 {
                        enemy_poison_stacks = (enemy_poison_stacks + 2).min(40);
                    }
                }
                CombatEvent::PoisonTick { damage, stacks, .. } => {
                    d0 = d0.saturating_add(*damage as u32);
                    enemy_hp -= damage;
                    enemy_poison_stacks = stacks.saturating_sub(1);
                }
                CombatEvent::ThornsReflect { damage, .. } => {
                    d0 = d0.saturating_add(*damage as u32);
                    enemy_hp -= damage;
                }
                CombatEvent::HeroHealed { target, amount } => {
                    if *target == 0 {
                        hero_hp = (hero_hp + amount).min(hero_max_hp);
                    } else if let (Some(a), Some(m)) = (partner_hp.as_mut(), partner_max_hp) {
                        *a = (*a + amount).min(m);
                    }
                }
                CombatEvent::EnemyAttacked { target, damage } => {
                    foe_last_target = Some(*target);
                    foe_meter = foe_meter.saturating_add(*damage as u32);
                    if *target == 0 {
                        hero_hp -= damage;
                    } else if let (Some(a), Some(m)) = (partner_hp.as_mut(), partner_max_hp) {
                        *a = (*a - damage).clamp(0, m);
                    }
                }
                CombatEvent::ThreatSnapshot { slot0, slot1 } => {
                    threat_slot0 = Some(*slot0);
                    threat_slot1 = Some(*slot1);
                }
                CombatEvent::TimingPulse {
                    player0_cast: lc,
                    player0_cd: lcdn,
                    player1_cast: ac,
                    player1_cd: acdn,
                    foe_cast: fc,
                    foe_cd: fcdn,
                    foe_alt_cast: fac,
                    foe_alt_cd: facd,
                    player0_skill_gcd: lsg,
                    player0_instant_recharge: lir,
                    player0_instant_charges: lic,
                    player0_instant_max_charges: lim,
                    player1_skill_gcd: asg,
                    player1_instant_recharge: air,
                    player1_instant_charges: aic,
                    player1_instant_max_charges: aim,
                } => {
                    player0_cast = *lc;
                    player0_cd = *lcdn;
                    player1_cast = *ac;
                    player1_cd = *acdn;
                    foe_cast_b = *fc;
                    foe_cd_bar = *fcdn;
                    foe_alt_cast_b = *fac;
                    foe_alt_cd_bar = *facd;
                    player0_skill_gcd = *lsg;
                    player0_instant_recharge = *lir;
                    player0_instant_charges = *lic;
                    player0_instant_max_charges = *lim;
                    player1_skill_gcd = *asg;
                    player1_instant_recharge = *air;
                    player1_instant_charges = *aic;
                    player1_instant_max_charges = *aim;
                }
                CombatEvent::BuffApplied {
                    target,
                    buff_id,
                    stacks,
                    ..
                } => {
                    let map = if *target == 0 {
                        &mut player0_buff_chips
                    } else {
                        &mut player1_buff_chips
                    };
                    map.insert(*buff_id, (*stacks).max(1));
                }
                CombatEvent::BuffExpired { target, buff_id } => {
                    let map = if *target == 0 {
                        &mut player0_buff_chips
                    } else {
                        &mut player1_buff_chips
                    };
                    map.remove(buff_id);
                }
                CombatEvent::BuffTick { .. } | CombatEvent::BuffChargeConsumed { .. } => {}
                CombatEvent::PartyMemberDown { .. } => {}
                CombatEvent::EnemyDefeated { .. } | CombatEvent::HeroDefeated => {}
            },
            PlaybackStep::MergedPoison {
                total_damage,
                tick_count,
                stacks_before_last,
                foe_index: _,
            } => {
                d0 = d0.saturating_add(*total_damage as u32);
                enemy_hp -= *total_damage;
                enemy_poison_stacks = stacks_before_last.saturating_sub(1);
                let _ = tick_count;
            }
        }

        let partner_clamped = match (partner_hp, partner_max_hp) {
            (Some(h), Some(m)) => Some(h.clamp(0, m)),
            _ => None,
        };
        let hd = hero_party_status_slots_from_buff_maps(
            hero_poison_stacks,
            &player0_buff_chips,
            &player1_buff_chips,
        );
        let ed = enemy_status_slots_from_poison(enemy_poison_stacks);

        let (caption, sfx_anchor) = match &step {
            PlaybackStep::Event(CombatEvent::TimingPulse { .. }) => {
                (last_caption.clone(), CombatSfxAnchor::Neutral)
            }
            PlaybackStep::Event(ev) => {
                let c = combat_event_caption(ev, partner_name);
                last_caption = c.clone();
                (c, sfx_anchor_for_event(ev))
            }
            PlaybackStep::MergedPoison {
                total_damage,
                tick_count,
                ..
            } => {
                let c = merged_poison_caption(*total_damage, *tick_count);
                last_caption = c.clone();
                (c, CombatSfxAnchor::Enemy)
            }
        };

        let frame_ix = frames.len() as u32;
        frames.push(CombatPlaybackFrame {
            enemy_name: enemy_name.to_string(),
            hero_hp: hero_hp.clamp(0, hero_max_hp),
            hero_max_hp,
            partner_hp: partner_clamped,
            partner_max_hp,
            enemy_hp: enemy_hp.clamp(0, enemy_max_hp),
            enemy_max_hp,
            caption,
            hero_debuff_slots: hd,
            enemy_debuff_slots: ed,
            foe_last_target,
            threat_slot0,
            threat_slot1,
            damage_meter_party_0: run_meter_party_0.saturating_add(d0),
            damage_meter_party_1: run_meter_party_1.saturating_add(d1),
            damage_meter_foe: run_meter_foe.saturating_add(foe_meter),
            run_sim_ticks: run_sim_tick_for(frame_ix),
            sfx_anchor,
            player0_cast,
            player0_cd,
            player1_cast,
            player1_cd,
            foe_cast: foe_cast_b,
            foe_cd: foe_cd_bar,
            foe_alt_cast: foe_alt_cast_b,
            foe_alt_cd: foe_alt_cd_bar,
            player0_skill_gcd,
            player0_instant_recharge,
            player0_instant_charges,
            player0_instant_max_charges,
            player1_skill_gcd,
            player1_instant_recharge,
            player1_instant_charges,
            player1_instant_max_charges,
        });
        partner_hp = partner_clamped;
    }
    frames
}

pub fn combat_playback_frames(
    hero: &HeroProfile,
    enemy: &Enemy,
    max_ticks: u32,
) -> Vec<CombatPlaybackFrame> {
    let stats = hero.derived_stats();
    let hero_start = stats.max_health;
    let result = simulate_combat(hero, enemy, max_ticks, hero_start);
    combat_playback_frames_from_result(
        hero,
        None,
        &result,
        &enemy.name,
        stats.max_health,
        enemy.max_health,
        hero_start,
        None,
        None,
        0,
        0,
        0,
        0,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::buff::BuffId;
    use crate::domain::dungeon::Enemy;
    use crate::domain::hero::HeroProfile;
    use crate::domain::items::{GearSlot, ItemAffix, ItemInstance};
    use crate::domain::skills::SkillId;
    use crate::domain::stats::Stats;

    #[test]
    fn party_strike_damage_white_yellow_sums_primary_and_cleave() {
        let events = vec![CombatEvent::HeroAttacked {
            attacker: 0,
            strike: HeroStrikeDamage {
                white: 10,
                yellow: 5,
                yellow_source_skill: None,
            },
            foe_primary: 0,
            cleave_strikes: vec![(
                1,
                HeroStrikeDamage {
                    white: 2,
                    yellow: 3,
                    yellow_source_skill: None,
                },
            )],
        }];
        assert_eq!(party_strike_damage_white_yellow(&events), (12, 8));
    }

    #[test]
    fn pick_party_enemy_solo_always_targets_player0() {
        assert_eq!(
            pick_party_enemy_target(0, [99, 1], 100, 100, false, Some(1)),
            0
        );
    }

    #[test]
    fn pick_party_enemy_player1_down_targets_player0() {
        assert_eq!(
            pick_party_enemy_target(0, [1, 99], 100, 0, true, Some(1)),
            0
        );
    }

    #[test]
    fn pick_party_enemy_higher_threat_on_player0_targets_player0() {
        assert_eq!(
            pick_party_enemy_target(0, [30, 10], 100, 100, true, Some(1)),
            0
        );
    }

    #[test]
    fn pick_party_enemy_higher_threat_on_player1_targets_player1() {
        assert_eq!(
            pick_party_enemy_target(0, [10, 40], 100, 100, true, Some(0)),
            1
        );
    }

    #[test]
    fn pick_party_enemy_threat_tie_keeps_last_target_on_player1() {
        assert_eq!(
            pick_party_enemy_target(7, [20, 20], 100, 100, true, Some(1)),
            1
        );
    }

    #[test]
    fn pick_party_enemy_threat_tie_keeps_last_target_on_player0() {
        assert_eq!(
            pick_party_enemy_target(7, [20, 20], 100, 100, true, Some(0)),
            0
        );
    }

    #[test]
    fn pick_party_enemy_strict_threat_beats_sticky_memory() {
        assert_eq!(
            pick_party_enemy_target(0, [50, 10], 100, 100, true, Some(1)),
            0
        );
    }

    #[test]
    fn pick_party_enemy_invalid_sticky_falls_back_when_only_player1_alive() {
        assert_eq!(pick_party_enemy_target(3, [5, 5], 0, 100, true, Some(0)), 1);
    }

    #[test]
    fn pick_party_enemy_tie_without_sticky_is_stable_for_tick() {
        let t = pick_party_enemy_target(4, [0, 0], 100, 100, true, None);
        assert_eq!(t, pick_party_enemy_target(4, [0, 0], 100, 100, true, None));
    }

    #[test]
    fn scheduler_extreme_attack_speed_is_deterministic() {
        let mut hero = HeroProfile::new(Stats {
            damage: 12,
            attack_speed: 80.0,
            ..Stats::default()
        });
        hero.unlock_skill_slots(1);

        let enemy = Enemy {
            name: "Speed bag".into(),
            max_health: 200,
            damage: 1,
            armor: 0,
            attack_speed: 80.0,
            cast_ticks: 0,
            cooldown_ticks: 0,
        };

        let a = simulate_combat_party(
            &hero,
            &enemy,
            40,
            hero.derived_stats().max_health,
            None,
            0xC0FFEE,
        );
        let b = simulate_combat_party(
            &hero,
            &enemy,
            40,
            hero.derived_stats().max_health,
            None,
            0xC0FFEE,
        );
        assert_eq!(a.outcome, b.outcome);
        assert_eq!(a.events, b.events);
    }

    #[test]
    fn mutual_ohko_outcome_follows_initiative_order_solo() {
        use crate::domain::combat_timing::{encounter_initiative_seed, initiative_ranks};

        let hero = HeroProfile::new(Stats {
            max_health: 50,
            damage: 50,
            armor: 0,
            attack_speed: 100.0,
            healing_power: 0,
        });
        let enemy = Enemy {
            name: "Mirror".into(),
            max_health: 50,
            damage: 50,
            armor: 0,
            attack_speed: 100.0,
            cast_ticks: 0,
            cooldown_ticks: 0,
        };
        let mix = (enemy.max_health as u64) ^ ((enemy.damage as u64).rotate_left(17));

        let mut hero_first_salt = None;
        let mut foe_first_salt = None;
        for salt in 0u64..512 {
            let enc = encounter_initiative_seed(salt, mix);
            let r = initiative_ranks(enc, 1);
            if r[0] < r[2] {
                hero_first_salt.get_or_insert(salt);
            } else {
                foe_first_salt.get_or_insert(salt);
            }
        }
        let hs = hero_first_salt.expect("expected a solo salt where lead swings before foe");
        let fs = foe_first_salt.expect("expected a solo salt where foe swings before lead");

        let hero_wins = simulate_combat_party(&hero, &enemy, 6, 50, None, hs);
        let foe_wins = simulate_combat_party(&hero, &enemy, 6, 50, None, fs);
        assert_eq!(hero_wins.outcome, CombatOutcome::HeroWon, "{hero_wins:?}");
        assert_eq!(foe_wins.outcome, CombatOutcome::EnemyWon, "{foe_wins:?}");
    }

    #[test]
    fn party_foe_hit_is_followed_by_threat_snapshot_when_both_alive() {
        let lead = HeroProfile::default();
        let mut partner = HeroProfile::default();
        partner.base_stats.attack_speed = 0.05;
        let enemy = Enemy {
            name: "Rival".into(),
            max_health: 9999,
            damage: 4,
            armor: 0,
            attack_speed: 1.0,
            cast_ticks: 0,
            cooldown_ticks: 0,
        };
        let r = simulate_combat_party(
            &lead,
            &enemy,
            6,
            lead.derived_stats().max_health,
            Some((&partner, partner.derived_stats().max_health)),
            0,
        );
        let idx = r
            .events
            .iter()
            .position(|e| matches!(e, CombatEvent::EnemyAttacked { .. }))
            .expect("expected a foe swing");
        assert!(
            matches!(
                r.events.get(idx + 1),
                Some(CombatEvent::ThreatSnapshot { .. })
            ),
            "expected threat telemetry after foe swing, got {:?}",
            r.events.get(idx + 1)
        );
    }

    #[test]
    fn hero_defeats_weaker_enemy() {
        let mut hero = HeroProfile::new(Stats {
            damage: 12,
            ..Stats::default()
        });
        hero.unlock_skill_slots(1);

        let enemy = Enemy {
            name: "Training Hollow".into(),
            max_health: 20,
            damage: 1,
            armor: 0,
            attack_speed: 1.0,
            cast_ticks: 0,
            cooldown_ticks: 0,
        };

        let result = simulate_combat(&hero, &enemy, 100, hero.derived_stats().max_health);

        assert_eq!(result.outcome, CombatOutcome::HeroWon);
        assert!(result
            .events
            .iter()
            .any(|event| matches!(event, CombatEvent::EnemyDefeated { .. })));
    }

    #[test]
    fn lifesteal_skill_restores_health_on_attack() {
        let mut hero = HeroProfile::default();
        hero.unlock_skill_slots(1);
        hero.equip_skill(0, SkillId::LifestealStrike).unwrap();

        let enemy = Enemy {
            name: "Durable Hollow".into(),
            max_health: 60,
            damage: 10,
            armor: 0,
            attack_speed: 1.0,
            cast_ticks: 0,
            cooldown_ticks: 0,
        };

        let result = simulate_combat(&hero, &enemy, 5, hero.derived_stats().max_health);

        assert!(result
            .events
            .iter()
            .any(|event| matches!(event, CombatEvent::HeroHealed { amount, .. } if *amount > 0)));
    }

    #[test]
    fn armor_reduces_incoming_damage_compared_to_unarmored_hero() {
        let unarmored_hero = HeroProfile::default();
        let armored_hero = HeroProfile::new(Stats {
            armor: 3,
            ..Stats::default()
        });
        let enemy = Enemy {
            name: "Rustblade Hollow".into(),
            max_health: 999,
            damage: 5,
            armor: 0,
            attack_speed: 1.0,
            cast_ticks: 0,
            cooldown_ticks: 0,
        };

        let unarmored_result = simulate_combat(
            &unarmored_hero,
            &enemy,
            1,
            unarmored_hero.derived_stats().max_health,
        );
        let armored_result = simulate_combat(
            &armored_hero,
            &enemy,
            1,
            armored_hero.derived_stats().max_health,
        );
        let unarmored_damage_taken =
            unarmored_hero.derived_stats().max_health - unarmored_result.hero_health;
        let armored_damage_taken =
            armored_hero.derived_stats().max_health - armored_result.hero_health;

        assert!(
            armored_damage_taken < unarmored_damage_taken,
            "expected armor to reduce incoming damage below {unarmored_damage_taken}, got {armored_damage_taken}"
        );
    }

    #[test]
    fn incoming_damage_is_at_least_one_when_armor_exceeds_enemy_damage() {
        let hero = HeroProfile::new(Stats {
            armor: 99,
            ..Stats::default()
        });
        let enemy = Enemy {
            name: "Weak Hollow".into(),
            max_health: 999,
            damage: 5,
            armor: 0,
            attack_speed: 1.0,
            cast_ticks: 0,
            cooldown_ticks: 0,
        };

        let result = simulate_combat(&hero, &enemy, 1, hero.derived_stats().max_health);

        let damage_taken = hero.derived_stats().max_health - result.hero_health;
        assert!(
            damage_taken >= 1,
            "expected at least 1 damage when enemy damage is floored to 1"
        );
    }

    #[test]
    fn guard_reduces_incoming_damage_beyond_armor() {
        let mut plain = HeroProfile::default();
        plain.unlock_skill_slots(1);
        let mut guarded = HeroProfile::default();
        guarded.unlock_skill_slots(1);
        guarded.equip_skill(0, SkillId::Guard).unwrap();

        let enemy = Enemy {
            name: "Spiker".into(),
            max_health: 999,
            damage: 8,
            armor: 0,
            attack_speed: 1.0,
            cast_ticks: 0,
            cooldown_ticks: 0,
        };

        let a = simulate_combat(&plain, &enemy, 1, 100);
        let b = simulate_combat(&guarded, &enemy, 1, 100);
        assert!(
            b.hero_health > a.hero_health,
            "guard should leave more hero hp after one exchange"
        );
    }

    #[test]
    fn guard_reduction_scales_with_healing_power() {
        let enemy = Enemy {
            name: "Baseline hit".into(),
            max_health: 999,
            damage: 24,
            armor: 0,
            attack_speed: 1.0,
            cast_ticks: 0,
            cooldown_ticks: 0,
        };

        let mut low = HeroProfile::new(Stats {
            healing_power: 0,
            ..Stats::default()
        });
        low.unlock_skill_slots(1);
        low.equip_skill(0, SkillId::Guard).unwrap();

        let mut high = HeroProfile::new(Stats {
            healing_power: 10,
            ..Stats::default()
        });
        high.unlock_skill_slots(1);
        high.equip_skill(0, SkillId::Guard).unwrap();

        let r_low = simulate_combat(&low, &enemy, 1, 100);
        let r_high = simulate_combat(&high, &enemy, 1, 100);
        assert!(
            r_high.hero_health > r_low.hero_health,
            "more healing_power should strengthen Guard flat reduction"
        );
    }

    #[test]
    fn heavy_strike_increases_weapon_damage() {
        let mut plain = HeroProfile::default();
        plain.unlock_skill_slots(1);
        let mut heavy = HeroProfile::default();
        heavy.unlock_skill_slots(1);
        heavy.equip_skill(0, SkillId::HeavyStrike).unwrap();

        let enemy = Enemy {
            name: "Dummy".into(),
            max_health: 999,
            damage: 0,
            armor: 0,
            attack_speed: 1.0,
            cast_ticks: 0,
            cooldown_ticks: 0,
        };

        let a = simulate_combat(&plain, &enemy, 20, 100);
        let b = simulate_combat(&heavy, &enemy, 20, 100);
        let plain_fist = a.events.iter().find_map(|e| {
            if let CombatEvent::HeroAttacked { strike, .. } = e {
                Some(strike.total())
            } else {
                None
            }
        });
        let heavy_fist = b.events.iter().find_map(|e| {
            if let CombatEvent::HeroAttacked { strike, .. } = e {
                Some(strike.total())
            } else {
                None
            }
        });
        assert_eq!(plain_fist, Some(10));
        assert_eq!(heavy_fist, Some(15));
    }

    #[test]
    fn cleave_matches_heavy_strike_damage_bonus() {
        let mut heavy = HeroProfile::default();
        heavy.unlock_skill_slots(1);
        heavy.equip_skill(0, SkillId::HeavyStrike).unwrap();
        let mut cleave = HeroProfile::default();
        cleave.unlock_skill_slots(1);
        cleave.equip_skill(0, SkillId::Cleave).unwrap();

        let enemy = Enemy {
            name: "Dummy".into(),
            max_health: 999,
            damage: 0,
            armor: 0,
            attack_speed: 1.0,
            cast_ticks: 0,
            cooldown_ticks: 0,
        };

        let h = simulate_combat(&heavy, &enemy, 20, 100);
        let c = simulate_combat(&cleave, &enemy, 20, 100);
        let heavy_dmg = h.events.iter().find_map(|e| match e {
            CombatEvent::HeroAttacked { strike, .. } => Some(strike.total()),
            _ => None,
        });
        let cleave_dmg = c.events.iter().find_map(|e| match e {
            CombatEvent::HeroAttacked { strike, .. } => Some(strike.total()),
            _ => None,
        });
        assert_eq!(heavy_dmg, cleave_dmg);
    }

    #[test]
    fn cleave_splashes_second_foe_while_both_alive_dual_pack() {
        let mut hero = HeroProfile::default();
        hero.unlock_skill_slots(1);
        hero.equip_skill(0, SkillId::Cleave).unwrap();
        let foes = [
            Enemy {
                name: "A".into(),
                max_health: 9999,
                damage: 0,
                armor: 0,
                attack_speed: 0.01,
                cast_ticks: 0,
                cooldown_ticks: 0,
            },
            Enemy {
                name: "B".into(),
                max_health: 9999,
                damage: 0,
                armor: 0,
                attack_speed: 0.01,
                cast_ticks: 0,
                cooldown_ticks: 0,
            },
        ];
        let r = simulate_party_vs_encounter_foes(
            &hero,
            &foes,
            80,
            100,
            None,
            1,
            &[],
            CombatSimOptions::default(),
        );
        let splashed = r.events.iter().find_map(|e| {
            if let CombatEvent::HeroAttacked {
                foe_primary,
                strike,
                cleave_strikes,
                ..
            } = e
            {
                if cleave_strikes.is_empty() {
                    None
                } else {
                    let sp: i32 = cleave_strikes.iter().map(|(_, c)| c.total()).sum();
                    Some((*foe_primary, strike.total(), sp, cleave_strikes.len()))
                }
            } else {
                None
            }
        });
        let Some((pri, main, sp, n_splash)) = splashed else {
            panic!(
                "expected cleave splash event while both foes live, events={:?}",
                r.events
            );
        };
        assert_eq!(pri, 0);
        assert_eq!(n_splash, 1, "two foes => one cleave off-target");
        assert!(main > 0 && sp > 0, "main={main} splash={sp}");
    }

    #[test]
    fn pack_timing_pulse_carries_alt_foe_meters_when_two_foes_alive() {
        let mut hero = HeroProfile::default();
        hero.unlock_skill_slots(1);
        hero.equip_skill(0, SkillId::Cleave).unwrap();
        let foes = [
            Enemy {
                name: "A".into(),
                max_health: 9999,
                damage: 0,
                armor: 0,
                attack_speed: 0.01,
                cast_ticks: 6,
                cooldown_ticks: 6,
            },
            Enemy {
                name: "B".into(),
                max_health: 9999,
                damage: 0,
                armor: 0,
                attack_speed: 0.01,
                cast_ticks: 6,
                cooldown_ticks: 6,
            },
        ];
        let r = simulate_party_vs_encounter_foes(
            &hero,
            &foes,
            2000,
            100,
            None,
            3,
            &[],
            CombatSimOptions::default(),
        );
        // Pack combat drains foe cast/CD to quiescence inside each clock tick before emitting
        // TimingPulse, so foe_alt fills are usually 0 alongside primary foe bars. Instead,
        // assert parity between primary (focus-fire) and off-target meters when both foes share
        // identical timing parameters (always synchronized when mid-tick state were visible).
        let pulses: Vec<(f32, f32, f32, f32)> = r
            .events
            .iter()
            .filter_map(|e| {
                if let CombatEvent::TimingPulse {
                    foe_cast,
                    foe_cd,
                    foe_alt_cast,
                    foe_alt_cd,
                    ..
                } = e
                {
                    Some((*foe_cast, *foe_cd, *foe_alt_cast, *foe_alt_cd))
                } else {
                    None
                }
            })
            .collect();
        assert!(
            !pulses.is_empty(),
            "expected TimingPulse stream in pack combat, events_len={}",
            r.events.len()
        );
        for (fc, cd, fa, ca) in &pulses {
            assert!(
                (fc - fa).abs() < 1e-4 && (cd - ca).abs() < 1e-4,
                "pack pulse: primary ({fc},{cd}) should match off-target ({fa},{ca}) for mirrored foes"
            );
        }
    }

    #[test]
    fn cleave_splashes_two_off_targets_with_three_foe_pack() {
        let mut hero = HeroProfile::default();
        hero.unlock_skill_slots(1);
        hero.equip_skill(0, SkillId::Cleave).unwrap();
        let dummy = Enemy {
            name: "D".into(),
            max_health: 9999,
            damage: 0,
            armor: 0,
            attack_speed: 0.01,
            cast_ticks: 0,
            cooldown_ticks: 0,
        };
        let foes = [dummy.clone(), dummy.clone(), dummy.clone()];
        let r = simulate_party_vs_encounter_foes(
            &hero,
            &foes,
            120,
            100,
            None,
            42,
            &[],
            CombatSimOptions::default(),
        );
        let hit = r.events.iter().find_map(|e| {
            if let CombatEvent::HeroAttacked { cleave_strikes, .. } = e {
                if cleave_strikes.len() >= 2 {
                    Some(cleave_strikes.len())
                } else {
                    None
                }
            } else {
                None
            }
        });
        assert_eq!(
            hit,
            Some(2),
            "three living foes => two cleave splashes, events={:?}",
            r.events
        );
        assert_eq!(r.foe_healths.len(), 3);
    }

    #[test]
    fn victory_rush_hits_are_yellow_and_icd_limits_frequency() {
        let mut hero = HeroProfile::default();
        hero.unlock_skill_slots(1);
        hero.equip_skill(0, SkillId::VictoryRush).unwrap();
        let enemy = Enemy {
            name: "Dummy".into(),
            max_health: 99999,
            damage: 0,
            armor: 0,
            attack_speed: 0.05,
            cast_ticks: 0,
            cooldown_ticks: 0,
        };
        let r = simulate_combat(&hero, &enemy, 120, 100);
        let vr_hits: Vec<HeroStrikeDamage> = r
            .events
            .iter()
            .filter_map(|e| {
                if let CombatEvent::HeroAttacked { strike, .. } = e {
                    if strike.yellow_source_skill == Some(SkillId::VictoryRush) {
                        Some(*strike)
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .collect();
        assert!(
            vr_hits.len() >= 2,
            "expected ICD to allow more than one Rush in a long window, got {vr_hits:?}"
        );
        assert!(
            vr_hits.iter().all(|s| s.white == 0 && s.yellow > 0),
            "Victory Rush should be ability-only damage: {vr_hits:?}"
        );
        assert!(
            vr_hits.len() <= 5,
            "long ICD should prevent spamming Rush on every GCD, got {} hits",
            vr_hits.len()
        );
        assert!(r.events.iter().any(|e| matches!(
            e,
            CombatEvent::BuffChargeConsumed {
                buff_id: BuffId::VictoryRush,
                ..
            }
        )));
    }

    fn count_victory_rush_hits(r: &CombatResult) -> usize {
        r.events
            .iter()
            .filter(|e| {
                matches!(
                    e,
                    CombatEvent::HeroAttacked { strike, .. }
                        if strike.yellow_source_skill == Some(SkillId::VictoryRush)
                )
            })
            .count()
    }

    #[test]
    fn instant_strike_one_charge_single_rush_in_short_fight() {
        let mut hero = HeroProfile::default();
        hero.unlock_skill_slots(1);
        hero.equip_skill(0, SkillId::VictoryRush).unwrap();
        let enemy = Enemy {
            name: "Dummy".into(),
            max_health: 99999,
            damage: 0,
            armor: 0,
            attack_speed: 0.05,
            cast_ticks: 0,
            cooldown_ticks: 0,
        };
        let r = simulate_combat_party_with_options(
            &hero,
            &enemy,
            15,
            100,
            None,
            0,
            &[],
            CombatSimOptions::default(),
        );
        assert_eq!(
            count_victory_rush_hits(&r),
            1,
            "one charge and long ICD should yield a single Rush in 15 ticks"
        );
    }

    #[test]
    fn instant_strike_two_max_charges_two_rushes_before_icd() {
        let mut hero = HeroProfile::default();
        hero.unlock_skill_slots(1);
        hero.equip_skill(0, SkillId::VictoryRush).unwrap();
        let enemy = Enemy {
            name: "Dummy".into(),
            max_health: 99999,
            damage: 0,
            armor: 0,
            attack_speed: 0.05,
            cast_ticks: 0,
            cooldown_ticks: 0,
        };
        let r = simulate_combat_party_with_options(
            &hero,
            &enemy,
            15,
            100,
            None,
            0,
            &[],
            CombatSimOptions {
                player0_instant_strike_max_charges: Some(2),
                ..Default::default()
            },
        );
        assert_eq!(
            count_victory_rush_hits(&r),
            2,
            "two charges allow a second Rush after GCD without waiting full ICD"
        );
        let vr_consumed: Vec<u32> = r
            .events
            .iter()
            .filter_map(|e| {
                if let CombatEvent::BuffChargeConsumed {
                    buff_id: BuffId::VictoryRush,
                    charges_remaining,
                    ..
                } = e
                {
                    Some(*charges_remaining)
                } else {
                    None
                }
            })
            .collect();
        assert!(
            vr_consumed.iter().any(|&c| c == 1) && vr_consumed.iter().any(|&c| c == 0),
            "expected descending charge telemetry, got {vr_consumed:?}"
        );
    }

    #[test]
    fn instant_strike_multi_charge_recharges_one_charge_per_recharge_interval() {
        let mut hero = HeroProfile::default();
        hero.unlock_skill_slots(1);
        hero.equip_skill(0, SkillId::VictoryRush).unwrap();
        let enemy = Enemy {
            name: "Dummy".into(),
            max_health: 99999,
            damage: 0,
            armor: 0,
            attack_speed: 0.05,
            cast_ticks: 0,
            cooldown_ticks: 0,
        };
        let r = simulate_combat_party_with_options(
            &hero,
            &enemy,
            45,
            100,
            None,
            0,
            &[],
            CombatSimOptions {
                player0_instant_strike_max_charges: Some(2),
                ..Default::default()
            },
        );
        assert!(
            count_victory_rush_hits(&r) >= 3,
            "two spends then one recharge pulse should allow a third Rush (GCD permitting), got {}",
            count_victory_rush_hits(&r)
        );
    }

    #[test]
    fn playback_party_status_row_reflects_buff_applied_and_expired() {
        let mut hero = HeroProfile::default();
        hero.unlock_skill_slots(1);
        hero.equip_skill(0, SkillId::EmpoweredBlow).unwrap();
        let enemy = Enemy {
            name: "Dummy".into(),
            max_health: 999,
            damage: 0,
            armor: 0,
            attack_speed: 1.0,
            cast_ticks: 0,
            cooldown_ticks: 0,
        };
        let max_h = hero.derived_stats().max_health;
        let result = simulate_combat(&hero, &enemy, 12, max_h);
        let frames = combat_playback_frames_from_result(
            &hero,
            None,
            &result,
            &enemy.name,
            max_h,
            enemy.max_health,
            max_h,
            None,
            None,
            0,
            0,
            0,
            0,
        );
        let has_empower_chip = frames
            .iter()
            .any(|f| f.hero_debuff_slots.iter().any(|s| s.contains("Empowered")));
        assert!(
            has_empower_chip,
            "expected Empowered Blow chip in hero status row: {:?}",
            frames
                .iter()
                .map(|f| &f.hero_debuff_slots)
                .collect::<Vec<_>>()
        );
    }

    #[test]
    fn empowered_blow_buffs_next_white_swing_with_yellow() {
        let mut hero = HeroProfile::default();
        hero.unlock_skill_slots(1);
        hero.equip_skill(0, SkillId::EmpoweredBlow).unwrap();
        let enemy = Enemy {
            name: "Dummy".into(),
            max_health: 999,
            damage: 0,
            armor: 0,
            attack_speed: 1.0,
            cast_ticks: 0,
            cooldown_ticks: 0,
        };
        let r = simulate_combat(&hero, &enemy, 20, 100);
        let empowered_swing = r.events.iter().find_map(|e| {
            if let CombatEvent::HeroAttacked { strike, .. } = e {
                if strike.white > 0
                    && strike.yellow > 0
                    && strike.yellow_source_skill == Some(SkillId::EmpoweredBlow)
                {
                    return Some(*strike);
                }
            }
            None
        });
        assert!(
            empowered_swing.is_some(),
            "expected Empowered Blow on a white+yellow melee hit"
        );
        assert!(
            r.events.iter().any(|e| matches!(
                e,
                CombatEvent::BuffApplied {
                    buff_id: BuffId::EmpoweredBlow,
                    ..
                }
            )),
            "Empowered Blow queue should emit BuffApplied"
        );
        assert!(
            r.events.iter().any(|e| matches!(
                e,
                CombatEvent::BuffExpired {
                    buff_id: BuffId::EmpoweredBlow,
                    ..
                }
            )),
            "consuming the queue should emit BuffExpired"
        );
    }

    #[test]
    fn phase3_initial_buff_applies_and_expires() {
        use crate::domain::buff::{BuffApplication, BuffId};

        let hero = HeroProfile::default();
        let enemy = Enemy {
            name: "Dummy".into(),
            max_health: 999,
            damage: 0,
            armor: 0,
            attack_speed: 1.0,
            cast_ticks: 0,
            cooldown_ticks: 0,
        };
        let initial = [(
            0_u8,
            BuffApplication {
                buff_id: BuffId::InnerStrength,
                stacks: 1,
                duration_ticks: Some(2),
                charges: None,
            },
        )];
        let r = simulate_combat_party_with_initial_buffs(&hero, &enemy, 8, 100, None, 0, &initial);
        assert!(r.events.iter().any(|e| matches!(
            e,
            CombatEvent::BuffApplied { buff_id, .. } if *buff_id == BuffId::InnerStrength
        )));
        assert!(r.events.iter().any(|e| matches!(
            e,
            CombatEvent::BuffExpired { buff_id, .. } if *buff_id == BuffId::InnerStrength
        )));
    }

    #[test]
    fn playback_opening_frame_shows_encounter_seeded_buffs() {
        use crate::domain::buff::{BuffApplication, BuffId};

        let hero = HeroProfile::default();
        let enemy = Enemy {
            name: "Dummy".into(),
            max_health: 999,
            damage: 0,
            armor: 0,
            attack_speed: 1.0,
            cast_ticks: 0,
            cooldown_ticks: 0,
        };
        let initial = [(
            0_u8,
            BuffApplication {
                buff_id: BuffId::InnerStrength,
                stacks: 1,
                duration_ticks: Some(20),
                charges: None,
            },
        )];
        let r = simulate_combat_party_with_initial_buffs(&hero, &enemy, 6, 100, None, 0, &initial);
        let max_h = hero.derived_stats().max_health;
        let frames = combat_playback_frames_from_result(
            &hero,
            None,
            &r,
            &enemy.name,
            max_h,
            enemy.max_health,
            max_h,
            None,
            None,
            0,
            0,
            0,
            0,
        );
        assert!(
            frames[0]
                .hero_debuff_slots
                .iter()
                .any(|s| s.contains("Inner Strength")),
            "opening frame should show encounter-seeded buff chips: {:?}",
            frames[0].hero_debuff_slots
        );
    }

    #[test]
    fn timing_pulse_includes_nonempty_ability_gcd_after_empower_queue() {
        let mut hero = HeroProfile::default();
        hero.unlock_skill_slots(1);
        hero.equip_skill(0, SkillId::EmpoweredBlow).unwrap();
        let enemy = Enemy {
            name: "Dummy".into(),
            max_health: 999,
            damage: 0,
            armor: 0,
            attack_speed: 1.0,
            cast_ticks: 0,
            cooldown_ticks: 0,
        };
        let max_h = hero.derived_stats().max_health;
        let r = simulate_combat(&hero, &enemy, 8, max_h);
        let pulse_skill_any = r.events.iter().any(|e| {
            matches!(
                e,
                CombatEvent::TimingPulse { player0_skill_gcd, .. } if *player0_skill_gcd > 0.01
            )
        });
        assert!(
            pulse_skill_any,
            "expected ability GCD bar mid-lockout after empower queue (same semantics as weapon CD: 0 fill when a fresh GCD begins)",
        );
    }

    #[test]
    fn phase3_initial_buffs_merge_stacks_same_id() {
        use crate::domain::buff::{BuffApplication, BuffId};

        let hero = HeroProfile::default();
        let enemy = Enemy {
            name: "Dummy".into(),
            max_health: 999,
            damage: 0,
            armor: 0,
            attack_speed: 1.0,
            cast_ticks: 0,
            cooldown_ticks: 0,
        };
        let initial = [
            (
                0_u8,
                BuffApplication {
                    buff_id: BuffId::InnerStrength,
                    stacks: 1,
                    duration_ticks: Some(10),
                    charges: None,
                },
            ),
            (
                0_u8,
                BuffApplication {
                    buff_id: BuffId::InnerStrength,
                    stacks: 2,
                    duration_ticks: Some(10),
                    charges: None,
                },
            ),
        ];
        let r = simulate_combat_party_with_initial_buffs(&hero, &enemy, 2, 100, None, 0, &initial);
        let applied: Vec<u32> = r
            .events
            .iter()
            .filter_map(|e| {
                if let CombatEvent::BuffApplied { stacks, .. } = e {
                    Some(*stacks)
                } else {
                    None
                }
            })
            .collect();
        assert_eq!(
            applied,
            vec![1, 3],
            "second grant should merge to combined stack height"
        );
    }

    #[test]
    fn heavy_strike_slows_attack_pacing() {
        let mut nimble = HeroProfile::default();
        nimble.base_stats.attack_speed = 2.0;
        nimble.unlock_skill_slots(1);

        let mut heavy = HeroProfile::default();
        heavy.base_stats.attack_speed = 2.0;
        heavy.unlock_skill_slots(1);
        heavy.equip_skill(0, SkillId::HeavyStrike).unwrap();

        let enemy = Enemy {
            name: "Dummy".into(),
            max_health: 999,
            damage: 0,
            armor: 0,
            attack_speed: 1.0,
            cast_ticks: 0,
            cooldown_ticks: 0,
        };

        let r_nimble = simulate_combat(&nimble, &enemy, 12, 100);
        let r_heavy = simulate_combat(&heavy, &enemy, 12, 100);

        let swings_nimble = r_nimble
            .events
            .iter()
            .filter(|e| matches!(e, CombatEvent::HeroAttacked { .. }))
            .count();
        let swings_heavy = r_heavy
            .events
            .iter()
            .filter(|e| matches!(e, CombatEvent::HeroAttacked { .. }))
            .count();

        assert!(
            swings_nimble > swings_heavy,
            "heavy kit should swing less often per window"
        );
    }

    #[test]
    fn rhythm_affix_reduces_heavy_weave_recovery_by_one_tick() {
        let mut base = HeroProfile::default();
        base.unlock_skill_slots(1);
        base.equip_skill(0, SkillId::HeavyStrike).unwrap();
        let cd_base = prepare_hero_combat(&base).test_attack_cd_total();
        assert!(cd_base > 1, "heavy should have post-swing recovery ticks");

        let mut rhythm_hero = HeroProfile::default();
        rhythm_hero.unlock_skill_slots(1);
        rhythm_hero.equip_skill(0, SkillId::HeavyStrike).unwrap();
        let mut grips = ItemInstance::basic(77, "Snap grips", GearSlot::Hands);
        grips.affixes.push(ItemAffix::Rhythm);
        rhythm_hero.equip_item(grips);
        let cd_rhythm = prepare_hero_combat(&rhythm_hero).test_attack_cd_total();

        assert_eq!(cd_base.saturating_sub(cd_rhythm), 1);
    }

    #[test]
    fn poison_edge_deals_extra_damage() {
        let mut hero = HeroProfile::default();
        hero.unlock_skill_slots(1);
        hero.equip_skill(0, SkillId::PoisonEdge).unwrap();

        let enemy = Enemy {
            name: "Dummy".into(),
            max_health: 999,
            damage: 0,
            armor: 0,
            attack_speed: 1.0,
            cast_ticks: 0,
            cooldown_ticks: 0,
        };

        let result = simulate_combat(&hero, &enemy, 1, 100);
        assert!(result.events.iter().any(|e| matches!(
            e,
            CombatEvent::PoisonTick { damage, .. } if *damage > 0
        )));
        assert!(result.events.iter().any(|e| matches!(
            e,
            CombatEvent::BuffTick {
                buff_id: BuffId::PoisonVenom,
                ..
            }
        )));
    }

    #[test]
    fn poison_deals_damage_across_clock_ticks() {
        let mut hero = HeroProfile::default();
        hero.base_stats.attack_speed = 0.12;
        hero.unlock_skill_slots(1);
        hero.equip_skill(0, SkillId::PoisonEdge).unwrap();

        let enemy = Enemy {
            name: "Pacer".into(),
            max_health: 999,
            damage: 1,
            armor: 0,
            attack_speed: 1.0,
            cast_ticks: 0,
            cooldown_ticks: 0,
        };

        let result = simulate_combat(&hero, &enemy, 12, hero.derived_stats().max_health);
        let hero_swings = result
            .events
            .iter()
            .filter(|e| matches!(e, CombatEvent::HeroAttacked { .. }))
            .count();
        let poison_ticks = result
            .events
            .iter()
            .filter(|e| matches!(e, CombatEvent::PoisonTick { .. }))
            .count();

        assert_eq!(
            hero_swings, 1,
            "slow hero should land one strike in this tick budget"
        );
        assert!(
            poison_ticks >= 2,
            "poison should tick on later clock iterations without a second hero swing, got {poison_ticks} PoisonTick events"
        );
    }

    #[test]
    fn poison_tick_batch_runs_immediately_after_timing_pulse_same_tick() {
        let mut hero = HeroProfile::default();
        hero.base_stats.attack_speed = 0.12;
        hero.unlock_skill_slots(1);
        hero.equip_skill(0, SkillId::PoisonEdge).unwrap();

        let enemy = Enemy {
            name: "Pacer".into(),
            max_health: 999,
            damage: 1,
            armor: 0,
            attack_speed: 1.0,
            cast_ticks: 0,
            cooldown_ticks: 0,
        };

        let result = simulate_combat(&hero, &enemy, 12, hero.derived_stats().max_health);
        let i = result
            .events
            .iter()
            .position(|e| matches!(e, CombatEvent::PoisonTick { .. }))
            .expect("expected at least one PoisonTick");
        assert!(
            matches!(
                result.events.get(i.wrapping_sub(1)),
                Some(CombatEvent::TimingPulse { .. })
            ),
            "expected PoisonTick immediately after TimingPulse, got {:?} then {:?}",
            result.events.get(i.wrapping_sub(1)),
            result.events.get(i)
        );
    }

    #[test]
    fn poison_tick_damage_increases_with_stack_potency() {
        let mut hero = HeroProfile::default();
        hero.base_stats.attack_speed = 0.12;
        hero.unlock_skill_slots(1);
        hero.equip_skill(0, SkillId::PoisonEdge).unwrap();

        let enemy = Enemy {
            name: "Pacer".into(),
            max_health: 999,
            damage: 1,
            armor: 0,
            attack_speed: 1.0,
            cast_ticks: 0,
            cooldown_ticks: 0,
        };

        let result = simulate_combat(&hero, &enemy, 12, hero.derived_stats().max_health);
        let damages: Vec<i32> = result
            .events
            .iter()
            .filter_map(|e| {
                if let CombatEvent::PoisonTick { damage, .. } = e {
                    Some(*damage)
                } else {
                    None
                }
            })
            .collect();

        assert!(
            damages.len() >= 2,
            "expected at least two poison ticks, got {:?}",
            damages
        );
        assert!(
            damages[0] > damages[1],
            "first tick at higher stack count should deal more: {:?}",
            damages
        );
    }

    #[test]
    fn thorns_reflect_hurts_enemy_on_hit() {
        let mut hero = HeroProfile::default();
        hero.unlock_skill_slots(1);
        hero.equip_skill(0, SkillId::ThornSkin).unwrap();

        let enemy = Enemy {
            name: "Bruiser".into(),
            max_health: 999,
            damage: 9,
            armor: 0,
            attack_speed: 1.0,
            cast_ticks: 0,
            cooldown_ticks: 0,
        };

        let result = simulate_combat(&hero, &enemy, 1, 100);
        assert!(result.events.iter().any(|e| matches!(
            e,
            CombatEvent::ThornsReflect { damage, .. } if *damage > 0
        )));
    }

    #[test]
    fn barrier_pulse_absorbs_part_of_first_hit() {
        let mut hero = HeroProfile::default();
        hero.unlock_skill_slots(1);
        hero.equip_skill(0, SkillId::BarrierPulse).unwrap();

        let enemy = Enemy {
            name: "Bruiser".into(),
            max_health: 999,
            damage: 12,
            armor: 0,
            attack_speed: 1.0,
            cast_ticks: 0,
            cooldown_ticks: 0,
        };

        let with_barrier = simulate_combat(&hero, &enemy, 1, 100);
        let mut plain = HeroProfile::default();
        plain.unlock_skill_slots(1);
        let no_barrier = simulate_combat(&plain, &enemy, 1, 100);

        assert!(
            with_barrier.hero_health > no_barrier.hero_health,
            "barrier should reduce hp lost to the first strike"
        );
    }

    #[test]
    fn barrier_pulse_fully_absorbs_hit_that_would_empty_hp() {
        let mut hero = HeroProfile::new(Stats {
            max_health: 100,
            healing_power: 20,
            ..Stats::default()
        });
        hero.unlock_skill_slots(1);
        hero.equip_skill(0, SkillId::BarrierPulse).unwrap();

        let enemy = Enemy {
            name: "Alpha".into(),
            max_health: 99,
            damage: 40,
            armor: 0,
            attack_speed: 1.0,
            cast_ticks: 0,
            cooldown_ticks: 0,
        };

        let start = 12;
        let r = simulate_combat(&hero, &enemy, 1, start);
        assert_eq!(
            r.hero_health, start,
            "barrier should eat the full strike so hero hp is unchanged"
        );
        assert_eq!(
            r.outcome,
            CombatOutcome::TimedOut,
            "enemy still alive after one swing each side in this setup"
        );
    }

    #[test]
    fn barrier_absorb_all_skips_thorns_when_no_hp_loss() {
        let mut hero = HeroProfile::new(Stats {
            max_health: 100,
            healing_power: 20,
            ..Stats::default()
        });
        hero.unlock_skill_slots(2);
        hero.equip_skill(0, SkillId::BarrierPulse).unwrap();
        hero.equip_skill(1, SkillId::ThornSkin).unwrap();

        let enemy = Enemy {
            name: "Chip".into(),
            max_health: 999,
            damage: 10,
            armor: 0,
            attack_speed: 1.0,
            cast_ticks: 0,
            cooldown_ticks: 0,
        };

        let r = simulate_combat(&hero, &enemy, 1, 100);
        assert!(
            !r.events
                .iter()
                .any(|e| matches!(e, CombatEvent::ThornsReflect { .. })),
            "thorns should not proc when barrier absorbs all incoming damage (hp_loss == 0)"
        );
    }

    #[test]
    fn thorns_reflect_defeats_enemy_after_enemy_attack() {
        let mut hero = HeroProfile::default();
        hero.base_stats.attack_speed = 0.12;
        hero.unlock_skill_slots(1);
        hero.equip_skill(0, SkillId::ThornSkin).unwrap();

        let enemy = Enemy {
            name: "Glass".into(),
            max_health: 10,
            damage: 30,
            armor: 0,
            attack_speed: 1.0,
            cast_ticks: 0,
            cooldown_ticks: 0,
        };

        let r = simulate_combat(&hero, &enemy, 5, 100);
        assert_eq!(r.outcome, CombatOutcome::HeroWon);
        assert!(
            r.events.iter().any(|e| matches!(
                e,
                CombatEvent::ThornsReflect { damage, .. } if *damage >= 10
            )),
            "reflect should kill the low-HP enemy before the hero swings this fight"
        );
        assert!(
            r.events
                .iter()
                .any(|e| matches!(e, CombatEvent::EnemyDefeated { .. })),
            "defeat should be attributed after thorns damage"
        );
    }

    #[test]
    fn spiked_affix_thorns_reflect_kills_low_hp_enemy() {
        let mut hero = HeroProfile::default();
        hero.base_stats.attack_speed = 0.12;
        hero.unlock_skill_slots(1);
        hero.equip_skill(0, SkillId::ThornSkin).unwrap();
        let mut mail = ItemInstance::basic(9, "Spiked mail", GearSlot::Chest);
        mail.affixes.push(ItemAffix::Spiked);
        hero.equip_item(mail);

        let enemy = Enemy {
            name: "Splinter target".into(),
            max_health: 15,
            damage: 30,
            armor: 0,
            attack_speed: 1.0,
            cast_ticks: 0,
            cooldown_ticks: 0,
        };

        let r = simulate_combat(&hero, &enemy, 15, 100);
        assert_eq!(r.outcome, CombatOutcome::HeroWon);
        assert!(
            r.events
                .iter()
                .any(|e| matches!(e, CombatEvent::ThornsReflect { .. })),
            "spiked should still route through hp_loss -> thorns (after barrier 0 here)"
        );
    }

    #[test]
    fn high_attack_speed_hero_strikes_more_per_clock_tick() {
        let mut fast = HeroProfile::default();
        fast.base_stats.attack_speed = 2.0;
        let enemy = Enemy {
            name: "Pacing dummy".into(),
            max_health: 999,
            damage: 1,
            armor: 0,
            attack_speed: 1.0,
            cast_ticks: 0,
            cooldown_ticks: 0,
        };

        let r = simulate_combat(&fast, &enemy, 1, 100);
        let hero_swings = r
            .events
            .iter()
            .filter(|e| matches!(e, CombatEvent::HeroAttacked { .. }))
            .count();
        let foe_swings = r
            .events
            .iter()
            .filter(|e| matches!(e, CombatEvent::EnemyAttacked { .. }))
            .count();
        assert_eq!(hero_swings, 2);
        assert_eq!(foe_swings, 1);
    }

    #[test]
    fn poison_damage_scales_with_healing_power() {
        let mut tuned = HeroProfile::default();
        tuned.base_stats.healing_power = 6;
        tuned.unlock_skill_slots(1);
        tuned.equip_skill(0, SkillId::PoisonEdge).unwrap();
        let weak = simulate_combat(&tuned, &dummy_enemy(), 1, 100);
        let weak_poison = weak.events.iter().find_map(|e| {
            if let CombatEvent::PoisonTick { damage, .. } = e {
                Some(*damage)
            } else {
                None
            }
        });

        let mut plain = HeroProfile::default();
        plain.unlock_skill_slots(1);
        plain.equip_skill(0, SkillId::PoisonEdge).unwrap();
        let base = simulate_combat(&plain, &dummy_enemy(), 1, 100);
        let base_poison = base.events.iter().find_map(|e| {
            if let CombatEvent::PoisonTick { damage, .. } = e {
                Some(*damage)
            } else {
                None
            }
        });

        assert!(weak_poison.unwrap() > base_poison.unwrap());
    }

    #[test]
    fn heavy_affix_synergy_boosts_heavy_strike() {
        let mut skill_only = HeroProfile::default();
        skill_only.unlock_skill_slots(1);
        skill_only.equip_skill(0, SkillId::HeavyStrike).unwrap();
        let mut club = ItemInstance::basic(1, "Club", GearSlot::MainHand);
        club.stats.damage = 4;
        skill_only.equip_item(club);

        let mut with_affix = HeroProfile::default();
        with_affix.unlock_skill_slots(1);
        with_affix.equip_skill(0, SkillId::HeavyStrike).unwrap();
        let mut maul = ItemInstance::basic(2, "Maul", GearSlot::MainHand);
        maul.stats.damage = 0;
        maul.stats.attack_speed = 0.2;
        maul.affixes.push(ItemAffix::Heavy);
        with_affix.equip_item(maul);

        let e = dummy_enemy();
        let a = first_hero_damage(&simulate_combat(&skill_only, &e, 20, 100));
        let b = first_hero_damage(&simulate_combat(&with_affix, &e, 20, 100));
        assert!(b > a);
        assert_eq!(a, 21);
        assert_eq!(b, 25);
    }

    #[test]
    fn vampiric_affix_improves_lifesteal_heal() {
        let mut plain = HeroProfile::default();
        plain.unlock_skill_slots(1);
        plain.equip_skill(0, SkillId::LifestealStrike).unwrap();

        let mut geared = HeroProfile::default();
        geared.unlock_skill_slots(1);
        geared.equip_skill(0, SkillId::LifestealStrike).unwrap();
        let mut blade = ItemInstance::basic(2, "Fang", GearSlot::MainHand);
        blade.affixes.push(ItemAffix::Vampiric);
        geared.equip_item(blade);

        let e = dummy_enemy();
        let h_plain = first_heal(&simulate_combat(&plain, &e, 1, 100));
        let h_geared = first_heal(&simulate_combat(&geared, &e, 1, 100));
        assert!(h_geared > h_plain);
    }

    #[test]
    fn spiked_affix_synergy_improves_thorns() {
        let mut thorns_only = HeroProfile::default();
        thorns_only.unlock_skill_slots(1);
        thorns_only.equip_skill(0, SkillId::ThornSkin).unwrap();

        let mut both = HeroProfile::default();
        both.unlock_skill_slots(1);
        both.equip_skill(0, SkillId::ThornSkin).unwrap();
        let mut spiky = ItemInstance::basic(3, "Spiky mail", GearSlot::Chest);
        spiky.affixes.push(ItemAffix::Spiked);
        both.equip_item(spiky);

        let enemy = Enemy {
            name: "Bruiser".into(),
            max_health: 999,
            damage: 12,
            armor: 0,
            attack_speed: 1.0,
            cast_ticks: 0,
            cooldown_ticks: 0,
        };

        let r1 = first_thorns(&simulate_combat(&thorns_only, &enemy, 1, 100));
        let r2 = first_thorns(&simulate_combat(&both, &enemy, 1, 100));
        assert!(r2 > r1);
        assert_eq!(r1, 4);
        assert_eq!(r2, 5);
    }

    #[test]
    fn cursed_affix_weakens_barrier_pulse() {
        let mut pure = HeroProfile::default();
        pure.unlock_skill_slots(1);
        pure.equip_skill(0, SkillId::BarrierPulse).unwrap();

        let mut cursed = HeroProfile::default();
        cursed.unlock_skill_slots(1);
        cursed.equip_skill(0, SkillId::BarrierPulse).unwrap();
        let mut cloth = ItemInstance::basic(4, "Shroud", GearSlot::Trinket1);
        cloth.affixes.push(ItemAffix::Cursed);
        cursed.equip_item(cloth);

        let enemy = Enemy {
            name: "Bruiser".into(),
            max_health: 999,
            damage: 12,
            armor: 0,
            attack_speed: 1.0,
            cast_ticks: 0,
            cooldown_ticks: 0,
        };

        let hp_pure = simulate_combat(&pure, &enemy, 1, 100).hero_health;
        let hp_cursed = simulate_combat(&cursed, &enemy, 1, 100).hero_health;
        assert!(
            hp_pure > hp_cursed,
            "cursed gear should shrink the barrier bundle"
        );
    }

    #[test]
    fn shattering_affix_pierces_enemy_armor() {
        let mut plain = HeroProfile::new(Stats {
            max_health: 100,
            damage: 16,
            armor: 0,
            attack_speed: 1.0,
            healing_power: 0,
        });
        plain.unlock_skill_slots(0);

        let mut pierce = HeroProfile::new(Stats {
            max_health: 100,
            damage: 14,
            armor: 0,
            attack_speed: 1.0,
            healing_power: 0,
        });
        pierce.unlock_skill_slots(0);
        let mut mace = ItemInstance::basic(9, "Ram", GearSlot::MainHand);
        mace.affixes.push(ItemAffix::Shattering);
        pierce.equip_item(mace);

        assert_eq!(plain.derived_stats().damage, pierce.derived_stats().damage);

        let enemy = Enemy {
            name: "Plate".into(),
            max_health: 999,
            damage: 0,
            armor: 8,
            attack_speed: 0.01,
            cast_ticks: 0,
            cooldown_ticks: 0,
        };

        let d_plain = first_hero_damage(&simulate_combat(&plain, &enemy, 2, 100));
        let d_pierce = first_hero_damage(&simulate_combat(&pierce, &enemy, 2, 100));
        assert_eq!(d_plain, 8);
        assert_eq!(d_pierce, 12);
    }

    #[test]
    fn virulent_affix_strengthens_poison_opening_tick() {
        let mut base = HeroProfile::default();
        base.unlock_skill_slots(1);
        base.equip_skill(0, SkillId::PoisonEdge).unwrap();

        let mut v = HeroProfile::default();
        v.unlock_skill_slots(1);
        v.equip_skill(0, SkillId::PoisonEdge).unwrap();
        let mut orb = ItemInstance::basic(10, "Ichor", GearSlot::Trinket1);
        orb.affixes.push(ItemAffix::Virulent);
        v.equip_item(orb);

        let enemy = Enemy {
            name: "Sponge".into(),
            max_health: 999,
            damage: 0,
            armor: 0,
            attack_speed: 0.05,
            cast_ticks: 0,
            cooldown_ticks: 0,
        };

        let r_base = simulate_combat(&base, &enemy, 4, 100);
        let r_v = simulate_combat(&v, &enemy, 4, 100);

        let tick_base = r_base
            .events
            .iter()
            .find_map(|e| {
                if let CombatEvent::PoisonTick { damage, .. } = e {
                    Some(*damage)
                } else {
                    None
                }
            })
            .unwrap();
        let tick_v = r_v
            .events
            .iter()
            .find_map(|e| {
                if let CombatEvent::PoisonTick { damage, .. } = e {
                    Some(*damage)
                } else {
                    None
                }
            })
            .unwrap();
        assert!(tick_v > tick_base);
    }

    #[test]
    fn bastion_affix_boosts_guard_with_skill() {
        let mut plain = HeroProfile::default();
        plain.unlock_skill_slots(1);
        plain.equip_skill(0, SkillId::Guard).unwrap();

        let mut wall = HeroProfile::default();
        wall.unlock_skill_slots(1);
        wall.equip_skill(0, SkillId::Guard).unwrap();
        let mut shield = ItemInstance::basic(11, "Bulwark", GearSlot::OffHand);
        shield.affixes.push(ItemAffix::Bastion);
        wall.equip_item(shield);

        let enemy = Enemy {
            name: "Bruiser".into(),
            max_health: 999,
            damage: 20,
            armor: 0,
            attack_speed: 1.0,
            cast_ticks: 0,
            cooldown_ticks: 0,
        };

        let loss_plain = first_enemy_hit_damage(&simulate_combat(&plain, &enemy, 1, 100));
        let loss_wall = first_enemy_hit_damage(&simulate_combat(&wall, &enemy, 1, 100));
        assert!(loss_wall < loss_plain);
    }

    #[test]
    fn titans_fury_boosts_weapon_damage_when_wounded() {
        let mut plain = HeroProfile::new(Stats {
            max_health: 100,
            damage: 24,
            armor: 0,
            attack_speed: 2.0,
            healing_power: 0,
        });
        plain.unlock_skill_slots(0);

        let mut fury = HeroProfile::new(Stats {
            max_health: 100,
            damage: 21,
            armor: 0,
            attack_speed: 2.0,
            healing_power: 0,
        });
        fury.unlock_skill_slots(0);
        let mut axe = ItemInstance::basic(55, "Titan maul", GearSlot::MainHand);
        axe.affixes.push(ItemAffix::TitansFury);
        fury.equip_item(axe);

        assert_eq!(plain.derived_stats().damage, fury.derived_stats().damage);

        let enemy = Enemy {
            name: "Dummy".into(),
            max_health: 999,
            damage: 0,
            armor: 0,
            attack_speed: 0.01,
            cast_ticks: 0,
            cooldown_ticks: 0,
        };

        let d_plain = first_hero_damage(&simulate_combat(&plain, &enemy, 2, 60));
        let d_fury = first_hero_damage(&simulate_combat(&fury, &enemy, 2, 50));
        assert_eq!(d_plain, 24);
        assert_eq!(d_fury, 30);
    }

    #[test]
    fn devourer_affix_heals_after_enemy_defeat() {
        let mut hero = HeroProfile::new(Stats {
            max_health: 120,
            damage: 80,
            armor: 0,
            attack_speed: 1.0,
            healing_power: 0,
        });
        hero.unlock_skill_slots(0);
        let mut glaive = ItemInstance::basic(56, "Maw", GearSlot::MainHand);
        glaive.affixes.push(ItemAffix::Devourer);
        hero.equip_item(glaive);

        let enemy = Enemy {
            name: "Wisp".into(),
            max_health: 40,
            damage: 0,
            armor: 0,
            attack_speed: 0.01,
            cast_ticks: 0,
            cooldown_ticks: 0,
        };

        let r = simulate_combat(&hero, &enemy, 5, 100);
        assert_eq!(r.outcome, CombatOutcome::HeroWon);

        let mut saw_defeat = false;
        let mut heal_after_defeat = false;
        for e in &r.events {
            if matches!(e, CombatEvent::EnemyDefeated { .. }) {
                saw_defeat = true;
                continue;
            }
            if saw_defeat {
                if let CombatEvent::HeroHealed { amount, .. } = e {
                    assert!(*amount > 0);
                    heal_after_defeat = true;
                    break;
                }
            }
        }
        assert!(heal_after_defeat, "expected Devourer heal after kill");
    }

    fn first_enemy_hit_damage(r: &CombatResult) -> i32 {
        r.events
            .iter()
            .find_map(|e| {
                if let CombatEvent::EnemyAttacked { damage, .. } = e {
                    Some(*damage)
                } else {
                    None
                }
            })
            .unwrap()
    }

    fn dummy_enemy() -> Enemy {
        Enemy {
            name: "Dummy".into(),
            max_health: 999,
            damage: 0,
            armor: 0,
            attack_speed: 1.0,
            ..Default::default()
        }
    }

    fn first_hero_damage(r: &CombatResult) -> i32 {
        r.events
            .iter()
            .find_map(|e| {
                if let CombatEvent::HeroAttacked { strike, .. } = e {
                    Some(strike.total())
                } else {
                    None
                }
            })
            .unwrap()
    }

    fn first_heal(r: &CombatResult) -> i32 {
        r.events
            .iter()
            .find_map(|e| {
                if let CombatEvent::HeroHealed { amount, .. } = e {
                    Some(*amount)
                } else {
                    None
                }
            })
            .unwrap()
    }

    fn first_thorns(r: &CombatResult) -> i32 {
        r.events
            .iter()
            .find_map(|e| {
                if let CombatEvent::ThornsReflect { damage, .. } = e {
                    Some(*damage)
                } else {
                    None
                }
            })
            .unwrap()
    }

    use crate::domain::party::default_party_partner_hero;

    #[test]
    fn party_threat_routes_first_foe_hit_to_tank_when_hero_is_slower() {
        let mut hero = HeroProfile::default();
        hero.base_stats.attack_speed = 0.55;
        let enemy = Enemy {
            name: "Chaser".into(),
            max_health: 999,
            damage: 12,
            armor: 0,
            attack_speed: 1.0,
            cast_ticks: 0,
            cooldown_ticks: 0,
        };
        let partner = default_party_partner_hero();
        let r = simulate_combat_party(
            &hero,
            &enemy,
            3,
            100,
            Some((&partner, partner.derived_stats().max_health)),
            0,
        );
        let first_foe = r
            .events
            .iter()
            .find(|e| matches!(e, CombatEvent::EnemyAttacked { .. }))
            .expect("expected a foe swing");
        assert!(
            matches!(first_foe, CombatEvent::EnemyAttacked { target: 1, .. }),
            "first hit should prefer tank while hero threat is lower: {first_foe:?}"
        );
    }

    #[test]
    fn next_encounter_starts_at_remaining_hp() {
        let hero = HeroProfile::default();
        let enemy = Enemy {
            name: "Poker".into(),
            max_health: 999,
            damage: 3,
            armor: 0,
            attack_speed: 1.0,
            cast_ticks: 0,
            cooldown_ticks: 0,
        };
        let first = simulate_combat(&hero, &enemy, 1, 100);
        assert_eq!(first.hero_health, 97);
        let second = simulate_combat(&hero, &enemy, 1, first.hero_health);
        assert_eq!(second.hero_health, 94);
    }
}
