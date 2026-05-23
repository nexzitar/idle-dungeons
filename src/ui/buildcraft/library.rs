//! Scrollable skill library grid (right column).

use bevy::prelude::*;

use crate::domain::skills::{skill_book_pick_order_for, SkillId};
use crate::ui::assets::UiPlaceholderImages;
use crate::ui::skill_presentation::accent_for_skill;
use crate::ui::interaction::UiClickAction;
use crate::ui::primitives::scroll::spawn_scroll_viewport;
use crate::ui::primitives::skill_icon::{spawn_skill_icon, SkillIconConfig};
use crate::ui::theme::{section_title, UiTheme};

#[derive(Component, Clone, Copy)]
pub struct BuildcraftLibrarySkill(pub SkillId);

#[derive(Component)]
pub struct BuildcraftClearTile;

pub fn spawn_library_column(
    parent: &mut ChildSpawnerCommands<'_>,
    unlocked: &[SkillId],
    ph: &UiPlaceholderImages,
) {
    parent
        .spawn((
            Node {
                box_sizing: BoxSizing::BorderBox,
                flex_grow: 1.0,
                flex_shrink: 1.0,
                min_height: Val::Px(0.0),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(8.0),
                ..default()
            },
        ))
        .with_children(|col| {
            col.spawn(section_title("SKILL LIBRARY"));
            col.spawn((
                Node {
                    box_sizing: BoxSizing::BorderBox,
                    flex_grow: 1.0,
                    min_height: Val::Px(0.0),
                    flex_direction: FlexDirection::Column,
                    border: UiRect::all(Val::Px(1.0)),
                    ..default()
                },
                BackgroundColor(UiTheme::panel_bg()),
                BorderColor::from(UiTheme::panel_border_inner()),
            ))
            .with_children(|frame| {
                spawn_scroll_viewport(frame, |scroll| {
                    scroll
                        .spawn(Node {
                            box_sizing: BoxSizing::BorderBox,
                            width: Val::Percent(100.0),
                            flex_direction: FlexDirection::Row,
                            flex_wrap: FlexWrap::Wrap,
                            column_gap: Val::Px(14.0),
                            row_gap: Val::Px(14.0),
                            padding: UiRect::all(Val::Px(UiTheme::PANEL_INSET)),
                            ..default()
                        })
                        .with_children(|grid| {
                            spawn_clear_tile(grid, ph);
                            for id in skill_book_pick_order_for(unlocked) {
                                spawn_library_skill(grid, id, ph);
                            }
                        });
                });
            });
        });
}

fn spawn_clear_tile(parent: &mut ChildSpawnerCommands<'_>, ph: &UiPlaceholderImages) {
    let ent = spawn_skill_icon(
        parent,
        SkillIconConfig {
            skill: None,
            size_px: 72.0,
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

fn spawn_library_skill(parent: &mut ChildSpawnerCommands<'_>, id: SkillId, ph: &UiPlaceholderImages) {
    let accent = accent_for_skill(id);
    let ent = spawn_skill_icon(
        parent,
        SkillIconConfig::filled(id, 72.0),
        ph,
    );
    parent.commands_mut().entity(ent).insert((
        Button,
        BuildcraftLibrarySkill(id),
        UiClickAction::BuildcraftPickSkill(id),
        BorderColor::from(accent.mix(&UiTheme::void_black(), 0.25)),
    ));
}
