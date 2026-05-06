use crate::domain::stats::Stats;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum GearSlot {
    Weapon,
    Armor,
    Trinket,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ItemRarity {
    Common,
    Uncommon,
    Rare,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ItemAffix {
    Vampiric,
    Heavy,
    Cursed,
    Spiked,
    Relentless,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ItemInstance {
    pub id: u64,
    pub name: String,
    pub slot: GearSlot,
    pub rarity: ItemRarity,
    pub stats: Stats,
    pub affixes: Vec<ItemAffix>,
}

impl ItemInstance {
    pub fn basic(id: u64, name: impl Into<String>, slot: GearSlot) -> Self {
        Self {
            id,
            name: name.into(),
            slot,
            rarity: ItemRarity::Common,
            stats: Stats::default_zero(),
            affixes: Vec::new(),
        }
    }

    pub fn affix_stats(&self) -> Stats {
        self.affixes
            .iter()
            .fold(Stats::default_zero(), |stats, affix| {
                stats
                    + match affix {
                        ItemAffix::Vampiric => Stats {
                            healing_power: 2,
                            ..Stats::default_zero()
                        },
                        ItemAffix::Heavy => Stats {
                            damage: 4,
                            attack_speed: -0.2,
                            ..Stats::default_zero()
                        },
                        ItemAffix::Cursed => Stats {
                            armor: 5,
                            healing_power: -2,
                            ..Stats::default_zero()
                        },
                        ItemAffix::Spiked => Stats {
                            armor: 2,
                            damage: 1,
                            ..Stats::default_zero()
                        },
                        ItemAffix::Relentless => Stats {
                            attack_speed: 0.3,
                            ..Stats::default_zero()
                        },
                    }
            })
    }
}
