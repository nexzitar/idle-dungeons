use crate::domain::hero::HeroProfile;

pub fn build_panel_text(hero: &HeroProfile) -> String {
    let stats = hero.derived_stats();
    format!(
        "Skill Slots: {}/{}\nMax Health: {}\nDamage: {}\nArmor: {}",
        hero.unlocked_skill_slots,
        hero.equipped_skills.len(),
        stats.max_health,
        stats.damage,
        stats.armor
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::hero::HeroProfile;

    #[test]
    fn build_panel_text_lists_skill_slots_and_stats() {
        let hero = HeroProfile::default();

        let text = build_panel_text(&hero);

        assert!(text.contains("Skill Slots"));
        assert!(text.contains("Max Health"));
    }
}
