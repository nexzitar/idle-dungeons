//! HP bars and cast / cooldown stacks used by the playback theater.

use bevy::prelude::*;

use crate::ui::components::{
    PlaybackEnemyBarFill, PlaybackFoeAltCastFill, PlaybackFoeAltCdFill, PlaybackFoeCastFill,
    PlaybackFoeCdFill, PlaybackPlayer0BarFill, PlaybackPlayer0CastFill, PlaybackPlayer0CdFill,
    PlaybackPlayer0InstantRechargeFill, PlaybackPlayer0SkillGcdFill, PlaybackPlayer1BarFill,
    PlaybackPlayer1CastFill, PlaybackPlayer1CdFill, PlaybackPlayer1InstantRechargeFill,
    PlaybackPlayer1SkillGcdFill,
};
use crate::ui::primitives::bar::{spawn_horizontal_bar, UiBarStyle};

fn spawn_playback_timing_bar<M: Component>(
    parent: &mut ChildSpawnerCommands<'_>,
    style: UiBarStyle,
    fill_marker: M,
) {
    spawn_horizontal_bar(parent, style, fill_marker, 0.0);
}

pub(super) fn playback_player0_bar(parent: &mut ChildSpawnerCommands<'_>, fill_pct: f32) {
    spawn_horizontal_bar(
        parent,
        UiBarStyle::playback_hp_lead(),
        PlaybackPlayer0BarFill,
        fill_pct,
    );
}

pub(super) fn playback_player1_bar(parent: &mut ChildSpawnerCommands<'_>, fill_pct: f32) {
    spawn_horizontal_bar(
        parent,
        UiBarStyle::playback_hp_ally(),
        PlaybackPlayer1BarFill,
        fill_pct,
    );
}

pub(super) fn playback_enemy_bar(parent: &mut ChildSpawnerCommands<'_>, fill_pct: f32) {
    spawn_horizontal_bar(
        parent,
        UiBarStyle::playback_hp_enemy(),
        PlaybackEnemyBarFill,
        fill_pct,
    );
}

pub(super) fn playback_cast_cd_stack_lead(parent: &mut ChildSpawnerCommands<'_>) {
    parent
        .spawn(Node {
            box_sizing: BoxSizing::BorderBox,
            width: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(2.0),
            ..default()
        })
        .with_children(|col| {
            spawn_playback_timing_bar(col, UiBarStyle::playback_cast_lead(), PlaybackPlayer0CastFill);
            spawn_playback_timing_bar(col, UiBarStyle::playback_cd_lead(), PlaybackPlayer0CdFill);
            spawn_playback_timing_bar(
                col,
                UiBarStyle::playback_skill_gcd_lead(),
                PlaybackPlayer0SkillGcdFill,
            );
            spawn_playback_timing_bar(
                col,
                UiBarStyle::playback_instant_recharge_lead(),
                PlaybackPlayer0InstantRechargeFill,
            );
        });
}

pub(super) fn playback_cast_cd_stack_ally(parent: &mut ChildSpawnerCommands<'_>) {
    parent
        .spawn(Node {
            box_sizing: BoxSizing::BorderBox,
            width: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(2.0),
            ..default()
        })
        .with_children(|col| {
            spawn_playback_timing_bar(col, UiBarStyle::playback_cast_ally(), PlaybackPlayer1CastFill);
            spawn_playback_timing_bar(col, UiBarStyle::playback_cd_ally(), PlaybackPlayer1CdFill);
            spawn_playback_timing_bar(
                col,
                UiBarStyle::playback_skill_gcd_ally(),
                PlaybackPlayer1SkillGcdFill,
            );
            spawn_playback_timing_bar(
                col,
                UiBarStyle::playback_instant_recharge_ally(),
                PlaybackPlayer1InstantRechargeFill,
            );
        });
}

pub(super) fn playback_cast_cd_stack_foe_alt(parent: &mut ChildSpawnerCommands<'_>) {
    parent
        .spawn(Node {
            box_sizing: BoxSizing::BorderBox,
            width: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(2.0),
            ..default()
        })
        .with_children(|col| {
            spawn_playback_timing_bar(
                col,
                UiBarStyle::playback_cast_foe_alt(),
                PlaybackFoeAltCastFill,
            );
            spawn_playback_timing_bar(col, UiBarStyle::playback_cd_foe_alt(), PlaybackFoeAltCdFill);
        });
}

pub(super) fn playback_cast_cd_stack_foe(parent: &mut ChildSpawnerCommands<'_>) {
    parent
        .spawn(Node {
            box_sizing: BoxSizing::BorderBox,
            width: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(2.0),
            ..default()
        })
        .with_children(|col| {
            spawn_playback_timing_bar(col, UiBarStyle::playback_cast_foe(), PlaybackFoeCastFill);
            spawn_playback_timing_bar(col, UiBarStyle::playback_cd_foe(), PlaybackFoeCdFill);
        });
}
