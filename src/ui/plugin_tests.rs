use crate::app::{
        AcceptRunRewards, GameState, IdleDungeonsPlugin, LatestRunSummary, OpenGearHub,
        SkipRunPlayback, StartRun,
    };
    use crate::ui::components::{
        AcceptRewardsButton, BuildScreen, GearHubRoot, MainCamera, StartRunButton, SummaryScreen,
        TitleScreen,
    };
    use crate::ui::UiPlugin;
    use bevy::ecs::message::Messages;
    use bevy::input::mouse::MouseButtonInput;
    use bevy::input::ButtonState;
    use bevy::prelude::*;
    use bevy::state::app::StatesPlugin;

    fn enter_build_from_title(app: &mut App) {
        app.world_mut()
            .resource_mut::<NextState<GameState>>()
            .set(GameState::Build);
        app.update();
    }

    #[test]
    fn ui_plugin_starts_on_title_then_camp_has_start_button() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(StatesPlugin);
        app.add_plugins(IdleDungeonsPlugin);
        app.add_plugins(UiPlugin);

        app.update();

        assert_eq!(entity_count::<MainCamera>(app.world_mut()), 1);
        assert_eq!(entity_count::<TitleScreen>(app.world_mut()), 1);
        assert_eq!(entity_count::<StartRunButton>(app.world_mut()), 0);

        enter_build_from_title(&mut app);

        assert_eq!(entity_count::<StartRunButton>(app.world_mut()), 1);
        assert_eq!(entity_count::<BuildScreen>(app.world_mut()), 1);
    }

    fn simulate_primary_click(app: &mut App, button: Entity) {
        let window = app.world_mut().spawn_empty().id();
        app.world_mut().write_message(MouseButtonInput {
            button: MouseButton::Left,
            state: ButtonState::Pressed,
            window,
        });
        app.world_mut()
            .entity_mut(button)
            .insert(Interaction::Pressed);
        app.update();
        app.world_mut().write_message(MouseButtonInput {
            button: MouseButton::Left,
            state: ButtonState::Released,
            window,
        });
        app.world_mut()
            .entity_mut(button)
            .insert(Interaction::Hovered);
        app.update();
    }

    #[test]
    fn pressing_start_button_sends_start_run_event() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(StatesPlugin);
        app.add_plugins(IdleDungeonsPlugin);
        app.add_plugins(UiPlugin);
        app.update();
        enter_build_from_title(&mut app);

        let button = single_entity::<StartRunButton>(app.world_mut());
        simulate_primary_click(&mut app, button);

        let msgs = app.world().resource::<Messages<StartRun>>();
        assert_eq!(msgs.len(), 1);
    }

    #[test]
    fn summary_state_spawns_summary_screen() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(StatesPlugin);
        app.add_plugins(IdleDungeonsPlugin);
        app.add_plugins(UiPlugin);
        app.world_mut().write_message(StartRun { seed: 1 });

        app.update();
        app.world_mut().write_message(SkipRunPlayback);
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
        app.world_mut().write_message(StartRun { seed: 1 });

        app.update();
        app.world_mut().write_message(SkipRunPlayback);
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
        app.world_mut().write_message(StartRun { seed: 1 });
        app.update();
        app.world_mut().write_message(SkipRunPlayback);
        app.update();
        app.update();

        let button = single_entity::<AcceptRewardsButton>(app.world_mut());
        simulate_primary_click(&mut app, button);

        assert_eq!(
            app.world().resource::<Messages<AcceptRunRewards>>().len(),
            1
        );
    }

    #[test]
    fn gear_hub_open_event_spawns_gear_hub_modal() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(StatesPlugin);
        app.add_plugins(IdleDungeonsPlugin);
        app.add_plugins(UiPlugin);
        app.update();

        app.world_mut().write_message(OpenGearHub);
        app.update();

        assert_eq!(entity_count::<GearHubRoot>(app.world_mut()), 1);
    }

    fn entity_count<T: Component>(world: &mut World) -> usize {
        let mut query = world.query_filtered::<Entity, With<T>>();
        query.iter(world).count()
    }

    fn single_entity<T: Component>(world: &mut World) -> Entity {
        let mut query = world.query_filtered::<Entity, With<T>>();
        query
            .single(world)
            .expect("expected exactly one matching entity")
}
