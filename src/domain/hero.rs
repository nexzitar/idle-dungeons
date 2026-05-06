use crate::domain::items::{GearSlot, ItemInstance};
use crate::domain::skills::SkillId;
use crate::domain::stats::Stats;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum HeroError {
    #[error("skill slot {slot} is locked")]
    SkillSlotLocked { slot: usize },
    #[error("skill slot {slot} is unavailable")]
    SkillSlotUnavailable { slot: usize },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HeroProfile {
    pub base_stats: Stats,
    pub unlocked_skill_slots: usize,
    pub equipped_skills: Vec<Option<SkillId>>,
    pub equipped_items: HashMap<GearSlot, ItemInstance>,
}

impl Default for HeroProfile {
    fn default() -> Self {
        Self::new(Stats::default())
    }
}

impl HeroProfile {
    pub fn new(base_stats: Stats) -> Self {
        Self {
            base_stats,
            unlocked_skill_slots: 0,
            equipped_skills: vec![None, None, None, None, None, None],
            equipped_items: HashMap::new(),
        }
    }

    pub fn unlock_skill_slots(&mut self, count: usize) {
        self.unlocked_skill_slots = count.min(self.equipped_skills.len());
    }

    pub fn equip_skill(&mut self, slot: usize, skill: SkillId) -> Result<(), HeroError> {
        if slot >= self.unlocked_skill_slots {
            return Err(HeroError::SkillSlotLocked { slot });
        }
        if slot >= self.equipped_skills.len() {
            return Err(HeroError::SkillSlotUnavailable { slot });
        }
        self.equipped_skills[slot] = Some(skill);
        Ok(())
    }

    pub fn equip_item(&mut self, item: ItemInstance) -> Result<(), HeroError> {
        self.equipped_items.insert(item.slot, item);
        Ok(())
    }

    pub fn equipped_item(&self, slot: GearSlot) -> Option<&ItemInstance> {
        self.equipped_items.get(&slot)
    }

    pub fn equipped_skill_ids(&self) -> impl Iterator<Item = SkillId> + '_ {
        self.equipped_skills.iter().filter_map(|skill| *skill)
    }

    pub fn derived_stats(&self) -> Stats {
        self.equipped_items
            .values()
            .fold(self.base_stats, |stats, item| {
                stats + item.stats + item.affix_stats()
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::items::{GearSlot, ItemAffix, ItemInstance, ItemRarity};
    use crate::domain::skills::SkillId;
    use crate::domain::stats::Stats;
    use std::panic::{catch_unwind, AssertUnwindSafe};

    #[test]
    fn derived_stats_include_base_stats_and_gear() {
        let mut hero = HeroProfile::new(Stats {
            max_health: 100,
            damage: 10,
            armor: 2,
            attack_speed: 1.0,
            healing_power: 0,
        });

        hero.unlock_skill_slots(2);
        hero.equip_skill(0, SkillId::LifestealStrike).unwrap();
        hero.equip_item(ItemInstance {
            id: 1,
            name: "Vampiric Sword".to_string(),
            slot: GearSlot::Weapon,
            rarity: ItemRarity::Uncommon,
            stats: Stats {
                max_health: 0,
                damage: 4,
                armor: 0,
                attack_speed: 0.2,
                healing_power: 0,
            },
            affixes: vec![ItemAffix::Vampiric],
        })
        .unwrap();

        let derived = hero.derived_stats();

        assert_eq!(derived.max_health, 100);
        assert_eq!(derived.damage, 14);
        assert_eq!(derived.armor, 2);
        assert_eq!(derived.healing_power, 2);
        assert!((derived.attack_speed - 1.2).abs() < f32::EPSILON);
    }

    #[test]
    fn locked_skill_slots_reject_skills() {
        let mut hero = HeroProfile::default();

        let result = hero.equip_skill(1, SkillId::Guard);

        assert_eq!(result, Err(HeroError::SkillSlotLocked { slot: 1 }));
    }

    #[test]
    fn invalid_skill_slot_state_returns_error_instead_of_panicking() {
        let mut hero = HeroProfile {
            base_stats: Stats::default(),
            unlocked_skill_slots: 3,
            equipped_skills: vec![None],
            equipped_items: HashMap::new(),
        };

        let result = catch_unwind(AssertUnwindSafe(|| hero.equip_skill(2, SkillId::Guard)));

        assert!(result.is_ok());
        assert!(result.unwrap().is_err());
    }

    #[test]
    fn gear_replaces_only_matching_slot() {
        let mut hero = HeroProfile::default();
        let armor = ItemInstance::basic(1, "Iron Armor", GearSlot::Armor);
        let weapon = ItemInstance::basic(2, "Iron Sword", GearSlot::Weapon);
        let replacement_armor = ItemInstance::basic(3, "Steel Armor", GearSlot::Armor);

        hero.equip_item(armor).unwrap();
        hero.equip_item(weapon.clone()).unwrap();
        hero.equip_item(replacement_armor.clone()).unwrap();

        assert_eq!(
            hero.equipped_item(GearSlot::Armor),
            Some(&replacement_armor)
        );
        assert_eq!(hero.equipped_item(GearSlot::Weapon), Some(&weapon));
        assert!(hero.equipped_item(GearSlot::Trinket).is_none());
    }
}
