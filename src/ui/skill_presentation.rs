//! Skill icon handles and category accents for buildcraft UI.

use bevy::prelude::*;

use crate::domain::skills::{skill_category, skill_definition, SkillCategory, SkillId, SkillKind};
use crate::ui::assets::UiPlaceholderImages;
use crate::ui::theme::{skill_category_chip_colors, UiTheme};

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
    accent_for_category(skill_category(id))
}

pub fn accent_for_category(cat: SkillCategory) -> Color {
    skill_category_chip_colors(cat).0
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
