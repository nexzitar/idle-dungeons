//! In-engine presentation editor: session state and (later) overlay / gizmos.

use crate::presentation::element::PresentationElementId;
use crate::presentation::scene::TitleCampSceneTuneTarget;
use bevy::prelude::Resource;

/// Debugging overlays toggled while editing presentation elements.
#[derive(Clone)]
pub struct PresentationEditorGizmoFlags {
    pub selection_outline: bool,
    pub pivot_marker: bool,
    pub anchor_marker: bool,
    pub layer_label: bool,
    pub glow_radius_preview: bool,
}

impl Default for PresentationEditorGizmoFlags {
    fn default() -> Self {
        Self {
            selection_outline: true,
            pivot_marker: false,
            anchor_marker: false,
            layer_label: false,
            glow_radius_preview: false,
        }
    }
}

/// Authoring session for layout / scene presentation editing (title camp first consumer).
#[derive(Resource)]
pub struct PresentationEditorSession {
    /// When true, layout hotkeys and editor overlays are active.
    pub active: bool,
    /// Selected element id (e.g. `"fireplace"`, `"lead_slot"`, `"ally_slot"` for title camp).
    pub selected_element: Option<PresentationElementId>,
    pub gizmo_flags: PresentationEditorGizmoFlags,
}

impl Default for PresentationEditorSession {
    fn default() -> Self {
        Self {
            active: false,
            selected_element: None,
            gizmo_flags: PresentationEditorGizmoFlags::default(),
        }
    }
}

/// Title-scene–specific ids used with [`TitleCampSceneTuneTarget`] compatibility.
pub const TITLE_ELEMENT_FIREPLACE: &str = "fireplace";
pub const TITLE_ELEMENT_LEAD_SLOT: &str = "lead_slot";
pub const TITLE_ELEMENT_ALLY_SLOT: &str = "ally_slot";

impl PresentationEditorSession {
    /// Compatibility: maps to [`Self::active`] (legacy `TitleSceneTuneSession.layout_mode`).
    #[must_use]
    pub fn layout_mode(&self) -> bool {
        self.active
    }

    /// Compatibility: maps to [`Self::active`].
    pub fn set_layout_mode(&mut self, value: bool) {
        self.active = value;
    }

    /// Convenience for backtick toggle (same as `set_layout_mode(!layout_mode())`).
    pub fn toggle_layout_mode(&mut self) {
        self.active = !self.active;
    }

    /// Compatibility: [`TitleCampSceneTuneTarget`] derived from [`Self::selected_element`].
    #[must_use]
    pub fn target(&self) -> TitleCampSceneTuneTarget {
        match self.selected_element.as_deref() {
            Some(TITLE_ELEMENT_LEAD_SLOT) => TitleCampSceneTuneTarget::LeadSlot,
            Some(TITLE_ELEMENT_ALLY_SLOT) => TitleCampSceneTuneTarget::AllySlot,
            Some(TITLE_ELEMENT_FIREPLACE) | None | Some(_) => TitleCampSceneTuneTarget::Fireplace,
        }
    }

    /// Compatibility: updates [`Self::selected_element`] from a title camp tune target.
    pub fn set_target(&mut self, target: TitleCampSceneTuneTarget) {
        self.selected_element = Some(match target {
            TitleCampSceneTuneTarget::Fireplace => TITLE_ELEMENT_FIREPLACE.to_string(),
            TitleCampSceneTuneTarget::LeadSlot => TITLE_ELEMENT_LEAD_SLOT.to_string(),
            TitleCampSceneTuneTarget::AllySlot => TITLE_ELEMENT_ALLY_SLOT.to_string(),
        });
    }
}
