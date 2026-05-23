use bevy::prelude::*;

/// Declarative click target: [`super::dispatch::dispatch_ui_clicks`] maps release on this entity to app effects.
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub enum UiClickAction {
    TitleEnterCamp,
    TitleQuit,
    StartRun,
    SkipPlayback,
    AcceptRewards,
    ResetProgress,
    OpenSettings,
    CloseSettings,
}
