//! Three-column mockup shell: ornate panels, centered header stats, footer dock.

use bevy::prelude::*;
use bevy::ui::{FocusPolicy, RelativeCursorPosition};

use crate::domain::progression::MetaProgression;
use crate::domain::progression::UpgradeId;
use crate::domain::run::RunOutcome;
use crate::domain::run::RunSummary;
use crate::ui::components::{
    AcceptRewardsButton, BuyUpgradeButton, ReturnToBuildButton, SettingsButton, TopBarField,
    UiButtonPalette, UiScrollContent, UiScrollRegion, UiScrollState,
};
use crate::ui::theme::{
    body_text, caption_text, format_item_stat_summary, headline_text, log_line_present,
    rarity_color, section_title, UiTheme,
};
use crate::ui::widgets::spawn_scrollable_log;

#[derive(Resource, Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum RightPanelTab {
    #[default]
    Inventory,
    Upgrades,
    Loot,
}

#[derive(Component, Clone, Copy)]
pub struct RightTabButton(pub RightPanelTab);

fn ornate_shell(content: impl FnOnce(&mut ChildBuilder)) -> impl FnOnce(&mut ChildBuilder) {
    move |parent: &mut ChildBuilder| {
        parent
            .spawn(NodeBundle {
                style: Style {
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
                background_color: UiTheme::panel_bg_deep().into(),
                border_color: BorderColor(UiTheme::ornate_gold()),
                ..default()
            })
            .insert(FocusPolicy::Pass)
            .with_children(|ornate| {
                ornate
                    .spawn(NodeBundle {
                        style: Style {
                            width: Val::Percent(100.0),
                            flex_grow: 1.0,
                            flex_shrink: 1.0,
                            min_height: Val::Px(0.0),
                            min_width: Val::Px(0.0),
                            flex_direction: FlexDirection::Column,
                            padding: UiRect::all(Val::Px(12.0)),
                            row_gap: Val::Px(8.0),
                            align_items: AlignItems::Stretch,
                            overflow: Overflow::clip_y(),
                            ..default()
                        },
                        background_color: UiTheme::panel_bg().into(),
                        border_color: BorderColor(UiTheme::panel_border_inner()),
                        ..default()
                    })
                    .with_children(content);
            });
    }
}

/// Fills remaining column height; scrolls when content exceeds the panel.
fn spawn_column_flex_scroll(parent: &mut ChildBuilder, content: impl FnOnce(&mut ChildBuilder)) {
    parent
        .spawn((
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    flex_grow: 1.0,
                    flex_shrink: 1.0,
                    min_height: Val::Px(0.0),
                    position_type: PositionType::Relative,
                    overflow: Overflow::clip_y(),
                    ..default()
                },
                ..default()
            },
            Interaction::default(),
            RelativeCursorPosition::default(),
            UiScrollState::default(),
            UiScrollRegion,
        ))
        .with_children(|vp| {
            vp.spawn((
                NodeBundle {
                    style: Style {
                        position_type: PositionType::Absolute,
                        left: Val::Px(0.0),
                        right: Val::Px(0.0),
                        top: Val::Px(0.0),
                        flex_direction: FlexDirection::Column,
                        row_gap: Val::Px(8.0),
                        align_items: AlignItems::Stretch,
                        ..default()
                    },
                    ..default()
                },
                UiScrollContent,
            ))
            .with_children(content);
        });
}

