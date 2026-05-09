//! Three-column mockup shell: ornate panels, centered header stats, footer dock.

use bevy::prelude::*;
use bevy::ui::{FocusPolicy, RelativeCursorPosition};
use bevy::text::{Justify, TextColor, TextFont, TextLayout};

use crate::domain::dungeon::RoomKind;
use crate::domain::party::PartyHeroKind;
use crate::domain::progression::MetaProgression;
use crate::domain::progression::PARTY_SLOT_2_UNLOCK_DEPTH;
use crate::domain::items::GearSlot;
use crate::domain::run::{RunOutcome, RunSummary, DEFAULT_RUN_MAX_DEPTH, DEFAULT_RUN_SEED};
use crate::app::PLAYBACK_SPEED_STEPS;
use crate::domain::skills::{skill_definition, SkillKind};
use crate::save::StashSortOrder;
use crate::ui::components::{
    GearHubOpenButton, HeroNameDisplayText,
    HeroNameEditButton, PlaybackAggroArrowLine, PlaybackAggroArrowText, PlaybackAllyCastFill,
    PlaybackAllyCdFill, PlaybackAllyPortraitBlock, PlaybackCaptionText, PlaybackCombatLogPanel,
    PlaybackDepthText, PlaybackDmgMeterEnemyFill, PlaybackDmgMeterEnemyValue,
    PlaybackDmgMeterLeadFill, PlaybackDmgMeterLeadValue, PlaybackDmgMeterPartnerFill,
    PlaybackDmgMeterPartnerRow, PlaybackDmgMeterPartnerValue, PlaybackEnemyBarFill,
    PlaybackEnemyDebuffLine, PlaybackEnemyNameText, PlaybackEnemyPortraitBlock, PlaybackFoeCastFill,
    PlaybackFoeCdFill, PlaybackHeroBarFill, PlaybackHeroDebuffLine, PlaybackLeadCastFill,
    PlaybackLeadCdFill, PlaybackLeadPortraitBlock, PlaybackLogScrollRegion, PlaybackLogText,
    PlaybackAllyBarFill, PlaybackProgressBarFill, PlaybackProgressLabel, PlaybackRoomKindText,
    PlaybackTheaterFloatLayer, PlaybackSpeedDecButton, PlaybackSpeedIncButton,
    PlaybackSpeedValueText, SkillShopOpenButton,
    PlaybackCombatLogToggleLabel, ResetProgressButton, SettingsButton, SettingsModalBackdrop,
    SettingsModalCloseButton, SettingsModalRoot, SkillSlotButton, SkipPlaybackButton,
    StashSortCycleButton, ToggleCombatLogButton, TopBarField, UiButtonPalette, UiScrollContent,
    UiScrollRegion, UiScrollState, UiTooltip,
};
use crate::ui::placeholder_graphics::UiPlaceholderImages;
use crate::ui::theme::{
    body_text, caption_text, format_item_affix_lines, format_item_stat_summary, headline_text,
    log_line_present, rarity_color, section_title, UiTheme,
};
use crate::ui::widgets::spawn_scrollable_log;

fn ornate_shell(content: impl FnOnce(&mut ChildSpawnerCommands<'_>)) -> impl FnOnce(&mut ChildSpawnerCommands<'_>) {
    move |parent: &mut ChildSpawnerCommands<'_>| {
        parent
            .spawn((
            Node {
                box_sizing: BoxSizing::BorderBox,
                width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    min_height: Val::Px(0.0),
                    min_width: Val::Px(0.0),
                    flex_direction: FlexDirection::Column,
                    padding: UiRect::all(Val::Px(3.0)),
                    border: UiRect::all(Val::Px(2.0)),
                    overflow: Overflow::clip_y(),
                    ..default()
            },
            BackgroundColor(UiTheme::panel_bg_deep().into()),
            BorderColor::from(UiTheme::ornate_gold())
        ))
            .insert(FocusPolicy::Pass)
            .with_children(|ornate| {
                ornate
                    .spawn((
            Node {
                box_sizing: BoxSizing::BorderBox,
                width: Val::Percent(100.0),
                            flex_grow: 1.0,
                            flex_shrink: 1.0,
                            min_height: Val::Px(0.0),
                            min_width: Val::Px(0.0),
                            flex_direction: FlexDirection::Column,
                            padding: UiRect::all(Val::Px(UiTheme::PANEL_INSET)),
                            row_gap: Val::Px(8.0),
                            align_items: AlignItems::Stretch,
                            overflow: Overflow::clip_y(),
                            ..default()
            },
            BackgroundColor(UiTheme::panel_bg().into()),
            BorderColor::from(UiTheme::panel_border_inner())
        ))
                    .with_children(content);
            });
    }
}

