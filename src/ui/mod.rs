pub mod build_panel;
pub mod components;
pub mod inventory_panel;
pub mod log_panel;
pub mod mockup_layout;
pub mod run_panel;
pub mod summary_panel;
pub mod theme;
pub mod upgrade_panel;
pub mod widgets;

use crate::app::{
    AcceptRunRewards, BuyUpgrade, EquipInventoryItem, GameState, LatestRunSummary, ProfileState,
    ReturnToBuild, RunSpeedSetting, SalvageInventoryItem, StartRun,
};
use crate::domain::items::ItemInstance;
use crate::domain::run::RunSummary;
use crate::ui::build_panel::build_panel_text;
use crate::ui::components::*;
use crate::ui::mockup_layout::RightPanelTab;
use crate::ui::theme::{body_text, caption_text, format_item_stat_summary, rarity_color, UiTheme};
use crate::ui::widgets::spawn_atmosphere;
use bevy::input::mouse::MouseWheel;
use bevy::prelude::*;
use bevy::transform::TransformSystem;
use bevy::ui::RelativeCursorPosition;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<RightPanelTab>();
        app.add_event::<MouseWheel>();
        app.add_systems(Startup, spawn_camera)
            .add_systems(
                OnEnter(GameState::Build),
                (reset_right_tab_inventory, spawn_build_screen).chain(),
            )
            .add_systems(OnExit(GameState::Build), cleanup_ui)
            .add_systems(
                Update,
                (
                    apply_ui_button_palettes,
                    handle_start_button.run_if(in_state(GameState::Build)),
                    sync_top_bar,
                    handle_settings_button,
                    handle_right_panel_tab_buttons,
                ),
            )
            .add_systems(
                OnEnter(GameState::Summary),
                (reset_right_tab_loot, spawn_summary_screen).chain(),
            )
            .add_systems(OnExit(GameState::Summary), cleanup_ui)
            .add_systems(
                OnEnter(GameState::Upgrades),
                (reset_right_tab_camp, spawn_upgrade_screen).chain(),
            )
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
                (
                    apply_ui_scroll.after(TransformSystem::TransformPropagate),
                    refresh_upgrade_screen_on_profile_change.run_if(in_state(GameState::Upgrades)),
                ),
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

fn reset_right_tab_inventory(mut tab: ResMut<RightPanelTab>) {
    *tab = RightPanelTab::Inventory;
}

fn reset_right_tab_loot(mut tab: ResMut<RightPanelTab>) {
    *tab = RightPanelTab::Loot;
}

fn reset_right_tab_camp(mut tab: ResMut<RightPanelTab>) {
    *tab = RightPanelTab::Inventory;
}

fn spawn_build_screen(
    mut commands: Commands,
    profile: Res<ProfileState>,
    speed: Res<RunSpeedSetting>,
    tab: Res<RightPanelTab>,
) {
    spawn_build_screen_root(&mut commands, &profile, speed.0, *tab);
}

fn spawn_build_screen_root(
    commands: &mut Commands,
    profile: &ProfileState,
    speed_mult: f32,
    tab: RightPanelTab,
) {
    let hero = profile.effective_hero();
    let meta = &profile.profile.meta;
    let loadout_lines: Vec<String> = build_panel_text(&hero)
        .lines()
        .map(|s| s.to_string())
        .collect();
    let stash = profile.profile.inventory.len();

    commands
        .spawn((root_shell(), UiRoot, BuildScreen))
        .with_children(|root| {
            spawn_atmosphere(root);
            root.spawn(content_column_bundle()).with_children(|col| {
                crate::ui::mockup_layout::spawn_mockup_header(
                    col,
                    meta.gold,
                    meta.salvage,
                    meta.unlocked_skill_slots,
                    hero.equipped_skills.len(),
                    "—",
                    speed_mult,
                );
                crate::ui::mockup_layout::spawn_three_column_row(col, |row| {
                    crate::ui::mockup_layout::spawn_ornate_column(row, 0.95, |panel| {
                        crate::ui::mockup_layout::spawn_hero_column_mockup(
                            panel,
                            &hero,
                            profile,
                            &loadout_lines,
                        );
                    });
                    crate::ui::mockup_layout::spawn_ornate_column(row, 1.05, |panel| {
                        crate::ui::mockup_layout::spawn_dungeon_briefing_column(panel, stash);
                    });
                    crate::ui::mockup_layout::spawn_ornate_column(row, 1.0, |panel| {
                        crate::ui::mockup_layout::spawn_right_management_column(
                            panel,
                            tab,
                            meta,
                            &profile.profile.inventory,
                            None,
                            false,
                        );
                    });
                });
                crate::ui::mockup_layout::spawn_mockup_footer(
                    col,
                    crate::ui::mockup_layout::FooterMode::Briefing,
                );
            });
        });
}

