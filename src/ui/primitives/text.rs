//! Re-exports text style bundles from [`crate::ui::theme`].

use bevy::prelude::*;

pub use crate::ui::theme::{body_text, caption_text, section_title};

/// Section heading plus a compact spacer for consistent vertical rhythm.
pub fn spawn_section_header(parent: &mut ChildSpawnerCommands<'_>, title: &str) {
    parent.spawn(section_title(title));
    parent.spawn(Node {
        height: Val::Px(6.0),
        flex_shrink: 0.0,
        flex_grow: 0.0,
        ..default()
    });
}