pub fn spawn_mockup_header(
    parent: &mut ChildBuilder,
    gold: u32,
    salvage: u32,
    skill_slots: usize,
    skill_cap: usize,
    depth_label: &str,
    speed_mult: f32,
) {
    parent
        .spawn(NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                min_height: Val::Px(68.0),
                flex_shrink: 0.0,
                padding: UiRect::axes(Val::Px(16.0), Val::Px(8.0)),
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::SpaceBetween,
                border: UiRect::bottom(Val::Px(2.0)),
                ..default()
            },
            background_color: UiTheme::panel_bg_deep().into(),
            border_color: BorderColor(UiTheme::ornate_gold()),
            ..default()
        })
        .with_children(|row| {
            row.spawn(TextBundle::from_section(
                "Idle Dungeons",
                TextStyle {
                    font_size: 26.0,
                    color: UiTheme::muted_gold(),
                    ..default()
                },
            ));
            row.spawn(NodeBundle {
                style: Style {
                    flex_grow: 1.0,
                    flex_direction: FlexDirection::Row,
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    flex_wrap: FlexWrap::Wrap,
                    column_gap: Val::Px(24.0),
                    row_gap: Val::Px(4.0),
                    ..default()
                },
                ..default()
            })
            .with_children(|center| {
                resource_chip(
                    center,
                    "\u{2694}",
                    "Gold",
                    TopBarField::Gold,
                    format!("{gold}"),
                );
                resource_chip(
                    center,
                    "*",
                    "Salvage",
                    TopBarField::Salvage,
                    format!("{salvage}"),
                );
                resource_chip(
                    center,
                    "\u{2726}",
                    "Skills",
                    TopBarField::SkillSlots,
                    format!("{skill_slots}/{skill_cap}"),
                );
                resource_chip(
                    center,
                    "\u{2022}",
                    "Depth",
                    TopBarField::Depth,
                    depth_label.to_string(),
                );
                resource_chip(
                    center,
                    "\u{21BB}",
                    "Speed",
                    TopBarField::Speed,
                    fmt_speed_label(speed_mult),
                );
            });
            {
                let p = UiButtonPalette::panel_outlined();
                row.spawn((
                    ButtonBundle {
                        style: Style {
                            min_width: Val::Px(100.0),
                            height: Val::Px(38.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            border: UiRect::all(Val::Px(1.0)),
                            ..default()
                        },
                        background_color: p.idle_bg.into(),
                        border_color: BorderColor(p.idle_border),
                        ..default()
                    },
                    SettingsButton,
                    p,
                ))
                .with_children(|btn| {
                    btn.spawn(TextBundle::from_section(
                        "\u{2699} Settings",
                        TextStyle {
                            font_size: 14.0,
                            color: UiTheme::muted_cream(),
                            ..default()
                        },
                    ));
                });
            }
        });
}

fn fmt_speed_label(mult: f32) -> String {
    if (mult - 1.0).abs() < f32::EPSILON {
        "1x".to_string()
    } else if (mult - 2.0).abs() < f32::EPSILON {
        "2x".to_string()
    } else {
        format!("{mult:.1}x")
    }
}

fn resource_chip(
    parent: &mut ChildBuilder,
    icon: &'static str,
    label: &'static str,
    field: TopBarField,
    value: String,
) {
    parent
        .spawn(NodeBundle {
            style: Style {
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                min_width: Val::Px(68.0),
                ..default()
            },
            ..default()
        })
        .with_children(|col| {
            col.spawn(TextBundle::from_section(
                format!("{icon} {label}"),
                TextStyle {
                    font_size: 11.0,
                    color: UiTheme::body_dim(),
                    ..default()
                },
            ));
            col.spawn((
                TextBundle::from_section(
                    value,
                    TextStyle {
                        font_size: 15.0,
                        color: UiTheme::body(),
                        ..default()
                    },
                ),
                field,
            ));
        });
}

pub fn spawn_three_column_row(parent: &mut ChildBuilder, f: impl FnOnce(&mut ChildBuilder)) {
    parent
        .spawn(NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                flex_grow: 1.0,
                flex_basis: Val::Px(0.0),
                min_height: Val::Px(0.0),
                flex_direction: FlexDirection::Row,
                column_gap: Val::Px(10.0),
                align_items: AlignItems::Stretch,
                ..default()
            },
            ..default()
        })
        .with_children(f);
}

