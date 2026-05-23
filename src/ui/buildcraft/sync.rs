//! Buildcraft sheet sync: inspect panel, hover, party bar visuals.

use bevy::prelude::*;

use crate::ui::assets::UiPlaceholderImages;
use crate::ui::buildcraft::library::BuildcraftLibrarySkill;
use crate::ui::buildcraft::session::{BuildcraftEditSession, InspectTarget};
use crate::ui::buildcraft::sheet::BuildcraftApplyButton;
use crate::ui::components::{SkillBookRoot, UiButtonPalette};
use crate::ui::primitives::inspect_panel::{
    inspect_content_library, inspect_content_none, inspect_content_slot, BuildcraftInspectBody,
    BuildcraftInspectIcon, BuildcraftInspectMeta, BuildcraftInspectSlotHint,
    BuildcraftInspectSynergy, BuildcraftInspectTags, BuildcraftInspectTitle,
};
use crate::ui::primitives::skill_bar::SkillBarSlot;
use crate::ui::primitives::skill_icon::BuildcraftSlotArt;
use crate::ui::skill_presentation::{empty_icon, frame_border_for_skill, icon_for, locked_icon};
use crate::ui::theme::UiTheme;

pub fn sync_buildcraft_hover(
    mut session: ResMut<BuildcraftEditSession>,
    book: Query<(), With<SkillBookRoot>>,
    slots: Query<(&SkillBarSlot, &Interaction)>,
    skills: Query<(&BuildcraftLibrarySkill, &Interaction)>,
) {
    if book.is_empty() {
        return;
    }
    let mut hover = session.hover_inspect;
    for (slot, interaction) in &slots {
        if *interaction == Interaction::Hovered {
            hover = InspectTarget::Slot {
                hero: slot.hero,
                index: slot.index,
            };
        }
    }
    for (skill, interaction) in &skills {
        if *interaction == Interaction::Hovered {
            hover = InspectTarget::Library(skill.0);
        }
    }
    session.hover_inspect = hover;
}

pub fn sync_buildcraft_inspect(
    session: Res<BuildcraftEditSession>,
    book: Query<(), With<SkillBookRoot>>,
    ph: Res<UiPlaceholderImages>,
    mut icon: Query<&mut ImageNode, With<BuildcraftInspectIcon>>,
    mut texts: ParamSet<(
        Query<&mut Text, With<BuildcraftInspectTitle>>,
        Query<&mut Text, With<BuildcraftInspectMeta>>,
        Query<&mut Text, With<BuildcraftInspectTags>>,
        Query<&mut Text, With<BuildcraftInspectBody>>,
        Query<&mut Text, With<BuildcraftInspectSynergy>>,
        Query<&mut Text, With<BuildcraftInspectSlotHint>>,
    )>,
) {
    if book.is_empty() {
        return;
    }
    let content = match session.hover_inspect {
        InspectTarget::None => inspect_content_none(&ph),
        InspectTarget::Slot { hero, index } => {
            let skill = session.loadout(hero).and_then(|e| e.slot(index));
            inspect_content_slot(hero, index, skill, &ph)
        }
        InspectTarget::Library(id) => inspect_content_library(id, &ph),
    };
    if let Ok(mut icon) = icon.single_mut() {
        icon.image = content.icon_image;
        icon.color = content.icon_color;
    }
    if let Ok(mut title) = texts.p0().single_mut() {
        title.0 = content.title;
    }
    if let Ok(mut meta) = texts.p1().single_mut() {
        meta.0 = content.meta;
    }
    if let Ok(mut tags) = texts.p2().single_mut() {
        tags.0 = content.tags;
    }
    if let Ok(mut body) = texts.p3().single_mut() {
        body.0 = content.body;
    }
    if let Ok(mut synergy) = texts.p4().single_mut() {
        synergy.0 = content.synergy;
    }
    if let Ok(mut hint) = texts.p5().single_mut() {
        hint.0 = content.hint;
    }
}

pub fn sync_buildcraft_apply_enabled(
    session: Res<BuildcraftEditSession>,
    book: Query<(), With<SkillBookRoot>>,
    mut apply: Query<
        (
            &BuildcraftApplyButton,
            &UiButtonPalette,
            &mut BackgroundColor,
            &mut BorderColor,
        ),
        With<Button>,
    >,
) {
    if book.is_empty() {
        return;
    }
    let dirty = session.is_dirty();
    for (_, pal, mut bg, mut border) in &mut apply {
        let (c, b) = if dirty {
            (pal.idle_bg, pal.idle_border)
        } else {
            (
                UiTheme::stone_deep().mix(&UiTheme::void_black(), 0.4),
                UiTheme::panel_border_inner(),
            )
        };
        *bg = c.into();
        *border = b.into();
    }
}

pub fn sync_buildcraft_party_bars(
    session: Res<BuildcraftEditSession>,
    book: Query<(), With<SkillBookRoot>>,
    ph: Res<UiPlaceholderImages>,
    mut cells: Query<(&SkillBarSlot, &mut BorderColor)>,
    mut arts: Query<(&BuildcraftSlotArt, &mut ImageNode)>,
) {
    if book.is_empty() {
        return;
    }
    if !session.is_changed() {
        return;
    }
    for (slot, mut border) in &mut cells {
        let Some(edit) = session.loadout(slot.hero) else {
            continue;
        };
        let locked = slot.index >= edit.unlocked;
        let skill = if locked {
            None
        } else {
            edit.pending[slot.index]
        };
        let focused = session.focused == (slot.hero, slot.index);
        *border = BorderColor::from(frame_border_for_skill(skill, focused));
    }
    for (art, mut img) in &mut arts {
        let Some(edit) = session.loadout(art.hero) else {
            continue;
        };
        let locked = art.index >= edit.unlocked;
        let skill = if locked {
            None
        } else {
            edit.pending[art.index]
        };
        if locked {
            img.image = locked_icon(&ph);
            img.color = Color::srgba(0.35, 0.35, 0.38, 0.85);
        } else if let Some(id) = skill {
            img.image = icon_for(id, &ph);
            img.color = Color::WHITE;
        } else {
            img.image = empty_icon(&ph);
            img.color = Color::srgba(0.55, 0.52, 0.48, 0.55);
        }
    }
}
