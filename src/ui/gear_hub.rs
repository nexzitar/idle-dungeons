//! Gear hub modal: paper doll + stash (sort, equip, salvage). Closes only via the close control.

use bevy::prelude::*;
use bevy::ui::{FocusPolicy, RelativeCursorPosition};
use bevy::text::{TextColor, TextFont};

use crate::save::StashSortOrder;
use crate::ui::components::{
    GearHubBackdrop, GearHubCloseButton, GearHubRoot, UiButtonPalette, UiScrollContent,
    UiScrollRegion, UiScrollState, UiTooltip,
};
use crate::ui::mockup_layout::spawn_stash_filters_and_sort_row;
use crate::ui::placeholder_graphics::UiPlaceholderImages;
use crate::ui::theme::{caption_text, headline_text, section_title, UiTheme};

pub fn spawn_gear_hub_modal(
    parent: &mut ChildSpawnerCommands<'_>,
    profile: &crate::app::ProfileState,
    inventory: &[crate::domain::items::ItemInstance],
    summary_loot: Option<&[crate::domain::items::ItemInstance]>,
    interactive_inventory: bool,
    ph: &UiPlaceholderImages,
) {
    let stash_sort = profile.profile.stash_sort;
    let use_rows: &[crate::domain::items::ItemInstance] = summary_loot.unwrap_or(inventory);

    parent
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                position_type: PositionType::Absolute,
                left: Val::Px(0.0),
                top: Val::Px(0.0),
                ..default()
            },
            GearHubRoot,
        ))
        .insert(FocusPolicy::Block)
        .with_children(|layer| {
            let backdrop_pal = UiButtonPalette {
                idle_bg: Color::srgba(0.02, 0.02, 0.04, 0.58),
                hover_bg: Color::srgba(0.04, 0.04, 0.06, 0.65),
                pressed_bg: Color::srgba(0.06, 0.06, 0.08, 0.72),
                idle_border: Color::NONE,
                hover_border: Color::NONE,
                pressed_border: Color::NONE,
            };
            layer.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    left: Val::Px(0.0),
                    top: Val::Px(0.0),
                    ..default()
                },
                Button,
                BackgroundColor(backdrop_pal.idle_bg),
                BorderColor::from(backdrop_pal.idle_border),
                GearHubBackdrop,
                backdrop_pal,
                UiTooltip::txt("Click outside to close the gear hub."),
            ));
            layer
                .spawn((
                    Node {
                        position_type: PositionType::Absolute,
                        left: Val::Percent(50.0),
                        top: Val::Percent(44.0),
                        margin: UiRect {
                            left: Val::Px(-280.0),
                            top: Val::Px(-198.0),
                            right: Val::Auto,
                            bottom: Val::Auto,
                        },
                        width: Val::Px(560.0),
                        max_height: Val::Px(540.0),
                        min_height: Val::Px(0.0),
                        padding: UiRect::all(Val::Px(UiTheme::PAD_ROOT)),
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Stretch,
                        align_self: AlignSelf::Center,
                        row_gap: Val::Px(UiTheme::PANEL_INSET),
                        flex_shrink: 1.0,
                        overflow: Overflow::clip_y(),
                        border: UiRect::all(Val::Px(2.0)),
                        ..default()
                    },
                    BackgroundColor(UiTheme::panel_bg_deep()),
                    BorderColor::from(UiTheme::ornate_gold()),
                ))
                .with_children(|dialog| {
                    dialog.spawn(headline_text("Gear"));
                    dialog.spawn(section_title("LOADOUT"));
                    dialog.spawn(Node {
                        width: Val::Percent(100.0),
                        flex_direction: FlexDirection::Column,
                        row_gap: Val::Px(6.0),
                        ..default()
                    })
                    .with_children(|loadout| {
                        crate::ui::mockup_layout::mockup_gear_cards(loadout, profile, ph);
                    });
                    dialog.spawn(section_title(if summary_loot.is_some() {
                        "RUN LOOT (preview)"
                    } else {
                        "STASH"
                    }));
                    spawn_stash_filters_and_sort_row(dialog, stash_sort);
                    gear_hub_scroll_list(
                        dialog,
                        use_rows,
                        interactive_inventory,
                        stash_sort,
                        ph,
                    );
                    let close_pal = UiButtonPalette::panel_outlined();
                    dialog
                        .spawn((
                            Node {
                                width: Val::Percent(100.0),
                                min_height: Val::Px(40.0),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                border: UiRect::all(Val::Px(1.0)),
                                ..default()
                            },
                            Button,
                            BackgroundColor(close_pal.idle_bg),
                            BorderColor::from(close_pal.idle_border),
                            GearHubCloseButton,
                            close_pal,
                            UiTooltip::txt("Close the gear hub."),
                        ))
                        .with_children(|b| {
                            b.spawn((
                                Text::new("Close"),
                                TextFont::from_font_size(UiTheme::FONT_BODY),
                                TextColor(UiTheme::muted_cream()),
                            ));
                        });
                });
        });
}

fn gear_hub_scroll_list(
    parent: &mut ChildSpawnerCommands<'_>,
    rows: &[crate::domain::items::ItemInstance],
    interactive_inventory: bool,
    stash_sort: StashSortOrder,
    ph: &UiPlaceholderImages,
) {
    parent
        .spawn((
            Node {
                width: Val::Percent(100.0),
                flex_grow: 1.0,
                flex_shrink: 1.0,
                flex_basis: Val::Px(0.0),
                min_height: Val::Px(72.0),
                max_height: Val::Px(240.0),
                position_type: PositionType::Relative,
                flex_direction: FlexDirection::Column,
                overflow: Overflow::clip_y(),
                border: UiRect::all(Val::Px(1.0)),
                ..default()
            },
            BackgroundColor(UiTheme::panel_bg()),
            BorderColor::from(UiTheme::panel_border_inner()),
            FocusPolicy::Pass,
            RelativeCursorPosition::default(),
            UiScrollState::default(),
            UiScrollRegion,
        ))
        .with_children(|viewport| {
            viewport
                .spawn((
                    Node {
                        position_type: PositionType::Absolute,
                        left: Val::Px(0.0),
                        right: Val::Px(0.0),
                        top: Val::Px(0.0),
                        padding: UiRect::all(Val::Px(UiTheme::PANEL_INSET_SM)),
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Stretch,
                        row_gap: Val::Px(6.0),
                        ..default()
                    },
                    UiScrollContent,
                ))
                .with_children(|body| {
                    if rows.is_empty() {
                        body.spawn(caption_text("No items in this list."));
                    } else {
                        let ix = crate::ui::stash_sort::stash_display_indices(rows, stash_sort);
                        for &i in ix.iter().take(40) {
                            let item = &rows[i];
                            if interactive_inventory {
                                crate::ui::spawn_item_card(body, item, ph);
                            } else {
                                body.spawn(caption_text(format!(
                                    "\u{2022} {} ({:?})",
                                    item.name, item.rarity
                                )));
                            }
                        }
                    }
                });
        });
}
