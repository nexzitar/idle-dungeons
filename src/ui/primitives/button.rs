//! Reusable bordered UI buttons backed by [`UiButtonPalette`] and hover systems.

use bevy::prelude::*;
use bevy::text::{TextColor, TextFont};
use bevy::ui::RelativeCursorPosition;

use crate::ui::components::UiButtonPalette;

#[derive(Clone, Copy, Debug)]
pub enum UiButtonVariant {
    Primary,
    Secondary,
    Danger,
    PanelOutlined,
    PanelSecondary,
    Equip,
    Salvage,
}

impl UiButtonVariant {
    pub fn palette(self) -> UiButtonPalette {
        match self {
            Self::Primary => UiButtonPalette::primary_cta(),
            Self::Secondary => UiButtonPalette::panel_outlined(),
            Self::Danger => UiButtonPalette::salvage(),
            Self::PanelOutlined => UiButtonPalette::panel_outlined(),
            Self::PanelSecondary => UiButtonPalette::panel_secondary(),
            Self::Equip => UiButtonPalette::equip(),
            Self::Salvage => UiButtonPalette::salvage(),
        }
    }
}

pub struct UiButtonConfig<'a> {
    pub label: &'a str,
    pub variant: UiButtonVariant,
    pub width: Val,
    pub height: Val,
    pub font_size: f32,
    pub text_color: Color,
    /// Used for flex layouts (e.g. modal footers): `0.0` keeps the button full-width.
    pub flex_shrink: f32,
}

pub fn spawn_button(parent: &mut ChildSpawnerCommands<'_>, config: UiButtonConfig<'_>) -> Entity {
    spawn_button_with_extra_text(parent, config, ())
}

pub fn spawn_button_with_extra_text<B: Bundle>(
    parent: &mut ChildSpawnerCommands<'_>,
    config: UiButtonConfig<'_>,
    extra_on_text: B,
) -> Entity {
    let pal = config.variant.palette();
    parent
        .spawn((
            Node {
                box_sizing: BoxSizing::BorderBox,
                width: config.width,
                height: config.height,
                flex_shrink: config.flex_shrink,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                border: UiRect::all(Val::Px(1.0)),
                ..default()
            },
            Button,
            BackgroundColor(pal.idle_bg),
            BorderColor::from(pal.idle_border),
            pal,
            Interaction::default(),
            RelativeCursorPosition::default(),
        ))
        .with_children(|btn| {
            btn.spawn((
                Text::new(config.label),
                TextFont::from_font_size(config.font_size),
                TextColor(config.text_color),
                extra_on_text,
            ));
        })
        .id()
}
