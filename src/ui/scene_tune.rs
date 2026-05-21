//! Title scene layout tuning: multi-target JSON, debug hotkeys, and selection gizmos.
//!
//! - **File:** `assets/tuning/title_scene.json` (falls back to legacy `title_campfire.json`).
//! - **Debug:** press **`** (backtick) on Title to enter **layout mode** (hint logs once). Then
//!   **Tab** / **Shift+Tab** cycles **Fireplace · Lead · Ally**; arrows **or IJKL** nudge the
//!   selection. Hotkeys run **before** UI focus navigation so arrows do not drive the wrong node.
//! - **Bloom:** stored in JSON for future post-processing; not drawn in the minimal `2d` pipeline.

use bevy::log::{info, warn};
use bevy::prelude::*;
use bevy::text::{TextColor, TextFont};
use bevy::ui::{GlobalZIndex, UiTransform, Val2};
use crate::presentation::editor::{
    TITLE_ELEMENT_ALLY_SLOT, TITLE_ELEMENT_FIREPLACE, TITLE_ELEMENT_LEAD_SLOT,
};
use crate::presentation::{
    pivot_translation_compensation_px, resolve_element_translation_px, PresentationEditorSession,
    PresentationFirePart, PresentationFireStackRoot, SceneAnchorPose, TitleCampSceneLayout,
    TitleCampSceneTuneTarget, TITLE_FIRE_GROUND_LIGHT_H_PX, TITLE_FIRE_GROUND_LIGHT_W_MULT,
};
use std::collections::HashMap;
use std::f32::consts::TAU;

pub type TitleUiElementTune = crate::presentation::PresentationElementTune;
pub type TitleSceneLayout = TitleCampSceneLayout;
pub type TitleSceneTuneTarget = TitleCampSceneTuneTarget;

/// Legacy name for [`PresentationEditorSession`] (title camp scene tuning).
pub type TitleSceneTuneSession = PresentationEditorSession;

const FIREPLACE_IMG_W: f32 = 1254.0;
const FIREPLACE_IMG_H: f32 = 1254.0;

pub const TITLE_SCENE_TUNE_HINT: &str = "\
Title · scene layout (debug): **`** = toggle layout mode; Tab / Shift+Tab = target; \
arrows or IJKL = move; [ ] = scale (Alt=width, Shift=height); - / = size basis; \
Q / E = rotate; N / M = layer (global Z); 1/2 exposure · 3/4 glow · 5/6 bloom (stored); \
P = print JSON; Ctrl+S = save; F5 = reload";

/// One-line summary for the presentation editor banner before the console hint is marked logged.
#[cfg(debug_assertions)]
pub const TITLE_SCENE_TUNE_BANNER_HINT: &str = "\
Press ` to toggle layout mode · Tab / Shift+Tab cycles targets · arrows or IJKL move · [ ] scale";

/// Pixel width × height of the fireplace art box from [`TitleUiElementTune::size_basis`] (image height).
#[must_use]
pub fn title_fireplace_base_px(tune: &TitleUiElementTune) -> (f32, f32) {
    let aspect = FIREPLACE_IMG_H / FIREPLACE_IMG_W;
    let base_h = tune.size_basis;
    let base_w = base_h / aspect;
    (base_w.max(24.0), base_h.max(24.0))
}

fn tune_to_image_color(t: &TitleUiElementTune) -> Color {
    let e = t.exposure.clamp(0.0, 4.0);
    let g = t.glow.clamp(0.0, 3.0);
    let r = (e + g * 0.35).min(1.0);
    let gb = (e * 0.98 + g * 0.12).min(1.0);
    let b = (e * 0.95 + g * 0.08).min(1.0);
    Color::srgba(r, gb, b, 1.0)
}

pub fn tune_to_figure_emoji_color(t: &TitleUiElementTune) -> Color {
    let base = [0.95_f32, 0.82, 0.6, 0.88];
    let e = t.exposure.clamp(0.0, 4.0);
    let g = t.glow.clamp(0.0, 3.0);
    Color::srgba(
        (base[0] * e + g * 0.12).min(1.0),
        (base[1] * e + g * 0.08).min(1.0),
        (base[2] * e + g * 0.05).min(1.0),
        base[3],
    )
}

