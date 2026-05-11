//! Per-tick strike ordering: initiative sort + round-robin interleaving.

/// Party slot indices: `0` = lead, `1` = partner, `2` = foe.
pub type PartySlot = u8;

/// Actors that can swing this tick, ordered by [`super::combat_timing::initiative_ranks`] (lower = earlier).
pub fn sorted_strike_actors(has_partner: bool, ranks: [u8; 3]) -> Vec<PartySlot> {
    let mut actors: Vec<PartySlot> = if has_partner {
        vec![0, 1, 2]
    } else {
        vec![0, 2]
    };
    actors.sort_by_key(|&a| ranks[a as usize]);
    actors
}

/// Interleave strikes: in each sweep, every actor with remaining swings takes **one** hit,
/// in initiative order. Continues until all `counts` are zero.
pub fn round_robin_strike_order(mut counts: [u32; 3], actor_order: &[PartySlot]) -> Vec<PartySlot> {
    let mut out = Vec::new();
    loop {
        let mut progressed = false;
        for &actor in actor_order {
            let i = actor as usize;
            if i < 3 && counts[i] > 0 {
                counts[i] -= 1;
                out.push(actor);
                progressed = true;
            }
        }
        if !progressed {
            break;
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_robin_alternates_when_counts_match() {
        let order = sorted_strike_actors(true, [0, 1, 2]);
        let seq = round_robin_strike_order([2, 2, 2], &order);
        assert_eq!(seq.len(), 6);
        assert_eq!(seq[0], 0);
        assert_eq!(seq[1], 1);
        assert_eq!(seq[2], 2);
    }

    #[test]
    fn solo_skips_partner_index() {
        let order = sorted_strike_actors(false, [0, 255, 1]);
        assert_eq!(order, vec![0, 2]);
        let seq = round_robin_strike_order([2, 0, 1], &order);
        assert_eq!(seq, vec![0, 2, 0]);
    }
}
