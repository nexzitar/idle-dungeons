//! Mouse pick / drag editing for tunable presentation elements (title camp).

use crate::presentation::editor::{
    tune_for_scene_mut, PresentationEditorDragState, PresentationEditorSession,
    DRAG_START_THRESHOLD_PX,
};
use crate::presentation::element::PresentationElementId;
use crate::presentation::markers::PresentationLayerHost;
use crate::ui::components::PresentationElementHost;
use crate::ui::scene_tune::TitleSceneLayout;
use bevy::input::mouse::{AccumulatedMouseMotion, MouseButton};
use bevy::prelude::*;
use bevy::ui::GlobalZIndex;

fn consider_pressed(
    best: &mut Option<(i32, PresentationElementId, Entity)>,
    entity: Entity,
    interaction: &Interaction,
    z: i32,
    id: PresentationElementId,
) {
    if *interaction != Interaction::Pressed {
        return;
    }
    let replace = match best {
        None => true,
        Some((bz, _, be)) => z > *bz || (z == *bz && entity > *be),
    };
    if replace {
        *best = Some((z, id, entity));
    }
}

fn top_pressed_target(
    fire_q: &Query<(
        Entity,
        &Interaction,
        &PresentationLayerHost,
        &GlobalZIndex,
    )>,
    host_q: &Query<(
        Entity,
        &Interaction,
        &PresentationElementHost,
        &GlobalZIndex,
    )>,
) -> Option<PresentationElementId> {
    let mut best: Option<(i32, PresentationElementId, Entity)> = None;
    for (entity, interaction, host, gz) in fire_q.iter() {
        consider_pressed(&mut best, entity, interaction, gz.0, host.0.clone());
    }
    for (entity, interaction, host, gz) in host_q.iter() {
        consider_pressed(&mut best, entity, interaction, gz.0, host.0.clone());
    }
    best.map(|(_, id, _)| id)
}

/// Left-click selects the topmost fire layer or camp element host.
pub fn presentation_editor_pick(
    mouse: Res<ButtonInput<MouseButton>>,
    mut session: ResMut<PresentationEditorSession>,
    mut field_edit: ResMut<crate::presentation::editor::PresentationEditorFieldEditState>,
    fire_q: Query<(
        Entity,
        &Interaction,
        &PresentationLayerHost,
        &GlobalZIndex,
    )>,
    host_q: Query<(
        Entity,
        &Interaction,
        &PresentationElementHost,
        &GlobalZIndex,
    )>,
) {
    if !session.active {
        return;
    }
    if !mouse.just_pressed(MouseButton::Left) {
        return;
    }
    if let Some(id) = top_pressed_target(&fire_q, &host_q) {
        session.selected_element = Some(id);
        field_edit.clear();
    }
}

/// Drag applies to the selected element host or fire sub-layer.
pub fn presentation_editor_drag(
    mouse: Res<ButtonInput<MouseButton>>,
    kb: Res<ButtonInput<KeyCode>>,
    session: Res<PresentationEditorSession>,
    mut layout: ResMut<TitleSceneLayout>,
    mut drag: ResMut<PresentationEditorDragState>,
    accumulated: Res<AccumulatedMouseMotion>,
    fire_q: Query<(
        Entity,
        &Interaction,
        &PresentationLayerHost,
        &GlobalZIndex,
    )>,
    host_q: Query<(
        Entity,
        &Interaction,
        &PresentationElementHost,
        &GlobalZIndex,
    )>,
) {
    let shift = kb.any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight]);

    if !session.active {
        drag.active_host_drag = None;
        drag.pending_delta = Vec2::ZERO;
        drag.dragging = false;
        return;
    }

    if mouse.just_released(MouseButton::Left) {
        drag.active_host_drag = None;
        drag.pending_delta = Vec2::ZERO;
        drag.dragging = false;
    }

    if mouse.just_pressed(MouseButton::Left) {
        drag.active_host_drag = top_pressed_target(&fire_q, &host_q);
        drag.pending_delta = Vec2::ZERO;
        drag.dragging = false;
    }

    let Some(target_id) = drag.active_host_drag.clone() else {
        return;
    };

    if !mouse.pressed(MouseButton::Left) || accumulated.delta == Vec2::ZERO {
        return;
    }

    let mult = if shift { 10.0 } else { 1.0 };

    if !drag.dragging {
        drag.pending_delta += accumulated.delta;
        if drag.pending_delta.length() < DRAG_START_THRESHOLD_PX {
            return;
        }
        drag.dragging = true;
        let dx = drag.pending_delta.x * mult;
        let dy = drag.pending_delta.y * mult;
        drag.pending_delta = Vec2::ZERO;
        apply_drag_delta(&mut layout, target_id.as_str(), dx, dy);
        return;
    }

    let dx = accumulated.delta.x * mult;
    let dy = accumulated.delta.y * mult;

    apply_drag_delta(&mut layout, target_id.as_str(), dx, dy);
}

fn apply_drag_delta(layout: &mut TitleSceneLayout, target_id: &str, dx: f32, dy: f32) {
    if let Some(layer) = layout.layer_tune_mut(target_id) {
        layer.offset_x += dx;
        layer.offset_y += dy;
    } else {
        let tune = tune_for_scene_mut(layout, target_id);
        tune.offset_x += dx;
        tune.offset_y += dy;
    }
}
