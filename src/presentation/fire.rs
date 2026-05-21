//! Layered title campfire illusion: static base, cross-faded warm variants, glow pulse, ground wash.
//! Intentionally cheap (tinted UI layers, no real lighting).
//!
//! The **host** node (transform, global Z, tuning marker) is spawned by `ui::title_camp`; this module
//! only adds layered children under that host.

use bevy::prelude::*;
use bevy::ui::{BorderRadius, UiTransform, ZIndex};
use serde::{Deserialize, Serialize};

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
            ground_flicker_hz: 1.65_f32,
            ground_max_alpha: 0.28_f32,
        }
    }
}

/// Layers only: call from `with_children` of the fireplace **host** (see `title_camp`).
pub fn spawn_title_fire_layers(
    col: &mut ChildSpawnerCommands<'_>,
    fireplace_image: Handle<Image>,
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
                width: Val::Px(base_w * 1.35),
                height: Val::Px(28.0),
                margin: UiRect::top(Val::Px(4.0)),
                border_radius: BorderRadius::all(Val::Px(40.0)),
                ..default()
            },
            BackgroundColor(
                Color::srgba(0.85, 0.35, 0.12, cfg.ground_max_alpha * 0.55).into(),
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
            stack.spawn((
                Node {
                    box_sizing: BoxSizing::BorderBox,
                    position_type: PositionType::Absolute,
                    left: Val::Px(-base_w * 0.12),
                    top: Val::Px(-base_h * 0.08),
                    right: Val::Px(-base_w * 0.12),
                    bottom: Val::Px(-base_h * 0.18),
                    border_radius: BorderRadius::all(Val::Px(120.0)),
                    ..default()
                },
                BackgroundColor(
                    Color::srgba(1.0, 0.55, 0.18, cfg.glow_max_alpha * 0.9).into(),
                ),
                UiTransform::IDENTITY,
                ZIndex(24),
                PresentationFirePart::Glow,
            ));
        }
    });
}