pub fn spawn_ornate_column(
    parent: &mut ChildBuilder,
    flex: f32,
    inner: impl FnOnce(&mut ChildBuilder),
) {
    parent
        .spawn(NodeBundle {
            style: Style {
                flex_grow: flex,
                flex_basis: Val::Px(0.0),
                min_width: Val::Px(200.0),
                min_height: Val::Px(0.0),
                flex_direction: FlexDirection::Column,
                overflow: Overflow::clip_y(),
                ..default()
            },
            ..default()
        })
        .with_children(ornate_shell(inner));
}

fn panel_title_centered(text: impl Into<String>) -> TextBundle {
    TextBundle::from_section(
        text,
        TextStyle {
            font_size: 17.0,
            color: UiTheme::muted_cream(),
            ..default()
        },
    )
    .with_text_justify(JustifyText::Center)
}

pub fn spawn_hero_column_mockup(
    parent: &mut ChildBuilder,
    hero: &crate::domain::hero::HeroProfile,
    profile: &crate::app::ProfileState,
    loadout_lines: &[String],
) {
    let inner = move |p: &mut ChildBuilder| {
        p.spawn(panel_title_centered("HERO"));
        spawn_column_flex_scroll(p, move |body| {
            let stats = hero.derived_stats();
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
            body.spawn(section_title("Equipment"));
            mockup_gear_cards(body, profile);
            body.spawn(section_title("Skills"));
            skill_slot_row(body, hero);
        });
    };
    inner(parent);
}

fn stat_line_row(parent: &mut ChildBuilder, label: &str, value: impl std::fmt::Display) {
    parent
        .spawn(NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::Center,
                padding: UiRect::vertical(Val::Px(2.0)),
                ..default()
            },
            ..default()
        })
        .with_children(|r| {
            r.spawn(TextBundle::from_section(
                format!("\u{25C8} {label}"),
                TextStyle {
                    font_size: 14.0,
                    color: UiTheme::body_dim(),
                    ..default()
                },
            ));
            r.spawn(TextBundle::from_section(
                format!("{value}"),
                TextStyle {
                    font_size: 15.0,
                    color: UiTheme::muted_cream(),
                    ..default()
                },
            ));
        });
}

