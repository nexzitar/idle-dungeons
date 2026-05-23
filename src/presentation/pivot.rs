use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// Normalized pivot in **[0, 1] × [0, 1]** (top-left = (0,0), bottom-right = (1,1)).
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq)]
pub struct NormalizedPivot(pub Vec2);

impl NormalizedPivot {
    pub const CENTER: Self = Self(Vec2::new(0.5, 0.5));
}

/// Layout pivot for **`UiTransform` scale/rotation** compensation so ground contact and anchors stay stable.
#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ScenePivot {
    #[default]
    Center,
    TopCenter,
    BottomCenter,
    BottomLeft,
    BottomRight,
    Custom(NormalizedPivot),
}

impl ScenePivot {
    /// Normalized pivot **(x, y)** in top-left origin space.
    #[must_use]
    pub fn normalized(self) -> Vec2 {
        match self {
            ScenePivot::Center => Vec2::new(0.5, 0.5),
            ScenePivot::TopCenter => Vec2::new(0.5, 0.0),
            ScenePivot::BottomCenter => Vec2::new(0.5, 1.0),
            ScenePivot::BottomLeft => Vec2::new(0.0, 1.0),
            ScenePivot::BottomRight => Vec2::new(1.0, 1.0),
            ScenePivot::Custom(NormalizedPivot(v)) => v.clamp(Vec2::ZERO, Vec2::splat(1.0)),
        }
    }
}

/// Extra translation (px) so scaling about the **box center** approximates scaling about `pivot`.
/// `base_w`, `base_h` = unscaled layout size; `scale_x`, `scale_y` = `UiTransform.scale`.
#[must_use]
pub fn pivot_translation_compensation_px(
    pivot: ScenePivot,
    base_w: f32,
    base_h: f32,
    scale_x: f32,
    scale_y: f32,
) -> Vec2 {
    let p = pivot.normalized();
    let cx = 0.5;
    let cy = 0.5;
    let dx = (p.x - cx) * base_w * (1.0 - scale_x);
    let dy = (p.y - cy) * base_h * (1.0 - scale_y);
    Vec2::new(dx, dy)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn center_pivot_zero_compensation_at_unit_scale() {
        let v = pivot_translation_compensation_px(ScenePivot::Center, 100.0, 200.0, 1.0, 1.0);
        assert!((v - Vec2::ZERO).length() < 1e-4);
    }

    #[test]
    fn bottom_center_pushes_up_when_growing_taller() {
        let v = pivot_translation_compensation_px(ScenePivot::BottomCenter, 100.0, 200.0, 1.0, 2.0);
        assert!(v.y < 0.0);
    }
}
