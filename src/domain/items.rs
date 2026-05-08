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
    /// Build-strong pairing of stats + mechanics; see [`ItemAffix`] synergies in combat.
    Epic,
    /// Run-defining combinations; usually two affixes from loot.
    Legendary,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ItemAffix {
    Vampiric,
    Heavy,
    Cursed,
    Spiked,
    Relentless,
    /// Hero swings treat enemy armor as 4 lower (minimum 0); see combat armor pierce.
    Shattering,
    /// With Poison Edge: +1 poison stack per hero hit compared to baseline (+3 vs +2).
    Virulent,
    /// With Guard: +2 flat damage reduction on blocked enemy swings.
    Bastion,
    /// **Legendary-only (loot):** when you land the killing blow, restore a slice of max HP.
    Devourer,
    /// **Legendary-only (loot):** while at or below half health, hero weapon hits deal ~25% more damage.
    TitansFury,
}

impl ItemAffix {
    /// Player-facing mechanical description (stats from [`ItemInstance::affix_stats`] are listed separately in UI).
    pub fn effect_description(self) -> &'static str {
        match self {
            ItemAffix::Vampiric => "Lifesteal strikes steal more health.",
            ItemAffix::Heavy => "Bonus weapon damage; slows attack speed.",
            ItemAffix::Cursed => "More armor; weakens barrier pulse size.",
            ItemAffix::Spiked => "Armor and thorns hits hit harder.",
            ItemAffix::Relentless => "Faster attack speed.",
            ItemAffix::Shattering => "Weapon hits ignore 4 enemy armor.",
            ItemAffix::Virulent => "With Poison Edge: +1 poison stack per attack.",
            ItemAffix::Bastion => "With Guard: +2 flat damage blocked per enemy hit.",
            ItemAffix::Devourer => "On kill: heal a portion of max HP.",
            ItemAffix::TitansFury => "At ≤50% HP: weapon hits deal +25% damage.",
        }
    }
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
                        ItemAffix::Shattering => Stats {
                            damage: 2,
                            ..Stats::default_zero()
                        },
                        ItemAffix::Virulent => Stats {
                            damage: 1,
                            healing_power: 1,
                            ..Stats::default_zero()
                        },
                        ItemAffix::Bastion => Stats {
                            armor: 3,
                            max_health: 8,
                            ..Stats::default_zero()
                        },
                        ItemAffix::Devourer => Stats {
                            damage: 4,
                            max_health: 14,
                            ..Stats::default_zero()
                        },
                        ItemAffix::TitansFury => Stats {
                            damage: 3,
                            armor: 2,
                            ..Stats::default_zero()
                        },
                    }
            })
    }
}