fn skill_slot_row(parent: &mut ChildBuilder, hero: &crate::domain::hero::HeroProfile) {
    parent
        .spawn(NodeBundle {
            style: Style {
                flex_direction: FlexDirection::Row,
                column_gap: Val::Px(8.0),
                flex_wrap: FlexWrap::Wrap,
                ..default()
            },
            ..default()
        })
        .with_children(|row| {
            let cap = hero.equipped_skills.len().max(6);
            for i in 0..cap {
                let unlocked = i < hero.unlocked_skill_slots;
                let label = if unlocked {
                    hero.equipped_skills
                        .get(i)
                        .and_then(|s| *s)
                        .map(|sk| {
                            crate::domain::skills::skill_definition(sk)
                                .name
                                .chars()
                                .next()
                                .unwrap_or('#')
                                .to_string()
                        })
                        .unwrap_or_else(|| (i + 1).to_string())
                } else {
                    "\u{1F512}".to_string()
                };
                row.spawn(NodeBundle {
                    style: Style {
                        width: Val::Px(52.0),
                        height: Val::Px(52.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        border: UiRect::all(Val::Px(1.0)),
                        ..default()
                    },
                    background_color: if unlocked {
                        UiTheme::panel_bg_deep().into()
                    } else {
                        Color::srgba(0.06, 0.06, 0.07, 1.0).into()
                    },
                    border_color: BorderColor(if unlocked {
                        UiTheme::ornate_gold()
                    } else {
                        UiTheme::panel_border()
                    }),
                    ..default()
                })
                .with_children(|s| {
                    s.spawn(TextBundle::from_section(
                        label,
                        TextStyle {
                            font_size: if unlocked { 18.0 } else { 16.0 },
                            color: if unlocked {
                                UiTheme::body()
                            } else {
                                UiTheme::body_dim()
                            },
                            ..default()
                        },
                    ));
                });
            }
        });
}

pub fn mockup_gear_cards(parent: &mut ChildBuilder, profile: &crate::app::ProfileState) {
    for (label, slot) in [
        ("Weapon", crate::domain::items::GearSlot::Weapon),
        ("Armor", crate::domain::items::GearSlot::Armor),
        ("Trinket", crate::domain::items::GearSlot::Trinket),
    ] {
        let item = profile.profile.hero.equipped_item(slot);
        parent
            .spawn(NodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    flex_direction: FlexDirection::Row,
                    column_gap: Val::Px(10.0),
                    padding: UiRect::all(Val::Px(8.0)),
                    border: UiRect::all(Val::Px(1.0)),
                    margin: UiRect::bottom(Val::Px(8.0)),
                    ..default()
                },
                background_color: UiTheme::panel_bg_deep().into(),
                border_color: BorderColor(UiTheme::ornate_gold()),
                ..default()
            })
            .with_children(|card| {
                card.spawn(NodeBundle {
                    style: Style {
                        width: Val::Px(56.0),
                        height: Val::Px(56.0),
                        flex_shrink: 0.0,
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        border: UiRect::all(Val::Px(1.0)),
                        ..default()
                    },
                    background_color: Color::srgb(0.18, 0.16, 0.2).into(),
                    border_color: BorderColor(UiTheme::panel_border_inner()),
                    ..default()
                })
                .with_children(|ph| {
                    ph.spawn(TextBundle::from_section(
                        "\u{2694}",
                        TextStyle {
                            font_size: 22.0,
                            color: UiTheme::body_dim(),
                            ..default()
                        },
                    ));
                });
                card.spawn(NodeBundle {
                    style: Style {
                        flex_direction: FlexDirection::Column,
                        flex_grow: 1.0,
                        align_items: AlignItems::FlexStart,
                        row_gap: Val::Px(2.0),
                        ..default()
                    },
                    ..default()
                })
                .with_children(|txt| {
                    txt.spawn(caption_text(label.to_string()));
                    if let Some(item) = item {
                        txt.spawn(TextBundle::from_section(
                            item.name.clone(),
                            TextStyle {
                                font_size: 15.0,
                                color: rarity_color(item.rarity),
                                ..default()
                            },
                        ));
                        txt.spawn(caption_text(format_item_stat_summary(item)));
                    } else {
                        txt.spawn(body_text("Empty slot"));
                    }
                });
            });
    }
}

