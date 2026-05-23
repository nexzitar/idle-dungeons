//! Deterministic combat tick and stable initiative ordering.
//!
//! ## Tick = time quantum
//!
//! One outer simulation step in [`crate::domain::combat::simulate_combat_party`] advances the combat
//! clock by **one tick**. **[`COMBAT_TICK_MS`]** (`100`) is the canonical fictional duration of that
//! step for UI copy and DPS math (e.g. `sim_ticks * COMBAT_TICK_MS` milliseconds).
//!
//! ## Wall clock vs tick cap
//!
//! `max_clock_ticks` is a **hard upper bound on tick iterations** for the encounter loop, not a
//! separate real-time timer. Fight ends earlier on victory/defeat.
//!
//! ## GCD, weapon cadence, casts (where to read rules)
//!
//! - **Weapon wind-up / post-swing cooldown:** [`crate::domain::skills::SkillCombatStyle::SwingWeave`]
//!   actives use per-hero `attack_cast_total` / `attack_cd_total` from [`crate::domain::skills::skill_timings`].
//! - **Shared ability GCD** (instant strikes + next-melee buff queues): gated by
//!   [`crate::domain::skills::skill_triggers_shared_ability_gcd`] and `h*_skill_gcd_left` in combat sim.
//! - **Per-charge recharge** (Victory Rush style): `ability_icd_ticks` pulses in [`crate::domain::combat`].
//!
//! ## Initiative
//!
//! Same-tick melee ordering uses [`initiative_ranks`] keyed by encounter salt (see
//! [`encounter_initiative_seed`]). [`ActionLane`] orders proactive swings vs end-of-tick poison batching.
//!
//! Active backlog / future hardening: `docs/superpowers/ACTIVE-REMAINING-WORK.md`.

/// In-fiction duration of one simulation tick (milliseconds).
pub const COMBAT_TICK_MS: u32 = 100;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ActionLane {
    /// Weapon swings / cast damage resolutions scheduled this tick.
    Proactive,
    /// Poison DoT slice and similar end-of-window effects (after proactive unless spec says otherwise).
    Dot,
    /// Reserved: barrier react, on-hit procs (Phase 3+).
    Reactive,
}

/// Mix seed with a tag; stable, dependency-free (no floats).
#[inline]
pub fn mix64(seed: u64, tag: u64) -> u64 {
    seed.wrapping_add(tag)
        .rotate_left(23)
        .wrapping_mul(0x9E37_79B97F4A7C15)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InitiativeActor {
    Player0,
    Player1,
    Foe,
}

/// Returns **unique** ranks `0..N-1` for actors in this encounter: **lower = earlier** when `ready_at_tick` ties.
/// Index: `0` = player 1, `1` = player 2, `2` = first foe, `3` = second foe (when `foe_count == 2`).
/// Unused slots stay `255`.
/// `party_slots`: `1` = solo (player 1 + foe(s) only), `2` = two players + foe(s).
/// `foe_count`: `1` or `2`.
pub fn initiative_ranks_four(encounter_seed: u64, party_slots: u8, foe_count: u8) -> [u8; 4] {
    debug_assert!(foe_count >= 1 && foe_count <= 2);
    let mut entries: Vec<(u8, u64)> = Vec::with_capacity(4);
    entries.push((0u8, mix64(encounter_seed, 0x4C454144_u64)));
    if party_slots >= 2 {
        entries.push((1u8, mix64(encounter_seed, 0x50415254_u64)));
    }
    entries.push((2u8, mix64(encounter_seed, 0x464F4520_u64)));
    if foe_count >= 2 {
        entries.push((3u8, mix64(encounter_seed, 0x464F4521_u64)));
    }
    entries.sort_by_key(|&(_, k)| k);
    let mut out = [255u8; 4];
    for (rank, (actor_id, _)) in entries.iter().enumerate() {
        out[*actor_id as usize] = rank as u8;
    }
    out
}

/// Initiative ranks for `party_count` heroes (slots `0..party_count-1`) plus `foe_count` foes.
/// Returned [`Vec`] length is `party_count + foe_count`; each entry is the rank for that global slot.
pub fn initiative_ranks_pack(encounter_seed: u64, party_count: u8, foe_count: u8) -> Vec<u8> {
    let pc = party_count as usize;
    let fc = foe_count as usize;
    let n = pc + fc;
    let mut entries: Vec<(usize, u64)> = Vec::with_capacity(n);
    for i in 0..pc {
        entries.push((
            i,
            mix64(encounter_seed, 0x4C450000u64.wrapping_add(i as u64)),
        ));
    }
    for j in 0..fc {
        let idx = pc + j;
        entries.push((
            idx,
            mix64(encounter_seed, 0x464F4500u64.wrapping_add(j as u64)),
        ));
    }
    entries.sort_by_key(|&(_, k)| k);
    let mut out = vec![255u8; n];
    for (rank, (actor_id, _)) in entries.iter().enumerate() {
        out[*actor_id] = rank as u8;
    }
    out
}

/// Legacy shape for single-foe encounters (`foe` at index `2`).
pub fn initiative_ranks(encounter_seed: u64, party_slots: u8) -> [u8; 3] {
    let four = initiative_ranks_four(encounter_seed, party_slots, 1);
    [four[0], four[1], four[2]]
}

#[inline]
pub fn initiative_rank_for(actor: InitiativeActor, encounter_seed: u64, party_slots: u8) -> u8 {
    let idx = match actor {
        InitiativeActor::Player0 => 0usize,
        InitiativeActor::Player1 => 1usize,
        InitiativeActor::Foe => 2usize,
    };
    initiative_ranks_four(encounter_seed, party_slots, 1)[idx]
}

/// Derives a per-encounter seed for initiative (and future scheduling hooks).
#[inline]
pub fn encounter_initiative_seed(run_seed: u64, enemy_id_mix: u64) -> u64 {
    mix64(run_seed, enemy_id_mix)
}

/// Total order for pending actions on one tick.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ActionSortKey {
    pub tick: u32,
    pub initiative: u8,
    pub lane: ActionLane,
    pub seq: u32,
}

