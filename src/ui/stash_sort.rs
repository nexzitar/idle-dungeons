//! Stash list ordering helpers (persisted [`crate::save::StashSortOrder`] on [`crate::save::SaveProfile`]).

use crate::domain::items::{ItemInstance, ItemRarity};

pub use crate::save::StashSortOrder;

#[inline]
fn rarity_rank(r: ItemRarity) -> u8 {
    match r {
        ItemRarity::Common => 0,
        ItemRarity::Uncommon => 1,
        ItemRarity::Rare => 2,
        ItemRarity::Epic => 3,
        ItemRarity::Legendary => 4,
    }
}

/// Indices into `inventory` in the order rows should be shown.
pub fn stash_display_indices(inventory: &[ItemInstance], order: StashSortOrder) -> Vec<usize> {
    match order {
        StashSortOrder::Recent => (0..inventory.len()).rev().collect(),
        StashSortOrder::RarityName => {
            let mut ix: Vec<usize> = (0..inventory.len()).collect();
            ix.sort_by(|&i, &j| {
                let a = &inventory[i];
                let b = &inventory[j];
                rarity_rank(b.rarity)
                    .cmp(&rarity_rank(a.rarity))
                    .then_with(|| a.name.cmp(&b.name))
                    .then_with(|| a.id.cmp(&b.id))
            });
            ix
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::items::GearSlot;

    fn item(id: u64, name: &str, rarity: ItemRarity) -> ItemInstance {
        let mut it = ItemInstance::basic(id, name, GearSlot::MainHand);
        it.rarity = rarity;
        it
    }

    #[test]
    fn recent_reverses_vec_order() {
        let inv = vec![
            item(1, "A", ItemRarity::Common),
            item(2, "B", ItemRarity::Rare),
        ];
        let ix = stash_display_indices(&inv, StashSortOrder::Recent);
        assert_eq!(ix, vec![1, 0]);
    }

    #[test]
    fn rarity_name_sorts_rare_first_then_name() {
        let inv = vec![
            item(10, "Zebra", ItemRarity::Common),
            item(20, "Apple", ItemRarity::Rare),
            item(30, "Mango", ItemRarity::Rare),
            item(40, "Banana", ItemRarity::Uncommon),
        ];
        let ix = stash_display_indices(&inv, StashSortOrder::RarityName);
        assert_eq!(
            ix.iter().map(|&i| inv[i].id).collect::<Vec<_>>(),
            vec![20, 30, 40, 10]
        );
    }
}