/// Fills remaining column height; scrolls when content exceeds the panel.
///
/// When `min_viewport_height_px` is set, guarantees a minimum viewport height so flex layout
/// does not collapse empty (e.g. run rewards loot list).
fn spawn_column_flex_scroll(
    parent: &mut ChildSpawnerCommands<'_>,
    min_viewport_height_px: Option<f32>,
    content: impl FnOnce(&mut ChildSpawnerCommands<'_>),
) {
    parent
        .spawn((
            Node {
                box_sizing: BoxSizing::BorderBox,
                width: Val::Percent(100.0),
                flex_grow: 1.0,
                flex_shrink: 1.0,
                min_height: min_viewport_height_px
                    .map(Val::Px)
                    .unwrap_or(Val::Px(0.0)),
                position_type: PositionType::Relative,
                overflow: Overflow::clip_y(),
                ..default()
            },
            FocusPolicy::Pass,
            RelativeCursorPosition::default(),
            UiScrollState::default(),
            UiScrollRegion,
        ))
        .with_children(|vp| {
            vp.spawn((
                Node {
                box_sizing: BoxSizing::BorderBox,
                position_type: PositionType::Absolute,
                        left: Val::Px(0.0),
                        right: Val::Px(0.0),
                        top: Val::Px(0.0),
                        flex_direction: FlexDirection::Column,
                        row_gap: Val::Px(8.0),
                        align_items: AlignItems::Stretch,
                        ..default()
            },
                UiScrollContent,
            ))
            .with_children(content);
        });
}

/// Combat log during playback — pins scroll to the latest line when content grows.
fn spawn_playback_combat_log_scroll(
    parent: &mut ChildSpawnerCommands<'_>,
    content: impl FnOnce(&mut ChildSpawnerCommands<'_>),
) {
    parent
        .spawn((
            Node {
                box_sizing: BoxSizing::BorderBox,
                width: Val::Percent(100.0),
                    flex_grow: 1.0,
                    flex_shrink: 1.0,
                    min_height: Val::Px(0.0),
                    position_type: PositionType::Relative,
                    overflow: Overflow::clip_y(),
                    ..default()
            },
            FocusPolicy::Pass,
            RelativeCursorPosition::default(),
            UiScrollState::default(),
            UiScrollRegion,
            PlaybackLogScrollRegion,
        ))
        .with_children(|vp| {
            vp.spawn((
                Node {
                box_sizing: BoxSizing::BorderBox,
                position_type: PositionType::Absolute,
                        left: Val::Px(0.0),
                        right: Val::Px(0.0),
                        top: Val::Px(0.0),
                        flex_direction: FlexDirection::Column,
                        row_gap: Val::Px(8.0),
                        align_items: AlignItems::Stretch,
                        ..default()
            },
                UiScrollContent,
            ))
            .with_children(content);
        });
}

/// Full-screen centered settings dialog (reset progress). Spawn as a child of [`UiRoot`].
pub fn spawn_settings_modal(parent: &mut ChildSpawnerCommands<'_>) {
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
            SettingsModalRoot,
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
                SettingsModalBackdrop,
                backdrop_pal,
                UiTooltip::txt("Click the dimmed backdrop to close settings (same as Close)."),
            ));
            layer
                .spawn((
            Node {
                box_sizing: BoxSizing::BorderBox,
                position_type: PositionType::Absolute,
                        left: Val::Percent(50.0),
                        top: Val::Percent(50.0),
                        margin: UiRect {
                            left: Val::Px(-170.0),
                            top: Val::Px(-155.0),
                            right: Val::Auto,
                            bottom: Val::Auto,
                        },
                        width: Val::Px(340.0),
                        padding: UiRect::all(Val::Px(UiTheme::PAD_ROOT)),
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Stretch,
                        row_gap: Val::Px(UiTheme::PANEL_INSET),
                        border: UiRect::all(Val::Px(2.0)),
                        ..default()
            },
            BackgroundColor(UiTheme::panel_bg_deep().into()),
            BorderColor::from(UiTheme::ornate_gold())
        ))
                .with_children(|dialog| {
                    dialog.spawn(headline_text("Settings"));
                    dialog.spawn(caption_text(
                        "Playback speed is controlled from the header (arrows next to Speed).",
                    ));
                    dialog.spawn(section_title("Save"));
                    let reset_pal = UiButtonPalette::salvage();
                    dialog
                        .spawn((
                            Node {
                box_sizing: BoxSizing::BorderBox,
                width: Val::Percent(100.0),
                                    min_height: Val::Px(44.0),
                                    justify_content: JustifyContent::Center,
                                    align_items: AlignItems::Center,
                                    border: UiRect::all(Val::Px(1.0)),
                                    ..default()
            },
            Button,
            BackgroundColor(reset_pal.idle_bg.into()),
            BorderColor::from(reset_pal.idle_border),
                            ResetProgressButton,
                            reset_pal,
                            UiTooltip::txt(
                                "Permanently wipe local save data—hero, stash, gold, upgrades—and return to a fresh profile.",
                            ),
                        ))
                        .with_children(|b| {
                            b.spawn((
                Text::new("Reset all progress"),
                TextFont::from_font_size(UiTheme::FONT_BODY),
                TextColor(UiTheme::body()),
            ));
                        });
                    let close_pal = UiButtonPalette::panel_secondary();
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
                            SettingsModalCloseButton,
                            close_pal,
                            UiTooltip::txt("Close the settings dialog without applying other changes."),
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

fn spawn_playback_speed_controls(parent: &mut ChildSpawnerCommands<'_>, initial_mult: f32) {
    parent
        .spawn((
            Node {
                box_sizing: BoxSizing::BorderBox,
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                min_width: Val::Px(128.0),
                ..default()
            },
            Interaction::default(),
            UiTooltip::txt(
                "Delve playback speed. ‹ › step through 1×, 2×, 3×, 5×, and 10×.",
            ),
        ))
        .with_children(|wrap| {
            wrap.spawn(Node {
                box_sizing: BoxSizing::BorderBox,
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                column_gap: Val::Px(5.0),
                ..default()
            })
            .with_children(|hdr| {
                hdr.spawn((
                    Text::new("\u{21BB}"),
                    TextFont::from_font_size(UiTheme::FONT_MICRO),
                    TextColor(UiTheme::body_dim()),
                ));
                hdr.spawn((
                    Text::new("Speed"),
                    TextFont::from_font_size(UiTheme::FONT_MICRO),
                    TextColor(UiTheme::body_dim()),
                ));
            });
            let p_dec = UiButtonPalette::panel_outlined();
            let p_inc = UiButtonPalette::panel_outlined();
            wrap
                .spawn(Node {
                    box_sizing: BoxSizing::BorderBox,
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    column_gap: Val::Px(6.0),
                    ..default()
                })
                .with_children(|row| {
                    row.spawn((
                        Node {
                            box_sizing: BoxSizing::BorderBox,
                            min_width: Val::Px(36.0),
                            min_height: Val::Px(32.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            border: UiRect::all(Val::Px(1.0)),
                            ..default()
                        },
                        Button,
                        BackgroundColor(p_dec.idle_bg.into()),
                        BorderColor::from(p_dec.idle_border),
                        PlaybackSpeedDecButton,
                        p_dec,
                        UiTooltip::txt("Slower delve playback (steps down to 1×)."),
                    ))
                    .with_children(|b| {
                        b.spawn((
                            Text::new("\u{2039}"),
                            TextFont::from_font_size(UiTheme::FONT_LABEL),
                            TextColor(UiTheme::body()),
                        ));
                    });
                    row.spawn((
                        Text::new(fmt_speed_label(initial_mult)),
                        TextFont::from_font_size(UiTheme::FONT_BODY),
                        TextColor(UiTheme::body()),
                        PlaybackSpeedValueText,
                    ));
                    row.spawn((
                        Node {
                            box_sizing: BoxSizing::BorderBox,
                            min_width: Val::Px(36.0),
                            min_height: Val::Px(32.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            border: UiRect::all(Val::Px(1.0)),
                            ..default()
                        },
                        Button,
                        BackgroundColor(p_inc.idle_bg.into()),
                        BorderColor::from(p_inc.idle_border),
                        PlaybackSpeedIncButton,
                        p_inc,
                        UiTooltip::txt("Faster delve playback (steps up to 10×)."),
                    ))
                    .with_children(|b| {
                        b.spawn((
                            Text::new("\u{203A}"),
                            TextFont::from_font_size(UiTheme::FONT_LABEL),
                            TextColor(UiTheme::body()),
                        ));
                    });
                });
        });
}

pub fn spawn_mockup_header(
    parent: &mut ChildSpawnerCommands<'_>,
    ph: &UiPlaceholderImages,
    gold: u32,
    salvage: u32,
    skill_slots: usize,
    skill_cap: usize,
    depth_label: &str,
    speed_mult: f32,
) {
    parent
        .spawn((
            Node {
                box_sizing: BoxSizing::BorderBox,
                width: Val::Percent(100.0),
                min_height: Val::Px(68.0),
                flex_shrink: 0.0,
                padding: UiRect::axes(Val::Px(UiTheme::PAD_ROOT), Val::Px(UiTheme::PAD_BAR_Y)),
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::SpaceBetween,
                border: UiRect::bottom(Val::Px(2.0)),
                ..default()
            },
            BackgroundColor(UiTheme::panel_bg_deep().into()),
            BorderColor::from(UiTheme::ornate_gold())
        ))
        .with_children(|row| {
            row.spawn((
                Text::new("Delvers"),
                TextFont::from_font_size(UiTheme::FONT_HEADLINE),
                TextColor(UiTheme::muted_gold()),
            ));
            row.spawn(Node {
                box_sizing: BoxSizing::BorderBox,
                flex_grow: 1.0,
                    flex_direction: FlexDirection::Row,
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    flex_wrap: FlexWrap::Wrap,
                    column_gap: Val::Px(24.0),
                    row_gap: Val::Px(4.0),
                    ..default()
            })
            .with_children(|center| {
                resource_chip(
                    center,
                    Some(ph.gold_coin.clone()),
                    "\u{2694}",
                    "Gold",
                    TopBarField::Gold,
                    format!("{gold}"),
                    "Gold pays for caravan upgrades. Earn it from dungeon runs when you accept rewards.",
                );
                resource_chip(
                    center,
                    Some(ph.salvage_shard.clone()),
                    "*",
                    "Salvage",
                    TopBarField::Salvage,
                    format!("{salvage}"),
                    "Salvage currency from breaking down spare gear in your stash.",
                );
                resource_chip(
                    center,
                    Some(ph.stat_chip.clone()),
                    "\u{2726}",
                    "Skills",
                    TopBarField::SkillSlots,
                    format!("{skill_slots}/{skill_cap}"),
                    "Equipped skill slots in use versus how many are unlocked for your hero.",
                );
                resource_chip(
                    center,
                    Some(ph.stat_chip.clone()),
                    "\u{2022}",
                    "Depth",
                    TopBarField::Depth,
                    depth_label.to_string(),
                    "Deepest floor reached on the latest run (or dash when not applicable).",
                );
                spawn_playback_speed_controls(center, speed_mult);
            });
            row.spawn(Node {
                box_sizing: BoxSizing::BorderBox,
                flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    column_gap: Val::Px(8.0),
                    ..default()
            })
            .with_children(|btn_row| {
                let p = UiButtonPalette::panel_outlined();
                btn_row
                    .spawn((
                        Node {
                box_sizing: BoxSizing::BorderBox,
                min_width: Val::Px(100.0),
                                height: Val::Px(38.0),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                border: UiRect::all(Val::Px(1.0)),
                                ..default()
            },
            Button,
            BackgroundColor(p.idle_bg.into()),
            BorderColor::from(p.idle_border),
                        SettingsButton,
                        p,
                        UiTooltip::txt(
                            "Open settings: reset all progress or review this note.",
                        ),
                    ))
                    .with_children(|btn| {
                        btn.spawn((
                Text::new("\u{2699} Settings"),
                TextFont::from_font_size(UiTheme::FONT_COMPACT),
                TextColor(UiTheme::muted_cream()),
            ));
                    });
            });
        });
}

/// Left-column settings control for the title screen (same behavior as header [`SettingsButton`]).
pub fn title_settings_menu_button(parent: &mut ChildSpawnerCommands<'_>) {
    let p = UiButtonPalette::panel_outlined();
    parent
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
            BackgroundColor(p.idle_bg.into()),
            BorderColor::from(p.idle_border),
            SettingsButton,
            p,
            UiTooltip::txt("Open settings: reset all progress and saves."),
        ))
        .with_children(|b| {
            b.spawn((
                Text::new("Settings"),
                TextFont::from_font_size(UiTheme::FONT_BODY),
                TextColor(Color::WHITE),
            ));
        });
}

pub(crate) fn fmt_speed_label(mult: f32) -> String {
    const EPS: f32 = 1e-3;
    if PLAYBACK_SPEED_STEPS
        .iter()
        .any(|s| (mult - *s).abs() < EPS)
    {
        return format!("{}x", mult as i32);
    }
    format!("{mult:.1}x")
}

fn resource_chip(
    parent: &mut ChildSpawnerCommands<'_>,
    icon_tex: Option<Handle<Image>>,
    icon_fallback: &'static str,
    label: &'static str,
    field: TopBarField,
    value: String,
    tooltip: &'static str,
) {
    parent
        .spawn((
            Node {
                box_sizing: BoxSizing::BorderBox,
                flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    min_width: Val::Px(68.0),
                    ..default()
            },
            Interaction::default(),
            UiTooltip::txt(tooltip),
        ))
        .with_children(|col| {
            col.spawn(Node {
                box_sizing: BoxSizing::BorderBox,
                flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    column_gap: Val::Px(5.0),
                    ..default()
            })
            .with_children(|hdr| {
                if let Some(h) = icon_tex {
                    hdr.spawn((
                        Node {
                            box_sizing: BoxSizing::BorderBox,
                            width: Val::Px(18.0),
                            height: Val::Px(18.0),
                            flex_shrink: 0.0,
                            ..default()
                        },
                        ImageNode {
                            image: h,
                            color: Color::WHITE,
                            ..default()
                        },
                    ));
                } else {
                    hdr.spawn((
                Text::new(icon_fallback),
                TextFont::from_font_size(UiTheme::FONT_MICRO),
                TextColor(UiTheme::body_dim()),
            ));
                }
                hdr.spawn((
                Text::new(label),
                TextFont::from_font_size(UiTheme::FONT_MICRO),
                TextColor(UiTheme::body_dim()),
            ));
            });
            col.spawn((
                Text::new(value),
                TextFont::from_font_size(UiTheme::FONT_BODY),
                TextColor(UiTheme::body()),
                field,
            ));
        });
}

pub fn spawn_three_column_row(parent: &mut ChildSpawnerCommands<'_>, f: impl FnOnce(&mut ChildSpawnerCommands<'_>)) {
    parent
        .spawn(Node {
                box_sizing: BoxSizing::BorderBox,
                width: Val::Percent(100.0),
                flex_grow: 1.0,
                flex_basis: Val::Px(0.0),
                min_height: Val::Px(0.0),
                flex_direction: FlexDirection::Row,
                column_gap: Val::Px(10.0),
                align_items: AlignItems::Stretch,
                ..default()
            })
        .with_children(f);
}

pub fn spawn_ornate_column(
    parent: &mut ChildSpawnerCommands<'_>,
    flex: f32,
    inner: impl FnOnce(&mut ChildSpawnerCommands<'_>),
) {
    parent
        .spawn(Node {
                box_sizing: BoxSizing::BorderBox,
                flex_grow: flex,
                flex_basis: Val::Px(0.0),
                min_width: Val::Px(200.0),
                min_height: Val::Px(0.0),
                flex_direction: FlexDirection::Column,
                overflow: Overflow::clip_y(),
                ..default()
            })
        .with_children(ornate_shell(inner));
}

fn panel_title_centered(text: impl Into<String>) -> impl Bundle {
    (
        Text::new(text.into()),
        TextLayout::new_with_justify(Justify::Center),
        TextFont::from_font_size(UiTheme::FONT_SECTION),
        TextColor(UiTheme::muted_cream()),
    )
}

fn spawn_hero_name_row(parent: &mut ChildSpawnerCommands<'_>, slot: u8, allow_rename: bool) {
    parent
        .spawn(Node {
                box_sizing: BoxSizing::BorderBox,
                width: Val::Percent(100.0),
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::SpaceBetween,
                column_gap: Val::Px(8.0),
                padding: UiRect::vertical(Val::Px(2.0)),
                ..default()
            })
        .with_children(|r| {
            r.spawn((
                Text::new(""),
                TextFont::from_font_size(UiTheme::FONT_BODY),
                TextColor(UiTheme::muted_cream()),
                HeroNameDisplayText { slot },
            ));
            if allow_rename {
                let p = UiButtonPalette::panel_outlined();
                r.spawn((
                    Node {
                box_sizing: BoxSizing::BorderBox,
                min_width: Val::Px(72.0),
                            min_height: Val::Px(30.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            padding: UiRect::horizontal(Val::Px(6.0)),
                            border: UiRect::all(Val::Px(1.0)),
                            ..default()
            },
            Button,
            BackgroundColor(p.idle_bg.into()),
            BorderColor::from(p.idle_border),
                    HeroNameEditButton { slot },
                    p,
                    UiTooltip::txt(
                        "Rename this hero: type, Enter to save, Esc to cancel.".to_string(),
                    ),
                ))
                .with_children(|b| {
                    b.spawn((
                Text::new("Rename"),
                TextFont::from_font_size(UiTheme::FONT_LABEL),
                TextColor(UiTheme::body()),
            ));
                });
            }
        });
}

pub fn spawn_hero_column_mockup(
    parent: &mut ChildSpawnerCommands<'_>,
    ph: &UiPlaceholderImages,
    lead: &crate::domain::hero::HeroProfile,
    partner: Option<&crate::domain::hero::HeroProfile>,
    party_slots_unlocked: usize,
    loadout_lines: &[String],
    skill_slots_interactive: bool,
    allow_rename: bool,
) {
    let inner = move |p: &mut ChildSpawnerCommands<'_>| {
        p.spawn(panel_title_centered("PARTY"));
        spawn_column_flex_scroll(p, None, move |body| {
            body.spawn(section_title("Lead"));
            spawn_hero_name_row(body, 0, allow_rename);
            let stats = lead.derived_stats();
            body.spawn(section_title("Vitals"));
            stat_line_row(body, "Max Health", stats.max_health);
            stat_line_row(body, "Damage", stats.damage);
            stat_line_row(body, "Armor", stats.armor);
            stat_line_row(body, "Healing", stats.healing_power);
            if !loadout_lines.is_empty() {
                body.spawn(caption_text("Loadout"));
                for line in loadout_lines {
                    body.spawn(caption_text(line.clone()));
                }
            }
            body.spawn(section_title("Skills"));
            if skill_slots_interactive {
                body.spawn(caption_text("Click a slot to open the skill book."));
            }
            skill_slot_row(
                body,
                ph,
                lead,
                skill_slots_interactive,
                PartyHeroKind::Lead,
            );

            if party_slots_unlocked >= 2 {
                body.spawn(section_title("Ally"));
                if let Some(phero) = partner {
                    spawn_hero_name_row(body, 1, allow_rename);
                    body.spawn(section_title("Vitals"));
                    let pst = phero.derived_stats();
                    stat_line_row(body, "Max Health", pst.max_health);
                    stat_line_row(body, "Damage", pst.damage);
                    stat_line_row(body, "Armor", pst.armor);
                    stat_line_row(body, "Healing", pst.healing_power);
                    body.spawn(section_title("Skills"));
                    if skill_slots_interactive {
                        body.spawn(caption_text(
                            "Ally has their own skills — click a slot to change them.",
                        ));
                    }
                    skill_slot_row(
                        body,
                        ph,
                        phero,
                        skill_slots_interactive,
                        PartyHeroKind::Partner,
                    );
                } else {
                    body.spawn(caption_text(
                        "Companion will appear after rewards sync (new unlock).",
                    ));
                }
            } else {
                body.spawn(caption_text(format!(
                    "Reach depth {} on a run to unlock a second party hero.",
                    PARTY_SLOT_2_UNLOCK_DEPTH
                )));
            }
        });
    };
    inner(parent);
}

fn stat_line_row(parent: &mut ChildSpawnerCommands<'_>, label: &str, value: impl std::fmt::Display) {
    parent
        .spawn(Node {
                box_sizing: BoxSizing::BorderBox,
                width: Val::Percent(100.0),
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::Center,
                padding: UiRect::vertical(Val::Px(2.0)),
                ..default()
            })
        .with_children(|r| {
            r.spawn((
                Text::new(format!("\u{25C8} {label}")),
                TextFont::from_font_size(UiTheme::FONT_COMPACT),
                TextColor(UiTheme::body_dim()),
            ));
            r.spawn((
                Text::new(format!("{value}")),
                TextFont::from_font_size(UiTheme::FONT_BODY),
                TextColor(UiTheme::muted_cream()),
            ));
        });
}

fn skill_slot_placeholder_handle(
    hero: &crate::domain::hero::HeroProfile,
    slot: usize,
    unlocked: bool,
    ph: &UiPlaceholderImages,
) -> Handle<Image> {
    if !unlocked {
        return ph.skill_locked.clone();
    }
    match hero.equipped_skills.get(slot).copied().flatten() {
        None => ph.skill_empty.clone(),
        Some(id) => {
            if skill_definition(id).kind == SkillKind::Passive {
                ph.skill_passive.clone()
            } else {
                ph.skill_active.clone()
            }
        }
    }
}

fn skill_slot_row(
    parent: &mut ChildSpawnerCommands<'_>,
    ph: &UiPlaceholderImages,
    hero: &crate::domain::hero::HeroProfile,
    skill_slots_interactive: bool,
    sheet: PartyHeroKind,
) {
    parent
        .spawn(Node {
                box_sizing: BoxSizing::BorderBox,
                flex_direction: FlexDirection::Row,
                column_gap: Val::Px(8.0),
                flex_wrap: FlexWrap::Wrap,
                ..default()
            })
        .with_children(|row| {
            let cap = hero.equipped_skills.len().max(6);
            for i in 0..cap {
                let unlocked = i < hero.unlocked_skill_slots;
                if unlocked && skill_slots_interactive {
                    let label = hero
                        .equipped_skills
                        .get(i)
                        .and_then(|s| *s)
                        .map(|sk| skill_definition(sk).name.to_string())
                        .unwrap_or_else(|| format!("Slot {}", i + 1));
                    let tip = hero
                        .equipped_skills
                        .get(i)
                        .and_then(|s| *s)
                        .map(|sk| {
                            let d = skill_definition(sk);
                            format!("{}\n{}", d.name, d.description)
                        })
                        .unwrap_or_else(|| {
                            "Open the skill book to assign or clear this slot (no duplicates across slots)."
                                .to_string()
                        });
                    let p = UiButtonPalette::skill_slot_chip();
                    let icon = skill_slot_placeholder_handle(hero, i, true, ph);
                    row.spawn((
                        Node {
                box_sizing: BoxSizing::BorderBox,
                min_width: Val::Px(118.0),
                                min_height: Val::Px(48.0),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                padding: UiRect::horizontal(Val::Px(4.0)),
                                border: UiRect::all(Val::Px(1.0)),
                                ..default()
            },
            Button,
            BackgroundColor(p.idle_bg.into()),
            BorderColor::from(p.idle_border),
                        SkillSlotButton { slot: i, kind: sheet },
                        p,
                        UiTooltip::txt(tip),
                    ))
                    .with_children(|s| {
                        s.spawn(Node {
                box_sizing: BoxSizing::BorderBox,
                width: Val::Percent(100.0),
                                height: Val::Px(44.0),
                                flex_direction: FlexDirection::Row,
                                align_items: AlignItems::Center,
                                column_gap: Val::Px(6.0),
                                ..default()
            })
                        .with_children(|inner| {
                            inner.spawn((
                                Node {
                                    box_sizing: BoxSizing::BorderBox,
                                    width: Val::Px(22.0),
                                    height: Val::Px(22.0),
                                    flex_shrink: 0.0,
                                    ..default()
                                },
                                ImageNode {
                                    image: icon,
                                    color: Color::WHITE,
                                    ..default()
                                },
                            ));
                            inner.spawn((
                Text::new(label),
                TextFont::from_font_size(UiTheme::FONT_LABEL),
                TextColor(UiTheme::body()),
            ));
                        });
                    });
                    continue;
                }

                let idle_tip = if !unlocked {
                    "Locked skill slot. Gain delve progress milestones to unlock up to six slots."
                        .to_string()
                } else {
                    hero.equipped_skills
                        .get(i)
                        .and_then(|s| *s)
                        .map(|sk| {
                            let d = skill_definition(sk);
                            format!("{}\n{}", d.name, d.description)
                        })
                        .unwrap_or_else(|| "Empty skill slot.".to_string())
                };
                let icon = skill_slot_placeholder_handle(hero, i, unlocked, ph);
                row.spawn((
                    Node {
                box_sizing: BoxSizing::BorderBox,
                width: Val::Px(52.0),
                            height: Val::Px(52.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            border: UiRect::all(Val::Px(1.0)),
                            ..default()
            },
            BackgroundColor(if unlocked {
                            UiTheme::panel_bg_deep().into()
                        } else {
                            Color::srgba(0.06, 0.06, 0.07, 1.0).into()
                        }),
            BorderColor::from(if unlocked {
                            UiTheme::ornate_gold()
                        } else {
                            UiTheme::panel_border()
                        }),
                    Interaction::default(),
                    UiTooltip::txt(idle_tip),
                ))
                .with_children(|s| {
                    s.spawn((
                        Node {
                            box_sizing: BoxSizing::BorderBox,
                            width: Val::Px(36.0),
                            height: Val::Px(36.0),
                            flex_shrink: 0.0,
                            ..default()
                        },
                        ImageNode {
                            image: icon,
                            color: Color::WHITE,
                            ..default()
                        },
                    ));
                });
            }
        });
}

pub fn mockup_gear_cards(
    parent: &mut ChildSpawnerCommands<'_>,
    profile: &crate::app::ProfileState,
    ph: &UiPlaceholderImages,
) {
    for (label, slot) in [
        ("Weapon", crate::domain::items::GearSlot::Weapon),
        ("Armor", crate::domain::items::GearSlot::Armor),
        ("Trinket", crate::domain::items::GearSlot::Trinket),
    ] {
        let item = profile.profile.hero.equipped_item(slot);
        let tip = if let Some(item) = item {
            let aff = format_item_affix_lines(item);
            if aff.is_empty() {
                format!(
                    "{}\n{:?}\n{}",
                    item.name,
                    item.rarity,
                    format_item_stat_summary(item)
                )
            } else {
                format!(
                    "{}\n{:?}\n{}\n{}",
                    item.name,
                    item.rarity,
                    format_item_stat_summary(item),
                    aff
                )
            }
        } else {
            format!(
                "No {label} equipped yet. Loot gear on runs and equip it from the Inventory tab."
            )
        };
        parent
            .spawn((
                Node {
                box_sizing: BoxSizing::BorderBox,
                width: Val::Percent(100.0),
                        flex_direction: FlexDirection::Row,
                        column_gap: Val::Px(10.0),
                        padding: UiRect::all(Val::Px(8.0)),
                        border: UiRect::all(Val::Px(1.0)),
                        margin: UiRect::bottom(Val::Px(8.0)),
                        ..default()
            },
            BackgroundColor(UiTheme::panel_bg_deep().into()),
            BorderColor::from(UiTheme::ornate_gold()),
                Interaction::default(),
                UiTooltip::txt(tip),
            ))
            .with_children(|card| {
                card.spawn((
            Node {
                box_sizing: BoxSizing::BorderBox,
                width: Val::Px(56.0),
                        height: Val::Px(56.0),
                        flex_shrink: 0.0,
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        border: UiRect::all(Val::Px(1.0)),
                        ..default()
            },
            BackgroundColor(Color::srgb(0.18, 0.16, 0.2).into()),
            BorderColor::from(UiTheme::panel_border_inner())
        ))
                .with_children(|icon_cell| {
                    let tint = match slot {
                        GearSlot::Weapon => Color::srgb(1.0, 0.72, 0.45),
                        GearSlot::Armor => Color::srgb(0.72, 0.82, 0.95),
                        GearSlot::Trinket => Color::srgb(0.85, 0.68, 1.0),
                    };
                    icon_cell.spawn((
                        Node {
                            box_sizing: BoxSizing::BorderBox,
                            width: Val::Px(44.0),
                            height: Val::Px(44.0),
                            flex_shrink: 0.0,
                            ..default()
                        },
                        ImageNode {
                            image: ph.item_generic.clone(),
                            color: tint,
                            ..default()
                        },
                    ));
                });
                card.spawn(Node {
                box_sizing: BoxSizing::BorderBox,
                flex_direction: FlexDirection::Column,
                        flex_grow: 1.0,
                        align_items: AlignItems::FlexStart,
                        row_gap: Val::Px(2.0),
                        ..default()
            })
                .with_children(|txt| {
                    txt.spawn(caption_text(label.to_string()));
                    if let Some(item) = item {
                        txt.spawn((
                Text::new(item.name.clone()),
                TextFont::from_font_size(UiTheme::FONT_BODY),
                TextColor(rarity_color(item.rarity)),
            ));
                        txt.spawn(caption_text(format_item_stat_summary(item)));
                        let aff = format_item_affix_lines(item);
                        if !aff.is_empty() {
                            txt.spawn(caption_text(aff));
                        }
                    } else {
                        txt.spawn(body_text("Empty slot"));
                    }
                });
            });
    }
}

pub fn room_kind_label(kind: RoomKind) -> &'static str {
    match kind {
        RoomKind::Monster => "Monster",
        RoomKind::Elite => "Elite",
        RoomKind::Boss => "Boss",
        RoomKind::Treasure => "Treasure",
        RoomKind::Shrine => "Shrine",
    }
}

fn playback_hero_bar(parent: &mut ChildSpawnerCommands<'_>, fill_pct: f32) {
    parent
        .spawn((
            Node {
                box_sizing: BoxSizing::BorderBox,
                width: Val::Percent(100.0),
                height: Val::Px(14.0),
                border: UiRect::all(Val::Px(1.0)),
                ..default()
            },
            BackgroundColor(UiTheme::void_black().into()),
            BorderColor::from(UiTheme::panel_border())
        ))
        .with_children(|bar| {
            bar.spawn((
                Node {
                box_sizing: BoxSizing::BorderBox,
                width: Val::Percent((fill_pct * 100.0).clamp(0.0, 100.0)),
                        height: Val::Percent(100.0),
                        ..default()
            },
            BackgroundColor(UiTheme::healing().into()),
                PlaybackHeroBarFill,
            ));
        });
}

fn playback_ally_bar(parent: &mut ChildSpawnerCommands<'_>, fill_pct: f32) {
    let tone = Color::srgb(0.38, 0.72, 0.92);
    parent
        .spawn((
            Node {
                box_sizing: BoxSizing::BorderBox,
                width: Val::Percent(100.0),
                height: Val::Px(14.0),
                border: UiRect::all(Val::Px(1.0)),
                ..default()
            },
            BackgroundColor(UiTheme::void_black().into()),
            BorderColor::from(UiTheme::panel_border())
        ))
        .with_children(|bar| {
            bar.spawn((
                Node {
                box_sizing: BoxSizing::BorderBox,
                width: Val::Percent((fill_pct * 100.0).clamp(0.0, 100.0)),
                        height: Val::Percent(100.0),
                        ..default()
            },
            BackgroundColor(tone.into()),
                PlaybackAllyBarFill,
            ));
        });
}

fn playback_enemy_bar(parent: &mut ChildSpawnerCommands<'_>, fill_pct: f32) {
    parent
        .spawn((
            Node {
                box_sizing: BoxSizing::BorderBox,
                width: Val::Percent(100.0),
                height: Val::Px(14.0),
                border: UiRect::all(Val::Px(1.0)),
                ..default()
            },
            BackgroundColor(UiTheme::void_black().into()),
            BorderColor::from(UiTheme::panel_border())
        ))
        .with_children(|bar| {
            bar.spawn((
                Node {
                box_sizing: BoxSizing::BorderBox,
                width: Val::Percent((fill_pct * 100.0).clamp(0.0, 100.0)),
                        height: Val::Percent(100.0),
                        ..default()
            },
            BackgroundColor(UiTheme::danger().into()),
                PlaybackEnemyBarFill,
            ));
        });
}

fn playback_cast_cd_stack_lead(parent: &mut ChildSpawnerCommands<'_>) {
    parent
        .spawn(Node {
            box_sizing: BoxSizing::BorderBox,
            width: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(2.0),
            ..default()
        })
        .with_children(|col| {
            col.spawn((
                Node {
                    box_sizing: BoxSizing::BorderBox,
                    width: Val::Percent(100.0),
                    height: Val::Px(5.0),
                    border: UiRect::all(Val::Px(1.0)),
                    ..default()
                },
                BackgroundColor(UiTheme::void_black().into()),
                BorderColor::from(UiTheme::panel_border()),
            ))
            .with_children(|track| {
                track.spawn((
                    Node {
                        box_sizing: BoxSizing::BorderBox,
                        width: Val::Percent(0.0),
                        height: Val::Percent(100.0),
                        ..default()
                    },
                    BackgroundColor(UiTheme::muted_gold().into()),
                    PlaybackLeadCastFill,
                ));
            });
            col.spawn((
                Node {
                    box_sizing: BoxSizing::BorderBox,
                    width: Val::Percent(100.0),
                    height: Val::Px(5.0),
                    border: UiRect::all(Val::Px(1.0)),
                    ..default()
                },
                BackgroundColor(UiTheme::void_black().into()),
                BorderColor::from(UiTheme::panel_border()),
            ))
            .with_children(|track| {
                track.spawn((
                    Node {
                        box_sizing: BoxSizing::BorderBox,
                        width: Val::Percent(0.0),
                        height: Val::Percent(100.0),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.28, 0.32, 0.42).into()),
                    PlaybackLeadCdFill,
                ));
            });
        });
}

fn playback_cast_cd_stack_ally(parent: &mut ChildSpawnerCommands<'_>) {
    let tone = Color::srgb(0.38, 0.72, 0.92);
    parent
        .spawn(Node {
            box_sizing: BoxSizing::BorderBox,
            width: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(2.0),
            ..default()
        })
        .with_children(|col| {
            col.spawn((
                Node {
                    box_sizing: BoxSizing::BorderBox,
                    width: Val::Percent(100.0),
                    height: Val::Px(5.0),
                    border: UiRect::all(Val::Px(1.0)),
                    ..default()
                },
                BackgroundColor(UiTheme::void_black().into()),
                BorderColor::from(UiTheme::panel_border()),
            ))
            .with_children(|track| {
                track.spawn((
                    Node {
                        box_sizing: BoxSizing::BorderBox,
                        width: Val::Percent(0.0),
                        height: Val::Percent(100.0),
                        ..default()
                    },
                    BackgroundColor(tone.into()),
                    PlaybackAllyCastFill,
                ));
            });
            col.spawn((
                Node {
                    box_sizing: BoxSizing::BorderBox,
                    width: Val::Percent(100.0),
                    height: Val::Px(5.0),
                    border: UiRect::all(Val::Px(1.0)),
                    ..default()
                },
                BackgroundColor(UiTheme::void_black().into()),
                BorderColor::from(UiTheme::panel_border()),
            ))
            .with_children(|track| {
                track.spawn((
                    Node {
                        box_sizing: BoxSizing::BorderBox,
                        width: Val::Percent(0.0),
                        height: Val::Percent(100.0),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.22, 0.36, 0.48).into()),
                    PlaybackAllyCdFill,
                ));
            });
        });
}

fn playback_cast_cd_stack_foe(parent: &mut ChildSpawnerCommands<'_>) {
    parent
        .spawn(Node {
            box_sizing: BoxSizing::BorderBox,
            width: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(2.0),
            ..default()
        })
        .with_children(|col| {
            col.spawn((
                Node {
                    box_sizing: BoxSizing::BorderBox,
                    width: Val::Percent(100.0),
                    height: Val::Px(5.0),
                    border: UiRect::all(Val::Px(1.0)),
                    ..default()
                },
                BackgroundColor(UiTheme::void_black().into()),
                BorderColor::from(UiTheme::panel_border()),
            ))
            .with_children(|track| {
                track.spawn((
                    Node {
                        box_sizing: BoxSizing::BorderBox,
                        width: Val::Percent(0.0),
                        height: Val::Percent(100.0),
                        ..default()
                    },
                    BackgroundColor(UiTheme::danger().mix(&Color::WHITE, 0.25).into()),
                    PlaybackFoeCastFill,
                ));
            });
            col.spawn((
                Node {
                    box_sizing: BoxSizing::BorderBox,
                    width: Val::Percent(100.0),
                    height: Val::Px(5.0),
                    border: UiRect::all(Val::Px(1.0)),
                    ..default()
                },
                BackgroundColor(UiTheme::void_black().into()),
                BorderColor::from(UiTheme::panel_border()),
            ))
            .with_children(|track| {
                track.spawn((
                    Node {
                        box_sizing: BoxSizing::BorderBox,
                        width: Val::Percent(0.0),
                        height: Val::Percent(100.0),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.35, 0.22, 0.22).into()),
                    PlaybackFoeCdFill,
                ));
            });
        });
}

