//! Shared icon cell framing — rarity-tinted borders for gear and loot tiles.

use bevy::prelude::*;

use crate::domain::items::ItemRarity;
use crate::ui::theme::{rarity_color, UiTheme};

/// Rarity-tinted border for item surfaces (design system §9 — items use rarity mix).
#[must_use]
pub fn rarity_frame_border(rarity: ItemRarity) -> Color {
    rarity_color(rarity).mix(&Color::BLACK, 0.45)
}

#[derive(Clone, Debug)]
pub struct RarityIconFrameConfig {
    pub image: Handle<Image>,
    pub rarity: ItemRarity,
    pub outer_px: f32,
}

/// Icon-first cell: recessed backing + rarity border + tinted art.
pub fn spawn_rarity_icon_frame(parent: &mut ChildSpawnerCommands<'_>, config: RarityIconFrameConfig) {
    let border = rarity_frame_border(config.rarity);
    let icon_tint = rarity_color(config.rarity).mix(&Color::WHITE, 0.35);
    let inset = 5.0;
    let icon_px = (config.outer_px - inset * 2.0).max(16.0);

    parent
        .spawn((
            Node {
                box_sizing: BoxSizing::BorderBox,
                width: Val::Px(config.outer_px),
                height: Val::Px(config.outer_px),
                flex_shrink: 0.0,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                border: UiRect::all(Val::Px(1.0)),
                ..default()
            },
            BackgroundColor(UiTheme::panel_bg_deep()),
            BorderColor::from(border),
        ))
        .with_children(|frame| {
            frame.spawn((
                Node {
                    box_sizing: BoxSizing::BorderBox,
                    width: Val::Px(icon_px),
                    height: Val::Px(icon_px),
                    flex_shrink: 0.0,
                    ..default()
                },
                ImageNode {
                    image: config.image,
                    color: icon_tint,
                    ..default()
                },
            ));
        });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::items::ItemRarity;

    #[test]
    fn rarity_frame_border_differs_by_tier() {
        let common = rarity_frame_border(ItemRarity::Common);
        let legendary = rarity_frame_border(ItemRarity::Legendary);
        assert_ne!(common, legendary);
    }
}
