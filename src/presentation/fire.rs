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
use crate::presentation::layer::{compose_layer_id, TITLE_ELEMENT_FIREPLACE};
use crate::presentation::markers::PresentationLayerHost;

/// Deterministic seed for [`PresentationTrack::evaluate`] on title fire ambient layers.
pub const TITLE_FIRE_TRACK_EVAL_SEED: u64 = 0xF1EE_CAFE_DA7A_u64;

/// Width multiplier (`base_w × this`) for the soft ground ellipse under the fire.
pub const TITLE_FIRE_GROUND_LIGHT_W_MULT: f32 = 1.65;
/// Layout height (px) for the ground wash; kept low versus width for an elliptical pool of light.
pub const TITLE_FIRE_GROUND_LIGHT_H_PX: f32 = 26.0;

/// Outer halo extends further than the core glow (fraction of `base_w` / `base_h`).
pub const TITLE_FIRE_GLOW_HALO_INSET_X: f32 = 0.58;
pub const TITLE_FIRE_GLOW_HALO_INSET_Y_TOP: f32 = 0.44;
pub const TITLE_FIRE_GLOW_HALO_INSET_Y_BOTTOM: f32 = 0.62;
/// Inner core glow sits tighter on the flame art.
pub const TITLE_FIRE_GLOW_CORE_INSET_X: f32 = 0.34;
pub const TITLE_FIRE_GLOW_CORE_INSET_Y_TOP: f32 = 0.26;
pub const TITLE_FIRE_GLOW_CORE_INSET_Y_BOTTOM: f32 = 0.44;

/// Subtle vertical breathe on flame layers (scale Y multiplier peak).
pub const TITLE_FIRE_FLAME_BREATHE_AMP: f32 = 0.035;
pub const TITLE_FIRE_FLAME_BREATHE_HZ: f32 = 0.19;

/// Root node that sizes to the fireplace slot (children handle layering).
#[derive(Component)]
pub struct PresentationFireStackRoot;

#[derive(Component, Clone, Copy)]
pub enum PresentationFirePart {
    GroundLight,
    BaseStatic,
    Flame(u8),
    /// Wide, low-alpha radial wash (same texture as core glow).
    GlowHalo,
    Glow,
    /// Small warm sparks above the logs (presentation-only).
    Ember(u8),
}

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
    #[must_use]
    pub fn get_by_key(&self, layer_key: &str) -> Option<&PresentationLayerTune> {
        match layer_key {
            "stack" => Some(&self.stack),
            "base" => Some(&self.base),
            "flame" => Some(&self.flame),
            "glow" => Some(&self.glow),
            "ground" => Some(&self.ground),
            _ => None,
        }
    }

    pub fn get_mut_by_key(&mut self, layer_key: &str) -> Option<&mut PresentationLayerTune> {
        match layer_key {
            "stack" => Some(&mut self.stack),
            "base" => Some(&mut self.base),
            "flame" => Some(&mut self.flame),
            "glow" => Some(&mut self.glow),
            "ground" => Some(&mut self.ground),
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
            crossfade_period_secs: 6.2_f32,
            glow_pulse_hz: 0.21_f32,
            glow_pulse_scale: 0.035_f32,
            glow_max_alpha: 0.24_f32,
            glow_min_alpha: 0.1_f32,
            ground_flicker_hz: 0.26_f32,
            ground_max_alpha: 0.26_f32,
            glow_alpha: Some(default_glow_alpha_track()),
            ground_alpha: Some(default_ground_alpha_track()),
            layers: FirePresentationLayerTunes::default(),
        }
    }
}

fn default_glow_alpha_track() -> PresentationTrack {
    PresentationTrack {
        base_value: 0.17,
        layers: vec![
            CurveLayer {
                kind: CurveKind::Sine,
                frequency_hz: 0.21,
                amplitude: 0.032,
                phase: 0.0,
                weight: 1.0,
                blend: CurveBlendMode::Multiplicative,
            },
            CurveLayer {
                kind: CurveKind::SmoothNoise,
                frequency_hz: 0.14,
                amplitude: 0.018,
                phase: 1.1,
                weight: 0.85,
                blend: CurveBlendMode::Additive,
            },
        ],
    }
}

