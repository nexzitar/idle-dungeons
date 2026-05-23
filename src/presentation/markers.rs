//! Presentation-side markers that **must not** pull in `crate::ui` (keeps module layering clean).

use crate::presentation::element::PresentationElementId;
use bevy::prelude::*;

/// Tunable root for the title campfire presentation stack (sync reads layout JSON).
#[derive(Component)]
pub struct CampfirePresentationRoot;

/// Hit target for a sub-layer of a composite presentation element (`element:layer` id).
#[derive(Component)]
pub struct PresentationLayerHost(pub PresentationElementId);

/// Legacy alias for [`PresentationLayerHost`].
pub type PresentationFireLayerHost = PresentationLayerHost;
