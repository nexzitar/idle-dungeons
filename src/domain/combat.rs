use crate::domain::buff::{buff_display_name, buff_expires_at_clock, BuffApplication, BuffId};
use crate::domain::dungeon::Enemy;
use crate::domain::hero::HeroProfile;
use crate::domain::items::ItemAffix;
use crate::domain::skills::{
    skill_definition, skill_timings, SkillCombatStyle, SkillId, SkillKind, SkillTrigger,
};
use crate::domain::stats::Stats;

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
    /// `attacker` 0 = lead hero ("you" in UI), 1 = party partner.
    HeroAttacked {
        attacker: u8,
        strike: HeroStrikeDamage,
    },
    /// Poison at end of clock iteration. `stacks` is potency **before** this tick (and before decrement).
    PoisonTick {
        damage: i32,
        stacks: u32,
    },
    /// `target` 0 = lead hero, 1 = party partner ([`HeroProfile`]).
    EnemyAttacked {
        target: u8,
        damage: i32,
    },
    ThornsReflect {
        damage: i32,
    },
    /// `target` 0 = lead, 1 = partner.
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
        lead_cast: f32,
        lead_cd: f32,
        ally_cast: f32,
        ally_cd: f32,
        foe_cast: f32,
        foe_cd: f32,
    },
    EnemyDefeated,
    /// Party member defeated (currently slot `1` = partner). Lead uses [`HeroDefeated`].
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

fn combat_event_caption(event: &CombatEvent, partner_name: Option<&str>) -> String {
    match event {
        CombatEvent::HeroAttacked { attacker, strike } => {
            let total = strike.total();
            if strike.yellow == 0 {
                if *attacker == 0 {
                    format!("You strike for {total} damage.")
                } else if let Some(n) = partner_name {
                    format!("{n} strikes for {total} damage.")
                } else {
                    format!("Partner strikes for {total} damage.")
                }
            } else if strike.white == 0 {
                if *attacker == 0 {
                    format!("You hit for {total} ability damage.")
                } else if let Some(n) = partner_name {
                    format!("{n} hits for {total} ability damage.")
                } else {
                    format!("Partner hits for {total} ability damage.")
                }
            } else if *attacker == 0 {
                format!(
                    "You strike for {} white and {} ability ({} total).",
                    strike.white, strike.yellow, total
                )
            } else if let Some(n) = partner_name {
                format!(
                    "{n} strikes for {} white and {} ability ({} total).",
                    strike.white, strike.yellow, total
                )
            } else {
                format!(
                    "Partner strikes for {} white and {} ability ({} total).",
                    strike.white, strike.yellow, total
                )
            }
        }
        CombatEvent::PoisonTick { damage, stacks } => {
            format!("Poison deals {} damage ({} stacks).", damage, stacks)
        }
        CombatEvent::EnemyAttacked { target, damage } => {
            if *target == 0 {
                format!("The foe hits you for {damage} damage.")
            } else if let Some(n) = partner_name {
                format!("The foe hits {n} for {damage} damage.")
            } else {
                format!("The foe hits your partner for {damage} damage.")
            }
        }
        CombatEvent::ThornsReflect { damage } => {
            format!("Thorns bite back for {} damage.", damage)
        }
        CombatEvent::HeroHealed { target, amount } => {
            if *target == 0 {
                format!("You recover {amount} health.")
            } else if let Some(n) = partner_name {
                format!("{n} recovers {amount} health.")
            } else {
                format!("Partner recovers {amount} health.")
            }
        }
        CombatEvent::ThreatSnapshot { slot0, slot1 } => {
            let p1 = partner_name.unwrap_or("Partner");
            format!("Threat · You {slot0} · {p1} {slot1}")
        }
        CombatEvent::TimingPulse { .. } => String::new(),
        CombatEvent::EnemyDefeated => "Enemy defeated.".to_string(),
        CombatEvent::PartyMemberDown { party_index } => {
            if *party_index == 1 {
                if let Some(n) = partner_name {
                    format!("{n} is down.")
                } else {
                    "A party member is down.".to_string()
                }
            } else {
                "A party member is down.".to_string()
            }
        }
        CombatEvent::HeroDefeated => "You collapse...".to_string(),
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
            if *target == 0 {
                format!("You gain {name} ×{stacks}{dur}.")
            } else if let Some(n) = partner_name {
                format!("{n} gains {name} ×{stacks}{dur}.")
            } else {
                format!("Ally gains {name} ×{stacks}{dur}.")
            }
        }
        CombatEvent::BuffExpired { target, buff_id } => {
            let name = buff_display_name(*buff_id);
            if *target == 0 {
                format!("{name} fades from you.")
            } else if let Some(n) = partner_name {
                format!("{name} fades from {n}.")
            } else {
                format!("{name} fades from your ally.")
            }
        }
        CombatEvent::BuffTick {
            target,
            buff_id,
            stacks,
        } => {
            let name = buff_display_name(*buff_id);
            if *target == 0 {
                format!("{name} ticks on you (×{stacks}).")
            } else if let Some(n) = partner_name {
                format!("{name} ticks on {n} (×{stacks}).")
            } else {
                format!("{name} ticks (×{stacks}).")
            }
        }
        CombatEvent::BuffChargeConsumed {
            target,
            buff_id,
            charges_remaining,
        } => {
            let name = buff_display_name(*buff_id);
            if *target == 0 {
                format!("{name} consumes a charge ({charges_remaining} left).")
            } else if let Some(n) = partner_name {
                format!("{name}: {n} consumes a charge ({charges_remaining} left).")
            } else {
                format!("{name}: ally consumes a charge ({charges_remaining} left).")
            }
        }
    }
}

