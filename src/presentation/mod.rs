//! Presentation-only scene composition: anchors, pivots, layered visuals, and tuning hooks.
//! Keeps simulation/domain free of render assumptions; JSON is the source of truth for camp layout.
//!
//! See `docs/presentation-scene-composition.md`.

mod anchor;
pub mod editor;
pub mod element;
mod fire;
pub mod markers;
pub mod pivot;
pub mod scene;

pub use anchor::{resolve_element_translation_px, SceneAnchorPose};
pub use editor::{PresentationEditorGizmoFlags, PresentationEditorSession};
pub use element::{PresentationElementId, PresentationElementTune};
pub use fire::{
    spawn_title_fire_layers, PresentationFirePart, PresentationFireStackRoot, TitleFirePresentationTune,
    TITLE_FIRE_GROUND_LIGHT_H_PX, TITLE_FIRE_GROUND_LIGHT_W_MULT,
};
pub use pivot::{pivot_translation_compensation_px, ScenePivot};
pub use scene::{
    TitleAmbientPresentationTune, TitleCampSceneLayout, TitleCampSceneTuneTarget,
};
