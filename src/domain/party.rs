//! Minimal party scaffolding for Phase 3 threat routing (MVP: one fixed tank ally in full runs).

use crate::domain::stats::Stats;

/// Display name for the built-in tank NPC that joins delve combat (not a saved hero).
pub const TANK_ALLY_NAME: &str = "Bulwark Squire";

/// Baseline stats for the tank ally (armor-focused; does not use the hero skill deck).
pub fn tank_ally_starting_stats() -> Stats {
    Stats {
        max_health: 72,
        damage: 5,
        armor: 9,
        attack_speed: 0.55,
        healing_power: 0,
    }
}
