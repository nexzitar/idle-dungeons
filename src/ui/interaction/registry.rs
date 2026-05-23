use bevy::prelude::*;

/// Declarative click target: [`super::dispatch::dispatch_ui_clicks`] maps release on this entity to UI effects.
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
    OpenGearHub,
    CloseGearHub,
    OpenSkillShop,
    CloseSkillShop,
    CloseSkillBook,
    SkillBookPick,
    BuySkillUnlock,
    CycleStashSort,
    OpenSkillBook,
    EquipItem,
    SalvageItem,
    PlaybackSpeedDec,
    PlaybackSpeedInc,
    ToggleCombatLog,
    HeroNameEdit,
    #[cfg(debug_assertions)]
    EditorToggleLayout,
    #[cfg(debug_assertions)]
    EditorHierarchySelect,
    #[cfg(debug_assertions)]
    EditorResetCenter,
    #[cfg(debug_assertions)]
    EditorResetAll,
    #[cfg(debug_assertions)]
    EditorSave,
    #[cfg(debug_assertions)]
    EditorReload,
    #[cfg(debug_assertions)]
    EditorTuneValue,
    #[cfg(debug_assertions)]
    EditorTuneDelta,
}
