pub mod build_panel;
pub mod components;
pub mod inventory_panel;
pub mod log_panel;
pub mod run_panel;
pub mod summary_panel;
pub mod theme;
pub mod upgrade_panel;
pub mod widgets;

use crate::app::{
    AcceptRunRewards, BuyUpgrade, EquipInventoryItem, GameState, LatestRunSummary, ProfileState,
    ReturnToBuild, RunSpeedSetting, SalvageInventoryItem, StartRun,
};
use crate::domain::items::GearSlot;
use crate::domain::items::ItemInstance;
use crate::domain::progression::UpgradeId;
use crate::domain::skills::skill_definition;
use crate::ui::build_panel::build_panel_text;
use crate::ui::components::*;
use crate::ui::summary_panel::{outcome_headline, reward_digest};
use crate::ui::theme::{
    body_text, caption_text, format_item_stat_summary, headline_text, log_line_present,
    rarity_color, section_title, UiTheme,
};
use crate::ui::widgets::{
    spawn_atmosphere, spawn_bottom_strip, spawn_framed_panel, spawn_top_resource_bar,
};
use bevy::prelude::*;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_camera)
            .add_systems(OnEnter(GameState::Build), spawn_build_screen)
            .add_systems(OnExit(GameState::Build), cleanup_ui)
            .add_systems(
                Update,
                (
                    handle_start_button.run_if(in_state(GameState::Build)),
                    sync_top_bar,
                    handle_settings_button,
                ),
            )
            .add_systems(OnEnter(GameState::Summary), spawn_summary_screen)
            .add_systems(OnExit(GameState::Summary), cleanup_ui)
            .add_systems(OnEnter(GameState::Upgrades), spawn_upgrade_screen)
            .add_systems(OnExit(GameState::Upgrades), cleanup_ui)
            .add_systems(
                Update,
                handle_accept_button.run_if(in_state(GameState::Summary)),
            )
            .add_systems(
                Update,
                (
                    handle_equip_buttons,
                    handle_salvage_buttons,
                    handle_buy_upgrade_buttons,
                    handle_return_to_build_button,
                )
                    .run_if(in_state(GameState::Upgrades)),
            )
            .add_systems(
                PostUpdate,
                refresh_upgrade_screen_on_profile_change.run_if(in_state(GameState::Upgrades)),
            );
    }
}

fn spawn_camera(mut commands: Commands) {
    commands.spawn((Camera2dBundle::default(), MainCamera));
}

fn root_shell() -> NodeBundle {
    NodeBundle {
        style: Style {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            position_type: PositionType::Relative,
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Stretch,
            ..default()
        },
        background_color: Color::NONE.into(),
        ..default()
    }
}

fn content_column_bundle() -> NodeBundle {
    NodeBundle {
        style: Style {
            width: Val::Percent(100.0),
            flex_grow: 1.0,
            min_height: Val::Px(0.0),
            flex_direction: FlexDirection::Column,
            padding: UiRect::axes(Val::Px(18.0), Val::Px(14.0)),
            row_gap: Val::Px(12.0),
            align_items: AlignItems::Stretch,
            ..default()
        },
        ..default()
    }
}

fn main_split_row_bundle() -> NodeBundle {
    NodeBundle {
        style: Style {
            width: Val::Percent(100.0),
            flex_grow: 1.0,
            flex_basis: Val::Px(0.0),
            min_height: Val::Px(0.0),
            flex_direction: FlexDirection::Row,
            column_gap: Val::Px(14.0),
            align_items: AlignItems::Stretch,
            ..default()
        },
        ..default()
    }
}

