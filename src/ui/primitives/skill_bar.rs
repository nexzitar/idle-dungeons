//! Fixed-width horizontal skill bar (6 cells) for party loadouts.

use bevy::prelude::*;

use crate::domain::party::PartyHeroKind;
use crate::domain::skills::SkillId;
use crate::ui::assets::UiPlaceholderImages;
use crate::ui::interaction::UiClickAction;
use crate::ui::primitives::skill_icon::{spawn_skill_icon, SkillIconConfig};
use crate::ui::theme::UiTheme;
use crate::ui::buildcraft::session::LOADOUT_SLOT_COUNT;

#[derive(Component, Clone, Copy, Debug)]
pub struct SkillBarSlot {
    pub hero: PartyHeroKind,
    pub index: usize,
}

#[derive(Clone, Copy, Debug)]
pub struct SkillBarConfig<'a> {
    pub hero: PartyHeroKind,
    pub slots: &'a [Option<SkillId>; LOADOUT_SLOT_COUNT],
    pub unlocked: usize,
    pub focused_index: Option<usize>,
    pub interactive: bool,
    pub cell_px: f32,
}

pub fn spawn_skill_bar(
    parent: &mut ChildSpawnerCommands<'_>,
    config: SkillBarConfig<'_>,
    ph: &UiPlaceholderImages,
) {
    parent
        .spawn(Node {
            box_sizing: BoxSizing::BorderBox,
            flex_direction: FlexDirection::Row,
            column_gap: Val::Px(8.0),
            align_items: AlignItems::Center,
            ..default()
        })
        .with_children(|row| {
            for index in 0..LOADOUT_SLOT_COUNT {
                let locked = index >= config.unlocked;
                let focused = config.focused_index == Some(index);
                let skill = if locked { None } else { config.slots[index] };
                let cell = spawn_skill_icon(
                    row,
                    SkillIconConfig::bar_slot(skill, config.hero, index, config.cell_px, focused, locked),
                    ph,
                );
                row.commands_mut().entity(cell).insert(SkillBarSlot {
                    hero: config.hero,
                    index,
                });
                if config.interactive && !locked {
                    row.commands_mut().entity(cell).insert((
                        Button,
                        UiClickAction::BuildcraftFocusSlot {
                            hero: config.hero,
                            index,
                        },
                    ));
                }
            }
        });
}