fn spawn_summary_screen(
    mut commands: Commands,
    profile: Res<ProfileState>,
    latest_summary: Option<Res<LatestRunSummary>>,
    speed: Res<RunSpeedSetting>,
    tab: Res<RightPanelTab>,
) {
    let summary = latest_summary
        .as_deref()
        .map(|s| s.summary.clone())
        .unwrap_or_else(crate::ui::summary_panel::empty_run_summary);
    spawn_summary_screen_root(&mut commands, &profile, &summary, speed.0, *tab);
}

fn spawn_summary_screen_root(
    commands: &mut Commands,
    profile: &ProfileState,
    summary: &RunSummary,
    speed_mult: f32,
    tab: RightPanelTab,
) {
    let meta = &profile.profile.meta;
    let hero = profile.effective_hero();
    let loadout_lines: Vec<String> = build_panel_text(&hero)
        .lines()
        .map(|s| s.to_string())
        .collect();

    commands
        .spawn((root_shell(), UiRoot, SummaryScreen))
        .with_children(|root| {
            spawn_atmosphere(root);
            root.spawn(content_column_bundle()).with_children(|col| {
                crate::ui::mockup_layout::spawn_mockup_header(
                    col,
                    meta.gold,
                    meta.salvage,
                    meta.unlocked_skill_slots,
                    hero.equipped_skills.len(),
                    &summary.deepest_depth.to_string(),
                    speed_mult,
                );
                crate::ui::mockup_layout::spawn_three_column_row(col, |row| {
                    crate::ui::mockup_layout::spawn_ornate_column(row, 0.95, |panel| {
                        crate::ui::mockup_layout::spawn_hero_column_mockup(
                            panel,
                            &hero,
                            profile,
                            &loadout_lines,
                        );
                    });
                    crate::ui::mockup_layout::spawn_ornate_column(row, 1.05, |panel| {
                        crate::ui::mockup_layout::spawn_dungeon_summary_column(panel, summary);
                    });
                    crate::ui::mockup_layout::spawn_ornate_column(row, 1.0, |panel| {
                        crate::ui::mockup_layout::spawn_right_management_column(
                            panel,
                            tab,
                            meta,
                            &profile.profile.inventory,
                            Some(summary.loot.as_slice()),
                            false,
                        );
                    });
                });
                crate::ui::mockup_layout::spawn_mockup_footer(
                    col,
                    crate::ui::mockup_layout::FooterMode::Summary,
                );
            });
        });
}

fn spawn_upgrade_screen(
    mut commands: Commands,
    profile: Res<ProfileState>,
    speed: Res<RunSpeedSetting>,
    tab: Res<RightPanelTab>,
) {
    spawn_upgrade_screen_root(&mut commands, &profile, speed.0, *tab);
}

fn spawn_upgrade_screen_root(
    commands: &mut Commands,
    profile: &ProfileState,
    speed_mult: f32,
    tab: RightPanelTab,
) {
    let hero = profile.effective_hero();
    let meta = &profile.profile.meta;
    let loadout_lines: Vec<String> = build_panel_text(&hero)
        .lines()
        .map(|s| s.to_string())
        .collect();
    let inventory = &profile.profile.inventory;

    commands
        .spawn((root_shell(), UiRoot, UpgradeScreen))
        .with_children(|root| {
            spawn_atmosphere(root);
            root.spawn(content_column_bundle()).with_children(|col| {
                crate::ui::mockup_layout::spawn_mockup_header(
                    col,
                    meta.gold,
                    meta.salvage,
                    meta.unlocked_skill_slots,
                    hero.equipped_skills.len(),
                    "—",
                    speed_mult,
                );
                crate::ui::mockup_layout::spawn_three_column_row(col, |row| {
                    crate::ui::mockup_layout::spawn_ornate_column(row, 0.95, |panel| {
                        crate::ui::mockup_layout::spawn_hero_column_mockup(
                            panel,
                            &hero,
                            profile,
                            &loadout_lines,
                        );
                    });
                    crate::ui::mockup_layout::spawn_ornate_column(row, 1.05, |panel| {
                        crate::ui::mockup_layout::spawn_dungeon_camp_column(panel);
                    });
                    crate::ui::mockup_layout::spawn_ornate_column(row, 1.0, |panel| {
                        crate::ui::mockup_layout::spawn_right_management_column(
                            panel, tab, meta, inventory, None, true,
                        );
                    });
                });
                crate::ui::mockup_layout::spawn_mockup_footer(
                    col,
                    crate::ui::mockup_layout::FooterMode::Camp,
                );
            });
        });
}

