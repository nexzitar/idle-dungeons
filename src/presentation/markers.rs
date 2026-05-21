//! Presentation-side markers that **must not** pull in `crate::ui` (keeps module layering clean).

use crate::presentation::element::PresentationElementId;
use bevy::prelude::*;

/// Tunable root for the title campfire presentation stack (sync reads layout JSON).
#[derive(Component)]
pub struct CampfirePresentationRoot;

/// Hit target for an individual fireplace presentation layer (glow, base, flame, …).
#[derive(Component)]
pub struct PresentationFireLayerHost(pub PresentationElementId);
