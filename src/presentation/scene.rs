//! Title camp presentation scene layout JSON (`assets/tuning/title_scene.json`).

use crate::presentation::anchor::SceneAnchorPose;
use crate::presentation::element::PresentationElementTune;
use crate::presentation::fire::TitleFirePresentationTune;
use bevy::log::{info, warn};
use bevy::prelude::Resource;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

const SCENE_PATH: &str = "assets/tuning/title_scene.json";
const LEGACY_CAMPFIRE_PATH: &str = "assets/tuning/title_campfire.json";

/// Fake environmental lighting magnitudes (presentation-only; not a real light engine).
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct TitleAmbientPresentationTune {
    /// 0..1 — future: warm bounce on crates, tents, hero silhouettes near the fire.
    pub near_fire_prop_tint: f32,
}

fn default_title_anchors() -> HashMap<String, SceneAnchorPose> {
    [
        "fire_center",
        "lead_slot",
        "ally_slot",
        "loot_spawn",
        "camera_focus",
        "ambient_light_origin",
    ]
    .into_iter()
    .map(|k| (k.to_string(), SceneAnchorPose::default()))
    .collect()
}

/// Serialized title camp UI layout (legacy JSON shape preserved).
#[derive(Resource, Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct TitleCampSceneLayout {
    pub fireplace: PresentationElementTune,
    pub lead_slot: PresentationElementTune,
    pub ally_slot: PresentationElementTune,
    #[serde(default = "default_title_anchors")]
    pub anchors: HashMap<String, SceneAnchorPose>,
    #[serde(default)]
    pub fire_presentation: TitleFirePresentationTune,
    #[serde(default)]
    pub ambient: TitleAmbientPresentationTune,
}

impl Default for TitleCampSceneLayout {
    fn default() -> Self {
        Self {
            fireplace: PresentationElementTune::default_fireplace(),
            lead_slot: PresentationElementTune::default_figure_lead(),
            ally_slot: PresentationElementTune::default_figure_ally(),
            anchors: default_title_anchors(),
            fire_presentation: TitleFirePresentationTune::default(),
            ambient: TitleAmbientPresentationTune::default(),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default, Serialize, Deserialize)]
pub enum TitleCampSceneTuneTarget {
    #[default]
    Fireplace,
    LeadSlot,
    AllySlot,
}

impl TitleCampSceneTuneTarget {
    pub fn next(self) -> Self {
        match self {
            Self::Fireplace => Self::LeadSlot,
            Self::LeadSlot => Self::AllySlot,
            Self::AllySlot => Self::Fireplace,
        }
    }

    pub fn prev(self) -> Self {
        match self {
            Self::Fireplace => Self::AllySlot,
            Self::LeadSlot => Self::Fireplace,
            Self::AllySlot => Self::LeadSlot,
        }
    }
}

impl TitleCampSceneLayout {
    pub fn try_load_from_disk() -> Self {
        let scene_p = PathBuf::from(SCENE_PATH);
        if let Ok(text) = std::fs::read_to_string(&scene_p) {
            match serde_json::from_str::<Self>(&text) {
                Ok(s) => return s,
                Err(e) => warn!(
                    "Ignoring malformed {} ({}). Trying legacy file.",
                    scene_p.display(),
                    e
                ),
            }
        }
        try_load_legacy_campfire()
    }

    pub fn try_save_to_disk(&self) {
        let path = PathBuf::from(SCENE_PATH);
        if let Some(parent) = path.parent() {
            if let Err(e) = std::fs::create_dir_all(parent) {
                warn!("Could not create {}: {}", parent.display(), e);
                return;
            }
        }
        match serde_json::to_string_pretty(self) {
            Ok(text) => {
                if let Err(e) = std::fs::write(&path, text) {
                    warn!("Could not write {}: {}", path.display(), e);
                } else {
                    info!("Wrote {}", path.display());
                }
            }
            Err(e) => warn!("Could not serialize title scene layout: {}", e),
        }
    }

    pub fn tune_mut(&mut self, target: TitleCampSceneTuneTarget) -> &mut PresentationElementTune {
        match target {
            TitleCampSceneTuneTarget::Fireplace => &mut self.fireplace,
            TitleCampSceneTuneTarget::LeadSlot => &mut self.lead_slot,
            TitleCampSceneTuneTarget::AllySlot => &mut self.ally_slot,
        }
    }

    pub fn tune(&self, target: TitleCampSceneTuneTarget) -> &PresentationElementTune {
        match target {
            TitleCampSceneTuneTarget::Fireplace => &self.fireplace,
            TitleCampSceneTuneTarget::LeadSlot => &self.lead_slot,
            TitleCampSceneTuneTarget::AllySlot => &self.ally_slot,
        }
    }
}

#[derive(Deserialize)]
struct LegacyCampfireFile {
    offset_x: f32,
    offset_y: f32,
    scale_x: f32,
    scale_y: f32,
    display_h: f32,
}

fn try_load_legacy_campfire() -> TitleCampSceneLayout {
    let path = PathBuf::from(LEGACY_CAMPFIRE_PATH);
    let Ok(text) = std::fs::read_to_string(&path) else {
        return TitleCampSceneLayout::default();
    };
    match serde_json::from_str::<LegacyCampfireFile>(&text) {
        Ok(o) => TitleCampSceneLayout {
            fireplace: PresentationElementTune {
                offset_x: o.offset_x,
                offset_y: o.offset_y,
                scale_x: o.scale_x,
                scale_y: o.scale_y,
                size_basis: o.display_h,
                ..PresentationElementTune::default_fireplace()
            },
            ..Default::default()
        },
        Err(e) => {
            warn!(
                "Ignoring malformed legacy {} ({}). Using defaults.",
                path.display(),
                e
            );
            TitleCampSceneLayout::default()
        }
    }
}
