use crate::domain::hero::HeroProfile;
use crate::domain::run::{simulate_run, RunConfig, RunSummary};
use bevy::prelude::*;
use bevy::state::app::StatesPlugin;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, States)]
pub enum GameState {
    #[default]
    Build,
    Running,
    Summary,
    Upgrades,
}

#[derive(Debug, Clone, Copy, Event)]
pub struct StartRun {
    pub seed: u64,
}

#[derive(Debug, Resource)]
pub struct LatestRunSummary(pub RunSummary);

pub struct IdleDungeonsPlugin;

impl Plugin for IdleDungeonsPlugin {
    fn build(&self, app: &mut App) {
        if !app.is_plugin_added::<StatesPlugin>() {
            app.add_plugins(StatesPlugin);
        }

        app.init_state::<GameState>()
            .add_event::<StartRun>()
            .add_systems(Update, start_run);
    }
}

fn start_run(
    mut commands: Commands,
    mut events: EventReader<StartRun>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    for event in events.read() {
        let hero = HeroProfile::default();
        let summary = simulate_run(
            &hero,
            RunConfig {
                seed: event.seed,
                max_depth: 25,
            },
        );
        commands.insert_resource(LatestRunSummary(summary));
        next_state.set(GameState::Summary);
    }
}

pub fn run() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(IdleDungeonsPlugin)
        .run();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn app_plugin_registers_game_state() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(IdleDungeonsPlugin);

        assert!(app.world().contains_resource::<State<GameState>>());
    }

    #[test]
    fn start_run_event_creates_run_summary_resource() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(IdleDungeonsPlugin);
        app.world_mut().send_event(StartRun { seed: 1 });

        app.update();

        assert!(app.world().contains_resource::<LatestRunSummary>());
    }
}
