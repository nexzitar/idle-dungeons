use crate::domain::items::{GearSlot, ItemAffix, ItemInstance};
use crate::domain::skills::{skill_definition, SkillId, SkillKind};
use crate::domain::stats::Stats;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub const DEFAULT_HERO_NAME: &str = "Adventurer";

/// Max length (Unicode scalar count) for persisted hero names from the build UI.
pub const MAX_HERO_NAME_LEN: usize = 32;

fn default_hero_name() -> String {
    DEFAULT_HERO_NAME.to_string()
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum HeroError {
    #[error("skill slot {slot} is locked")]
    SkillSlotLocked { slot: usize },
    #[error("skill slot {slot} is unavailable")]
    SkillSlotUnavailable { slot: usize },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HeroProfile {
    /// Character name (party combat log / threat UI). Persisted; defaults for older saves.
    #[serde(default = "default_hero_name")]
    pub name: String,
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
            name: default_hero_name(),
            base_stats,
            unlocked_skill_slots: 0,
            equipped_skills: vec![None, None, None, None, None, None],
            equipped_items: HashMap::new(),
        }
    }

    pub fn unlock_skill_slots(&mut self, count: usize) {
        self.unlocked_skill_slots = count.min(self.equipped_skills.len());
    }

    pub fn assign_skill_to_slot(
        &mut self,
        slot: usize,
        skill: Option<SkillId>,
    ) -> Result<(), HeroError> {
        if slot >= self.unlocked_skill_slots {
            return Err(HeroError::SkillSlotLocked { slot });
        }
        if slot >= self.equipped_skills.len() {
            return Err(HeroError::SkillSlotUnavailable { slot });
        }
        if let Some(s) = skill {
            for i in 0..self.equipped_skills.len() {
                if i != slot && self.equipped_skills.get(i) == Some(&Some(s)) {
                    self.equipped_skills[i] = None;
                }
            }
            self.equipped_skills[slot] = Some(s);
            Ok(())
        } else {
            self.equipped_skills[slot] = None;
            Ok(())
        }
    }

    fn passive_skill_stat_bonus(&self) -> Stats {
        let mut b = Stats::default_zero();
        for id in self.equipped_skill_ids() {
            if skill_definition(id).kind != SkillKind::Passive {
                continue;
            }
            match id {
                SkillId::ThickHide => b.armor += 3,
                SkillId::ArcaneOverflow => b.healing_power += 3,
                SkillId::Berserker => {
                    b.damage += 4;
                    b.armor -= 2;
                }
                SkillId::SwiftStrikes => b.attack_speed += 0.08,
                SkillId::IronWill => b.max_health += 5,
                SkillId::BattleFocus => b.damage += 2,
                SkillId::CautiousAdvance => {
                    b.armor += 4;
                    b.damage -= 1;
                }
                _ => {}
            }
        }
        b
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

    pub fn clear_skill_slot(&mut self, slot: usize) -> Result<(), HeroError> {
        if slot >= self.unlocked_skill_slots {
            return Err(HeroError::SkillSlotLocked { slot });
        }
        if slot >= self.equipped_skills.len() {
            return Err(HeroError::SkillSlotUnavailable { slot });
        }
        self.equipped_skills[slot] = None;
        Ok(())
    }

    /// Equip `item`, returning any pieces **removed** from the loadout (same-slot replace,
    /// off-hand bumped by a new two-hander, or two-hander bumped by a new off-hand).
    pub fn equip_item(&mut self, item: ItemInstance) -> Vec<ItemInstance> {
        let mut displaced: Vec<ItemInstance> = Vec::new();
        debug_assert!(
            !item.two_handed || item.slot == GearSlot::MainHand,
            "two_handed items must use MainHand slot"
        );

        match item.slot {
            GearSlot::MainHand => {
                if item.two_handed {
                    if let Some(off) = self.equipped_items.remove(&GearSlot::OffHand) {
                        displaced.push(off);
                    }
                }
                if let Some(old) = self.equipped_items.insert(GearSlot::MainHand, item) {
                    displaced.push(old);
                }
            }
            GearSlot::OffHand => {
                if let Some(mh) = self.equipped_items.get(&GearSlot::MainHand) {
                    if mh.two_handed {
                        if let Some(two_h) = self.equipped_items.remove(&GearSlot::MainHand) {
                            displaced.push(two_h);
                        }
                    }
                }
                if let Some(old) = self.equipped_items.insert(GearSlot::OffHand, item) {
                    displaced.push(old);
                }
            }
            _ => {
                if let Some(old) = self.equipped_items.insert(item.slot, item) {
                    displaced.push(old);
                }
            }
        }
        displaced
    }

    pub fn equipped_item(&self, slot: GearSlot) -> Option<&ItemInstance> {
        self.equipped_items.get(&slot)
    }

    pub fn has_affix(&self, affix: ItemAffix) -> bool {
        self.equipped_items
            .values()
            .any(|item| item.affixes.iter().any(|a| *a == affix))
    }

    pub fn equipped_skill_ids(&self) -> impl Iterator<Item = SkillId> + '_ {
        self.equipped_skills.iter().filter_map(|skill| *skill)
    }

    pub fn derived_stats(&self) -> Stats {
        let gear = self
            .equipped_items
            .values()
            .fold(Stats::default_zero(), |stats, item| {
                stats + item.stats + item.affix_stats()
            });
        self.base_stats + gear + self.passive_skill_stat_bonus()
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
    fn has_affix_detects_gear_affix() {
        let mut hero = HeroProfile::default();
        let mut blade = ItemInstance::basic(1, "Test", GearSlot::MainHand);
        blade.affixes.push(ItemAffix::Vampiric);
        hero.equip_item(blade);
        assert!(hero.has_affix(ItemAffix::Vampiric));
        assert!(!hero.has_affix(ItemAffix::Heavy));
    }

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
            slot: GearSlot::MainHand,
            rarity: ItemRarity::Uncommon,
            stats: Stats {
                max_health: 0,
                damage: 4,
                armor: 0,
                attack_speed: 0.2,
                healing_power: 0,
            },
            affixes: vec![ItemAffix::Vampiric],
            two_handed: false,
        });

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
    fn clear_skill_slot_empties_slot() {
        let mut hero = HeroProfile::default();
        hero.unlock_skill_slots(1);
        hero.equip_skill(0, SkillId::Guard).unwrap();
        hero.clear_skill_slot(0).unwrap();
        assert_eq!(hero.equipped_skills[0], None);
    }

    #[test]
    fn invalid_skill_slot_state_returns_error_instead_of_panicking() {
        let mut hero = HeroProfile {
            name: default_hero_name(),
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
    fn assign_skill_to_slot_clears_duplicate_elsewhere() {
        let mut hero = HeroProfile::default();
        hero.unlock_skill_slots(2);
        hero.assign_skill_to_slot(0, Some(SkillId::Guard)).unwrap();
        hero.assign_skill_to_slot(1, Some(SkillId::Guard)).unwrap();
        assert_eq!(hero.equipped_skills[0], None);
        assert_eq!(hero.equipped_skills[1], Some(SkillId::Guard));
    }

    #[test]
    fn passive_thick_hide_adds_armor_to_derived_stats() {
        let mut hero = HeroProfile::new(Stats {
            armor: 1,
            ..Stats::default()
        });
        hero.unlock_skill_slots(1);
        hero.assign_skill_to_slot(0, Some(SkillId::ThickHide))
            .unwrap();
        assert_eq!(hero.derived_stats().armor, 4);
    }

    #[test]
    fn gear_replaces_only_matching_slot() {
        let mut hero = HeroProfile::default();
        let armor = ItemInstance::basic(1, "Iron Armor", GearSlot::Chest);
        let weapon = ItemInstance::basic(2, "Iron Sword", GearSlot::MainHand);
        let replacement_armor = ItemInstance::basic(3, "Steel Armor", GearSlot::Chest);

        hero.equip_item(armor);
        hero.equip_item(weapon.clone());
        hero.equip_item(replacement_armor.clone());

        assert_eq!(
            hero.equipped_item(GearSlot::Chest),
            Some(&replacement_armor)
        );
        assert_eq!(hero.equipped_item(GearSlot::MainHand), Some(&weapon));
        assert!(hero.equipped_item(GearSlot::Trinket1).is_none());
    }

    #[test]
    fn two_handed_main_displaces_off_hand() {
        let mut hero = HeroProfile::default();
        let shield = ItemInstance::basic(1, "Shield", GearSlot::OffHand);
        let pole = ItemInstance {
            two_handed: true,
            ..ItemInstance::basic(2, "Greatstaff", GearSlot::MainHand)
        };
        hero.equip_item(shield);
        let displaced = hero.equip_item(pole.clone());
        assert_eq!(displaced.len(), 1);
        assert_eq!(displaced[0].id, 1);
        assert_eq!(hero.equipped_item(GearSlot::MainHand), Some(&pole));
        assert!(hero.equipped_item(GearSlot::OffHand).is_none());
    }

    #[test]
    fn off_hand_displaces_two_handed_main() {
        let mut hero = HeroProfile::default();
        let pole = ItemInstance {
            two_handed: true,
            ..ItemInstance::basic(2, "Greatstaff", GearSlot::MainHand)
        };
        let shield = ItemInstance::basic(1, "Shield", GearSlot::OffHand);
        hero.equip_item(pole.clone());
        let displaced = hero.equip_item(shield.clone());
        assert_eq!(displaced.len(), 1);
        assert_eq!(displaced[0].id, 2);
        assert_eq!(hero.equipped_item(GearSlot::OffHand), Some(&shield));
        assert!(hero.equipped_item(GearSlot::MainHand).is_none());
    }
}
