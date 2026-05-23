//! Full-screen modal shell: semi-opaque clickable backdrop plus centered layout row.

use bevy::prelude::*;
use bevy::ui::FocusPolicy;

use crate::ui::components::UiButtonPalette;

/// Layout and interaction for [`spawn_modal_shell`].
#[derive(Clone, Copy, Debug)]
pub struct ModalShellConfig {
    /// When true, spawns backdrop with [`Button`] so it participates in mouse hit tests like other overlay buttons.
    pub backdrop_clicks_close: bool,
}

impl Default for ModalShellConfig {
    fn default() -> Self {
        Self {
            backdrop_clicks_close: true,
        }
    }
}

/// Spawn targets for layering feature markers (`GearHubBackdrop`, etc.).
#[derive(Clone, Copy, Debug)]
pub struct ModalShellHandles {
    pub backdrop: Entity,
    pub content_root: Entity,
}

/// Spawns a full-screen translucent backdrop plus a fullscreen flex layer for centered modal content.
///
/// Returns the **`content`** entity (`content_root`): add modal UI as children of this node (via the
/// `content` closure). Use [`ModalShellHandles::backdrop`] from [`spawn_modal_shell_with_handles`]
/// when you need marker components (`GearHubBackdrop`, etc.).
pub fn spawn_modal_shell(
    parent: &mut ChildSpawnerCommands<'_>,
    config: ModalShellConfig,
    content: impl FnOnce(&mut ChildSpawnerCommands<'_>),
) -> Entity {
    spawn_modal_shell_with_handles(parent, config, content).content_root
}

/// Same as [`spawn_modal_shell`] but exposes the backdrop [`Entity`] so callers can attach feature markers.
pub fn spawn_modal_shell_with_handles(
    parent: &mut ChildSpawnerCommands<'_>,
    config: ModalShellConfig,
    content: impl FnOnce(&mut ChildSpawnerCommands<'_>),
) -> ModalShellHandles {
    let backdrop_pal = UiButtonPalette {
        idle_bg: Color::srgba(0.02, 0.02, 0.04, 0.58),
        hover_bg: Color::srgba(0.04, 0.04, 0.06, 0.65),
        pressed_bg: Color::srgba(0.06, 0.06, 0.08, 0.72),
        idle_border: Color::NONE,
        hover_border: Color::NONE,
        pressed_border: Color::NONE,
    };

    let backdrop = parent
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                left: Val::Px(0.0),
                top: Val::Px(0.0),
                ..default()
            },
            BackgroundColor(backdrop_pal.idle_bg),
            BorderColor::from(backdrop_pal.idle_border),
            backdrop_pal,
        ))
        .id();

    if config.backdrop_clicks_close {
        parent.commands_mut().entity(backdrop).insert(Button);
    }

    let content_root = parent
        .spawn((
            Node {
                box_sizing: BoxSizing::BorderBox,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                position_type: PositionType::Absolute,
                left: Val::Px(0.0),
                top: Val::Px(0.0),
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Stretch,
                ..default()
            },
            FocusPolicy::Pass,
        ))
        .with_children(content)
        .id();

    ModalShellHandles {
        backdrop,
        content_root,
    }
}
