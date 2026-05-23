//! Footer dock, stash filter row, and post-run rewards modal.

use bevy::prelude::*;
use bevy::text::{TextColor, TextFont};
use bevy::ui::FocusPolicy;

use crate::domain::run::RunSummary;
use crate::save::StashSortOrder;
use crate::ui::assets::UiPlaceholderImages;
use crate::ui::components::{
    GearHubOpenButton, SkillShopOpenButton, SkipPlaybackButton, StashSortCycleButton,
    UiButtonPalette, UiTooltip,
};
use crate::ui::theme::{body_text, caption_text, headline_text, section_title, UiTheme};

use super::layout::spawn_column_flex_scroll;

pub fn spawn_stash_filters_and_sort_row(
    parent: &mut ChildSpawnerCommands<'_>,
    stash_sort: StashSortOrder,
) {
    parent
        .spawn(Node {
                box_sizing: BoxSizing::BorderBox,
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                column_gap: Val::Px(10.0),
                justify_content: JustifyContent::SpaceBetween,
                flex_wrap: FlexWrap::Wrap,
                margin: UiRect::bottom(Val::Px(2.0)),
                ..default()
            })
        .with_children(|row| {
            row.spawn(caption_text("Stash filters: —"));
            let p = UiButtonPalette::panel_secondary();
            row.spawn((
                Node {
                box_sizing: BoxSizing::BorderBox,
                min_width: Val::Px(168.0),
                        height: Val::Px(28.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        padding: UiRect::horizontal(Val::Px(8.0)),
                        border: UiRect::all(Val::Px(1.0)),
                        ..default()
            },
            Button,
            BackgroundColor(p.idle_bg.into()),
            BorderColor::from(p.idle_border),
                StashSortCycleButton,
                p,
                UiTooltip::txt(
                    "Cycle stash sort. Newest-first follows save-file order (last appended = newest). Rarity: Rare → Uncommon → Common, then name A–Z, then item id.",
                ),
            ))
            .with_children(|b| {
                b.spawn((
                Text::new(stash_sort.button_label()),
                TextFont::from_font_size(UiTheme::FONT_LABEL),
                TextColor(UiTheme::body_dim()),
            ));
            });
        });
}

/// Post-run rewards modal: loot list + accept (non-dismissible dimmer).
pub fn spawn_summary_rewards_modal(
    parent: &mut ChildSpawnerCommands<'_>,
    summary: &RunSummary,
    stash_sort: StashSortOrder,
    ph: &UiPlaceholderImages,
) {
    use crate::ui::components::{AcceptRewardsButton, SummaryRewardsModalRoot};
    parent
        .spawn((
            Node {
                box_sizing: BoxSizing::BorderBox,
                width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    position_type: PositionType::Absolute,
                    left: Val::Px(0.0),
                    top: Val::Px(0.0),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    ..default()
            },
            SummaryRewardsModalRoot,
        ))
        .with_children(|layer| {
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
            BackgroundColor(Color::srgba(0.02, 0.02, 0.04, 0.72).into()),
            FocusPolicy::Pass
        ));
            layer
                .spawn((
            Node {
                box_sizing: BoxSizing::BorderBox,
                min_width: Val::Px(460.0),
                max_width: Val::Px(620.0),
                max_height: Val::Percent(85.0),
                padding: UiRect {
                    left: Val::Px(UiTheme::PAD_ROOT),
                    right: Val::Px(UiTheme::PAD_ROOT),
                    top: Val::Px(UiTheme::PAD_ROOT),
                    bottom: Val::Px(UiTheme::PAD_ROOT + 22.0),
                },
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Stretch,
                row_gap: Val::Px(10.0),
                border: UiRect::all(Val::Px(2.0)),
                ..default()
            },
            BackgroundColor(UiTheme::panel_bg_deep().into()),
            BorderColor::from(UiTheme::ornate_gold())
        ))
                .with_children(|dialog| {
                    dialog.spawn(headline_text("Run rewards"));
                    dialog.spawn(caption_text(format!(
                        "Gold +{} · Salvage +{} · Depth {}",
                        summary.gold_earned, summary.salvage_earned, summary.deepest_depth
                    )));
                    if let Some(pct) = summary.strike_ability_share_percent() {
                        dialog.spawn(caption_text(format!(
                            "Strikes: {}% ability · {} weapon / {} ability",
                            pct,
                            summary.party_strike_damage_white,
                            summary.party_strike_damage_yellow
                        )));
                    }
                    if summary.loot.is_empty() {
                        dialog.spawn(body_text(
                            "No gear dropped this run—gold and salvage still apply.",
                        ));
                    } else {
                        let n = summary.loot.len();
                        dialog.spawn((
                            Text::new(if n == 1 {
                                "YOU FOUND NEW GEAR (1)".to_string()
                            } else {
                                format!("YOU FOUND NEW GEAR ({n})")
                            }),
                            TextFont::from_font_size(UiTheme::FONT_SKILL_ACTIVE),
                            TextColor(UiTheme::muted_gold()),
                        ));
                        dialog.spawn(body_text(
                            "It is not equipped until after you Accept. Then use Gear on the footer bar to stash and equip.",
                        ));
                    }
                    dialog.spawn(section_title("LOOT"));
                    spawn_stash_filters_and_sort_row(dialog, stash_sort);
                    spawn_column_flex_scroll(dialog, Some(200.0), |scroll| {
                        if summary.loot.is_empty() {
                            scroll.spawn(caption_text("No items this run."));
                        } else {
                            let ix = crate::ui::stash_sort::stash_display_indices(
                                &summary.loot,
                                stash_sort,
                            );
                            for &i in ix.iter() {
                                let item = &summary.loot[i];
                                crate::ui::spawn_item_card_preview(scroll, item, ph);
                            }
                        }
                    });
                    let p = UiButtonPalette::primary_cta();
                    dialog
                        .spawn((
                            Node {
                box_sizing: BoxSizing::BorderBox,
                width: Val::Percent(100.0),
                                    min_height: Val::Px(48.0),
                                    justify_content: JustifyContent::Center,
                                    align_items: AlignItems::Center,
                                    border: UiRect::all(Val::Px(2.0)),
                                    margin: UiRect {
                                        left: Val::Px(0.0),
                                        right: Val::Px(0.0),
                                        top: Val::Px(12.0),
                                        bottom: Val::Px(6.0),
                                    },
                                    ..default()
            },
            Button,
            BackgroundColor(p.idle_bg.into()),
            BorderColor::from(p.idle_border),
                            AcceptRewardsButton,
                            p,
                            UiTooltip::txt(
                                "Add this run's gold, salvage, and loot to your profile and return to briefing.",
                            ),
                        ))
                        .with_children(|b| {
                            b.spawn((
                Text::new("\u{2713} Accept rewards"),
                TextFont::from_font_size(UiTheme::FONT_SKILL_ACTIVE),
                TextColor(Color::WHITE),
            ));
                        });
                });
        });
}

