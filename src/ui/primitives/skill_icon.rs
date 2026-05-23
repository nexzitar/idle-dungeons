//! Layered skill icon — base art plus hidden overlay hooks for future cooldown/GCD presentation.

use bevy::prelude::*;
use bevy::ui::FocusPolicy;

use crate::domain::party::PartyHeroKind;
use crate::domain::skills::SkillId;
use crate::ui::assets::UiPlaceholderImages;
use crate::ui::skill_presentation::{empty_icon, frame_border_for_skill, icon_for, locked_icon};
use crate::ui::theme::UiTheme;

/// Future sync target for playback / build overlays (Phase 3).
#[derive(Clone, Copy, Debug, Default)]
pub struct SkillIconOverlayState {
    pub cooldown_frac: f32,
    pub gcd_frac: f32,
    pub ready_pulse: bool,
    pub disabled: bool,
    pub desaturate: bool,
}

#[derive(Component)]
pub struct SkillIconArt;

#[derive(Component, Clone, Copy)]
pub struct BuildcraftSlotArt {
    pub hero: PartyHeroKind,
    pub index: usize,
}

/// Soft gold wash when a buildcraft slot is focused.
#[derive(Component, Clone, Copy)]
pub struct SkillIconFocusGlow {
    pub hero: PartyHeroKind,
    pub index: usize,
}

/// Per-slot cooldown dim — hidden until playback provides per-skill timing.
#[derive(Component, Clone, Copy)]
pub struct SkillIconCooldownOverlay {
    pub hero: PartyHeroKind,
    pub index: usize,
}

/// Shared ability GCD sweep — hidden until playback sync enables it.
#[derive(Component, Clone, Copy)]
pub struct SkillIconGcdOverlay {
    pub hero: PartyHeroKind,
    pub index: usize,
}

#[derive(Component)]
pub struct SkillIconDimOverlay;

#[derive(Component)]
pub struct SkillIconSlotLabel;

#[derive(Clone, Copy, Debug)]
pub struct SkillIconConfig {
    pub skill: Option<SkillId>,
    pub size_px: f32,
    pub slot_index: Option<usize>,
    pub bar_hero: Option<PartyHeroKind>,
    pub focused: bool,
    pub locked: bool,
    pub empty: bool,
}

impl SkillIconConfig {
    pub fn filled(skill: SkillId, size_px: f32) -> Self {
        Self {
            skill: Some(skill),
            size_px,
            slot_index: None,
            bar_hero: None,
            focused: false,
            locked: false,
            empty: false,
        }
    }

    pub fn bar_slot(
        skill: Option<SkillId>,
        hero: PartyHeroKind,
        index: usize,
        size_px: f32,
        focused: bool,
        locked: bool,
    ) -> Self {
        Self {
            skill,
            size_px,
            slot_index: Some(index + 1),
            bar_hero: Some(hero),
            focused,
            locked,
            empty: skill.is_none() && !locked,
        }
    }
}

pub fn spawn_skill_icon(
    parent: &mut ChildSpawnerCommands<'_>,
    config: SkillIconConfig,
    ph: &UiPlaceholderImages,
) -> Entity {
    let border = frame_border_for_skill(config.skill, config.focused);
    let image = if config.locked {
        locked_icon(ph)
    } else if let Some(id) = config.skill {
        icon_for(id, ph)
    } else {
        empty_icon(ph)
    };
    let tint = if config.locked {
        Color::srgba(0.35, 0.35, 0.38, 0.85)
    } else if config.empty {
        Color::srgba(0.55, 0.52, 0.48, 0.55)
    } else {
        Color::WHITE
    };
    let slot_ix = config
        .slot_index
        .map(|n| n.saturating_sub(1))
        .unwrap_or(0);
    let overlay_slot = config.bar_hero.map(|hero| (hero, slot_ix));

    parent
        .spawn((
            Node {
                box_sizing: BoxSizing::BorderBox,
                width: Val::Px(config.size_px),
                height: Val::Px(config.size_px),
                flex_shrink: 0.0,
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                border: UiRect::all(Val::Px(if config.focused { 2.0 } else { 1.0 })),
                ..default()
            },
            BackgroundColor(UiTheme::panel_bg_deep()),
            BorderColor::from(border),
            FocusPolicy::Pass,
        ))
        .with_children(|root| {
            root.spawn((
                Node {
                    box_sizing: BoxSizing::BorderBox,
                    width: Val::Percent(88.0),
                    height: Val::Percent(88.0),
                    position_type: PositionType::Relative,
                    ..default()
                },
                FocusPolicy::Pass,
            ))
            .with_children(|stack| {
                let mut art = stack.spawn((
                    Node {
                        box_sizing: BoxSizing::BorderBox,
                        width: Val::Percent(100.0),
                        height: Val::Percent(100.0),
                        ..default()
                    },
                    ImageNode {
                        image,
                        color: tint,
                        ..default()
                    },
                    SkillIconArt,
                ));
                if let (Some(hero), Some(slot_n)) = (config.bar_hero, config.slot_index) {
                    art.insert(BuildcraftSlotArt {
                        hero,
                        index: slot_n.saturating_sub(1),
                    });
                }
                if let Some((hero, index)) = overlay_slot {
                    let glow_color = UiTheme::torch_glow()
                        .mix(&UiTheme::muted_gold(), 0.45)
                        .with_alpha(0.38);
                    stack.spawn((
                        Node {
                            box_sizing: BoxSizing::BorderBox,
                            position_type: PositionType::Absolute,
                            width: Val::Percent(100.0),
                            height: Val::Percent(100.0),
                            ..default()
                        },
                        BackgroundColor(glow_color),
                        if config.focused {
                            Visibility::Visible
                        } else {
                            Visibility::Hidden
                        },
                        SkillIconFocusGlow { hero, index },
                    ));
                    stack
                        .spawn((
                            Node {
                                box_sizing: BoxSizing::BorderBox,
                                position_type: PositionType::Absolute,
                                left: Val::Px(0.0),
                                right: Val::Px(0.0),
                                bottom: Val::Px(0.0),
                                height: Val::Percent(0.0),
                                ..default()
                            },
                            BackgroundColor(Color::srgba(0.05, 0.05, 0.08, 0.72)),
                            Visibility::Hidden,
                        ))
                        .insert(SkillIconCooldownOverlay { hero, index });
                    stack
                        .spawn((
                            Node {
                                box_sizing: BoxSizing::BorderBox,
                                position_type: PositionType::Absolute,
                                left: Val::Px(0.0),
                                right: Val::Px(0.0),
                                top: Val::Px(0.0),
                                height: Val::Percent(0.0),
                                ..default()
                            },
                            BackgroundColor(Color::srgba(0.52, 0.38, 0.62, 0.55)),
                            Visibility::Hidden,
                        ))
                        .insert(SkillIconGcdOverlay { hero, index });
                }
                stack
                    .spawn((
                        Node {
                            box_sizing: BoxSizing::BorderBox,
                            position_type: PositionType::Absolute,
                            width: Val::Percent(100.0),
                            height: Val::Percent(100.0),
                            ..default()
                        },
                        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.55)),
                        Visibility::Hidden,
                    ))
                    .insert(SkillIconDimOverlay);
            });
            if let Some(n) = config.slot_index {
                root.spawn((
                    Text::new(n.to_string()),
                    TextFont::from_font_size(UiTheme::FONT_MICRO),
                    TextColor(UiTheme::body_dim()),
                    SkillIconSlotLabel,
                ));
            }
        })
        .id()
}
