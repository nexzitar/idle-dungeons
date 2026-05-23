//! Composite presentation elements: `element_id:layer_key` ids, registry, and layout routing.

use crate::presentation::element::PresentationLayerTune;
use serde::{Deserialize, Serialize};

pub use crate::domain::title_camp::CAMP_FIRE_SEATS;

/// Presentation element id for campfire seat `seat` (1-based: `player1` … `player6`).
#[must_use]
pub fn title_player_element_id(seat_one_based: usize) -> &'static str {
    match seat_one_based {
        1 => "player1",
        2 => "player2",
        3 => "player3",
        4 => "player4",
        5 => "player5",
        6 => "player6",
        _ => "player1",
    }
}

/// Title camp presentation element ids (shared with the editor and spawn code).
pub const TITLE_ELEMENT_FIREPLACE: &str = "fireplace";

/// Legacy ids (normalized on read).
pub const TITLE_ELEMENT_LEAD_SLOT: &str = "lead_slot";
pub const TITLE_ELEMENT_ALLY_SLOT: &str = "ally_slot";

/// Full editor / JSON id: `"fireplace:stack"`, `"player3:emoji"`.
#[must_use]
pub fn compose_layer_id(element_id: &str, layer_key: &str) -> String {
    format!("{element_id}:{layer_key}")
}

/// Split `"element:layer"`; returns `None` for host-only ids (no colon).
#[must_use]
pub fn parse_layer_id(id: &str) -> Option<(&str, &str)> {
    let (element, layer) = id.split_once(':')?;
    if element.is_empty() || layer.is_empty() {
        return None;
    }
    Some((element, layer))
}

/// Maps legacy `fire:*`, `lead_slot`, `ally_slot` to current ids.
#[must_use]
pub fn normalize_layer_id(id: &str) -> String {
    if let Some(rest) = id.strip_prefix("fire:") {
        return compose_layer_id(TITLE_ELEMENT_FIREPLACE, rest);
    }
    if id == TITLE_ELEMENT_LEAD_SLOT {
        return title_player_element_id(1).to_string();
    }
    if id == TITLE_ELEMENT_ALLY_SLOT {
        return title_player_element_id(2).to_string();
    }
    id.to_string()
}

#[must_use]
pub fn is_presentation_layer_id(id: &str) -> bool {
    parse_layer_id(id).is_some()
}

/// Parse `player3` → seat index 0..5 (`None` if not a player seat id).
#[must_use]
pub fn parse_player_element_seat(id: &str) -> Option<usize> {
    let num = id.strip_prefix("player")?.parse::<usize>().ok()?;
    if (1..=CAMP_FIRE_SEATS).contains(&num) {
        Some(num - 1)
    } else {
        None
    }
}

/// One selectable sub-layer row in the presentation editor hierarchy.
pub struct PresentationLayerRow {
    /// Full editor id (`fireplace:stack`, `player3:emoji`, …).
    pub id: &'static str,
    pub label: &'static str,
}

/// Host element plus optional child layers (data-driven editor tree).
pub struct PresentationElementLayers {
    pub element_id: &'static str,
    pub host_label: &'static str,
    pub layers: &'static [PresentationLayerRow],
}

/// Static registry for editor (six seats + fireplace).
pub const TITLE_CAMP_LAYER_REGISTRY: &[PresentationElementLayers] = &[
    PresentationElementLayers {
        element_id: TITLE_ELEMENT_FIREPLACE,
        host_label: "Fireplace (host)",
        layers: &[
            PresentationLayerRow {
                id: "fireplace:stack",
                label: "Stack",
            },
            PresentationLayerRow {
                id: "fireplace:base",
                label: "Base",
            },
            PresentationLayerRow {
                id: "fireplace:flame",
                label: "Flame",
            },
            PresentationLayerRow {
                id: "fireplace:glow",
                label: "Glow",
            },
            PresentationLayerRow {
                id: "fireplace:ground",
                label: "Ground",
            },
        ],
    },
    PresentationElementLayers {
        element_id: "player1",
        host_label: "Player 1 seat",
        layers: &[PresentationLayerRow {
            id: "player1:emoji",
            label: "Emoji",
        }],
    },
    PresentationElementLayers {
        element_id: "player2",
        host_label: "Player 2 seat",
        layers: &[PresentationLayerRow {
            id: "player2:emoji",
            label: "Emoji",
        }],
    },
    PresentationElementLayers {
        element_id: "player3",
        host_label: "Player 3 seat",
        layers: &[PresentationLayerRow {
            id: "player3:emoji",
            label: "Emoji",
        }],
    },
    PresentationElementLayers {
        element_id: "player4",
        host_label: "Player 4 seat",
        layers: &[PresentationLayerRow {
            id: "player4:emoji",
            label: "Emoji",
        }],
    },
    PresentationElementLayers {
        element_id: "player5",
        host_label: "Player 5 seat",
        layers: &[PresentationLayerRow {
            id: "player5:emoji",
            label: "Emoji",
        }],
    },
    PresentationElementLayers {
        element_id: "player6",
        host_label: "Player 6 seat",
        layers: &[PresentationLayerRow {
            id: "player6:emoji",
            label: "Emoji",
        }],
    },
];

/// Per-seat sub-layer tunes (emoji offset under the seat host).
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct FigureSlotLayerTunes {
    pub emoji: PresentationLayerTune,
}

/// Figure seat layers persisted in `title_scene.json` (`player1` … `player6` seats).
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct TitleCampFigureLayerTunes {
    #[serde(default = "default_figure_layer_players")]
    pub players: [FigureSlotLayerTunes; CAMP_FIRE_SEATS],
}

fn default_figure_layer_players() -> [FigureSlotLayerTunes; CAMP_FIRE_SEATS] {
    std::array::from_fn(|_| FigureSlotLayerTunes::default())
}

/// Optional layers for assets not yet split into typed structs (`"element:layer"` → tune).
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct TitleCampExtraLayerTunes(pub std::collections::HashMap<String, PresentationLayerTune>);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_legacy_fire_prefix() {
        assert_eq!(
            normalize_layer_id("fire:glow"),
            compose_layer_id(TITLE_ELEMENT_FIREPLACE, "glow")
        );
    }

    #[test]
    fn normalize_legacy_lead_ally_slots() {
        assert_eq!(normalize_layer_id("lead_slot"), "player1");
        assert_eq!(normalize_layer_id("ally_slot"), "player2");
    }

    #[test]
    fn parse_player_element_seat_indices() {
        assert_eq!(super::parse_player_element_seat("player3"), Some(2));
        assert_eq!(super::parse_player_element_seat("fireplace"), None);
    }
}