impl Ord for ActionSortKey {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        (self.tick, self.initiative, self.lane, self.seq).cmp(&(
            other.tick,
            other.initiative,
            other.lane,
            other.seq,
        ))
    }
}

impl PartialOrd for ActionSortKey {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn initiative_deterministic_for_seed() {
        let a = initiative_rank_for(InitiativeActor::Player0, 42, 2);
        let b = initiative_rank_for(InitiativeActor::Player0, 42, 2);
        let c = initiative_rank_for(InitiativeActor::Foe, 42, 2);
        assert_eq!(a, b);
        assert_ne!(a, c, "lead and foe should not always tie on mixed keys");
    }

    #[test]
    fn initiative_ranks_are_permutation_no_collisions_solo() {
        let r = initiative_ranks(99, 1);
        assert_eq!(r[1], 255, "partner absent");
        let mut seen = [false, false];
        for slot in [0usize, 2usize] {
            let k = r[slot] as usize;
            assert!(k < 2, "rank must be 0..2 for solo");
            assert!(!seen[k], "duplicate rank");
            seen[k] = true;
        }
    }

    #[test]
    fn sort_key_orders_lane_then_seq() {
        let a = ActionSortKey {
            tick: 1,
            initiative: 0,
            lane: ActionLane::Proactive,
            seq: 1,
        };
        let b = ActionSortKey {
            tick: 1,
            initiative: 0,
            lane: ActionLane::Dot,
            seq: 0,
        };
        assert!(a < b, "Proactive before Dot at same initiative");
    }

    #[test]
    fn not_all_run_salts_collide_on_initiative_solo() {
        let first = initiative_ranks(encounter_initiative_seed(0, 0), 1);
        let mut found_distinct = false;
        for salt in 1..512u64 {
            if initiative_ranks(encounter_initiative_seed(salt, 0), 1) != first {
                found_distinct = true;
                break;
            }
        }
        assert!(
            found_distinct,
            "expected some run salts to change lead/foe initiative order"
        );
    }

    #[test]
    fn encounter_initiative_seed_is_deterministic() {
        assert_eq!(
            encounter_initiative_seed(7, 0xABC),
            encounter_initiative_seed(7, 0xABC)
        );
    }
}
