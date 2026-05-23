use bevy::input::mouse::MouseButton;
use bevy::prelude::*;

use crate::app::{
    AcceptRunRewards, GameState, ResetProgress, SkipRunPlayback, StartRun,
};
use crate::ui::interaction::click::{ui_click_release_confirms, UiClickPress};
use crate::ui::interaction::registry::UiClickAction;

pub(crate) fn dispatch_ui_clicks(
    mouse: Res<ButtonInput<MouseButton>>,
    press: Res<UiClickPress>,
    buttons: Query<(Entity, &Interaction, &UiClickAction)>,
    mut next_state: ResMut<NextState<GameState>>,
    mut exit: MessageWriter<AppExit>,
    mut start_run: MessageWriter<StartRun>,
    mut skip: MessageWriter<SkipRunPlayback>,
    mut accept: MessageWriter<AcceptRunRewards>,
    mut reset: MessageWriter<ResetProgress>,
) {
    if !mouse.just_released(MouseButton::Left) {
        return;
    }
    let Some(target) = press.0 else {
        return;
    };
    for (entity, interaction, action) in &buttons {
        if entity != target || !ui_click_release_confirms(*interaction) {
            continue;
        }
        match *action {
            UiClickAction::TitleEnterCamp => {
                next_state.set(GameState::Build);
            }
            UiClickAction::TitleQuit => {
                exit.write(AppExit::Success);
            }
            UiClickAction::StartRun => {
                start_run.write(StartRun {
                    seed: crate::domain::run::DEFAULT_RUN_SEED,
                });
            }
            UiClickAction::SkipPlayback => {
                skip.write(SkipRunPlayback);
            }
            UiClickAction::AcceptRewards => {
                accept.write(AcceptRunRewards);
            }
            UiClickAction::ResetProgress => {
                reset.write(ResetProgress);
            }
            UiClickAction::OpenSettings | UiClickAction::CloseSettings => {}
        }
        break;
    }
}