fn spawn_playback_hero_plate_lead(parent: &mut ChildSpawnerCommands<'_>) {
    parent
        .spawn((
            Node {
                box_sizing: BoxSizing::BorderBox,
                flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(4.0),
                    align_items: AlignItems::FlexStart,
                    ..default()
            },
            PlaybackLeadPortraitBlock,
        ))
        .with_children(|plate| {
            plate
                .spawn((
            Node {
                box_sizing: BoxSizing::BorderBox,
                width: Val::Px(76.0),
                        height: Val::Px(76.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        border: UiRect::all(Val::Px(2.0)),
                        ..default()
            },
            BackgroundColor(UiTheme::panel_bg_deep().into()),
            BorderColor::from(UiTheme::ornate_gold())
        ))
                .with_children(|port| {
                    port.spawn((
                Text::new("\u{2694}"),
                TextFont::from_font_size(UiTheme::FONT_DISPLAY_SUB),
                TextColor(UiTheme::elite()),
            ));
                });
            plate.spawn(caption_text("You"));
            playback_hero_bar(plate, 1.0);
            playback_cast_cd_stack_lead(plate);
            plate.spawn((
                crate::ui::theme::playback_debuff_line_bundle("—  ·  —  ·  —  ·  —"),
                PlaybackHeroDebuffLine,
            ));
        });
}

fn spawn_playback_hero_plate_ally(parent: &mut ChildSpawnerCommands<'_>) {
    let tone = Color::srgb(0.38, 0.72, 0.92);
    parent
        .spawn((
            Node {
                box_sizing: BoxSizing::BorderBox,
                flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(4.0),
                    align_items: AlignItems::FlexStart,
                    ..default()
            },
            PlaybackAllyPortraitBlock,
        ))
        .with_children(|plate| {
            plate
                .spawn((
            Node {
                box_sizing: BoxSizing::BorderBox,
                width: Val::Px(76.0),
                        height: Val::Px(76.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        border: UiRect::all(Val::Px(2.0)),
                        ..default()
            },
            BackgroundColor(UiTheme::panel_bg_deep().into()),
            BorderColor::from(UiTheme::ornate_gold())
        ))
                .with_children(|port| {
                    port.spawn((
                Text::new("\u{1F9D1}"),
                TextFont::from_font_size(UiTheme::FONT_DISPLAY_SUB),
                TextColor(tone),
            ));
                });
            plate.spawn(caption_text("Ally"));
            playback_ally_bar(plate, 1.0);
            playback_cast_cd_stack_ally(plate);
        });
}

fn spawn_playback_enemy_plate(parent: &mut ChildSpawnerCommands<'_>) {
    parent
        .spawn((
            Node {
                box_sizing: BoxSizing::BorderBox,
                flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(4.0),
                    align_items: AlignItems::FlexEnd,
                    ..default()
            },
            PlaybackEnemyPortraitBlock,
        ))
        .with_children(|plate| {
            plate
                .spawn((
            Node {
                box_sizing: BoxSizing::BorderBox,
                width: Val::Px(76.0),
                        height: Val::Px(76.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        border: UiRect::all(Val::Px(2.0)),
                        ..default()
            },
            BackgroundColor(UiTheme::panel_bg_deep().into()),
            BorderColor::from(UiTheme::panel_border())
        ))
                .with_children(|port| {
                    port.spawn((
                Text::new("\u{1F480}"),
                TextFont::from_font_size(UiTheme::FONT_DISPLAY_SUB),
                TextColor(UiTheme::body_dim()),
            ));
                });
            plate.spawn((headline_text("—"), PlaybackEnemyNameText));
            plate.spawn((
                caption_text("\u{2192} \u{2014}"),
                PlaybackAggroArrowText,
            ));
            playback_enemy_bar(plate, 1.0);
            playback_cast_cd_stack_foe(plate);
            plate.spawn((
                crate::ui::theme::playback_debuff_line_bundle("—  ·  —  ·  —  ·  —"),
                PlaybackEnemyDebuffLine,
            ));
        });
}

fn spawn_playback_damage_meters_block(parent: &mut ChildSpawnerCommands<'_>) {
    parent.spawn(section_title("DAMAGE (RUN TOTAL)"));
    parent
        .spawn(Node {
                box_sizing: BoxSizing::BorderBox,
                width: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(3.0),
                margin: UiRect::bottom(Val::Px(10.0)),
                ..default()
            })
        .with_children(|col| {
            col.spawn(Node {
                box_sizing: BoxSizing::BorderBox,
                width: Val::Percent(100.0),
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    column_gap: Val::Px(10.0),
                    margin: UiRect::bottom(Val::Px(2.0)),
                    ..default()
            })
            .with_children(|r| {
                r.spawn(Node {
                box_sizing: BoxSizing::BorderBox,
                min_width: Val::Px(56.0),
                        ..default()
            })
                .with_children(|n| {
                    n.spawn(caption_text("Hero"));
                });
                r.spawn((
            Node {
                box_sizing: BoxSizing::BorderBox,
                flex_grow: 1.0,
                        min_width: Val::Px(48.0),
                        height: Val::Px(12.0),
                        ..default()
            },
            BackgroundColor(Color::srgba(0.06, 0.06, 0.09, 1.0).into())
        ))
                .with_children(|track| {
                    track.spawn((
                        Node {
                box_sizing: BoxSizing::BorderBox,
                width: Val::Percent(0.0),
                                height: Val::Percent(100.0),
                                ..default()
            },
            BackgroundColor(Color::srgb(0.92, 0.55, 0.22).into()),
                        PlaybackDmgMeterLeadFill,
                    ));
                });
                r.spawn((
                    (
                Text::new("0"),
                TextFont::from_font_size(UiTheme::FONT_COMPACT),
                TextColor(UiTheme::body_dim()),
            ),
                    PlaybackDmgMeterLeadValue,
                ));
            });

            col.spawn((
                Node {
                box_sizing: BoxSizing::BorderBox,
                width: Val::Percent(100.0),
                        flex_direction: FlexDirection::Row,
                        align_items: AlignItems::Center,
                        column_gap: Val::Px(10.0),
                        margin: UiRect::bottom(Val::Px(2.0)),
                        ..default()
            },
                PlaybackDmgMeterPartnerRow,
            ))
            .with_children(|r| {
                r.spawn(Node {
                box_sizing: BoxSizing::BorderBox,
                min_width: Val::Px(56.0),
                        ..default()
            })
                .with_children(|n| {
                    n.spawn(caption_text("Ally"));
                });
                r.spawn((
            Node {
                box_sizing: BoxSizing::BorderBox,
                flex_grow: 1.0,
                        min_width: Val::Px(48.0),
                        height: Val::Px(12.0),
                        ..default()
            },
            BackgroundColor(Color::srgba(0.06, 0.06, 0.09, 1.0).into())
        ))
                .with_children(|track| {
                    track.spawn((
                        Node {
                box_sizing: BoxSizing::BorderBox,
                width: Val::Percent(0.0),
                                height: Val::Percent(100.0),
                                ..default()
            },
            BackgroundColor(Color::srgb(0.38, 0.72, 0.95).into()),
                        PlaybackDmgMeterPartnerFill,
                    ));
                });
                r.spawn((
                    (
                Text::new("0"),
                TextFont::from_font_size(UiTheme::FONT_COMPACT),
                TextColor(UiTheme::body_dim()),
            ),
                    PlaybackDmgMeterPartnerValue,
                ));
            });

            col.spawn(Node {
                box_sizing: BoxSizing::BorderBox,
                width: Val::Percent(100.0),
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    column_gap: Val::Px(10.0),
                    ..default()
            })
            .with_children(|r| {
                r.spawn(Node {
                box_sizing: BoxSizing::BorderBox,
                min_width: Val::Px(56.0),
                        ..default()
            })
                .with_children(|n| {
                    n.spawn(caption_text("Foe"));
                });
                r.spawn((
            Node {
                box_sizing: BoxSizing::BorderBox,
                flex_grow: 1.0,
                        min_width: Val::Px(48.0),
                        height: Val::Px(12.0),
                        ..default()
            },
            BackgroundColor(Color::srgba(0.06, 0.06, 0.09, 1.0).into())
        ))
                .with_children(|track| {
                    track.spawn((
                        Node {
                box_sizing: BoxSizing::BorderBox,
                width: Val::Percent(0.0),
                                height: Val::Percent(100.0),
                                ..default()
            },
            BackgroundColor(Color::srgb(0.55, 0.38, 0.42).into()),
                        PlaybackDmgMeterEnemyFill,
                    ));
                });
                r.spawn((
                    (
                Text::new("0"),
                TextFont::from_font_size(UiTheme::FONT_COMPACT),
                TextColor(UiTheme::body_dim()),
            ),
                    PlaybackDmgMeterEnemyValue,
                ));
            });
        });
}

/// Middle column during [`crate::app::GameState::Running`] — synced from [`crate::app::ActiveRunPlayback`].
pub fn spawn_run_playback_middle_column(parent: &mut ChildSpawnerCommands<'_>, ph: &UiPlaceholderImages) {
    let ph = ph.clone();
    let inner = move |p: &mut ChildSpawnerCommands<'_>| {
        p.spawn(panel_title_centered("LIVE DELVE"));
        p.spawn(Node {
                box_sizing: BoxSizing::BorderBox,
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::Center,
                width: Val::Percent(100.0),
                flex_wrap: FlexWrap::Wrap,
                column_gap: Val::Px(8.0),
                ..default()
            })
        .with_children(|r| {
            r.spawn(Node {
                box_sizing: BoxSizing::BorderBox,
                flex_direction: FlexDirection::Row,
                    column_gap: Val::Px(12.0),
                    flex_wrap: FlexWrap::Wrap,
                    align_items: AlignItems::Center,
                    ..default()
            })
            .with_children(|meta| {
                meta.spawn((caption_text("Depth: —"), PlaybackDepthText));
                meta.spawn((caption_text("Type: —"), PlaybackRoomKindText));
            });
            let log_pal = UiButtonPalette::panel_secondary();
            r.spawn((
                Node {
                box_sizing: BoxSizing::BorderBox,
                min_height: Val::Px(32.0),
                        padding: UiRect::axes(Val::Px(12.0), Val::Px(7.0)),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        border: UiRect::all(Val::Px(1.0)),
                        ..default()
            },
            Button,
            BackgroundColor(log_pal.idle_bg.into()),
            BorderColor::from(log_pal.idle_border),
                ToggleCombatLogButton,
                log_pal,
                UiTooltip::txt("Show or hide the text combat log."),
            ))
            .with_children(|b| {
                b.spawn((
                    (
                Text::new("Show log"),
                TextFont::from_font_size(UiTheme::FONT_COMPACT),
                TextColor(UiTheme::muted_cream()),
            ),
                    PlaybackCombatLogToggleLabel,
                ));
            });
        });

        p.spawn(Node {
                box_sizing: BoxSizing::BorderBox,
                width: Val::Percent(100.0),
                min_height: Val::Px(240.0),
                flex_grow: 1.0,
                flex_shrink: 1.0,
                position_type: PositionType::Relative,
                margin: UiRect::vertical(Val::Px(8.0)),
                ..default()
            })
        .with_children(|theater| {
            theater.spawn((
                Node {
                    box_sizing: BoxSizing::BorderBox,
                    position_type: PositionType::Absolute,
                    left: Val::Px(0.0),
                    top: Val::Px(0.0),
                    right: Val::Px(0.0),
                    bottom: Val::Px(0.0),
                    ..default()
                },
                ImageNode {
                    image: ph.dungeon_theater.clone(),
                    color: Color::srgba(0.72, 0.7, 0.78, 0.52),
                    ..default()
                },
            ));
            theater
                .spawn(Node {
                box_sizing: BoxSizing::BorderBox,
                position_type: PositionType::Absolute,
                        left: Val::Px(0.0),
                        top: Val::Px(0.0),
                        right: Val::Px(0.0),
                        bottom: Val::Px(0.0),
                        flex_direction: FlexDirection::Row,
                        justify_content: JustifyContent::SpaceBetween,
                        align_items: AlignItems::FlexStart,
                        column_gap: Val::Px(10.0),
                        padding: UiRect::axes(Val::Px(6.0), Val::Px(4.0)),
                        ..default()
            })
                .with_children(|row| {
                    row.spawn(Node {
                box_sizing: BoxSizing::BorderBox,
                flex_direction: FlexDirection::Column,
                            row_gap: Val::Px(12.0),
                            align_items: AlignItems::FlexStart,
                            flex_shrink: 0.0,
                            ..default()
            })
                    .with_children(|left| {
                        spawn_playback_hero_plate_lead(left);
                        spawn_playback_hero_plate_ally(left);
                    });
                    row.spawn(Node {
                box_sizing: BoxSizing::BorderBox,
                flex_direction: FlexDirection::Column,
                            flex_grow: 1.0,
                            min_width: Val::Px(72.0),
                            align_items: AlignItems::Center,
                            row_gap: Val::Px(6.0),
                            padding: UiRect::all(Val::Px(4.0)),
                            ..default()
            })
                    .with_children(|mid| {
                        mid.spawn(section_title("NOW"));
                        mid.spawn((body_text("…"), PlaybackCaptionText));
                    });
                    row.spawn(Node {
                box_sizing: BoxSizing::BorderBox,
                flex_direction: FlexDirection::Column,
                            flex_shrink: 0.0,
                            ..default()
            })
                    .with_children(|right| {
                        spawn_playback_enemy_plate(right);
                    });
                });
            theater.spawn((
                Node {
                box_sizing: BoxSizing::BorderBox,
                position_type: PositionType::Absolute,
                        height: Val::Px(4.0),
                        width: Val::Percent(44.0),
                        right: Val::Percent(14.0),
                        top: Val::Percent(30.0),
                        ..default()
            },
            BackgroundColor(Color::srgb(0.92, 0.28, 0.2).into()),
            Visibility::Hidden,
                PlaybackAggroArrowLine,
            ));
            theater.spawn((
                Node {
                box_sizing: BoxSizing::BorderBox,
                position_type: PositionType::Absolute,
                        left: Val::Px(0.0),
                        top: Val::Px(0.0),
                        right: Val::Px(0.0),
                        bottom: Val::Px(0.0),
                        ..default()
            },
                PlaybackTheaterFloatLayer,
            ));
        });

        p.spawn((
            Node {
                box_sizing: BoxSizing::BorderBox,
                flex_direction: FlexDirection::Column,
                    width: Val::Percent(100.0),
                    max_height: Val::Px(200.0),
                    flex_shrink: 0.0,
                    overflow: Overflow::clip_y(),
                    ..default()
            },
            Visibility::Hidden,
            PlaybackCombatLogPanel,
        ))
        .with_children(|wrap| {
            wrap.spawn(section_title("COMBAT LOG"));
            spawn_playback_combat_log_scroll(wrap, |scroll| {
                scroll.spawn((body_text(""), PlaybackLogText));
            });
        });

        spawn_playback_damage_meters_block(p);

        p.spawn(section_title("PROGRESS"));
        spawn_playback_delve_progress_section(p);
    };
    inner(parent);
}

pub fn spawn_dungeon_briefing_column(
    parent: &mut ChildSpawnerCommands<'_>,
    stash_count: usize,
    meta: &MetaProgression,
) {
    let inner = move |p: &mut ChildSpawnerCommands<'_>| {
        p.spawn(panel_title_centered("DUNGEON RUN"));
        p.spawn(Node {
                box_sizing: BoxSizing::BorderBox,
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::SpaceBetween,
                width: Val::Percent(100.0),
                ..default()
            })
        .with_children(|r| {
            r.spawn(caption_text(format!(
                "Target depth: {DEFAULT_RUN_MAX_DEPTH}"
            )));
            r.spawn(caption_text("Phase: briefing"));
        });
        p.spawn(Node {
                box_sizing: BoxSizing::BorderBox,
                flex_direction: FlexDirection::Row,
                column_gap: Val::Px(12.0),
                align_items: AlignItems::Center,
                ..default()
            })
        .with_children(|row| {
            row.spawn((
            Node {
                box_sizing: BoxSizing::BorderBox,
                width: Val::Px(96.0),
                    height: Val::Px(96.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    border: UiRect::all(Val::Px(2.0)),
                    ..default()
            },
            BackgroundColor(UiTheme::panel_bg_deep().into()),
            BorderColor::from(UiTheme::ornate_gold())
        ))
            .with_children(|port| {
                port.spawn((
                Text::new("\u{1F480}"),
                TextFont::from_font_size(UiTheme::FONT_DISPLAY_HERO),
                TextColor(UiTheme::body_dim()),
            ));
            });
            row.spawn(Node {
                box_sizing: BoxSizing::BorderBox,
                flex_direction: FlexDirection::Column,
                    flex_grow: 1.0,
                    row_gap: Val::Px(6.0),
                    ..default()
            })
            .with_children(|col| {
                col.spawn(headline_text("Awaiting delve"));
                col.spawn(caption_text(format!(
                    "Boss at depth {DEFAULT_RUN_MAX_DEPTH} · MVP run seed {DEFAULT_RUN_SEED}",
                )));
                health_bar(col, 1.0, UiTheme::healing());
                col.spawn(caption_text(format!("Stash waiting: {stash_count} items")));
                if meta.party_slots_unlocked() < 2 {
                    col.spawn(caption_text(format!(
                        "Reach depth {} to unlock a second hero slot.",
                        PARTY_SLOT_2_UNLOCK_DEPTH
                    )));
                } else {
                    col.spawn(caption_text(
                        "Second party slot unlocked — meet your ally in the party panel.",
                    ));
                }
            });
        });
        p.spawn(section_title("COMBAT LOG"));
        p.spawn(caption_text(
            "Encounter text streams here once live combat ships; scroll with mouse wheel.",
        ));
        p.spawn(caption_text("Mouse wheel scrolls any framed panel."));
        p.spawn(section_title("PROGRESS"));
        spawn_static_delve_progress_section(p, 0, DEFAULT_RUN_MAX_DEPTH);
    };
    inner(parent);
}

pub fn spawn_dungeon_camp_column(parent: &mut ChildSpawnerCommands<'_>) {
    let inner = move |p: &mut ChildSpawnerCommands<'_>| {
        p.spawn(panel_title_centered("DUNGEON RUN"));
        p.spawn(Node {
                box_sizing: BoxSizing::BorderBox,
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::SpaceBetween,
                width: Val::Percent(100.0),
                ..default()
            })
        .with_children(|r| {
            r.spawn(caption_text("Depth: —"));
            r.spawn(caption_text("Type: Camp"));
        });
        p.spawn(section_title("Theater"));
        p.spawn(body_text(
            "Stash, forging contracts, and meta upgrades live in the panel to the right. Head back to briefing when you are ready to delve.",
        ));
        p.spawn(section_title("PROGRESS"));
        spawn_static_delve_progress_section(p, 0, DEFAULT_RUN_MAX_DEPTH);
    };
    inner(parent);
}

pub fn spawn_dungeon_summary_column(parent: &mut ChildSpawnerCommands<'_>, summary: &RunSummary) {
    let inner = move |p: &mut ChildSpawnerCommands<'_>| {
        p.spawn(panel_title_centered("DUNGEON RUN"));
        let depth = summary.deepest_depth;
        let is_death = summary.outcome == RunOutcome::HeroDied;
        let type_color = if is_death {
            UiTheme::danger()
        } else {
            UiTheme::muted_gold()
        };
        let type_label = if is_death { "Defeat" } else { "Boss" };
        p.spawn(Node {
                box_sizing: BoxSizing::BorderBox,
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::SpaceBetween,
                width: Val::Percent(100.0),
                ..default()
            })
        .with_children(|r| {
            r.spawn(caption_text(format!("Depth: {depth}")));
            r.spawn((
                Text::new(format!("Type: {type_label}")),
                TextFont::from_font_size(UiTheme::FONT_COMPACT),
                TextColor(type_color),
            ));
        });
        if !summary.peak_risk_note.is_empty() {
            p.spawn(caption_text(summary.peak_risk_note.clone()));
        }
        let foe = summary
            .death_reason
            .clone()
            .unwrap_or_else(|| "Victory".to_string());
        p.spawn(Node {
                box_sizing: BoxSizing::BorderBox,
                flex_direction: FlexDirection::Row,
                column_gap: Val::Px(12.0),
                align_items: AlignItems::Center,
                ..default()
            })
        .with_children(|row| {
            row.spawn((
            Node {
                box_sizing: BoxSizing::BorderBox,
                width: Val::Px(96.0),
                    height: Val::Px(96.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    border: UiRect::all(Val::Px(2.0)),
                    ..default()
            },
            BackgroundColor(UiTheme::panel_bg_deep().into()),
            BorderColor::from(UiTheme::ornate_gold())
        ))
            .with_children(|port| {
                port.spawn((
                Text::new(if is_death { "\u{2620}" } else { "\u{1F3F9}" }),
                TextFont::from_font_size(UiTheme::FONT_DISPLAY_SUB),
                TextColor(type_color),
            ));
            });
            row.spawn(Node {
                box_sizing: BoxSizing::BorderBox,
                flex_direction: FlexDirection::Column,
                    flex_grow: 1.0,
                    row_gap: Val::Px(6.0),
                    ..default()
            })
            .with_children(|col| {
                col.spawn((
                Text::new(foe.clone()),
                TextFont::from_font_size(UiTheme::FONT_SECTION),
                TextColor(type_color),
            ));
                let frac = if is_death { 0.35 } else { 1.0 };
                health_bar(col, frac, type_color);
            });
        });
        if !summary.loot.is_empty() {
            let n = summary.loot.len();
            p.spawn(section_title("GEAR FROM THIS RUN"));
            p.spawn(body_text(format!(
                "{n} piece(s) here go to your stash when you Accept rewards below. Open Gear after that to equip."
            )));
            for item in summary.loot.iter().take(4) {
                p.spawn(caption_text(format!(
                    "\u{2022} {} ({:?})",
                    item.name, item.rarity
                )));
            }
            if summary.loot.len() > 4 {
                p.spawn(caption_text(format!(
                    "\u{2026} and {} more in the rewards popup.",
                    summary.loot.len() - 4
                )));
            }
        }
        p.spawn(section_title("COMBAT LOG"));
        p.spawn(caption_text("Mouse wheel scrolls."));
        let log_lines: Vec<_> = summary
            .log
            .iter()
            .map(|line| {
                let (color, size) = log_line_present(line);
                (line.clone(), size, color)
            })
            .collect();
        spawn_scrollable_log(p, 200.0, log_lines);
        p.spawn(section_title("PROGRESS"));
        let cap = summary.dungeon_depth_cap.max(1);
        spawn_static_delve_progress_section(p, summary.floors_cleared, cap);
    };
    inner(parent);
}

fn spawn_playback_delve_progress_section(parent: &mut ChildSpawnerCommands<'_>) {
    parent.spawn((
        caption_text(format!("Floors cleared: 0 / {}", DEFAULT_RUN_MAX_DEPTH)),
        PlaybackProgressLabel,
    ));
    parent
        .spawn((
            Node {
                box_sizing: BoxSizing::BorderBox,
                width: Val::Percent(100.0),
                height: Val::Px(16.0),
                border: UiRect::all(Val::Px(1.0)),
                ..default()
            },
            BackgroundColor(UiTheme::void_black().into()),
            BorderColor::from(UiTheme::panel_border())
        ))
        .with_children(|bar| {
            bar.spawn((
                Node {
                box_sizing: BoxSizing::BorderBox,
                width: Val::Percent(0.0),
                        height: Val::Percent(100.0),
                        ..default()
            },
            BackgroundColor(UiTheme::muted_gold().into()),
                PlaybackProgressBarFill,
            ));
        });
}

fn spawn_static_delve_progress_section(parent: &mut ChildSpawnerCommands<'_>, cleared: u32, cap: u32) {
    let cap_n = cap.max(1);
    let frac = cleared as f32 / cap_n as f32;
    parent.spawn(caption_text(format!("Floors cleared: {cleared} / {cap_n}")));
    parent
        .spawn((
            Node {
                box_sizing: BoxSizing::BorderBox,
                width: Val::Percent(100.0),
                height: Val::Px(16.0),
                border: UiRect::all(Val::Px(1.0)),
                ..default()
            },
            BackgroundColor(UiTheme::void_black().into()),
            BorderColor::from(UiTheme::panel_border())
        ))
        .with_children(|bar| {
            bar.spawn((
            Node {
                box_sizing: BoxSizing::BorderBox,
                width: Val::Percent((frac * 100.0).clamp(0.0, 100.0)),
                    height: Val::Percent(100.0),
                    ..default()
            },
            BackgroundColor(UiTheme::muted_gold().into())
        ));
        });
}

fn health_bar(parent: &mut ChildSpawnerCommands<'_>, frac: f32, fill: Color) {
    parent
        .spawn((
            Node {
                box_sizing: BoxSizing::BorderBox,
                width: Val::Percent(100.0),
                height: Val::Px(14.0),
                border: UiRect::all(Val::Px(1.0)),
                ..default()
            },
            BackgroundColor(UiTheme::void_black().into()),
            BorderColor::from(UiTheme::panel_border())
        ))
        .with_children(|bar| {
            bar.spawn((
            Node {
                box_sizing: BoxSizing::BorderBox,
                width: Val::Percent((frac * 100.0).clamp(0.0, 100.0)),
                    height: Val::Percent(100.0),
                    ..default()
            },
            BackgroundColor(fill.into())
        ));
        });
}

pub fn spawn_stash_filters_and_sort_row(parent: &mut ChildSpawnerCommands<'_>, stash_sort: StashSortOrder) {
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
            BorderColor::from(border)
        ))
        .with_children(|n| {
            n.spawn((
                Text::new(label),
                TextFont::from_font_size(UiTheme::FONT_CAPTION),
                TextColor(text),
            ));
        });
}
