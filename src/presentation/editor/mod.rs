//! In-engine presentation editor: session state, overlay UI, and (later) mouse / gizmos.

mod field_edit;
mod mouse;
mod overlay;
mod selection;

pub use overlay::{
    spawn_presentation_editor_overlay, PresentationEditorBannerHintText,
    PresentationEditorBannerTitleText, PresentationEditorFireAtmosphereBlock,
    PresentationEditorHierarchyButton, PresentationEditorPivotSummaryText,
    PresentationEditorReloadButton, PresentationEditorResetAllButton,
    PresentationEditorResetCenterButton, PresentationEditorRoot, PresentationEditorSaveButton,
    PresentationEditorSettingsToggleButton, PresentationEditorSettingsToggleText,
    PresentationEditorTuneDeltaButton, PresentationEditorTuneField,
    PresentationEditorTuneValueButton, PresentationEditorTuneValueText,
};

pub use field_edit::{presentation_editor_tune_field_keyboard, PresentationEditorFieldEditState};

#[cfg(debug_assertions)]
pub use mouse::{presentation_editor_drag, presentation_editor_pick};
#[cfg(debug_assertions)]
pub use selection::presentation_editor_hover_outline;

use crate::presentation::element::{
    PresentationElementId, PresentationElementTune, PresentationLayerTune,
};
use crate::presentation::is_presentation_layer_id;
use crate::presentation::scene::TitleCampSceneLayout;
use crate::presentation::track::CurveKind;
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

/// Tracks an in-progress drag on a [`PresentationElementHost`] (debug title editor).
#[derive(Resource, Default)]
pub struct PresentationEditorDragState {
    pub active_host_drag: Option<PresentationElementId>,
    /// Pointer motion accumulated before layout offsets change (avoids click nudge).
    pub pending_delta: Vec2,
    /// True after the pointer has moved past [`DRAG_START_THRESHOLD_PX`].
    pub dragging: bool,
}

pub(crate) const DRAG_START_THRESHOLD_PX: f32 = 1.0;

