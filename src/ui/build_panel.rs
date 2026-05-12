use crate::domain::hero::HeroProfile;
use crate::domain::skill_layering::layering_warnings;

pub fn build_panel_text(hero: &HeroProfile) -> String {
    let stats = hero.derived_stats();
    let layers = layering_warnings(hero.equipped_skill_ids());
    let layer_note = if layers.is_empty() {
        String::new()
    } else {
        format!("\n\n{}", layers.join("\n"))
    };
    format!(
        "Skill Slots: {}/{}\nMax Health: {}\nDamage: {}\nArmor: {}{}",
        hero.unlocked_skill_slots,
        hero.equipped_skills.len(),
        stats.max_health,
        stats.damage,
        stats.armor,
        layer_note
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

    #[test]
    fn build_panel_shows_layering_when_heavy_and_cleave() {
        let mut hero = HeroProfile::default();
        hero.unlock_skill_slots(2);
        hero.equip_skill(0, crate::domain::skills::SkillId::HeavyStrike)
            .unwrap();
        hero.equip_skill(1, crate::domain::skills::SkillId::Cleave)
            .unwrap();
        let text = build_panel_text(&hero);
        assert!(
            text.contains("Cleave") && text.contains("Heavy"),
            "{text}"
        );
    }
}
