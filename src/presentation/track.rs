//! Compositional scalar curves for presentation modulation (`PresentationTrack`).
//!
//! Evaluation is deterministic from `(t_secs, seed)` — no RNG.

use serde::{Deserialize, Serialize};
use std::f32::consts::TAU;

/// Waveform / modulation primitive for one [`CurveLayer`].
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CurveKind {
    Sine,
    Triangle,
    Saw,
    SmoothNoise,
    RandomPulse,
    #[serde(alias = "ease")]
    EaseInOut,
}

/// How a layer combines with the running value (see [`PresentationTrack::evaluate`]).
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CurveBlendMode {
    #[default]
    Additive,
    Multiplicative,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct CurveLayer {
    pub kind: CurveKind,
    pub frequency_hz: f32,
    pub amplitude: f32,
    /// Phase offset in radians (matches sine-style layering).
    pub phase: f32,
    pub weight: f32,
    pub blend: CurveBlendMode,
}

impl Default for CurveLayer {
    fn default() -> Self {
        Self {
            kind: CurveKind::Sine,
            frequency_hz: 0.22,
            amplitude: 0.05,
            phase: 0.0,
            weight: 1.0,
            blend: CurveBlendMode::Additive,
        }
    }
}

/// Evaluates to a single `f32` at time `t_secs` using deterministic hashing (`seed`).
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct PresentationTrack {
    pub base_value: f32,
    pub layers: Vec<CurveLayer>,
}

impl PresentationTrack {
    /// Evaluation order (stable):
    ///
    /// 1. Start with `base_value`.
    /// 2. For each layer in order, sample `kind` → roughly **−1..1** (noise/pulses included).
    /// 3. Let `contrib = sample * amplitude * weight`.
    ///    - **Additive:** `value += contrib`
    ///    - **Multiplicative:** `value *= 1.0 + contrib`
    #[must_use]
    pub fn evaluate(&self, t_secs: f32, seed: u64) -> f32 {
        let mut value = self.base_value;
        for layer in &self.layers {
            let sample = layer
                .kind
                .sample(t_secs, layer.frequency_hz, layer.phase, seed);
            let contrib = sample * layer.amplitude * layer.weight;
            match layer.blend {
                CurveBlendMode::Additive => value += contrib,
                CurveBlendMode::Multiplicative => value *= 1.0 + contrib,
            }
        }
        if value.is_nan() {
            return self.base_value;
        }
        value
    }
}

impl CurveKind {
    fn sample(self, t_secs: f32, freq_hz: f32, phase_rad: f32, seed: u64) -> f32 {
        match self {
            CurveKind::Sine => (TAU * freq_hz * t_secs + phase_rad).sin(),
            CurveKind::Triangle => {
                let u = fract(t_secs * freq_hz + phase_rad / TAU);
                let tri = if u < 0.5 {
                    4.0 * u - 1.0
                } else {
                    3.0 - 4.0 * u
                };
                tri.clamp(-1.0, 1.0)
            }
            CurveKind::Saw => {
                let u = fract(t_secs * freq_hz + phase_rad / TAU);
                2.0 * u - 1.0
            }
            CurveKind::SmoothNoise => smooth_noise_sample(t_secs, freq_hz, phase_rad, seed),
            CurveKind::RandomPulse => random_pulse_sample(t_secs, freq_hz, phase_rad, seed),
            CurveKind::EaseInOut => ease_in_out_sample(t_secs, freq_hz, phase_rad),
        }
    }
}

#[inline]
fn fract(x: f32) -> f32 {
    x - x.floor()
}

#[inline]
fn smoothstep01(edge0: f32, edge1: f32, x: f32) -> f32 {
    let t = ((x - edge0) / (edge1 - edge0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

#[inline]
fn splitmix64(mut z: u64) -> u64 {
    z = z.wrapping_add(0x9E3779B97F4A7C15);
    let mut x = z;
    x ^= x >> 30;
    x = x.wrapping_mul(0xBF58476D1CE4E5B9);
    x ^= x >> 27;
    x = x.wrapping_mul(0x94D049BB133111EB);
    x ^ (x >> 31)
}

#[inline]
fn hash01(cell: i64, seed: u64) -> f32 {
    let h = splitmix64(seed.wrapping_add(cell as u64));
    // Upper bits → stable float in [0, 1)
    ((h >> 40) as f32) / ((1_u64 << 24) as f32)
}

fn smooth_noise_sample(t_secs: f32, freq_hz: f32, phase_rad: f32, seed: u64) -> f32 {
    let x = t_secs * freq_hz + phase_rad / TAU;
    let i = x.floor() as i64;
    let f = x - i as f32;
    let h0 = hash01(i, seed);
    let h1 = hash01(i + 1, seed);
    let v = h0 + (h1 - h0) * smoothstep01(0.0, 1.0, f);
    v * 2.0 - 1.0
}

fn random_pulse_sample(t_secs: f32, freq_hz: f32, phase_rad: f32, seed: u64) -> f32 {
    let cycles = t_secs * freq_hz + phase_rad / TAU;
    let cell = cycles.floor() as i64;
    let frac = fract(cycles);
    let h = splitmix64(seed ^ splitmix64(cell as u64));
    let duty = ((h >> 32) & 0xFFFF) as f32 / 65536.0;
    if frac < duty.clamp(0.05, 0.95) {
        1.0
    } else {
        -1.0
    }
}

fn ease_in_out_sample(t_secs: f32, freq_hz: f32, phase_rad: f32) -> f32 {
    let u = fract(t_secs * freq_hz + phase_rad / TAU);
    let s = if u < 0.5 {
        smoothstep01(0.0, 1.0, u * 2.0)
    } else {
        smoothstep01(1.0, 0.0, (u - 0.5) * 2.0)
    };
    s * 2.0 - 1.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn track_additive_layers_are_deterministic() {
        let track = PresentationTrack {
            base_value: 0.2,
            layers: vec![
                CurveLayer {
                    kind: CurveKind::Sine,
                    frequency_hz: 0.22,
                    amplitude: 0.04,
                    phase: 0.0,
                    weight: 1.0,
                    blend: CurveBlendMode::Additive,
                },
                CurveLayer {
                    kind: CurveKind::SmoothNoise,
                    frequency_hz: 0.17,
                    amplitude: 0.03,
                    phase: 1.37,
                    weight: 0.6,
                    blend: CurveBlendMode::Additive,
                },
            ],
        };
        let a = track.evaluate(1.25, 42);
        let b = track.evaluate(1.25, 42);
        assert_eq!(a, b);
    }

    #[test]
    fn multiplicative_blend_formula_stable() {
        let track = PresentationTrack {
            base_value: 0.22,
            layers: vec![CurveLayer {
                kind: CurveKind::Sine,
                frequency_hz: 1.0,
                amplitude: 0.5,
                phase: std::f32::consts::FRAC_PI_2,
                weight: 1.0,
                blend: CurveBlendMode::Multiplicative,
            }],
        };
        // sin(pi/2 + 2pi*t)|_{t=0} = 1 → value = 0.22 * (1 + 0.5)
        assert!((track.evaluate(0.0, 0) - 0.33).abs() < 1e-5);
    }

    #[test]
    fn legacy_scalar_maps_to_default_track() {
        let hz = 0.22_f32;
        assert!(
            hz >= 0.15 && hz <= 0.35,
            "glow_pulse_hz default should sit in fire/light band"
        );
        let track = PresentationTrack {
            base_value: 0.22_f32 * 0.9,
            layers: vec![CurveLayer {
                kind: CurveKind::Sine,
                frequency_hz: hz,
                amplitude: 0.04,
                phase: 0.0,
                weight: 1.0,
                blend: CurveBlendMode::Multiplicative,
            }],
        };
        let t = 1.37_f32;
        let pulse = (1.0_f32 + 0.04 * (TAU * t * hz).sin()).max(0.0);
        let legacy_alpha = 0.22_f32 * 0.9 * pulse;
        let evaluated = track.evaluate(t, 99);
        assert!(
            (evaluated - legacy_alpha).abs() < 1e-5,
            "legacy glow ≈ synthesized multiplicative sine track"
        );
    }
}
