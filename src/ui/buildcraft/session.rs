//! Transient party loadout edit state — commits to profile on Apply only.

use bevy::prelude::*;

use crate::domain::hero::HeroProfile;
use crate::domain::party::PartyHeroKind;
use crate::domain::skills::SkillId;
use crate::save::SaveProfile;

pub use crate::ui::primitives::skill_bar::LOADOUT_SLOT_COUNT;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InspectTarget {
    None,
    Slot { hero: PartyHeroKind, index: usize },
    Library(SkillId),
}

#[derive(Debug, Clone)]
pub struct HeroLoadoutEdit {
    pub hero: PartyHeroKind,
    pub display_name: String,
    pub unlocked: usize,
    pub snapshot: [Option<SkillId>; LOADOUT_SLOT_COUNT],
    pub pending: [Option<SkillId>; LOADOUT_SLOT_COUNT],
}

impl HeroLoadoutEdit {
    pub fn from_hero(hero: &HeroProfile, kind: PartyHeroKind) -> Self {
        let mut snapshot = [None; LOADOUT_SLOT_COUNT];
        for (i, slot) in hero.equipped_skills.iter().enumerate().take(LOADOUT_SLOT_COUNT) {
            snapshot[i] = *slot;
        }
        Self {
            hero: kind,
            display_name: hero.name.clone(),
            unlocked: hero.unlocked_skill_slots,
            snapshot,
            pending: snapshot,
        }
    }

    pub fn slot(&self, index: usize) -> Option<SkillId> {
        self.pending.get(index).copied().flatten()
    }
}

#[derive(Resource, Debug, Clone)]
pub struct BuildcraftEditSession {
    pub lead: HeroLoadoutEdit,
    pub partner: Option<HeroLoadoutEdit>,
    pub focused: (PartyHeroKind, usize),
    pub hover_inspect: InspectTarget,
}

impl Default for BuildcraftEditSession {
    fn default() -> Self {
        Self::open_from_profile(&SaveProfile::default(), PartyHeroKind::Player1, 0)
    }
}

impl BuildcraftEditSession {
    pub fn open_from_profile(
        profile: &SaveProfile,
        focus_hero: PartyHeroKind,
        focus_slot: usize,
    ) -> Self {
        let lead = HeroLoadoutEdit::from_hero(&profile.hero, PartyHeroKind::Player1);
        let partner = profile
            .party_partner
            .as_ref()
            .map(|h| HeroLoadoutEdit::from_hero(h, PartyHeroKind::Player2));
        let focused = (
            focus_hero,
            focus_slot.min(LOADOUT_SLOT_COUNT.saturating_sub(1)),
        );
        Self {
            lead,
            partner,
            focused,
            hover_inspect: InspectTarget::Slot {
                hero: focused.0,
                index: focused.1,
            },
        }
    }

    pub fn loadout_mut(&mut self, hero: PartyHeroKind) -> Option<&mut HeroLoadoutEdit> {
        match hero {
            PartyHeroKind::Player1 => Some(&mut self.lead),
            PartyHeroKind::Player2 => self.partner.as_mut(),
        }
    }

    pub fn loadout(&self, hero: PartyHeroKind) -> Option<&HeroLoadoutEdit> {
        match hero {
            PartyHeroKind::Player1 => Some(&self.lead),
            PartyHeroKind::Player2 => self.partner.as_ref(),
        }
    }

    pub fn set_focus(&mut self, hero: PartyHeroKind, index: usize) {
        if index >= LOADOUT_SLOT_COUNT {
            return;
        }
        self.focused = (hero, index);
        self.hover_inspect = InspectTarget::Slot { hero, index };
    }

    /// Assign to focused slot; evict same skill from other slots on **this hero only**.
    pub fn assign_to_focused(&mut self, skill: Option<SkillId>) {
        let (hero, slot) = self.focused;
        let Some(edit) = self.loadout_mut(hero) else {
            return;
        };
        if slot >= edit.unlocked {
            return;
        }
        if let Some(s) = skill {
            for i in 0..LOADOUT_SLOT_COUNT {
                if i != slot && edit.pending[i] == Some(s) {
                    edit.pending[i] = None;
                }
            }
            edit.pending[slot] = Some(s);
        } else {
            edit.pending[slot] = None;
        }
        self.hover_inspect = InspectTarget::Slot { hero, index: slot };
    }

    pub fn is_dirty(&self) -> bool {
        self.lead.pending != self.lead.snapshot
            || self
                .partner
                .as_ref()
                .is_some_and(|p| p.pending != p.snapshot)
    }

    pub fn commit(&self, profile: &mut SaveProfile) -> bool {
        profile.sync_skill_slot_unlocks();
        let mut changed = false;
        if Self::commit_hero(&mut profile.hero, &self.lead) {
            changed = true;
        }
        if let (Some(partner), Some(edit)) = (&mut profile.party_partner, &self.partner) {
            if Self::commit_hero(partner, edit) {
                changed = true;
            }
        }
        changed
    }

    fn commit_hero(hero: &mut HeroProfile, edit: &HeroLoadoutEdit) -> bool {
        let mut any = false;
        for slot in 0..LOADOUT_SLOT_COUNT {
            if edit.pending[slot] == edit.snapshot[slot] {
                continue;
            }
            if hero.assign_skill_to_slot(slot, edit.pending[slot]).is_ok() {
                any = true;
            }
        }
        any
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::skills::SkillId;

    #[test]
    fn intra_hero_duplicate_evicted_in_pending() {
        let mut profile = SaveProfile::default();
        profile.hero.unlock_skill_slots(3);
        profile
            .hero
            .assign_skill_to_slot(0, Some(SkillId::Guard))
            .unwrap();
        let mut session = BuildcraftEditSession::open_from_profile(&profile, PartyHeroKind::Player1, 0);
        session.set_focus(PartyHeroKind::Player1, 2);
        session.assign_to_focused(Some(SkillId::Guard));
        assert_eq!(session.lead.pending[0], None);
        assert_eq!(session.lead.pending[2], Some(SkillId::Guard));
    }

    #[test]
    fn cross_hero_same_skill_allowed() {
        let mut profile = SaveProfile::default();
        profile.hero.unlock_skill_slots(2);
        profile
            .hero
            .assign_skill_to_slot(0, Some(SkillId::Guard))
            .unwrap();
        profile.party_partner = Some(crate::domain::party::default_party_partner_hero());
        profile.party_partner.as_mut().unwrap().unlock_skill_slots(2);
        let mut session = BuildcraftEditSession::open_from_profile(&profile, PartyHeroKind::Player2, 0);
        session.set_focus(PartyHeroKind::Player2, 0);
        session.assign_to_focused(Some(SkillId::Guard));
        assert_eq!(session.lead.pending[0], Some(SkillId::Guard));
        assert_eq!(session.partner.as_ref().unwrap().pending[0], Some(SkillId::Guard));
    }

    #[test]
    fn commit_writes_profile() {
        let mut profile = SaveProfile::default();
        profile.hero.unlock_skill_slots(2);
        let mut session = BuildcraftEditSession::open_from_profile(&profile, PartyHeroKind::Player1, 0);
        session.assign_to_focused(Some(SkillId::HeavyStrike));
        assert!(session.commit(&mut profile));
        assert_eq!(profile.hero.equipped_skills[0], Some(SkillId::HeavyStrike));
    }
}
