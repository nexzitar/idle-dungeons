use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MetaProgression {
    pub gold: u32,
    pub salvage: u32,
    pub unlocked_skill_slots: usize,
    pub skill_slot_progress: u32,
    /// Deepest floor reached on any finished run (used for party slot unlocks).
    #[serde(default)]
    pub deepest_floor_reached: u32,
}

impl Default for MetaProgression {
    fn default() -> Self {
        Self {
            gold: 0,
            salvage: 0,
            unlocked_skill_slots: 2,
            skill_slot_progress: 0,
            deepest_floor_reached: 0,
        }
    }
}

/// Second party member unlocks after reaching this floor depth on at least one run.
pub const PARTY_SLOT_2_UNLOCK_DEPTH: u32 = 75;

impl MetaProgression {
    pub fn add_skill_slot_progress(&mut self, amount: u32) {
        self.skill_slot_progress = self.skill_slot_progress.saturating_add(amount);
        let cap = if self.skill_slot_progress >= 1000 {
            6
        } else if self.skill_slot_progress >= 600 {
            5
        } else if self.skill_slot_progress >= 300 {
            4
        } else if self.skill_slot_progress >= 100 {
            3
        } else {
            2
        };
        self.unlocked_skill_slots = self.unlocked_skill_slots.max(cap);
    }

    /// Party size cap from meta progression (currently 1 until [`PARTY_SLOT_2_UNLOCK_DEPTH`], then 2).
    pub fn party_slots_unlocked(&self) -> usize {
        if self.deepest_floor_reached >= PARTY_SLOT_2_UNLOCK_DEPTH {
            2
        } else {
            1
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn skill_slot_unlocks_after_progress_threshold() {
        let mut profile = MetaProgression::default();

        profile.add_skill_slot_progress(100);

        assert_eq!(profile.unlocked_skill_slots, 3);
    }

    #[test]
    fn skill_slot_unlocks_through_six_slots_with_progress_milestones() {
        let mut profile = MetaProgression::default();

        profile.add_skill_slot_progress(100);
        assert_eq!(profile.unlocked_skill_slots, 3);

        profile.add_skill_slot_progress(200);
        assert_eq!(profile.unlocked_skill_slots, 4);

        profile.add_skill_slot_progress(300);
        assert_eq!(profile.unlocked_skill_slots, 5);

        profile.add_skill_slot_progress(400);
        assert_eq!(profile.unlocked_skill_slots, 6);
    }

    #[test]
    fn party_second_slot_unlocks_at_depth_75() {
        let mut m = MetaProgression::default();
        m.deepest_floor_reached = 74;
        assert_eq!(m.party_slots_unlocked(), 1);
        m.deepest_floor_reached = 75;
        assert_eq!(m.party_slots_unlocked(), 2);
    }
}