/// Host node for the campfire presentation: min size + UI transform from anchors/pivots.
pub fn apply_fireplace_host(
    node: &mut Node,
    ui_tx: &mut UiTransform,
    tune: &TitleUiElementTune,
    anchors: &HashMap<String, SceneAnchorPose>,
) {
    let (base_w, base_h) = title_fireplace_base_px(tune);
    node.margin = UiRect::default();
    node.min_width = Val::Px(base_w);
    node.min_height = Val::Px(base_h);
    node.width = Val::Auto;
    node.height = Val::Auto;
    let base = resolve_element_translation_px(
        anchors,
        tune.anchor_ref.as_deref(),
        tune.offset_x,
        tune.offset_y,
    );
    let comp = pivot_translation_compensation_px(
        tune.pivot,
        base_w,
        base_h,
        tune.scale_x,
        tune.scale_y,
    );
    ui_tx.translation = Val2::px(base.x + comp.x, base.y + comp.y);
    ui_tx.scale = Vec2::new(tune.scale_x, tune.scale_y);
    ui_tx.rotation = Rot2::degrees(tune.rotation_deg);
}

pub fn apply_figure_root(
    node: &mut Node,
    ui_tx: &mut UiTransform,
    tune: &TitleUiElementTune,
    anchors: &HashMap<String, SceneAnchorPose>,
) {
    let fig_w = tune.size_basis * 1.15;
    let fig_h = tune.size_basis * 1.85;
    node.margin = UiRect::default();
    let base = resolve_element_translation_px(
        anchors,
        tune.anchor_ref.as_deref(),
        tune.offset_x,
        tune.offset_y,
    );
    let comp =
        pivot_translation_compensation_px(tune.pivot, fig_w, fig_h, tune.scale_x, tune.scale_y);
    ui_tx.translation = Val2::px(base.x + comp.x, base.y + comp.y);
    ui_tx.scale = Vec2::new(tune.scale_x, tune.scale_y);
    ui_tx.rotation = Rot2::degrees(tune.rotation_deg);
}

pub fn sync_title_scene_elements(
    layout: Res<TitleSceneLayout>,
    mut groups: ParamSet<(
        Query<
            (
                &mut Node,
                &mut UiTransform,
                &mut GlobalZIndex,
            ),
            With<crate::ui::components::TitleCampfireTuneMarker>,
        >,
        Query<
            (
                &crate::ui::components::TitleCampFigureTuneMarker,
                &mut Node,
                &mut UiTransform,
                &mut GlobalZIndex,
            ),
            Without<crate::ui::components::TitleCampfireTuneMarker>,
        >,
        Query<
            (
                &crate::ui::components::TitleCampFigureEmoji,
                &mut TextFont,
                &mut TextColor,
            ),
            Without<crate::ui::components::TitleCampfireTuneMarker>,
        >,
    )>,
) {
    if !layout.is_changed() {
        return;
    }
    for (mut node, mut ui_tx, mut gz) in groups.p0().iter_mut() {
        apply_fireplace_host(&mut node, &mut ui_tx, &layout.fireplace, &layout.anchors);
        gz.0 = layout.fireplace.global_z;
    }
    for (marker, mut node, mut ui_tx, mut gz) in groups.p1().iter_mut() {
        let tune = match marker.0 {
            0 => &layout.lead_slot,
            _ => &layout.ally_slot,
        };
        apply_figure_root(&mut node, &mut ui_tx, tune, &layout.anchors);
        gz.0 = tune.global_z;
    }
    for (marker, mut font, mut color) in groups.p2().iter_mut() {
        let tune = match marker.0 {
            0 => &layout.lead_slot,
            _ => &layout.ally_slot,
        };
        font.font_size = tune.size_basis.clamp(8.0, 160.0);
        color.0 = tune_to_figure_emoji_color(tune);
    }
}

