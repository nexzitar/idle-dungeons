//! Full-screen UI roots (build, running playback, summary).

use bevy::prelude::*;

pub mod build;
pub mod running;
pub mod summary;

pub(crate) use build::{spawn_build_screen, spawn_build_screen_root};
pub(crate) use running::{spawn_running_screen, spawn_running_screen_root};
pub(crate) use summary::{spawn_summary_screen, spawn_summary_screen_root};

pub(crate) fn root_shell() -> impl Bundle {
    (
        Node {
            box_sizing: BoxSizing::BorderBox,
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            position_type: PositionType::Relative,
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Stretch,
            ..default()
        },
        BackgroundColor(Color::NONE),
    )
}

pub(crate) fn content_column_bundle() -> impl Bundle {
    (Node {
        box_sizing: BoxSizing::BorderBox,
        width: Val::Percent(100.0),
        flex_grow: 1.0,
        min_height: Val::Px(0.0),
        flex_direction: FlexDirection::Column,
        padding: UiRect::axes(Val::Px(18.0), Val::Px(14.0)),
        row_gap: Val::Px(12.0),
        align_items: AlignItems::Stretch,
        ..default()
    },)
}
