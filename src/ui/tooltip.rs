//! Cursor-following hover tooltips with a short delay.

use bevy::input::touch::Touches;
use bevy::prelude::*;
use bevy::ui::{FocusPolicy, ZIndex};
use bevy::text::{TextColor, TextFont};
use bevy::ui::ComputedNode;
use bevy::window::PrimaryWindow;

use crate::ui::components::UiTooltip;
use crate::ui::theme::UiTheme;

/// Root node for the tooltip panel (one per [`crate::ui::components::UiRoot`]).
#[derive(Component)]
pub struct TooltipLayer;

/// Text entity updated by [`update_tooltip`].
#[derive(Component)]
pub struct TooltipText;

#[derive(Resource)]
pub struct TooltipState {
    delay: Timer,
    tracked: Option<Entity>,
    /// While true, the tooltip stays hidden so it cannot sit above buttons between press and release.
    suppress_until_pointer_release: bool,
}

impl Default for TooltipState {
    fn default() -> Self {
        Self {
            delay: Timer::from_seconds(0.38, TimerMode::Once),
            tracked: None,
            suppress_until_pointer_release: false,
        }
    }
}

/// Spawn last under [`crate::ui::components::UiRoot`] so it draws above gameplay UI.
pub fn spawn_tooltip_layer(parent: &mut ChildSpawnerCommands<'_>) {
    parent
        .spawn((
            Node {
                box_sizing: BoxSizing::BorderBox,
                position_type: PositionType::Absolute,
                    left: Val::Px(0.0),
                    top: Val::Px(0.0),
                    width: Val::Auto,
                    max_width: Val::Px(280.0),
                    padding: UiRect::all(Val::Px(UiTheme::PAD_TOOLTIP)),
                    border: UiRect::all(Val::Px(1.0)),
                    ..default()
            },
            BackgroundColor(UiTheme::panel_bg_deep().into()),
            BorderColor::from(UiTheme::ornate_gold()),
            FocusPolicy::Pass,
            Visibility::Hidden,
            TooltipLayer,
        ))
        .with_children(|layer| {
            layer.spawn((
                Text::new(""),
                TextFont::from_font_size(UiTheme::FONT_CAPTION),
                TextColor(UiTheme::body()),
                FocusPolicy::Pass,
                TooltipText,
            ));
        });
}

/// Hide the tooltip **before** [`UiSystems::Focus`] runs so the panel does not consume the current
/// pointer press (see `FocusPolicy::Pass` quirks with deep UI trees / global z-index).
pub fn hide_tooltip_layer_before_pointer_focus(
    mouse: Res<ButtonInput<MouseButton>>,
    touches: Res<Touches>,
    mut state: ResMut<TooltipState>,
    mut layer_q: Query<&mut Visibility, With<TooltipLayer>>,
) {
    let press = mouse.just_pressed(MouseButton::Left) || touches.any_just_pressed();
    if !press {
        return;
    }
    state.suppress_until_pointer_release = true;
    state.tracked = None;
    state.delay.reset();
    for mut vis in &mut layer_q {
        *vis = Visibility::Hidden;
    }
}

fn tooltip_interaction_ok(i: Interaction) -> bool {
    matches!(i, Interaction::Hovered | Interaction::Pressed)
}

pub fn update_tooltip(
    time: Res<Time>,
    mouse: Res<ButtonInput<MouseButton>>,
    touches: Res<Touches>,
    mut state: ResMut<TooltipState>,
    windows: Query<&Window, With<PrimaryWindow>>,
    mut layer_q: Query<(&mut Node, &mut Visibility, &ComputedNode), With<TooltipLayer>>,
    mut text_q: Query<&mut Text, With<TooltipText>>,
    tooltip_targets: Query<(Entity, &Interaction, &UiTooltip), With<UiTooltip>>,
) {
    let Ok(window) = windows.single() else {
        return;
    };
    let cursor = window.cursor_position();

    let mut hovered: Option<(Entity, &str)> = None;
    for (entity, interaction, tip) in &tooltip_targets {
        if tooltip_interaction_ok(*interaction) {
            hovered = Some((entity, tip.0.as_str()));
        }
    }

    let Ok((mut panel, mut vis, computed)) = layer_q.single_mut() else {
        return;
    };
    let Ok(mut text) = text_q.single_mut() else {
        return;
    };

    let mut hide = || {
        *vis = Visibility::Hidden;
    };

    let released = mouse.just_released(MouseButton::Left) || touches.any_just_released();
    if state.suppress_until_pointer_release {
        if released {
            state.suppress_until_pointer_release = false;
        } else {
            state.tracked = None;
            state.delay.reset();
            hide();
            return;
        }
    }

    match hovered {
        None => {
            state.tracked = None;
            state.delay.reset();
            hide();
        }
        Some((e, content)) => {
            if state.tracked != Some(e) {
                state.tracked = Some(e);
                state.delay.reset();
                hide();
            }
            state.delay.tick(time.delta());
            if !state.delay.is_finished() {
                return;
            }
            let content_owned = content.to_string();
            if text.0 != content_owned {
                text.0.clone_from(&content_owned);
            }
            let Some(pos) = cursor else {
                hide();
                return;
            };
            let w = window.width();
            let h = window.height();
            let m = 12.0;
            let offset = 14.0;
            let mut x = pos.x + offset;
            let mut y = pos.y + offset;
            let tw = computed.size.x.max(120.0);
            let th = computed.size.y.max(36.0);
            if x + tw + m > w {
                x = (w - tw - m).max(m);
            }
            if y + th + m > h {
                y = (h - th - m).max(m);
            }
            if x < m {
                x = m;
            }
            if y < m {
                y = m;
            }
            panel.left = Val::Px(x);
            panel.top = Val::Px(y);
            *vis = Visibility::Visible;
        }
    }
}