/// Where floating combat text should appear relative to the theater layout.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CombatSfxAnchor {
    /// Narration / engage / threat telemetry — skip floating spam.
    #[default]
    Neutral,
    Lead,
    Ally,
    Enemy,
}

fn sfx_anchor_for_event(event: &CombatEvent) -> CombatSfxAnchor {
    match event {
        CombatEvent::HeroAttacked { .. } => CombatSfxAnchor::Enemy,
        CombatEvent::PoisonTick { .. } => CombatSfxAnchor::Enemy,
        CombatEvent::EnemyAttacked { target: 0, .. } => CombatSfxAnchor::Lead,
        CombatEvent::EnemyAttacked { target: 1, .. } => CombatSfxAnchor::Ally,
        CombatEvent::EnemyAttacked { .. } => CombatSfxAnchor::Lead,
        CombatEvent::ThornsReflect { .. } => CombatSfxAnchor::Enemy,
        CombatEvent::HeroHealed { target: 0, .. } => CombatSfxAnchor::Lead,
        CombatEvent::HeroHealed { target: 1, .. } => CombatSfxAnchor::Ally,
        CombatEvent::HeroHealed { .. } => CombatSfxAnchor::Lead,
        CombatEvent::ThreatSnapshot { .. } => CombatSfxAnchor::Neutral,
        CombatEvent::TimingPulse { .. } => CombatSfxAnchor::Neutral,
        CombatEvent::EnemyDefeated => CombatSfxAnchor::Enemy,
        CombatEvent::PartyMemberDown { .. } => CombatSfxAnchor::Ally,
        CombatEvent::HeroDefeated => CombatSfxAnchor::Lead,
        CombatEvent::BuffApplied { target: 0, .. } => CombatSfxAnchor::Lead,
        CombatEvent::BuffApplied { .. } => CombatSfxAnchor::Ally,
        CombatEvent::BuffExpired { target: 0, .. } => CombatSfxAnchor::Lead,
        CombatEvent::BuffExpired { .. } => CombatSfxAnchor::Ally,
        CombatEvent::BuffTick { target: 0, .. } => CombatSfxAnchor::Lead,
        CombatEvent::BuffTick { .. } => CombatSfxAnchor::Ally,
        CombatEvent::BuffChargeConsumed { target: 0, .. } => CombatSfxAnchor::Lead,
        CombatEvent::BuffChargeConsumed { .. } => CombatSfxAnchor::Ally,
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
        "You"
    } else {
        "Ally"
    }
}

