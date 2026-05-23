//! Party helpers: second hero slot for WoW-style threat routing (not NPC escorts).

use crate::domain::hero::HeroProfile;
use crate::domain::skills::SkillId;
use crate::domain::stats::Stats;

/// Which persisted hero sheet skills / rename controls apply to (0-based roster index).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub enum PartyHeroKind {
    #[default]
    Player1,
    Player2,
}

impl PartyHeroKind {
    #[must_use]
    pub fn roster_index(self) -> usize {
        match self {
            Self::Player1 => 0,
            Self::Player2 => 1,
        }
    }

    #[must_use]
    pub fn label(self) -> &'static str {
        match self {
            Self::Player1 => "Player 1",
            Self::Player2 => "Player 2",
        }
    }

    #[must_use]
    pub fn from_roster_index(i: usize) -> Option<Self> {
        match i {
            0 => Some(Self::Player1),
            1 => Some(Self::Player2),
            _ => None,
        }
    }
}

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

/// Clock ticks between **taunt pulse** bursts (partner tank stance only).
pub const PARTY_THREAT_TAUNT_PULSE_INTERVAL: u32 = 14;

/// Flat threat added to slot **1** on each taunt pulse.
const PARTY_THREAT_TAUNT_PULSE_BONUS_SLOT1: i32 = 5;

/// Max threat moved from slot **0** → **1** on pulse (% of slot0 before transfer, capped).
const PARTY_THREAT_TRANSFER_CAP: i32 = 12;

/// Per-tick passive decay on **each** party threat slot (both heroes alive).
const PARTY_THREAT_PASSIVE_DECAY: i32 = 1;

/// Wave 5: decay both party threat buckets, and on interval **transfer** from lead to partner plus a **taunt burst** when the partner is in tank stance (`Guard` + `ThickHide`).
///
/// Returns `true` if a taunt pulse fired (emit [`CombatEvent::ThreatSnapshot`](crate::domain::combat::CombatEvent) in combat after).
pub fn tick_party_threat_routing(
    threat: &mut [i32; 2],
    partner_tank_stance: bool,
    combat_clock: u32,
) -> bool {
    threat[0] = threat[0].saturating_sub(PARTY_THREAT_PASSIVE_DECAY);
    threat[1] = threat[1].saturating_sub(PARTY_THREAT_PASSIVE_DECAY);

    if !partner_tank_stance {
        return false;
    }
    if combat_clock == 0 || combat_clock % PARTY_THREAT_TAUNT_PULSE_INTERVAL != 0 {
        return false;
    }
    let xfer = (threat[0] / 10).min(PARTY_THREAT_TRANSFER_CAP);
    threat[0] = threat[0].saturating_sub(xfer);
    threat[1] = threat[1].saturating_add(xfer);
    threat[1] = threat[1].saturating_add(PARTY_THREAT_TAUNT_PULSE_BONUS_SLOT1);
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn threat_routing_pulse_transfers_and_buffs_partner() {
        let mut t = [100, 20];
        let pulsed = tick_party_threat_routing(&mut t, true, PARTY_THREAT_TAUNT_PULSE_INTERVAL);
        assert!(pulsed);
        assert!(t[1] > 20, "partner should gain transfer + burst: {:?}", t);
        assert!(t[0] < 100, "lead should lose decay + transfer: {:?}", t);
    }

    #[test]
    fn threat_routing_no_pulse_off_interval() {
        let mut t = [50, 50];
        let pulsed = tick_party_threat_routing(&mut t, true, 1);
        assert!(!pulsed);
        assert_eq!(t[0], 49);
        assert_eq!(t[1], 49);
    }
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
