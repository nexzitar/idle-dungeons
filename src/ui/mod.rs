pub mod build_panel;
pub mod inventory_panel;
pub mod log_panel;
pub mod run_panel;
pub mod summary_panel;
pub mod upgrade_panel;

use crate::app::{GameState, LatestRunSummary, StartRun};
use crate::domain::hero::HeroProfile;
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
            .add_systems(OnExit(GameState::Summary), cleanup_ui);
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

fn spawn_camera(mut commands: Commands) {
    commands.spawn((Camera2dBundle::default(), MainCamera));
}

fn spawn_build_screen(mut commands: Commands) {
    let hero = HeroProfile::default();
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
                build_text,
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
        .map(|summary| summary_panel_text(&summary.0))
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
        });
}

fn cleanup_ui(mut commands: Commands, roots: Query<Entity, With<UiRoot>>) {
    for root in &roots {
        commands.entity(root).despawn_recursive();
    }
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
    use crate::app::{GameState, IdleDungeonsPlugin, LatestRunSummary, StartRun};
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

    fn entity_count<T: Component>(world: &mut World) -> usize {
        let mut query = world.query_filtered::<Entity, With<T>>();
        query.iter(world).count()
    }

    fn single_entity<T: Component>(world: &mut World) -> Entity {
        let mut query = world.query_filtered::<Entity, With<T>>();
        query.single(world)
    }
}
