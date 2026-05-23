use crate::ui::theme::UiTheme;
use bevy::prelude::*;

pub fn column_stretch() -> Node {
    Node {
        box_sizing: BoxSizing::BorderBox,
        width: Val::Percent(100.0),
        flex_direction: FlexDirection::Column,
        align_items: AlignItems::Stretch,
        ..default()
    }
}

pub fn row_gap(gap_px: f32) -> Node {
    Node {
        flex_direction: FlexDirection::Row,
        column_gap: Val::Px(gap_px),
        align_items: AlignItems::Center,
        ..default()
    }
}

pub const ROOT_PAD_X: f32 = UiTheme::PAD_ROOT;
pub const ROOT_PAD_Y: f32 = UiTheme::PAD_BAR_Y;
