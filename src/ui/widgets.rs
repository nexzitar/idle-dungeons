//! Panel primitives and atmospheric backdrop layering.

use bevy::prelude::*;
use bevy::text::{TextColor, TextFont};
use bevy::ui::FocusPolicy;

use crate::ui::components::{SettingsButton, TopBarField};
use crate::ui::theme::UiTheme;

pub use crate::ui::primitives::scroll::{
    spawn_scroll_viewport, spawn_scrollable_flex_column, spawn_scrollable_log,
};
pub use crate::ui::primitives::{
    spawn_bottom_strip, spawn_button, spawn_framed_panel, UiButtonConfig, UiButtonVariant,
};

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

            row.spawn((Node {
                box_sizing: BoxSizing::BorderBox,
                flex_grow: 1.0,
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::FlexEnd,
                align_items: AlignItems::Center,
                flex_wrap: FlexWrap::Wrap,
                column_gap: Val::Px(18.0),
                row_gap: Val::Px(6.0),
                ..default()
            },))
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

            let settings = spawn_button(
                row,
                UiButtonConfig {
                    label: "Settings",
                    variant: UiButtonVariant::PanelOutlined,
                    width: Val::Px(96.0),
                    height: Val::Px(34.0),
                    font_size: UiTheme::FONT_COMPACT,
                    text_color: Color::WHITE,
                    flex_shrink: 1.0,
                },
            );
            row.commands_mut().entity(settings).insert(SettingsButton);
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
