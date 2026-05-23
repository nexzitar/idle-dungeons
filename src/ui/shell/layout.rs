//! Ornate column shell, centered header stats, dungeon briefing/summary panels.
use bevy::prelude::*;
use bevy::text::{Justify, TextColor, TextFont, TextLayout};
use bevy::ui::FocusPolicy;

use crate::app::PLAYBACK_SPEED_STEPS;
use crate::domain::dungeon::RoomKind;
use crate::domain::items::GearSlot;
use crate::domain::progression::MetaProgression;
use crate::domain::run::{RunOutcome, RunSummary, DEFAULT_RUN_MAX_DEPTH, DEFAULT_RUN_SEED};
use crate::domain::skills::skill_definition;
use crate::ui::components::{
    PlaybackSpeedDecButton, PlaybackSpeedIncButton, PlaybackSpeedValueText,
    ResetProgressButton, SettingsButton, SettingsModalBackdrop,
    SettingsModalCloseButton, SettingsModalRoot,
    TopBarField, UiButtonPalette, UiTooltip,
};
use crate::ui::assets::UiPlaceholderImages;
use crate::ui::primitives::panel::{spawn_mounted_panel, MountedPanelConfig};
use crate::ui::primitives::section::spawn_framed_section_header;
use crate::ui::primitives::scroll::{spawn_scrollable_flex_column, spawn_scrollable_log};
use crate::ui::theme::{
    body_text, caption_text, format_item_affix_lines, format_item_stat_summary, headline_text,
    log_line_present, rarity_color, section_title, UiDensity, UiTheme,
};

fn ornate_shell(
    content: impl FnOnce(&mut ChildSpawnerCommands<'_>),
) -> impl FnOnce(&mut ChildSpawnerCommands<'_>) {
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
                BorderColor::from(UiTheme::ornate_gold()),
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
                        BorderColor::from(UiTheme::panel_border_inner()),
                    ))
                    .with_children(content);
            });
    }
}

