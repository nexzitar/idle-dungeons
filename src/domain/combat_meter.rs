//! Fixed-point **weapon swing meters** (no `f32` accumulation drift).
//!
//! One full swing is **[`WEAPON_METER_ONE_SWING`]** sub-units. Each outer tick adds
//! `attack_speed × WEAPON_METER_ONE_SWING` (via `f64` conversion from stat `f32`) at most once per
//! actor pass; [`meter_try_consume_swing`] subtracts one full swing when the meter is ready.

/// Sub-units representing **1.0** ready swing (micro-units keep room for typical `attack_speed` additions).
pub const WEAPON_METER_ONE_SWING: u64 = 1_000_000;

#[inline]
pub fn meter_add_attack_speed(m: &mut u64, attack_speed: f32) {
    let inc =
        (f64::from(attack_speed).clamp(0.0, 1.0e6) * WEAPON_METER_ONE_SWING as f64) as u64;
    *m = m.saturating_add(inc);
}

/// Returns `true` if a swing was consumed (`m` decreased by one full unit).
#[inline]
pub fn meter_try_consume_swing(m: &mut u64) -> bool {
    if *m >= WEAPON_METER_ONE_SWING {
        *m -= WEAPON_METER_ONE_SWING;
        true
    } else {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn meter_consumes_exactly_one_swing_unit() {
        let mut m = WEAPON_METER_ONE_SWING - 1;
        assert!(!meter_try_consume_swing(&mut m));
        assert_eq!(m, WEAPON_METER_ONE_SWING - 1);

        m = WEAPON_METER_ONE_SWING;
        assert!(meter_try_consume_swing(&mut m));
        assert_eq!(m, 0);
    }

    #[test]
    fn repeated_add_then_consume_no_drift_for_typical_speed() {
        let mut m = 0u64;
        for _ in 0..10_000 {
            meter_add_attack_speed(&mut m, 0.17);
        }
        let mut swings = 0u32;
        while meter_try_consume_swing(&mut m) {
            swings += 1;
        }
        assert_eq!(swings, 1700, "remainder meter = {m}");
        assert!(m < WEAPON_METER_ONE_SWING);
    }
}
