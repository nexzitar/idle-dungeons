//! Full-width framed panels backed by [`crate::ui::primitives::scroll::spawn_scroll_viewport`].

use bevy::prelude::*;

use crate::ui::primitives::scroll::spawn_scroll_viewport;
use crate::ui::theme::UiTheme;

pub fn spawn_framed_panel(
    parent: &mut ChildSpawnerCommands<'_>,
    flex: f32,
    content: impl FnOnce(&mut ChildSpawnerCommands<'_>),
) {
    parent
        .spawn((
            Node {
                box_sizing: BoxSizing::BorderBox,
                flex_grow: flex,
                flex_basis: Val::Px(0.0),
                flex_shrink: 1.0,
                min_width: Val::Px(220.0),
                min_height: Val::Px(0.0),
                padding: UiRect::all(Val::Px(UiTheme::PANEL_INSET_LG)),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Stretch,
                row_gap: Val::Px(10.0),
                border: UiRect::all(Val::Px(1.0)),
                overflow: Overflow::clip_y(),
                ..default()
            },
            BackgroundColor(UiTheme::panel_bg()),
            BorderColor::from(UiTheme::panel_border()),
        ))
        .with_children(|panel| {
            spawn_scroll_viewport(panel, content);
        });
}

pub fn spawn_bottom_strip(
    parent: &mut ChildSpawnerCommands<'_>,
    content: impl FnOnce(&mut ChildSpawnerCommands<'_>),
) {
    parent
        .spawn((
            Node {
                box_sizing: BoxSizing::BorderBox,
                width: Val::Percent(100.0),
                flex_grow: 0.0,
                flex_shrink: 0.0,
                min_height: Val::Px(120.0),
                max_height: Val::Percent(38.0),
                padding: UiRect::all(Val::Px(UiTheme::PANEL_INSET_LG)),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Stretch,
                row_gap: Val::Px(12.0),
                border: UiRect::all(Val::Px(1.0)),
                overflow: Overflow::clip_y(),
                ..default()
            },
            BackgroundColor(UiTheme::panel_bg()),
            BorderColor::from(UiTheme::panel_border()),
        ))
        .with_children(|strip| {
            spawn_scroll_viewport(strip, content);
        });
}
