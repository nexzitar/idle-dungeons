use bevy::prelude::*;

use crate::domain::progression::UpgradeId;
use crate::ui::theme::UiTheme;

#[derive(Component)]
pub struct UiRoot;

#[derive(Component)]
pub struct MainCamera;

#[derive(Component)]
pub struct StartRunButton;

#[derive(Component)]
pub struct SkillSlotButton {
    pub slot: usize,
}

#[derive(Component)]
pub struct BuildScreen;

#[derive(Component)]
pub struct SummaryScreen;

#[derive(Component)]
pub struct UpgradeScreen;

#[derive(Component)]
pub struct RunPlaybackScreen;

#[derive(Component)]
pub struct SkipPlaybackButton;

#[derive(Component)]
pub struct PlaybackDepthText;

#[derive(Component)]
pub struct PlaybackRoomKindText;

#[derive(Component)]
pub struct PlaybackEnemyNameText;

#[derive(Component)]
pub struct PlaybackHeroBarFill;

#[derive(Component)]
pub struct PlaybackEnemyBarFill;

#[derive(Component)]
pub struct PlaybackCaptionText;

#[derive(Component)]
pub struct PlaybackLogText;

#[derive(Component)]
pub struct PlaybackProgressBarFill;

#[derive(Component)]
pub struct PlaybackProgressLabel;

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
pub struct StashSortCycleButton;

#[derive(Component)]
pub struct ReturnToBuildButton;

#[derive(Component)]
pub struct SettingsButton;

#[derive(Component)]
pub struct ResetProgressButton;

/// Root of the settings overlay (spawned under [`UiRoot`]).
#[derive(Component)]
pub struct SettingsModalRoot;

/// Full-screen dim layer; click closes the modal.
#[derive(Component)]
pub struct SettingsModalBackdrop;

#[derive(Component)]
pub struct SettingsModalCloseButton;

#[derive(Component)]
pub struct SettingsModalSpeedButton;

#[derive(Component)]
pub struct SettingsModalSpeedLabel;

/// Scroll region for the live delve combat log (auto-scroll to latest).
#[derive(Component)]
pub struct PlaybackLogScrollRegion;

/// Text shown after a short hover delay (`crate::ui::tooltip`).
#[derive(Component, Clone)]
pub struct UiTooltip(pub String);

impl UiTooltip {
    pub fn txt(s: impl Into<String>) -> Self {
        UiTooltip(s.into())
    }
}

/// Driving colors for [`crate::ui::apply_ui_button_palettes`]. Attach next to [`Button`].
#[derive(Component, Clone, Copy)]
pub struct UiButtonPalette {
    pub idle_bg: Color,
    pub hover_bg: Color,
    pub pressed_bg: Color,
    pub idle_border: Color,
    pub hover_border: Color,
    pub pressed_border: Color,
}

impl UiButtonPalette {
    /// Large red CTAs (Start run, Accept rewards).
    pub fn primary_cta() -> Self {
        let idle = UiTheme::muted_red();
        Self {
            idle_bg: idle,
            hover_bg: idle.mix(&Color::WHITE, 0.18),
            pressed_bg: UiTheme::accent_red(),
            idle_border: UiTheme::ornate_gold(),
            hover_border: UiTheme::muted_gold(),
            pressed_border: idle.mix(&Color::BLACK, 0.25),
        }
    }

    /// Bordered panel actions (header Settings).
    pub fn panel_outlined() -> Self {
        let idle = UiTheme::panel_bg_deep();
        Self {
            idle_bg: idle,
            hover_bg: idle.mix(&Color::WHITE, 0.1),
            pressed_bg: idle.mix(&Color::BLACK, 0.18),
            idle_border: UiTheme::ornate_gold(),
            hover_border: UiTheme::muted_gold(),
            pressed_border: UiTheme::accent_red(),
        }
    }

    /// Secondary panel button (Return to briefing).
    pub fn panel_secondary() -> Self {
        let idle = UiTheme::panel_bg();
        Self {
            idle_bg: idle,
            hover_bg: idle.mix(&Color::WHITE, 0.1),
            pressed_bg: idle.mix(&Color::BLACK, 0.15),
            idle_border: UiTheme::ornate_gold(),
            hover_border: UiTheme::muted_gold(),
            pressed_border: UiTheme::accent_red(),
        }
    }

    pub fn equip() -> Self {
        let idle = UiTheme::muted_red();
        Self {
            idle_bg: idle,
            hover_bg: idle.mix(&Color::WHITE, 0.2),
            pressed_bg: UiTheme::accent_red(),
            idle_border: idle.mix(&Color::BLACK, 0.35),
            hover_border: UiTheme::muted_gold(),
            pressed_border: UiTheme::accent_red(),
        }
    }

    pub fn salvage() -> Self {
        let idle = UiTheme::panel_bg();
        Self {
            idle_bg: idle,
            hover_bg: idle.mix(&Color::WHITE, 0.12),
            pressed_bg: idle.mix(&Color::BLACK, 0.2),
            idle_border: UiTheme::panel_border(),
            hover_border: UiTheme::muted_gold(),
            pressed_border: UiTheme::ornate_gold(),
        }
    }

    pub fn buy_upgrade() -> Self {
        let idle = UiTheme::panel_bg_deep();
        Self {
            idle_bg: idle,
            hover_bg: idle.mix(&Color::WHITE, 0.12),
            pressed_bg: idle.mix(&Color::BLACK, 0.22),
            idle_border: UiTheme::panel_border(),
            hover_border: UiTheme::ornate_gold(),
            pressed_border: UiTheme::muted_gold(),
        }
    }

    /// Compact hero skill slot (briefing / camp).
    pub fn skill_slot_chip() -> Self {
        let idle = UiTheme::panel_bg_deep();
        Self {
            idle_bg: idle,
            hover_bg: idle.mix(&Color::WHITE, 0.12),
            pressed_bg: idle.mix(&Color::BLACK, 0.18),
            idle_border: UiTheme::ornate_gold(),
            hover_border: UiTheme::muted_gold(),
            pressed_border: UiTheme::accent_red(),
        }
    }

    pub fn stash_tab(active: bool) -> Self {
        if active {
            let gold = UiTheme::ornate_gold();
            Self {
                idle_bg: Color::NONE,
                hover_bg: Color::srgba(0.78, 0.64, 0.38, 0.14),
                pressed_bg: Color::srgba(0.78, 0.64, 0.38, 0.22),
                idle_border: gold,
                hover_border: UiTheme::muted_gold(),
                pressed_border: UiTheme::accent_red(),
            }
        } else {
            Self {
                idle_bg: Color::NONE,
                hover_bg: Color::srgba(1.0, 1.0, 1.0, 0.06),
                pressed_bg: Color::srgba(1.0, 1.0, 1.0, 0.11),
                idle_border: Color::NONE,
                hover_border: UiTheme::panel_border(),
                pressed_border: UiTheme::ornate_gold(),
            }
        }
    }
}

/// Label synced by `sync_top_bar` (gold, salvage, etc.).
#[derive(Component, Clone, Copy, PartialEq, Eq)]
pub enum TopBarField {
    Gold,
    Salvage,
    SkillSlots,
    Depth,
    Speed,
}

/// Marks the clip viewport for mouse-wheel scrolling (`apply_ui_scroll`).
#[derive(Component)]
pub struct UiScrollRegion;

/// Inner node whose `Style.top` is driven by [`UiScrollState::offset`].
#[derive(Component)]
pub struct UiScrollContent;

#[derive(Component, Default)]
pub struct UiScrollState {
    pub offset: f32,
}
