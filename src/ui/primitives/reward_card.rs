//! Celebratory loot tile — library-tier icon, rarity glow, optional NEW badge.

use bevy::prelude::*;
use bevy::text::Justify;

use crate::domain::items::{ItemInstance, ItemRarity};
use crate::ui::assets::UiPlaceholderImages;
use crate::ui::primitives::icon_frame::{rarity_frame_border, spawn_rarity_icon_frame, RarityIconFrameConfig};
use crate::ui::theme::{caption_text, rarity_color, UiDensity, UiTheme};

const REWARD_CARD_W: f32 = 152.0;

#[derive(Clone, Copy, Debug)]
pub struct RewardCardConfig {
    pub show_new_badge: bool,
}

impl Default for RewardCardConfig {
    fn default() -> Self {
        Self {
            show_new_badge: true,
        }
    }
}

/// Single loot tile for summary grids and the rewards modal.
pub fn spawn_reward_card(
    parent: &mut ChildSpawnerCommands<'_>,
    item: &ItemInstance,
    config: RewardCardConfig,
    ph: &UiPlaceholderImages,
) {
    let border = rarity_frame_border(item.rarity);
    let glow = rarity_glow_fill(item.rarity);
    let icon_px = UiDensity::Summary.icon_library_px();

    parent
        .spawn((
            Node {
                box_sizing: BoxSizing::BorderBox,
                width: Val::Px(REWARD_CARD_W),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                row_gap: Val::Px(6.0),
                padding: UiRect::all(Val::Px(UiTheme::PANEL_INSET_SM)),
                border: UiRect::all(Val::Px(1.0)),
                position_type: PositionType::Relative,
                ..default()
            },
            BackgroundColor(UiTheme::panel_bg_deep()),
            BorderColor::from(border),
        ))
        .with_children(|card| {
            if config.show_new_badge {
                card.spawn((
                    Node {
                        box_sizing: BoxSizing::BorderBox,
                        position_type: PositionType::Absolute,
                        right: Val::Px(4.0),
                        top: Val::Px(2.0),
                        padding: UiRect::axes(Val::Px(5.0), Val::Px(2.0)),
                        border: UiRect::all(Val::Px(1.0)),
                        ..default()
                    },
                    BackgroundColor(UiTheme::treasure().mix(&Color::BLACK, 0.55)),
                    BorderColor::from(UiTheme::treasure().mix(&Color::BLACK, 0.35)),
                ))
                .with_children(|badge| {
                    badge.spawn((
                        Text::new("NEW"),
                        TextFont::from_font_size(UiTheme::FONT_MICRO),
                        TextColor(UiTheme::muted_gold()),
                    ));
                });
            }
            card.spawn((
                Node {
                    box_sizing: BoxSizing::BorderBox,
                    padding: UiRect::all(Val::Px(6.0)),
                    border: UiRect::all(Val::Px(1.0)),
                    ..default()
                },
                BackgroundColor(glow),
                BorderColor::from(border.mix(&Color::WHITE, 0.12)),
            ))
            .with_children(|glow| {
                spawn_rarity_icon_frame(
                    glow,
                    RarityIconFrameConfig {
                        image: ph.item_generic.clone(),
                        rarity: item.rarity,
                        outer_px: icon_px,
                    },
                );
            });
            card.spawn((
                Text::new(item.name.clone()),
                TextFont::from_font_size(UiTheme::FONT_COMPACT),
                TextColor(rarity_color(item.rarity)),
                TextLayout::new_with_justify(Justify::Center),
            ));
            card.spawn(caption_text(format!(
                "{:?} · {}",
                item.rarity,
                item.slot.display_label()
            )));
        });
}

/// Flex-wrapped grid of reward tiles (summary column + rewards modal).
pub fn spawn_reward_loot_grid(
    parent: &mut ChildSpawnerCommands<'_>,
    items: &[ItemInstance],
    ph: &UiPlaceholderImages,
    max_visible: usize,
) {
    if items.is_empty() {
        parent.spawn(caption_text("No gear dropped this run."));
        return;
    }
    let visible = items.len().min(max_visible);
    parent
        .spawn(Node {
            box_sizing: BoxSizing::BorderBox,
            width: Val::Percent(100.0),
            flex_direction: FlexDirection::Row,
            flex_wrap: FlexWrap::Wrap,
            column_gap: Val::Px(10.0),
            row_gap: Val::Px(10.0),
            justify_content: JustifyContent::FlexStart,
            ..default()
        })
        .with_children(|grid| {
            for item in items.iter().take(visible) {
                spawn_reward_card(grid, item, RewardCardConfig::default(), ph);
            }
        });
    if items.len() > visible {
        parent.spawn(caption_text(format!(
            "\u{2026} and {} more in the rewards popup.",
            items.len() - visible
        )));
    }
}

#[must_use]
fn rarity_glow_fill(rarity: ItemRarity) -> Color {
    rarity_color(rarity).mix(&Color::BLACK, 0.82).with_alpha(0.55)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn legendary_glow_is_brighter_than_common() {
        let common = rarity_glow_fill(ItemRarity::Common);
        let legendary = rarity_glow_fill(ItemRarity::Legendary);
        assert!(legendary.to_srgba().alpha > common.to_srgba().alpha
            || legendary.to_srgba().red + legendary.to_srgba().green
                > common.to_srgba().red + common.to_srgba().green);
    }
}