#[derive(Clone, Copy)]
pub enum FooterMode {
    Briefing,
    Summary,
    DelvePlayback,
}

pub fn spawn_mockup_footer(parent: &mut ChildSpawnerCommands<'_>, mode: FooterMode) {
    parent
        .spawn((
            Node {
                box_sizing: BoxSizing::BorderBox,
                width: Val::Percent(100.0),
                flex_shrink: 0.0,
                min_height: Val::Px(64.0),
                margin: UiRect::top(Val::Px(6.0)),
                padding: UiRect::axes(Val::Px(14.0), Val::Px(10.0)),
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                column_gap: Val::Px(12.0),
                justify_content: JustifyContent::SpaceBetween,
                border: UiRect::top(Val::Px(2.0)),
                ..default()
            },
            BackgroundColor(UiTheme::panel_bg_deep().into()),
            BorderColor::from(UiTheme::ornate_gold())
        ))
        .with_children(|row| {
            row.spawn(Node {
                box_sizing: BoxSizing::BorderBox,
                flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    column_gap: Val::Px(8.0),
                    flex_wrap: FlexWrap::Wrap,
                    ..default()
            })
            .with_children(|nav| {
                footer_pill(
                    nav,
                    "RUN",
                    matches!(mode, FooterMode::Briefing | FooterMode::DelvePlayback),
                );
                footer_pill(nav, "HERO", false);
                footer_pill(nav, "CODEX", false);
            });
            row.spawn(Node {
                box_sizing: BoxSizing::BorderBox,
                flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    column_gap: Val::Px(10.0),
                    flex_shrink: 0.0,
                    flex_wrap: FlexWrap::Wrap,
                    ..default()
            })
            .with_children(|right| {
                footer_gear_hub_button(right);
                match mode {
                FooterMode::Briefing => {
                    footer_skill_shop_button(right);
                    let p = UiButtonPalette::primary_cta();
                    right.spawn((
                        Node {
                box_sizing: BoxSizing::BorderBox,
                min_width: Val::Px(220.0),
                                height: Val::Px(52.0),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                border: UiRect::all(Val::Px(2.0)),
                                ..default()
            },
            Button,
            BackgroundColor(p.idle_bg.into()),
            BorderColor::from(p.idle_border),
                        crate::ui::components::StartRunButton,
                        p,
                        UiTooltip::txt(
                            "Begin a seeded dungeon run using your current hero build and stash.",
                        ),
                    ))
                    .with_children(|b| {
                        b.spawn((
                Text::new("\u{2694} START RUN"),
                TextFont::from_font_size(UiTheme::FONT_STRONG),
                TextColor(Color::WHITE),
            ));
                    });
                }
                FooterMode::DelvePlayback => {
                    let p = UiButtonPalette::panel_secondary();
                    right.spawn((
                        Node {
                box_sizing: BoxSizing::BorderBox,
                min_width: Val::Px(220.0),
                                height: Val::Px(44.0),
                                padding: UiRect::axes(Val::Px(14.0), Val::Px(8.0)),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                border: UiRect::all(Val::Px(1.0)),
                                ..default()
            },
            Button,
            BackgroundColor(p.idle_bg.into()),
            BorderColor::from(p.idle_border),
                        SkipPlaybackButton,
                        p,
                        UiTooltip::txt(
                            "Jump straight to the run summary without watching the rest of playback.",
                        ),
                    ))
                    .with_children(|b| {
                        b.spawn((
                Text::new("Skip to results"),
                TextFont::from_font_size(UiTheme::FONT_SKILL_DIM),
                TextColor(UiTheme::muted_cream()),
            ));
                    });
                }
                FooterMode::Summary => {}
                }
            });
        });
}

