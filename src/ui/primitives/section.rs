//! Section headers with consistent vertical rhythm.

use bevy::prelude::*;

use crate::ui::theme::{section_title, UiTheme};

/// Section label plus spacer — use at the top of a panel territory.
pub fn spawn_framed_section_header(parent: &mut ChildSpawnerCommands<'_>, title: &str) {
    parent
        .spawn((
            Node {
                box_sizing: BoxSizing::BorderBox,
                width: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(4.0),
                flex_shrink: 0.0,
                ..default()
            },
        ))
        .with_children(|head| {
            head.spawn(section_title(title));
            head.spawn((
                Node {
                    box_sizing: BoxSizing::BorderBox,
                    width: Val::Percent(100.0),
                    height: Val::Px(1.0),
                    ..default()
                },
                BackgroundColor(UiTheme::panel_border_inner()),
            ));
        });
    parent.spawn(Node {
        box_sizing: BoxSizing::BorderBox,
        height: Val::Px(6.0),
        flex_shrink: 0.0,
        ..default()
    });
}
