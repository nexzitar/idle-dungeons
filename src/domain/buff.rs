//! Buff identifiers and application specs for combat simulation (Phase 3).
//!
//! Runtime state lives in [`crate::domain::combat`] to avoid cycles with [`CombatEvent`].

/// Identifiers for simulated buffs (`CombatEvent` replay / UI).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BuffId {
    /// Test / extension hook; future skills map here.
    InnerStrength,
}

pub fn buff_display_name(id: BuffId) -> &'static str {
    match id {
        BuffId::InnerStrength => "Inner Strength",
    }
}

/// Spec for spawning a buff at encounter start (or later from skill hooks).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BuffApplication {
    pub buff_id: BuffId,
    pub stacks: u32,
    pub duration_ticks: Option<u32>,
    pub charges: Option<u32>,
}

/// `apply_at` is the combat clock value when the buff is granted; `duration_ticks` is `None` for permanent.
pub(crate) fn buff_expires_at_clock(
    apply_at_combat_clock: u32,
    duration_ticks: Option<u32>,
) -> Option<u32> {
    duration_ticks.map(|d| apply_at_combat_clock.saturating_add(d))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn expires_at_adds_duration() {
        assert_eq!(buff_expires_at_clock(0, Some(3)), Some(3));
        assert_eq!(buff_expires_at_clock(5, Some(1)), Some(6));
        assert_eq!(buff_expires_at_clock(0, None), None);
    }
}
