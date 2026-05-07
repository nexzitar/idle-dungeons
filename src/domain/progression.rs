use crate::domain::stats::Stats;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum UpgradeId {
    MaxHealth,
    BaseDamage,
    Armor,
    HealingPower,
    GoldGain,
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ProgressionError {
    #[error("not enough gold: required {required}, available {available}")]
    NotEnoughGold { required: u32, available: u32 },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MetaProgression {
    pub gold: u32,
    pub salvage: u32,
    pub unlocked_skill_slots: usize,
    pub skill_slot_progress: u32,
    upgrades: HashMap<UpgradeId, u32>,
}

impl Default for MetaProgression {
    fn default() -> Self {
        Self {
            gold: 0,
            salvage: 0,
            unlocked_skill_slots: 2,
            skill_slot_progress: 0,
            upgrades: HashMap::new(),
        }
    }
}

impl MetaProgression {
    pub fn upgrade_level(&self, upgrade: UpgradeId) -> u32 {
        *self.upgrades.get(&upgrade).unwrap_or(&0)
    }

    pub fn stat_bonus(&self) -> Stats {
        Stats {
            max_health: self.upgrade_level(UpgradeId::MaxHealth) as i32 * 10,
            damage: self.upgrade_level(UpgradeId::BaseDamage) as i32 * 2,
            armor: self.upgrade_level(UpgradeId::Armor) as i32,
            attack_speed: 0.0,
            healing_power: self.upgrade_level(UpgradeId::HealingPower) as i32,
        }
    }

    pub fn upgrade_cost(&self, upgrade: UpgradeId) -> u32 {
        10 + self.upgrade_level(upgrade) * 5
    }

    pub fn buy_upgrade(&mut self, upgrade: UpgradeId) -> Result<(), ProgressionError> {
        let cost = self.upgrade_cost(upgrade);
        if self.gold < cost {
            return Err(ProgressionError::NotEnoughGold {
                required: cost,
                available: self.gold,
            });
        }
        self.gold -= cost;
        *self.upgrades.entry(upgrade).or_insert(0) += 1;
        Ok(())
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn upgrades_spend_gold_and_increase_level() {
        let mut profile = MetaProgression {
            gold: 100,
            ..MetaProgression::default()
        };

        profile.buy_upgrade(UpgradeId::BaseDamage).unwrap();

        assert_eq!(profile.gold, 90);
        assert_eq!(profile.upgrade_level(UpgradeId::BaseDamage), 1);
    }

    #[test]
    fn cannot_buy_upgrade_without_enough_gold() {
        let mut profile = MetaProgression::default();

        let result = profile.buy_upgrade(UpgradeId::BaseDamage);

        assert_eq!(
            result,
            Err(ProgressionError::NotEnoughGold {
                required: 10,
                available: 0
            })
        );
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
    fn upgrade_levels_produce_stat_bonuses() {
        let mut profile = MetaProgression {
            gold: 100,
            ..MetaProgression::default()
        };

        profile.buy_upgrade(UpgradeId::MaxHealth).unwrap();
        profile.buy_upgrade(UpgradeId::BaseDamage).unwrap();
        profile.buy_upgrade(UpgradeId::Armor).unwrap();
        profile.buy_upgrade(UpgradeId::HealingPower).unwrap();

        let bonus = profile.stat_bonus();

        assert_eq!(bonus.max_health, 10);
        assert_eq!(bonus.damage, 2);
        assert_eq!(bonus.armor, 1);
        assert_eq!(bonus.healing_power, 1);
    }
}
