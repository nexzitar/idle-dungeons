//! Tunable presentation element layout (offsets, pivot, anchor binding).

use crate::presentation::pivot::ScenePivot;
use serde::{Deserialize, Serialize};

pub type PresentationElementId = String;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct PresentationElementTune {
    pub offset_x: f32,
    pub offset_y: f32,
    pub scale_x: f32,
    pub scale_y: f32,
    /// Base size: fireplace image height (px), or figure emoji font size (px).
    pub size_basis: f32,
    /// Clockwise, degrees (`UiTransform.rotation`).
    pub rotation_deg: f32,
    /// Multiplier on RGB (approximates exposure), >= 0.
    pub exposure: f32,
    /// 0 = off; adds a warm push on top of exposure (slots: emoji tint).
    pub glow: f32,
    /// Reserved for bloom / blur strength when a post-process path exists.
    pub bloom: f32,
    /// Draw order within UI (`GlobalZIndex`).
    pub global_z: i32,
    #[serde(default)]
    pub pivot: ScenePivot,
    /// When set, `offset_*` are applied on top of [`crate::presentation::anchor::SceneAnchorPose`] for this name.
    #[serde(default)]
    pub anchor_ref: Option<String>,
}

impl Default for PresentationElementTune {
    fn default() -> Self {
        Self {
            offset_x: 0.0,
            offset_y: 0.0,
            scale_x: 1.0,
            scale_y: 1.0,
            size_basis: 210.0,
            rotation_deg: 0.0,
            exposure: 1.0,
            glow: 0.0,
            bloom: 0.0,
            global_z: 0,
            pivot: ScenePivot::Center,
            anchor_ref: None,
        }
    }
}

impl PresentationElementTune {
    #[must_use]
    pub fn with_size_basis(mut self, v: f32) -> Self {
        self.size_basis = v;
        self
    }

    #[must_use]
    pub fn default_fireplace() -> Self {
        Self {
            size_basis: 210.0,
            pivot: ScenePivot::BottomCenter,
            anchor_ref: Some("fire_center".into()),
            ..Default::default()
        }
    }

    #[must_use]
    pub fn default_figure_lead() -> Self {
        Self {
            size_basis: 42.0,
            pivot: ScenePivot::BottomCenter,
            anchor_ref: Some("lead_slot".into()),
            ..Default::default()
        }
    }

    #[must_use]
    pub fn default_figure_ally() -> Self {
        Self {
            size_basis: 42.0,
            pivot: ScenePivot::BottomCenter,
            anchor_ref: Some("ally_slot".into()),
            ..Default::default()
        }
    }

    /// Snap placement back onto the bound anchor (offsets zero, neutral scale/rotation).
    pub fn reset_placement_to_anchor(&mut self) {
        self.offset_x = 0.0;
        self.offset_y = 0.0;
        self.rotation_deg = 0.0;
        self.scale_x = 1.0;
        self.scale_y = 1.0;
    }
}
