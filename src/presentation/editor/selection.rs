//! Hover feedback for tunable hosts while the presentation editor is active.

use crate::presentation::editor::PresentationEditorSession;
use crate::ui::components::PresentationElementHost;
use bevy::ecs::system::ParamSet;
use bevy::prelude::*;

/// Light outline on hovered (non-selected) presentation hosts during editor mode.
///
/// Runs after [`crate::ui::scene_tune::title_scene_tune_selection_gizmo`], which resets borders and paints the primary selection outline.
pub fn presentation_editor_hover_outline(
    session: Res<PresentationEditorSession>,
    mut set: ParamSet<(
        Query<(
            Entity,
            &PresentationElementHost,
            &Interaction,
            &GlobalZIndex,
        )>,
        Query<(&mut BorderColor, &mut Node), With<PresentationElementHost>>,
    )>,
) {
    if !session.active || !session.gizmo_flags.selection_outline {
        return;
    }

    let sel = session.selected_element.as_deref();

    let hovered_entity = {
        let hints = set.p0();
        let mut best_hover: Option<(i32, Entity)> = None;
        for (entity, host, interaction, gz) in hints.iter() {
            if *interaction != Interaction::Hovered {
                continue;
            }
            if sel.is_some_and(|s| s == host.0.as_str()) {
                continue;
            }
            let z = gz.0;
            let replace = best_hover
                .as_ref()
                .map(|&(bz, be)| z > bz || (z == bz && entity > be))
                .unwrap_or(true);
            if replace {
                best_hover = Some((z, entity));
            }
        }
        best_hover.map(|(_, e)| e)
    };

    let Some(hovered_entity) = hovered_entity else {
        return;
    };

    let hover_col = BorderColor::all(Color::srgba(1.0, 1.0, 1.0, 0.35));

    let mut paint = set.p1();
    if let Ok((mut border, mut node)) = paint.get_mut(hovered_entity) {
        *border = hover_col;
        node.border = UiRect::all(Val::Px(2.0));
    }
}
