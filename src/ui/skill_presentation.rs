//! Skill icon handles and category accents for buildcraft UI.
//!
//! Domain [`SkillCategory`] maps to presentation [`SkillDisplayFamily`] per
//! [`docs/ui-design-system.md`](../../docs/ui-design-system.md) §11.

use bevy::prelude::*;

use crate::domain::skills::{skill_category, skill_definition, SkillCategory, SkillId, SkillKind};
use crate::ui::assets::UiPlaceholderImages;
use crate::ui::theme::{skill_category_chip_colors, UiTheme};

/// Player-facing skill family for frames, chips, and accents (presentation only).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SkillDisplayFamily {
    Attack,
    Reaction,
    Passive,
    Sustain,
    Utility,
}

impl SkillDisplayFamily {
    pub fn label(self) -> &'static str {
        match self {
            Self::Attack => "Attack",
            Self::Reaction => "Reaction",
            Self::Passive => "Passive",
            Self::Sustain => "Sustain",
            Self::Utility => "Utility",
        }
    }
}

/// Maps domain category to the canonical display family.
pub fn display_family_for_category(cat: SkillCategory) -> SkillDisplayFamily {
    match cat {
        SkillCategory::BasicAttack | SkillCategory::AttackSkill | SkillCategory::Proc => {
            SkillDisplayFamily::Attack
        }
        SkillCategory::Reactive => SkillDisplayFamily::Reaction,
        SkillCategory::Passive | SkillCategory::Channel => SkillDisplayFamily::Passive,
        SkillCategory::Buff => SkillDisplayFamily::Sustain,
    }
}

pub fn display_family_for_skill(id: SkillId) -> SkillDisplayFamily {
    display_family_for_category(skill_category(id))
}

pub fn accent_for_display_family(family: SkillDisplayFamily) -> Color {
    match family {
        SkillDisplayFamily::Attack => UiTheme::category_attack(),
        SkillDisplayFamily::Reaction => UiTheme::category_reaction(),
        SkillDisplayFamily::Passive => UiTheme::category_passive(),
        SkillDisplayFamily::Sustain => UiTheme::category_sustain(),
        SkillDisplayFamily::Utility => UiTheme::category_utility(),
    }
}

pub fn icon_for(id: SkillId, ph: &UiPlaceholderImages) -> Handle<Image> {
    match skill_definition(id).kind {
        SkillKind::Active => ph.skill_active.clone(),
        SkillKind::Passive => ph.skill_passive.clone(),
    }
}

pub fn empty_icon(ph: &UiPlaceholderImages) -> Handle<Image> {
    ph.skill_empty.clone()
}

pub fn locked_icon(ph: &UiPlaceholderImages) -> Handle<Image> {
    ph.skill_locked.clone()
}

pub fn accent_for_skill(id: SkillId) -> Color {
    accent_for_display_family(display_family_for_skill(id))
}

/// Legacy chip path — prefers display family accent, falls back to chip bg tint.
pub fn accent_for_category(cat: SkillCategory) -> Color {
    accent_for_display_family(display_family_for_category(cat))
        .mix(&skill_category_chip_colors(cat).0, 0.35)
}

pub fn frame_border_for_skill(id: Option<SkillId>, focused: bool) -> Color {
    if focused {
        return UiTheme::ornate_gold();
    }
    match id {
        Some(s) => accent_for_skill(s).mix(&UiTheme::void_black(), 0.35),
        None => UiTheme::panel_border_inner(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::skills::SkillId;

    #[test]
    fn display_family_mapping_is_stable() {
        assert_eq!(
            display_family_for_category(SkillCategory::AttackSkill),
            SkillDisplayFamily::Attack
        );
        assert_eq!(
            display_family_for_category(SkillCategory::Reactive),
            SkillDisplayFamily::Reaction
        );
        assert_eq!(
            display_family_for_category(SkillCategory::Passive),
            SkillDisplayFamily::Passive
        );
        assert_eq!(
            display_family_for_category(SkillCategory::Buff),
            SkillDisplayFamily::Sustain
        );
        assert_eq!(
            display_family_for_skill(SkillId::Guard),
            SkillDisplayFamily::Reaction
        );
    }
}
