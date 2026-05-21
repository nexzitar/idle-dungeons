use bevy::prelude::*;

use crate::domain::party::PartyHeroKind;
use crate::domain::skills::SkillId;
use crate::presentation::element::PresentationElementId;
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
    pub kind: PartyHeroKind,
}

#[derive(Component)]
pub struct BuildScreen;

#[derive(Component)]
pub struct SummaryScreen;

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
pub struct PlaybackAllyBarFill;

#[derive(Component)]
pub struct PlaybackEnemyBarFill;

#[derive(Component)]
pub struct PlaybackAggroArrowText;

/// Thin UI bar drawn from the enemy toward the focused party member (threat target).
#[derive(Component)]
pub struct PlaybackAggroArrowLine;

#[derive(Component)]
pub struct PlaybackLeadPortraitBlock;

#[derive(Component)]
pub struct PlaybackAllyPortraitBlock;

#[derive(Component)]
pub struct PlaybackEnemyPortraitBlock;

#[derive(Component)]
pub struct PlaybackTheaterFloatLayer;

#[derive(Component)]
pub struct FloatingCombatPopup {
    pub ttl: f32,
    pub seq: u32,
}

#[derive(Component)]
pub struct PlaybackCombatLogPanel;

#[derive(Component)]
pub struct ToggleCombatLogButton;

#[derive(Component)]
pub struct PlaybackCombatLogToggleLabel;

/// Open the gear hub from chrome (footer).
#[derive(Component)]
pub struct GearHubOpenButton;

#[derive(Component)]
pub struct GearHubRoot;

#[derive(Component)]
pub struct GearHubBackdrop;

#[derive(Component)]
pub struct GearHubCloseButton;

/// Run summary overlay: loot + accept rewards.
#[derive(Component)]
pub struct SummaryRewardsModalRoot;

#[derive(Component)]
pub struct PlaybackDmgMeterPartnerRow;

#[derive(Component)]
pub struct PlaybackDmgMeterLeadFill;

#[derive(Component)]
pub struct PlaybackDmgMeterPartnerFill;

#[derive(Component)]
pub struct PlaybackDmgMeterEnemyFill;

#[derive(Component)]
pub struct PlaybackDmgMeterLeadValue;

#[derive(Component)]
pub struct PlaybackDmgMeterPartnerValue;

#[derive(Component)]
pub struct PlaybackDmgMeterEnemyValue;

#[derive(Component)]
pub struct PlaybackHeroDebuffLine;

#[derive(Component)]
pub struct PlaybackEnemyDebuffLine;

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
pub struct StashSortCycleButton;

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

/// Root of the skill book overlay.
#[derive(Component)]
pub struct SkillBookRoot;

#[derive(Component)]
pub struct SkillBookBackdrop;

#[derive(Component)]
pub struct SkillBookCloseButton;

#[derive(Component, Clone, Copy)]
pub struct SkillBookPickButton {
    pub slot: usize,
    pub skill: Option<SkillId>,
    pub kind: PartyHeroKind,
}

#[derive(Component)]
pub struct SkillShopRoot;

#[derive(Component)]
pub struct SkillShopBackdrop;

#[derive(Component)]
pub struct SkillShopCloseButton;

#[derive(Component)]
pub struct SkillShopOpenButton;

#[derive(Component, Clone, Copy)]
pub struct SkillShopBuyButton {
    pub skill: SkillId,
}

#[derive(Component)]
pub struct PlaybackLeadCastFill;

#[derive(Component)]
pub struct PlaybackLeadCdFill;

#[derive(Component)]
pub struct PlaybackLeadSkillGcdFill;

#[derive(Component)]
pub struct PlaybackLeadInstantRechargeFill;

#[derive(Component)]
pub struct PlaybackAllyCastFill;

#[derive(Component)]
pub struct PlaybackAllyCdFill;

#[derive(Component)]
pub struct PlaybackAllySkillGcdFill;

#[derive(Component)]
pub struct PlaybackAllyInstantRechargeFill;

#[derive(Component)]
pub struct PlaybackFoeCastFill;

#[derive(Component)]
pub struct PlaybackFoeCdFill;

#[derive(Component)]
pub struct PlaybackFoeAltCastFill;

#[derive(Component)]
pub struct PlaybackFoeAltCdFill;

/// Second foe timing row (pack / flank); hidden in solo fights.
#[derive(Component)]
pub struct PlaybackFoeAltTimingRow;

/// Scroll region for the live delve combat log (auto-scroll to latest).
#[derive(Component)]
pub struct PlaybackLogScrollRegion;

#[derive(Component, Clone, Copy)]
pub struct HeroNameDisplayText {
    pub slot: u8,
}

#[derive(Component, Clone, Copy)]
pub struct HeroNameEditButton {
    pub slot: u8,
}

/// Build-screen hero rename: click Edit, type, Enter saves, Esc cancels.
#[derive(Resource, Default)]
pub struct HeroNameEditState {
    pub active_slot: Option<u8>,
    pub buffer: String,
}

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
}

/// Header control: slower delve playback (steps through [`crate::app::PLAYBACK_SPEED_STEPS`]).
#[derive(Component)]
pub struct PlaybackSpeedDecButton;

/// Header control: faster delve playback.
#[derive(Component)]
pub struct PlaybackSpeedIncButton;

/// Header text showing current playback multiplier (synced from [`crate::app::RunSpeedSetting`]).
#[derive(Component)]
pub struct PlaybackSpeedValueText;

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

// --- Title / campfire hub ---

#[derive(Component)]
pub struct TitleScreen;

#[derive(Component)]
pub struct TitleEnterCampButton;

#[derive(Component)]
pub struct TitleQuitButton;

#[derive(Component)]
pub struct TitleCampFigureSlot(pub u8);

/// Debug tuning: figure column root for party slot `0 = Lead`, `1 = Ally`.
#[derive(Component)]
pub struct TitleCampFigureTuneMarker(pub u8);

/// Emoji row in a figure column (same slot index as [`TitleCampFigureTuneMarker`]).
#[derive(Component)]
pub struct TitleCampFigureEmoji(pub u8);

#[derive(Component)]
pub struct TitleCampMilestoneExtras;

#[derive(Component)]
pub struct TitleCampSceneRoot;

/// Marker for the tunable title campfire presentation root (alias for [`crate::presentation::markers::CampfirePresentationRoot`]).
pub use crate::presentation::markers::CampfirePresentationRoot as TitleCampfireTuneMarker;

/// Lower-third layout root over [`UiPlaceholderImages::campfire_scene`]: tent, figure slots, fire layer.
/// Reparent hero portraits/sprites here when art lands; align `Fireplace.png` with the stone ring in the painting.
#[derive(Component)]
pub struct TitleCampStageRoot;

/// UI hit target for tuning a logical presentation element (title camp fireplace / figure slots).
#[derive(Component)]
pub struct PresentationElementHost(pub PresentationElementId);

/// Re-export — defined in [`crate::presentation::markers`].
pub use crate::presentation::markers::PresentationFireLayerHost;
