//! Gear hub: loadout and stash as two side-by-side panels (shared backdrop + close).

use bevy::prelude::*;
use bevy::text::{TextColor, TextFont};
use bevy::ui::{FocusPolicy, RelativeCursorPosition};

use crate::save::StashSortOrder;
use crate::ui::components::{
    GearHubBackdrop, GearHubCloseButton, GearHubRoot, UiButtonPalette, UiScrollContent,
    UiScrollRegion, UiScrollState, UiTooltip,
};
use crate::ui::mockup_layout::spawn_stash_filters_and_sort_row;
use crate::ui::placeholder_graphics::UiPlaceholderImages;
use crate::ui::theme::{caption_text, headline_text, section_title, UiTheme};

/// Equipped-gear column — three slot rows fit without crowding.
const GEAR_LOADOUT_PANEL_W: f32 = 300.0;
/// Stash column — item cards plus equip/salvage need a little more width.
const GEAR_STASH_PANEL_W: f32 = 372.0;
/// Max height shared by both panels so the pair clears the footer dock.
const GEAR_HUB_PANEL_MAX_H: f32 = 528.0;
/// Stash list viewport inside the right panel.
const GEAR_STASH_SCROLL_MAX_PX: f32 = 392.0;

fn gear_side_panel_node(width_px: f32) -> Node {
    Node {
        box_sizing: BoxSizing::BorderBox,
        width: Val::Px(width_px),
        max_height: Val::Px(GEAR_HUB_PANEL_MAX_H),
        min_height: Val::Px(0.0),
        flex_shrink: 0.0,
        flex_direction: FlexDirection::Column,
        align_items: AlignItems::Stretch,
        row_gap: Val::Px(UiTheme::PANEL_INSET),
        padding: UiRect::all(Val::Px(UiTheme::PAD_ROOT)),
        overflow: Overflow::clip_y(),
        border: UiRect::all(Val::Px(2.0)),
        ..default()
    }
}

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
                UiTooltip::txt("Click outside empty space to close the gear hub."),
            ));

            layer
                .spawn((
                    Node {
                        box_sizing: BoxSizing::BorderBox,
                        width: Val::Percent(100.0),
                        height: Val::Percent(100.0),
                        position_type: PositionType::Absolute,
                        left: Val::Px(0.0),
                        top: Val::Px(0.0),
                        flex_direction: FlexDirection::Row,
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::FlexStart,
                        align_content: AlignContent::FlexStart,
                        column_gap: Val::Px(20.0),
                        padding: UiRect::new(
                            Val::Px(18.0),
                            Val::Px(18.0),
                            Val::Px(54.0),
                            Val::Px(92.0),
                        ),
                        ..default()
                    },
                    FocusPolicy::Pass,
                ))
                .with_children(|columns| {
                    columns
                        .spawn((
                            gear_side_panel_node(GEAR_LOADOUT_PANEL_W),
                            BackgroundColor(UiTheme::panel_bg_deep()),
                            BorderColor::from(UiTheme::ornate_gold()),
                        ))
                        .with_children(|loadout_panel| {
                            loadout_panel.spawn(headline_text("Equipped"));
                            loadout_panel.spawn(section_title("LOADOUT"));
                            loadout_panel
                                .spawn(Node {
                                    width: Val::Percent(100.0),
                                    flex_direction: FlexDirection::Column,
                                    flex_shrink: 0.0,
                                    row_gap: Val::Px(6.0),
                                    ..default()
                                })
                                .with_children(|loadout| {
                                    crate::ui::mockup_layout::mockup_gear_cards(
                                        loadout, profile, ph,
                                    );
                                });
                        });

                    columns
                        .spawn((
                            gear_side_panel_node(GEAR_STASH_PANEL_W),
                            BackgroundColor(UiTheme::panel_bg_deep()),
                            BorderColor::from(UiTheme::ornate_gold()),
                        ))
                        .with_children(|stash_panel| {
                            stash_panel.spawn(headline_text("Stash"));
                            stash_panel.spawn(section_title(if summary_loot.is_some() {
                                "RUN LOOT (preview)"
                            } else {
                                "INVENTORY"
                            }));
                            spawn_stash_filters_and_sort_row(stash_panel, stash_sort);
                            gear_hub_scroll_list(
                                stash_panel,
                                use_rows,
                                interactive_inventory,
                                stash_sort,
                                ph,
                                GEAR_STASH_SCROLL_MAX_PX,
                            );
                            let close_pal = UiButtonPalette::panel_outlined();
                            stash_panel
                                .spawn((
                                    Node {
                                        width: Val::Percent(100.0),
                                        min_height: Val::Px(40.0),
                                        flex_shrink: 0.0,
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
                                    UiTooltip::txt("Close both gear panels."),
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
        });
}

fn gear_hub_scroll_list(
    parent: &mut ChildSpawnerCommands<'_>,
    rows: &[crate::domain::items::ItemInstance],
    interactive_inventory: bool,
    stash_sort: StashSortOrder,
    ph: &UiPlaceholderImages,
    scroll_max_px: f32,
) {
    parent
        .spawn((
            Node {
                width: Val::Percent(100.0),
                flex_grow: 1.0,
                flex_shrink: 1.0,
                flex_basis: Val::Px(0.0),
                min_height: Val::Px(72.0),
                max_height: Val::Px(scroll_max_px),
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
