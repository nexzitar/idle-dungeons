pub mod build_panel;
pub mod inventory_panel;
pub mod log_panel;
pub mod run_panel;
pub mod summary_panel;
pub mod upgrade_panel;

use crate::app::{
    AcceptRunRewards, BuyUpgrade, EquipInventoryItem, GameState, LatestRunSummary, ProfileState,
    ReturnToBuild, SalvageInventoryItem, StartRun,
};
use crate::domain::items::GearSlot;
use crate::domain::progression::UpgradeId;
use crate::ui::build_panel::build_panel_text;
use crate::ui::summary_panel::summary_panel_text;
use bevy::prelude::*;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_camera)
            .add_systems(OnEnter(GameState::Build), spawn_build_screen)
            .add_systems(OnExit(GameState::Build), cleanup_ui)
            .add_systems(
                Update,
                handle_start_button.run_if(in_state(GameState::Build)),
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
            );
    }
}

#[derive(Component)]
struct UiRoot;

#[derive(Component)]
struct MainCamera;

#[derive(Component)]
struct StartRunButton;

#[derive(Component)]
struct SummaryScreen;

#[derive(Component)]
struct UpgradeScreen;

#[derive(Component)]
struct AcceptRewardsButton;

#[derive(Component)]
struct EquipItemButton {
    item_id: u64,
}

#[derive(Component)]
struct SalvageItemButton {
    item_id: u64,
}

#[derive(Component)]
struct BuyUpgradeButton {
    upgrade: UpgradeId,
}

#[derive(Component)]
struct ReturnToBuildButton;

fn spawn_camera(mut commands: Commands) {
    commands.spawn((Camera2dBundle::default(), MainCamera));
}

fn spawn_build_screen(mut commands: Commands, profile: Res<ProfileState>) {
    let hero = profile.effective_hero();
    let build_text = build_panel_text(&hero);

    commands
        .spawn((
            NodeBundle {
                style: full_screen_column(),
                background_color: Color::srgb(0.03, 0.025, 0.04).into(),
                ..default()
            },
            UiRoot,
        ))
        .with_children(|parent| {
            parent.spawn(TextBundle::from_section(
                "Idle Dungeons",
                TextStyle {
                    font_size: 48.0,
                    color: Color::srgb(0.9, 0.82, 0.62),
                    ..default()
                },
            ));
            parent.spawn(TextBundle::from_section(
                "Configure an automated delver, start a run, and review the results.",
                TextStyle {
                    font_size: 20.0,
                    color: Color::srgb(0.75, 0.73, 0.8),
                    ..default()
                },
            ));
            parent.spawn(TextBundle::from_section(
                format!(
                    "{}\nGold: {} | Salvage: {} | Inventory: {} items",
                    build_text,
                    profile.profile.meta.gold,
                    profile.profile.meta.salvage,
                    profile.profile.inventory.len()
                ),
                TextStyle {
                    font_size: 20.0,
                    color: Color::WHITE,
                    ..default()
                },
            ));
            parent
                .spawn((
                    ButtonBundle {
                        style: Style {
                            width: Val::Px(220.0),
                            height: Val::Px(64.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            margin: UiRect::top(Val::Px(24.0)),
                            ..default()
                        },
                        background_color: Color::srgb(0.35, 0.12, 0.11).into(),
                        ..default()
                    },
                    StartRunButton,
                ))
                .with_children(|button| {
                    button.spawn(TextBundle::from_section(
                        "Start Run",
                        TextStyle {
                            font_size: 24.0,
                            color: Color::WHITE,
                            ..default()
                        },
                    ));
                });
        });
}

fn handle_start_button(
    mut interactions: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<StartRunButton>),
    >,
    mut start_run_events: EventWriter<StartRun>,
) {
    for (interaction, mut color) in &mut interactions {
        match *interaction {
            Interaction::Pressed => {
                *color = Color::srgb(0.55, 0.18, 0.15).into();
                start_run_events.send(StartRun { seed: 1 });
            }
            Interaction::Hovered => {
                *color = Color::srgb(0.45, 0.15, 0.13).into();
            }
            Interaction::None => {
                *color = Color::srgb(0.35, 0.12, 0.11).into();
            }
        }
    }
}

fn spawn_summary_screen(mut commands: Commands, latest_summary: Option<Res<LatestRunSummary>>) {
    let summary_text = latest_summary
        .as_deref()
        .map(|summary| summary_panel_text(&summary.summary))
        .unwrap_or_else(|| "No run summary yet.".to_string());

    commands
        .spawn((
            NodeBundle {
                style: full_screen_column(),
                background_color: Color::srgb(0.025, 0.03, 0.04).into(),
                ..default()
            },
            UiRoot,
            SummaryScreen,
        ))
        .with_children(|parent| {
            parent.spawn(TextBundle::from_section(
                "Run Summary",
                TextStyle {
                    font_size: 42.0,
                    color: Color::srgb(0.9, 0.82, 0.62),
                    ..default()
                },
            ));
            parent.spawn(TextBundle::from_section(
                summary_text,
                TextStyle {
                    font_size: 22.0,
                    color: Color::WHITE,
                    ..default()
                },
            ));
            parent
                .spawn((primary_button(), AcceptRewardsButton))
                .with_children(|button| {
                    button.spawn(button_text("Accept Rewards"));
                });
        });
}