/// Layer geometry + base tint when [`TitleSceneLayout`] hot-reloads (separate system avoids Bevy `Query` conflicts on `Node`).
pub fn sync_title_fire_presentation_from_layout(
    layout: Res<TitleSceneLayout>,
    mut groups: ParamSet<(
        Query<&mut Node, With<PresentationFireStackRoot>>,
        Query<(&PresentationFirePart, &mut Node), With<PresentationFirePart>>,
        Query<(&PresentationFirePart, &mut ImageNode), With<ImageNode>>,
    )>,
) {
    if !layout.is_changed() {
        return;
    }
    let (base_w, base_h) = title_fireplace_base_px(&layout.fireplace);
    let cfg = &layout.fire_presentation;
    for mut node in groups.p0().iter_mut() {
        node.width = Val::Px(base_w);
        node.height = Val::Px(base_h);
    }
    for (part, mut node) in groups.p1().iter_mut() {
        match *part {
            PresentationFirePart::GroundLight if cfg.enabled && cfg.ground_max_alpha > 0.001 => {
                node.width = Val::Px(base_w * TITLE_FIRE_GROUND_LIGHT_W_MULT);
                node.height = Val::Px(TITLE_FIRE_GROUND_LIGHT_H_PX);
            }
            PresentationFirePart::Glow if cfg.enabled && cfg.glow_max_alpha > 0.001 => {
                node.left = Val::Px(-base_w * 0.42);
                node.top = Val::Px(-base_h * 0.32);
                node.right = Val::Px(-base_w * 0.42);
                node.bottom = Val::Px(-base_h * 0.52);
            }
            _ => {}
        }
    }
    for (part, mut img) in groups.p2().iter_mut() {
        if matches!(*part, PresentationFirePart::BaseStatic) {
            img.color = tune_to_image_color(&layout.fireplace);
        }
    }
}

/// Ambient motion for fire layers (presentation-only).
pub fn tick_title_fire_ambient(
    time: Res<Time>,
    layout: Res<TitleSceneLayout>,
    mut parts: ParamSet<(
        Query<(&PresentationFirePart, &mut ImageNode), With<ImageNode>>,
        Query<(&PresentationFirePart, &mut BackgroundColor)>,
        Query<(&PresentationFirePart, &mut UiTransform)>,
    )>,
) {
    let cfg = &layout.fire_presentation;
    if !cfg.enabled {
        return;
    }

    let t = time.elapsed_secs();
    let period = cfg.crossfade_period_secs.max(0.25);
    let n = cfg.flame_variants.clamp(2, 8) as f32;

    let glow_pulse =
        (1.0_f32 + cfg.glow_pulse_scale * (TAU * t * cfg.glow_pulse_hz).sin()).max(0.0);

    for (part, mut img) in parts.p0().iter_mut() {
        match *part {
            PresentationFirePart::Flame(idx) => {
                let i = idx as f32;
                let wave = 0.5 + 0.5 * (TAU * t / period + i / n * TAU).sin();
                let norm_den = (n * 0.5).max(1.0);
                let alpha = (wave / norm_den).clamp(0.12, 0.42);
                let base = img.color.to_srgba();
                img.color = Color::srgba(base.red, base.green, base.blue, alpha);
            }
            PresentationFirePart::Glow => {
                let a = (cfg.glow_max_alpha * 0.9 * glow_pulse)
                    .clamp(cfg.glow_min_alpha.max(0.0_f32), 1.0);
                img.color = Color::srgba(1.0, 0.55, 0.18, a);
            }
            _ => {}
        }
    }

    for (part, mut ui) in parts.p2().iter_mut() {
        if matches!(*part, PresentationFirePart::Glow) {
            ui.scale = Vec2::splat(glow_pulse);
            ui.rotation = Rot2::degrees(0.0_f32);
        }
    }

    let ground_phase =
        TAU * t * cfg.ground_flicker_hz + 0.3 * (TAU * t * (cfg.ground_flicker_hz * 0.5)).sin();
    let g_alpha =
        cfg.ground_max_alpha * (0.5 + 0.11 * ground_phase.sin());

    for (part, mut bg) in parts.p1().iter_mut() {
        if matches!(*part, PresentationFirePart::GroundLight) {
            let c = bg.0;
            let s = c.to_srgba();
            bg.0 = Color::srgba(s.red, s.green, s.blue, g_alpha.clamp(0.0, 1.0));
        }
    }
}

