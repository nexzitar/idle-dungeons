//! Playback theater composition (middle column during live delve).

use bevy::prelude::*;
use bevy::text::{TextColor, TextFont};
use bevy::ui::{FocusPolicy, RelativeCursorPosition};

use crate::domain::hero::HeroProfile;
use crate::domain::party::PartyHeroKind;
use crate::domain::run::DEFAULT_RUN_MAX_DEPTH;
use crate::ui::assets::UiPlaceholderImages;
use crate::ui::components::{
    PlaybackAggroArrowLine, PlaybackAggroArrowText, PlaybackCaptionText, PlaybackCombatLogPanel,
    PlaybackCombatLogToggleLabel, PlaybackDepthText, PlaybackDmgMeterEnemyFill,
    PlaybackDmgMeterEnemyValue, PlaybackDmgMeterPlayer0Fill, PlaybackDmgMeterPlayer0Value,
    PlaybackDmgMeterPlayer1Fill, PlaybackDmgMeterPlayer1Row, PlaybackDmgMeterPlayer1Value,
    PlaybackEnemyDebuffLine, PlaybackEnemyNameText, PlaybackEnemyPortraitBlock,
    PlaybackFoeAltTimingRow, PlaybackLogScrollRegion, PlaybackLogText, PlaybackPlayer0DebuffLine,
    PlaybackPlayer0PortraitBlock, PlaybackPlayer1PortraitBlock, PlaybackProgressBarFill,
    PlaybackProgressLabel, PlaybackRoomKindText, PlaybackTheaterFloatLayer, ToggleCombatLogButton,
    UiButtonPalette, UiScrollContent, UiScrollRegion, UiScrollState,
};
use crate::ui::inspect::{InspectHint, InspectRegion, InspectRegionScope};
use crate::ui::primitives::inspect_panel::spawn_inspect_panel_compact;
use crate::ui::primitives::loadout::slots_from_hero;
use crate::ui::primitives::skill_bar::{spawn_skill_bar, SkillBarConfig, SkillBarInteraction};
use crate::ui::theme::{body_text, caption_text, headline_text, section_title, UiDensity, UiTheme};

use super::layout::panel_title_centered;
use super::playback_bars::{
    playback_cast_cd_stack_ally, playback_cast_cd_stack_foe, playback_cast_cd_stack_foe_alt,
    playback_cast_cd_stack_lead, playback_enemy_bar, playback_player0_bar, playback_player1_bar,
};

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
fn spawn_playback_player0_plate(
    parent: &mut ChildSpawnerCommands<'_>,
    lead: &HeroProfile,
    ph: &UiPlaceholderImages,
) {
    parent
        .spawn((
            Node {
                box_sizing: BoxSizing::BorderBox,
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(4.0),
                align_items: AlignItems::FlexStart,
                ..default()
            },
            PlaybackPlayer0PortraitBlock,
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
                    BorderColor::from(UiTheme::ornate_gold()),
                ))
                .with_children(|port| {
                    port.spawn((
                        Text::new("\u{2694}"),
                        TextFont::from_font_size(UiTheme::FONT_DISPLAY_SUB),
                        TextColor(UiTheme::elite()),
                    ));
                });
            plate.spawn(caption_text("You"));
            playback_player0_bar(plate, 1.0);
            let slots = slots_from_hero(lead);
            spawn_skill_bar(
                plate,
                SkillBarConfig {
                    hero: PartyHeroKind::Player1,
                    slots: &slots,
                    unlocked: lead.unlocked_skill_slots,
                    focused_index: None,
                    interaction: SkillBarInteraction::None,
                    density: UiDensity::Combat,
                },
                ph,
            );
            playback_cast_cd_stack_lead(plate);
            plate.spawn((
                crate::ui::theme::playback_debuff_line_bundle("—  ·  —  ·  —  ·  —"),
                PlaybackPlayer0DebuffLine,
            ));
        });
}

