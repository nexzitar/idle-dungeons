//! Title camp presentation scene layout JSON (`assets/tuning/title_scene.json`).

use crate::domain::title_camp::CAMP_FIRE_SEATS;
use crate::presentation::anchor::SceneAnchorPose;
use crate::presentation::element::PresentationElementTune;
use crate::presentation::fire::TitleFirePresentationTune;
use crate::presentation::element::PresentationLayerTune;
use crate::presentation::layer::{
    normalize_layer_id, parse_layer_id, parse_player_element_seat,
    TitleCampExtraLayerTunes, TitleCampFigureLayerTunes, TITLE_ELEMENT_FIREPLACE,
};
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
    let mut anchors: HashMap<String, SceneAnchorPose> = [
        ("fire_center", SceneAnchorPose::default()),
        ("loot_spawn", SceneAnchorPose::default()),
        ("camera_focus", SceneAnchorPose::default()),
        ("ambient_light_origin", SceneAnchorPose::default()),
    ]
    .into_iter()
    .map(|(k, v)| (k.to_string(), v))
    .collect();
    for seat in 1..=CAMP_FIRE_SEATS {
        anchors.insert(
            format!("player{seat}"),
            SceneAnchorPose::default(),
        );
    }
    anchors
}

fn default_six_player_tunes() -> [PresentationElementTune; CAMP_FIRE_SEATS] {
    std::array::from_fn(|i| PresentationElementTune::default_figure_seat(i + 1))
}

/// Serialized title camp UI layout.
#[derive(Resource, Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct TitleCampSceneLayout {
    pub fireplace: PresentationElementTune,
    /// Physical seat tuning around the fire (`player1` … `player6` anchors).
    #[serde(default = "default_six_player_tunes")]
    pub players: [PresentationElementTune; CAMP_FIRE_SEATS],
    #[serde(default = "default_title_anchors")]
    pub anchors: HashMap<String, SceneAnchorPose>,
    #[serde(default)]
    pub fire_presentation: TitleFirePresentationTune,
    #[serde(default)]
    pub figure_layers: TitleCampFigureLayerTunes,
    #[serde(default)]
    pub extra_layers: TitleCampExtraLayerTunes,
    #[serde(default)]
    pub ambient: TitleAmbientPresentationTune,
}

/// On-disk JSON may still carry legacy `lead_slot` / `ally_slot` keys.
#[derive(Deserialize)]
struct TitleCampSceneLayoutFile {
    #[serde(default)]
    fireplace: PresentationElementTune,
    #[serde(default)]
    lead_slot: Option<PresentationElementTune>,
    #[serde(default)]
    ally_slot: Option<PresentationElementTune>,
    #[serde(default = "default_six_player_tunes")]
    players: [PresentationElementTune; CAMP_FIRE_SEATS],
    #[serde(default = "default_title_anchors")]
    anchors: HashMap<String, SceneAnchorPose>,
    #[serde(default)]
    fire_presentation: TitleFirePresentationTune,
    #[serde(default)]
    figure_layers: TitleCampFigureLayerTunes,
    #[serde(default)]
    extra_layers: TitleCampExtraLayerTunes,
    #[serde(default)]
    ambient: TitleAmbientPresentationTune,
}

impl From<TitleCampSceneLayoutFile> for TitleCampSceneLayout {
    fn from(f: TitleCampSceneLayoutFile) -> Self {
        let mut players = f.players;
        if let Some(lead) = f.lead_slot {
            players[0] = lead;
        }
        if let Some(ally) = f.ally_slot {
            players[1] = ally;
        }
        Self {
            fireplace: f.fireplace,
            players,
            anchors: f.anchors,
            fire_presentation: f.fire_presentation,
            figure_layers: f.figure_layers,
            extra_layers: f.extra_layers,
            ambient: f.ambient,
        }
    }
}

impl Default for TitleCampSceneLayout {
    fn default() -> Self {
        Self {
            fireplace: PresentationElementTune::default_fireplace(),
            players: default_six_player_tunes(),
            anchors: default_title_anchors(),
            fire_presentation: TitleFirePresentationTune::default(),
            figure_layers: TitleCampFigureLayerTunes::default(),
            extra_layers: TitleCampExtraLayerTunes::default(),
            ambient: TitleAmbientPresentationTune::default(),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default, Serialize, Deserialize)]
pub enum TitleCampSceneTuneTarget {
    #[default]
    Fireplace,
    Player1,
    Player2,
    Player3,
    Player4,
    Player5,
    Player6,
}

impl TitleCampSceneTuneTarget {
    pub fn next(self) -> Self {
        match self {
            Self::Fireplace => Self::Player1,
            Self::Player1 => Self::Player2,
            Self::Player2 => Self::Player3,
            Self::Player3 => Self::Player4,
            Self::Player4 => Self::Player5,
            Self::Player5 => Self::Player6,
            Self::Player6 => Self::Fireplace,
        }
    }

