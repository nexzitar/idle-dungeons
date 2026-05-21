//! Click-to-type numeric editing for presentation inspector fields.

use crate::presentation::editor::{
    banner_selected_id, format_editor_tune_field, tune_for_scene_mut, PresentationEditorTuneField,
};
use crate::presentation::element::{PresentationElementTune, PresentationLayerTune};
use crate::presentation::fire::is_fire_layer_id;
use crate::presentation::PresentationEditorSession;
use crate::ui::scene_tune::TitleSceneLayout;
use bevy::input::keyboard::KeyboardInput;
use bevy::input::{ButtonState, ButtonInput};
use bevy::prelude::*;

/// Active inspector field being typed (Enter applies, Esc cancels).
#[derive(Resource, Default)]
pub struct PresentationEditorFieldEditState {
    pub field: Option<PresentationEditorTuneField>,
    pub buffer: String,
}

impl PresentationEditorFieldEditState {
    pub fn clear(&mut self) {
        self.field = None;
        self.buffer.clear();
    }

    pub fn begin(
        &mut self,
        field: PresentationEditorTuneField,
        layout: &TitleSceneLayout,
        selected_id: &str,
    ) {
        self.field = Some(field);
        self.buffer = format_editor_tune_field(layout, selected_id, field);
    }

    pub fn is_editing(&self) -> bool {
        self.field.is_some()
    }
}

fn apply_buffer_to_element(
    tune: &mut PresentationElementTune,
    field: PresentationEditorTuneField,
    buf: &str,
) {
    let trimmed = buf.trim();
    if trimmed.is_empty() {
        return;
    }
    match field {
        PresentationEditorTuneField::OffsetX | PresentationEditorTuneField::OffsetY
        | PresentationEditorTuneField::RotationDeg | PresentationEditorTuneField::SizeBasis => {
            if let Ok(v) = trimmed.parse::<f32>() {
                match field {
                    PresentationEditorTuneField::OffsetX => tune.offset_x = v,
                    PresentationEditorTuneField::OffsetY => tune.offset_y = v,
                    PresentationEditorTuneField::RotationDeg => tune.rotation_deg = v,
                    PresentationEditorTuneField::SizeBasis => {
                        tune.size_basis = v.clamp(20.0, 640.0);
                    }
                    _ => {}
                }
            }
        }
        PresentationEditorTuneField::ScaleX | PresentationEditorTuneField::ScaleY
        | PresentationEditorTuneField::Exposure | PresentationEditorTuneField::Glow
        | PresentationEditorTuneField::Bloom => {
            if let Ok(v) = trimmed.parse::<f32>() {
                match field {
                    PresentationEditorTuneField::ScaleX => {
                        tune.scale_x = v.clamp(0.15, 3.0);
                    }
                    PresentationEditorTuneField::ScaleY => {
                        tune.scale_y = v.clamp(0.15, 3.0);
                    }
                    PresentationEditorTuneField::Exposure => {
                        tune.exposure = v.clamp(0.0, 4.0);
                    }
                    PresentationEditorTuneField::Glow => tune.glow = v.clamp(0.0, 3.0),
                    PresentationEditorTuneField::Bloom => tune.bloom = v.clamp(0.0, 2.0),
                    _ => {}
                }
            }
        }
        PresentationEditorTuneField::GlobalZ => {
            if let Ok(v) = trimmed.parse::<i32>() {
                tune.global_z = v;
            }
        }
    }
}

fn apply_buffer_to_layer(
    tune: &mut PresentationLayerTune,
    field: PresentationEditorTuneField,
    buf: &str,
) {
    let trimmed = buf.trim();
    if trimmed.is_empty() {
        return;
    }
    if let Ok(v) = trimmed.parse::<f32>() {
        match field {
            PresentationEditorTuneField::OffsetX => tune.offset_x = v,
            PresentationEditorTuneField::OffsetY => tune.offset_y = v,
            PresentationEditorTuneField::ScaleX => tune.scale_x = v.clamp(0.05, 4.0),
            PresentationEditorTuneField::ScaleY => tune.scale_y = v.clamp(0.05, 4.0),
            _ => {}
        }
    }
}

/// Type into the focused inspector value; runs while presentation mode is on.
#[cfg(debug_assertions)]
pub fn presentation_editor_tune_field_keyboard(
    session: Res<PresentationEditorSession>,
    mut edit: ResMut<PresentationEditorFieldEditState>,
    mut layout: ResMut<TitleSceneLayout>,
    keys: Res<ButtonInput<KeyCode>>,
    mut kb: MessageReader<KeyboardInput>,
) {
    if !session.active {
        if edit.is_editing() {
            edit.clear();
        }
        for _ in kb.read() {}
        return;
    }

    let Some(field) = edit.field else {
        for _ in kb.read() {}
        return;
    };

    if keys.just_pressed(KeyCode::Escape) {
        edit.clear();
        for _ in kb.read() {}
        return;
    }

    if keys.just_pressed(KeyCode::Enter) {
        let sel = banner_selected_id(&session);
        if is_fire_layer_id(sel) {
            if let Some(layer) = layout.fire_presentation.layers.get_mut(sel) {
                apply_buffer_to_layer(layer, field, &edit.buffer);
            }
        } else {
            let tune = tune_for_scene_mut(&mut layout, sel);
            apply_buffer_to_element(tune, field, &edit.buffer);
        }
        edit.clear();
        for _ in kb.read() {}
        return;
    }

    if keys.just_pressed(KeyCode::Backspace) {
        edit.buffer.pop();
    }

    for ev in kb.read() {
        if ev.state != ButtonState::Pressed || ev.repeat {
            continue;
        }
        let Some(t) = ev.text.as_ref() else {
            continue;
        };
        for ch in t.chars() {
            if field == PresentationEditorTuneField::GlobalZ {
                if ch.is_ascii_digit() || (ch == '-' && edit.buffer.is_empty()) {
                    edit.buffer.push(ch);
                }
                continue;
            }
            if ch.is_ascii_digit() || ch == '.' || ch == '-' {
                if ch == '.' && edit.buffer.contains('.') {
                    continue;
                }
                if ch == '-' && !edit.buffer.is_empty() {
                    continue;
                }
                edit.buffer.push(ch);
            }
        }
    }
}
