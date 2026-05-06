use crate::domain::items::{GearSlot, ItemAffix, ItemInstance, ItemRarity};
use crate::domain::stats::Stats;
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;

pub fn roll_loot(depth: u32, seed: u64) -> ItemInstance {
    let mut rng = ChaCha8Rng::seed_from_u64(seed ^ depth as u64);
    let slot = match rng.gen_range(0..3) {
        0 => GearSlot::Weapon,
        1 => GearSlot::Armor,
        _ => GearSlot::Trinket,
    };
    let rarity = if depth >= 18 {
        ItemRarity::Rare
    } else if depth >= 8 {
        ItemRarity::Uncommon
    } else {
        ItemRarity::Common
    };
    let budget = depth as i32 + rarity_bonus(rarity);
    let affix = match rng.gen_range(0..5) {
        0 => ItemAffix::Vampiric,
        1 => ItemAffix::Heavy,
        2 => ItemAffix::Cursed,
        3 => ItemAffix::Spiked,
        _ => ItemAffix::Relentless,
    };

    ItemInstance {
        id: seed ^ ((depth as u64) << 32),
        name: format!("{rarity:?} {slot:?}"),
        slot,
        rarity,
        stats: match slot {
            GearSlot::Weapon => Stats {
                damage: budget,
                ..Stats::default_zero()
            },
            GearSlot::Armor => Stats {
                armor: budget / 2,
                max_health: budget * 4,
                ..Stats::default_zero()
            },
            GearSlot::Trinket => Stats {
                healing_power: budget / 2,
                attack_speed: 0.1,
                ..Stats::default_zero()
            },
        },
        affixes: vec![affix],
    }
}

pub fn salvage_value(item: &ItemInstance) -> u32 {
    match item.rarity {
        ItemRarity::Common => 5,
        ItemRarity::Uncommon => 15,
        ItemRarity::Rare => 40,
    }
}

fn rarity_bonus(rarity: ItemRarity) -> i32 {
    match rarity {
        ItemRarity::Common => 0,
        ItemRarity::Uncommon => 4,
        ItemRarity::Rare => 10,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::items::GearSlot;

    #[test]
    fn loot_rolls_are_repeatable_for_seed_and_depth() {
        let first = roll_loot(10, 99);
        let second = roll_loot(10, 99);

        assert_eq!(first, second);
    }

    #[test]
    fn deeper_loot_has_at_least_as_much_stat_budget() {
        let shallow = roll_loot(1, 42);
        let deep = roll_loot(20, 42);

        assert!(
            deep.stats.damage + deep.stats.armor + deep.stats.max_health
                >= shallow.stats.damage + shallow.stats.armor + shallow.stats.max_health
        );
    }

    #[test]
    fn salvage_value_scales_by_rarity() {
        let item = roll_loot(10, 123);

        assert!(salvage_value(&item) > 0);
    }

    #[test]
    fn rolled_items_use_mvp_gear_slots() {
        let item = roll_loot(3, 5);

        assert!(matches!(
            item.slot,
            GearSlot::Weapon | GearSlot::Armor | GearSlot::Trinket
        ));
    }
}
