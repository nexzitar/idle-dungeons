//! Scroll viewport primitives and wheel scroll systems.

use bevy::input::mouse::MouseWheel;
use bevy::prelude::*;
use bevy::text::{TextColor, TextFont};
use bevy::ui::{ComputedNode, FocusPolicy, RelativeCursorPosition};

use crate::app::ActiveRunPlayback;
use crate::ui::components::{
    PlaybackLogScrollRegion, UiScrollContent, UiScrollRegion, UiScrollState,
};
use crate::ui::theme::UiTheme;

use crate::ui::PlaybackCombatLogVisible;

pub fn spawn_scroll_viewport(
    parent: &mut ChildSpawnerCommands<'_>,
    content: impl FnOnce(&mut ChildSpawnerCommands<'_>),
) {
    parent
        .spawn((
            Node {
                box_sizing: BoxSizing::BorderBox,
                width: Val::Percent(100.0),
                flex_grow: 1.0,
                flex_shrink: 1.0,
                min_height: Val::Px(0.0),
                position_type: PositionType::Relative,
                flex_direction: FlexDirection::Column,
                overflow: Overflow::clip_y(),
                ..default()
            },
            FocusPolicy::Pass,
            RelativeCursorPosition::default(),
            UiScrollState::default(),
            UiScrollRegion,
        ))
        .with_children(|viewport| {
            viewport
                .spawn((
                    Node {
                        box_sizing: BoxSizing::BorderBox,
                        position_type: PositionType::Absolute,
                        left: Val::Px(0.0),
                        right: Val::Px(0.0),
                        top: Val::Px(0.0),
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::FlexStart,
                        row_gap: Val::Px(10.0),
                        ..default()
                    },
                    UiScrollContent,
                ))
                .with_children(content);
        });
}

/// Fills remaining column height in a flex parent; scrolls when content exceeds the viewport.
///
/// When `min_viewport_height_px` is set, the viewport is at least that tall (stops flex from
/// collapsing empty lists).
pub fn spawn_scrollable_flex_column(
    parent: &mut ChildSpawnerCommands<'_>,
    min_viewport_height_px: Option<f32>,
    content: impl FnOnce(&mut ChildSpawnerCommands<'_>),
) {
    parent
        .spawn((
            Node {
                box_sizing: BoxSizing::BorderBox,
                width: Val::Percent(100.0),
                flex_grow: 1.0,
                flex_shrink: 1.0,
                flex_basis: Val::Px(0.0),
                min_height: min_viewport_height_px.map(Val::Px).unwrap_or(Val::Px(0.0)),
                position_type: PositionType::Relative,
                overflow: Overflow::clip_y(),
                ..default()
            },
            FocusPolicy::Pass,
            RelativeCursorPosition::default(),
            UiScrollState::default(),
            UiScrollRegion,
        ))
        .with_children(|vp| {
            vp.spawn((
                Node {
                    box_sizing: BoxSizing::BorderBox,
                    position_type: PositionType::Absolute,
                    left: Val::Px(0.0),
                    right: Val::Px(0.0),
                    top: Val::Px(0.0),
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(8.0),
                    align_items: AlignItems::Stretch,
                    ..default()
                },
                UiScrollContent,
            ))
            .with_children(content);
        });
}

/// Log list with fixed viewport height and wheel scrolling.
pub fn spawn_scrollable_log(
    parent: &mut ChildSpawnerCommands<'_>,
    max_height_px: f32,
    lines: Vec<(String, f32, Color)>,
) {
    parent
        .spawn((
            Node {
                box_sizing: BoxSizing::BorderBox,
                width: Val::Percent(100.0),
                height: Val::Px(max_height_px),
                flex_shrink: 0.0,
                padding: UiRect::all(Val::Px(UiTheme::PAD_TOOLTIP)),
                position_type: PositionType::Relative,
                flex_direction: FlexDirection::Column,
                overflow: Overflow::clip_y(),
                border: UiRect::all(Val::Px(1.0)),
                ..default()
            },
            BackgroundColor(UiTheme::panel_bg_deep()),
            BorderColor::from(UiTheme::panel_border_inner()),
            FocusPolicy::Pass,
            RelativeCursorPosition::default(),
            UiScrollState::default(),
            UiScrollRegion,
        ))
        .with_children(|viewport| {
            viewport
                .spawn((
                    Node {
                        box_sizing: BoxSizing::BorderBox,
                        position_type: PositionType::Absolute,
                        left: Val::Px(0.0),
                        right: Val::Px(0.0),
                        top: Val::Px(0.0),
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::FlexStart,
                        row_gap: Val::Px(4.0),
                        ..default()
                    },
                    UiScrollContent,
                ))
                .with_children(|inner| {
                    for (text, font_size, color) in lines {
                        inner.spawn((
                            Text::new(text),
                            TextFont::from_font_size(font_size),
                            TextColor(color),
                        ));
                    }
                });
        });
}

pub(crate) fn pin_playback_combat_log_scroll(
    playback: Res<ActiveRunPlayback>,
    vis: Res<PlaybackCombatLogVisible>,
    mut prev_log: Local<String>,
    mut regions: Query<(Entity, &mut UiScrollState, &ComputedNode), With<PlaybackLogScrollRegion>>,
    children: Query<&Children>,
    mut content_set: ParamSet<(
        Query<&ComputedNode, With<UiScrollContent>>,
        Query<&mut Node, With<UiScrollContent>>,
    )>,
) {
    if !vis.0 {
        return;
    }
    let body = playback.log_lines.join("\n");
    if body == *prev_log {
        return;
    }
    *prev_log = body.clone();

    for (entity, mut state, viewport_node) in &mut regions {
        let view_h = viewport_node.size().y;
        if view_h <= 0.0 {
            continue;
        }
        let Ok(ch) = children.get(entity) else {
            continue;
        };
        let mut scroll_child = None;
        for e in ch.iter() {
            if content_set.p0().get(e).is_ok() {
                scroll_child = Some(e);
                break;
            }
        }
        let Some(child) = scroll_child else {
            continue;
        };
        let content_h = content_set
            .p0()
            .get(child)
            .map(|n| n.size().y)
            .unwrap_or(0.0);
        let max_scroll = (content_h - view_h).max(0.0);
        state.offset = max_scroll;
        if let Ok(mut style) = content_set.p1().get_mut(child) {
            style.top = Val::Px(-state.offset);
        }
    }
}

pub fn apply_ui_scroll(
    mut wheel_events: MessageReader<MouseWheel>,
    mut regions: Query<
        (
            Entity,
            &RelativeCursorPosition,
            &mut UiScrollState,
            &ComputedNode,
        ),
        With<UiScrollRegion>,
    >,
    children: Query<&Children>,
    mut content_style: Query<&mut Node, With<UiScrollContent>>,
    content_node: Query<&ComputedNode, With<UiScrollContent>>,
) {
    // Inverted from raw wheel delta: scroll feels like "grab and drag" the content.
    // Pixels per wheel notch (lower = slower). Trackpads accumulate small y values.
    const SCROLL_PIXELS_PER_LINE: f32 = 12.0;
    let delta: f32 = wheel_events
        .read()
        .map(|e| -e.y * SCROLL_PIXELS_PER_LINE)
        .sum();
    if delta.abs() < f32::EPSILON {
        return;
    }

    for (entity, rel_pos, mut state, viewport_node) in &mut regions {
        if !rel_pos.cursor_over() {
            continue;
        }
        let view_h = viewport_node.size().y;
        if view_h <= 0.0 {
            continue;
        }
        let Ok(ch) = children.get(entity) else {
            continue;
        };
        let mut scroll_child = None;
        for e in ch.iter() {
            if content_node.get(e).is_ok() {
                scroll_child = Some(e);
                break;
            }
        }
        let Some(child) = scroll_child else {
            continue;
        };
        let Ok(inner_node) = content_node.get(child) else {
            continue;
        };
        let content_h = inner_node.size().y;
        let max_scroll = (content_h - view_h).max(0.0);
        state.offset = (state.offset + delta).clamp(0.0, max_scroll);
        if let Ok(mut style) = content_style.get_mut(child) {
            style.top = Val::Px(-state.offset);
        }
        break;
    }
}
