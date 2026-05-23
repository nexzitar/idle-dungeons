//! Footer dock, stash filter row, and post-run rewards modal.

use bevy::prelude::*;
use bevy::text::{TextColor, TextFont};
use bevy::ui::FocusPolicy;

use crate::domain::run::RunSummary;
use crate::save::StashSortOrder;
use crate::ui::assets::UiPlaceholderImages;
use crate::ui::components::{
    AcceptRewardsButton, GearHubOpenButton, SkillShopOpenButton, SkipPlaybackButton,
    StashSortCycleButton, SummaryRewardsModalRoot, UiButtonPalette,
};
use crate::ui::inspect::InspectHint;
use crate::ui::interaction::UiClickAction;
use crate::ui::primitives::button::{spawn_button, UiButtonConfig, UiButtonVariant};
use crate::ui::primitives::modal::{spawn_modal_shell_with_handles, ModalShellConfig};
use crate::ui::primitives::reward_card::spawn_reward_loot_grid;
use crate::ui::primitives::scroll::spawn_scrollable_flex_column;
use crate::ui::primitives::section::spawn_framed_section_header;
use crate::ui::summary_panel::spawn_treasure_stat_row;
use crate::ui::theme::{body_text, caption_text, headline_text, UiTheme};

const REWARDS_MODAL_MARGIN_X: f32 = 28.0;
const REWARDS_MODAL_MARGIN_Y: f32 = 44.0;

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
                UiClickAction::CycleStashSort,
                p,
                InspectHint("Cycle stash sort order."),
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

/// Post-run rewards modal: loot grid + accept (non-dismissible backdrop).
pub fn spawn_summary_rewards_modal(
    parent: &mut ChildSpawnerCommands<'_>,
    summary: &RunSummary,
    _stash_sort: StashSortOrder,
    ph: &UiPlaceholderImages,
) {
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
            SummaryRewardsModalRoot,
        ))
        .insert(FocusPolicy::Block)
        .with_children(|layer| {
            spawn_modal_shell_with_handles(
                layer,
                ModalShellConfig {
                    backdrop_clicks_close: false,
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
                                    Val::Px(REWARDS_MODAL_MARGIN_X),
                                    Val::Px(REWARDS_MODAL_MARGIN_Y),
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
                                        min_width: Val::Px(480.0),
                                        max_width: Val::Px(640.0),
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
                                    dialog.spawn(headline_text("Run rewards"));
                                    spawn_framed_section_header(dialog, "TREASURE");
                                    spawn_treasure_stat_row(dialog, summary);
                                    if summary.loot.is_empty() {
                                        dialog.spawn(body_text(
                                            "No gear dropped this run — gold and salvage still apply.",
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
                                        dialog.spawn(caption_text(
                                            "Items go to your stash when you Accept. Use Gear on the footer to equip.",
                                        ));
                                        spawn_framed_section_header(dialog, "LOOT");
                                        spawn_scrollable_flex_column(dialog, Some(240.0), |scroll| {
                                            spawn_reward_loot_grid(
                                                scroll,
                                                &summary.loot,
                                                ph,
                                                summary.loot.len(),
                                            );
                                        });
                                    }
                                    let accept = spawn_button(
                                        dialog,
                                        UiButtonConfig {
                                            label: "\u{2713} Accept rewards",
                                            variant: UiButtonVariant::Primary,
                                            width: Val::Percent(100.0),
                                            height: Val::Px(48.0),
                                            font_size: UiTheme::FONT_SKILL_ACTIVE,
                                            text_color: Color::WHITE,
                                            flex_shrink: 0.0,
                                        },
                                    );
                                    dialog.commands_mut().entity(accept).insert((
                                        AcceptRewardsButton,
                                        UiClickAction::AcceptRewards,
                                        InspectHint(
                                            "Accept run rewards and return to camp.",
                                        ),
                                    ));
                                });
                        });
                },
            );
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
                    let start = spawn_button(
                        right,
                        UiButtonConfig {
                            label: "\u{2694} START RUN",
                            variant: UiButtonVariant::Primary,
                            width: Val::Px(220.0),
                            height: Val::Px(52.0),
                            font_size: UiTheme::FONT_STRONG,
                            text_color: Color::WHITE,
                            flex_shrink: 0.0,
                        },
                    );
                    right.commands_mut().entity(start).insert((
                        crate::ui::components::StartRunButton,
                        UiClickAction::StartRun,
                        InspectHint("Begin a seeded run with your current build."),
                    ));
                }
                FooterMode::DelvePlayback => {
                    let skip = spawn_button(
                        right,
                        UiButtonConfig {
                            label: "Skip to results",
                            variant: UiButtonVariant::PanelSecondary,
                            width: Val::Px(220.0),
                            height: Val::Px(44.0),
                            font_size: UiTheme::FONT_SKILL_DIM,
                            text_color: UiTheme::muted_cream(),
                            flex_shrink: 0.0,
                        },
                    );
                    right.commands_mut().entity(skip).insert((
                        SkipPlaybackButton,
                        UiClickAction::SkipPlayback,
                        InspectHint("Jump to the run summary."),
                    ));
                }
                FooterMode::Summary => {}
                }
            });
        });
}

fn footer_gear_hub_button(parent: &mut ChildSpawnerCommands<'_>) {
    let gear = spawn_button(
        parent,
        UiButtonConfig {
            label: "\u{2692} Gear",
            variant: UiButtonVariant::PanelSecondary,
            width: Val::Px(92.0),
            height: Val::Px(40.0),
            font_size: UiTheme::FONT_BODY,
            text_color: UiTheme::muted_cream(),
            flex_shrink: 0.0,
        },
    );
    parent.commands_mut().entity(gear).insert((
        GearHubOpenButton,
        UiClickAction::OpenGearHub,
        InspectHint("Loadout and stash."),
    ));
}

fn footer_skill_shop_button(parent: &mut ChildSpawnerCommands<'_>) {
    let skills = spawn_button(
        parent,
        UiButtonConfig {
            label: "\u{1F4DA} Skills",
            variant: UiButtonVariant::PanelSecondary,
            width: Val::Px(104.0),
            height: Val::Px(40.0),
            font_size: UiTheme::FONT_BODY,
            text_color: UiTheme::muted_cream(),
            flex_shrink: 0.0,
        },
    );
    parent.commands_mut().entity(skills).insert((
        SkillShopOpenButton,
        UiClickAction::OpenSkillShop,
        InspectHint("Buy skills with gold."),
    ));
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
