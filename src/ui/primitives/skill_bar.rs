//! Fixed-width horizontal skill bar (6 cells) for party loadouts.

use bevy::prelude::*;

use crate::domain::party::PartyHeroKind;
use crate::domain::skills::SkillId;
use crate::ui::assets::UiPlaceholderImages;
use crate::ui::components::SkillSlotButton;
use crate::ui::interaction::UiClickAction;
use crate::ui::primitives::skill_icon::{spawn_skill_icon, SkillIconConfig};
use crate::ui::theme::UiDensity;

pub const LOADOUT_SLOT_COUNT: usize = 6;

#[derive(Component, Clone, Copy, Debug)]
pub struct SkillBarSlot {
    pub hero: PartyHeroKind,
    pub index: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SkillBarInteraction {
    None,
    OpenSkillBook(PartyHeroKind),
    BuildcraftFocus,
}

#[derive(Clone, Copy, Debug)]
pub struct SkillBarConfig<'a> {
    pub hero: PartyHeroKind,
    pub slots: &'a [Option<SkillId>; LOADOUT_SLOT_COUNT],
    pub unlocked: usize,
    pub focused_index: Option<usize>,
    pub interaction: SkillBarInteraction,
    pub density: UiDensity,
}

impl<'a> SkillBarConfig<'a> {
    pub fn cell_px(self) -> f32 {
        self.density.icon_bar_px()
    }
}

pub fn spawn_skill_bar(
    parent: &mut ChildSpawnerCommands<'_>,
    config: SkillBarConfig<'_>,
    ph: &UiPlaceholderImages,
) {
    let cell_px = config.cell_px();
    let gutter = config.density.gutter_slot();
    parent
        .spawn(Node {
            box_sizing: BoxSizing::BorderBox,
            flex_direction: FlexDirection::Row,
            column_gap: Val::Px(gutter),
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
                    SkillIconConfig::bar_slot(skill, config.hero, index, cell_px, focused, locked),
                    ph,
                );
                row.commands_mut().entity(cell).insert(SkillBarSlot {
                    hero: config.hero,
                    index,
                });
                if locked {
                    continue;
                }
                match config.interaction {
                    SkillBarInteraction::None => {
                        row.commands_mut().entity(cell).insert((
                            Button,
                            Interaction::default(),
                        ));
                    }
                    SkillBarInteraction::OpenSkillBook(kind) => {
                        row.commands_mut().entity(cell).insert((
                            Button,
                            SkillSlotButton { slot: index, kind },
                            UiClickAction::OpenSkillBook,
                        ));
                    }
                    SkillBarInteraction::BuildcraftFocus => {
                        row.commands_mut().entity(cell).insert((
                            Button,
                            UiClickAction::BuildcraftFocusSlot {
                                hero: config.hero,
                                index,
                            },
                        ));
                    }
                }
            }
        });
}