fn spawn_build_screen(
    mut commands: Commands,
    profile: Res<ProfileState>,
    speed: Res<RunSpeedSetting>,
) {
    let hero = profile.effective_hero();
    let meta = &profile.profile.meta;
    let build_compact = build_panel_text(&hero);

    commands
        .spawn((root_shell(), UiRoot))
        .with_children(|root| {
            spawn_atmosphere(root);
            root.spawn(content_column_bundle()).with_children(|col| {
                spawn_top_resource_bar(
                    col,
                    meta.gold,
                    meta.salvage,
                    meta.unlocked_skill_slots,
                    "—",
                    speed.0,
                );
                col.spawn(main_split_row_bundle()).with_children(|row| {
                    spawn_framed_panel(row, 1.0, |panel| {
                        panel.spawn(headline_text("Delver"));
                        panel.spawn(section_title("Party status"));
                        for line in build_compact.lines() {
                            panel.spawn(body_text(line.to_string()));
                        }
                        spawn_skills_section(panel, &hero);
                        spawn_gear_slots(panel, &profile);
                    });
                    spawn_framed_panel(row, 1.05, |panel| {
                        panel.spawn(headline_text("Expedition"));
                        panel.spawn(body_text(
                            "Rostrum briefing — your delver will fight, loot, and retreat on their own.",
                        ));
                        panel.spawn(caption_text(
                            "No live combat feed yet: resolution is instant, then you review chronicles.",
                        ));
                        panel.spawn(caption_text(format!(
                            "Stash waiting: {} items",
                            profile.profile.inventory.len()
                        )));
                        panel.spawn(NodeBundle {
                            style: Style {
                                width: Val::Percent(100.0),
                                height: Val::Px(1.0),
                                margin: UiRect::vertical(Val::Px(8.0)),
                                ..default()
                            },
                            background_color: UiTheme::panel_border_inner().into(),
                            ..default()
                        });
                        panel
                            .spawn((
                                ButtonBundle {
                                    style: Style {
                                        width: Val::Percent(100.0),
                                        max_width: Val::Px(320.0),
                                        height: Val::Px(52.0),
                                        justify_content: JustifyContent::Center,
                                        align_items: AlignItems::Center,
                                        align_self: AlignSelf::FlexStart,
                                        margin: UiRect::top(Val::Px(6.0)),
                                        ..default()
                                    },
                                    background_color: UiTheme::muted_red().into(),
                                    ..default()
                                },
                                StartRunButton,
                            ))
                            .with_children(|button| {
                                button.spawn(TextBundle::from_section(
                                    "Begin Delve",
                                    TextStyle {
                                        font_size: 20.0,
                                        color: Color::WHITE,
                                        ..default()
                                    },
                                ));
                            });
                    });
                });
                spawn_bottom_strip(col, |strip| {
                    strip.spawn(section_title("Bandcamp notes"));
                    strip.spawn(body_text(
                        "After each run you will manage stash, gear, and permanent upgrades before delving again.",
                    ));
                });
            });
        });
}

