//! Presentation-only scene composition: anchors, pivots, layered visuals, and tuning hooks.
//! Keeps simulation/domain free of render assumptions; JSON is the source of truth for camp layout.
//!
//! See `docs/presentation-scene-composition.md`.

mod anchor;
pub mod editor;
pub mod element;
mod fire;
pub mod layer;
pub mod markers;
pub use markers::{CampfirePresentationRoot, PresentationFireLayerHost, PresentationLayerHost};
pub mod pivot;
pub mod scene;
mod track;

pub use anchor::{resolve_element_translation_px, SceneAnchorPose};
pub use editor::{PresentationEditorGizmoFlags, PresentationEditorSession};
pub use element::{PresentationElementId, PresentationElementTune, PresentationLayerTune};
pub use fire::{
    apply_layer_ui_transform, spawn_title_fire_layers, FirePresentationLayerTunes,
    PresentationFirePart, PresentationFireStackRoot, TitleFirePresentationTune,
    TITLE_FIRE_FLAME_BREATHE_AMP, TITLE_FIRE_FLAME_BREATHE_HZ, TITLE_FIRE_GLOW_CORE_INSET_X,
    TITLE_FIRE_GLOW_CORE_INSET_Y_BOTTOM, TITLE_FIRE_GLOW_CORE_INSET_Y_TOP,
    TITLE_FIRE_GLOW_HALO_INSET_X, TITLE_FIRE_GLOW_HALO_INSET_Y_BOTTOM,
    TITLE_FIRE_GLOW_HALO_INSET_Y_TOP, TITLE_FIRE_GROUND_LIGHT_H_PX, TITLE_FIRE_GROUND_LIGHT_W_MULT,
    TITLE_FIRE_TRACK_EVAL_SEED,
};
pub use layer::{
    compose_layer_id, is_presentation_layer_id, normalize_layer_id, parse_layer_id,
    FigureSlotLayerTunes, TitleCampExtraLayerTunes, TitleCampFigureLayerTunes,
    PresentationElementLayers, PresentationLayerRow, TITLE_CAMP_LAYER_REGISTRY,
    TITLE_ELEMENT_ALLY_SLOT, TITLE_ELEMENT_FIREPLACE, TITLE_ELEMENT_LEAD_SLOT,
};
pub use pivot::{pivot_translation_compensation_px, ScenePivot};
pub use scene::{TitleAmbientPresentationTune, TitleCampSceneLayout, TitleCampSceneTuneTarget};
pub use track::{CurveBlendMode, CurveKind, CurveLayer, PresentationTrack};
