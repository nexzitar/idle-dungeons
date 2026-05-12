//! Mutually exclusive skill **layer slots** for loadout clarity (see `docs/superpowers/ACTIVE-REMAINING-WORK.md` Wave 2).
//!
//! Combat may still resolve overlaps deterministically (e.g. Cleave wins over Heavy in [`crate::domain::combat::prepare_hero_combat`]);
//! this module exposes **player-facing** warnings so builds do not silently waste a slot.

use crate::domain::skills::SkillId;

/// Layer keys: at most one skill per slot should be “effective” for that rules package.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SkillLayerSlot {
    /// Heavy Strike vs Cleave: both use merged swing cadence; Cleave takes precedence in sim.
    PrimaryWeaponPattern,
}

/// Which overlap bucket a skill occupies, if any.
pub fn skill_layer_slot(id: SkillId) -> Option<SkillLayerSlot> {
    match id {
        SkillId::HeavyStrike | SkillId::Cleave => Some(SkillLayerSlot::PrimaryWeaponPattern),
        _ => None,
    }
}

/// Static notices when the loadout duplicates a layer (the “masked” skill still occupies a slot but does not apply that package).
pub fn layering_warnings(equipped: impl IntoIterator<Item = SkillId>) -> Vec<&'static str> {
    let list: Vec<SkillId> = equipped.into_iter().collect();
    let mut out = Vec::new();
    if list.contains(&SkillId::HeavyStrike) && list.contains(&SkillId::Cleave) {
        out.push(
            "Layering: Cleave overrides Heavy Strike for heavy-style swing damage and yellow credit.",
        );
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::hero::HeroProfile;

    #[test]
    fn heavy_and_cleave_emits_warning() {
        let w = layering_warnings([SkillId::HeavyStrike, SkillId::Cleave]);
        assert_eq!(w.len(), 1);
        assert!(w[0].contains("Cleave"));
    }

    #[test]
    fn cleave_only_no_warning() {
        assert!(layering_warnings([SkillId::Cleave]).is_empty());
    }

    #[test]
    fn hero_default_no_layering_warning() {
        let hero = HeroProfile::default();
        let w = layering_warnings(hero.equipped_skill_ids());
        assert!(w.is_empty());
    }
}
