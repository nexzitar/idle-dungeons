//! Presentation-side markers that **must not** pull in `crate::ui` (keeps module layering clean).

use bevy::prelude::*;

/// Tunable root for the title campfire presentation stack (sync reads layout JSON).
#[derive(Component)]
pub struct CampfirePresentationRoot;
