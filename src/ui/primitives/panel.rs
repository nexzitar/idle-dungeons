//! Full-width framed panels backed by [`crate::ui::primitives::scroll::spawn_scroll_viewport`].

use bevy::prelude::*;
use bevy::ui::FocusPolicy;

use crate::ui::primitives::scroll::spawn_scroll_viewport;
use crate::ui::theme::{UiPanelStyle, UiTheme};

/// Full-screen atmospheric backdrop (stone bands, torch glow, vignette).
pub fn spawn_atmosphere(parent: &mut ChildSpawnerCommands<'_>) {
    parent
        .spawn((
            Node {
                box_sizing: BoxSizing::BorderBox,
                position_type: PositionType::Absolute,
                left: Val::Px(0.0),
                right: Val::Px(0.0),
                top: Val::Px(0.0),
                bottom: Val::Px(0.0),
                flex_direction: FlexDirection::Column,
                ..default()
            },
            FocusPolicy::Pass,
        ))
        .with_children(|layer| {
            layer.spawn((
                Node {
                    box_sizing: BoxSizing::BorderBox,
                    width: Val::Percent(100.0),
                    height: Val::Percent(38.0),
                    ..default()
                },
                BackgroundColor(UiTheme::stone_highlight()),
                FocusPolicy::Pass,
            ));
            layer.spawn((
                Node {
                    box_sizing: BoxSizing::BorderBox,
                    width: Val::Percent(100.0),
                    flex_grow: 1.0,
                    ..default()
                },
                BackgroundColor(UiTheme::stone_mid()),
                FocusPolicy::Pass,
            ));
            layer.spawn((
                Node {
                    box_sizing: BoxSizing::BorderBox,
                    width: Val::Percent(100.0),
                    height: Val::Percent(28.0),
                    ..default()
                },
                BackgroundColor(UiTheme::stone_deep()),
                FocusPolicy::Pass,
            ));
        });

    parent.spawn((
        Node {
            box_sizing: BoxSizing::BorderBox,
            position_type: PositionType::Absolute,
            left: Val::Px(0.0),
            right: Val::Px(0.0),
            top: Val::Px(0.0),
            bottom: Val::Px(0.0),
            ..default()
        },
        BackgroundColor(UiTheme::torch_glow()),
        FocusPolicy::Pass,
    ));

    parent.spawn((
        Node {
            box_sizing: BoxSizing::BorderBox,
            position_type: PositionType::Absolute,
            left: Val::Px(0.0),
            right: Val::Px(0.0),
            top: Val::Px(0.0),
            bottom: Val::Px(0.0),
            border: UiRect::all(Val::Px(56.0)),
            ..default()
        },
        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.55)),
        FocusPolicy::Pass,
    ));
}

pub fn spawn_framed_panel(
    parent: &mut ChildSpawnerCommands<'_>,
    flex: f32,
    content: impl FnOnce(&mut ChildSpawnerCommands<'_>),
) {
    let style = UiPanelStyle::framed();
    parent
        .spawn((
            Node {
                box_sizing: BoxSizing::BorderBox,
                flex_grow: flex,
                flex_basis: Val::Px(0.0),
                flex_shrink: 1.0,
                min_width: Val::Px(220.0),
                min_height: Val::Px(0.0),
                padding: UiRect::all(Val::Px(style.padding_px)),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Stretch,
                row_gap: Val::Px(style.row_gap_px),
                border: UiRect::all(Val::Px(style.border_px)),
                overflow: Overflow::clip_y(),
                ..default()
            },
            BackgroundColor(style.background),
            BorderColor::from(style.border),
        ))
        .with_children(|panel| {
            spawn_scroll_viewport(panel, content);
        });
}

pub fn spawn_bottom_strip(
    parent: &mut ChildSpawnerCommands<'_>,
    content: impl FnOnce(&mut ChildSpawnerCommands<'_>),
) {
    let style = UiPanelStyle::bottom_strip();
    parent
        .spawn((
            Node {
                box_sizing: BoxSizing::BorderBox,
                width: Val::Percent(100.0),
                flex_grow: 0.0,
                flex_shrink: 0.0,
                min_height: Val::Px(120.0),
                max_height: Val::Percent(38.0),
                padding: UiRect::all(Val::Px(style.padding_px)),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Stretch,
                row_gap: Val::Px(style.row_gap_px),
                border: UiRect::all(Val::Px(style.border_px)),
                overflow: Overflow::clip_y(),
                ..default()
            },
            BackgroundColor(style.background),
            BorderColor::from(style.border),
        ))
        .with_children(|strip| {
            spawn_scroll_viewport(strip, content);
        });
}
