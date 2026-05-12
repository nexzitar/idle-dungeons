//! Emergent **combat archetype hints** from loadout (skills + gear affixes).
//!
//! Not character classes: the same hero can match several hints. Used for light loot biasing and future telemetry (Wave 5).

use crate::domain::hero::HeroProfile;
use crate::domain::items::ItemAffix;
use crate::domain::skills::SkillId;

/// Build-derived tag for loot weight nudges and run analytics (no class lock-in).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum CombatArchetypeHint {
    ThreatAnchor,
    PoisonRamping,
    HeavyWeave,
    CleaveAoE,
    InstantStrike,
    ReactiveThorns,
    SustainStrike,
    BurstStrike,
    WardFocused,
}

/// Heuristic hints for one hero sheet (stable sort order, deduped).
pub fn hints_for_hero(hero: &HeroProfile) -> Vec<CombatArchetypeHint> {
    let mut v: Vec<CombatArchetypeHint> = Vec::new();
    let skills: Vec<SkillId> = hero.equipped_skill_ids().collect();
    let has_skill = |id: SkillId| skills.iter().any(|&s| s == id);

    if has_skill(SkillId::Guard) && has_skill(SkillId::ThickHide) {
        v.push(CombatArchetypeHint::ThreatAnchor);
        v.push(CombatArchetypeHint::WardFocused);
    }
    if has_skill(SkillId::Taunt) {
        v.push(CombatArchetypeHint::ThreatAnchor);
    }
    if has_skill(SkillId::PoisonEdge) {
        v.push(CombatArchetypeHint::PoisonRamping);
    }
    if has_skill(SkillId::HeavyStrike) {
        v.push(CombatArchetypeHint::HeavyWeave);
    }
    if has_skill(SkillId::Cleave) {
        v.push(CombatArchetypeHint::CleaveAoE);
    }
    if has_skill(SkillId::VictoryRush) {
        v.push(CombatArchetypeHint::InstantStrike);
    }
    if has_skill(SkillId::ThornSkin) {
        v.push(CombatArchetypeHint::ReactiveThorns);
    }
    if has_skill(SkillId::BarrierPulse) {
        v.push(CombatArchetypeHint::WardFocused);
    }
    if hero.has_affix(ItemAffix::Spiked) {
        v.push(CombatArchetypeHint::ReactiveThorns);
    }
    if hero.has_affix(ItemAffix::Vampiric) || hero.has_affix(ItemAffix::Devourer) {
        v.push(CombatArchetypeHint::SustainStrike);
    }
    if hero.has_affix(ItemAffix::Relentless) || hero.has_affix(ItemAffix::TitansFury) {
        v.push(CombatArchetypeHint::BurstStrike);
    }

    v.sort();
    v.dedup();
    v
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::hero::HeroProfile;
    use crate::domain::party::default_party_partner_hero;
    use crate::domain::skills::SkillId;
    use crate::domain::stats::Stats;

    #[test]
    fn default_partner_has_threat_anchor_hint() {
        let p = default_party_partner_hero();
        let h = hints_for_hero(&p);
        assert!(h.contains(&CombatArchetypeHint::ThreatAnchor));
    }

    #[test]
    fn cleave_loadout_gets_aoe_hint() {
        let mut hero = HeroProfile::new(Stats::default());
        hero.unlock_skill_slots(1);
        let _ = hero.equip_skill(0, SkillId::Cleave);
        assert!(hints_for_hero(&hero).contains(&CombatArchetypeHint::CleaveAoE));
    }
}
