use crate::domain::items::{GearSlot, ItemAffix, ItemInstance, ItemRarity};
use crate::domain::stats::Stats;
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;

/// Baseline affix pool (always eligible on loot).
const STANDARD_AFFIXES: [ItemAffix; 8] = [
    ItemAffix::Vampiric,
    ItemAffix::Heavy,
    ItemAffix::Cursed,
    ItemAffix::Spiked,
    ItemAffix::Relentless,
    ItemAffix::Shattering,
    ItemAffix::Virulent,
    ItemAffix::Bastion,
];

fn rarity_for_depth(depth: u32) -> ItemRarity {
    if depth >= 24 {
        ItemRarity::Legendary
    } else if depth >= 18 {
        ItemRarity::Epic
    } else if depth >= 8 {
        ItemRarity::Rare
    } else if depth >= 3 {
        ItemRarity::Uncommon
    } else {
        ItemRarity::Common
    }
}

fn affix_count_for_rarity(rarity: ItemRarity, rng: &mut ChaCha8Rng) -> usize {
    match rarity {
        ItemRarity::Common | ItemRarity::Uncommon => 1,
        ItemRarity::Rare => {
            if rng.gen_bool(0.38) {
                2
            } else {
                1
            }
        }
        ItemRarity::Epic => {
            if rng.gen_bool(0.72) {
                2
            } else {
                1
            }
        }
        ItemRarity::Legendary => 2,
    }
}

/// Higher weight = more likely when rolling loot for this slot.
fn affix_slot_weight(affix: ItemAffix, slot: GearSlot) -> u32 {
    let base = 2u32;
    let bonus = match (slot, affix) {
        (GearSlot::Weapon, ItemAffix::Shattering | ItemAffix::Heavy) => 5,
        (GearSlot::Weapon, ItemAffix::Vampiric | ItemAffix::Relentless) => 3,
        (GearSlot::Weapon, ItemAffix::Spiked | ItemAffix::Cursed) => 2,

        (GearSlot::Armor, ItemAffix::Bastion | ItemAffix::Spiked) => 5,
        (GearSlot::Armor, ItemAffix::Cursed | ItemAffix::Heavy) => 3,
        (GearSlot::Armor, ItemAffix::Shattering | ItemAffix::Virulent) => 1,

        (GearSlot::Trinket, ItemAffix::Virulent | ItemAffix::Vampiric) => 5,
        (GearSlot::Trinket, ItemAffix::Cursed | ItemAffix::Relentless) => 3,
        (GearSlot::Trinket, ItemAffix::Bastion | ItemAffix::Spiked) => 2,

        _ => 0,
    };
    base + bonus
}

fn affix_pick_table(slot: GearSlot, rarity: ItemRarity) -> Vec<(ItemAffix, u32)> {
    let mut v: Vec<(ItemAffix, u32)> = STANDARD_AFFIXES
        .iter()
        .copied()
        .map(|a| (a, affix_slot_weight(a, slot)))
        .collect();
    if rarity == ItemRarity::Legendary {
        v.push((ItemAffix::Devourer, 8));
        v.push((ItemAffix::TitansFury, 8));
    }
    v
}

fn pick_weighted(rng: &mut ChaCha8Rng, table: &[(ItemAffix, u32)]) -> ItemAffix {
    let total: u32 = table.iter().map(|(_, w)| w).sum();
    if total == 0 {
        return STANDARD_AFFIXES[0];
    }
    let mut roll = rng.gen_range(0..total);
    for &(a, w) in table {
        if roll < w {
            return a;
        }
        roll -= w;
    }
    table[0].0
}

fn roll_affixes(rng: &mut ChaCha8Rng, slot: GearSlot, rarity: ItemRarity) -> Vec<ItemAffix> {
    let want = affix_count_for_rarity(rarity, rng);
    let table = affix_pick_table(slot, rarity);
    let mut out = Vec::with_capacity(want);
    for _ in 0..64 {
        if out.len() >= want {
            break;
        }
        let a = pick_weighted(rng, &table);
        if !out.contains(&a) {
            out.push(a);
        }
    }
    while out.len() < want {
        out.push(pick_weighted(rng, &table));
    }
    out.truncate(want);
    out
}

fn affix_keyword(a: ItemAffix) -> &'static str {
    match a {
        ItemAffix::Vampiric => "Sanguine",
        ItemAffix::Heavy => "Weighted",
        ItemAffix::Cursed => "Hexed",
        ItemAffix::Spiked => "Barbed",
        ItemAffix::Relentless => "Relentless",
        ItemAffix::Shattering => "Shattering",
        ItemAffix::Virulent => "Virulent",
        ItemAffix::Bastion => "Bastion",
        ItemAffix::Devourer => "Devouring",
        ItemAffix::TitansFury => "Titan",
    }
}

