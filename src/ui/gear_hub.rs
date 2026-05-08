//! Gear hub modal: paper doll + stash (sort, equip, salvage). Closes only via the close control.

use bevy::prelude::*;
use bevy::ui::{FocusPolicy, RelativeCursorPosition};

use crate::save::StashSortOrder;
use crate::ui::components::{
    GearHubBackdrop, GearHubCloseButton, GearHubRoot, UiButtonPalette, UiScrollContent,
    UiScrollRegion, UiScrollState, UiTooltip,
};
use crate::ui::mockup_layout::spawn_stash_filters_and_sort_row;
use crate::ui::placeholder_graphics::UiPlaceholderImages;
use crate::ui::theme::{caption_text, headline_text, section_title, UiTheme};

pub fn spawn_gear_hub_modal(
    parent: &mut ChildBuilder,
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
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    position_type: PositionType::Absolute,
                    left: Val::Px(0.0),
                    top: Val::Px(0.0),
                    ..default()
                },
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
                ButtonBundle {
                    style: Style {
                        position_type: PositionType::Absolute,
                        width: Val::Percent(100.0),
                        height: Val::Percent(100.0),
                        left: Val::Px(0.0),
                        top: Val::Px(0.0),
                        ..default()
                    },
                    background_color: backdrop_pal.idle_bg.into(),
                    border_color: BorderColor(backdrop_pal.idle_border),
                    ..default()
                },
                GearHubBackdrop,
                backdrop_pal,
                UiTooltip::txt("Click outside to close the gear hub."),
            ));
            layer
                .spawn(NodeBundle {
                    style: Style {
                        position_type: PositionType::Absolute,
                        left: Val::Percent(50.0),
                        top: Val::Percent(42.0),
                        margin: UiRect {
                            left: Val::Px(-280.0),
                            top: Val::Px(-220.0),
                            right: Val::Auto,
                            bottom: Val::Auto,
                        },
                        width: Val::Px(560.0),
                        max_height: Val::Percent(90.0),
                        padding: UiRect::all(Val::Px(UiTheme::PAD_ROOT)),
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Stretch,
                        row_gap: Val::Px(UiTheme::PANEL_INSET),
                        border: UiRect::all(Val::Px(2.0)),
                        ..default()
                    },
                    background_color: UiTheme::panel_bg_deep().into(),
                    border_color: BorderColor(UiTheme::ornate_gold()),
                    ..default()
                })
                .with_children(|dialog| {
                    dialog.spawn(headline_text("Gear"));
                    dialog.spawn(section_title("LOADOUT"));
                    dialog.spawn(NodeBundle {
                        style: Style {
                            width: Val::Percent(100.0),
                            flex_direction: FlexDirection::Column,
                            row_gap: Val::Px(6.0),
                            ..default()
                        },
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
                            ButtonBundle {
                                style: Style {
                                    width: Val::Percent(100.0),
                                    min_height: Val::Px(40.0),
                                    justify_content: JustifyContent::Center,
                                    align_items: AlignItems::Center,
                                    border: UiRect::all(Val::Px(1.0)),
                                    ..default()
                                },
                                background_color: close_pal.idle_bg.into(),
                                border_color: BorderColor(close_pal.idle_border),
                                ..default()
                            },
                            GearHubCloseButton,
                            close_pal,
                            UiTooltip::txt("Close the gear hub."),
                        ))
                        .with_children(|b| {
                            b.spawn(TextBundle::from_section(
                                "Close",
                                TextStyle {
                                    font_size: UiTheme::FONT_BODY,
                                    color: UiTheme::muted_cream(),
                                    ..default()
                                },
                            ));
                        });
                });
        });
}

fn gear_hub_scroll_list(
    parent: &mut ChildBuilder,
    rows: &[crate::domain::items::ItemInstance],
    interactive_inventory: bool,
    stash_sort: StashSortOrder,
    ph: &UiPlaceholderImages,
) {
    parent
        .spawn((
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    height: Val::Px(340.0),
                    flex_shrink: 0.0,
                    position_type: PositionType::Relative,
                    flex_direction: FlexDirection::Column,
                    overflow: Overflow::clip_y(),
                    border: UiRect::all(Val::Px(1.0)),
                    ..default()
                },
                background_color: UiTheme::panel_bg().into(),
                border_color: BorderColor(UiTheme::panel_border_inner()),
                focus_policy: FocusPolicy::Pass,
                ..default()
            },
            RelativeCursorPosition::default(),
            UiScrollState::default(),
            UiScrollRegion,
        ))
        .with_children(|viewport| {
            viewport
                .spawn((
                    NodeBundle {
                        style: Style {
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
                        ..default()
                    },
                    UiScrollContent,
                ))
                .with_children(|body| {
                    if rows.is_empty() {
                        body.spawn(caption_text("No items in this list."));
                    } else {
                        let ix =
                            crate::ui::stash_sort::stash_display_indices(rows, stash_sort);
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
