//! Presentation-only scene composition: anchors, pivots, layered visuals, and tuning hooks.
//! Keeps simulation/domain free of render assumptions; JSON is the source of truth for camp layout.
//!
//! See `docs/presentation-scene-composition.md`.

mod anchor;
pub mod editor;
pub mod element;
mod fire;
pub mod markers;
pub use markers::{CampfirePresentationRoot, PresentationFireLayerHost};
pub mod pivot;
pub mod scene;
mod track;

pub use anchor::{resolve_element_translation_px, SceneAnchorPose};
pub use editor::{PresentationEditorGizmoFlags, PresentationEditorSession};
pub use element::{PresentationElementId, PresentationElementTune, PresentationLayerTune};
pub use fire::{
    apply_layer_ui_transform, is_fire_layer_id, spawn_title_fire_layers, FirePresentationLayerTunes,
    PresentationFirePart, PresentationFireStackRoot, TitleFirePresentationTune,
    FIRE_LAYER_BASE, FIRE_LAYER_FLAME, FIRE_LAYER_GLOW, FIRE_LAYER_GROUND, FIRE_LAYER_STACK,
    TITLE_FIRE_GROUND_LIGHT_H_PX, TITLE_FIRE_GROUND_LIGHT_W_MULT, TITLE_FIRE_TRACK_EVAL_SEED,
};
pub use pivot::{pivot_translation_compensation_px, ScenePivot};
pub use scene::{TitleAmbientPresentationTune, TitleCampSceneLayout, TitleCampSceneTuneTarget};
pub use track::{CurveBlendMode, CurveKind, CurveLayer, PresentationTrack};