pub fn spawn_dungeon_briefing_column(parent: &mut ChildBuilder, stash_count: usize) {
    let inner = move |p: &mut ChildBuilder| {
        p.spawn(panel_title_centered("DUNGEON RUN"));
        p.spawn(NodeBundle {
            style: Style {
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::SpaceBetween,
                width: Val::Percent(100.0),
                ..default()
            },
            ..default()
        })
        .with_children(|r| {
            r.spawn(caption_text("Depth: -"));
            r.spawn(caption_text("Type: Briefing"));
        });
        p.spawn(NodeBundle {
            style: Style {
                flex_direction: FlexDirection::Row,
                column_gap: Val::Px(12.0),
                align_items: AlignItems::Center,
                ..default()
            },
            ..default()
        })
        .with_children(|row| {
            row.spawn(NodeBundle {
                style: Style {
                    width: Val::Px(96.0),
                    height: Val::Px(96.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    border: UiRect::all(Val::Px(2.0)),
                    ..default()
                },
                background_color: UiTheme::panel_bg_deep().into(),
                border_color: BorderColor(UiTheme::ornate_gold()),
                ..default()
            })
            .with_children(|port| {
                port.spawn(TextBundle::from_section(
                    "\u{1F480}",
                    TextStyle {
                        font_size: 38.0,
                        color: UiTheme::body_dim(),
                        ..default()
                    },
                ));
            });
            row.spawn(NodeBundle {
                style: Style {
                    flex_direction: FlexDirection::Column,
                    flex_grow: 1.0,
                    row_gap: Val::Px(6.0),
                    ..default()
                },
                ..default()
            })
            .with_children(|col| {
                col.spawn(headline_text("Awaiting delve"));
                health_bar(col, 1.0, UiTheme::healing());
                col.spawn(caption_text(format!("Stash waiting: {stash_count} items")));
            });
        });
        p.spawn(section_title("COMBAT LOG"));
        p.spawn(caption_text(
            "Encounter text streams here once live combat ships; scroll with mouse wheel.",
        ));
        p.spawn(caption_text("Mouse wheel scrolls any framed panel."));
        p.spawn(section_title("PROGRESS"));
        progress_row(p, None);
    };
    inner(parent);
}

pub fn spawn_dungeon_camp_column(parent: &mut ChildBuilder) {
    let inner = move |p: &mut ChildBuilder| {
        p.spawn(panel_title_centered("DUNGEON RUN"));
        p.spawn(NodeBundle {
            style: Style {
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::SpaceBetween,
                width: Val::Percent(100.0),
                ..default()
            },
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
        progress_row(p, None);
    };
    inner(parent);
}

pub fn spawn_dungeon_summary_column(parent: &mut ChildBuilder, summary: &RunSummary) {
    let inner = move |p: &mut ChildBuilder| {
        p.spawn(panel_title_centered("DUNGEON RUN"));
        let depth = summary.deepest_depth;
        let is_death = summary.outcome == RunOutcome::HeroDied;
        let type_color = if is_death {
            UiTheme::danger()
        } else {
            UiTheme::muted_gold()
        };
        let type_label = if is_death { "Defeat" } else { "Boss" };
        p.spawn(NodeBundle {
            style: Style {
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::SpaceBetween,
                width: Val::Percent(100.0),
                ..default()
            },
            ..default()
        })
        .with_children(|r| {
            r.spawn(caption_text(format!("Depth: {depth}")));
            r.spawn(TextBundle::from_section(
                format!("Type: {type_label}"),
                TextStyle {
                    font_size: 14.0,
                    color: type_color,
                    ..default()
                },
            ));
        });
        let foe = summary
            .death_reason
            .clone()
            .unwrap_or_else(|| "Victory".to_string());
        p.spawn(NodeBundle {
            style: Style {
                flex_direction: FlexDirection::Row,
                column_gap: Val::Px(12.0),
                align_items: AlignItems::Center,
                ..default()
            },
            ..default()
        })
        .with_children(|row| {
            row.spawn(NodeBundle {
                style: Style {
                    width: Val::Px(96.0),
                    height: Val::Px(96.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    border: UiRect::all(Val::Px(2.0)),
                    ..default()
                },
                background_color: UiTheme::panel_bg_deep().into(),
                border_color: BorderColor(UiTheme::ornate_gold()),
                ..default()
            })
            .with_children(|port| {
                port.spawn(TextBundle::from_section(
                    if is_death { "\u{2620}" } else { "\u{1F3F9}" },
                    TextStyle {
                        font_size: 34.0,
                        color: type_color,
                        ..default()
                    },
                ));
            });
            row.spawn(NodeBundle {
                style: Style {
                    flex_direction: FlexDirection::Column,
                    flex_grow: 1.0,
                    row_gap: Val::Px(6.0),
                    ..default()
                },
                ..default()
            })
            .with_children(|col| {
                col.spawn(TextBundle::from_section(
                    foe.clone(),
                    TextStyle {
                        font_size: 17.0,
                        color: type_color,
                        ..default()
                    },
                ));
                let frac = if is_death { 0.35 } else { 1.0 };
                health_bar(col, frac, type_color);
            });
        });
        p.spawn(section_title("COMBAT LOG"));
        p.spawn(caption_text("Mouse wheel scrolls."));
        let log_lines: Vec<_> = summary
            .log
            .iter()
            .map(|line| {
                let (color, size) = log_line_present(line);
                (
                    line.clone(),
                    TextStyle {
                        font_size: size,
                        color,
                        ..default()
                    },
                )
            })
            .collect();
        spawn_scrollable_log(p, 200.0, log_lines);
        p.spawn(section_title("PROGRESS"));
        progress_row(p, Some(summary.deepest_depth));
    };
    inner(parent);
}

fn health_bar(parent: &mut ChildBuilder, frac: f32, fill: Color) {
    parent
        .spawn(NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                height: Val::Px(14.0),
                border: UiRect::all(Val::Px(1.0)),
                ..default()
            },
            background_color: UiTheme::void_black().into(),
            border_color: BorderColor(UiTheme::panel_border()),
            ..default()
        })
        .with_children(|bar| {
            bar.spawn(NodeBundle {
                style: Style {
                    width: Val::Percent((frac * 100.0).clamp(0.0, 100.0)),
                    height: Val::Percent(100.0),
                    ..default()
                },
                background_color: fill.into(),
                ..default()
            });
        });
}

fn progress_row(parent: &mut ChildBuilder, deepest: Option<u32>) {
    parent
        .spawn(NodeBundle {
            style: Style {
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::SpaceBetween,
                width: Val::Percent(100.0),
                padding: UiRect::vertical(Val::Px(6.0)),
                ..default()
            },
            ..default()
        })
        .with_children(|row| {
            for d in [5u32, 10, 15, 20, 25] {
                let done = deepest.map(|dep| dep >= d).unwrap_or(false);
                let sym = if done { "\u{2713}" } else { "\u{25CB}" };
                row.spawn(TextBundle::from_section(
                    format!("{sym} {d}"),
                    TextStyle {
                        font_size: 13.0,
                        color: if done {
                            UiTheme::healing()
                        } else {
                            UiTheme::body_dim()
                        },
                        ..default()
                    },
                ));
            }
        });
}

pub fn spawn_right_management_column(
    parent: &mut ChildBuilder,
    tab: RightPanelTab,
    meta: &MetaProgression,
    inventory: &[crate::domain::items::ItemInstance],
    summary_loot: Option<&[crate::domain::items::ItemInstance]>,
    interactive_inventory: bool,
) {
    parent
        .spawn(NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                min_height: Val::Px(0.0),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(6.0),
                align_items: AlignItems::Stretch,
                ..default()
            },
            ..default()
        })
        .with_children(|col| {
            col.spawn(panel_title_centered("STASH"));
            col.spawn(NodeBundle {
                style: Style {
                    flex_direction: FlexDirection::Row,
                    column_gap: Val::Px(6.0),
                    margin: UiRect::bottom(Val::Px(4.0)),
                    ..default()
                },
                ..default()
            })
            .with_children(|tabs| {
                tab_btn(tabs, "INVENTORY", RightPanelTab::Inventory, tab);
                tab_btn(tabs, "UPGRADES", RightPanelTab::Upgrades, tab);
                tab_btn(tabs, "LOOT", RightPanelTab::Loot, tab);
            });
            col.spawn(caption_text("Filters: all rarities   |   Sort: newest"));
            spawn_right_scroll_body(
                col,
                tab,
                meta,
                inventory,
                summary_loot,
                interactive_inventory,
            );
        });
}