fn spawn_summary_screen(
    mut commands: Commands,
    profile: Res<ProfileState>,
    latest_summary: Option<Res<LatestRunSummary>>,
    speed: Res<RunSpeedSetting>,
) {
    let summary = latest_summary
        .as_deref()
        .map(|s| s.summary.clone())
        .unwrap_or_else(|| crate::ui::summary_panel::empty_run_summary());
    let meta = &profile.profile.meta;
    let hero = profile.effective_hero();

    commands
        .spawn((root_shell(), UiRoot, SummaryScreen))
        .with_children(|root| {
            spawn_atmosphere(root);
            root.spawn(content_column_bundle()).with_children(|col| {
                spawn_top_resource_bar(
                    col,
                    meta.gold,
                    meta.salvage,
                    meta.unlocked_skill_slots,
                    &summary.deepest_depth.to_string(),
                    speed.0,
                );
                col.spawn(main_split_row_bundle()).with_children(|row| {
                    spawn_framed_panel(row, 0.95, |panel| {
                        panel.spawn(headline_text("Delver (pre-reward)"));
                        panel.spawn(section_title("Loadout"));
                        for line in build_panel_text(&hero).lines() {
                            panel.spawn(body_text(line.to_string()));
                        }
                        spawn_skills_section(panel, &hero);
                        spawn_gear_slots(panel, &profile);
                    });
                    spawn_framed_panel(row, 1.1, |panel| {
                        panel.spawn(headline_text("Run chronicle"));
                        panel.spawn(TextBundle::from_section(
                            outcome_headline(&summary),
                            TextStyle {
                                font_size: 18.0,
                                color: UiTheme::muted_cream(),
                                ..default()
                            },
                        ));
                        panel.spawn(caption_text(reward_digest(&summary)));
                        panel.spawn(section_title("Signal events"));
                        panel
                            .spawn(NodeBundle {
                                style: Style {
                                    width: Val::Percent(100.0),
                                    max_height: Val::Px(220.0),
                                    padding: UiRect::all(Val::Px(10.0)),
                                    flex_direction: FlexDirection::Column,
                                    align_items: AlignItems::FlexStart,
                                    row_gap: Val::Px(4.0),
                                    overflow: Overflow::clip_y(),
                                    border: UiRect::all(Val::Px(1.0)),
                                    ..default()
                                },
                                background_color: UiTheme::panel_bg_deep().into(),
                                border_color: BorderColor(UiTheme::panel_border_inner()),
                                ..default()
                            })
                            .with_children(|log| {
                                for line in summary.log.iter() {
                                    let (color, size) = log_line_present(line);
                                    log.spawn(TextBundle::from_section(
                                        line.clone(),
                                        TextStyle {
                                            font_size: size,
                                            color,
                                            ..default()
                                        },
                                    ));
                                }
                            });
                        panel
                            .spawn((
                                ButtonBundle {
                                    style: Style {
                                        width: Val::Percent(100.0),
                                        max_width: Val::Px(360.0),
                                        height: Val::Px(52.0),
                                        justify_content: JustifyContent::Center,
                                        align_items: AlignItems::Center,
                                        align_self: AlignSelf::FlexStart,
                                        margin: UiRect::top(Val::Px(10.0)),
                                        ..default()
                                    },
                                    background_color: UiTheme::muted_red().into(),
                                    ..default()
                                },
                                AcceptRewardsButton,
                            ))
                            .with_children(|button| {
                                button.spawn(TextBundle::from_section(
                                    "Accept rewards & continue",
                                    TextStyle {
                                        font_size: 18.0,
                                        color: Color::WHITE,
                                        ..default()
                                    },
                                ));
                            });
                    });
                });
                spawn_bottom_strip(col, |strip| {
                    strip.spawn(section_title("Loot arriving"));
                    strip.spawn(caption_text(format!(
                        "{} pieces earmarked for stash · Salvage +{}",
                        summary.loot.len(),
                        summary.salvage_earned
                    )));
                });
            });
        });
}

fn spawn_upgrade_screen(
    mut commands: Commands,
    profile: Res<ProfileState>,
    speed: Res<RunSpeedSetting>,
) {
    spawn_upgrade_screen_root(&mut commands, &profile, speed.0);
}