    pub fn prev(self) -> Self {
        match self {
            Self::Fireplace => Self::Player6,
            Self::Player1 => Self::Fireplace,
            Self::Player2 => Self::Player1,
            Self::Player3 => Self::Player2,
            Self::Player4 => Self::Player3,
            Self::Player5 => Self::Player4,
            Self::Player6 => Self::Player5,
        }
    }

    #[must_use]
    pub fn seat_index(self) -> Option<usize> {
        match self {
            Self::Fireplace => None,
            Self::Player1 => Some(0),
            Self::Player2 => Some(1),
            Self::Player3 => Some(2),
            Self::Player4 => Some(3),
            Self::Player5 => Some(4),
            Self::Player6 => Some(5),
        }
    }

    #[must_use]
    pub fn from_seat_index(seat: usize) -> Option<Self> {
        match seat {
            0 => Some(Self::Player1),
            1 => Some(Self::Player2),
            2 => Some(Self::Player3),
            3 => Some(Self::Player4),
            4 => Some(Self::Player5),
            5 => Some(Self::Player6),
            _ => None,
        }
    }
}

impl TitleCampSceneLayout {
    pub fn try_load_from_disk() -> Self {
        let scene_p = PathBuf::from(SCENE_PATH);
        if let Ok(text) = std::fs::read_to_string(&scene_p) {
            match serde_json::from_str::<TitleCampSceneLayoutFile>(&text) {
                Ok(f) => return f.into(),
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

    #[must_use]
    pub fn player_tune(&self, seat: usize) -> &PresentationElementTune {
        &self.players[seat.min(CAMP_FIRE_SEATS.saturating_sub(1))]
    }

    pub fn player_tune_mut(&mut self, seat: usize) -> &mut PresentationElementTune {
        &mut self.players[seat.min(CAMP_FIRE_SEATS.saturating_sub(1))]
    }

    pub fn tune_mut(&mut self, target: TitleCampSceneTuneTarget) -> &mut PresentationElementTune {
        match target.seat_index() {
            Some(i) => self.player_tune_mut(i),
            None => &mut self.fireplace,
        }
    }

    pub fn tune(&self, target: TitleCampSceneTuneTarget) -> &PresentationElementTune {
        match target.seat_index() {
            Some(i) => self.player_tune(i),
            None => &self.fireplace,
        }
    }

    #[must_use]
    pub fn layer_tune(&self, id: &str) -> Option<&PresentationLayerTune> {
        let id = normalize_layer_id(id);
        let (element, layer) = parse_layer_id(id.as_str())?;
        if element == TITLE_ELEMENT_FIREPLACE {
            return self.fire_presentation.layers.get_by_key(layer);
        }
        if layer == "emoji" {
            if let Some(seat) = parse_player_element_seat(element) {
                return Some(&self.figure_layers.players[seat].emoji);
            }
        }
        self.extra_layers.0.get(id.as_str())
    }

    pub fn layer_tune_mut(&mut self, id: &str) -> Option<&mut PresentationLayerTune> {
        let id = normalize_layer_id(id);
        let (element, layer) = parse_layer_id(id.as_str())?;
        if element == TITLE_ELEMENT_FIREPLACE {
            return self.fire_presentation.layers.get_mut_by_key(layer);
        }
        if layer == "emoji" {
            if let Some(seat) = parse_player_element_seat(element) {
                return Some(&mut self.figure_layers.players[seat].emoji);
            }
        }
        Some(self.extra_layers.0.entry(id).or_default())
    }

    pub fn reset_all_layer_placements(&mut self) {
        self.fire_presentation.layers.reset_all();
        for slot in &mut self.figure_layers.players {
            slot.emoji.reset_placement_to_anchor();
        }
        for tune in self.extra_layers.0.values_mut() {
            tune.reset_placement_to_anchor();
        }
    }

    pub fn reset_all_player_placements(&mut self) {
        for tune in &mut self.players {
            tune.reset_placement_to_anchor();
        }
        self.reset_all_layer_placements();
    }
}

#[cfg(test)]
mod layer_routing_tests {
    use super::*;
    use crate::presentation::layer::compose_layer_id;

    #[test]
    fn layer_tune_routes_fireplace_and_player_seat() {
        let mut layout = TitleCampSceneLayout::default();
        assert!(layout
            .layer_tune_mut(compose_layer_id(TITLE_ELEMENT_FIREPLACE, "stack").as_str())
            .is_some());
        layout.figure_layers.players[2].emoji.offset_x = 3.0;
        assert_eq!(
            layout
                .layer_tune(compose_layer_id("player3", "emoji").as_str())
                .unwrap()
                .offset_x,
            3.0
        );
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
