//! Spend gold to permanently add skills to the account library.

use bevy::prelude::*;
use bevy::text::{TextColor, TextFont};
use bevy::ui::FocusPolicy;

use crate::domain::skills::{skill_shop_price_gold, SkillId};
use crate::ui::assets::UiPlaceholderImages;
use crate::ui::components::{SkillShopBackdrop, SkillShopBuyButton, SkillShopCloseButton, SkillShopRoot};
use crate::ui::inspect::{InspectHint, InspectRegion, InspectRegionScope, SkillShopInspect};
use crate::ui::interaction::UiClickAction;
use crate::ui::primitives::inspect_panel::spawn_inspect_panel_compact;
use crate::ui::primitives::modal::{spawn_modal_shell_with_handles, ModalShellConfig};
use crate::ui::primitives::panel::{spawn_mounted_panel, MountedPanelConfig};
use crate::ui::primitives::scroll::spawn_scroll_viewport;
use crate::ui::primitives::section::spawn_framed_section_header;
use crate::ui::primitives::skill_icon::{spawn_skill_icon, SkillIconConfig};
use crate::ui::primitives::{spawn_button, UiButtonConfig, UiButtonVariant};
use crate::ui::skill_presentation::accent_for_skill;
use crate::ui::theme::{caption_text, headline_text, UiDensity, UiTheme};

const SHOP_MODAL_MARGIN_X: f32 = 28.0;
const SHOP_MODAL_MARGIN_Y: f32 = 44.0;
const SHOP_MODAL_W: f32 = 520.0;
const SHOP_CATALOG_H: f32 = 280.0;

