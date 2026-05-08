//! Party helpers: second hero slot for WoW-style threat routing (not NPC escorts).

use crate::domain::hero::HeroProfile;
use crate::domain::skills::SkillId;
use crate::domain::stats::Stats;

/// Extra starting threat for a hero in a **tank stance** (baseline “sticky” aggro like WoW tanks).
pub fn threat_stance_seed(hero: &HeroProfile) -> i32 {
    let skills: Vec<SkillId> = hero.equipped_skill_ids().collect();
    let has = |id: SkillId| skills.iter().any(|&s| s == id);
    if has(SkillId::Guard) && has(SkillId::ThickHide) {
        8
    } else {
        0
    }
}

/// Passive threat trickle per clock tick while this hero is up (tank-stance only).
pub fn threat_stance_tick_drip(hero: &HeroProfile) -> bool {
    threat_stance_seed(hero) > 0
}

/// Default second hero for new profiles until party UI exists — a tank-skewed **HeroProfile**, not an NPC.
pub fn default_party_partner_hero() -> HeroProfile {
    let mut h = HeroProfile::new(Stats {
        max_health: 72,
        damage: 5,
        armor: 9,
        attack_speed: 0.55,
        healing_power: 0,
    });
    h.name = "Briar".to_string();
    h.unlock_skill_slots(3);
    let _ = h.equip_skill(0, SkillId::Guard);
    let _ = h.equip_skill(1, SkillId::ThickHide);
    let _ = h.equip_skill(2, SkillId::CautiousAdvance);
    h
}