fn spawn_upgrade_screen(mut commands: Commands, profile: Res<ProfileState>) {
    let hero = profile.effective_hero();
    let stats = hero.derived_stats();
    let meta = &profile.profile.meta;
    let inventory = &profile.profile.inventory;

    commands
        .spawn((
            NodeBundle {
                style: full_screen_column(),
                background_color: Color::srgb(0.025, 0.03, 0.04).into(),
                ..default()
            },
            UiRoot,
            UpgradeScreen,
        ))
        .with_children(|parent| {
            parent.spawn(TextBundle::from_section(
                "Delver Camp",
                TextStyle {
                    font_size: 42.0,
                    color: Color::srgb(0.9, 0.82, 0.62),
                    ..default()
                },
            ));
            parent.spawn(TextBundle::from_section(
                format!(
                    "Gold: {} | Salvage: {} | Skill slots: {}\nHP {} | Damage {} | Armor {} | Healing {}",
                    meta.gold,
                    meta.salvage,
                    meta.unlocked_skill_slots,
                    stats.max_health,
                    stats.damage,
                    stats.armor,
                    stats.healing_power
                ),
                TextStyle {
                    font_size: 18.0,
                    color: Color::WHITE,
                    ..default()
                },
            ));

            spawn_equipped_text(parent, &profile);
            spawn_inventory_buttons(parent, inventory);
            spawn_upgrade_buttons(parent, meta);

            parent
                .spawn((primary_button(), ReturnToBuildButton))
                .with_children(|button| {
                    button.spawn(button_text("Run Again"));
                });
        });
}

fn spawn_equipped_text(parent: &mut ChildBuilder, profile: &ProfileState) {
    let equipped_name = |slot| {
        profile
            .profile
            .hero
            .equipped_item(slot)
            .map(|item| item.name.clone())
            .unwrap_or_else(|| "Empty".to_string())
    };

    parent.spawn(TextBundle::from_section(
        format!(
            "Equipped\nWeapon: {}\nArmor: {}\nTrinket: {}",
            equipped_name(GearSlot::Weapon),
            equipped_name(GearSlot::Armor),
            equipped_name(GearSlot::Trinket)
        ),
        TextStyle {
            font_size: 18.0,
            color: Color::srgb(0.75, 0.73, 0.8),
            ..default()
        },
    ));
}

fn spawn_inventory_buttons(
    parent: &mut ChildBuilder,
    inventory: &[crate::domain::items::ItemInstance],
) {
    parent.spawn(TextBundle::from_section(
        if inventory.is_empty() {
            "Inventory: empty".to_string()
        } else {
            "Inventory".to_string()
        },
        TextStyle {
            font_size: 22.0,
            color: Color::srgb(0.9, 0.82, 0.62),
            ..default()
        },
    ));

    for item in inventory.iter().take(4) {
        parent.spawn(TextBundle::from_section(
            format!("{} {:?} {:?}", item.name, item.slot, item.rarity),
            TextStyle {
                font_size: 16.0,
                color: Color::WHITE,
                ..default()
            },
        ));
        parent
            .spawn((small_button(), EquipItemButton { item_id: item.id }))
            .with_children(|button| {
                button.spawn(button_text("Equip"));
            });
        parent
            .spawn((small_button(), SalvageItemButton { item_id: item.id }))
            .with_children(|button| {
                button.spawn(button_text("Salvage"));
            });
    }
}

fn spawn_upgrade_buttons(
    parent: &mut ChildBuilder,
    meta: &crate::domain::progression::MetaProgression,
) {
    parent.spawn(TextBundle::from_section(
        "Upgrades",
        TextStyle {
            font_size: 22.0,
            color: Color::srgb(0.9, 0.82, 0.62),
            ..default()
        },
    ));

    for upgrade in [
        UpgradeId::MaxHealth,
        UpgradeId::BaseDamage,
        UpgradeId::Armor,
        UpgradeId::HealingPower,
        UpgradeId::GoldGain,
    ] {
        parent
            .spawn((small_button(), BuyUpgradeButton { upgrade }))
            .with_children(|button| {
                button.spawn(button_text(format!(
                    "{upgrade:?} L{} - {} gold",
                    meta.upgrade_level(upgrade),
                    meta.upgrade_cost(upgrade)
                )));
            });
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

fn cleanup_ui(mut commands: Commands, roots: Query<Entity, With<UiRoot>>) {
    for root in &roots {
        commands.entity(root).despawn_recursive();
    }
}

fn primary_button() -> ButtonBundle {
    ButtonBundle {
        style: Style {
            width: Val::Px(240.0),
            height: Val::Px(58.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            margin: UiRect::top(Val::Px(16.0)),
            ..default()
        },
        background_color: Color::srgb(0.35, 0.12, 0.11).into(),
        ..default()
    }
}

fn small_button() -> ButtonBundle {
    ButtonBundle {
        style: Style {
            width: Val::Px(190.0),
            height: Val::Px(38.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            margin: UiRect::all(Val::Px(4.0)),
            ..default()
        },
        background_color: Color::srgb(0.22, 0.11, 0.16).into(),
        ..default()
    }
}

fn button_text(text: impl Into<String>) -> TextBundle {
    TextBundle::from_section(
        text,
        TextStyle {
            font_size: 18.0,
            color: Color::WHITE,
            ..default()
        },
    )
}

fn full_screen_column() -> Style {
    Style {
        width: Val::Percent(100.0),
        height: Val::Percent(100.0),
        flex_direction: FlexDirection::Column,
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        row_gap: Val::Px(16.0),
        padding: UiRect::all(Val::Px(32.0)),
        ..default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::{
        AcceptRunRewards, GameState, IdleDungeonsPlugin, LatestRunSummary, ProfileState, StartRun,
    };
    use crate::domain::items::{GearSlot, ItemInstance};
    use crate::domain::progression::UpgradeId;
    use bevy::state::app::StatesPlugin;

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
}
