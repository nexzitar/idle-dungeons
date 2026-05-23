//! Hero loadout row — label + shared skill bar (the combat script strip).

use bevy::prelude::*;

use crate::domain::hero::HeroProfile;
use crate::domain::party::PartyHeroKind;
use crate::domain::skills::SkillId;
use crate::ui::assets::UiPlaceholderImages;
use crate::ui::primitives::skill_bar::{spawn_skill_bar, SkillBarConfig, SkillBarInteraction, LOADOUT_SLOT_COUNT};
use crate::ui::theme::{UiDensity, UiTheme};

#[derive(Clone, Copy, Debug)]
pub struct LoadoutRowConfig<'a> {
    pub hero_kind: PartyHeroKind,
    pub label: Option<&'a str>,
    pub slots: &'a [Option<SkillId>; LOADOUT_SLOT_COUNT],
    pub unlocked: usize,
    pub focused_index: Option<usize>,
    pub interaction: SkillBarInteraction,
    pub density: UiDensity,
}

pub fn slots_from_hero(hero: &HeroProfile) -> [Option<SkillId>; LOADOUT_SLOT_COUNT] {
    let mut slots = [None; LOADOUT_SLOT_COUNT];
    for (i, slot) in hero.equipped_skills.iter().enumerate().take(LOADOUT_SLOT_COUNT) {
        slots[i] = *slot;
    }
    slots
}

pub fn spawn_loadout_row(
    parent: &mut ChildSpawnerCommands<'_>,
    config: LoadoutRowConfig<'_>,
    ph: &UiPlaceholderImages,
) {
    parent
        .spawn((
            Node {
                box_sizing: BoxSizing::BorderBox,
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(config.density.gutter_row()),
                ..default()
            },
        ))
        .with_children(|block| {
            if let Some(label) = config.label {
                block.spawn((
                    Text::new(label),
                    TextFont::from_font_size(UiTheme::FONT_SECTION),
                    TextColor(UiTheme::muted_cream()),
                ));
            }
            spawn_skill_bar(
                block,
                SkillBarConfig {
                    hero: config.hero_kind,
                    slots: config.slots,
                    unlocked: config.unlocked,
                    focused_index: config.focused_index,
                    interaction: config.interaction,
                    density: config.density,
                },
                ph,
            );
        });
}
