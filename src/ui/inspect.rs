//! Inspect region policy — fixed territories that show skill/gear detail on hover or focus.
//!
//! ## Sync pattern (mirror `buildcraft/sync.rs`)
//!
//! 1. **Hover/focus system** updates a small resource (`CampInspectState`) from `Interaction` on
//!    loadout slots (or item cards in gear hub).
//! 2. **Inspect sync system** maps the target to `InspectPanelContent` via helpers in
//!    `primitives/inspect_panel.rs`, then writes marked text/image entities — never ad-hoc strings
//!    in screen spawn code.
//! 3. When a modal owns inspect (e.g. Party Buildcraft), camp sync bails if `SkillBookRoot` exists.

use bevy::prelude::*;

use crate::app::ProfileState;
use crate::domain::party::PartyHeroKind;
use crate::ui::assets::UiPlaceholderImages;
use crate::ui::components::{BuildScreen, SkillBookRoot};
use crate::ui::primitives::inspect_panel::{
    inspect_content_camp_slot, inspect_content_none_camp, CampInspectBody, CampInspectHint,
    CampInspectIcon, CampInspectMeta, CampInspectTags, CampInspectTitle,
};
use crate::ui::primitives::skill_bar::SkillBarSlot;

/// Marks stable inspect real estate on a screen (see `docs/ui-design-system.md` §4).
#[derive(Component, Clone, Copy, Debug)]
pub struct InspectRegion;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum CampInspectTarget {
    #[default]
    None,
    Slot {
        hero: PartyHeroKind,
        index: usize,
    },
}

#[derive(Resource, Default)]
pub struct CampInspectState {
    pub hover: CampInspectTarget,
}

pub fn sync_camp_inspect_hover(
    build: Query<(), With<BuildScreen>>,
    book: Query<(), With<SkillBookRoot>>,
    mut state: ResMut<CampInspectState>,
    slots: Query<(&SkillBarSlot, &Interaction)>,
) {
    if build.is_empty() || !book.is_empty() {
        return;
    }
    let mut hover = CampInspectTarget::None;
    for (slot, interaction) in &slots {
        if *interaction == Interaction::Hovered {
            hover = CampInspectTarget::Slot {
                hero: slot.hero,
                index: slot.index,
            };
        }
    }
    if state.hover != hover {
        state.hover = hover;
    }
}

pub fn sync_camp_inspect_panel(
    build: Query<(), With<BuildScreen>>,
    book: Query<(), With<SkillBookRoot>>,
    state: Res<CampInspectState>,
    profile: Res<ProfileState>,
    ph: Res<UiPlaceholderImages>,
    panels: Query<(), With<InspectRegion>>,
    mut icon: Query<&mut ImageNode, With<CampInspectIcon>>,
    mut texts: ParamSet<(
        Query<&mut Text, With<CampInspectTitle>>,
        Query<&mut Text, With<CampInspectMeta>>,
        Query<&mut Text, With<CampInspectTags>>,
        Query<&mut Text, With<CampInspectBody>>,
        Query<&mut Text, With<CampInspectHint>>,
    )>,
) {
    if build.is_empty() || !book.is_empty() || panels.is_empty() {
        return;
    }
    if !state.is_changed() {
        return;
    }
    let content = match state.hover {
        CampInspectTarget::None => inspect_content_none_camp(&ph),
        CampInspectTarget::Slot { hero, index } => {
            let skill = skill_at_camp_slot(&profile, hero, index);
            inspect_content_camp_slot(hero, index, skill, &ph)
        }
    };
    let Ok(mut icon) = icon.single_mut() else {
        return;
    };
    icon.image = content.icon_image.clone();
    icon.color = content.icon_color;
    if let Ok(mut title) = texts.p0().single_mut() {
        title.0 = content.title.clone();
    }
    if let Ok(mut meta) = texts.p1().single_mut() {
        meta.0 = content.meta.clone();
    }
    if let Ok(mut tags) = texts.p2().single_mut() {
        tags.0 = content.tags.clone();
    }
    if let Ok(mut body) = texts.p3().single_mut() {
        body.0 = content.body.clone();
    }
    if let Ok(mut hint) = texts.p4().single_mut() {
        hint.0 = content.hint.clone();
    }
}

fn skill_at_camp_slot(
    profile: &ProfileState,
    hero: PartyHeroKind,
    index: usize,
) -> Option<crate::domain::skills::SkillId> {
    match hero {
        PartyHeroKind::Player1 => profile
            .effective_hero()
            .equipped_skills
            .get(index)
            .and_then(|s| *s),
        PartyHeroKind::Player2 => profile
            .effective_party_partner()
            .and_then(|h| h.equipped_skills.get(index).and_then(|s| *s)),
    }
}