fn handle_right_panel_tab_buttons(
    mut interactions: Query<
        (&Interaction, &crate::ui::mockup_layout::RightTabButton),
        Changed<Interaction>,
    >,
    mut tab: ResMut<RightPanelTab>,
    mut commands: Commands,
    profile: Res<ProfileState>,
    speed: Res<RunSpeedSetting>,
    latest_summary: Option<Res<LatestRunSummary>>,
    state: Res<State<GameState>>,
    build_roots: Query<Entity, With<BuildScreen>>,
    upgrade_roots: Query<Entity, With<UpgradeScreen>>,
    summary_roots: Query<Entity, With<SummaryScreen>>,
) {
    for (interaction, btn) in &mut interactions {
        if *interaction != Interaction::Pressed {
            continue;
        }
        if *tab == btn.0 {
            continue;
        }
        *tab = btn.0;

        match state.get() {
            GameState::Build => {
                for e in &build_roots {
                    commands.entity(e).despawn_recursive();
                }
                spawn_build_screen_root(&mut commands, &profile, speed.0, *tab);
            }
            GameState::Upgrades => {
                for e in &upgrade_roots {
                    commands.entity(e).despawn_recursive();
                }
                spawn_upgrade_screen_root(&mut commands, &profile, speed.0, *tab);
            }
            GameState::Summary => {
                let summary = latest_summary
                    .as_deref()
                    .map(|s| s.summary.clone())
                    .unwrap_or_else(crate::ui::summary_panel::empty_run_summary);
                for e in &summary_roots {
                    commands.entity(e).despawn_recursive();
                }
                spawn_summary_screen_root(&mut commands, &profile, &summary, speed.0, *tab);
            }
            GameState::Running => {}
        }
    }
}

fn refresh_upgrade_screen_on_profile_change(
    mut commands: Commands,
    profile: Res<ProfileState>,
    speed: Res<RunSpeedSetting>,
    tab: Res<RightPanelTab>,
    upgrade_roots: Query<Entity, With<UpgradeScreen>>,
) {
    if !profile.is_changed() || upgrade_roots.is_empty() {
        return;
    }

    for root in &upgrade_roots {
        commands.entity(root).despawn_recursive();
    }
    spawn_upgrade_screen_root(&mut commands, &profile, speed.0, *tab);
}

