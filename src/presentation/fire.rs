//! Layered title campfire illusion: static base, cross-faded warm variants, glow pulse, ground wash.
//! Intentionally cheap (tinted UI layers, no real lighting).
//!
//! The **host** node (transform, global Z, tuning marker) is spawned by `ui::title_camp`; this module
//! only adds layered children under that host.

use bevy::prelude::*;
use bevy::ui::{BorderRadius, UiTransform, ZIndex};
use serde::{Deserialize, Serialize};

use crate::presentation::element::PresentationLayerTune;
use crate::presentation::track::{CurveBlendMode, CurveKind, CurveLayer, PresentationTrack};
use crate::presentation::markers::PresentationFireLayerHost;

/// Deterministic seed for [`PresentationTrack::evaluate`] on title fire ambient layers.
pub const TITLE_FIRE_TRACK_EVAL_SEED: u64 = 0xF1EE_CAFE_DA7A_u64;

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

/// Editor / JSON ids for individual fireplace presentation layers.
pub const FIRE_LAYER_STACK: &str = "fire:stack";
pub const FIRE_LAYER_BASE: &str = "fire:base";
pub const FIRE_LAYER_FLAME: &str = "fire:flame";
pub const FIRE_LAYER_GLOW: &str = "fire:glow";
pub const FIRE_LAYER_GROUND: &str = "fire:ground";

/// Per-layer placement under the fireplace host (offsets relative to parent stack).
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct FirePresentationLayerTunes {
    pub stack: PresentationLayerTune,
    pub base: PresentationLayerTune,
    pub flame: PresentationLayerTune,
    pub glow: PresentationLayerTune,
    pub ground: PresentationLayerTune,
}

impl FirePresentationLayerTunes {
    pub fn get(&self, id: &str) -> Option<&PresentationLayerTune> {
        match id {
            FIRE_LAYER_STACK => Some(&self.stack),
            FIRE_LAYER_BASE => Some(&self.base),
            FIRE_LAYER_FLAME => Some(&self.flame),
            FIRE_LAYER_GLOW => Some(&self.glow),
            FIRE_LAYER_GROUND => Some(&self.ground),
            _ => None,
        }
    }

    pub fn get_mut(&mut self, id: &str) -> Option<&mut PresentationLayerTune> {
        match id {
            FIRE_LAYER_STACK => Some(&mut self.stack),
            FIRE_LAYER_BASE => Some(&mut self.base),
            FIRE_LAYER_FLAME => Some(&mut self.flame),
            FIRE_LAYER_GLOW => Some(&mut self.glow),
            FIRE_LAYER_GROUND => Some(&mut self.ground),
            _ => None,
        }
    }

    pub fn reset_all(&mut self) {
        self.stack.reset_placement_to_anchor();
        self.base.reset_placement_to_anchor();
        self.flame.reset_placement_to_anchor();
        self.glow.reset_placement_to_anchor();
        self.ground.reset_placement_to_anchor();
    }
}

#[must_use]
pub fn is_fire_layer_id(id: &str) -> bool {
    id.starts_with("fire:")
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
    /// Optional compositional glow alpha (`ImageNode` radial). Absent → legacy scalar synthesis.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub glow_alpha: Option<PresentationTrack>,
    /// Optional compositional ground wash alpha. Absent → legacy scalar synthesis.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ground_alpha: Option<PresentationTrack>,
    #[serde(default)]
    pub layers: FirePresentationLayerTunes,
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
            glow_alpha: None,
            ground_alpha: None,
            layers: FirePresentationLayerTunes::default(),
        }
    }
}

/// Apply layer placement on a UI node (translation + scale; rotation unused for layers).
pub fn apply_layer_ui_transform(ui: &mut UiTransform, tune: &PresentationLayerTune) {
    ui.translation = bevy::ui::Val2::px(tune.offset_x, tune.offset_y);
    ui.scale = Vec2::new(tune.scale_x.clamp(0.05, 4.0), tune.scale_y.clamp(0.05, 4.0));
}