/// Magenta outline on the selected target while layout mode is on.
pub fn title_scene_tune_selection_gizmo(
    session: Res<PresentationEditorSession>,
    mut fireplace: Query<
        (&mut BorderColor, &mut Node),
        With<crate::ui::components::TitleCampfireTuneMarker>,
    >,
    mut figures: Query<
        (
            &crate::ui::components::TitleCampFigureTuneMarker,
            &mut BorderColor,
            &mut Node,
        ),
        Without<crate::ui::components::TitleCampfireTuneMarker>,
    >,
) {
    let show = session.layout_mode()
        && session.gizmo_flags.selection_outline;
    let sel = session
        .selected_element
        .as_deref()
        .unwrap_or(TITLE_ELEMENT_FIREPLACE);
    let col_active = BorderColor::all(Color::srgba(1.0, 0.2, 0.85, 0.95));
    let col_off = BorderColor::DEFAULT;

    for (mut border, mut node) in &mut fireplace {
        let on = show && sel == TITLE_ELEMENT_FIREPLACE;
        *border = if on { col_active } else { col_off };
        node.border = UiRect::all(Val::Px(if on { 2.0 } else { 0.0 }));
    }
    for (marker, mut border, mut node) in &mut figures {
        let id = if marker.0 == 0 {
            TITLE_ELEMENT_LEAD_SLOT
        } else {
            TITLE_ELEMENT_ALLY_SLOT
        };
        let on = show && sel == id;
        *border = if on { col_active } else { col_off };
        node.border = UiRect::all(Val::Px(if on { 2.0 } else { 0.0 }));
    }
}

