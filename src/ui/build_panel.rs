use crate::domain::hero::HeroProfile;
use crate::domain::skill_layering::layering_warnings;
use crate::domain::skills::{skill_category, skill_definition};

pub fn build_panel_text(hero: &HeroProfile) -> String {
    let stats = hero.derived_stats();
    let layers = layering_warnings(hero.equipped_skill_ids());
    let layer_note = if layers.is_empty() {
        String::new()
    } else {
        format!("\n\n{}", layers.join("\n"))
    };
    let mut loadout = String::new();
    let n = hero
        .unlocked_skill_slots
        .min(hero.equipped_skills.len())
        .max(0);
    for i in 0..n {
        let line = match hero.equipped_skills.get(i).and_then(|x| *x) {
            Some(id) => {
                let d = skill_definition(id);
                let c = skill_category(id);
                format!(
                    "  {}. {} [{} — {}]",
                    i + 1,
                    d.name,
                    c.category_abbr(),
                    c.display_label()
                )
            }
            None => format!("  {}. (empty slot)", i + 1),
        };
        loadout.push_str(&line);
        if i + 1 < n {
            loadout.push('\n');
        }
    }
    let loadout_block = if loadout.is_empty() {
        String::new()
    } else {
        format!("\n\nLoadout:\n{loadout}")
    };
    format!(
        "Skill Slots: {}/{}\nMax Health: {}\nDamage: {}\nArmor: {}{}{}",
        hero.unlocked_skill_slots,
        hero.equipped_skills.len(),
        stats.max_health,
        stats.damage,
        stats.armor,
        loadout_block,
        layer_note
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::hero::HeroProfile;

    #[test]
    fn build_panel_text_lists_skill_slots_and_stats() {
        let mut hero = HeroProfile::default();
        hero.unlock_skill_slots(1);
        hero.assign_skill_to_slot(0, Some(crate::domain::skills::SkillId::Guard))
            .unwrap();

        let text = build_panel_text(&hero);

        assert!(text.contains("Skill Slots"));
        assert!(text.contains("Max Health"));
        assert!(text.contains("Loadout:"));
        assert!(
            text.contains("Guard") || text.contains("Rxn"),
            "expected category chip or skill name in {}",
            text
        );
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
        assert!(text.contains("Cleave") && text.contains("Heavy"), "{text}");
    }
}
