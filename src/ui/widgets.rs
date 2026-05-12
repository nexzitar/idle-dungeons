//! Panel primitives and atmospheric backdrop layering.

use bevy::prelude::*;
use bevy::ui::{FocusPolicy, RelativeCursorPosition};
use bevy::text::{TextColor, TextFont};

use crate::ui::components::{
    SettingsButton, TopBarField, UiButtonPalette, UiScrollContent, UiScrollRegion, UiScrollState,
};
use crate::ui::theme::UiTheme;

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

pub fn spawn_top_resource_bar(
    parent: &mut ChildSpawnerCommands<'_>,
    gold: u32,
    salvage: u32,
    skill_slots: usize,
    depth_label: &str,
) {
    parent
        .spawn((
            Node {
                box_sizing: BoxSizing::BorderBox,
                width: Val::Percent(100.0),
                min_height: Val::Px(52.0),
                flex_shrink: 0.0,
                padding: UiRect::axes(Val::Px(UiTheme::PAD_ROOT), Val::Px(UiTheme::PAD_BAR_Y)),
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::SpaceBetween,
                column_gap: Val::Px(16.0),
                border: UiRect::bottom(Val::Px(2.0)),
                ..default()
            },
            BackgroundColor(UiTheme::panel_bg_deep()),
            BorderColor::from(UiTheme::panel_border()),
        ))
        .with_children(|row| {
            row.spawn((
                Text::new("Delvers"),
                TextFont::from_font_size(UiTheme::FONT_TITLE),
                TextColor(UiTheme::muted_gold()),
            ));

            row.spawn((
                Node {
                    box_sizing: BoxSizing::BorderBox,
                    flex_grow: 1.0,
                    flex_direction: FlexDirection::Row,
                    justify_content: JustifyContent::FlexEnd,
                    align_items: AlignItems::Center,
                    flex_wrap: FlexWrap::Wrap,
                    column_gap: Val::Px(18.0),
                    row_gap: Val::Px(6.0),
                    ..default()
                },
            ))
            .with_children(|metrics| {
                metric_chip(metrics, TopBarField::Gold, format!("Gold: {gold}"));
                metric_chip(metrics, TopBarField::Salvage, format!("Salvage: {salvage}"));
                metric_chip(
                    metrics,
                    TopBarField::SkillSlots,
                    format!("Skills: {skill_slots}"),
                );
                metric_chip(metrics, TopBarField::Depth, format!("Depth: {depth_label}"));
            });

            let p = UiButtonPalette::panel_outlined();
            row.spawn((
                Node {
                    box_sizing: BoxSizing::BorderBox,
                    width: Val::Px(96.0),
                    height: Val::Px(34.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    border: UiRect::all(Val::Px(1.0)),
                    ..default()
                },
                Button,
                BackgroundColor(p.idle_bg),
                BorderColor::from(p.idle_border),
                SettingsButton,
                p,
            ))
            .with_children(|btn| {
                btn.spawn((
                    Text::new("Settings"),
                    TextFont::from_font_size(UiTheme::FONT_COMPACT),
                    TextColor(Color::WHITE),
                ));
            });
        });
}

fn metric_chip(parent: &mut ChildSpawnerCommands<'_>, field: TopBarField, label: String) {
    parent.spawn((
        Text::new(label),
        TextFont::from_font_size(UiTheme::FONT_COMPACT),
        TextColor(UiTheme::body()),
        field,
    ));
}

fn spawn_panel_scroll_viewport(parent: &mut ChildSpawnerCommands<'_>, content: impl FnOnce(&mut ChildSpawnerCommands<'_>)) {
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
                min_height: min_viewport_height_px
                    .map(Val::Px)
                    .unwrap_or(Val::Px(0.0)),
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
            spawn_panel_scroll_viewport(panel, content);
        });
}

pub fn spawn_bottom_strip(parent: &mut ChildSpawnerCommands<'_>, content: impl FnOnce(&mut ChildSpawnerCommands<'_>)) {
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
            spawn_panel_scroll_viewport(strip, content);
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
