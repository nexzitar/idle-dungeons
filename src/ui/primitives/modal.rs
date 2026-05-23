//! Full-screen modal shell: semi-opaque clickable backdrop plus centered layout row.

use bevy::prelude::*;
use bevy::ui::FocusPolicy;

use crate::ui::theme::UiModalStyle;

/// Marks the full-screen hit-blocking root spawned by [`spawn_modal_shell_with_handles`].
#[derive(Component)]
pub struct ModalShellRoot;

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
    /// Full-screen [`FocusPolicy::Block`] root — attach feature modal markers here (`SkillBookRoot`, etc.).
    pub shell_root: Entity,
    pub backdrop: Entity,
    pub content_root: Entity,
}

/// Spawns a full-screen translucent backdrop plus a fullscreen flex layer for centered modal content.
///
/// Returns the **`content`** entity (`content_root`): add modal UI as children of this node (via the
/// `content` closure). Use [`ModalShellHandles::shell_root`] for modal marker components and
/// [`ModalShellHandles::backdrop`] for backdrop click-to-close actions.
pub fn spawn_modal_shell(
    parent: &mut ChildSpawnerCommands<'_>,
    config: ModalShellConfig,
    content: impl FnOnce(&mut ChildSpawnerCommands<'_>),
) -> Entity {
    spawn_modal_shell_with_handles(parent, config, content).content_root
}

/// Same as [`spawn_modal_shell`] but exposes shell, backdrop, and content [`Entity`] handles.
pub fn spawn_modal_shell_with_handles(
    parent: &mut ChildSpawnerCommands<'_>,
    config: ModalShellConfig,
    content: impl FnOnce(&mut ChildSpawnerCommands<'_>),
) -> ModalShellHandles {
    let backdrop_pal = UiModalStyle::standard().backdrop_palette();

    let shell_root = parent
        .spawn((
            Node {
                box_sizing: BoxSizing::BorderBox,
                position_type: PositionType::Absolute,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                left: Val::Px(0.0),
                top: Val::Px(0.0),
                ..default()
            },
            FocusPolicy::Block,
            ModalShellRoot,
        ))
        .id();

    let mut backdrop = Entity::PLACEHOLDER;
    let mut content_root = Entity::PLACEHOLDER;

    parent.commands_mut().entity(shell_root).with_children(|layer| {
        backdrop = layer
            .spawn((
                Node {
                    box_sizing: BoxSizing::BorderBox,
                    position_type: PositionType::Absolute,
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    left: Val::Px(0.0),
                    top: Val::Px(0.0),
                    ..default()
                },
                BackgroundColor(backdrop_pal.idle_bg),
                BorderColor::from(backdrop_pal.idle_border),
                FocusPolicy::Block,
                backdrop_pal,
            ))
            .id();

        if config.backdrop_clicks_close {
            layer.commands_mut().entity(backdrop).insert(Button);
        }

        content_root = layer
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
                FocusPolicy::Block,
            ))
            .with_children(content)
            .id();
    });

    ModalShellHandles {
        shell_root,
        backdrop,
        content_root,
    }
}
