//! Skill book entry — delegates to party buildcraft sheet.

use bevy::prelude::*;

use crate::domain::party::PartyHeroKind;
use crate::domain::skills::SkillId;
use crate::ui::assets::UiPlaceholderImages;
use crate::ui::buildcraft::{spawn_buildcraft_sheet, BuildcraftEditSession};

pub fn spawn_skill_book_modal(
    parent: &mut ChildSpawnerCommands<'_>,
    target_slot: usize,
    sheet: PartyHeroKind,
    unlocked: &[SkillId],
    ph: &UiPlaceholderImages,
    session: &BuildcraftEditSession,
) {
    spawn_buildcraft_sheet(parent, session, unlocked, ph);
    let _ = (target_slot, sheet);
}
