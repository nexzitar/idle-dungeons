//! Horizontal fill bars (playback HP-style tracks).

use bevy::prelude::*;

use crate::ui::theme::UiTheme;

pub const PLAYBACK_HP_HEIGHT_PX: f32 = 14.0;
pub const PLAYBACK_CAST_HEIGHT_PX: f32 = 5.0;
pub const PLAYBACK_THIN_HEIGHT_PX: f32 = 4.0;

#[derive(Clone, Copy, Debug)]
pub struct UiBarStyle {
    pub track: Color,
    pub fill: Color,
    pub height_px: f32,
    /// When true, a 1 px panel-border outline like playback HP bars.
    pub border: bool,
}

impl UiBarStyle {
    fn playback_track() -> Color {
        UiTheme::void_black()
    }

    fn playback_bordered(track: Color, fill: Color, height_px: f32) -> Self {
        Self {
            track,
            fill,
            height_px,
            border: true,
        }
    }

    pub fn playback_hp_lead() -> Self {
        Self::playback_bordered(
            Self::playback_track(),
            UiTheme::healing(),
            PLAYBACK_HP_HEIGHT_PX,
        )
    }

    pub fn playback_hp_ally() -> Self {
        Self::playback_bordered(
            Self::playback_track(),
            Color::srgb(0.38, 0.72, 0.92),
            PLAYBACK_HP_HEIGHT_PX,
        )
    }

    pub fn playback_hp_enemy() -> Self {
        Self::playback_bordered(
            Self::playback_track(),
            UiTheme::danger(),
            PLAYBACK_HP_HEIGHT_PX,
        )
    }

    pub fn playback_cast_lead() -> Self {
        Self::playback_bordered(
            Self::playback_track(),
            UiTheme::muted_gold(),
            PLAYBACK_CAST_HEIGHT_PX,
        )
    }

    pub fn playback_cast_ally() -> Self {
        Self::playback_bordered(
            Self::playback_track(),
            Color::srgb(0.38, 0.72, 0.92),
            PLAYBACK_CAST_HEIGHT_PX,
        )
    }

    pub fn playback_cast_foe() -> Self {
        Self::playback_bordered(
            Self::playback_track(),
            UiTheme::danger().mix(&Color::WHITE, 0.25),
            PLAYBACK_CAST_HEIGHT_PX,
        )
    }

    pub fn playback_cast_foe_alt() -> Self {
        Self::playback_bordered(
            Self::playback_track(),
            UiTheme::body_dim().mix(&UiTheme::danger(), 0.35),
            PLAYBACK_THIN_HEIGHT_PX,
        )
    }

    pub fn playback_cd_lead() -> Self {
        Self::playback_bordered(
            Self::playback_track(),
            Color::srgb(0.28, 0.32, 0.42),
            PLAYBACK_CAST_HEIGHT_PX,
        )
    }

    pub fn playback_cd_ally() -> Self {
        Self::playback_bordered(
            Self::playback_track(),
            Color::srgb(0.22, 0.36, 0.48),
            PLAYBACK_CAST_HEIGHT_PX,
        )
    }

    pub fn playback_cd_foe() -> Self {
        Self::playback_bordered(
            Self::playback_track(),
            Color::srgb(0.35, 0.22, 0.22),
            PLAYBACK_CAST_HEIGHT_PX,
        )
    }

    pub fn playback_cd_foe_alt() -> Self {
        Self::playback_bordered(
            Self::playback_track(),
            UiTheme::stone_highlight(),
            PLAYBACK_THIN_HEIGHT_PX,
        )
    }

    pub fn playback_skill_gcd_lead() -> Self {
        Self::playback_bordered(
            Self::playback_track(),
            Color::srgb(0.52, 0.38, 0.62),
            PLAYBACK_THIN_HEIGHT_PX,
        )
    }

    pub fn playback_skill_gcd_ally() -> Self {
        Self::playback_bordered(
            Self::playback_track(),
            Color::srgb(0.32, 0.48, 0.62),
            PLAYBACK_THIN_HEIGHT_PX,
        )
    }

    pub fn playback_instant_recharge_lead() -> Self {
        Self::playback_bordered(
            Self::playback_track(),
            Color::srgb(0.34, 0.52, 0.40),
            PLAYBACK_THIN_HEIGHT_PX,
        )
    }

    pub fn playback_instant_recharge_ally() -> Self {
        Self::playback_bordered(
            Self::playback_track(),
            Color::srgb(0.28, 0.55, 0.45),
            PLAYBACK_THIN_HEIGHT_PX,
        )
    }
}

pub fn spawn_horizontal_bar<M: Component>(
    parent: &mut ChildSpawnerCommands<'_>,
    style: UiBarStyle,
    fill_marker: M,
    initial_fill_pct: f32,
) -> Entity {
    let pct = (initial_fill_pct * 100.0).clamp(0.0, 100.0);
    let mut track_node = Node {
        box_sizing: BoxSizing::BorderBox,
        width: Val::Percent(100.0),
        height: Val::Px(style.height_px),
        ..default()
    };
    if style.border {
        track_node.border = UiRect::all(Val::Px(1.0));
    }

    let mut cmds = parent.spawn((track_node, BackgroundColor(style.track.into())));
    if style.border {
        cmds.insert(BorderColor::from(UiTheme::panel_border()));
    }

    cmds.with_children(|bar| {
        bar.spawn((
            Node {
                box_sizing: BoxSizing::BorderBox,
                width: Val::Percent(pct),
                height: Val::Percent(100.0),
                ..default()
            },
            BackgroundColor(style.fill.into()),
            fill_marker,
        ));
    })
    .id()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn playback_hp_presets_use_standard_height() {
        assert_eq!(UiBarStyle::playback_hp_lead().height_px, PLAYBACK_HP_HEIGHT_PX);
        assert_eq!(UiBarStyle::playback_hp_enemy().height_px, PLAYBACK_HP_HEIGHT_PX);
    }

    #[test]
    fn playback_cast_presets_use_cast_height() {
        assert_eq!(UiBarStyle::playback_cast_lead().height_px, PLAYBACK_CAST_HEIGHT_PX);
        assert_eq!(
            UiBarStyle::playback_cast_foe_alt().height_px,
            PLAYBACK_THIN_HEIGHT_PX
        );
    }
}
