use crate::presentation::pivot::ScenePivot;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Named anchor pose in **the same pixel space** as element `offset_x` / `offset_y` (stage-relative).
#[derive(Clone, Debug, Serialize, Deserialize, Default, PartialEq)]
#[serde(default)]
pub struct SceneAnchorPose {
    pub offset_x: f32,
    pub offset_y: f32,
    #[serde(default)]
    pub rotation_deg: f32,
    #[serde(default)]
    pub pivot: ScenePivot,
}

/// Returns **anchor base + element offsets** before pivot compensation is applied.
#[must_use]
pub fn resolve_element_translation_px(
    anchors: &HashMap<String, SceneAnchorPose>,
    anchor_ref: Option<&str>,
    element_offset_x: f32,
    element_offset_y: f32,
) -> Vec2 {
    let (ax, ay) = anchor_ref
        .and_then(|k| anchors.get(k))
        .map(|a| (a.offset_x, a.offset_y))
        .unwrap_or((0.0, 0.0));
    Vec2::new(ax + element_offset_x, ay + element_offset_y)
}