fn footer_gear_hub_button(parent: &mut ChildSpawnerCommands<'_>) {
    let p = UiButtonPalette::panel_secondary();
    parent
        .spawn((
            Node {
                box_sizing: BoxSizing::BorderBox,
                min_width: Val::Px(92.0),
                height: Val::Px(40.0),
                padding: UiRect::axes(Val::Px(12.0), Val::Px(8.0)),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                border: UiRect::all(Val::Px(1.0)),
                ..default()
            },
            Button,
            BackgroundColor(p.idle_bg.into()),
            BorderColor::from(p.idle_border),
            GearHubOpenButton,
            p,
            UiTooltip::txt("Open the gear hub (loadout and stash)."),
        ))
        .with_children(|b| {
            b.spawn((
                Text::new("\u{2692} Gear"),
                TextFont::from_font_size(UiTheme::FONT_BODY),
                TextColor(UiTheme::muted_cream()),
            ));
        });
}

fn footer_skill_shop_button(parent: &mut ChildSpawnerCommands<'_>) {
    let p = UiButtonPalette::panel_secondary();
    parent
        .spawn((
            Node {
                box_sizing: BoxSizing::BorderBox,
                min_width: Val::Px(104.0),
                height: Val::Px(40.0),
                padding: UiRect::axes(Val::Px(12.0), Val::Px(8.0)),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                border: UiRect::all(Val::Px(1.0)),
                ..default()
            },
            Button,
            BackgroundColor(p.idle_bg.into()),
            BorderColor::from(p.idle_border),
            SkillShopOpenButton,
            p,
            UiTooltip::txt("Spend gold to add skills to your library."),
        ))
        .with_children(|b| {
            b.spawn((
                Text::new("\u{1F4DA} Skills"),
                TextFont::from_font_size(UiTheme::FONT_BODY),
                TextColor(UiTheme::muted_cream()),
            ));
        });
}

fn footer_pill(parent: &mut ChildSpawnerCommands<'_>, label: &str, active: bool) {
    let (bg, border, text) = if active {
        (
            UiTheme::panel_bg().into(),
            BorderColor::from(UiTheme::ornate_gold()),
            UiTheme::muted_gold(),
        )
    } else {
        (
            UiTheme::panel_bg_deep().into(),
            BorderColor::from(UiTheme::panel_border()),
            UiTheme::body_dim(),
        )
    };
    parent
        .spawn((
            Node {
                box_sizing: BoxSizing::BorderBox,
                height: Val::Px(36.0),
                padding: UiRect::axes(Val::Px(12.0), Val::Px(6.0)),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                border: UiRect::all(Val::Px(1.0)),
                ..default()
            },
            BackgroundColor(bg),
            BorderColor::from(border),
        ))
        .with_children(|n| {
            n.spawn((
                Text::new(label),
                TextFont::from_font_size(UiTheme::FONT_CAPTION),
                TextColor(text),
            ));
        });
}
