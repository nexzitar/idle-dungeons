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

/// Roll item rarity from **dungeon depth** and RNG (same seed + depth = same tier).
///
/// - **Legendary** starts around **1%** at shallow depths and rises slowly with depth;
///   it is **not** guaranteed before depth **1000**.
/// - At depth **≥ 1000**, loot is **always** Legendary.
/// - Below 1000, non-legendary rolls bias toward Common/Uncommon early and Rare/Epic deeper.
fn roll_rarity_for_depth(depth: u32, rng: &mut ChaCha8Rng) -> ItemRarity {
    if depth >= 1000 {
        return ItemRarity::Legendary;
    }
    let t = if depth <= 1 {
        0.0
    } else {
        ((depth - 1) as f64 / 999.0).clamp(0.0, 1.0)
    };
    // ~1% at depth 1, ~99% at depth 999 (never 100% before floor 1000).
    let p_legendary = 0.01 + 0.98 * t * t;
    let r1: f64 = rng.gen();
    if r1 < p_legendary {
        return ItemRarity::Legendary;
    }

    let w_common = (1.0 - t).powf(2.5);
    let w_uncommon = (1.0 - t).max(0.01) * (0.35 + 0.4 * t);
    let w_rare = t * t * 1.4 + 0.05;
    let w_epic = t * t * t * 2.5 + 0.02 * t;
    let sum = w_common + w_uncommon + w_rare + w_epic;
    let r2: f64 = rng.gen();
    let c0 = w_common / sum;
    let c1 = c0 + w_uncommon / sum;
    let c2 = c1 + w_rare / sum;
    if r2 < c0 {
        ItemRarity::Common
    } else if r2 < c1 {
        ItemRarity::Uncommon
    } else if r2 < c2 {
        ItemRarity::Rare
    } else {
        ItemRarity::Epic
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

fn affix_prefix_word(a: ItemAffix) -> &'static str {
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

/// Short suffix after "of …" for a second affix (flavor, e.g. Spiked → Thorns).
fn affix_of_suffix(a: ItemAffix) -> &'static str {
    match a {
        ItemAffix::Vampiric => "Blood",
        ItemAffix::Heavy => "Weight",
        ItemAffix::Cursed => "Hexes",
        ItemAffix::Spiked => "Thorns",
        ItemAffix::Relentless => "Pursuit",
        ItemAffix::Shattering => "Shattering",
        ItemAffix::Virulent => "Venom",
        ItemAffix::Bastion => "Warding",
        ItemAffix::Devourer => "Feasting",
        ItemAffix::TitansFury => "Titans",
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
    match affixes.len() {
        0 => format!("{rarity:?} Plain {kind}"),
        1 => {
            let a = affixes[0];
            format!("{rarity:?} {} {kind}", affix_prefix_word(a))
        }
        _ => {
            let a0 = affixes[0];
            let a1 = affixes[1];
            format!(
                "{rarity:?} {} {kind} of {}",
                affix_prefix_word(a0),
                affix_of_suffix(a1)
            )
        }
    }
}

fn item_from_slot_and_affixes(
    depth: u32,
    id_seed: u64,
    slot: GearSlot,
    rarity: ItemRarity,
    affixes: Vec<ItemAffix>,
) -> ItemInstance {
    let budget = depth as i32 + rarity_bonus(rarity);
    ItemInstance {
        id: id_seed ^ ((depth as u64) << 32),
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

/// Roll loot with a locked gear slot — used for onboarding salvage pacing.
pub fn roll_loot_for_slot(depth: u32, seed: u64, slot: GearSlot) -> ItemInstance {
    let lane = match slot {
        GearSlot::Weapon => 0xD15EA5Du64,
        GearSlot::Armor => 0xBAD00D7Au64,
        GearSlot::Trinket => 0x731B1EFu64,
    };
    let lane_shift = lane.rotate_left(slot as u32);
    let mut rng = ChaCha8Rng::seed_from_u64(seed.wrapping_add(depth as u64) ^ lane_shift);
    let rarity = roll_rarity_for_depth(depth, &mut rng);
    let affixes = roll_affixes(&mut rng, slot, rarity);
    let id_seed = seed ^ lane_shift;
    item_from_slot_and_affixes(depth, id_seed, slot, rarity, affixes)
}

pub fn roll_loot(depth: u32, seed: u64) -> ItemInstance {
    let mut rng = ChaCha8Rng::seed_from_u64(seed ^ depth as u64);
    let slot = match rng.gen_range(0..3) {
        0 => GearSlot::Weapon,
        1 => GearSlot::Armor,
        _ => GearSlot::Trinket,
    };
    let rarity = roll_rarity_for_depth(depth, &mut rng);
    let affixes = roll_affixes(&mut rng, slot, rarity);
    item_from_slot_and_affixes(depth, seed, slot, rarity, affixes)
}

/// Profile milestone for guaranteed combat salvage before depth **10**:
/// first **weapon**, second **armor**; later uses [`roll_loot`] with salted RNG so repeats are not clones.
pub fn roll_profile_guided_early_combat_drop(
    depth: u32,
    run_seed: u64,
    room_depth: u32,
    profile_claim_count_before_grant: u32,
) -> ItemInstance {
    let salt = run_seed
        .wrapping_mul(31_337)
        .wrapping_add(u64::from(room_depth).wrapping_mul(17))
        .wrapping_add(
            u64::from(profile_claim_count_before_grant)
                .rotate_left(7)
                .wrapping_mul(0x9E37_79B97F4A7C15),
        );
    match profile_claim_count_before_grant {
        0 => roll_loot_for_slot(depth, salt, GearSlot::Weapon),
        1 => roll_loot_for_slot(depth, salt, GearSlot::Armor),
        _ => roll_loot(depth, salt),
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
    fn guided_early_combat_first_weapon_second_armor() {
        let w = roll_profile_guided_early_combat_drop(1, 99, 1, 0);
        let a = roll_profile_guided_early_combat_drop(1, 99, 1, 1);
        assert_eq!(w.slot, GearSlot::Weapon);
        assert_eq!(a.slot, GearSlot::Armor);
    }

    #[test]
    fn guided_early_claim_tiers_are_not_duplicate_items_same_seed() {
        let first = roll_profile_guided_early_combat_drop(3, 1, 3, 0);
        let second = roll_profile_guided_early_combat_drop(3, 1, 3, 1);
        assert_ne!(
            first, second,
            "milestones differ while MVP run seed stays fixed"
        );
    }

    #[test]
    fn deeper_loot_has_at_least_as_much_stat_budget() {
        let shallow = roll_loot_for_slot(100, 42, GearSlot::Weapon);
        let deep = roll_loot_for_slot(200, 42, GearSlot::Weapon);

        assert!(deep.stats.damage >= shallow.stats.damage);
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
    fn floor_1000_loot_is_always_legendary_with_two_affixes() {
        for seed in 0..24u64 {
            let item = roll_loot(1000, seed);
            assert_eq!(item.rarity, ItemRarity::Legendary, "seed {seed}");
            assert_eq!(item.affixes.len(), 2, "seed {seed}");
        }
    }

    #[test]
    fn shallow_depth_legendary_is_possible_but_not_guaranteed() {
        let mut any_legendary = false;
        let mut all_legendary = true;
        for seed in 0..3000u64 {
            let is_leg = roll_loot(10, seed).rarity == ItemRarity::Legendary;
            any_legendary |= is_leg;
            all_legendary &= is_leg;
        }
        assert!(
            !all_legendary,
            "legendary must not be guaranteed below depth 1000"
        );
        assert!(
            any_legendary,
            "legendary should sometimes roll at shallow depth"
        );
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