fn default_ground_alpha_track() -> PresentationTrack {
    PresentationTrack {
        base_value: 0.13,
        layers: vec![
            CurveLayer {
                kind: CurveKind::Sine,
                frequency_hz: 0.24,
                amplitude: 0.055,
                phase: 2.2,
                weight: 1.0,
                blend: CurveBlendMode::Additive,
            },
            CurveLayer {
                kind: CurveKind::SmoothNoise,
                frequency_hz: 0.08,
                amplitude: 0.022,
                phase: 0.5,
                weight: 0.8,
                blend: CurveBlendMode::Additive,
            },
        ],
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

    /// Ensures a compositional glow track exists (for editor / inspector).
    pub fn glow_alpha_track_mut(&mut self) -> &mut PresentationTrack {
        if self.glow_alpha.is_none() {
            self.glow_alpha = Some(self.synthesize_glow_track_from_legacy());
        }
        self.glow_alpha.as_mut().expect("glow_alpha just initialized")
    }

    /// Ensures a compositional ground track exists (for editor / inspector).
    pub fn ground_alpha_track_mut(&mut self) -> &mut PresentationTrack {
        if self.ground_alpha.is_none() {
            self.ground_alpha = Some(self.synthesize_ground_track_from_legacy());
        }
        self.ground_alpha.as_mut().expect("ground_alpha just initialized")
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
                margin: UiRect::top(Val::Px(2.0)),
                border_radius: BorderRadius::percent(72.0, 72.0, 64.0, 64.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.92, 0.42, 0.16, cfg.ground_max_alpha * 0.28).into()),
            ZIndex(-4),
            PresentationFirePart::GroundLight,
            PresentationLayerHost(compose_layer_id(TITLE_ELEMENT_FIREPLACE, "ground")),
            Interaction::default(),
        ));
    }

    col.spawn((
        Node {
            box_sizing: BoxSizing::BorderBox,
            position_type: PositionType::Relative,
            width: Val::Px(base_w),
            height: Val::Px(base_h),
            overflow: Overflow::visible(),
            ..default()
        },
        PresentationFireStackRoot,
        PresentationLayerHost(compose_layer_id(TITLE_ELEMENT_FIREPLACE, "stack")),
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
            PresentationLayerHost(compose_layer_id(TITLE_ELEMENT_FIREPLACE, "base")),
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
                        ..default()
                    },
                    ImageNode {
                        image: fireplace_image.clone(),
                        color: warm.with_alpha(0.0),
                        image_mode: NodeImageMode::Auto,
                        ..default()
                    },
                    UiTransform::default(),
                    ZIndex(1 + i as i32),
                    PresentationFirePart::Flame(i as u8),
                    PresentationLayerHost(compose_layer_id(TITLE_ELEMENT_FIREPLACE, "flame")),
                    Interaction::default(),
                ));
            }
        }

        if cfg.enabled && cfg.glow_max_alpha > 0.001 {
            let floor = cfg.glow_min_alpha.max(0.0);
            let a_core = (cfg.glow_max_alpha * 0.88).clamp(floor, 1.0);
            let a_halo = (a_core * 0.42).clamp(floor * 0.65, 0.55);

            stack.spawn((
                Node {
                    box_sizing: BoxSizing::BorderBox,
                    position_type: PositionType::Absolute,
                    left: Val::Px(-base_w * TITLE_FIRE_GLOW_HALO_INSET_X),
                    top: Val::Px(-base_h * TITLE_FIRE_GLOW_HALO_INSET_Y_TOP),
                    right: Val::Px(-base_w * TITLE_FIRE_GLOW_HALO_INSET_X),
                    bottom: Val::Px(-base_h * TITLE_FIRE_GLOW_HALO_INSET_Y_BOTTOM),
                    ..default()
                },
                ImageNode {
                    image: glow_radial_image.clone(),
                    color: Color::srgba(1.0, 0.5, 0.14, a_halo),
                    image_mode: NodeImageMode::Auto,
                    ..default()
                },
                UiTransform::default(),
                ZIndex(22),
                PresentationFirePart::GlowHalo,
                Interaction::default(),
            ));

            stack.spawn((
                Node {
                    box_sizing: BoxSizing::BorderBox,
                    position_type: PositionType::Absolute,
                    left: Val::Px(-base_w * TITLE_FIRE_GLOW_CORE_INSET_X),
                    top: Val::Px(-base_h * TITLE_FIRE_GLOW_CORE_INSET_Y_TOP),
                    right: Val::Px(-base_w * TITLE_FIRE_GLOW_CORE_INSET_X),
                    bottom: Val::Px(-base_h * TITLE_FIRE_GLOW_CORE_INSET_Y_BOTTOM),
                    ..default()
                },
                ImageNode {
                    image: glow_radial_image.clone(),
                    color: Color::srgba(1.0, 0.58, 0.2, a_core),
                    image_mode: NodeImageMode::Auto,
                    ..default()
                },
                UiTransform::IDENTITY,
                ZIndex(24),
                PresentationFirePart::Glow,
                PresentationLayerHost(compose_layer_id(TITLE_ELEMENT_FIREPLACE, "glow")),
                Interaction::default(),
            ));
        }

        if cfg.enabled {
            const EMBER_COUNT: u8 = 3;
            for i in 0..EMBER_COUNT {
                let x = base_w * (0.28 + 0.18 * i as f32);
                let y = base_h * (0.08 + 0.04 * (i % 2) as f32);
                stack.spawn((
                    Node {
                        box_sizing: BoxSizing::BorderBox,
                        position_type: PositionType::Absolute,
                        left: Val::Px(x),
                        top: Val::Px(-y),
                        width: Val::Px(5.0 + (i as f32) * 1.5),
                        height: Val::Px(5.0 + (i as f32) * 1.5),
                        border_radius: BorderRadius::MAX,
                        ..default()
                    },
                    BackgroundColor(Color::srgba(1.0, 0.72, 0.32, 0.0).into()),
                    ZIndex(18 + i as i32),
                    PresentationFirePart::Ember(i),
                    Interaction::default(),
                ));
            }
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn synthesized_glow_track_matches_scalar_fallback() {
        let mut cfg = TitleFirePresentationTune::default();
        cfg.glow_alpha = None;
        let t = 0.777_f32;
        let seed = 12_u64;
        let a = cfg.evaluate_glow_alpha_at(t, seed);
        let b = cfg.synthesize_glow_track_from_legacy().evaluate(t, seed);
        assert!((a - b).abs() < 1e-5, "a={} b={}", a, b);
    }

    #[test]
    fn default_glow_track_is_in_fire_breathing_band() {
        let cfg = TitleFirePresentationTune::default();
        let track = cfg.glow_alpha.as_ref().expect("default includes glow_alpha");
        assert!(track.base_value > 0.05 && track.base_value < 0.4);
        let hz = track.layers.first().map(|l| l.frequency_hz).unwrap_or(0.0);
        assert!(hz >= 0.15 && hz <= 0.35, "hz={hz}");
    }
}
