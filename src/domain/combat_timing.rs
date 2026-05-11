//! Deterministic combat tick and initiative (see combat feel spec).

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
    Lead,
    Partner,
    Foe,
}

/// Returns **unique** ranks `0..N-1` for actors in this encounter: **lower = earlier** when `ready_at_tick` ties.
/// Index: `0` = lead, `1` = partner, `2` = foe. Unused slots stay `255`.
/// `party_slots`: `1` = solo (lead + foe only), `2` = lead + partner + foe.
pub fn initiative_ranks(encounter_seed: u64, party_slots: u8) -> [u8; 3] {
    let mut entries: Vec<(u8, u64)> = Vec::with_capacity(3);
    entries.push((0u8, mix64(encounter_seed, 0x4C454144_u64)));
    if party_slots >= 2 {
        entries.push((1u8, mix64(encounter_seed, 0x50415254_u64)));
    }
    entries.push((2u8, mix64(encounter_seed, 0x464F4520_u64)));
    entries.sort_by_key(|&(_, k)| k);
    let mut out = [255u8; 3];
    for (rank, (actor_id, _)) in entries.iter().enumerate() {
        out[*actor_id as usize] = rank as u8;
    }
    out
}

#[inline]
pub fn initiative_rank_for(actor: InitiativeActor, encounter_seed: u64, party_slots: u8) -> u8 {
    let idx = match actor {
        InitiativeActor::Lead => 0usize,
        InitiativeActor::Partner => 1usize,
        InitiativeActor::Foe => 2usize,
    };
    initiative_ranks(encounter_seed, party_slots)[idx]
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
        let a = initiative_rank_for(InitiativeActor::Lead, 42, 2);
        let b = initiative_rank_for(InitiativeActor::Lead, 42, 2);
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