impl TitleFirePresentationTune {
    /// Legacy glow scalar mapping → compositional track (matches absent `glow_alpha` runtime path).
    #[must_use]
    pub fn synthesize_glow_track_from_legacy(&self) -> PresentationTrack {
        PresentationTrack {
            base_value: self.glow_max_alpha * 0.9,
            layers: vec![CurveLayer {
                kind: CurveKind::Sine,
                frequency_hz: self.glow_pulse_hz,
                amplitude: self.glow_pulse_scale,
                phase: 0.0,
                weight: 1.0,
                blend: CurveBlendMode::Multiplicative,
            }],
        }
    }

    /// Legacy ground scalar approximation → compositional track (single sine vs nested legacy sine).
    #[must_use]
    pub fn synthesize_ground_track_from_legacy(&self) -> PresentationTrack {
        PresentationTrack {
            base_value: self.ground_max_alpha * 0.5,
            layers: vec![CurveLayer {
                kind: CurveKind::Sine,
                frequency_hz: self.ground_flicker_hz,
                amplitude: self.ground_max_alpha * 0.11,
                phase: 0.0,
                weight: 1.0,
                blend: CurveBlendMode::Additive,
            }],
        }
    }

    /// Glow opacity before clamp to `[glow_min_alpha, 1]`.
    #[must_use]
    pub fn evaluate_glow_alpha_at(&self, t_secs: f32, seed: u64) -> f32 {
        match &self.glow_alpha {
            Some(track) => track.evaluate(t_secs, seed),
            None => {
                let pulse = (1.0_f32
                    + self.glow_pulse_scale
                        * (std::f32::consts::TAU * t_secs * self.glow_pulse_hz).sin())
                .max(0.0);
                self.glow_max_alpha * 0.9 * pulse
            }
        }
    }

    /// Ground wash RGBA alpha factor before clamp.
    #[must_use]
    pub fn evaluate_ground_alpha_at(&self, t_secs: f32, seed: u64) -> f32 {
        match &self.ground_alpha {
            Some(track) => track.evaluate(t_secs, seed),
            None => {
                let ground_phase = std::f32::consts::TAU * t_secs * self.ground_flicker_hz
                    + 0.3 * (std::f32::consts::TAU * t_secs * (self.ground_flicker_hz * 0.5)).sin();
                self.ground_max_alpha * (0.5 + 0.11 * ground_phase.sin())
            }
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
                border: UiRect::all(Val::Px(2.0)),
                border_radius: BorderRadius::percent(62.0, 62.0, 54.0, 54.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.86, 0.36, 0.13, cfg.ground_max_alpha * 0.36).into()),
            ZIndex(-4),
            PresentationFirePart::GroundLight,
            PresentationFireLayerHost(FIRE_LAYER_GROUND.into()),
            Interaction::default(),
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
        PresentationFireLayerHost(FIRE_LAYER_STACK.into()),
        Interaction::default(),
        UiTransform::default(),
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
                border: UiRect::all(Val::Px(2.0)),
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
            PresentationFireLayerHost(FIRE_LAYER_BASE.into()),
            Interaction::default(),
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
                        border: UiRect::all(Val::Px(2.0)),
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
                    PresentationFireLayerHost(FIRE_LAYER_FLAME.into()),
                    Interaction::default(),
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
                    border: UiRect::all(Val::Px(2.0)),
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
                PresentationFireLayerHost(FIRE_LAYER_GLOW.into()),
                Interaction::default(),
            ));
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn synthesized_glow_track_matches_scalar_fallback() {
        let cfg = TitleFirePresentationTune::default();
        let t = 0.777_f32;
        let seed = 12_u64;
        let a = cfg.evaluate_glow_alpha_at(t, seed);
        let b = cfg.synthesize_glow_track_from_legacy().evaluate(t, seed);
        assert!((a - b).abs() < 1e-5, "a={} b={}", a, b);
    }
}