fn spawn_playback_player1_plate(
    parent: &mut ChildSpawnerCommands<'_>,
    partner: Option<&HeroProfile>,
    party_slots_unlocked: usize,
    ph: &UiPlaceholderImages,
) {
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
            PlaybackPlayer1PortraitBlock,
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
                    BorderColor::from(UiTheme::ornate_gold()),
                ))
                .with_children(|port| {
                    port.spawn((
                        Text::new("\u{1F9D1}"),
                        TextFont::from_font_size(UiTheme::FONT_DISPLAY_SUB),
                        TextColor(tone),
                    ));
                });
            plate.spawn(caption_text("Player 2"));
            playback_player1_bar(plate, 1.0);
            if party_slots_unlocked >= 2 {
                let slots = partner.map(slots_from_hero).unwrap_or([None; 6]);
                let unlocked = partner.map(|h| h.unlocked_skill_slots).unwrap_or(0);
                spawn_skill_bar(
                    plate,
                    SkillBarConfig {
                        hero: PartyHeroKind::Player2,
                        slots: &slots,
                        unlocked,
                        focused_index: None,
                        interaction: SkillBarInteraction::None,
                        density: UiDensity::Combat,
                    },
                    ph,
                );
            }
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
                    BorderColor::from(UiTheme::panel_border()),
                ))
                .with_children(|port| {
                    port.spawn((
                        Text::new("\u{1F480}"),
                        TextFont::from_font_size(UiTheme::FONT_DISPLAY_SUB),
                        TextColor(UiTheme::body_dim()),
                    ));
                });
            plate.spawn((headline_text("—"), PlaybackEnemyNameText));
            plate.spawn((caption_text("\u{2192} \u{2014}"), PlaybackAggroArrowText));
            playback_enemy_bar(plate, 1.0);
            playback_cast_cd_stack_foe(plate);
            plate
                .spawn((
                    Node {
                        box_sizing: BoxSizing::BorderBox,
                        flex_direction: FlexDirection::Column,
                        row_gap: Val::Px(2.0),
                        align_items: AlignItems::FlexEnd,
                        ..default()
                    },
                    Visibility::Hidden,
                    PlaybackFoeAltTimingRow,
                ))
                .with_children(|alt| {
                    alt.spawn((
                        Text::new("Flank"),
                        TextFont::from_font_size(UiTheme::FONT_MICRO),
                        TextColor(UiTheme::body_dim()),
                    ));
                    playback_cast_cd_stack_foe_alt(alt);
                });
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
                    n.spawn(caption_text("Player 1"));
                });
                r.spawn((
                    Node {
                        box_sizing: BoxSizing::BorderBox,
                        flex_grow: 1.0,
                        min_width: Val::Px(48.0),
                        height: Val::Px(12.0),
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.06, 0.06, 0.09, 1.0).into()),
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
                        PlaybackDmgMeterPlayer0Fill,
                    ));
                });
                r.spawn((
                    (
                        Text::new("0"),
                        TextFont::from_font_size(UiTheme::FONT_COMPACT),
                        TextColor(UiTheme::body_dim()),
                    ),
                    PlaybackDmgMeterPlayer0Value,
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
                PlaybackDmgMeterPlayer1Row,
            ))
            .with_children(|r| {
                r.spawn(Node {
                    box_sizing: BoxSizing::BorderBox,
                    min_width: Val::Px(56.0),
                    ..default()
                })
                .with_children(|n| {
                    n.spawn(caption_text("Player 2"));
                });
                r.spawn((
                    Node {
                        box_sizing: BoxSizing::BorderBox,
                        flex_grow: 1.0,
                        min_width: Val::Px(48.0),
                        height: Val::Px(12.0),
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.06, 0.06, 0.09, 1.0).into()),
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
                        PlaybackDmgMeterPlayer1Fill,
                    ));
                });
                r.spawn((
                    (
                        Text::new("0"),
                        TextFont::from_font_size(UiTheme::FONT_COMPACT),
                        TextColor(UiTheme::body_dim()),
                    ),
                    PlaybackDmgMeterPlayer1Value,
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
                    BackgroundColor(Color::srgba(0.06, 0.06, 0.09, 1.0).into()),
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
pub fn spawn_run_playback_middle_column(
    parent: &mut ChildSpawnerCommands<'_>,
    ph: &UiPlaceholderImages,
    lead: &HeroProfile,
    partner: Option<&HeroProfile>,
    party_slots_unlocked: usize,
) {
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
                crate::ui::interaction::UiClickAction::ToggleCombatLog,
                log_pal,
                InspectHint("Show or hide the text combat log."),
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
                        spawn_playback_player0_plate(left, lead, &ph);
                        spawn_playback_player1_plate(left, partner, party_slots_unlocked, &ph);
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

        let inspect = spawn_inspect_panel_compact(p, InspectRegionScope::PlaybackTheater);
        p.commands_mut().entity(inspect).insert((
            InspectRegion,
            InspectRegionScope::PlaybackTheater,
        ));

        p.spawn(section_title("PROGRESS"));
        spawn_playback_delve_progress_section(p);
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
            BorderColor::from(UiTheme::panel_border()),
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