pub(crate) fn spawn_item_card(parent: &mut ChildBuilder, item: &ItemInstance) {
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
                let equip_pal = UiButtonPalette::equip();
                row.spawn((
                    ButtonBundle {
                        style: Style {
                            width: Val::Px(108.0),
                            height: Val::Px(36.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            border: UiRect::all(Val::Px(1.0)),
                            ..default()
                        },
                        background_color: equip_pal.idle_bg.into(),
                        border_color: BorderColor(equip_pal.idle_border),
                        ..default()
                    },
                    EquipItemButton { item_id: item.id },
                    equip_pal,
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
                let salvage_pal = UiButtonPalette::salvage();
                row.spawn((
                    ButtonBundle {
                        style: Style {
                            width: Val::Px(108.0),
                            height: Val::Px(36.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            border: UiRect::all(Val::Px(1.0)),
                            ..default()
                        },
                        background_color: salvage_pal.idle_bg.into(),
                        border_color: BorderColor(salvage_pal.idle_border),
                        ..default()
                    },
                    SalvageItemButton { item_id: item.id },
                    salvage_pal,
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
    let hero = profile.effective_hero();
    let depth = match state.get() {
        GameState::Summary => latest_summary
            .as_deref()
            .map(|s| s.summary.deepest_depth.to_string())
            .unwrap_or_else(|| "—".to_string()),
        _ => "—".to_string(),
    };
    let skill_cap = hero.equipped_skills.len().max(1);
    for (field, mut text) in &mut q {
        let value = match field {
            TopBarField::Gold => format!("{}", meta.gold),
            TopBarField::Salvage => format!("{}", meta.salvage),
            TopBarField::SkillSlots => {
                format!("{}/{}", meta.unlocked_skill_slots, skill_cap)
            }
            TopBarField::Depth => depth.clone(),
            TopBarField::Speed => {
                if (speed.0 - 1.0).abs() < f32::EPSILON {
                    "1x".to_string()
                } else if (speed.0 - 2.0).abs() < f32::EPSILON {
                    "2x".to_string()
                } else {
                    format!("{:.1}x", speed.0)
                }
            }
        };
        if text.sections[0].value != value {
            text.sections[0].value = value;
        }
    }
}

fn apply_ui_button_palettes(
    mut q: Query<
        (
            &Interaction,
            &UiButtonPalette,
            &mut BackgroundColor,
            &mut BorderColor,
        ),
        With<Button>,
    >,
) {
    for (interaction, pal, mut bg, mut border) in &mut q {
        let (cb, bo) = match *interaction {
            Interaction::None => (pal.idle_bg, pal.idle_border),
            Interaction::Hovered => (pal.hover_bg, pal.hover_border),
            Interaction::Pressed => (pal.pressed_bg, pal.pressed_border),
        };
        *bg = cb.into();
        *border = BorderColor(bo);
    }
}

fn handle_settings_button(
    mut interactions: Query<&Interaction, (Changed<Interaction>, With<SettingsButton>)>,
    mut speed: ResMut<RunSpeedSetting>,
) {
    let next = if (speed.0 - 1.0).abs() < f32::EPSILON {
        2.0
    } else {
        1.0
    };
    for interaction in &mut interactions {
        if *interaction == Interaction::Pressed {
            speed.0 = next;
        }
    }
}

fn handle_start_button(
    mut interactions: Query<&Interaction, (Changed<Interaction>, With<StartRunButton>)>,
    mut start_run_events: EventWriter<StartRun>,
) {
    for interaction in &mut interactions {
        if *interaction == Interaction::Pressed {
            start_run_events.send(StartRun { seed: 1 });
        }
    }
}

fn handle_accept_button(
    mut interactions: Query<&Interaction, (Changed<Interaction>, With<AcceptRewardsButton>)>,
    mut events: EventWriter<AcceptRunRewards>,
) {
    for interaction in &mut interactions {
        if *interaction == Interaction::Pressed {
            events.send(AcceptRunRewards);
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
    mut interactions: Query<&Interaction, (Changed<Interaction>, With<ReturnToBuildButton>)>,
    mut events: EventWriter<ReturnToBuild>,
) {
    for interaction in &mut interactions {
        if *interaction == Interaction::Pressed {
            events.send(ReturnToBuild);
        }
    }
}

fn apply_ui_scroll(
    mut wheel_events: EventReader<MouseWheel>,
    mut regions: Query<
        (Entity, &RelativeCursorPosition, &mut UiScrollState, &Node),
        With<UiScrollRegion>,
    >,
    children: Query<&Children>,
    mut content_style: Query<&mut Style, With<UiScrollContent>>,
    content_node: Query<&Node, With<UiScrollContent>>,
) {
    let delta: f32 = wheel_events.read().map(|e| e.y * 28.0).sum();
    if delta.abs() < f32::EPSILON {
        return;
    }

    for (entity, rel_pos, mut state, viewport_node) in &mut regions {
        if !rel_pos.mouse_over() {
            continue;
        }
        let view_h = viewport_node.size().y;
        if view_h <= 0.0 {
            continue;
        }
        let Ok(ch) = children.get(entity) else {
            continue;
        };
        let Some(child) = ch.iter().copied().find(|&e| content_node.get(e).is_ok()) else {
            continue;
        };
        let Ok(inner_node) = content_node.get(child) else {
            continue;
        };
        let content_h = inner_node.size().y;
        let max_scroll = (content_h - view_h).max(0.0);
        state.offset = (state.offset + delta).clamp(0.0, max_scroll);
        if let Ok(mut style) = content_style.get_mut(child) {
            style.top = Val::Px(-state.offset);
        }
        break;
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
    use crate::ui::mockup_layout::{RightPanelTab, RightTabButton};
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
        press_right_panel_tab(&mut app, RightPanelTab::Upgrades);
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

        press_right_panel_tab(&mut app, RightPanelTab::Upgrades);

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
            dumped.contains("90"),
            "expected refreshed gold value in UI, got: {dumped}"
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

        press_right_panel_tab(&mut app, RightPanelTab::Upgrades);

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

    fn press_right_panel_tab(app: &mut App, tab: RightPanelTab) {
        let entity = app
            .world_mut()
            .query::<(Entity, &RightTabButton)>()
            .iter(app.world())
            .find_map(|(e, b)| (b.0 == tab).then_some(e))
            .expect("tab header button");
        app.world_mut()
            .entity_mut(entity)
            .insert(Interaction::Pressed);
        app.update();
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
