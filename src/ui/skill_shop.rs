//! Spend gold to permanently add skills to the account library.

use bevy::prelude::*;
use bevy::text::{TextColor, TextFont};
use bevy::ui::FocusPolicy;

use crate::domain::skills::{skill_definition, skill_shop_price_gold, SkillId, SkillKind};
use crate::ui::components::{
    SkillShopBackdrop, SkillShopBuyButton, SkillShopCloseButton, SkillShopRoot, UiButtonPalette,
    UiScrollContent, UiScrollRegion, UiScrollState, UiTooltip,
};
use crate::ui::theme::{caption_text, headline_text, section_title, UiTheme};

pub fn spawn_skill_shop_modal(
    parent: &mut ChildSpawnerCommands<'_>,
    unlocked: &[SkillId],
    gold: u32,
) {
    let unlocked_set: std::collections::HashSet<SkillId> = unlocked.iter().copied().collect();
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
                    box_sizing: BoxSizing::BorderBox,
                    position_type: PositionType::Absolute,
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    left: Val::Px(0.0),
                    top: Val::Px(0.0),
                    ..default()
                },
                Button,
                BackgroundColor(backdrop_pal.idle_bg.into()),
                BorderColor::from(backdrop_pal.idle_border),
                SkillShopBackdrop,
                backdrop_pal,
                UiTooltip::txt("Click outside to close."),
            ));
            layer
                .spawn((
                    Node {
                        box_sizing: BoxSizing::BorderBox,
                        position_type: PositionType::Absolute,
                        left: Val::Percent(50.0),
                        top: Val::Percent(45.0),
                        margin: UiRect {
                            left: Val::Px(-260.0),
                            top: Val::Px(-220.0),
                            right: Val::Auto,
                            bottom: Val::Auto,
                        },
                        width: Val::Px(520.0),
                        max_height: Val::Percent(88.0),
                        padding: UiRect::all(Val::Px(UiTheme::PAD_ROOT)),
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Stretch,
                        row_gap: Val::Px(UiTheme::PANEL_INSET),
                        border: UiRect::all(Val::Px(2.0)),
                        ..default()
                    },
                    BackgroundColor(UiTheme::panel_bg_deep().into()),
                    BorderColor::from(UiTheme::ornate_gold()),
                ))
                .with_children(|dialog| {
                    dialog.spawn(headline_text("Skill guild"));
                    dialog.spawn(caption_text(format!(
                        "Gold: {} — purchase skills to expand your library.",
                        gold
                    )));
                    dialog.spawn(section_title("CATALOGUE"));
                    dialog
                        .spawn((
                            Node {
                                box_sizing: BoxSizing::BorderBox,
                                width: Val::Percent(100.0),
                                height: Val::Px(280.0),
                                flex_shrink: 0.0,
                                position_type: PositionType::Relative,
                                flex_direction: FlexDirection::Column,
                                overflow: Overflow::clip_y(),
                                border: UiRect::all(Val::Px(1.0)),
                                ..default()
                            },
                            BackgroundColor(UiTheme::panel_bg().into()),
                            BorderColor::from(UiTheme::panel_border_inner()),
                            FocusPolicy::Pass,
                            bevy::ui::RelativeCursorPosition::default(),
                            UiScrollState::default(),
                            UiScrollRegion,
                        ))
                        .with_children(|viewport| {
                            viewport
                                .spawn((
                                    Node {
                                        box_sizing: BoxSizing::BorderBox,
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
                                .with_children(|inner| {
                                    let mut any = false;
                                    for &id in SkillId::ALL {
                                        if unlocked_set.contains(&id) {
                                            continue;
                                        }
                                        let Some(price) = skill_shop_price_gold(id) else {
                                            continue;
                                        };
                                        any = true;
                                        spawn_buy_row(inner, id, price, gold);
                                    }
                                    if !any {
                                        inner.spawn(caption_text(
                                            "Every discoverable skill is in your book.",
                                        ));
                                    }
                                });
                        });
                    let close_pal = UiButtonPalette::panel_outlined();
                    dialog
                        .spawn((
                            Node {
                                box_sizing: BoxSizing::BorderBox,
                                width: Val::Percent(100.0),
                                min_height: Val::Px(40.0),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                border: UiRect::all(Val::Px(1.0)),
                                ..default()
                            },
                            Button,
                            BackgroundColor(close_pal.idle_bg.into()),
                            BorderColor::from(close_pal.idle_border),
                            SkillShopCloseButton,
                            close_pal,
                            UiTooltip::txt("Close"),
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

fn spawn_buy_row(inner: &mut ChildSpawnerCommands<'_>, id: SkillId, price: u32, gold: u32) {
    let d = skill_definition(id);
    let kind_str = match d.kind {
        SkillKind::Active => "Active",
        SkillKind::Passive => "Passive",
    };
    let can_afford = gold >= price;
    let label = format!(
        "Buy {} · {} · {} gold {}",
        d.name,
        kind_str,
        price,
        if can_afford { "" } else { "(need gold)" }
    );
    let tip = if can_afford {
        format!("{}\n{}\n\n{}", d.name, d.description, d.synergy_hint)
    } else {
        format!(
            "{}\n{}\n\n{}\nNeed {} more gold.",
            d.name,
            d.description,
            d.synergy_hint,
            price.saturating_sub(gold),
        )
    };

    let row_bg = UiTheme::panel_bg_deep();

    // When broke, show a passive row — no [`Button`] / [`SkillShopBuyButton`] so we never
    // enqueue a doomed purchase click (was easy to confuse with "button dead").
    if !can_afford {
        let p = UiButtonPalette::panel_outlined();
        inner
            .spawn((
                Node {
                    box_sizing: BoxSizing::BorderBox,
                    width: Val::Percent(100.0),
                    min_height: Val::Px(44.0),
                    justify_content: JustifyContent::FlexStart,
                    align_items: AlignItems::Center,
                    padding: UiRect::axes(Val::Px(8.0), Val::Px(4.0)),
                    border: UiRect::all(Val::Px(1.0)),
                    ..default()
                },
                BackgroundColor(row_bg.into()),
                BorderColor::from(p.idle_border),
                UiTooltip::txt(tip.clone()),
            ))
            .with_children(|b| {
                b.spawn((
                    Text::new(label),
                    TextFont::from_font_size(UiTheme::FONT_COMPACT),
                    TextColor(UiTheme::body_dim()),
                ));
            });
        return;
    }

    let p = UiButtonPalette::primary_cta();
    inner
        .spawn((
            Node {
                box_sizing: BoxSizing::BorderBox,
                width: Val::Percent(100.0),
                min_height: Val::Px(44.0),
                justify_content: JustifyContent::FlexStart,
                align_items: AlignItems::Center,
                padding: UiRect::axes(Val::Px(8.0), Val::Px(4.0)),
                border: UiRect::all(Val::Px(1.0)),
                ..default()
            },
            Button,
            BackgroundColor(p.idle_bg.into()),
            BorderColor::from(p.idle_border),
            SkillShopBuyButton { skill: id },
            p,
            UiTooltip::txt(tip),
        ))
        .with_children(|b| {
            b.spawn((
                Text::new(label),
                TextFont::from_font_size(UiTheme::FONT_COMPACT),
                TextColor(UiTheme::body()),
            ));
        });
}
