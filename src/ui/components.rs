use bevy::prelude::*;

use crate::domain::progression::UpgradeId;

#[derive(Component)]
pub struct UiRoot;

#[derive(Component)]
pub struct MainCamera;

#[derive(Component)]
pub struct StartRunButton;

#[derive(Component)]
pub struct SummaryScreen;

#[derive(Component)]
pub struct UpgradeScreen;

#[derive(Component)]
pub struct AcceptRewardsButton;

#[derive(Component)]
pub struct EquipItemButton {
    pub item_id: u64,
}

#[derive(Component)]
pub struct SalvageItemButton {
    pub item_id: u64,
}

#[derive(Component)]
pub struct BuyUpgradeButton {
    pub upgrade: UpgradeId,
}

#[derive(Component)]
pub struct ReturnToBuildButton;

#[derive(Component)]
pub struct SettingsButton;

/// Label synced by `sync_top_bar` (gold, salvage, etc.).
#[derive(Component, Clone, Copy, PartialEq, Eq)]
pub enum TopBarField {
    Gold,
    Salvage,
    SkillSlots,
    Depth,
    Speed,
}
