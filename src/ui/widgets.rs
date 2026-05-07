//! Panel primitives and atmospheric backdrop layering.

use bevy::prelude::*;

use crate::ui::components::{SettingsButton, TopBarField};
use crate::ui::theme::UiTheme;

/// Full-viewport background stack: subtle stone bands + soft torch tint.
pub fn spawn_atmosphere(parent: &mut ChildBuilder) {
    parent
        .spawn(NodeBundle {
            style: Style {
                position_type: PositionType::Absolute,
                left: Val::Px(0.0),
                right: Val::Px(0.0),
                top: Val::Px(0.0),
                bottom: Val::Px(0.0),
                flex_direction: FlexDirection::Column,
                ..default()
            },
            ..default()
        })
        .with_children(|layer| {
            layer.spawn(NodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    height: Val::Percent(38.0),
                    ..default()
                },
                background_color: UiTheme::stone_highlight().into(),
                ..default()
            });
            layer.spawn(NodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    flex_grow: 1.0,
                    ..default()
                },
                background_color: UiTheme::stone_mid().into(),
                ..default()
            });
            layer.spawn(NodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    height: Val::Percent(28.0),
                    ..default()
                },
                background_color: UiTheme::stone_deep().into(),
                ..default()
            });
        });

    parent.spawn(NodeBundle {
        style: Style {
            position_type: PositionType::Absolute,
            left: Val::Px(0.0),
            right: Val::Px(0.0),
            top: Val::Px(0.0),
            bottom: Val::Px(0.0),
            ..default()
        },
        background_color: UiTheme::torch_glow().into(),
        ..default()
    });

    parent.spawn(NodeBundle {
        style: Style {
            position_type: PositionType::Absolute,
            left: Val::Px(0.0),
            right: Val::Px(0.0),
            top: Val::Px(0.0),
            bottom: Val::Px(0.0),
            border: UiRect::all(Val::Px(56.0)),
            ..default()
        },
        background_color: Color::srgba(0.0, 0.0, 0.0, 0.55).into(),
        ..default()
    });
}

pub fn spawn_top_resource_bar(
    parent: &mut ChildBuilder,
    gold: u32,
    salvage: u32,
    skill_slots: usize,
    depth_label: &str,
    speed_mult: f32,
) {
    parent
        .spawn(NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                min_height: Val::Px(52.0),
                padding: UiRect::axes(Val::Px(20.0), Val::Px(10.0)),
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::SpaceBetween,
                column_gap: Val::Px(16.0),
                border: UiRect::bottom(Val::Px(2.0)),
                ..default()
            },
            background_color: UiTheme::panel_bg_deep().into(),
            border_color: BorderColor(UiTheme::panel_border()),
            ..default()
        })
        .with_children(|row| {
            row.spawn(TextBundle::from_section(
                "Idle Dungeons",
                TextStyle {
                    font_size: 22.0,
                    color: UiTheme::muted_gold(),
                    ..default()
                },
            ));

            row.spawn(NodeBundle {
                style: Style {
                    flex_grow: 1.0,
                    flex_direction: FlexDirection::Row,
                    justify_content: JustifyContent::FlexEnd,
                    align_items: AlignItems::Center,
                    flex_wrap: FlexWrap::Wrap,
                    column_gap: Val::Px(18.0),
                    row_gap: Val::Px(6.0),
                    ..default()
                },
                ..default()
            })
            .with_children(|metrics| {
                metric_chip(metrics, TopBarField::Gold, format!("Gold: {gold}"));
                metric_chip(metrics, TopBarField::Salvage, format!("Salvage: {salvage}"));
                metric_chip(
                    metrics,
                    TopBarField::SkillSlots,
                    format!("Skills: {skill_slots}"),
                );
                metric_chip(metrics, TopBarField::Depth, format!("Depth: {depth_label}"));
                metric_chip(
                    metrics,
                    TopBarField::Speed,
                    format!("Speed: {}x", fmt_speed(speed_mult)),
                );
            });

            row.spawn((
                ButtonBundle {
                    style: Style {
                        width: Val::Px(96.0),
                        height: Val::Px(34.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    background_color: UiTheme::muted_red().into(),
                    ..default()
                },
                SettingsButton,
            ))
            .with_children(|btn| {
                btn.spawn(TextBundle::from_section(
                    "Settings",
                    TextStyle {
                        font_size: 14.0,
                        color: Color::WHITE,
                        ..default()
                    },
                ));
            });
        });
}

fn metric_chip(parent: &mut ChildBuilder, field: TopBarField, label: String) {
    parent.spawn((
        TextBundle::from_section(
            label,
            TextStyle {
                font_size: 14.0,
                color: UiTheme::body(),
                ..default()
            },
        ),
        field,
    ));
}

fn fmt_speed(mult: f32) -> String {
    if (mult - 1.0).abs() < f32::EPSILON {
        "1".to_string()
    } else if (mult - 2.0).abs() < f32::EPSILON {
        "2".to_string()
    } else {
        format!("{mult:.1}")
    }
}

pub fn spawn_framed_panel(
    parent: &mut ChildBuilder,
    flex: f32,
    content: impl FnOnce(&mut ChildBuilder),
) {
    parent
        .spawn(NodeBundle {
            style: Style {
                flex_grow: flex,
                flex_basis: Val::Px(0.0),
                min_width: Val::Px(220.0),
                min_height: Val::Px(200.0),
                padding: UiRect::all(Val::Px(14.0)),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Stretch,
                row_gap: Val::Px(10.0),
                border: UiRect::all(Val::Px(1.0)),
                ..default()
            },
            background_color: UiTheme::panel_bg().into(),
            border_color: BorderColor(UiTheme::panel_border()),
            ..default()
        })
        .with_children(content);
}

pub fn spawn_bottom_strip(parent: &mut ChildBuilder, content: impl FnOnce(&mut ChildBuilder)) {
    parent
        .spawn(NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                flex_grow: 1.0,
                min_height: Val::Px(120.0),
                max_height: Val::Percent(45.0),
                padding: UiRect::all(Val::Px(14.0)),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Stretch,
                row_gap: Val::Px(12.0),
                border: UiRect::all(Val::Px(1.0)),
                overflow: Overflow::clip_y(),
                ..default()
            },
            background_color: UiTheme::panel_bg().into(),
            border_color: BorderColor(UiTheme::panel_border()),
            ..default()
        })
        .with_children(content);
}
