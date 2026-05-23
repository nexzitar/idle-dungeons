//! Stash and loot item row cards.

use bevy::prelude::*;

use crate::domain::items::ItemInstance;
use crate::ui::assets::UiPlaceholderImages;
use crate::ui::components::{EquipItemButton, SalvageItemButton};
use crate::ui::inspect::{InspectHint, StashItemInspect};
use crate::ui::interaction::UiClickAction;
use crate::ui::primitives::button::{spawn_button, UiButtonConfig, UiButtonVariant};
use crate::ui::theme::{
    body_text, caption_text, format_item_affix_lines, format_item_stat_summary, rarity_color,
    UiPanelStyle, UiTheme,
};

pub fn spawn_item_card(
    parent: &mut ChildSpawnerCommands<'_>,
    item: &ItemInstance,
    ph: &UiPlaceholderImages,
) {
    let style = UiPanelStyle::deep_card();
    let border = rarity_color(item.rarity).mix(&Color::BLACK, 0.45);
    parent
        .spawn((
            style.inset_column_node(),
            BackgroundColor(style.background),
            BorderColor::from(border),
            Interaction::default(),
            StashItemInspect { item_id: item.id },
        ))
        .with_children(|card| {
            spawn_item_card_header(card, item, ph);
            card.spawn(Node {
                box_sizing: BoxSizing::BorderBox,
                flex_direction: FlexDirection::Row,
                column_gap: Val::Px(10.0),
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
    let border = rarity_color(item.rarity).mix(&Color::BLACK, 0.45);
    parent
        .spawn((
            style.inset_column_node(),
            BackgroundColor(style.background),
            BorderColor::from(border),
        ))
        .with_children(|card| {
            spawn_item_card_header(card, item, ph);
            card.spawn(caption_text(
                "Added to stash when you Accept rewards.".to_string(),
            ));
        });
}

fn spawn_item_card_header(
    card: &mut ChildSpawnerCommands<'_>,
    item: &ItemInstance,
    ph: &UiPlaceholderImages,
) {
    card.spawn(Node {
        box_sizing: BoxSizing::BorderBox,
        width: Val::Percent(100.0),
        flex_direction: FlexDirection::Row,
        column_gap: Val::Px(10.0),
        align_items: AlignItems::FlexStart,
        ..default()
    })
    .with_children(|head| {
        head.spawn((
            Node {
                box_sizing: BoxSizing::BorderBox,
                width: Val::Px(40.0),
                height: Val::Px(40.0),
                flex_shrink: 0.0,
                ..default()
            },
            ImageNode {
                image: ph.item_generic.clone(),
                color: rarity_color(item.rarity).mix(&Color::WHITE, 0.35),
                ..default()
            },
        ));
        head.spawn(Node {
            box_sizing: BoxSizing::BorderBox,
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::FlexStart,
            row_gap: Val::Px(4.0),
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
            txt.spawn(caption_text(format!("{:?} · {:?}", item.rarity, item.slot)));
            txt.spawn(body_text(format_item_stat_summary(item)));
            let aff = format_item_affix_lines(item);
            if !aff.is_empty() {
                txt.spawn(caption_text(aff));
            }
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
            width: Val::Px(108.0),
            height: Val::Px(36.0),
            font_size: UiTheme::FONT_BODY,
            text_color,
            flex_shrink: 0.0,
        },
    );
    row.commands_mut()
        .entity(entity)
        .insert((marker, action, InspectHint(tip)));
}
