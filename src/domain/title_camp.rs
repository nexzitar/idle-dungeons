//! Title campfire hub: physical seats around the fire (not combat roles).

/// Campfire scene has this many **physical seats** (tuning ids `player1` … `player6`).
pub const CAMP_FIRE_SEATS: usize = 6;

/// Assign `unlocked_players` (0..=6) to distinct random seats. `out[seat]` = player index (0-based).
#[must_use]
pub fn assign_players_to_camp_seats(unlocked_players: usize, seed: u64) -> [Option<u8>; CAMP_FIRE_SEATS] {
    let mut out = [None; CAMP_FIRE_SEATS];
    let n = unlocked_players.min(CAMP_FIRE_SEATS);
    if n == 0 {
        return out;
    }

    let mut seat_order: [usize; CAMP_FIRE_SEATS] = [0, 1, 2, 3, 4, 5];
    shuffle_usize(&mut seat_order, seed);

    for (player_idx, &seat) in (0..n).zip(seat_order.iter().take(n)) {
        out[seat] = Some(player_idx as u8);
    }
    out
}

/// Deterministic Fisher–Yates using xorshift64 (no per-frame alloc).
fn shuffle_usize(slice: &mut [usize], mut seed: u64) {
    let n = slice.len();
    if n <= 1 {
        return;
    }
    for i in (1..n).rev() {
        seed ^= seed << 13;
        seed ^= seed >> 7;
        seed ^= seed << 17;
        let j = (seed as usize) % (i + 1);
        slice.swap(i, j);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn assignment_uses_distinct_seats() {
        let a = assign_players_to_camp_seats(2, 42);
        let seats: Vec<_> = a.iter().filter_map(|&x| x).collect();
        assert_eq!(seats.len(), 2);
        assert_ne!(seats[0], seats[1]);
    }

    #[test]
    fn assignment_is_deterministic() {
        let a = assign_players_to_camp_seats(2, 99);
        let b = assign_players_to_camp_seats(2, 99);
        assert_eq!(a, b);
    }
}