fn gear_kind(slot: GearSlot) -> &'static str {
    match slot {
        GearSlot::Weapon => "Blade",
        GearSlot::Armor => "Mail",
        GearSlot::Trinket => "Charm",
    }
}

fn loot_display_name(rarity: ItemRarity, slot: GearSlot, affixes: &[ItemAffix]) -> String {
    let kind = gear_kind(slot);
    let tag = affixes
        .first()
        .copied()
        .map(affix_keyword)
        .unwrap_or("Plain");
    format!("{rarity:?} {tag} {kind}")
}

pub fn roll_loot(depth: u32, seed: u64) -> ItemInstance {
    let mut rng = ChaCha8Rng::seed_from_u64(seed ^ depth as u64);
    let slot = match rng.gen_range(0..3) {
        0 => GearSlot::Weapon,
        1 => GearSlot::Armor,
        _ => GearSlot::Trinket,
    };
    let rarity = rarity_for_depth(depth);
    let budget = depth as i32 + rarity_bonus(rarity);
    let affixes = roll_affixes(&mut rng, slot, rarity);

    ItemInstance {
        id: seed ^ ((depth as u64) << 32),
        name: loot_display_name(rarity, slot, &affixes),
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
        affixes,
    }
}

pub fn salvage_value(item: &ItemInstance) -> u32 {
    match item.rarity {
        ItemRarity::Common => 5,
        ItemRarity::Uncommon => 15,
        ItemRarity::Rare => 40,
        ItemRarity::Epic => 75,
        ItemRarity::Legendary => 130,
    }
}

fn rarity_bonus(rarity: ItemRarity) -> i32 {
    match rarity {
        ItemRarity::Common => 0,
        ItemRarity::Uncommon => 4,
        ItemRarity::Rare => 10,
        ItemRarity::Epic => 16,
        ItemRarity::Legendary => 24,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::items::{GearSlot, ItemAffix, ItemInstance, ItemRarity};

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
        let mut common = ItemInstance::basic(1, "Common Sword", GearSlot::Weapon);
        common.rarity = ItemRarity::Common;
        let mut uncommon = ItemInstance::basic(2, "Uncommon Sword", GearSlot::Weapon);
        uncommon.rarity = ItemRarity::Uncommon;
        let mut rare = ItemInstance::basic(3, "Rare Sword", GearSlot::Weapon);
        rare.rarity = ItemRarity::Rare;
        let mut epic = ItemInstance::basic(4, "Epic Sword", GearSlot::Weapon);
        epic.rarity = ItemRarity::Epic;
        let mut legendary = ItemInstance::basic(5, "Legendary Sword", GearSlot::Weapon);
        legendary.rarity = ItemRarity::Legendary;

        assert!(salvage_value(&uncommon) > salvage_value(&common));
        assert!(salvage_value(&rare) > salvage_value(&uncommon));
        assert!(salvage_value(&epic) > salvage_value(&rare));
        assert!(salvage_value(&legendary) > salvage_value(&epic));
    }

    #[test]
    fn rolled_items_use_mvp_gear_slots() {
        let item = roll_loot(3, 5);

        assert!(matches!(
            item.slot,
            GearSlot::Weapon | GearSlot::Armor | GearSlot::Trinket
        ));
    }

    #[test]
    fn very_deep_loot_can_be_legendary_with_two_affixes() {
        let item = roll_loot(30, 12345);
        assert_eq!(item.rarity, ItemRarity::Legendary);
        assert_eq!(item.affixes.len(), 2);
    }

    #[test]
    fn legendary_pick_table_includes_devourer_and_titans_fury() {
        let t = affix_pick_table(GearSlot::Weapon, ItemRarity::Legendary);
        assert!(t.iter().any(|(a, _)| *a == ItemAffix::Devourer));
        assert!(t.iter().any(|(a, _)| *a == ItemAffix::TitansFury));
    }

    #[test]
    fn non_legendary_table_excludes_legendary_only_affixes() {
        let t = affix_pick_table(GearSlot::Trinket, ItemRarity::Epic);
        assert!(!t.iter().any(|(a, _)| *a == ItemAffix::Devourer));
        assert!(!t.iter().any(|(a, _)| *a == ItemAffix::TitansFury));
    }
}