fn tab_btn(parent: &mut ChildBuilder, label: &str, id: RightPanelTab, active: RightPanelTab) {
    let is_on = id == active;
    let p = UiButtonPalette::stash_tab(is_on);
    parent
        .spawn((
            ButtonBundle {
                style: Style {
                    min_width: Val::Px(92.0),
                    height: Val::Px(32.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    border: UiRect::bottom(if is_on { Val::Px(3.0) } else { Val::Px(1.0) }),
                    padding: UiRect::horizontal(Val::Px(6.0)),
                    ..default()
                },
                background_color: p.idle_bg.into(),
                border_color: BorderColor(p.idle_border),
                ..default()
            },
            RightTabButton(id),
            p,
        ))
        .with_children(|b| {
            b.spawn(TextBundle::from_section(
                label,
                TextStyle {
                    font_size: 13.0,
                    color: if is_on {
                        UiTheme::muted_gold()
                    } else {
                        UiTheme::body_dim()
                    },
                    ..default()
                },
            ));
        });
}

fn spawn_right_scroll_body(
    parent: &mut ChildBuilder,
    tab: RightPanelTab,
    meta: &MetaProgression,
    inventory: &[crate::domain::items::ItemInstance],
    summary_loot: Option<&[crate::domain::items::ItemInstance]>,
    interactive_inventory: bool,
) {
    parent
        .spawn((
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    flex_grow: 1.0,
                    min_height: Val::Px(0.0),
                    position_type: PositionType::Relative,
                    overflow: Overflow::clip_y(),
                    ..default()
                },
                ..default()
            },
            Interaction::default(),
            RelativeCursorPosition::default(),
            UiScrollState::default(),
            UiScrollRegion,
        ))
        .with_children(|vp| {
            vp.spawn((
                NodeBundle {
                    style: Style {
                        position_type: PositionType::Absolute,
                        left: Val::Px(0.0),
                        right: Val::Px(0.0),
                        top: Val::Px(0.0),
                        flex_direction: FlexDirection::Column,
                        row_gap: Val::Px(10.0),
                        align_items: AlignItems::Stretch,
                        ..default()
                    },
                    ..default()
                },
                UiScrollContent,
            ))
            .with_children(|body| match tab {
                RightPanelTab::Inventory => {
                    if inventory.is_empty() {
                        body.spawn(caption_text("No items in stash."));
                    } else {
                        for item in inventory.iter().take(14) {
                            if interactive_inventory {
                                super::spawn_item_card(body, item);
                            } else {
                                body.spawn(caption_text(format!(
                                    "\u{2022} {} ({:?})",
                                    item.name, item.rarity
                                )));
                            }
                        }
                    }
                }
                RightPanelTab::Upgrades => {
                    body.spawn(section_title("Caravan contracts"));
                    for upgrade in [
                        UpgradeId::MaxHealth,
                        UpgradeId::BaseDamage,
                        UpgradeId::Armor,
                        UpgradeId::HealingPower,
                        UpgradeId::GoldGain,
                    ] {
                        let p = UiButtonPalette::buy_upgrade();
                        body.spawn((
                            ButtonBundle {
                                style: Style {
                                    width: Val::Percent(100.0),
                                    min_height: Val::Px(40.0),
                                    justify_content: JustifyContent::Center,
                                    align_items: AlignItems::Center,
                                    margin: UiRect::bottom(Val::Px(6.0)),
                                    border: UiRect::all(Val::Px(1.0)),
                                    ..default()
                                },
                                background_color: p.idle_bg.into(),
                                border_color: BorderColor(p.idle_border),
                                ..default()
                            },
                            BuyUpgradeButton { upgrade },
                            p,
                        ))
                        .with_children(|b| {
                            b.spawn(TextBundle::from_section(
                                format!(
                                    "{upgrade:?} L{} - {} g",
                                    meta.upgrade_level(upgrade),
                                    meta.upgrade_cost(upgrade)
                                ),
                                TextStyle {
                                    font_size: 14.0,
                                    color: UiTheme::body(),
                                    ..default()
                                },
                            ));
                        });
                    }
                }
                RightPanelTab::Loot => {
                    let loot = summary_loot.unwrap_or(&[]);
                    if loot.is_empty() {
                        body.spawn(caption_text("No loot in this view yet."));
                    } else {
                        for item in loot {
                            body.spawn(caption_text(format!(
                                "\u{2728} {} ({:?})",
                                item.name, item.rarity
                            )));
                        }
                    }
                }
            });
        });
}

