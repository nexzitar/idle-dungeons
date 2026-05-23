use crate::domain::hero::HeroProfile;
use crate::domain::skill_layering::layering_warnings;

/// Player-facing loadout overlap notices for camp / build UI.
pub fn hero_layering_warnings(hero: &HeroProfile) -> Vec<&'static str> {
    layering_warnings(hero.equipped_skill_ids())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::hero::HeroProfile;
    use crate::domain::skills::SkillId;

    #[test]
    fn hero_layering_warnings_empty_for_default_loadout() {
        let hero = HeroProfile::default();
        assert!(hero_layering_warnings(&hero).is_empty());
    }

    #[test]
    fn hero_layering_warnings_when_heavy_and_cleave() {
        let mut hero = HeroProfile::default();
        hero.unlock_skill_slots(2);
        hero.equip_skill(0, SkillId::HeavyStrike).unwrap();
        hero.equip_skill(1, SkillId::Cleave).unwrap();
        let warnings = hero_layering_warnings(&hero);
        assert_eq!(warnings.len(), 1);
        assert!(warnings[0].contains("Cleave") && warnings[0].contains("Heavy"));
    }
}
