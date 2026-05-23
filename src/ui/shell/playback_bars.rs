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
use crate::ui::theme::UiTheme;

pub(super) fn playback_player0_bar(parent: &mut ChildSpawnerCommands<'_>, fill_pct: f32) {
    spawn_horizontal_bar(
        parent,
        UiBarStyle {
            track: UiTheme::void_black(),
            fill: UiTheme::healing(),
            height_px: 14.0,
            border: true,
        },
        PlaybackPlayer0BarFill,
        fill_pct,
    );
}

pub(super) fn playback_player1_bar(parent: &mut ChildSpawnerCommands<'_>, fill_pct: f32) {
    spawn_horizontal_bar(
        parent,
        UiBarStyle {
            track: UiTheme::void_black(),
            fill: Color::srgb(0.38, 0.72, 0.92),
            height_px: 14.0,
            border: true,
        },
        PlaybackPlayer1BarFill,
        fill_pct,
    );
}

pub(super) fn playback_enemy_bar(parent: &mut ChildSpawnerCommands<'_>, fill_pct: f32) {
    parent
        .spawn((
            Node {
                box_sizing: BoxSizing::BorderBox,
                width: Val::Percent(100.0),
                height: Val::Px(14.0),
                border: UiRect::all(Val::Px(1.0)),
                ..default()
            },
            BackgroundColor(UiTheme::void_black().into()),
            BorderColor::from(UiTheme::panel_border()),
        ))
        .with_children(|bar| {
            bar.spawn((
                Node {
                    box_sizing: BoxSizing::BorderBox,
                    width: Val::Percent((fill_pct * 100.0).clamp(0.0, 100.0)),
                    height: Val::Percent(100.0),
                    ..default()
                },
                BackgroundColor(UiTheme::danger().into()),
                PlaybackEnemyBarFill,
            ));
        });
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
            col.spawn((
                Node {
                    box_sizing: BoxSizing::BorderBox,
                    width: Val::Percent(100.0),
                    height: Val::Px(5.0),
                    border: UiRect::all(Val::Px(1.0)),
                    ..default()
                },
                BackgroundColor(UiTheme::void_black().into()),
                BorderColor::from(UiTheme::panel_border()),
            ))
            .with_children(|track| {
                track.spawn((
                    Node {
                        box_sizing: BoxSizing::BorderBox,
                        width: Val::Percent(0.0),
                        height: Val::Percent(100.0),
                        ..default()
                    },
                    BackgroundColor(UiTheme::muted_gold().into()),
                    PlaybackPlayer0CastFill,
                ));
            });
            col.spawn((
                Node {
                    box_sizing: BoxSizing::BorderBox,
                    width: Val::Percent(100.0),
                    height: Val::Px(5.0),
                    border: UiRect::all(Val::Px(1.0)),
                    ..default()
                },
                BackgroundColor(UiTheme::void_black().into()),
                BorderColor::from(UiTheme::panel_border()),
            ))
            .with_children(|track| {
                track.spawn((
                    Node {
                        box_sizing: BoxSizing::BorderBox,
                        width: Val::Percent(0.0),
                        height: Val::Percent(100.0),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.28, 0.32, 0.42).into()),
                    PlaybackPlayer0CdFill,
                ));
            });
            col.spawn((
                Node {
                    box_sizing: BoxSizing::BorderBox,
                    width: Val::Percent(100.0),
                    height: Val::Px(4.0),
                    border: UiRect::all(Val::Px(1.0)),
                    ..default()
                },
                BackgroundColor(UiTheme::void_black().into()),
                BorderColor::from(UiTheme::panel_border()),
            ))
            .with_children(|track| {
                track.spawn((
                    Node {
                        box_sizing: BoxSizing::BorderBox,
                        width: Val::Percent(0.0),
                        height: Val::Percent(100.0),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.52, 0.38, 0.62).into()),
                    PlaybackPlayer0SkillGcdFill,
                ));
            });
            col.spawn((
                Node {
                    box_sizing: BoxSizing::BorderBox,
                    width: Val::Percent(100.0),
                    height: Val::Px(4.0),
                    border: UiRect::all(Val::Px(1.0)),
                    ..default()
                },
                BackgroundColor(UiTheme::void_black().into()),
                BorderColor::from(UiTheme::panel_border()),
            ))
            .with_children(|track| {
                track.spawn((
                    Node {
                        box_sizing: BoxSizing::BorderBox,
                        width: Val::Percent(0.0),
                        height: Val::Percent(100.0),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.34, 0.52, 0.40).into()),
                    PlaybackPlayer0InstantRechargeFill,
                ));
            });
        });
}