#[derive(Clone, Copy)]
pub enum FooterMode {
    Briefing,
    Camp,
    Summary,
}

pub fn spawn_mockup_footer(parent: &mut ChildBuilder, mode: FooterMode) {
    parent
        .spawn(NodeBundle {
            style: Style {
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
            background_color: UiTheme::panel_bg_deep().into(),
            border_color: BorderColor(UiTheme::ornate_gold()),
            ..default()
        })
        .with_children(|row| {
            row.spawn(NodeBundle {
                style: Style {
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    column_gap: Val::Px(8.0),
                    flex_wrap: FlexWrap::Wrap,
                    ..default()
                },
                ..default()
            })
            .with_children(|nav| {
                footer_pill(nav, "CAMP", matches!(mode, FooterMode::Camp));
                footer_pill(nav, "RUN", matches!(mode, FooterMode::Briefing));
                footer_pill(nav, "HERO", false);
                footer_pill(nav, "UPGRADES", false);
                footer_pill(nav, "CODEX", false);
            });
            match mode {
                FooterMode::Camp => {
                    let p = UiButtonPalette::panel_secondary();
                    row.spawn((
                        ButtonBundle {
                            style: Style {
                                height: Val::Px(40.0),
                                padding: UiRect::axes(Val::Px(14.0), Val::Px(8.0)),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                border: UiRect::all(Val::Px(1.0)),
                                ..default()
                            },
                            background_color: p.idle_bg.into(),
                            border_color: BorderColor(p.idle_border),
                            ..default()
                        },
                        ReturnToBuildButton,
                        p,
                    ))
                    .with_children(|b| {
                        b.spawn(TextBundle::from_section(
                            "\u{2190} Briefing",
                            TextStyle {
                                font_size: 15.0,
                                color: UiTheme::muted_cream(),
                                ..default()
                            },
                        ));
                    });
                }
                FooterMode::Briefing => {
                    let p = UiButtonPalette::primary_cta();
                    row.spawn((
                        ButtonBundle {
                            style: Style {
                                min_width: Val::Px(220.0),
                                height: Val::Px(52.0),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                border: UiRect::all(Val::Px(2.0)),
                                ..default()
                            },
                            background_color: p.idle_bg.into(),
                            border_color: BorderColor(p.idle_border),
                            ..default()
                        },
                        crate::ui::components::StartRunButton,
                        p,
                    ))
                    .with_children(|b| {
                        b.spawn(TextBundle::from_section(
                            "\u{2694} START RUN",
                            TextStyle {
                                font_size: 20.0,
                                color: Color::WHITE,
                                ..default()
                            },
                        ));
                    });
                }
                FooterMode::Summary => {
                    let p = UiButtonPalette::primary_cta();
                    row.spawn((
                        ButtonBundle {
                            style: Style {
                                min_width: Val::Px(260.0),
                                height: Val::Px(52.0),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                border: UiRect::all(Val::Px(2.0)),
                                ..default()
                            },
                            background_color: p.idle_bg.into(),
                            border_color: BorderColor(p.idle_border),
                            ..default()
                        },
                        AcceptRewardsButton,
                        p,
                    ))
                    .with_children(|b| {
                        b.spawn(TextBundle::from_section(
                            "\u{2713} Accept rewards",
                            TextStyle {
                                font_size: 18.0,
                                color: Color::WHITE,
                                ..default()
                            },
                        ));
                    });
                }
            }
        });
}

fn footer_pill(parent: &mut ChildBuilder, label: &str, active: bool) {
    let (bg, border, text) = if active {
        (
            UiTheme::panel_bg().into(),
            BorderColor(UiTheme::ornate_gold()),
            UiTheme::muted_gold(),
        )
    } else {
        (
            UiTheme::panel_bg_deep().into(),
            BorderColor(UiTheme::panel_border()),
            UiTheme::body_dim(),
        )
    };
    parent
        .spawn(NodeBundle {
            style: Style {
                height: Val::Px(36.0),
                padding: UiRect::axes(Val::Px(12.0), Val::Px(6.0)),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                border: UiRect::all(Val::Px(1.0)),
                ..default()
            },
            background_color: bg,
            border_color: border,
            ..default()
        })
        .with_children(|n| {
            n.spawn(TextBundle::from_section(
                label,
                TextStyle {
                    font_size: 13.0,
                    color: text,
                    ..default()
                },
            ));
        });
}
