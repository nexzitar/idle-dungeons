//! Scrollable skill library grid (right column).

use bevy::prelude::*;

use crate::domain::skills::{skill_book_pick_order_for, SkillId};
use crate::ui::assets::UiPlaceholderImages;
use crate::ui::interaction::UiClickAction;
use crate::ui::primitives::panel::{spawn_mounted_panel, MountedPanelConfig};
use crate::ui::primitives::scroll::spawn_scroll_viewport;
use crate::ui::primitives::section::spawn_framed_section_header;
use crate::ui::primitives::skill_icon::{spawn_skill_icon, SkillIconConfig};
use crate::ui::skill_presentation::accent_for_skill;
use crate::ui::theme::{UiDensity, UiTheme};

#[derive(Component, Clone, Copy)]
pub struct BuildcraftLibrarySkill(pub SkillId);

#[derive(Component)]
pub struct BuildcraftClearTile;

pub fn spawn_library_column(
    parent: &mut ChildSpawnerCommands<'_>,
    unlocked: &[SkillId],
    ph: &UiPlaceholderImages,
) {
    let density = UiDensity::Buildcraft;
    let icon_px = density.icon_library_px();
    let grid_gap = density.gutter_grid();

    parent
        .spawn((
            Node {
                box_sizing: BoxSizing::BorderBox,
                flex_grow: 1.0,
                flex_shrink: 1.0,
                min_height: Val::Px(0.0),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(density.gutter_row()),
                ..default()
            },
        ))
        .with_children(|col| {
            spawn_framed_section_header(col, "SKILL LIBRARY");
            spawn_mounted_panel(col, MountedPanelConfig::recessed_flex(), |frame| {
                spawn_scroll_viewport(frame, |scroll| {
                    scroll
                        .spawn(Node {
                            box_sizing: BoxSizing::BorderBox,
                            width: Val::Percent(100.0),
                            flex_direction: FlexDirection::Row,
                            flex_wrap: FlexWrap::Wrap,
                            column_gap: Val::Px(grid_gap),
                            row_gap: Val::Px(grid_gap),
                            ..default()
                        })
                        .with_children(|grid| {
                            spawn_clear_tile(grid, ph, icon_px);
                            for id in skill_book_pick_order_for(unlocked) {
                                spawn_library_skill(grid, id, ph, icon_px);
                            }
                        });
                });
            });
        });
}

fn spawn_clear_tile(parent: &mut ChildSpawnerCommands<'_>, ph: &UiPlaceholderImages, size_px: f32) {
    let ent = spawn_skill_icon(
        parent,
        SkillIconConfig {
            skill: None,
            size_px,
            slot_index: None,
            bar_hero: None,
            focused: false,
            locked: false,
            empty: true,
        },
        ph,
    );
    parent.commands_mut().entity(ent).insert((
        Button,
        BuildcraftClearTile,
        UiClickAction::BuildcraftClearSlot,
    ));
    parent.commands_mut().entity(ent).with_children(|c| {
        c.spawn((
            Text::new("Clear"),
            TextFont::from_font_size(UiTheme::FONT_MICRO),
            TextColor(UiTheme::body_dim()),
        ));
    });
}

fn spawn_library_skill(
    parent: &mut ChildSpawnerCommands<'_>,
    id: SkillId,
    ph: &UiPlaceholderImages,
    size_px: f32,
) {
    let accent = accent_for_skill(id);
    let ent = spawn_skill_icon(parent, SkillIconConfig::filled(id, size_px), ph);
    parent.commands_mut().entity(ent).insert((
        Button,
        BuildcraftLibrarySkill(id),
        UiClickAction::BuildcraftPickSkill(id),
        BorderColor::from(accent.mix(&UiTheme::void_black(), 0.25)),
    ));
}