/// Fills remaining column height; scrolls when content exceeds the panel.
///
/// When `min_viewport_height_px` is set, guarantees a minimum viewport height so flex layout
/// does not collapse empty (e.g. run rewards loot list).
pub(super) fn spawn_column_flex_scroll(
    parent: &mut ChildSpawnerCommands<'_>,
    min_viewport_height_px: Option<f32>,
    content: impl FnOnce(&mut ChildSpawnerCommands<'_>),
) {
    spawn_scrollable_flex_column(parent, min_viewport_height_px, content);
}

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
                crate::ui::interaction::UiClickAction::CloseSettings,
                backdrop_pal,
                UiTooltip::txt("Click backdrop to close."),
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
                            crate::ui::interaction::UiClickAction::ResetProgress,
                            reset_pal,
                            UiTooltip::txt("Wipe save data and start fresh."),
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
                            crate::ui::interaction::UiClickAction::CloseSettings,
                            close_pal,
                            UiTooltip::txt("Close settings."),
                        ))
                        .with_children(|b| {
                            b.spawn((
                Text::new("Close"),
                TextFont::from_font_size(UiTheme::FONT_BODY),
                TextColor(UiTheme::muted_cream()),
            ));
                        });
                    #[cfg(debug_assertions)]
                    {
                        use crate::presentation::editor::{
                            PresentationEditorSettingsToggleButton, PresentationEditorSettingsToggleText,
                        };
                        dialog.spawn(section_title("Debug"));
                        let pe_pal = UiButtonPalette::panel_secondary();
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
                                BackgroundColor(pe_pal.idle_bg.into()),
                                BorderColor::from(pe_pal.idle_border),
                                PresentationEditorSettingsToggleButton,
                                crate::ui::interaction::UiClickAction::EditorToggleLayout,
                                pe_pal,
                                UiTooltip::txt(
                                    "Open the fullscreen presentation editor overlay (same as layout mode).",
                                ),
                            ))
                            .with_children(|b| {
                                b.spawn((
                                    Text::new("Presentation editor · OFF"),
                                    TextFont::from_font_size(UiTheme::FONT_BODY),
                                    TextColor(UiTheme::body()),
                                    PresentationEditorSettingsToggleText,
                                ));
                            });
                    }
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
            UiTooltip::txt("Delve playback speed."),
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
            wrap.spawn(Node {
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
                    crate::ui::interaction::UiClickAction::PlaybackSpeedDec,
                    p_dec,
                    UiTooltip::txt("Slower playback."),
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
                    crate::ui::interaction::UiClickAction::PlaybackSpeedInc,
                    p_inc,
                    UiTooltip::txt("Faster playback."),
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
                        crate::ui::interaction::UiClickAction::OpenSettings,
                        p,
                        UiTooltip::txt("Open settings."),
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
            crate::ui::interaction::UiClickAction::OpenSettings,
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

pub fn fmt_speed_label(mult: f32) -> String {
    const EPS: f32 = 1e-3;
    if PLAYBACK_SPEED_STEPS.iter().any(|s| (mult - *s).abs() < EPS) {
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

pub fn spawn_three_column_row(
    parent: &mut ChildSpawnerCommands<'_>,
    f: impl FnOnce(&mut ChildSpawnerCommands<'_>),
) {
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

pub(super) fn panel_title_centered(text: impl Into<String>) -> impl Bundle {
    (
        Text::new(text.into()),
        TextLayout::new_with_justify(Justify::Center),
        TextFont::from_font_size(UiTheme::FONT_SECTION),
        TextColor(UiTheme::muted_cream()),
    )
}

fn gear_slot_row_color(slot: GearSlot) -> Color {
    match slot {
        GearSlot::MainHand | GearSlot::OffHand | GearSlot::Hands => Color::srgb(1.0, 0.72, 0.45),
        GearSlot::Head | GearSlot::Chest | GearSlot::Feet => Color::srgb(0.72, 0.82, 0.95),
        GearSlot::Trinket1 | GearSlot::Trinket2 => Color::srgb(0.85, 0.68, 1.0),
        GearSlot::Relic => Color::srgb(0.95, 0.78, 0.45),
    }
}

pub fn mockup_gear_cards(
    parent: &mut ChildSpawnerCommands<'_>,
    profile: &crate::app::ProfileState,
    ph: &UiPlaceholderImages,
) {
    let main_two_handed = profile
        .profile
        .hero
        .equipped_item(GearSlot::MainHand)
        .is_some_and(|i| i.two_handed);
    for slot in GearSlot::ALL {
        let label = slot.display_label();
        let item = profile.profile.hero.equipped_item(slot);
        let blocked_off_hand = matches!(slot, GearSlot::OffHand) && main_two_handed;
        let tip = if blocked_off_hand {
            "Two-handed weapon equipped — off-hand is locked while this weapon is in use. \
             Equip a one-handed main weapon to use a shield or focus again."
                .to_string()
        } else if let Some(item) = item {
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
                "No {} equipped yet. Loot gear on runs and equip it from the Inventory tab.",
                label.to_lowercase()
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
                    BorderColor::from(UiTheme::panel_border_inner()),
                ))
                .with_children(|icon_cell| {
                    let tint = gear_slot_row_color(slot);
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
                    if blocked_off_hand {
                        txt.spawn(body_text("Held by two-hander"));
                    } else if let Some(item) = item {
                        if item.two_handed {
                            txt.spawn(caption_text("Two-handed"));
                        }
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
pub fn spawn_dungeon_briefing_column(
    parent: &mut ChildSpawnerCommands<'_>,
    stash_count: usize,
    meta: &MetaProgression,
) {
    spawn_mounted_panel(
        parent,
        MountedPanelConfig {
            style: crate::ui::theme::MountedPanelStyle::Recessed,
            width: Val::Percent(100.0),
            flex_grow: 1.0,
            flex_shrink: 1.0,
            min_height: Val::Px(0.0),
        },
        |panel| {
            spawn_framed_section_header(panel, "DUNGEON RUN");
            panel.spawn(Node {
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
            panel.spawn(Node {
                box_sizing: BoxSizing::BorderBox,
                flex_direction: FlexDirection::Row,
                column_gap: Val::Px(UiDensity::Camp.gutter_section()),
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
                    BorderColor::from(UiTheme::ornate_gold()),
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
                    row_gap: Val::Px(UiDensity::Camp.gutter_row()),
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
                            crate::domain::progression::PARTY_SLOT_2_UNLOCK_DEPTH
                        )));
                    } else {
                        col.spawn(caption_text(
                            "Second party slot unlocked — meet your ally in the party panel.",
                        ));
                    }
                });
            });
            panel.spawn(section_title("COMBAT LOG"));
            panel.spawn(caption_text(
                "Encounter text streams here once live combat ships; scroll with mouse wheel.",
            ));
            panel.spawn(caption_text("Mouse wheel scrolls any framed panel."));
            panel.spawn(section_title("PROGRESS"));
            spawn_static_delve_progress_section(panel, 0, DEFAULT_RUN_MAX_DEPTH);
        },
    );
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
                BorderColor::from(UiTheme::ornate_gold()),
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
fn spawn_static_delve_progress_section(
    parent: &mut ChildSpawnerCommands<'_>,
    cleared: u32,
    cap: u32,
) {
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
            BorderColor::from(UiTheme::panel_border()),
        ))
        .with_children(|bar| {
            bar.spawn((
                Node {
                    box_sizing: BoxSizing::BorderBox,
                    width: Val::Percent((frac * 100.0).clamp(0.0, 100.0)),
                    height: Val::Percent(100.0),
                    ..default()
                },
                BackgroundColor(UiTheme::muted_gold().into()),
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
            BorderColor::from(UiTheme::panel_border()),
        ))
        .with_children(|bar| {
            bar.spawn((
                Node {
                    box_sizing: BoxSizing::BorderBox,
                    width: Val::Percent((frac * 100.0).clamp(0.0, 100.0)),
                    height: Val::Percent(100.0),
                    ..default()
                },
                BackgroundColor(fill.into()),
            ));
        });
}