fn spawn_upgrade_screen_root(commands: &mut Commands, profile: &ProfileState, speed_mult: f32) {
    let hero = profile.effective_hero();
    let stats = hero.derived_stats();
    let meta = &profile.profile.meta;
    let inventory = &profile.profile.inventory;

    commands
        .spawn((root_shell(), UiRoot, UpgradeScreen))
        .with_children(|root| {
            spawn_atmosphere(root);
            root.spawn(content_column_bundle()).with_children(|col| {
                spawn_top_resource_bar(
                    col,
                    meta.gold,
                    meta.salvage,
                    meta.unlocked_skill_slots,
                    "—",
                    speed_mult,
                );
                col.spawn(main_split_row_bundle()).with_children(|row| {
                    spawn_framed_panel(row, 1.0, |panel| {
                        panel.spawn(headline_text("Delver"));
                        panel.spawn(body_text(format!(
                            "HP {} · DMG {} · ARM {} · HEAL {}",
                            stats.max_health, stats.damage, stats.armor, stats.healing_power
                        )));
                        spawn_skills_section(panel, &hero);
                        spawn_gear_slots(panel, profile);
                        panel
                            .spawn((
                                ButtonBundle {
                                    style: Style {
                                        width: Val::Percent(100.0),
                                        max_width: Val::Px(300.0),
                                        height: Val::Px(50.0),
                                        justify_content: JustifyContent::Center,
                                        align_items: AlignItems::Center,
                                        align_self: AlignSelf::FlexStart,
                                        margin: UiRect::top(Val::Px(12.0)),
                                        ..default()
                                    },
                                    background_color: UiTheme::muted_red().into(),
                                    ..default()
                                },
                                ReturnToBuildButton,
                            ))
                            .with_children(|button| {
                                button.spawn(TextBundle::from_section(
                                    "Return to bastion briefing",
                                    TextStyle {
                                        font_size: 17.0,
                                        color: Color::WHITE,
                                        ..default()
                                    },
                                ));
                            });
                    });
                    spawn_framed_panel(row, 1.05, |panel| {
                        panel.spawn(headline_text("Expedition theater"));
                        panel.spawn(body_text(
                            "Live encounter feed arrives in a later milestone — review chronicles after each delve for now.",
                        ));
                        panel.spawn(caption_text(
                            "Tip: keep the log concise by running shorter seeds while iterating loadouts.",
                        ));
                        panel.spawn(section_title("Camp actions"));
                        panel.spawn(body_text(
                            "Stash, forging contracts, and meta upgrades live in the band below.",
                        ));
                    });
                });
                spawn_bottom_strip(col, |strip| {
                    strip.spawn(section_title("Stash & caravan contracts"));
                    if inventory.is_empty() {
                        strip.spawn(caption_text("Stash is empty — the dungeon owes you loot."));
                    } else {
                        strip.spawn(NodeBundle {
                            style: Style {
                                width: Val::Percent(100.0),
                                flex_direction: FlexDirection::Column,
                                row_gap: Val::Px(10.0),
                                align_items: AlignItems::Stretch,
                                ..default()
                            },
                            ..default()
                        })
                        .with_children(|list| {
                            for item in inventory.iter().take(8) {
                                spawn_item_card(list, item);
                            }
                        });
                    }
                    strip.spawn(section_title("Permanent upgrades"));
                    strip.spawn(NodeBundle {
                        style: Style {
                            width: Val::Percent(100.0),
                            flex_direction: FlexDirection::Row,
                            flex_wrap: FlexWrap::Wrap,
                            align_items: AlignItems::Center,
                            column_gap: Val::Px(8.0),
                            row_gap: Val::Px(8.0),
                            ..default()
                        },
                        ..default()
                    })
                    .with_children(|grid| {
                        for upgrade in [
                            UpgradeId::MaxHealth,
                            UpgradeId::BaseDamage,
                            UpgradeId::Armor,
                            UpgradeId::HealingPower,
                            UpgradeId::GoldGain,
                        ] {
                            grid.spawn((
                                ButtonBundle {
                                    style: Style {
                                        min_width: Val::Px(200.0),
                                        height: Val::Px(40.0),
                                        justify_content: JustifyContent::Center,
                                        align_items: AlignItems::Center,
                                        padding: UiRect::axes(Val::Px(12.0), Val::Px(6.0)),
                                        ..default()
                                    },
                                    background_color: UiTheme::panel_bg_deep().into(),
                                    border_color: BorderColor(UiTheme::panel_border()),
                                    ..default()
                                },
                                BuyUpgradeButton { upgrade },
                            ))
                            .with_children(|button| {
                                button.spawn(TextBundle::from_section(
                                    format!(
                                        "{upgrade:?} L{} — {}g",
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
                    });
                });
            });
        });
}

fn refresh_upgrade_screen_on_profile_change(
    mut commands: Commands,
    profile: Res<ProfileState>,
    speed: Res<RunSpeedSetting>,
    upgrade_roots: Query<Entity, With<UpgradeScreen>>,
) {
    if !profile.is_changed() || upgrade_roots.is_empty() {
        return;
    }

    for root in &upgrade_roots {
        commands.entity(root).despawn_recursive();
    }
    spawn_upgrade_screen_root(&mut commands, &profile, speed.0);
}

fn spawn_skills_section(parent: &mut ChildBuilder, hero: &crate::domain::hero::HeroProfile) {
    parent.spawn(section_title("Skills"));
    let mut any = false;
    for slot in 0..hero.unlocked_skill_slots.min(hero.equipped_skills.len()) {
        if let Some(skill) = hero.equipped_skills[slot] {
            any = true;
            let def = skill_definition(skill);
            parent.spawn(body_text(format!("· {} — {}", def.name, def.description)));
        }
    }
    if !any {
        parent.spawn(caption_text("No skills equipped in unlocked slots."));
    }
}

fn spawn_gear_slots(parent: &mut ChildBuilder, profile: &ProfileState) {
    parent.spawn(section_title("Equipment"));
    spawn_gear_slot(
        parent,
        "Weapon",
        GearSlot::Weapon,
        profile.profile.hero.equipped_item(GearSlot::Weapon),
    );
    spawn_gear_slot(
        parent,
        "Armor",
        GearSlot::Armor,
        profile.profile.hero.equipped_item(GearSlot::Armor),
    );
    spawn_gear_slot(
        parent,
        "Trinket",
        GearSlot::Trinket,
        profile.profile.hero.equipped_item(GearSlot::Trinket),
    );
}

fn spawn_gear_slot(
    parent: &mut ChildBuilder,
    label: &str,
    _slot: GearSlot,
    item: Option<&ItemInstance>,
) {
    parent
        .spawn(NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                padding: UiRect::all(Val::Px(10.0)),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::FlexStart,
                row_gap: Val::Px(4.0),
                border: UiRect::all(Val::Px(1.0)),
                margin: UiRect::bottom(Val::Px(8.0)),
                ..default()
            },
            background_color: UiTheme::panel_bg_deep().into(),
            border_color: BorderColor(UiTheme::panel_border_inner()),
            ..default()
        })
        .with_children(|slot| {
            slot.spawn(caption_text(format!("[{label} slot]")));
            if let Some(item) = item {
                slot.spawn(TextBundle::from_section(
                    &item.name,
                    TextStyle {
                        font_size: 16.0,
                        color: rarity_color(item.rarity),
                        ..default()
                    },
                ));
                slot.spawn(caption_text(format!("{:?} · {:?}", item.rarity, item.slot)));
                slot.spawn(caption_text(format_item_stat_summary(item)));
            } else {
                slot.spawn(body_text("Empty — assign from stash below."));
            }
        });
}

fn spawn_item_card(parent: &mut ChildBuilder, item: &ItemInstance) {
    parent
        .spawn(NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                padding: UiRect::all(Val::Px(12.0)),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::FlexStart,
                row_gap: Val::Px(8.0),
                border: UiRect::all(Val::Px(1.0)),
                ..default()
            },
            background_color: UiTheme::panel_bg_deep().into(),
            border_color: BorderColor(rarity_color(item.rarity).mix(&Color::BLACK, 0.45)),
            ..default()
        })
        .with_children(|card| {
            card.spawn(TextBundle::from_section(
                &item.name,
                TextStyle {
                    font_size: 17.0,
                    color: rarity_color(item.rarity),
                    ..default()
                },
            ));
            card.spawn(caption_text(format!("{:?} · {:?}", item.rarity, item.slot)));
            card.spawn(body_text(format_item_stat_summary(item)));
            card.spawn(NodeBundle {
                style: Style {
                    flex_direction: FlexDirection::Row,
                    column_gap: Val::Px(10.0),
                    align_items: AlignItems::Center,
                    ..default()
                },
                ..default()
            })
            .with_children(|row| {
                row.spawn((
                    ButtonBundle {
                        style: Style {
                            width: Val::Px(108.0),
                            height: Val::Px(36.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        background_color: UiTheme::muted_red().into(),
                        ..default()
                    },
                    EquipItemButton { item_id: item.id },
                ))
                .with_children(|b| {
                    b.spawn(TextBundle::from_section(
                        "Equip",
                        TextStyle {
                            font_size: 15.0,
                            color: Color::WHITE,
                            ..default()
                        },
                    ));
                });
                row.spawn((
                    ButtonBundle {
                        style: Style {
                            width: Val::Px(108.0),
                            height: Val::Px(36.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        background_color: UiTheme::panel_bg().into(),
                        border_color: BorderColor(UiTheme::panel_border()),
                        ..default()
                    },
                    SalvageItemButton { item_id: item.id },
                ))
                .with_children(|b| {
                    b.spawn(TextBundle::from_section(
                        "Salvage",
                        TextStyle {
                            font_size: 15.0,
                            color: UiTheme::body(),
                            ..default()
                        },
                    ));
                });
            });
        });
}

fn sync_top_bar(
    state: Res<State<GameState>>,
    profile: Res<ProfileState>,
    latest_summary: Option<Res<LatestRunSummary>>,
    speed: Res<RunSpeedSetting>,
    mut q: Query<(&TopBarField, &mut Text)>,
) {
    let meta = &profile.profile.meta;
    let depth = match state.get() {
        GameState::Summary => latest_summary
            .as_deref()
            .map(|s| s.summary.deepest_depth.to_string())
            .unwrap_or_else(|| "—".to_string()),
        _ => "—".to_string(),
    };
    for (field, mut text) in &mut q {
        let value = match field {
            TopBarField::Gold => format!("Gold: {}", meta.gold),
            TopBarField::Salvage => format!("Salvage: {}", meta.salvage),
            TopBarField::SkillSlots => format!("Skills: {}", meta.unlocked_skill_slots),
            TopBarField::Depth => format!("Depth: {depth}"),
            TopBarField::Speed => format!(
                "Speed: {}x",
                if (speed.0 - 1.0).abs() < f32::EPSILON {
                    "1".to_string()
                } else if (speed.0 - 2.0).abs() < f32::EPSILON {
                    "2".to_string()
                } else {
                    format!("{:.1}", speed.0)
                }
            ),
        };
        if text.sections[0].value != value {
            text.sections[0].value = value;
        }
    }
}

fn handle_settings_button(
    mut interactions: Query<
        (&Interaction, &mut BorderColor),
        (Changed<Interaction>, With<SettingsButton>),
    >,
    mut speed: ResMut<RunSpeedSetting>,
) {
    let next = if (speed.0 - 1.0).abs() < f32::EPSILON {
        2.0
    } else {
        1.0
    };
    for (interaction, mut border) in &mut interactions {
        match *interaction {
            Interaction::Pressed => {
                speed.0 = next;
                *border = BorderColor(UiTheme::accent_red());
            }
            Interaction::Hovered => {
                *border = BorderColor(UiTheme::muted_gold());
            }
            Interaction::None => {
                *border = BorderColor::DEFAULT;
            }
        }
    }
}

fn handle_start_button(
    mut interactions: Query<
        (&Interaction, &mut BackgroundColor, &mut BorderColor),
        (Changed<Interaction>, With<StartRunButton>),
    >,
    mut start_run_events: EventWriter<StartRun>,
) {
    for (interaction, mut color, mut border) in &mut interactions {
        match *interaction {
            Interaction::Pressed => {
                *color = UiTheme::accent_red().into();
                *border = BorderColor(UiTheme::accent_red());
                start_run_events.send(StartRun { seed: 1 });
            }
            Interaction::Hovered => {
                *color = UiTheme::muted_red().into();
                *border = BorderColor(UiTheme::muted_gold());
            }
            Interaction::None => {
                *color = UiTheme::muted_red().into();
                *border = BorderColor::DEFAULT;
            }
        }
    }
}

fn handle_accept_button(
    mut interactions: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<AcceptRewardsButton>),
    >,
    mut events: EventWriter<AcceptRunRewards>,
) {
    for (interaction, mut color) in &mut interactions {
        match *interaction {
            Interaction::Pressed => {
                *color = UiTheme::accent_red().into();
                events.send(AcceptRunRewards);
            }
            Interaction::Hovered => {
                *color = UiTheme::muted_red().into();
            }
            Interaction::None => {
                *color = UiTheme::muted_red().into();
            }
        }
    }
}

fn handle_equip_buttons(
    mut interactions: Query<(&Interaction, &EquipItemButton), Changed<Interaction>>,
    mut events: EventWriter<EquipInventoryItem>,
) {
    for (interaction, button) in &mut interactions {
        if *interaction == Interaction::Pressed {
            events.send(EquipInventoryItem {
                item_id: button.item_id,
            });
        }
    }
}

fn handle_salvage_buttons(
    mut interactions: Query<(&Interaction, &SalvageItemButton), Changed<Interaction>>,
    mut events: EventWriter<SalvageInventoryItem>,
) {
    for (interaction, button) in &mut interactions {
        if *interaction == Interaction::Pressed {
            events.send(SalvageInventoryItem {
                item_id: button.item_id,
            });
        }
    }
}

fn handle_buy_upgrade_buttons(
    mut interactions: Query<(&Interaction, &BuyUpgradeButton), Changed<Interaction>>,
    mut events: EventWriter<BuyUpgrade>,
) {
    for (interaction, button) in &mut interactions {
        if *interaction == Interaction::Pressed {
            events.send(BuyUpgrade {
                upgrade: button.upgrade,
            });
        }
    }
}

fn handle_return_to_build_button(
    mut interactions: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<ReturnToBuildButton>),
    >,
    mut events: EventWriter<ReturnToBuild>,
) {
    for (interaction, mut color) in &mut interactions {
        match *interaction {
            Interaction::Pressed => {
                *color = UiTheme::accent_red().into();
                events.send(ReturnToBuild);
            }
            Interaction::Hovered => {
                *color = UiTheme::muted_red().into();
            }
            Interaction::None => {
                *color = UiTheme::muted_red().into();
            }
        }
    }
}

fn cleanup_ui(mut commands: Commands, roots: Query<Entity, With<UiRoot>>) {
    for root in &roots {
        commands.entity(root).despawn_recursive();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::{
        AcceptRunRewards, GameState, IdleDungeonsPlugin, LatestRunSummary, ProfileSavePath,
        ProfileState, StartRun,
    };
    use crate::domain::items::{GearSlot, ItemInstance};
    use crate::domain::progression::UpgradeId;
    use bevy::state::app::StatesPlugin;
    use tempfile::tempdir;

    #[test]
    fn ui_plugin_spawns_build_screen_with_start_button() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(StatesPlugin);
        app.add_plugins(IdleDungeonsPlugin);
        app.add_plugins(UiPlugin);

        app.update();

        assert_eq!(entity_count::<MainCamera>(app.world_mut()), 1);
        assert_eq!(entity_count::<StartRunButton>(app.world_mut()), 1);
    }

    #[test]
    fn pressing_start_button_sends_start_run_event() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(StatesPlugin);
        app.add_plugins(IdleDungeonsPlugin);
        app.add_plugins(UiPlugin);
        app.update();

        let button = single_entity::<StartRunButton>(app.world_mut());
        app.world_mut()
            .entity_mut(button)
            .insert(Interaction::Pressed);

        app.update();

        let events = app.world().resource::<Events<StartRun>>();
        assert_eq!(events.len(), 1);
    }

    #[test]
    fn summary_state_spawns_summary_screen() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(StatesPlugin);
        app.add_plugins(IdleDungeonsPlugin);
        app.add_plugins(UiPlugin);
        app.world_mut().send_event(StartRun { seed: 1 });

        app.update();
        app.update();

        assert_eq!(
            *app.world().resource::<State<GameState>>().get(),
            GameState::Summary
        );
        assert!(app.world().contains_resource::<LatestRunSummary>());
        assert_eq!(entity_count::<SummaryScreen>(app.world_mut()), 1);
    }

    #[test]
    fn summary_screen_has_accept_rewards_button() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(StatesPlugin);
        app.add_plugins(IdleDungeonsPlugin);
        app.add_plugins(UiPlugin);
        app.world_mut().send_event(StartRun { seed: 1 });

        app.update();
        app.update();

        assert_eq!(entity_count::<AcceptRewardsButton>(app.world_mut()), 1);
    }

    #[test]
    fn pressing_accept_rewards_button_sends_event() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(StatesPlugin);
        app.add_plugins(IdleDungeonsPlugin);
        app.add_plugins(UiPlugin);
        app.world_mut().send_event(StartRun { seed: 1 });
        app.update();
        app.update();

        let button = single_entity::<AcceptRewardsButton>(app.world_mut());
        app.world_mut()
            .entity_mut(button)
            .insert(Interaction::Pressed);
        app.update();

        assert_eq!(app.world().resource::<Events<AcceptRunRewards>>().len(), 1);
    }

    #[test]
    fn upgrades_screen_has_inventory_and_upgrade_buttons() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(StatesPlugin);
        app.add_plugins(IdleDungeonsPlugin);
        app.add_plugins(UiPlugin);
        app.world_mut()
            .resource_mut::<ProfileState>()
            .profile
            .inventory
            .push(ItemInstance::basic(1, "Iron Sword", GearSlot::Weapon));
        app.world_mut()
            .resource_mut::<NextState<GameState>>()
            .set(GameState::Upgrades);

        app.update();
        app.update();

        assert_eq!(entity_count::<UpgradeScreen>(app.world_mut()), 1);
        assert!(entity_count::<EquipItemButton>(app.world_mut()) >= 1);
        assert!(entity_count::<SalvageItemButton>(app.world_mut()) >= 1);
        assert!(entity_count::<BuyUpgradeButton>(app.world_mut()) >= 1);
        assert_eq!(entity_count::<ReturnToBuildButton>(app.world_mut()), 1);
    }

    #[test]
    fn pressing_upgrade_button_updates_profile_and_refreshes_screen_text() {
        let dir = tempdir().unwrap();
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(StatesPlugin);
        app.insert_resource(ProfileSavePath(dir.path().join("profile.json")));
        app.add_plugins(IdleDungeonsPlugin);
        app.add_plugins(UiPlugin);
        app.update();

        app.world_mut()
            .resource_mut::<ProfileState>()
            .profile
            .meta
            .gold = 100;
        app.world_mut()
            .resource_mut::<NextState<GameState>>()
            .set(GameState::Upgrades);
        app.update();
        app.update();

        let button = app
            .world_mut()
            .query_filtered::<(Entity, &BuyUpgradeButton), ()>()
            .iter(app.world())
            .find_map(|(entity, button)| {
                (button.upgrade == UpgradeId::BaseDamage).then_some(entity)
            })
            .unwrap();
        app.world_mut()
            .entity_mut(button)
            .insert(Interaction::Pressed);

        app.update();
        app.update();

        let profile = app.world().resource::<ProfileState>();
        assert_eq!(profile.profile.meta.gold, 90);
        assert_eq!(profile.profile.meta.upgrade_level(UpgradeId::BaseDamage), 1);

        let dumped = all_ui_text(app.world_mut());
        assert!(
            dumped.contains("Gold: 90"),
            "expected refreshed gold line in UI, got: {dumped}"
        );
        assert!(
            dumped.contains("BaseDamage L1"),
            "expected refreshed upgrade label in UI, got: {dumped}"
        );
    }

    #[test]
    fn pressing_upgrade_button_sends_buy_upgrade_event() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(StatesPlugin);
        app.add_plugins(IdleDungeonsPlugin);
        app.add_plugins(UiPlugin);
        app.world_mut()
            .resource_mut::<NextState<GameState>>()
            .set(GameState::Upgrades);
        app.update();
        app.update();

        let button = app
            .world_mut()
            .query_filtered::<(Entity, &BuyUpgradeButton), ()>()
            .iter(app.world())
            .find_map(|(entity, button)| {
                (button.upgrade == UpgradeId::BaseDamage).then_some(entity)
            })
            .unwrap();
        app.world_mut()
            .entity_mut(button)
            .insert(Interaction::Pressed);
        app.update();

        assert_eq!(
            app.world()
                .resource::<Events<crate::app::BuyUpgrade>>()
                .len(),
            1
        );
    }

    fn entity_count<T: Component>(world: &mut World) -> usize {
        let mut query = world.query_filtered::<Entity, With<T>>();
        query.iter(world).count()
    }

    fn single_entity<T: Component>(world: &mut World) -> Entity {
        let mut query = world.query_filtered::<Entity, With<T>>();
        query.single(world)
    }

    fn all_ui_text(world: &mut World) -> String {
        world
            .query::<&Text>()
            .iter(world)
            .flat_map(|text| text.sections.iter().map(|s| s.value.as_str()))
            .collect::<Vec<_>>()
            .join(" | ")
    }
}
