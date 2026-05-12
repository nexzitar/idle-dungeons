use crate::domain::skills::{SkillId, STARTER_SKILLS};
use serde::{Deserialize, Serialize};

fn default_unlocked_skill_ids_migration() -> Vec<SkillId> {
    STARTER_SKILLS.iter().copied().collect()
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MetaProgression {
    pub gold: u32,
    pub salvage: u32,
    pub unlocked_skill_slots: usize,
    pub skill_slot_progress: u32,
    /// Deepest floor reached on any finished run (used for party slot unlocks).
    #[serde(default)]
    pub deepest_floor_reached: u32,
    /// Skill book entries purchased or granted (starters).
    /// Saves that omit this field deserialize to the **starter set** (matches a fresh profile).
    #[serde(default = "default_unlocked_skill_ids_migration")]
    pub unlocked_skill_ids: Vec<SkillId>,
    /// Lifetime claims of guaranteed **before-depth-10** combat salvage on the player's profile:
    /// first is **main hand**, second **chest**; later rolls use varied seeds so repeats are not clones.
    #[serde(default)]
    pub guided_early_combat_drop_count: u32,
}

impl Default for MetaProgression {
    fn default() -> Self {
        Self {
            gold: 0,
            salvage: 0,
            unlocked_skill_slots: 2,
            skill_slot_progress: 0,
            deepest_floor_reached: 0,
            unlocked_skill_ids: STARTER_SKILLS.iter().copied().collect(),
            guided_early_combat_drop_count: 0,
        }
    }
}

/// Second party member unlocks after reaching this floor depth on at least one run.
pub const PARTY_SLOT_2_UNLOCK_DEPTH: u32 = 75;

impl MetaProgression {
    pub fn has_skill_unlocked(&self, id: SkillId) -> bool {
        self.unlocked_skill_ids.iter().any(|&s| s == id)
    }

    pub fn add_skill_unlock(&mut self, id: SkillId) -> bool {
        if self.has_skill_unlocked(id) {
            return false;
        }
        self.unlocked_skill_ids.push(id);
        self.unlocked_skill_ids.sort_by_key(|s| {
            SkillId::ALL
                .iter()
                .position(|x| x == s)
                .unwrap_or(999)
        });
        true
    }

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
    use crate::domain::skills::SkillId;

    #[test]
    fn starter_skills_only_in_default_meta() {
        let m = MetaProgression::default();
        assert_eq!(m.guided_early_combat_drop_count, 0);
        assert_eq!(m.unlocked_skill_ids.len(), 4);
        assert!(m.has_skill_unlocked(SkillId::HeavyStrike));
        assert!(!m.has_skill_unlocked(SkillId::PoisonEdge));
    }

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
