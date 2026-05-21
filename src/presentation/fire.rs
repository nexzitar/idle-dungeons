//! Layered title campfire illusion: static base, cross-faded warm variants, glow pulse, ground wash.
//! Intentionally cheap (tinted UI layers, no real lighting).
//!
//! The **host** node (transform, global Z, tuning marker) is spawned by `ui::title_camp`; this module
//! only adds layered children under that host.

use bevy::prelude::*;
use bevy::ui::{BorderRadius, UiTransform, ZIndex};
use serde::{Deserialize, Serialize};

/// Width multiplier (`base_w × this`) for the soft ground ellipse under the fire.
pub const TITLE_FIRE_GROUND_LIGHT_W_MULT: f32 = 1.5;
/// Layout height (px) for the ground wash; kept low versus width for an elliptical pool of light.
pub const TITLE_FIRE_GROUND_LIGHT_H_PX: f32 = 20.0;

/// Root node that sizes to the fireplace slot (children handle layering).
#[derive(Component)]
pub struct PresentationFireStackRoot;

#[derive(Component, Clone, Copy)]
pub enum PresentationFirePart {
    GroundLight,
    BaseStatic,
    Flame(u8),
    Glow,
}

/// Serialized with `TitleCampSceneLayout`; drives fire tick in `scene_tune`.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct TitleFirePresentationTune {
    pub enabled: bool,
    /// Number of tinted flame layers (same art; cross-faded).
    pub flame_variants: u8,
    pub crossfade_period_secs: f32,
    pub glow_pulse_hz: f32,
    pub glow_pulse_scale: f32,
    pub glow_max_alpha: f32,
    /// Floor so breathing never fully dims the radial glow tint.
    pub glow_min_alpha: f32,
    pub ground_flicker_hz: f32,
    pub ground_max_alpha: f32,
}

impl Default for TitleFirePresentationTune {
    fn default() -> Self {
        Self {
            enabled: true,
            flame_variants: 4,
            crossfade_period_secs: 5.5_f32,
            glow_pulse_hz: 0.22_f32,
            glow_pulse_scale: 0.04_f32,
            glow_max_alpha: 0.22_f32,
            glow_min_alpha: 0.08_f32,
            ground_flicker_hz: 0.28_f32,
            ground_max_alpha: 0.28_f32,
        }
    }
}

/// Layers only: call from `with_children` of the fireplace **host** (see `title_camp`).
pub fn spawn_title_fire_layers(
    col: &mut ChildSpawnerCommands<'_>,
    fireplace_image: Handle<Image>,
    glow_radial_image: Handle<Image>,
    base_w: f32,
    base_h: f32,
    cfg: &TitleFirePresentationTune,
) {
    let base_w = base_w.max(24.0);
    let base_h = base_h.max(24.0);

    if cfg.enabled && cfg.ground_max_alpha > 0.001 {
        col.spawn((
            Node {
                box_sizing: BoxSizing::BorderBox,
                width: Val::Px(base_w * TITLE_FIRE_GROUND_LIGHT_W_MULT),
                height: Val::Px(TITLE_FIRE_GROUND_LIGHT_H_PX),
                margin: UiRect::top(Val::Px(4.0)),
                border_radius: BorderRadius::percent(62.0, 62.0, 54.0, 54.0),
                ..default()
            },
            BackgroundColor(
                Color::srgba(0.86, 0.36, 0.13, cfg.ground_max_alpha * 0.36).into(),
            ),
            ZIndex(-4),
            PresentationFirePart::GroundLight,
        ));
    }

    col.spawn((
        Node {
            box_sizing: BoxSizing::BorderBox,
            position_type: PositionType::Relative,
            width: Val::Px(base_w),
            height: Val::Px(base_h),
            overflow: Overflow::clip(),
            ..default()
        },
        PresentationFireStackRoot,
        ZIndex(0),
    ))
    .with_children(|stack| {
        stack.spawn((
            Node {
                box_sizing: BoxSizing::BorderBox,
                position_type: PositionType::Absolute,
                left: Val::Px(0.0),
                top: Val::Px(0.0),
                right: Val::Px(0.0),
                bottom: Val::Px(0.0),
                ..default()
            },
            ImageNode {
                image: fireplace_image.clone(),
                color: Color::WHITE,
                image_mode: NodeImageMode::Auto,
                ..default()
            },
            ZIndex(0),
            PresentationFirePart::BaseStatic,
        ));

        if cfg.enabled && cfg.flame_variants > 0 {
            let n = cfg.flame_variants.clamp(2, 8);
            for i in 0..n {
                let phase = (i as f32) / (n as f32);
                let warm = Color::srgb(1.0, 0.78 + 0.08 * phase, 0.42 + 0.2 * phase);
                stack.spawn((
                    Node {
                        box_sizing: BoxSizing::BorderBox,
                        position_type: PositionType::Absolute,
                        left: Val::Px(0.0),
                        top: Val::Px(0.0),
                        right: Val::Px(0.0),
                        bottom: Val::Px(0.0),
                        ..default()
                    },
                    ImageNode {
                        image: fireplace_image.clone(),
                        color: warm.with_alpha(0.0),
                        image_mode: NodeImageMode::Auto,
                        ..default()
                    },
                    ZIndex(1 + i as i32),
                    PresentationFirePart::Flame(i as u8),
                ));
            }
        }

        if cfg.enabled && cfg.glow_max_alpha > 0.001 {
            let floor = cfg.glow_min_alpha.max(0.0);
            let a0 = (cfg.glow_max_alpha * 0.9).clamp(floor, 1.0);
            stack.spawn((
                Node {
                    box_sizing: BoxSizing::BorderBox,
                    position_type: PositionType::Absolute,
                    left: Val::Px(-base_w * 0.42),
                    top: Val::Px(-base_h * 0.32),
                    right: Val::Px(-base_w * 0.42),
                    bottom: Val::Px(-base_h * 0.52),
                    ..default()
                },
                ImageNode {
                    image: glow_radial_image.clone(),
                    color: Color::srgba(1.0, 0.55, 0.18, a0),
                    image_mode: NodeImageMode::Auto,
                    ..default()
                },
                UiTransform::IDENTITY,
                ZIndex(24),
                PresentationFirePart::Glow,
            ));
        }
    });
}