pub(super) fn playback_cast_cd_stack_ally(parent: &mut ChildSpawnerCommands<'_>) {
    let tone = Color::srgb(0.38, 0.72, 0.92);
    parent
        .spawn(Node {
            box_sizing: BoxSizing::BorderBox,
            width: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(2.0),
            ..default()
        })
        .with_children(|col| {
            col.spawn((
                Node {
                    box_sizing: BoxSizing::BorderBox,
                    width: Val::Percent(100.0),
                    height: Val::Px(5.0),
                    border: UiRect::all(Val::Px(1.0)),
                    ..default()
                },
                BackgroundColor(UiTheme::void_black().into()),
                BorderColor::from(UiTheme::panel_border()),
            ))
            .with_children(|track| {
                track.spawn((
                    Node {
                        box_sizing: BoxSizing::BorderBox,
                        width: Val::Percent(0.0),
                        height: Val::Percent(100.0),
                        ..default()
                    },
                    BackgroundColor(tone.into()),
                    PlaybackPlayer1CastFill,
                ));
            });
            col.spawn((
                Node {
                    box_sizing: BoxSizing::BorderBox,
                    width: Val::Percent(100.0),
                    height: Val::Px(5.0),
                    border: UiRect::all(Val::Px(1.0)),
                    ..default()
                },
                BackgroundColor(UiTheme::void_black().into()),
                BorderColor::from(UiTheme::panel_border()),
            ))
            .with_children(|track| {
                track.spawn((
                    Node {
                        box_sizing: BoxSizing::BorderBox,
                        width: Val::Percent(0.0),
                        height: Val::Percent(100.0),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.22, 0.36, 0.48).into()),
                    PlaybackPlayer1CdFill,
                ));
            });
            col.spawn((
                Node {
                    box_sizing: BoxSizing::BorderBox,
                    width: Val::Percent(100.0),
                    height: Val::Px(4.0),
                    border: UiRect::all(Val::Px(1.0)),
                    ..default()
                },
                BackgroundColor(UiTheme::void_black().into()),
                BorderColor::from(UiTheme::panel_border()),
            ))
            .with_children(|track| {
                track.spawn((
                    Node {
                        box_sizing: BoxSizing::BorderBox,
                        width: Val::Percent(0.0),
                        height: Val::Percent(100.0),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.32, 0.48, 0.62).into()),
                    PlaybackPlayer1SkillGcdFill,
                ));
            });
            col.spawn((
                Node {
                    box_sizing: BoxSizing::BorderBox,
                    width: Val::Percent(100.0),
                    height: Val::Px(4.0),
                    border: UiRect::all(Val::Px(1.0)),
                    ..default()
                },
                BackgroundColor(UiTheme::void_black().into()),
                BorderColor::from(UiTheme::panel_border()),
            ))
            .with_children(|track| {
                track.spawn((
                    Node {
                        box_sizing: BoxSizing::BorderBox,
                        width: Val::Percent(0.0),
                        height: Val::Percent(100.0),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.28, 0.55, 0.45).into()),
                    PlaybackPlayer1InstantRechargeFill,
                ));
            });
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
            col.spawn((
                Node {
                    box_sizing: BoxSizing::BorderBox,
                    width: Val::Percent(100.0),
                    height: Val::Px(4.0),
                    border: UiRect::all(Val::Px(1.0)),
                    ..default()
                },
                BackgroundColor(UiTheme::void_black().into()),
                BorderColor::from(UiTheme::panel_border()),
            ))
            .with_children(|track| {
                track.spawn((
                    Node {
                        box_sizing: BoxSizing::BorderBox,
                        width: Val::Percent(0.0),
                        height: Val::Percent(100.0),
                        ..default()
                    },
                    BackgroundColor(UiTheme::body_dim().mix(&UiTheme::danger(), 0.35).into()),
                    PlaybackFoeAltCastFill,
                ));
            });
            col.spawn((
                Node {
                    box_sizing: BoxSizing::BorderBox,
                    width: Val::Percent(100.0),
                    height: Val::Px(4.0),
                    border: UiRect::all(Val::Px(1.0)),
                    ..default()
                },
                BackgroundColor(UiTheme::void_black().into()),
                BorderColor::from(UiTheme::panel_border()),
            ))
            .with_children(|track| {
                track.spawn((
                    Node {
                        box_sizing: BoxSizing::BorderBox,
                        width: Val::Percent(0.0),
                        height: Val::Percent(100.0),
                        ..default()
                    },
                    BackgroundColor(UiTheme::stone_highlight().into()),
                    PlaybackFoeAltCdFill,
                ));
            });
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
            col.spawn((
                Node {
                    box_sizing: BoxSizing::BorderBox,
                    width: Val::Percent(100.0),
                    height: Val::Px(5.0),
                    border: UiRect::all(Val::Px(1.0)),
                    ..default()
                },
                BackgroundColor(UiTheme::void_black().into()),
                BorderColor::from(UiTheme::panel_border()),
            ))
            .with_children(|track| {
                track.spawn((
                    Node {
                        box_sizing: BoxSizing::BorderBox,
                        width: Val::Percent(0.0),
                        height: Val::Percent(100.0),
                        ..default()
                    },
                    BackgroundColor(UiTheme::danger().mix(&Color::WHITE, 0.25).into()),
                    PlaybackFoeCastFill,
                ));
            });
            col.spawn((
                Node {
                    box_sizing: BoxSizing::BorderBox,
                    width: Val::Percent(100.0),
                    height: Val::Px(5.0),
                    border: UiRect::all(Val::Px(1.0)),
                    ..default()
                },
                BackgroundColor(UiTheme::void_black().into()),
                BorderColor::from(UiTheme::panel_border()),
            ))
            .with_children(|track| {
                track.spawn((
                    Node {
                        box_sizing: BoxSizing::BorderBox,
                        width: Val::Percent(0.0),
                        height: Val::Percent(100.0),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.35, 0.22, 0.22).into()),
                    PlaybackFoeCdFill,
                ));
            });
        });
}