/// Authoring session for layout / scene presentation editing (title camp first consumer).
#[derive(Resource)]
pub struct PresentationEditorSession {
    /// When true, layout hotkeys and editor overlays are active.
    pub active: bool,
    /// Selected element id (e.g. `"fireplace"`, `"player3"` for title camp seats).
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

pub use crate::presentation::layer::{
    parse_player_element_seat, title_player_element_id, TITLE_ELEMENT_ALLY_SLOT,
    TITLE_ELEMENT_FIREPLACE, TITLE_ELEMENT_LEAD_SLOT,
};

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
        match self
            .selected_element
            .as_deref()
            .map(crate::presentation::normalize_layer_id)
        {
            Some(id) if id == TITLE_ELEMENT_FIREPLACE => {
                crate::presentation::scene::TitleCampSceneTuneTarget::Fireplace
            }
            Some(id) => {
                let id = id.as_str();
                if let Some(seat) = parse_player_element_seat(id) {
                    crate::presentation::scene::TitleCampSceneTuneTarget::from_seat_index(seat)
                        .unwrap_or_default()
                } else {
                    crate::presentation::scene::TitleCampSceneTuneTarget::Fireplace
                }
            }
            None => crate::presentation::scene::TitleCampSceneTuneTarget::Fireplace,
        }
    }

    /// Compatibility: updates [`Self::selected_element`] from a title camp tune target.
    pub fn set_target(&mut self, target: crate::presentation::scene::TitleCampSceneTuneTarget) {
        self.selected_element = Some(match target.seat_index() {
            None => TITLE_ELEMENT_FIREPLACE.to_string(),
            Some(seat) => title_player_element_id(seat + 1).to_string(),
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
    field_edit: Res<PresentationEditorFieldEditState>,
    hint_logged: Option<Res<crate::ui::scene_tune::TitleSceneTuneHintLogged>>,
    mut text_queries: ParamSet<(
        Query<&mut Text, With<PresentationEditorBannerTitleText>>,
        Query<&mut Text, With<PresentationEditorBannerHintText>>,
        Query<&mut Text, With<PresentationEditorPivotSummaryText>>,
        Query<(&PresentationEditorTuneValueText, &mut Text)>,
        Query<&mut Text, With<PresentationEditorSettingsToggleText>>,
    )>,
    mut hierarchy: Query<
        (
            &PresentationEditorHierarchyButton,
            &UiButtonPalette,
            &mut BackgroundColor,
            &mut BorderColor,
        ),
        With<Button>,
    >,
    mut fire_atmosphere: Query<&mut Visibility, With<PresentationEditorFireAtmosphereBlock>>,
) {
    use crate::ui::scene_tune::TITLE_SCENE_TUNE_BANNER_HINT;

    let hint_changed = hint_logged
        .as_ref()
        .map(|r| r.is_changed())
        .unwrap_or(false);
    let show_first_visit = hint_logged.as_ref().map(|h| !h.0).unwrap_or(false);

    if !session.is_changed()
        && !layout.is_changed()
        && !field_edit.is_changed()
        && !hint_changed
        && !show_first_visit
    {
        return;
    }

    let sel = banner_selected_id(&session);
    let title_line = format!("Presentation Mode · {sel}");
    for mut t in text_queries.p0() {
        **t = title_line.clone();
    }

    for mut t in text_queries.p1() {
        **t = if show_first_visit {
            TITLE_SCENE_TUNE_BANNER_HINT.to_string()
        } else {
            String::new()
        };
    }

    for mut t in text_queries.p4() {
        **t = if session.active {
            "Presentation editor · ON".to_string()
        } else {
            "Presentation editor · OFF".to_string()
        };
    }

    if is_presentation_layer_id(sel) {
        for mut t in text_queries.p2() {
            **t = format!("Layer · {sel}");
        }
    } else {
        let tune = tune_for_scene(&layout, sel);
        for mut t in text_queries.p2() {
            **t = format_pivot_line(tune);
        }
    }

    for (marker, mut text) in text_queries.p3() {
        **text = if field_edit.field == Some(marker.0) {
            format!("{}▏", field_edit.buffer)
        } else {
            format_editor_tune_field(&layout, sel, marker.0)
        };
    }

    let show_fire_atm = selection_uses_fire_atmosphere(sel);
    for mut vis in &mut fire_atmosphere {
        *vis = if show_fire_atm {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
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

fn apply_layer_tune_delta(
    tune: &mut PresentationLayerTune,
    field: PresentationEditorTuneField,
    positive: bool,
    coarse: bool,
) {
    let sign = if positive { 1.0 } else { -1.0 };
    let mult = if coarse { 10.0 } else { 1.0 };
    match field {
        PresentationEditorTuneField::OffsetX => tune.offset_x += sign * mult,
        PresentationEditorTuneField::OffsetY => tune.offset_y += sign * mult,
        PresentationEditorTuneField::ScaleX => {
            tune.scale_x = (tune.scale_x + sign * 0.01 * mult).clamp(0.05, 4.0);
        }
        PresentationEditorTuneField::ScaleY => {
            tune.scale_y = (tune.scale_y + sign * 0.01 * mult).clamp(0.05, 4.0);
        }
        _ => {}
    }
}

#[must_use]
pub(crate) fn selection_uses_fire_atmosphere(sel: &str) -> bool {
    let sel = crate::presentation::normalize_layer_id(sel);
    let s = sel.as_str();
    s == TITLE_ELEMENT_FIREPLACE || s.starts_with("fireplace:") || s.starts_with("fire:")
}

fn apply_fire_atmosphere_delta(
    fire: &mut crate::presentation::TitleFirePresentationTune,
    field: PresentationEditorTuneField,
    positive: bool,
    coarse: bool,
) {
    let sign = if positive { 1.0 } else { -1.0 };
    let mult = if coarse { 10.0 } else { 1.0 };
    match field {
        PresentationEditorTuneField::FireGlowAlphaBase => {
            let track = fire.glow_alpha_track_mut();
            track.base_value = (track.base_value + sign * 0.01 * mult).clamp(0.0, 1.0);
        }
        PresentationEditorTuneField::FireGlowBreathHz => {
            let track = fire.glow_alpha_track_mut();
            if track.layers.is_empty() {
                track.layers.push(crate::presentation::CurveLayer {
                    kind: CurveKind::Sine,
                    frequency_hz: 0.21,
                    amplitude: 0.03,
                    phase: 0.0,
                    weight: 1.0,
                    blend: crate::presentation::CurveBlendMode::Multiplicative,
                });
            }
            let layer = &mut track.layers[0];
            layer.frequency_hz = (layer.frequency_hz + sign * 0.01 * mult).clamp(0.03, 1.2);
        }
        PresentationEditorTuneField::FireGroundAlphaBase => {
            let track = fire.ground_alpha_track_mut();
            track.base_value = (track.base_value + sign * 0.01 * mult).clamp(0.0, 1.0);
        }
        PresentationEditorTuneField::FireCrossfadeSecs => {
            fire.crossfade_period_secs =
                (fire.crossfade_period_secs + sign * 0.1 * mult).clamp(1.0, 24.0);
        }
        PresentationEditorTuneField::FireGlowMinAlpha => {
            fire.glow_min_alpha = (fire.glow_min_alpha + sign * 0.01 * mult).clamp(0.0, 0.5);
        }
        _ => {}
    }
}

/// Apply inspector delta to the current selection (camp element or fire sub-layer).
pub fn apply_editor_tune_delta(
    layout: &mut TitleCampSceneLayout,
    selected_id: &str,
    field: PresentationEditorTuneField,
    positive: bool,
    coarse: bool,
) {
    if matches!(
        field,
        PresentationEditorTuneField::FireGlowAlphaBase
            | PresentationEditorTuneField::FireGlowBreathHz
            | PresentationEditorTuneField::FireGroundAlphaBase
            | PresentationEditorTuneField::FireCrossfadeSecs
            | PresentationEditorTuneField::FireGlowMinAlpha
    ) {
        apply_fire_atmosphere_delta(&mut layout.fire_presentation, field, positive, coarse);
        return;
    }

    if is_presentation_layer_id(selected_id) {
        if let Some(layer) = layout.layer_tune_mut(selected_id) {
            apply_layer_tune_delta(layer, field, positive, coarse);
        }
    } else {
        apply_presentation_tune_delta(
            tune_for_scene_mut(layout, selected_id),
            field,
            positive,
            coarse,
        );
    }
}

/// Apply the same deltas as [`crate::ui::scene_tune::title_scene_tune_hotkeys`].
pub fn apply_presentation_tune_delta(
    tune: &mut PresentationElementTune,
    field: PresentationEditorTuneField,
    positive: bool,
    coarse: bool,
) {
    let sign = if positive { 1.0 } else { -1.0 };
    let mult = if coarse { 10.0 } else { 1.0 };
    match field {
        PresentationEditorTuneField::OffsetX => tune.offset_x += sign * mult,
        PresentationEditorTuneField::OffsetY => tune.offset_y += sign * mult,
        PresentationEditorTuneField::ScaleX => {
            tune.scale_x = (tune.scale_x + sign * 0.01 * mult).clamp(0.15, 3.0);
        }
        PresentationEditorTuneField::ScaleY => {
            tune.scale_y = (tune.scale_y + sign * 0.01 * mult).clamp(0.15, 3.0);
        }
        PresentationEditorTuneField::SizeBasis => {
            tune.size_basis = (tune.size_basis + sign * 2.0 * mult).clamp(20.0, 640.0);
        }
        PresentationEditorTuneField::RotationDeg => tune.rotation_deg += sign * mult,
        PresentationEditorTuneField::Exposure => {
            tune.exposure = (tune.exposure + sign * 0.02 * mult).clamp(0.0, 4.0);
        }
        PresentationEditorTuneField::Glow => {
            tune.glow = (tune.glow + sign * 0.02 * mult).clamp(0.0, 3.0);
        }
        PresentationEditorTuneField::Bloom => {
            tune.bloom = (tune.bloom + sign * 0.02 * mult).clamp(0.0, 2.0);
        }
        PresentationEditorTuneField::GlobalZ => {
            let d = if positive {
                mult as i32
            } else {
                -(mult as i32)
            };
            tune.global_z = tune.global_z.saturating_add(d);
        }
        PresentationEditorTuneField::FireGlowAlphaBase
        | PresentationEditorTuneField::FireGlowBreathHz
        | PresentationEditorTuneField::FireGroundAlphaBase
        | PresentationEditorTuneField::FireCrossfadeSecs
        | PresentationEditorTuneField::FireGlowMinAlpha => {}
    }
}

pub(crate) fn tune_for_scene_mut<'a>(
    layout: &'a mut TitleCampSceneLayout,
    id: &str,
) -> &'a mut PresentationElementTune {
    let id = crate::presentation::normalize_layer_id(id);
    if let Some(seat) = parse_player_element_seat(id.as_str()) {
        return layout.player_tune_mut(seat);
    }
    &mut layout.fireplace
}

/// Reset offsets, scale, and rotation for every title-camp element.
pub fn reset_all_title_placements(layout: &mut TitleCampSceneLayout) {
    layout.fireplace.reset_placement_to_anchor();
    layout.reset_all_player_placements();
}

pub fn reset_editor_selection_placement(layout: &mut TitleCampSceneLayout, selected_id: &str) {
    if is_presentation_layer_id(selected_id) {
        if let Some(layer) = layout.layer_tune_mut(selected_id) {
            layer.reset_placement_to_anchor();
        }
    } else {
        tune_for_scene_mut(layout, selected_id).reset_placement_to_anchor();
    }
}

pub fn tune_for_scene<'a>(
    layout: &'a TitleCampSceneLayout,
    id: &str,
) -> &'a PresentationElementTune {
    let id = crate::presentation::normalize_layer_id(id);
    if let Some(seat) = parse_player_element_seat(id.as_str()) {
        return layout.player_tune(seat);
    }
    &layout.fireplace
}

pub(crate) fn format_editor_tune_field(
    layout: &TitleCampSceneLayout,
    selected_id: &str,
    field: PresentationEditorTuneField,
) -> String {
    let fire = &layout.fire_presentation;
    match field {
        PresentationEditorTuneField::FireGlowAlphaBase => {
            let base = fire
                .glow_alpha
                .as_ref()
                .map(|t| t.base_value)
                .unwrap_or_else(|| fire.synthesize_glow_track_from_legacy().base_value);
            return format!("{:.3}", base);
        }
        PresentationEditorTuneField::FireGlowBreathHz => {
            let hz = fire
                .glow_alpha
                .as_ref()
                .and_then(|t| t.layers.first())
                .map(|l| l.frequency_hz)
                .unwrap_or(fire.glow_pulse_hz);
            return format!("{:.2}", hz);
        }
        PresentationEditorTuneField::FireGroundAlphaBase => {
            let base = fire
                .ground_alpha
                .as_ref()
                .map(|t| t.base_value)
                .unwrap_or_else(|| fire.synthesize_ground_track_from_legacy().base_value);
            return format!("{:.3}", base);
        }
        PresentationEditorTuneField::FireCrossfadeSecs => {
            return format!("{:.1}", fire.crossfade_period_secs);
        }
        PresentationEditorTuneField::FireGlowMinAlpha => {
            return format!("{:.3}", fire.glow_min_alpha);
        }
        _ => {}
    }
    if let Some(layer) = layout.layer_tune(selected_id) {
        return format_layer_tune_field(layer, field);
    }
    format_tune_field(tune_for_scene(layout, selected_id), field)
}

pub(crate) fn format_layer_tune_field(
    tune: &PresentationLayerTune,
    field: PresentationEditorTuneField,
) -> String {
    match field {
        PresentationEditorTuneField::OffsetX => format!("{:.1}", tune.offset_x),
        PresentationEditorTuneField::OffsetY => format!("{:.1}", tune.offset_y),
        PresentationEditorTuneField::ScaleX => format!("{:.2}", tune.scale_x),
        PresentationEditorTuneField::ScaleY => format!("{:.2}", tune.scale_y),
        PresentationEditorTuneField::SizeBasis
        | PresentationEditorTuneField::RotationDeg
        | PresentationEditorTuneField::Exposure
        | PresentationEditorTuneField::Glow
        | PresentationEditorTuneField::Bloom
        | PresentationEditorTuneField::GlobalZ
        | PresentationEditorTuneField::FireGlowAlphaBase
        | PresentationEditorTuneField::FireGlowBreathHz
        | PresentationEditorTuneField::FireGroundAlphaBase
        | PresentationEditorTuneField::FireCrossfadeSecs
        | PresentationEditorTuneField::FireGlowMinAlpha => "—".to_string(),
    }
}

pub(crate) fn format_tune_field(
    tune: &PresentationElementTune,
    field: PresentationEditorTuneField,
) -> String {
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
        PresentationEditorTuneField::FireGlowAlphaBase
        | PresentationEditorTuneField::FireGlowBreathHz
        | PresentationEditorTuneField::FireGroundAlphaBase
        | PresentationEditorTuneField::FireCrossfadeSecs
        | PresentationEditorTuneField::FireGlowMinAlpha => "—".to_string(),
    }
}
