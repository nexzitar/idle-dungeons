//! In-engine presentation editor: session state, overlay UI, and (later) mouse / gizmos.

mod overlay;

pub use overlay::{
    spawn_presentation_editor_overlay, PresentationEditorBannerHintText, PresentationEditorBannerTitleText,
    PresentationEditorHierarchyButton, PresentationEditorPivotSummaryText, PresentationEditorReloadButton,
    PresentationEditorRoot, PresentationEditorSaveButton, PresentationEditorSettingsToggleButton,
    PresentationEditorSettingsToggleText, PresentationEditorTuneDeltaButton, PresentationEditorTuneField,
    PresentationEditorTuneValueText,
};

use crate::presentation::element::{PresentationElementId, PresentationElementTune};
use crate::presentation::scene::TitleCampSceneLayout;
use crate::ui::components::UiButtonPalette;
use crate::ui::scene_tune::TitleSceneLayout;
use crate::ui::theme::UiTheme;
use bevy::prelude::*;
use overlay::{banner_selected_id, format_pivot_line};

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

/// Title-scene–specific ids used with [`crate::presentation::scene::TitleCampSceneTuneTarget`] compatibility.
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

    /// Compatibility: [`crate::presentation::scene::TitleCampSceneTuneTarget`] derived from [`Self::selected_element`].
    #[must_use]
    pub fn target(&self) -> crate::presentation::scene::TitleCampSceneTuneTarget {
        match self.selected_element.as_deref() {
            Some(TITLE_ELEMENT_LEAD_SLOT) => crate::presentation::scene::TitleCampSceneTuneTarget::LeadSlot,
            Some(TITLE_ELEMENT_ALLY_SLOT) => crate::presentation::scene::TitleCampSceneTuneTarget::AllySlot,
            Some(TITLE_ELEMENT_FIREPLACE) | None | Some(_) => {
                crate::presentation::scene::TitleCampSceneTuneTarget::Fireplace
            }
        }
    }

    /// Compatibility: updates [`Self::selected_element`] from a title camp tune target.
    pub fn set_target(&mut self, target: crate::presentation::scene::TitleCampSceneTuneTarget) {
        self.selected_element = Some(match target {
            crate::presentation::scene::TitleCampSceneTuneTarget::Fireplace => {
                TITLE_ELEMENT_FIREPLACE.to_string()
            }
            crate::presentation::scene::TitleCampSceneTuneTarget::LeadSlot => {
                TITLE_ELEMENT_LEAD_SLOT.to_string()
            }
            crate::presentation::scene::TitleCampSceneTuneTarget::AllySlot => {
                TITLE_ELEMENT_ALLY_SLOT.to_string()
            }
        });
    }
}

// --- Systems ---

pub fn toggle_presentation_editor_visibility(
    session: Res<PresentationEditorSession>,
    mut q: Query<&mut Visibility, With<PresentationEditorRoot>>,
) {
    if !session.is_changed() {
        return;
    }
    let vis = if session.active {
        Visibility::Visible
    } else {
        Visibility::Hidden
    };
    for mut v in &mut q {
        *v = vis;
    }
}

#[cfg(debug_assertions)]
pub fn sync_presentation_editor_ui(
    session: Res<PresentationEditorSession>,
    layout: Res<TitleSceneLayout>,
    hint_logged: Option<Res<crate::ui::scene_tune::TitleSceneTuneHintLogged>>,
    mut banner: Query<&mut Text, With<PresentationEditorBannerTitleText>>,
    mut hint: Query<&mut Text, With<PresentationEditorBannerHintText>>,
    mut pivot_line: Query<&mut Text, With<PresentationEditorPivotSummaryText>>,
    mut field_texts: Query<(&PresentationEditorTuneValueText, &mut Text)>,
    mut toggle_label: Query<&mut Text, With<PresentationEditorSettingsToggleText>>,
    mut hierarchy: Query<
        (
            &PresentationEditorHierarchyButton,
            &UiButtonPalette,
            &mut BackgroundColor,
            &mut BorderColor,
        ),
        With<Button>,
    >,
) {
    use crate::ui::scene_tune::TITLE_SCENE_TUNE_BANNER_HINT;

    let hint_changed = hint_logged.as_ref().map(|r| r.is_changed()).unwrap_or(false);
    let show_first_visit = hint_logged.as_ref().map(|h| !h.0).unwrap_or(false);

    if !session.is_changed() && !layout.is_changed() && !hint_changed && !show_first_visit {
        return;
    }

    let sel = banner_selected_id(&session);
    let title_line = format!("Presentation Mode · {sel}");
    for mut t in &mut banner {
        **t = title_line.clone();
    }

    for mut t in &mut hint {
        **t = if show_first_visit {
            TITLE_SCENE_TUNE_BANNER_HINT.to_string()
        } else {
            String::new()
        };
    }

    for mut t in &mut toggle_label {
        **t = if session.active {
            "Presentation editor · ON".to_string()
        } else {
            "Presentation editor · OFF".to_string()
        };
    }

    let tune = tune_for_scene(&layout, sel);
    for mut t in &mut pivot_line {
        **t = format_pivot_line(tune);
    }

    for (marker, mut text) in &mut field_texts {
        **text = format_tune_field(tune, marker.0);
    }

    let sel_owned = sel.to_string();
    for (hb, pal, mut bg, mut bd) in &mut hierarchy {
        let on = hb.0 == sel_owned;
        if on {
            *bg = UiTheme::stone_highlight().into();
            *bd = BorderColor::all(UiTheme::muted_gold());
        } else {
            *bg = pal.idle_bg.into();
            *bd = BorderColor::from(pal.idle_border);
        }
    }
}