#[cfg(debug_assertions)]
pub fn title_scene_tune_hotkeys(
    kb: Res<ButtonInput<KeyCode>>,
    mut layout: ResMut<TitleSceneLayout>,
    mut session: ResMut<PresentationEditorSession>,
) {
    if kb.just_pressed(KeyCode::Backquote) {
        session.toggle_layout_mode();
        info!(
            "Title scene layout mode: {} (target: {:?})",
            session.layout_mode(),
            session.target()
        );
    }
    if !session.layout_mode() {
        return;
    }

    let shift = kb.any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight]);
    let alt = kb.any_pressed([KeyCode::AltLeft, KeyCode::AltRight]);
    let ctrl = kb.any_pressed([KeyCode::ControlLeft, KeyCode::ControlRight]);
    let step = if shift { 10.0 } else { 1.0 };
    let scale_step = if shift { 0.05 } else { 0.01 };

    if kb.just_pressed(KeyCode::Tab) {
        let next = if shift {
            session.target().prev()
        } else {
            session.target().next()
        };
        session.set_target(next);
        info!("Layout target: {:?}", session.target());
    }

    let t = layout.tune_mut(session.target());
    let move_left = kb.just_pressed(KeyCode::ArrowLeft) || kb.just_pressed(KeyCode::KeyJ);
    let move_right = kb.just_pressed(KeyCode::ArrowRight) || kb.just_pressed(KeyCode::KeyL);
    let move_up = kb.just_pressed(KeyCode::ArrowUp) || kb.just_pressed(KeyCode::KeyI);
    let move_down = kb.just_pressed(KeyCode::ArrowDown) || kb.just_pressed(KeyCode::KeyK);
    if move_left {
        t.offset_x -= step;
    }
    if move_right {
        t.offset_x += step;
    }
    if move_up {
        t.offset_y -= step;
    }
    if move_down {
        t.offset_y += step;
    }

    if kb.just_pressed(KeyCode::BracketLeft) {
        if alt {
            t.scale_x = (t.scale_x - scale_step).clamp(0.15, 3.0);
        } else if shift {
            t.scale_y = (t.scale_y - scale_step).clamp(0.15, 3.0);
        } else {
            t.scale_x = (t.scale_x - scale_step).clamp(0.15, 3.0);
            t.scale_y = (t.scale_y - scale_step).clamp(0.15, 3.0);
        }
    }
    if kb.just_pressed(KeyCode::BracketRight) {
        if alt {
            t.scale_x = (t.scale_x + scale_step).clamp(0.15, 3.0);
        } else if shift {
            t.scale_y = (t.scale_y + scale_step).clamp(0.15, 3.0);
        } else {
            t.scale_x = (t.scale_x + scale_step).clamp(0.15, 3.0);
            t.scale_y = (t.scale_y + scale_step).clamp(0.15, 3.0);
        }
    }

    let dh_step = if shift { 10.0 } else { 2.0 };
    if kb.just_pressed(KeyCode::Minus) {
        t.size_basis = (t.size_basis - dh_step).clamp(20.0, 640.0);
    }
    if kb.just_pressed(KeyCode::Equal) {
        t.size_basis = (t.size_basis + dh_step).clamp(20.0, 640.0);
    }

    let rot_step = if shift { 5.0 } else { 1.0 };
    if kb.just_pressed(KeyCode::KeyQ) {
        t.rotation_deg -= rot_step;
    }
    if kb.just_pressed(KeyCode::KeyE) {
        t.rotation_deg += rot_step;
    }

    if kb.just_pressed(KeyCode::KeyN) {
        t.global_z = t.global_z.saturating_sub(1);
    }
    if kb.just_pressed(KeyCode::KeyM) {
        t.global_z = t.global_z.saturating_add(1);
    }

    let fine = if shift { 0.05 } else { 0.02 };
    if kb.just_pressed(KeyCode::Digit1) {
        t.exposure = (t.exposure - fine).clamp(0.0, 4.0);
    }
    if kb.just_pressed(KeyCode::Digit2) {
        t.exposure = (t.exposure + fine).clamp(0.0, 4.0);
    }
    if kb.just_pressed(KeyCode::Digit3) {
        t.glow = (t.glow - fine).clamp(0.0, 3.0);
    }
    if kb.just_pressed(KeyCode::Digit4) {
        t.glow = (t.glow + fine).clamp(0.0, 3.0);
    }
    if kb.just_pressed(KeyCode::Digit5) {
        t.bloom = (t.bloom - fine).clamp(0.0, 2.0);
    }
    if kb.just_pressed(KeyCode::Digit6) {
        t.bloom = (t.bloom + fine).clamp(0.0, 2.0);
    }

    if kb.just_pressed(KeyCode::KeyP) {
        match serde_json::to_string_pretty(&*layout) {
            Ok(js) => info!("title_scene layout JSON:\n{}", js),
            Err(e) => warn!("could not print layout: {}", e),
        }
    }

    if ctrl && kb.just_pressed(KeyCode::KeyS) {
        layout.try_save_to_disk();
    }

    if kb.just_pressed(KeyCode::F5) {
        *layout = TitleSceneLayout::try_load_from_disk();
        info!("Reloaded title scene layout from disk.");
    }
}

#[cfg(debug_assertions)]
#[derive(Resource, Default)]
pub struct TitleSceneTuneHintLogged(pub bool);

#[cfg(debug_assertions)]
pub fn log_title_scene_tune_hint_on_first_title_visit(
    mut logged: ResMut<TitleSceneTuneHintLogged>,
) {
    if logged.0 {
        return;
    }
    logged.0 = true;
    info!("{}", TITLE_SCENE_TUNE_HINT);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn title_scene_layout_serde_stable_round_trip() {
        let layout = TitleSceneLayout::default();
        let v = serde_json::to_value(&layout).expect("serialize");
        let back: TitleSceneLayout = serde_json::from_value(v.clone()).expect("deserialize");
        let v2 = serde_json::to_value(&back).expect("re-serialize");
        assert_eq!(v, v2);
    }
}
