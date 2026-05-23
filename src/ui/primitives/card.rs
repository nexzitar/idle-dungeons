//! Stash and loot item row cards.

use bevy::prelude::*;

use crate::domain::items::ItemInstance;
use crate::ui::assets::UiPlaceholderImages;
use crate::ui::components::{EquipItemButton, SalvageItemButton};
use crate::ui::inspect::{InspectHint, StashItemInspect};
use crate::ui::interaction::UiClickAction;
use crate::ui::primitives::button::{spawn_button, UiButtonConfig, UiButtonVariant};
use crate::ui::primitives::icon_frame::{rarity_frame_border, spawn_rarity_icon_frame, RarityIconFrameConfig};
use crate::ui::theme::{caption_text, rarity_color, UiDensity, UiPanelStyle, UiTheme};

const ITEM_ACTION_W: f32 = 88.0;
const ITEM_ACTION_H: f32 = 32.0;

pub fn spawn_item_card(
    parent: &mut ChildSpawnerCommands<'_>,
    item: &ItemInstance,
    ph: &UiPlaceholderImages,
) {
    let style = UiPanelStyle::deep_card();
    let border = rarity_frame_border(item.rarity);
    parent
        .spawn((
            style.inset_column_node(),
            BackgroundColor(style.background),
            BorderColor::from(border),
            Interaction::default(),
            StashItemInspect { item_id: item.id },
        ))
        .with_children(|card| {
            spawn_item_card_header(card, item, ph, UiDensity::Gear);
            card.spawn(Node {
                box_sizing: BoxSizing::BorderBox,
                flex_direction: FlexDirection::Row,
                column_gap: Val::Px(8.0),
                align_items: AlignItems::Center,
                ..default()
            })
            .with_children(|row| {
                spawn_item_action_button(
                    row,
                    "Equip",
                    UiButtonVariant::Equip,
                    Color::WHITE,
                    EquipItemButton { item_id: item.id },
                    UiClickAction::EquipItem,
                    "Equip this item on your hero. It replaces whatever is currently in this gear slot.",
                );
                spawn_item_action_button(
                    row,
                    "Salvage",
                    UiButtonVariant::Salvage,
                    UiTheme::body(),
                    SalvageItemButton { item_id: item.id },
                    UiClickAction::SalvageItem,
                    "Salvage this item for currency. The item is removed from your stash permanently.",
                );
            });
        });
}

/// Read-only stash-style card for run rewards (loot not yet in profile).
pub fn spawn_item_card_preview(
    parent: &mut ChildSpawnerCommands<'_>,
    item: &ItemInstance,
    ph: &UiPlaceholderImages,
) {
    let style = UiPanelStyle::deep_card();
    let border = rarity_frame_border(item.rarity);
    parent
        .spawn((
            style.inset_column_node(),
            BackgroundColor(style.background),
            BorderColor::from(border),
        ))
        .with_children(|card| {
            spawn_item_card_header(card, item, ph, UiDensity::Summary);
            card.spawn(caption_text(
                "Added to stash when you Accept rewards.".to_string(),
            ));
        });
}

fn spawn_item_card_header(
    card: &mut ChildSpawnerCommands<'_>,
    item: &ItemInstance,
    ph: &UiPlaceholderImages,
    density: UiDensity,
) {
    let icon_px = density.icon_bar_px();
    card.spawn(Node {
        box_sizing: BoxSizing::BorderBox,
        width: Val::Percent(100.0),
        flex_direction: FlexDirection::Row,
        column_gap: Val::Px(10.0),
        align_items: AlignItems::Center,
        ..default()
    })
    .with_children(|head| {
        spawn_rarity_icon_frame(
            head,
            RarityIconFrameConfig {
                image: ph.item_generic.clone(),
                rarity: item.rarity,
                outer_px: icon_px,
            },
        );
        head.spawn(Node {
            box_sizing: BoxSizing::BorderBox,
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::FlexStart,
            row_gap: Val::Px(2.0),
            flex_grow: 1.0,
            min_width: Val::Px(0.0),
            ..default()
        })
        .with_children(|txt| {
            txt.spawn((
                Text::new(item.name.clone()),
                TextFont::from_font_size(UiTheme::FONT_SECTION),
                TextColor(rarity_color(item.rarity)),
            ));
            txt.spawn(caption_text(format!(
                "{:?} · {}",
                item.rarity,
                item.slot.display_label()
            )));
        });
    });
}

fn spawn_item_action_button<M: Component>(
    row: &mut ChildSpawnerCommands<'_>,
    label: &'static str,
    variant: UiButtonVariant,
    text_color: Color,
    marker: M,
    action: UiClickAction,
    tip: &'static str,
) {
    let entity = spawn_button(
        row,
        UiButtonConfig {
            label,
            variant,
            width: Val::Px(ITEM_ACTION_W),
            height: Val::Px(ITEM_ACTION_H),
            font_size: UiTheme::FONT_COMPACT,
            text_color,
            flex_shrink: 0.0,
        },
    );
    row.commands_mut()
        .entity(entity)
        .insert((marker, action, InspectHint(tip)));
}