/// Apply the same deltas as [`crate::ui::scene_tune::title_scene_tune_hotkeys`].
pub fn apply_presentation_tune_delta(
    tune: &mut PresentationElementTune,
    field: PresentationEditorTuneField,
    positive: bool,
) {
    let sign = if positive { 1.0 } else { -1.0 };
    match field {
        PresentationEditorTuneField::OffsetX => tune.offset_x += sign * 1.0,
        PresentationEditorTuneField::OffsetY => tune.offset_y += sign * 1.0,
        PresentationEditorTuneField::ScaleX => {
            tune.scale_x = (tune.scale_x + sign * 0.01).clamp(0.15, 3.0);
        }
        PresentationEditorTuneField::ScaleY => {
            tune.scale_y = (tune.scale_y + sign * 0.01).clamp(0.15, 3.0);
        }
        PresentationEditorTuneField::SizeBasis => {
            tune.size_basis = (tune.size_basis + sign * 2.0).clamp(20.0, 640.0);
        }
        PresentationEditorTuneField::RotationDeg => tune.rotation_deg += sign * 1.0,
        PresentationEditorTuneField::Exposure => {
            tune.exposure = (tune.exposure + sign * 0.02).clamp(0.0, 4.0);
        }
        PresentationEditorTuneField::Glow => {
            tune.glow = (tune.glow + sign * 0.02).clamp(0.0, 3.0);
        }
        PresentationEditorTuneField::Bloom => {
            tune.bloom = (tune.bloom + sign * 0.02).clamp(0.0, 2.0);
        }
        PresentationEditorTuneField::GlobalZ => {
            let d = if positive { 1 } else { -1 };
            tune.global_z = tune.global_z.saturating_add(d);
        }
    }
}

pub(crate) fn tune_for_scene_mut<'a>(
    layout: &'a mut TitleCampSceneLayout,
    id: &str,
) -> &'a mut PresentationElementTune {
    match id {
        TITLE_ELEMENT_LEAD_SLOT => &mut layout.lead_slot,
        TITLE_ELEMENT_ALLY_SLOT => &mut layout.ally_slot,
        _ => &mut layout.fireplace,
    }
}

pub(crate) fn tune_for_scene<'a>(
    layout: &'a TitleCampSceneLayout,
    id: &str,
) -> &'a PresentationElementTune {
    match id {
        TITLE_ELEMENT_LEAD_SLOT => &layout.lead_slot,
        TITLE_ELEMENT_ALLY_SLOT => &layout.ally_slot,
        _ => &layout.fireplace,
    }
}

fn format_tune_field(tune: &PresentationElementTune, field: PresentationEditorTuneField) -> String {
    match field {
        PresentationEditorTuneField::OffsetX => format!("{:.1}", tune.offset_x),
        PresentationEditorTuneField::OffsetY => format!("{:.1}", tune.offset_y),
        PresentationEditorTuneField::ScaleX => format!("{:.2}", tune.scale_x),
        PresentationEditorTuneField::ScaleY => format!("{:.2}", tune.scale_y),
        PresentationEditorTuneField::SizeBasis => format!("{:.1}", tune.size_basis),
        PresentationEditorTuneField::RotationDeg => format!("{:.1}", tune.rotation_deg),
        PresentationEditorTuneField::Exposure => format!("{:.2}", tune.exposure),
        PresentationEditorTuneField::Glow => format!("{:.2}", tune.glow),
        PresentationEditorTuneField::Bloom => format!("{:.2}", tune.bloom),
        PresentationEditorTuneField::GlobalZ => format!("{}", tune.global_z),
    }
}