pub fn spawn_skill_shop_modal(
    parent: &mut ChildSpawnerCommands<'_>,
    unlocked: &[SkillId],
    gold: u32,
    ph: &UiPlaceholderImages,
) {
    let unlocked_set: std::collections::HashSet<SkillId> = unlocked.iter().copied().collect();
    let density = UiDensity::Camp;
    let icon_px = density.icon_library_px();
    let grid_gap = density.gutter_grid();

    parent
        .spawn((
            Node {
                box_sizing: BoxSizing::BorderBox,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                position_type: PositionType::Absolute,
                left: Val::Px(0.0),
                top: Val::Px(0.0),
                ..default()
            },
            SkillShopRoot,
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
                                width: Val::Percent(100.0),
                                height: Val::Percent(100.0),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                padding: UiRect::axes(
                                    Val::Px(SHOP_MODAL_MARGIN_X),
                                    Val::Px(SHOP_MODAL_MARGIN_Y),
                                ),
                                ..default()
                            },
                            FocusPolicy::Pass,
                        ))
                        .with_children(|center| {
                            center
                                .spawn((
                                    Node {
                                        box_sizing: BoxSizing::BorderBox,
                                        width: Val::Px(SHOP_MODAL_W),
                                        max_height: Val::Percent(88.0),
                                        padding: UiRect::all(Val::Px(UiTheme::PAD_ROOT)),
                                        flex_direction: FlexDirection::Column,
                                        align_items: AlignItems::Stretch,
                                        row_gap: Val::Px(UiTheme::PANEL_INSET),
                                        border: UiRect::all(Val::Px(2.0)),
                                        overflow: Overflow::clip_y(),
                                        ..default()
                                    },
                                    BackgroundColor(UiTheme::panel_bg_deep()),
                                    BorderColor::from(UiTheme::ornate_gold()),
                                ))
                                .with_children(|dialog| {
                                    dialog.spawn(headline_text("Skill guild"));
                                    dialog.spawn(caption_text(format!(
                                        "Gold: {gold} — purchase skills to expand your library."
                                    )));
                                    spawn_framed_section_header(dialog, "CATALOGUE");
                                    dialog
                                        .spawn(Node {
                                            box_sizing: BoxSizing::BorderBox,
                                            width: Val::Percent(100.0),
                                            height: Val::Px(SHOP_CATALOG_H),
                                            min_height: Val::Px(SHOP_CATALOG_H),
                                            flex_shrink: 0.0,
                                            flex_direction: FlexDirection::Column,
                                            ..default()
                                        })
                                        .with_children(|holder| {
                                            spawn_mounted_panel(
                                                holder,
                                                MountedPanelConfig::recessed_flex(),
                                                |frame| {
                                                    spawn_scroll_viewport(frame, |scroll| {
                                                        scroll
                                                            .spawn(Node {
                                                                box_sizing: BoxSizing::BorderBox,
                                                                width: Val::Percent(100.0),
                                                                flex_direction: FlexDirection::Row,
                                                                flex_wrap: FlexWrap::Wrap,
                                                                column_gap: Val::Px(grid_gap),
                                                                row_gap: Val::Px(grid_gap),
                                                                ..default()
                                                            })
                                                            .with_children(|grid| {
                                                                let mut any = false;
                                                                for &id in SkillId::ALL {
                                                                    if unlocked_set.contains(&id) {
                                                                        continue;
                                                                    }
                                                                    let Some(price) =
                                                                        skill_shop_price_gold(id)
                                                                    else {
                                                                        continue;
                                                                    };
                                                                    any = true;
                                                                    spawn_catalog_skill(
                                                                        grid,
                                                                        id,
                                                                        price,
                                                                        gold,
                                                                        ph,
                                                                        icon_px,
                                                                    );
                                                                }
                                                                if !any {
                                                                    grid.spawn(caption_text(
                                                                        "Every discoverable skill is in your book.",
                                                                    ));
                                                                }
                                                            });
                                                    });
                                                },
                                            );
                                        });
                                    let inspect = spawn_inspect_panel_compact(
                                        dialog,
                                        InspectRegionScope::SkillShop,
                                    );
                                    dialog.commands_mut().entity(inspect).insert((
                                        InspectRegion,
                                        InspectRegionScope::SkillShop,
                                    ));
                                    let close_ent = spawn_button(
                                        dialog,
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
                                    dialog.commands_mut().entity(close_ent).insert((
                                        SkillShopCloseButton,
                                        UiClickAction::CloseSkillShop,
                                        InspectHint("Close skill shop."),
                                    ));
                                });
                        });
                },
            );
            layer.commands_mut().entity(shell.backdrop).insert((
                SkillShopBackdrop,
                UiClickAction::CloseSkillShop,
                InspectHint("Click outside to close."),
            ));
        });
}

fn spawn_catalog_skill(
    parent: &mut ChildSpawnerCommands<'_>,
    id: SkillId,
    price: u32,
    gold: u32,
    ph: &UiPlaceholderImages,
    icon_px: f32,
) {
    let can_afford = gold >= price;
    let accent = accent_for_skill(id);

    parent
        .spawn(Node {
            box_sizing: BoxSizing::BorderBox,
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            row_gap: Val::Px(4.0),
            width: Val::Px(icon_px),
            ..default()
        })
        .with_children(|cell| {
            let icon_ent = spawn_skill_icon(
                cell,
                SkillIconConfig::filled(id, icon_px),
                ph,
            );
            cell.commands_mut().entity(icon_ent).insert(SkillShopInspect(id));

            if can_afford {
                cell.commands_mut().entity(icon_ent).insert((
                    Button,
                    SkillShopBuyButton { skill: id },
                    UiClickAction::BuySkillUnlock,
                    BorderColor::from(accent.mix(&UiTheme::void_black(), 0.25)),
                    InspectHint("Click to buy — added to your library permanently."),
                ));
            } else {
                cell.commands_mut().entity(icon_ent).insert((
                    Interaction::default(),
                    BorderColor::from(UiTheme::panel_border_inner()),
                ));
            }

            cell.spawn((
                Text::new(format!("{price}g")),
                TextFont::from_font_size(UiTheme::FONT_MICRO),
                TextColor(if can_afford {
                    UiTheme::muted_gold()
                } else {
                    UiTheme::body_dim()
                }),
            ));
        });
}