enum PlaybackStep {
    Event(CombatEvent),
    MergedPoison {
        total_damage: i32,
        tick_count: u32,
        stacks_before_last: u32,
    },
}

fn merged_poison_caption(total: i32, count: u32) -> String {
    if count <= 1 {
        format!("Poison deals {total} damage.")
    } else {
        format!("Poison deals {total} damage (×{count}).")
    }
}

fn flatten_playback_steps(events: &[CombatEvent]) -> Vec<PlaybackStep> {
    let mut out = Vec::new();
    let mut i = 0usize;
    while i < events.len() {
        if let CombatEvent::PoisonTick { damage, stacks } = events[i] {
            let mut total_damage = damage;
            let mut tick_count = 1u32;
            let mut stacks_before_last = stacks;
            i += 1;
            while i < events.len() {
                if let CombatEvent::PoisonTick {
                    damage: d2,
                    stacks: s2,
                } = events[i]
                {
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
    /// Last party slot the foe attacked (`0` lead, `1` ally), when known.
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
    /// 0–1 cast bar fill for the lead hero (weapon swing wind-up).
    pub lead_cast: f32,
    pub lead_cd: f32,
    pub ally_cast: f32,
    pub ally_cd: f32,
    pub foe_cast: f32,
    pub foe_cd: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CombatResult {
    pub outcome: CombatOutcome,
    pub hero_health: i32,
    /// Second party hero (slot 1) when present.
    pub partner_health: Option<i32>,
    pub partner_max_health: Option<i32>,
    pub enemy_health: i32,
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
    /// Cleave wins over Heavy when both equipped (yellow damage credit).
    heavy_skill: Option<SkillId>,
    has_empowered_blow: bool,
    has_victory_rush: bool,
    empower_gcd_ticks: u8,
    empower_icd_ticks: u8,
    vr_gcd_ticks: u8,
    vr_icd_ticks: u8,
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

    let (attack_cast_total, attack_cd_total) = attack_cadence_ticks(hero);

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
        vr_gcd_ticks: vr_def.gcd_ticks,
        vr_icd_ticks: vr_def.ability_icd_ticks,
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
        let w =
            ((scaled_total as i64 * white_sub as i64) / raw_total as i64).max(1) as i32;
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
    meters: &mut [f32; 2],
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
    h0_vr_icd_left: &mut u32,
    h0_empower_queued: &mut bool,
    h0_empower_icd_left: &mut u32,
) -> (Option<CombatBreak>, bool) {
    if *h0 > 0 && *enemy_health > 0 && p0.has_victory_rush {
        if *h0_skill_gcd_left == 0 && *h0_vr_icd_left == 0 {
            if events.len() >= max_events {
                return (None, true);
            }
            let strike = victory_rush_strike(p0, enemy);
            let hero_damage = strike.total();
            *enemy_health -= hero_damage;
            events.push(CombatEvent::HeroAttacked {
                attacker: 0,
                strike,
            });
            if has_partner {
                threat[0] = threat[0].saturating_add(hero_damage);
            }
            apply_lifesteal(p0, hero_damage, h0, p0.max_h, 0, events);
            *h0_skill_gcd_left = p0.vr_gcd_ticks.max(1) as u32;
            *h0_vr_icd_left = p0.vr_icd_ticks.max(1) as u32;
            if *enemy_health <= 0 {
                events.push(CombatEvent::EnemyDefeated);
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
            meters[0] += p0.attack_speed;
            as_applied[0] = true;
        }
        if meters[0] >= 1.0 && *h0 > 0 && *enemy_health > 0 {
            meters[0] -= 1.0;
            if events.len() >= max_events {
                return (None, true);
            }
            let consume = *h0_empower_queued;
            if consume {
                *h0_empower_queued = false;
                *h0_empower_icd_left = p0.empower_icd_ticks.max(1) as u32;
            }
            let strike = hero_strike_damage(p0, enemy, *h0, p0.max_h, consume);
            let hero_damage = strike.total();
            *enemy_health -= hero_damage;
            events.push(CombatEvent::HeroAttacked {
                attacker: 0,
                strike,
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
                events.push(CombatEvent::EnemyDefeated);
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
                }
                let strike = hero_strike_damage(p0, enemy, *h0, p0.max_h, consume);
                let hero_damage = strike.total();
                *enemy_health -= hero_damage;
                events.push(CombatEvent::HeroAttacked {
                    attacker: 0,
                    strike,
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
                    events.push(CombatEvent::EnemyDefeated);
                    maybe_devourer_heal_on_kill(lead, h0, p0.max_h, events);
                    return (Some(CombatBreak::HeroWin), true);
                }
                return (None, true);
            }
            return (None, true);
        }
        if !as_applied[0] {
            meters[0] += p0.attack_speed;
            as_applied[0] = true;
        }
        if meters[0] >= 1.0 && *h0 > 0 && *enemy_health > 0 {
            meters[0] -= 1.0;
            if p0.attack_cast_total > 0 {
                *h0_cast_left = p0.attack_cast_total;
                return (None, true);
            }
            if events.len() < max_events {
                let consume = *h0_empower_queued;
                if consume {
                    *h0_empower_queued = false;
                    *h0_empower_icd_left = p0.empower_icd_ticks.max(1) as u32;
                }
                let strike = hero_strike_damage(p0, enemy, *h0, p0.max_h, consume);
                let hero_damage = strike.total();
                *enemy_health -= hero_damage;
                events.push(CombatEvent::HeroAttacked {
                    attacker: 0,
                    strike,
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
                    events.push(CombatEvent::EnemyDefeated);
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
    lead: &HeroProfile,
    enemy: &Enemy,
    meters: &mut [f32; 2],
    h1_cd_left: &mut u32,
    h1_cast_left: &mut u32,
    as_applied: &mut [bool; 3],
    h0_lead: &mut i32,
    h1: &mut i32,
    enemy_health: &mut i32,
    poison_stacks: &mut u32,
    threat: &mut [i32; 2],
    events: &mut Vec<CombatEvent>,
    max_events: usize,
    p0_max_h: i32,
) -> (Option<CombatBreak>, bool) {
    if p1prep.attack_cast_total == 0 && p1prep.attack_cd_total == 0 {
        if *h1 <= 0 || *enemy_health <= 0 {
            return (None, false);
        }
        if !as_applied[1] {
            meters[1] += p1prep.attack_speed;
            as_applied[1] = true;
        }
        if meters[1] >= 1.0 && *h1 > 0 && *enemy_health > 0 {
            meters[1] -= 1.0;
            if events.len() >= max_events {
                return (None, true);
            }
            let strike = hero_strike_damage(p1prep, enemy, *h1, p1prep.max_h, false);
            let hero_damage = strike.total();
            *enemy_health -= hero_damage;
            events.push(CombatEvent::HeroAttacked {
                attacker: 1,
                strike,
            });
            threat[1] = threat[1].saturating_add(hero_damage);
            apply_lifesteal(p1prep, hero_damage, h1, p1prep.max_h, 1, events);
            if p1prep.has_poison && strike.white > 0 {
                let inc = if p1prep.affix_virulent { 3 } else { 2 };
                *poison_stacks = (*poison_stacks + inc).min(40);
            }
            if *enemy_health <= 0 {
                events.push(CombatEvent::EnemyDefeated);
                maybe_devourer_heal_on_kill(lead, h0_lead, p0_max_h, events);
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
                let strike = hero_strike_damage(p1prep, enemy, *h1, p1prep.max_h, false);
                let hero_damage = strike.total();
                *enemy_health -= hero_damage;
                events.push(CombatEvent::HeroAttacked {
                    attacker: 1,
                    strike,
                });
                threat[1] = threat[1].saturating_add(hero_damage);
                apply_lifesteal(p1prep, hero_damage, h1, p1prep.max_h, 1, events);
                if p1prep.has_poison && strike.white > 0 {
                    let inc = if p1prep.affix_virulent { 3 } else { 2 };
                    *poison_stacks = (*poison_stacks + inc).min(40);
                }
                *h1_cd_left = p1prep.attack_cd_total;
                if *enemy_health <= 0 {
                    events.push(CombatEvent::EnemyDefeated);
                    maybe_devourer_heal_on_kill(lead, h0_lead, p0_max_h, events);
                    return (Some(CombatBreak::HeroWin), true);
                }
                return (None, true);
            }
            return (None, true);
        }
        if !as_applied[1] {
            meters[1] += p1prep.attack_speed;
            as_applied[1] = true;
        }
        if meters[1] >= 1.0 && *h1 > 0 && *enemy_health > 0 {
            meters[1] -= 1.0;
            if p1prep.attack_cast_total > 0 {
                *h1_cast_left = p1prep.attack_cast_total;
                return (None, true);
            }
            if events.len() < max_events {
                let strike = hero_strike_damage(p1prep, enemy, *h1, p1prep.max_h, false);
                let hero_damage = strike.total();
                *enemy_health -= hero_damage;
                events.push(CombatEvent::HeroAttacked {
                    attacker: 1,
                    strike,
                });
                threat[1] = threat[1].saturating_add(hero_damage);
                apply_lifesteal(p1prep, hero_damage, h1, p1prep.max_h, 1, events);
                if p1prep.has_poison && strike.white > 0 {
                    let inc = if p1prep.affix_virulent { 3 } else { 2 };
                    *poison_stacks = (*poison_stacks + inc).min(40);
                }
                *h1_cd_left = p1prep.attack_cd_total;
                if *enemy_health <= 0 {
                    events.push(CombatEvent::EnemyDefeated);
                    maybe_devourer_heal_on_kill(lead, h0_lead, p0_max_h, events);
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
        events.push(CombatEvent::ThornsReflect { damage: reflect });
        if *enemy_health <= 0 {
            events.push(CombatEvent::EnemyDefeated);
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

/// One foe swing / cast tick / GCD tick (matches legacy `simulate_combat_party` branch ordering).
fn foe_weapon_pass(
    enemy: &Enemy,
    p0: &PreparedHero,
    p1prep: Option<&PreparedHero>,
    enemy_meter: &mut f32,
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
            *enemy_meter += enemy_as;
            as_applied[2] = true;
        }
        if *enemy_meter >= 1.0 && *enemy_health > 0 && *h0 > 0 && (!has_partner || *h1 > 0) {
            *enemy_meter -= 1.0;
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
                events.push(CombatEvent::EnemyDefeated);
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
                    events.push(CombatEvent::EnemyDefeated);
                    maybe_devourer_heal_on_kill(lead, h0, p0.max_h, events);
                    return (Some(CombatBreak::HeroWin), true);
                }
                *foe_cd_left = foe_dt;
                return (None, true);
            }
            return (None, true);
        }
        if !as_applied[2] {
            *enemy_meter += enemy_as;
            as_applied[2] = true;
        }
        if *enemy_meter >= 1.0 && *enemy_health > 0 && *h0 > 0 && (!has_partner || *h1 > 0) {
            *enemy_meter -= 1.0;
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
                    events.push(CombatEvent::EnemyDefeated);
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
            let expires_at = buff_expires_at_clock(combat_clock, app.duration_ticks);
            let buff = ActivePartyBuff {
                id: app.buff_id,
                stacks: app.stacks,
                expires_at,
                charges: app.charges,
            };
            match target {
                0 => self.slot0.push(buff),
                1 => self.slot1.push(buff),
                _ => continue,
            }
            events.push(CombatEvent::BuffApplied {
                target,
                buff_id: app.buff_id,
                stacks: app.stacks,
                duration_ticks: app.duration_ticks,
            });
        }
    }

    fn tick_end(
        &mut self,
        combat_clock: u32,
        events: &mut Vec<CombatEvent>,
        max_events: usize,
    ) {
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
            let drop = slot[i]
                .expires_at
                .is_some_and(|ex| combat_clock >= ex);
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
    let p0 = prepare_hero_combat(lead);
    let p1 = partner.map(|(h, hp)| (prepare_hero_combat(h), h, hp));
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
            enemy_health: enemy.max_health,
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
            enemy_health: enemy.max_health,
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
    let mut meters = [0.0_f32, 0.0_f32];
    let mut enemy_meter = 0.0_f32;

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
                enemy_health,
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
                enemy_health,
                clock_ticks: $cc,
                events,
            };
        };
    }

    let mut last_enemy_target: Option<u8> = None;

    let mut h0_cast_left = 0u32;
    let mut h0_cd_left = 0u32;
    let mut h0_skill_gcd_left = 0u32;
    let mut h0_vr_icd_left = 0u32;
    let mut h0_empower_queued = false;
    let mut h0_empower_icd_left = 0u32;
    let mut h1_cast_left = 0u32;
    let mut h1_cd_left = 0u32;
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
            if h0 > 0
                && h1 > 0
                && enemy_health > 0
                && crate::domain::party::threat_stance_tick_drip(h1hero)
            {
                threat[1] = threat[1].saturating_add(1);
            }
        }

        if p0.has_victory_rush && h0_vr_icd_left > 0 {
            h0_vr_icd_left -= 1;
        }
        if p0.has_empowered_blow && h0_empower_icd_left > 0 {
            h0_empower_icd_left -= 1;
        }
        if p0.has_empowered_blow || p0.has_victory_rush {
            if h0_skill_gcd_left > 0 {
                h0_skill_gcd_left -= 1;
            }
        }
        let vr_ready =
            p0.has_victory_rush && h0_vr_icd_left == 0 && h0_skill_gcd_left == 0;
        if p0.has_empowered_blow
            && !h0_empower_queued
            && h0_skill_gcd_left == 0
            && h0_empower_icd_left == 0
            && !vr_ready
        {
            h0_empower_queued = true;
            h0_skill_gcd_left = p0.empower_gcd_ticks.max(1) as u32;
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
                        &mut h0_vr_icd_left,
                        &mut h0_empower_queued,
                        &mut h0_empower_icd_left,
                    ),
                    1 => {
                        if let Some((ref p1prep, _, _)) = p1 {
                            partner_weapon_pass(
                                p1prep,
                                lead,
                                enemy,
                                &mut meters,
                                &mut h1_cd_left,
                                &mut h1_cast_left,
                                &mut as_applied_tick,
                                &mut h0,
                                &mut h1,
                                &mut enemy_health,
                                &mut poison_stacks,
                                &mut threat,
                                &mut events,
                                MAX_EVENTS,
                                p0.max_h,
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

        let lead_cast = if p0.attack_cast_total == 0 {
            0.0
        } else if h0_cast_left > 0 {
            1.0 - (h0_cast_left as f32 / p0.attack_cast_total as f32)
        } else {
            0.0
        };
        let lead_cd = if p0.attack_cd_total == 0 {
            0.0
        } else if h0_cd_left > 0 {
            1.0 - (h0_cd_left as f32 / p0.attack_cd_total as f32)
        } else {
            0.0
        };
        let ally_cast = if has_partner {
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
        let ally_cd = if has_partner {
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
        if events.len() < MAX_EVENTS {
            events.push(CombatEvent::TimingPulse {
                lead_cast,
                lead_cd,
                ally_cast,
                ally_cd,
                foe_cast,
                foe_cd: foe_cd_b,
            });
        }

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
            });
            poison_stacks -= 1;
            enemy_health -= d;
            if enemy_health <= 0 {
                events.push(CombatEvent::EnemyDefeated);
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
        enemy_health,
        clock_ticks: combat_clock,
        events,
    }
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

fn debuff_slots_from_poison(hero_poison: u32, enemy_poison: u32) -> ([String; 4], [String; 4]) {
    let mut hero = empty_debuff_slots();
    let mut enemy = empty_debuff_slots();
    if enemy_poison > 0 {
        enemy[0] = format!("Poison ×{}", enemy_poison);
    }
    if hero_poison > 0 {
        hero[0] = format!("Poison ×{}", hero_poison);
    }
    (hero, enemy)
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

    let (h0, e0) = debuff_slots_from_poison(hero_poison_stacks, enemy_poison_stacks);

    let mut d0 = 0u32;
    let mut d1 = 0u32;
    let mut foe_meter = 0u32;

    let mut last_caption = format!("Engaging {enemy_name}.");

    let steps = flatten_playback_steps(&result.events);
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
        lead_cast: 0.0,
        lead_cd: 0.0,
        ally_cast: 0.0,
        ally_cd: 0.0,
        foe_cast: 0.0,
        foe_cd: 0.0,
    }];

    let mut lead_cast = 0.0f32;
    let mut lead_cd = 0.0f32;
    let mut ally_cast = 0.0f32;
    let mut ally_cd = 0.0f32;
    let mut foe_cast_b = 0.0f32;
    let mut foe_cd_bar = 0.0f32;

    for step in steps {
        match &step {
            PlaybackStep::Event(event) => match event {
                CombatEvent::HeroAttacked { attacker, strike } => {
                    let total = strike.total();
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
                CombatEvent::PoisonTick { damage, stacks } => {
                    d0 = d0.saturating_add(*damage as u32);
                    enemy_hp -= damage;
                    enemy_poison_stacks = stacks.saturating_sub(1);
                }
                CombatEvent::ThornsReflect { damage } => {
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
                    lead_cast: lc,
                    lead_cd: lcdn,
                    ally_cast: ac,
                    ally_cd: acdn,
                    foe_cast: fc,
                    foe_cd: fcdn,
                } => {
                    lead_cast = *lc;
                    lead_cd = *lcdn;
                    ally_cast = *ac;
                    ally_cd = *acdn;
                    foe_cast_b = *fc;
                    foe_cd_bar = *fcdn;
                }
                CombatEvent::BuffApplied { .. }
                | CombatEvent::BuffExpired { .. }
                | CombatEvent::BuffTick { .. }
                | CombatEvent::BuffChargeConsumed { .. } => {}
                CombatEvent::PartyMemberDown { .. } => {}
                CombatEvent::EnemyDefeated | CombatEvent::HeroDefeated => {}
            },
            PlaybackStep::MergedPoison {
                total_damage,
                tick_count,
                stacks_before_last,
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
        let (hd, ed) = debuff_slots_from_poison(hero_poison_stacks, enemy_poison_stacks);

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
            lead_cast,
            lead_cd,
            ally_cast,
            ally_cd,
            foe_cast: foe_cast_b,
            foe_cd: foe_cd_bar,
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
    use crate::domain::dungeon::Enemy;
    use crate::domain::hero::HeroProfile;
    use crate::domain::items::{GearSlot, ItemAffix, ItemInstance};
    use crate::domain::skills::SkillId;
    use crate::domain::stats::Stats;

    #[test]
    fn pick_party_enemy_no_partner_is_always_lead() {
        assert_eq!(
            pick_party_enemy_target(0, [99, 1], 100, 100, false, Some(1)),
            0
        );
    }

    #[test]
    fn pick_party_enemy_partner_down_targets_lead() {
        assert_eq!(
            pick_party_enemy_target(0, [1, 99], 100, 0, true, Some(1)),
            0
        );
    }

    #[test]
    fn pick_party_enemy_higher_threat_on_lead_targets_lead() {
        assert_eq!(
            pick_party_enemy_target(0, [30, 10], 100, 100, true, Some(1)),
            0
        );
    }

    #[test]
    fn pick_party_enemy_higher_threat_on_ally_targets_ally() {
        assert_eq!(
            pick_party_enemy_target(0, [10, 40], 100, 100, true, Some(0)),
            1
        );
    }

    #[test]
    fn pick_party_enemy_threat_tie_keeps_last_target_on_ally() {
        assert_eq!(
            pick_party_enemy_target(7, [20, 20], 100, 100, true, Some(1)),
            1
        );
    }

    #[test]
    fn pick_party_enemy_threat_tie_keeps_last_target_on_lead() {
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
    fn pick_party_enemy_invalid_sticky_falls_back_when_only_ally_alive() {
        assert_eq!(pick_party_enemy_target(3, [5, 5], 0, 100, true, Some(0)), 1);
    }

    #[test]
    fn pick_party_enemy_tie_without_sticky_is_stable_for_tick() {
        let t = pick_party_enemy_target(4, [0, 0], 100, 100, true, None);
        assert_eq!(t, pick_party_enemy_target(4, [0, 0], 100, 100, true, None));
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
            .any(|event| matches!(event, CombatEvent::EnemyDefeated)));
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
        let r = simulate_combat_party_with_initial_buffs(
            &hero,
            &enemy,
            8,
            100,
            None,
            0,
            &initial,
        );
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
            CombatEvent::ThornsReflect { damage } if *damage > 0
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
                CombatEvent::ThornsReflect { damage } if *damage >= 10
            )),
            "reflect should kill the low-HP enemy before the hero swings this fight"
        );
        assert!(
            r.events
                .iter()
                .any(|e| matches!(e, CombatEvent::EnemyDefeated)),
            "defeat should be attributed after thorns damage"
        );
    }

    #[test]
    fn spiked_affix_thorns_reflect_kills_low_hp_enemy() {
        let mut hero = HeroProfile::default();
        hero.base_stats.attack_speed = 0.12;
        hero.unlock_skill_slots(1);
        hero.equip_skill(0, SkillId::ThornSkin).unwrap();
        let mut mail = ItemInstance::basic(9, "Spiked mail", GearSlot::Armor);
        mail.affixes.push(ItemAffix::Spiked);
        hero.equip_item(mail).unwrap();

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
        let mut club = ItemInstance::basic(1, "Club", GearSlot::Weapon);
        club.stats.damage = 4;
        skill_only.equip_item(club).unwrap();

        let mut with_affix = HeroProfile::default();
        with_affix.unlock_skill_slots(1);
        with_affix.equip_skill(0, SkillId::HeavyStrike).unwrap();
        let mut maul = ItemInstance::basic(2, "Maul", GearSlot::Weapon);
        maul.stats.damage = 0;
        maul.stats.attack_speed = 0.2;
        maul.affixes.push(ItemAffix::Heavy);
        with_affix.equip_item(maul).unwrap();

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
        let mut blade = ItemInstance::basic(2, "Fang", GearSlot::Weapon);
        blade.affixes.push(ItemAffix::Vampiric);
        geared.equip_item(blade).unwrap();

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
        let mut spiky = ItemInstance::basic(3, "Spiky mail", GearSlot::Armor);
        spiky.affixes.push(ItemAffix::Spiked);
        both.equip_item(spiky).unwrap();

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
        let mut cloth = ItemInstance::basic(4, "Shroud", GearSlot::Trinket);
        cloth.affixes.push(ItemAffix::Cursed);
        cursed.equip_item(cloth).unwrap();

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
        let mut mace = ItemInstance::basic(9, "Ram", GearSlot::Weapon);
        mace.affixes.push(ItemAffix::Shattering);
        pierce.equip_item(mace).unwrap();

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
        let mut orb = ItemInstance::basic(10, "Ichor", GearSlot::Trinket);
        orb.affixes.push(ItemAffix::Virulent);
        v.equip_item(orb).unwrap();

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
        let mut shield = ItemInstance::basic(11, "Bulwark", GearSlot::Armor);
        shield.affixes.push(ItemAffix::Bastion);
        wall.equip_item(shield).unwrap();

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
        let mut axe = ItemInstance::basic(55, "Titan maul", GearSlot::Weapon);
        axe.affixes.push(ItemAffix::TitansFury);
        fury.equip_item(axe).unwrap();

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
        let mut glaive = ItemInstance::basic(56, "Maw", GearSlot::Weapon);
        glaive.affixes.push(ItemAffix::Devourer);
        hero.equip_item(glaive).unwrap();

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
            if matches!(e, CombatEvent::EnemyDefeated) {
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
                if let CombatEvent::ThornsReflect { damage } = e {
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
