//! Gear hub: loadout + stash side-by-side (shared backdrop + close).

use bevy::prelude::*;
use bevy::ui::{FocusPolicy, RelativeCursorPosition};

use crate::save::StashSortOrder;
use crate::ui::components::{
    GearHubBackdrop, GearHubCloseButton, GearHubRoot, UiScrollContent, UiScrollRegion,
    UiScrollState, UiTooltip,
};
use crate::ui::shell::spawn_stash_filters_and_sort_row;
use crate::ui::assets::UiPlaceholderImages;
use crate::ui::primitives::modal::{spawn_modal_shell_with_handles, ModalShellConfig};
use crate::ui::primitives::scroll::spawn_scrollable_flex_column;
use crate::ui::primitives::{spawn_button, UiButtonConfig, UiButtonVariant};
use crate::ui::theme::{caption_text, headline_text, section_title, UiTheme};

/// Equipped-gear column.
const GEAR_LOADOUT_PANEL_W: f32 = 292.0;
/// Stash — wide enough for two item cards side by side.
const GEAR_STASH_PANEL_W: f32 = 652.0;
/// Outer padding: equal top / bottom so panels sit with symmetric vertical margin.
const GEAR_HUB_MARGIN_Y: f32 = 44.0;
/// Equal horizontal inset when the hub is centered.
const GEAR_HUB_MARGIN_X: f32 = 28.0;
const GEAR_HUB_COLUMN_GAP: f32 = 18.0;

fn gear_side_panel_node(width_px: f32) -> Node {
    Node {
        box_sizing: BoxSizing::BorderBox,
        width: Val::Px(width_px),
        min_height: Val::Px(0.0),
        flex_shrink: 0.0,
        flex_direction: FlexDirection::Column,
        align_items: AlignItems::Stretch,
        align_self: AlignSelf::Stretch,
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
            let shell = spawn_modal_shell_with_handles(
                layer,
                ModalShellConfig {
                    backdrop_clicks_close: true,
                },
                |columns| {
                    columns
                        .spawn((
                            Node {
                                box_sizing: BoxSizing::BorderBox,
                                max_width: Val::Percent(100.0),
                                height: Val::Percent(100.0),
                                flex_direction: FlexDirection::Row,
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Stretch,
                                column_gap: Val::Px(GEAR_HUB_COLUMN_GAP),
                                padding: UiRect::axes(
                                    Val::Px(GEAR_HUB_MARGIN_X),
                                    Val::Px(GEAR_HUB_MARGIN_Y),
                                ),
                                ..default()
                            },
                            FocusPolicy::Pass,
                        ))
                        .with_children(|panels| {
                            panels
                                .spawn((
                                    gear_side_panel_node(GEAR_LOADOUT_PANEL_W),
                                    BackgroundColor(UiTheme::panel_bg_deep()),
                                    BorderColor::from(UiTheme::ornate_gold()),
                                ))
                                .with_children(|loadout_panel| {
                                    loadout_panel.spawn(headline_text("Equipped"));
                                    loadout_panel.spawn(section_title("LOADOUT"));
                                    spawn_scrollable_flex_column(loadout_panel, None, |loadout| {
                                        crate::ui::shell::mockup_gear_cards(
                                            loadout, profile, ph,
                                        );
                                    });
                                });

                            panels
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
                                    );
                                    let close_ent = spawn_button(
                                        stash_panel,
                                        UiButtonConfig {
                                            label: "Close",
                                            variant: UiButtonVariant::PanelOutlined,
                                            width: Val::Percent(100.0),
                                            height: Val::Px(40.0),
                                            font_size: UiTheme::FONT_BODY,
                                            text_color: UiTheme::muted_cream(),
                                            flex_shrink: 0.0,
                                        },
                                    );
                                    stash_panel.commands_mut().entity(close_ent).insert((
                                        GearHubCloseButton,
                                        UiTooltip::txt("Close both gear panels."),
                                    ));
                                });
                        });
                },
            );
            layer.commands_mut().entity(shell.backdrop).insert((
                GearHubBackdrop,
                UiTooltip::txt("Click outside empty space to close the gear hub."),
            ));
        });
}

/// Split ordered indices into two columns (top-to-bottom in each, left column gets the extra when odd count).
fn partition_stash_indices(ix: &[usize]) -> (&[usize], &[usize]) {
    let mid = (ix.len() + 1) / 2;
    ix.split_at(mid)
}

fn spawn_stash_column(
    parent: &mut ChildSpawnerCommands<'_>,
    rows: &[crate::domain::items::ItemInstance],
    col_ix: &[usize],
    interactive_inventory: bool,
    ph: &UiPlaceholderImages,
) {
    parent
        .spawn(Node {
            box_sizing: BoxSizing::BorderBox,
            flex_grow: 1.0,
            flex_basis: Val::Px(0.0),
            min_width: Val::Px(0.0),
            width: Val::Percent(50.0),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Stretch,
            row_gap: Val::Px(6.0),
            ..default()
        })
        .with_children(|col| {
            for &i in col_ix {
                let item = &rows[i];
                if interactive_inventory {
                    crate::ui::spawn_item_card(col, item, ph);
                } else {
                    col.spawn(caption_text(format!(
                        "\u{2022} {} ({:?})",
                        item.name, item.rarity
                    )));
                }
            }
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
                        flex_direction: FlexDirection::Row,
                        align_items: AlignItems::FlexStart,
                        column_gap: Val::Px(8.0),
                        width: Val::Percent(100.0),
                        ..default()
                    },
                    UiScrollContent,
                ))
                .with_children(|body| {
                    if rows.is_empty() {
                        body.spawn(caption_text("No items in this list."));
                        return;
                    }
                    let ix = crate::ui::stash_sort::stash_display_indices(rows, stash_sort);
                    let ix: Vec<usize> = ix.into_iter().take(40).collect();
                    let (left_ix, right_ix) = partition_stash_indices(&ix);
                    spawn_stash_column(body, rows, left_ix, interactive_inventory, ph);
                    spawn_stash_column(body, rows, right_ix, interactive_inventory, ph);
                });
        });
}
