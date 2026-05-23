//! Horizontal fill bars (playback HP-style tracks).

use bevy::prelude::*;

use crate::ui::theme::UiTheme;

#[derive(Clone, Copy, Debug)]
pub struct UiBarStyle {
    pub track: Color,
    pub fill: Color,
    pub height_px: f32,
    /// When true, a 1 px panel-border outline like playback HP bars.
    pub border: bool,
}

pub fn spawn_horizontal_bar<M: Component>(
    parent: &mut ChildSpawnerCommands<'_>,
    style: UiBarStyle,
    fill_marker: M,
    initial_fill_pct: f32,
) -> Entity {
    let pct = (initial_fill_pct * 100.0).clamp(0.0, 100.0);
    let mut track_node = Node {
        box_sizing: BoxSizing::BorderBox,
        width: Val::Percent(100.0),
        height: Val::Px(style.height_px),
        ..default()
    };
    if style.border {
        track_node.border = UiRect::all(Val::Px(1.0));
    }

    let mut cmds = parent.spawn((track_node, BackgroundColor(style.track.into())));
    if style.border {
        cmds.insert(BorderColor::from(UiTheme::panel_border()));
    }

    cmds.with_children(|bar| {
        bar.spawn((
            Node {
                box_sizing: BoxSizing::BorderBox,
                width: Val::Percent(pct),
                height: Val::Percent(100.0),
                ..default()
            },
            BackgroundColor(style.fill.into()),
            fill_marker,
        ));
    })
    .id()
}
