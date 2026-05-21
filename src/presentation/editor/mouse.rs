//! Mouse pick / drag editing for tunable presentation elements (title camp).

use crate::presentation::editor::{
    tune_for_scene_mut, PresentationEditorDragState, PresentationEditorSession,
};
use crate::presentation::element::PresentationElementId;
use crate::ui::components::PresentationElementHost;
use crate::ui::scene_tune::TitleSceneLayout;
use bevy::input::mouse::{AccumulatedMouseMotion, MouseButton};
use bevy::prelude::*;
use bevy::ui::GlobalZIndex;

fn top_pressed_presentation_host(
    q: &Query<(
        Entity,
        &Interaction,
        &PresentationElementHost,
        &GlobalZIndex,
    )>,
) -> Option<PresentationElementId> {
    let mut best: Option<(i32, PresentationElementId, Entity)> = None;
    for (entity, interaction, host, gz) in q.iter() {
        if *interaction != Interaction::Pressed {
            continue;
        }
        let z = gz.0;
        let replace = match &best {
            None => true,
            Some((bz, _, be)) => z > *bz || (z == *bz && entity > *be),
        };
        if replace {
            best = Some((z, host.0.clone(), entity));
        }
    }
    best.map(|(_, id, _)| id)
}

/// Left-click selects the topmost hovered presentation element (when the editor session is active).
pub fn presentation_editor_pick(
    mouse: Res<ButtonInput<MouseButton>>,
    mut session: ResMut<PresentationEditorSession>,
    q: Query<(
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
    if let Some(id) = top_pressed_presentation_host(&q) {
        session.selected_element = Some(id);
    }
}

/// While holding the button after pressing a host, apply [`AccumulatedMouseMotion`] deltas to layout offsets (shift ×10).
pub fn presentation_editor_drag(
    mouse: Res<ButtonInput<MouseButton>>,
    kb: Res<ButtonInput<KeyCode>>,
    session: Res<PresentationEditorSession>,
    mut layout: ResMut<TitleSceneLayout>,
    mut drag: ResMut<PresentationEditorDragState>,
    accumulated: Res<AccumulatedMouseMotion>,
    q: Query<(
        Entity,
        &Interaction,
        &PresentationElementHost,
        &GlobalZIndex,
    )>,
) {
    let shift = kb.any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight]);

    if !session.active {
        drag.active_host_drag = None;
        return;
    }

    if mouse.just_released(MouseButton::Left) {
        drag.active_host_drag = None;
    }

    if mouse.just_pressed(MouseButton::Left) {
        drag.active_host_drag = top_pressed_presentation_host(&q);
    }

    let Some(host_id) = drag.active_host_drag.clone() else {
        return;
    };

    if !mouse.pressed(MouseButton::Left) || accumulated.delta == Vec2::ZERO {
        return;
    }

    let mult = if shift { 10.0 } else { 1.0 };
    let tune = tune_for_scene_mut(&mut layout, host_id.as_str());
    tune.offset_x += accumulated.delta.x * mult;
    tune.offset_y += accumulated.delta.y * mult;
}
