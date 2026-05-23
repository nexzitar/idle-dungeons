use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

use crate::app::{
    AcceptRunRewards, AssignHeroSkill, BuySkillUnlock, EquipInventoryItem, GameState,
    LatestRunSummary, OpenGearHub, OpenSkillBook, OpenSkillShop, ProfileSavePath, ProfileState,
    ResetProgress, RunSpeedSetting, SalvageInventoryItem, SkipRunPlayback, StartRun,
};
use crate::presentation::PresentationEditorSession;
use crate::ui::components::{
    BuildScreen, EquipItemButton, GearHubRoot, HeroNameEditButton, HeroNameEditState,
    RunPlaybackScreen, SalvageItemButton, SettingsModalRoot, SkillBookPickButton, SkillBookRoot,
    SkillShopBuyButton, SkillShopRoot, SkillSlotButton, SummaryScreen, UiRoot,
};
use crate::ui::assets::UiPlaceholderImages;
use crate::ui::interaction::registry::UiClickAction;
use crate::ui::scene_tune::TitleSceneLayout;
use crate::ui::{GearHubKeepOpen, PlaybackCombatLogVisible};

#[derive(SystemParam)]
pub(crate) struct UiClickWriters<'w> {
    pub exit: MessageWriter<'w, AppExit>,
    pub start_run: MessageWriter<'w, StartRun>,
    pub skip: MessageWriter<'w, SkipRunPlayback>,
    pub accept: MessageWriter<'w, AcceptRunRewards>,
    pub reset: MessageWriter<'w, ResetProgress>,
    pub open_gear: MessageWriter<'w, OpenGearHub>,
    pub open_shop: MessageWriter<'w, OpenSkillShop>,
    pub open_book: MessageWriter<'w, OpenSkillBook>,
    pub buy_skill: MessageWriter<'w, BuySkillUnlock>,
    pub equip: MessageWriter<'w, EquipInventoryItem>,
    pub salvage: MessageWriter<'w, SalvageInventoryItem>,
    pub assign_skill: MessageWriter<'w, AssignHeroSkill>,
}

#[derive(SystemParam)]
pub(crate) struct UiClickQueries<'w, 's> {
    pub clicked: Query<'w, 's, (Entity, &'static Interaction, &'static UiClickAction)>,
    pub ui_roots: Query<'w, 's, Entity, With<UiRoot>>,
    pub settings_modal: Query<'w, 's, Entity, With<SettingsModalRoot>>,
    pub gear_modal: Query<'w, 's, Entity, With<GearHubRoot>>,
    pub shop_modal: Query<'w, 's, Entity, With<SkillShopRoot>>,
    pub book_modal: Query<'w, 's, Entity, With<SkillBookRoot>>,
    pub skill_slots: Query<'w, 's, &'static SkillSlotButton>,
    pub equip_btns: Query<'w, 's, &'static EquipItemButton>,
    pub salvage_btns: Query<'w, 's, &'static SalvageItemButton>,
    pub shop_buy: Query<'w, 's, &'static SkillShopBuyButton>,
    pub book_pick: Query<'w, 's, &'static SkillBookPickButton>,
    pub hero_edit: Query<'w, 's, &'static HeroNameEditButton>,
    pub build_roots: Query<'w, 's, Entity, With<BuildScreen>>,
    pub summary_roots: Query<'w, 's, Entity, With<SummaryScreen>>,
    pub running_roots: Query<'w, 's, Entity, With<RunPlaybackScreen>>,
}

#[derive(SystemParam)]
pub(crate) struct UiClickState<'w> {
    pub next_state: ResMut<'w, NextState<GameState>>,
    pub speed: ResMut<'w, RunSpeedSetting>,
    pub log_vis: ResMut<'w, PlaybackCombatLogVisible>,
    pub name_edit: ResMut<'w, HeroNameEditState>,
    pub gear_keep: ResMut<'w, GearHubKeepOpen>,
    pub profile: ResMut<'w, ProfileState>,
    pub save_path: Res<'w, ProfileSavePath>,
    pub latest_summary: Option<Res<'w, LatestRunSummary>>,
    pub ph: Res<'w, UiPlaceholderImages>,
    pub game_state: Res<'w, State<GameState>>,
    #[cfg(debug_assertions)]
    pub layout: ResMut<'w, TitleSceneLayout>,
    #[cfg(debug_assertions)]
    pub session: ResMut<'w, PresentationEditorSession>,
    #[cfg(debug_assertions)]
    pub field_edit: ResMut<'w, crate::presentation::editor::PresentationEditorFieldEditState>,
    #[cfg(debug_assertions)]
    pub kb: Res<'w, ButtonInput<KeyCode>>,
}
