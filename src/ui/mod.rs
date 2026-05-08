pub mod build_panel;
pub mod components;
pub mod inventory_panel;
pub mod log_panel;
pub mod mockup_layout;
pub mod placeholder_graphics;
pub mod run_panel;
pub mod skill_book;
pub mod stash_sort;
pub mod summary_panel;
pub mod theme;
pub mod tooltip;
pub mod upgrade_panel;
pub mod widgets;

use crate::app::{
    AcceptRunRewards, ActiveRunPlayback, AssignHeroSkill, BuyUpgrade, EquipInventoryItem,
    GameState, LatestRunSummary, OpenSkillBook, ProfileSavePath, ProfileState, ResetProgress,
    ReturnToBuild, RunSpeedSetting, SalvageInventoryItem, SkipRunPlayback, StartRun,
};
use crate::domain::items::ItemInstance;
use crate::domain::run::{RunPlaybackFrameKind, RunSummary};
use crate::ui::build_panel::build_panel_text;
use crate::ui::components::{
    AcceptRewardsButton, BuildScreen, BuyUpgradeButton, EquipItemButton, HeroNameDisplayText,
    HeroNameEditButton, HeroNameEditState, MainCamera,
    PlaybackCaptionText, PlaybackDepthText, PlaybackEnemyBarFill, PlaybackEnemyDebuffLine,
    PlaybackEnemyNameText, PlaybackHeroBarFill, PlaybackAllyBarFill, PlaybackHeroDebuffLine,
    PlaybackLogScrollRegion, PlaybackAggroLineText,
    PlaybackLogText, PlaybackProgressBarFill, PlaybackProgressLabel, PlaybackRoomKindText,
    ResetProgressButton, ReturnToBuildButton, RunPlaybackScreen, SalvageItemButton, SettingsButton,
    SettingsModalBackdrop, SettingsModalCloseButton, SettingsModalRoot, SettingsModalSpeedButton,
    SettingsModalSpeedLabel, SkillBookBackdrop, SkillBookCloseButton, SkillBookPickButton,
    SkillBookRoot, SkillSlotButton, SkipPlaybackButton, StartRunButton, StashSortCycleButton,
    SummaryScreen, TopBarField, UiButtonPalette, UiRoot, UiScrollContent, UiScrollRegion,
    UiScrollState, UiTooltip, UpgradeScreen,
};
use crate::ui::mockup_layout::RightPanelTab;
use crate::ui::placeholder_graphics::UiPlaceholderImages;
use crate::ui::theme::{
    body_text, caption_text, format_item_affix_lines, format_item_stat_summary, rarity_color,
    UiTheme,
};
use crate::ui::widgets::spawn_atmosphere;
use bevy::app::MainScheduleOrder;
use bevy::asset::AssetPlugin;
use bevy::ecs::schedule::ScheduleLabel;
use bevy::input::mouse::{MouseButton, MouseWheel};
use bevy::input::InputPlugin;
use bevy::prelude::*;
use bevy::transform::TransformSystem;
use bevy::ui::{RelativeCursorPosition, UiSystem};
#[allow(deprecated)]
use bevy::window::ReceivedCharacter;

/// Button that received [`Interaction::Pressed`] on press; used to confirm click on mouse-up.
#[derive(Resource, Default)]
struct UiClickPress(Option<Entity>);

pub struct UiPlugin;

#[derive(ScheduleLabel, Clone, Debug, Hash, PartialEq, Eq)]
struct RegisterUiPlaceholderImages;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        if !app.is_plugin_added::<AssetPlugin>() {
            app.add_plugins(AssetPlugin::default());
        }
        if !app.world().contains_resource::<Assets<Image>>() {
            app.init_asset::<Image>();
        }
        if !app.is_plugin_added::<InputPlugin>() {
            app.add_plugins(InputPlugin);
        }
        #[allow(deprecated)]
        app.add_event::<ReceivedCharacter>();
        app.init_resource::<RightPanelTab>();
        app.init_resource::<UiClickPress>();
        app.init_resource::<crate::ui::tooltip::TooltipState>();
        app.init_resource::<HeroNameEditState>();
        app.add_systems(
            Update,
            crate::ui::tooltip::hide_tooltip_layer_before_pointer_focus.before(UiSystem::Focus),
        );
        app.add_schedule(Schedule::new(RegisterUiPlaceholderImages));
        app.add_systems(
            RegisterUiPlaceholderImages,
            crate::ui::placeholder_graphics::register_ui_placeholder_images,
        );
        app.world_mut()
            .resource_mut::<MainScheduleOrder>()
            .insert_startup_before(StateTransition, RegisterUiPlaceholderImages);
        app.add_systems(Startup, spawn_camera)
            .add_systems(
                OnEnter(GameState::Build),
                (reset_right_tab_inventory, spawn_build_screen).chain(),
            )
            .add_systems(OnExit(GameState::Build), cleanup_ui)
            .add_systems(OnEnter(GameState::Running), spawn_running_screen)
            .add_systems(OnExit(GameState::Running), cleanup_running_exit)
            .add_systems(
                Update,
                (
                    (
                        capture_ui_click_start,
                        apply_ui_button_palettes,
                        handle_start_button.run_if(in_state(GameState::Build)),
                        handle_skip_playback_button.run_if(in_state(GameState::Running)),
                        send_reset_progress_requests,
                        fulfill_reset_progress,
                        open_settings_modal,
                        close_settings_modal,
                        handle_settings_modal_speed,
                        close_skill_book_modal,
                        handle_skill_book_pick,
                        handle_right_panel_tab_buttons,
                        handle_stash_sort_button,
                        handle_skill_slot_buttons,
                        handle_open_skill_book,
                        handle_hero_name_edit_button,
                        hero_rename_keyboard,
                    )
                        .chain(),
                    (
                        handle_accept_button.run_if(in_state(GameState::Summary)),
                        handle_equip_buttons.run_if(in_state(GameState::Upgrades)),
                        handle_salvage_buttons.run_if(in_state(GameState::Upgrades)),
                        handle_buy_upgrade_buttons.run_if(in_state(GameState::Upgrades)),
                        handle_return_to_build_button.run_if(in_state(GameState::Upgrades)),
                        clear_ui_click_after_release,
                    )
                        .chain()
                        .after(handle_open_skill_book),
                    sync_top_bar,
                    sync_hero_name_labels,
                    sync_run_playback_ui.run_if(in_state(GameState::Running)),
                    sync_run_playback_party_bars
                        .run_if(in_state(GameState::Running))
                        .after(sync_run_playback_ui),
                    sync_run_playback_debuff_slots
                        .run_if(in_state(GameState::Running))
                        .after(sync_run_playback_ui),
                    sync_playback_delve_progress_bar.run_if(in_state(GameState::Running)),
                ),
            )
            .add_systems(
                OnEnter(GameState::Summary),
                (reset_right_tab_loot, spawn_summary_screen).chain(),
            )
            .add_systems(OnExit(GameState::Summary), cleanup_ui)
            .add_systems(
                OnEnter(GameState::Upgrades),
                (reset_right_tab_camp, spawn_upgrade_screen).chain(),
            )
            .add_systems(OnExit(GameState::Upgrades), cleanup_ui)
            .add_systems(
                PostUpdate,
                (
                    apply_ui_scroll.after(TransformSystem::TransformPropagate),
                    pin_playback_combat_log_scroll
                        .run_if(in_state(GameState::Running))
                        .after(apply_ui_scroll),
                    refresh_build_screen_on_profile_change.run_if(in_state(GameState::Build)),
                    refresh_upgrade_screen_on_profile_change.run_if(in_state(GameState::Upgrades)),
                    crate::ui::tooltip::update_tooltip.after(UiSystem::Layout),
                ),
            );
    }
}

fn spawn_camera(mut commands: Commands) {
    commands.spawn((Camera2dBundle::default(), MainCamera));
}

fn capture_ui_click_start(
    mouse: Res<ButtonInput<MouseButton>>,
    mut press: ResMut<UiClickPress>,
    buttons: Query<(Entity, &Interaction), With<Button>>,
) {
    if !mouse.just_pressed(MouseButton::Left) {
        return;
    }
    press.0 = buttons
        .iter()
        .find_map(|(e, i)| (*i == Interaction::Pressed).then_some(e));
}

/// Bevy UI's `ui_focus_system` may leave [`Interaction::Pressed`] on the frame where the
/// mouse button is released if press and release occur in the same update (common with quick taps).
/// After a longer hold, release instead becomes [`Interaction::Hovered`].
#[inline]
fn ui_click_release_confirms(interaction: Interaction) -> bool {
    matches!(interaction, Interaction::Hovered | Interaction::Pressed)
}

fn clear_ui_click_after_release(
    mouse: Res<ButtonInput<MouseButton>>,
    mut press: ResMut<UiClickPress>,
) {
    if mouse.just_released(MouseButton::Left) {
        press.0 = None;
    }
}

fn root_shell() -> NodeBundle {
    NodeBundle {
        style: Style {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            position_type: PositionType::Relative,
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Stretch,
            ..default()
        },
        background_color: Color::NONE.into(),
        ..default()
    }
}

fn content_column_bundle() -> NodeBundle {
    NodeBundle {
        style: Style {
            width: Val::Percent(100.0),
            flex_grow: 1.0,
            min_height: Val::Px(0.0),
            flex_direction: FlexDirection::Column,
            padding: UiRect::axes(Val::Px(18.0), Val::Px(14.0)),
            row_gap: Val::Px(12.0),
            align_items: AlignItems::Stretch,
            ..default()
        },
        ..default()
    }
}

fn reset_right_tab_inventory(mut tab: ResMut<RightPanelTab>) {
    *tab = RightPanelTab::Inventory;
}

fn reset_right_tab_loot(mut tab: ResMut<RightPanelTab>) {
    *tab = RightPanelTab::Loot;
}

fn reset_right_tab_camp(mut tab: ResMut<RightPanelTab>) {
    *tab = RightPanelTab::Inventory;
}

fn cleanup_running_exit(mut commands: Commands, roots: Query<Entity, With<UiRoot>>) {
    for root in &roots {
        commands.entity(root).despawn_recursive();
    }
    commands.remove_resource::<ActiveRunPlayback>();
}

fn spawn_running_screen(
    mut commands: Commands,
    profile: Res<ProfileState>,
    speed: Res<RunSpeedSetting>,
    tab: Res<RightPanelTab>,
    ph: Res<UiPlaceholderImages>,
) {
    spawn_running_screen_root(&mut commands, &profile, speed.0, *tab, &ph);
}

fn spawn_running_screen_root(
    commands: &mut Commands,
    profile: &ProfileState,
    speed_mult: f32,
    tab: RightPanelTab,
    ph: &UiPlaceholderImages,
) {
    let lead = profile.effective_hero();
    let partner = profile.effective_party_partner();
    let party_slots = profile.profile.meta.party_slots_unlocked();
    let meta = &profile.profile.meta;
    let loadout_lines: Vec<String> = build_panel_text(&lead)
        .lines()
        .map(|s| s.to_string())
        .collect();

    commands
        .spawn((root_shell(), UiRoot, RunPlaybackScreen))
        .with_children(|root| {
            spawn_atmosphere(root);
            root.spawn(content_column_bundle()).with_children(|col| {
                crate::ui::mockup_layout::spawn_mockup_header(
                    col,
                    ph,
                    meta.gold,
                    meta.salvage,
                    meta.unlocked_skill_slots,
                    lead.equipped_skills.len(),
                    "—",
                    speed_mult,
                );
                crate::ui::mockup_layout::spawn_three_column_row(col, |row| {
                    crate::ui::mockup_layout::spawn_ornate_column(row, 0.95, |panel| {
                        crate::ui::mockup_layout::spawn_hero_column_mockup(
                            panel,
                            ph,
                            &lead,
                            partner.as_ref(),
                            party_slots,
                            &loadout_lines,
                            false,
                            false,
                        );
                    });
                    crate::ui::mockup_layout::spawn_ornate_column(row, 1.05, |panel| {
                        crate::ui::mockup_layout::spawn_run_playback_middle_column(panel);
                    });
                    crate::ui::mockup_layout::spawn_ornate_column(row, 1.0, |panel| {
                        crate::ui::mockup_layout::spawn_right_management_column(
                            panel,
                            ph,
                            tab,
                            meta,
                            profile,
                            &profile.profile.inventory,
                            None,
                            false,
                        );
                    });
                });
                crate::ui::mockup_layout::spawn_mockup_footer(
                    col,
                    crate::ui::mockup_layout::FooterMode::DelvePlayback,
                );
            });
            crate::ui::tooltip::spawn_tooltip_layer(root);
        });
}

fn spawn_build_screen(
    mut commands: Commands,
    profile: Res<ProfileState>,
    speed: Res<RunSpeedSetting>,
    tab: Res<RightPanelTab>,
    ph: Res<UiPlaceholderImages>,
) {
    spawn_build_screen_root(&mut commands, &profile, speed.0, *tab, &ph);
}

fn spawn_build_screen_root(
    commands: &mut Commands,
    profile: &ProfileState,
    speed_mult: f32,
    tab: RightPanelTab,
    ph: &UiPlaceholderImages,
) {
    let lead = profile.effective_hero();
    let partner = profile.effective_party_partner();
    let party_slots = profile.profile.meta.party_slots_unlocked();
    let meta = &profile.profile.meta;
    let loadout_lines: Vec<String> = build_panel_text(&lead)
        .lines()
        .map(|s| s.to_string())
        .collect();
    let stash = profile.profile.inventory.len();

    commands
        .spawn((root_shell(), UiRoot, BuildScreen))
        .with_children(|root| {
            spawn_atmosphere(root);
            root.spawn(content_column_bundle()).with_children(|col| {
                crate::ui::mockup_layout::spawn_mockup_header(
                    col,
                    ph,
                    meta.gold,
                    meta.salvage,
                    meta.unlocked_skill_slots,
                    lead.equipped_skills.len(),
                    "—",
                    speed_mult,
                );
                crate::ui::mockup_layout::spawn_three_column_row(col, |row| {
                    crate::ui::mockup_layout::spawn_ornate_column(row, 0.95, |panel| {
                        crate::ui::mockup_layout::spawn_hero_column_mockup(
                            panel,
                            ph,
                            &lead,
                            partner.as_ref(),
                            party_slots,
                            &loadout_lines,
                            true,
                            true,
                        );
                    });
                    crate::ui::mockup_layout::spawn_ornate_column(row, 1.05, |panel| {
                        crate::ui::mockup_layout::spawn_dungeon_briefing_column(
                            panel, stash, meta,
                        );
                    });
                    crate::ui::mockup_layout::spawn_ornate_column(row, 1.0, |panel| {
                        crate::ui::mockup_layout::spawn_right_management_column(
                            panel,
                            ph,
                            tab,
                            meta,
                            profile,
                            &profile.profile.inventory,
                            None,
                            false,
                        );
                    });
                });
                crate::ui::mockup_layout::spawn_mockup_footer(
                    col,
                    crate::ui::mockup_layout::FooterMode::Briefing,
                );
            });
            crate::ui::tooltip::spawn_tooltip_layer(root);
        });
}

fn spawn_summary_screen(
    mut commands: Commands,
    profile: Res<ProfileState>,
    latest_summary: Option<Res<LatestRunSummary>>,
    speed: Res<RunSpeedSetting>,
    tab: Res<RightPanelTab>,
    ph: Res<UiPlaceholderImages>,
) {
    let summary = latest_summary
        .as_deref()
        .map(|s| s.summary.clone())
        .unwrap_or_else(crate::ui::summary_panel::empty_run_summary);
    spawn_summary_screen_root(&mut commands, &profile, &summary, speed.0, *tab, &ph);
}

fn spawn_summary_screen_root(
    commands: &mut Commands,
    profile: &ProfileState,
    summary: &RunSummary,
    speed_mult: f32,
    tab: RightPanelTab,
    ph: &UiPlaceholderImages,
) {
    let meta = &profile.profile.meta;
    let lead = profile.effective_hero();
    let partner = profile.effective_party_partner();
    let party_slots = profile.profile.meta.party_slots_unlocked();
    let loadout_lines: Vec<String> = build_panel_text(&lead)
        .lines()
        .map(|s| s.to_string())
        .collect();

    commands
        .spawn((root_shell(), UiRoot, SummaryScreen))
        .with_children(|root| {
            spawn_atmosphere(root);
            root.spawn(content_column_bundle()).with_children(|col| {
                crate::ui::mockup_layout::spawn_mockup_header(
                    col,
                    ph,
                    meta.gold,
                    meta.salvage,
                    meta.unlocked_skill_slots,
                    lead.equipped_skills.len(),
                    &summary.deepest_depth.to_string(),
                    speed_mult,
                );
                crate::ui::mockup_layout::spawn_three_column_row(col, |row| {
                    crate::ui::mockup_layout::spawn_ornate_column(row, 0.95, |panel| {
                        crate::ui::mockup_layout::spawn_hero_column_mockup(
                            panel,
                            ph,
                            &lead,
                            partner.as_ref(),
                            party_slots,
                            &loadout_lines,
                            false,
                            false,
                        );
                    });
                    crate::ui::mockup_layout::spawn_ornate_column(row, 1.05, |panel| {
                        crate::ui::mockup_layout::spawn_dungeon_summary_column(panel, summary);
                    });
                    crate::ui::mockup_layout::spawn_ornate_column(row, 1.0, |panel| {
                        crate::ui::mockup_layout::spawn_right_management_column(
                            panel,
                            ph,
                            tab,
                            meta,
                            profile,
                            &profile.profile.inventory,
                            Some(summary.loot.as_slice()),
                            false,
                        );
                    });
                });
                crate::ui::mockup_layout::spawn_mockup_footer(
                    col,
                    crate::ui::mockup_layout::FooterMode::Summary,
                );
            });
            crate::ui::tooltip::spawn_tooltip_layer(root);
        });
}

fn spawn_upgrade_screen(
    mut commands: Commands,
    profile: Res<ProfileState>,
    speed: Res<RunSpeedSetting>,
    tab: Res<RightPanelTab>,
    ph: Res<UiPlaceholderImages>,
) {
    spawn_upgrade_screen_root(&mut commands, &profile, speed.0, *tab, &ph);
}

fn spawn_upgrade_screen_root(
    commands: &mut Commands,
    profile: &ProfileState,
    speed_mult: f32,
    tab: RightPanelTab,
    ph: &UiPlaceholderImages,
) {
    let lead = profile.effective_hero();
    let partner = profile.effective_party_partner();
    let party_slots = profile.profile.meta.party_slots_unlocked();
    let meta = &profile.profile.meta;
    let loadout_lines: Vec<String> = build_panel_text(&lead)
        .lines()
        .map(|s| s.to_string())
        .collect();
    let inventory = &profile.profile.inventory;

    commands
        .spawn((root_shell(), UiRoot, UpgradeScreen))
        .with_children(|root| {
            spawn_atmosphere(root);
            root.spawn(content_column_bundle()).with_children(|col| {
                crate::ui::mockup_layout::spawn_mockup_header(
                    col,
                    ph,
                    meta.gold,
                    meta.salvage,
                    meta.unlocked_skill_slots,
                    lead.equipped_skills.len(),
                    "—",
                    speed_mult,
                );
                crate::ui::mockup_layout::spawn_three_column_row(col, |row| {
                    crate::ui::mockup_layout::spawn_ornate_column(row, 0.95, |panel| {
                        crate::ui::mockup_layout::spawn_hero_column_mockup(
                            panel,
                            ph,
                            &lead,
                            partner.as_ref(),
                            party_slots,
                            &loadout_lines,
                            true,
                            true,
                        );
                    });
                    crate::ui::mockup_layout::spawn_ornate_column(row, 1.05, |panel| {
                        crate::ui::mockup_layout::spawn_dungeon_camp_column(panel);
                    });
                    crate::ui::mockup_layout::spawn_ornate_column(row, 1.0, |panel| {
                        crate::ui::mockup_layout::spawn_right_management_column(
                            panel, ph, tab, meta, profile, inventory, None, true,
                        );
                    });
                });
                crate::ui::mockup_layout::spawn_mockup_footer(
                    col,
                    crate::ui::mockup_layout::FooterMode::Camp,
                );
            });
            crate::ui::tooltip::spawn_tooltip_layer(root);
        });
}

fn handle_right_panel_tab_buttons(
    mouse: Res<ButtonInput<MouseButton>>,
    press: Res<UiClickPress>,
    tab_buttons: Query<(
        Entity,
        &Interaction,
        &crate::ui::mockup_layout::RightTabButton,
    )>,
    mut tab: ResMut<RightPanelTab>,
    mut commands: Commands,
    profile: Res<ProfileState>,
    speed: Res<RunSpeedSetting>,
    latest_summary: Option<Res<LatestRunSummary>>,
    state: Res<State<GameState>>,
    build_roots: Query<Entity, With<BuildScreen>>,
    upgrade_roots: Query<Entity, With<UpgradeScreen>>,
    summary_roots: Query<Entity, With<SummaryScreen>>,
    running_roots: Query<Entity, With<RunPlaybackScreen>>,
    ph: Res<UiPlaceholderImages>,
    mut name_edit: ResMut<HeroNameEditState>,
) {
    if !mouse.just_released(MouseButton::Left) {
        return;
    }
    let Some(target) = press.0 else {
        return;
    };
    for (entity, interaction, btn) in &tab_buttons {
        if entity != target || !ui_click_release_confirms(*interaction) {
            continue;
        }
        if *tab == btn.0 {
            return;
        }
        *tab = btn.0;

        match state.get() {
            GameState::Build => {
                for e in &build_roots {
                    commands.entity(e).despawn_recursive();
                }
                *name_edit = HeroNameEditState::default();
                spawn_build_screen_root(&mut commands, &profile, speed.0, *tab, &ph);
            }
            GameState::Upgrades => {
                for e in &upgrade_roots {
                    commands.entity(e).despawn_recursive();
                }
                *name_edit = HeroNameEditState::default();
                spawn_upgrade_screen_root(&mut commands, &profile, speed.0, *tab, &ph);
            }
            GameState::Summary => {
                let summary = latest_summary
                    .as_deref()
                    .map(|s| s.summary.clone())
                    .unwrap_or_else(crate::ui::summary_panel::empty_run_summary);
                for e in &summary_roots {
                    commands.entity(e).despawn_recursive();
                }
                spawn_summary_screen_root(&mut commands, &profile, &summary, speed.0, *tab, &ph);
            }
            GameState::Running => {
                for e in &running_roots {
                    commands.entity(e).despawn_recursive();
                }
                spawn_running_screen_root(&mut commands, &profile, speed.0, *tab, &ph);
            }
        }
        break;
    }
}

fn handle_stash_sort_button(
    mouse: Res<ButtonInput<MouseButton>>,
    press: Res<UiClickPress>,
    sort_buttons: Query<(Entity, &Interaction), With<StashSortCycleButton>>,
    mut profile: ResMut<ProfileState>,
    save_path: Res<ProfileSavePath>,
    mut commands: Commands,
    speed: Res<RunSpeedSetting>,
    tab: Res<RightPanelTab>,
    latest_summary: Option<Res<LatestRunSummary>>,
    state: Res<State<GameState>>,
    build_roots: Query<Entity, With<BuildScreen>>,
    upgrade_roots: Query<Entity, With<UpgradeScreen>>,
    summary_roots: Query<Entity, With<SummaryScreen>>,
    running_roots: Query<Entity, With<RunPlaybackScreen>>,
    ph: Res<UiPlaceholderImages>,
    mut name_edit: ResMut<HeroNameEditState>,
) {
    if !mouse.just_released(MouseButton::Left) {
        return;
    }
    let Some(target) = press.0 else {
        return;
    };
    for (entity, interaction) in &sort_buttons {
        if entity != target || !ui_click_release_confirms(*interaction) {
            continue;
        }
        profile.profile.stash_sort = profile.profile.stash_sort.toggle();
        if let Err(e) = crate::save::save_profile(&save_path.0, &profile.profile) {
            warn!("failed to save stash sort preference: {e}");
        }

        match state.get() {
            GameState::Build => {
                for e in &build_roots {
                    commands.entity(e).despawn_recursive();
                }
                *name_edit = HeroNameEditState::default();
                spawn_build_screen_root(&mut commands, &profile, speed.0, *tab, &ph);
            }
            GameState::Upgrades => {
                for e in &upgrade_roots {
                    commands.entity(e).despawn_recursive();
                }
                *name_edit = HeroNameEditState::default();
                spawn_upgrade_screen_root(&mut commands, &profile, speed.0, *tab, &ph);
            }
            GameState::Summary => {
                let summary = latest_summary
                    .as_deref()
                    .map(|s| s.summary.clone())
                    .unwrap_or_else(crate::ui::summary_panel::empty_run_summary);
                for e in &summary_roots {
                    commands.entity(e).despawn_recursive();
                }
                spawn_summary_screen_root(&mut commands, &profile, &summary, speed.0, *tab, &ph);
            }
            GameState::Running => {
                for e in &running_roots {
                    commands.entity(e).despawn_recursive();
                }
                spawn_running_screen_root(&mut commands, &profile, speed.0, *tab, &ph);
            }
        }
        break;
    }
}

fn refresh_upgrade_screen_on_profile_change(
    mut commands: Commands,
    profile: Res<ProfileState>,
    speed: Res<RunSpeedSetting>,
    tab: Res<RightPanelTab>,
    upgrade_roots: Query<Entity, With<UpgradeScreen>>,
    ph: Res<UiPlaceholderImages>,
    mut name_edit: ResMut<HeroNameEditState>,
) {
    if !profile.is_changed() || upgrade_roots.is_empty() {
        return;
    }

    for root in &upgrade_roots {
        commands.entity(root).despawn_recursive();
    }
    *name_edit = HeroNameEditState::default();
    spawn_upgrade_screen_root(&mut commands, &profile, speed.0, *tab, &ph);
}

fn refresh_build_screen_on_profile_change(
    mut commands: Commands,
    profile: Res<ProfileState>,
    speed: Res<RunSpeedSetting>,
    tab: Res<RightPanelTab>,
    build_roots: Query<Entity, With<BuildScreen>>,
    ph: Res<UiPlaceholderImages>,
    mut name_edit: ResMut<HeroNameEditState>,
) {
    if !profile.is_changed() || build_roots.is_empty() {
        return;
    }

    for root in &build_roots {
        commands.entity(root).despawn_recursive();
    }
    *name_edit = HeroNameEditState::default();
    spawn_build_screen_root(&mut commands, &profile, speed.0, *tab, &ph);
}

pub(crate) fn spawn_item_card(
    parent: &mut ChildBuilder,
    item: &ItemInstance,
    ph: &UiPlaceholderImages,
) {
    parent
        .spawn(NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                padding: UiRect::all(Val::Px(UiTheme::PANEL_INSET)),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::FlexStart,
                row_gap: Val::Px(8.0),
                border: UiRect::all(Val::Px(1.0)),
                ..default()
            },
            background_color: UiTheme::panel_bg_deep().into(),
            border_color: BorderColor(rarity_color(item.rarity).mix(&Color::BLACK, 0.45)),
            ..default()
        })
        .with_children(|card| {
            card.spawn(NodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    flex_direction: FlexDirection::Row,
                    column_gap: Val::Px(10.0),
                    align_items: AlignItems::FlexStart,
                    ..default()
                },
                ..default()
            })
            .with_children(|head| {
                head.spawn(ImageBundle {
                    style: Style {
                        width: Val::Px(40.0),
                        height: Val::Px(40.0),
                        flex_shrink: 0.0,
                        ..default()
                    },
                    image: UiImage::new(ph.item_generic.clone())
                        .with_color(rarity_color(item.rarity).mix(&Color::WHITE, 0.35)),
                    background_color: Color::NONE.into(),
                    ..default()
                });
                head.spawn(NodeBundle {
                    style: Style {
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::FlexStart,
                        row_gap: Val::Px(4.0),
                        flex_grow: 1.0,
                        min_width: Val::Px(0.0),
                        ..default()
                    },
                    ..default()
                })
                .with_children(|txt| {
                    txt.spawn(TextBundle::from_section(
                        &item.name,
                        TextStyle {
                            font_size: UiTheme::FONT_SECTION,
                            color: rarity_color(item.rarity),
                            ..default()
                        },
                    ));
                    txt.spawn(caption_text(format!("{:?} · {:?}", item.rarity, item.slot)));
                    txt.spawn(body_text(format_item_stat_summary(item)));
                    let aff = format_item_affix_lines(item);
                    if !aff.is_empty() {
                        txt.spawn(caption_text(aff));
                    }
                });
            });
            card.spawn(NodeBundle {
                style: Style {
                    flex_direction: FlexDirection::Row,
                    column_gap: Val::Px(10.0),
                    align_items: AlignItems::Center,
                    ..default()
                },
                ..default()
            })
            .with_children(|row| {
                let equip_pal = UiButtonPalette::equip();
                row.spawn((
                    ButtonBundle {
                        style: Style {
                            width: Val::Px(108.0),
                            height: Val::Px(36.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            border: UiRect::all(Val::Px(1.0)),
                            ..default()
                        },
                        background_color: equip_pal.idle_bg.into(),
                        border_color: BorderColor(equip_pal.idle_border),
                        ..default()
                    },
                    EquipItemButton { item_id: item.id },
                    equip_pal,
                    UiTooltip::txt(
                        "Equip this item on your hero. It replaces whatever is currently in this gear slot.",
                    ),
                ))
                .with_children(|b| {
                    b.spawn(TextBundle::from_section(
                        "Equip",
                        TextStyle {
                            font_size: UiTheme::FONT_BODY,
                            color: Color::WHITE,
                            ..default()
                        },
                    ));
                });
                let salvage_pal = UiButtonPalette::salvage();
                row.spawn((
                    ButtonBundle {
                        style: Style {
                            width: Val::Px(108.0),
                            height: Val::Px(36.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            border: UiRect::all(Val::Px(1.0)),
                            ..default()
                        },
                        background_color: salvage_pal.idle_bg.into(),
                        border_color: BorderColor(salvage_pal.idle_border),
                        ..default()
                    },
                    SalvageItemButton { item_id: item.id },
                    salvage_pal,
                    UiTooltip::txt(
                        "Salvage this item for currency. The item is removed from your stash permanently.",
                    ),
                ))
                .with_children(|b| {
                    b.spawn(TextBundle::from_section(
                        "Salvage",
                        TextStyle {
                            font_size: UiTheme::FONT_BODY,
                            color: UiTheme::body(),
                            ..default()
                        },
                    ));
                });
            });
        });
}

fn sync_top_bar(
    state: Res<State<GameState>>,
    profile: Res<ProfileState>,
    latest_summary: Option<Res<LatestRunSummary>>,
    playback: Option<Res<ActiveRunPlayback>>,
    speed: Res<RunSpeedSetting>,
    mut q: Query<(&TopBarField, &mut Text)>,
) {
    let meta = &profile.profile.meta;
    let hero = profile.effective_hero();
    let depth = match state.get() {
        GameState::Summary => latest_summary
            .as_deref()
            .map(|s| s.summary.deepest_depth.to_string())
            .unwrap_or_else(|| "—".to_string()),
        GameState::Running => playback
            .as_ref()
            .and_then(|p| {
                if p.frames.is_empty() {
                    return None;
                }
                let idx = p.display_index.min(p.frames.len() - 1);
                Some(p.frames[idx].depth.to_string())
            })
            .unwrap_or_else(|| "—".to_string()),
        _ => "—".to_string(),
    };
    let skill_cap = hero.equipped_skills.len().max(1);
    for (field, mut text) in &mut q {
        let value = match field {
            TopBarField::Gold => format!("{}", meta.gold),
            TopBarField::Salvage => format!("{}", meta.salvage),
            TopBarField::SkillSlots => {
                format!("{}/{}", meta.unlocked_skill_slots, skill_cap)
            }
            TopBarField::Depth => depth.clone(),
            TopBarField::Speed => {
                if (speed.0 - 1.0).abs() < f32::EPSILON {
                    "1x".to_string()
                } else if (speed.0 - 2.0).abs() < f32::EPSILON {
                    "2x".to_string()
                } else {
                    format!("{:.1}x", speed.0)
                }
            }
        };
        if text.sections[0].value != value {
            text.sections[0].value = value;
        }
    }
}

fn apply_ui_button_palettes(
    mut q: Query<
        (
            &Interaction,
            &UiButtonPalette,
            &mut BackgroundColor,
            &mut BorderColor,
        ),
        With<Button>,
    >,
) {
    for (interaction, pal, mut bg, mut border) in &mut q {
        let (cb, bo) = match *interaction {
            Interaction::None => (pal.idle_bg, pal.idle_border),
            Interaction::Hovered => (pal.hover_bg, pal.hover_border),
            Interaction::Pressed => (pal.pressed_bg, pal.pressed_border),
        };
        *bg = cb.into();
        *border = BorderColor(bo);
    }
}

fn send_reset_progress_requests(
    mouse: Res<ButtonInput<MouseButton>>,
    press: Res<UiClickPress>,
    interactions: Query<(Entity, &Interaction), With<ResetProgressButton>>,
    mut events: EventWriter<ResetProgress>,
) {
    if !mouse.just_released(MouseButton::Left) {
        return;
    }
    let Some(target) = press.0 else {
        return;
    };
    for (entity, interaction) in &interactions {
        if entity == target && ui_click_release_confirms(*interaction) {
            events.send(ResetProgress);
            break;
        }
    }
}

fn fulfill_reset_progress(
    mut events: EventReader<ResetProgress>,
    mut profile: ResMut<ProfileState>,
    save_path: Res<ProfileSavePath>,
    mut commands: Commands,
    mut next_state: ResMut<NextState<GameState>>,
    mut tab: ResMut<RightPanelTab>,
    state: Res<State<GameState>>,
    speed: Res<RunSpeedSetting>,
    build_roots: Query<Entity, With<BuildScreen>>,
    ph: Res<UiPlaceholderImages>,
    mut name_edit: ResMut<HeroNameEditState>,
) {
    for _ in events.read() {
        profile.profile = crate::save::SaveProfile::default();
        if let Err(e) = crate::save::save_profile(&save_path.0, &profile.profile) {
            warn!("failed to save profile after reset: {e}");
        }
        commands.remove_resource::<LatestRunSummary>();
        commands.remove_resource::<ActiveRunPlayback>();
        *name_edit = HeroNameEditState::default();

        if *state.get() == GameState::Build {
            *tab = RightPanelTab::Inventory;
            for e in &build_roots {
                commands.entity(e).despawn_recursive();
            }
            spawn_build_screen_root(&mut commands, &profile, speed.0, *tab, &ph);
        } else {
            next_state.set(GameState::Build);
        }
    }
}

fn open_settings_modal(
    mouse: Res<ButtonInput<MouseButton>>,
    press: Res<UiClickPress>,
    interactions: Query<(Entity, &Interaction), With<SettingsButton>>,
    roots: Query<Entity, With<UiRoot>>,
    existing: Query<(), With<SettingsModalRoot>>,
    speed: Res<RunSpeedSetting>,
    mut commands: Commands,
) {
    if !mouse.just_released(MouseButton::Left) {
        return;
    }
    let Some(target) = press.0 else {
        return;
    };
    for (entity, interaction) in &interactions {
        if entity != target || !ui_click_release_confirms(*interaction) {
            continue;
        }
        if !existing.is_empty() {
            return;
        }
        let Ok(root) = roots.get_single() else {
            return;
        };
        commands.entity(root).with_children(|parent| {
            crate::ui::mockup_layout::spawn_settings_modal(parent, speed.0);
        });
        break;
    }
}

fn close_settings_modal(
    mouse: Res<ButtonInput<MouseButton>>,
    press: Res<UiClickPress>,
    backdrop: Query<(Entity, &Interaction), With<SettingsModalBackdrop>>,
    close_btn: Query<(Entity, &Interaction), With<SettingsModalCloseButton>>,
    modal: Query<Entity, With<SettingsModalRoot>>,
    mut commands: Commands,
) {
    if !mouse.just_released(MouseButton::Left) {
        return;
    }
    let Some(target) = press.0 else {
        return;
    };
    let should_close = backdrop
        .iter()
        .any(|(e, i)| e == target && ui_click_release_confirms(*i))
        || close_btn
            .iter()
            .any(|(e, i)| e == target && ui_click_release_confirms(*i));
    if should_close {
        for entity in &modal {
            commands.entity(entity).despawn_recursive();
        }
    }
}

fn handle_open_skill_book(
    mut events: EventReader<OpenSkillBook>,
    roots: Query<Entity, With<UiRoot>>,
    existing: Query<(), With<SkillBookRoot>>,
    mut commands: Commands,
    ph: Res<UiPlaceholderImages>,
) {
    for ev in events.read() {
        if !existing.is_empty() {
            continue;
        }
        let Ok(root) = roots.get_single() else {
            continue;
        };
        commands.entity(root).with_children(|parent| {
            crate::ui::skill_book::spawn_skill_book_modal(parent, ev.slot, ev.kind, &ph);
        });
    }
}

fn close_skill_book_modal(
    mouse: Res<ButtonInput<MouseButton>>,
    press: Res<UiClickPress>,
    backdrop: Query<(Entity, &Interaction), With<SkillBookBackdrop>>,
    close_btn: Query<(Entity, &Interaction), With<SkillBookCloseButton>>,
    modal: Query<Entity, With<SkillBookRoot>>,
    mut commands: Commands,
) {
    if !mouse.just_released(MouseButton::Left) {
        return;
    }
    let Some(target) = press.0 else {
        return;
    };
    let should_close = backdrop
        .iter()
        .any(|(e, i)| e == target && ui_click_release_confirms(*i))
        || close_btn
            .iter()
            .any(|(e, i)| e == target && ui_click_release_confirms(*i));
    if should_close {
        for entity in &modal {
            commands.entity(entity).despawn_recursive();
        }
    }
}

fn handle_skill_book_pick(
    mouse: Res<ButtonInput<MouseButton>>,
    press: Res<UiClickPress>,
    buttons: Query<(Entity, &Interaction, &SkillBookPickButton)>,
    mut writer: EventWriter<AssignHeroSkill>,
    modal: Query<Entity, With<SkillBookRoot>>,
    mut commands: Commands,
) {
    if !mouse.just_released(MouseButton::Left) {
        return;
    }
    let Some(target) = press.0 else {
        return;
    };
    for (entity, interaction, pick) in &buttons {
        if entity == target && ui_click_release_confirms(*interaction) {
            writer.send(AssignHeroSkill {
                slot: pick.slot,
                skill: pick.skill,
                kind: pick.kind,
            });
            for e in &modal {
                commands.entity(e).despawn_recursive();
            }
            break;
        }
    }
}

fn handle_settings_modal_speed(
    mouse: Res<ButtonInput<MouseButton>>,
    press: Res<UiClickPress>,
    interactions: Query<(Entity, &Interaction), With<SettingsModalSpeedButton>>,
    mut speed: ResMut<RunSpeedSetting>,
    mut labels: Query<&mut Text, With<SettingsModalSpeedLabel>>,
) {
    if !mouse.just_released(MouseButton::Left) {
        return;
    }
    let Some(target) = press.0 else {
        return;
    };
    for (entity, interaction) in &interactions {
        if entity != target || !ui_click_release_confirms(*interaction) {
            continue;
        }
        speed.0 = if (speed.0 - 1.0).abs() < f32::EPSILON {
            2.0
        } else {
            1.0
        };
        let speed_label = if (speed.0 - 1.0).abs() < f32::EPSILON {
            "1x".to_string()
        } else if (speed.0 - 2.0).abs() < f32::EPSILON {
            "2x".to_string()
        } else {
            format!("{:.1}x", speed.0)
        };
        for mut text in &mut labels {
            text.sections[0].value = format!("Speed: {speed_label} (click to toggle)");
        }
        break;
    }
}

fn handle_start_button(
    mouse: Res<ButtonInput<MouseButton>>,
    press: Res<UiClickPress>,
    buttons: Query<(Entity, &Interaction), With<StartRunButton>>,
    mut start_run_events: EventWriter<StartRun>,
) {
    if !mouse.just_released(MouseButton::Left) {
        return;
    }
    let Some(target) = press.0 else {
        return;
    };
    for (entity, interaction) in &buttons {
        if entity == target && ui_click_release_confirms(*interaction) {
            start_run_events.send(StartRun {
                seed: crate::domain::run::DEFAULT_RUN_SEED,
            });
            break;
        }
    }
}

fn handle_skip_playback_button(
    mouse: Res<ButtonInput<MouseButton>>,
    press: Res<UiClickPress>,
    buttons: Query<(Entity, &Interaction), With<SkipPlaybackButton>>,
    mut events: EventWriter<SkipRunPlayback>,
) {
    if !mouse.just_released(MouseButton::Left) {
        return;
    }
    let Some(target) = press.0 else {
        return;
    };
    for (entity, interaction) in &buttons {
        if entity == target && ui_click_release_confirms(*interaction) {
            events.send(SkipRunPlayback);
            break;
        }
    }
}

fn sync_run_playback_ui(
    playback: Res<ActiveRunPlayback>,
    mut params: ParamSet<(
        Query<&mut Text, With<PlaybackDepthText>>,
        Query<&mut Text, With<PlaybackRoomKindText>>,
        Query<&mut Text, With<PlaybackEnemyNameText>>,
        Query<&mut Text, With<PlaybackCaptionText>>,
        Query<&mut Text, With<PlaybackLogText>>,
        Query<&mut Style, With<PlaybackHeroBarFill>>,
        Query<&mut Style, With<PlaybackEnemyBarFill>>,
    )>,
) {
    if playback.frames.is_empty() {
        return;
    }
    let idx = playback.display_index.min(playback.frames.len() - 1);
    let frame = &playback.frames[idx];

    let depth_s = format!("Depth: {}", frame.depth);
    let kind_s = format!(
        "Type: {} · Risk: {}",
        crate::ui::mockup_layout::room_kind_label(frame.room_kind),
        frame.risk_hint
    );

    let hero_max_snap = frame.hero_snapshot_max_hp.max(1) as f32;
    let hero_f = (frame.hero_snapshot_hp as f32 / hero_max_snap).clamp(0.0, 1.0);
    let (enemy_f, enemy_name, caption) = match &frame.kind {
        RunPlaybackFrameKind::Narration { text } => (0.0, "—".to_string(), text.clone()),
        RunPlaybackFrameKind::Combat(c) => (
            c.enemy_hp as f32 / c.enemy_max_hp.max(1) as f32,
            c.enemy_name.clone(),
            c.caption.clone(),
        ),
    };

    let log_body = playback.log_lines.join("\n");
    let log_rich = crate::ui::theme::playback_log_rich_text(&playback.log_lines);

    for mut text in params.p0().iter_mut() {
        if text.sections[0].value != depth_s {
            text.sections[0].value = depth_s.clone();
        }
    }
    for mut text in params.p1().iter_mut() {
        if text.sections[0].value != kind_s {
            text.sections[0].value = kind_s.clone();
        }
    }
    for mut text in params.p2().iter_mut() {
        if text.sections[0].value != enemy_name {
            text.sections[0].value = enemy_name.clone();
        }
    }
    for mut text in params.p3().iter_mut() {
        if text.sections[0].value != caption {
            text.sections[0].value = caption.clone();
        }
    }
    for mut text in params.p4().iter_mut() {
        if crate::ui::theme::text_flatten(&text) != log_body {
            text.sections = log_rich.sections.clone();
        }
    }

    let hero_w = Val::Percent((hero_f * 100.0).clamp(0.0, 100.0));
    let enemy_w = Val::Percent((enemy_f * 100.0).clamp(0.0, 100.0));
    for mut style in params.p5().iter_mut() {
        style.width = hero_w;
    }
    for mut style in params.p6().iter_mut() {
        style.width = enemy_w;
    }
}

fn playback_aggro_status_line(c: &crate::domain::combat::CombatPlaybackFrame) -> String {
    let party = c.partner_max_hp.is_some();
    if !party {
        return "Party: solo".to_string();
    }
    let focus = match c.foe_last_target {
        None => "—",
        Some(0) => "You",
        Some(1) => "Ally",
        _ => "—",
    };
    let th = match (c.threat_slot0, c.threat_slot1) {
        (Some(a), Some(b)) => format!("You {a} · Ally {b}"),
        _ => "—".to_string(),
    };
    format!("Foe focus: {focus} · Threat {th}")
}

fn sync_run_playback_party_bars(
    playback: Res<ActiveRunPlayback>,
    mut ally_bar: Query<&mut Style, With<PlaybackAllyBarFill>>,
    mut aggro: Query<&mut Text, With<PlaybackAggroLineText>>,
) {
    if playback.frames.is_empty() {
        return;
    }
    let idx = playback.display_index.min(playback.frames.len() - 1);
    let frame = &playback.frames[idx];

    let ally_w = match (frame.partner_snapshot_hp, frame.partner_snapshot_max_hp) {
        (Some(h), Some(m)) if m > 0 => Val::Percent(
            ((h as f32 / m as f32).clamp(0.0, 1.0) * 100.0).clamp(0.0, 100.0),
        ),
        _ => Val::Percent(0.0),
    };
    for mut style in &mut ally_bar {
        style.width = ally_w;
    }

    let aggro_s = match &frame.kind {
        RunPlaybackFrameKind::Narration { .. } => {
            if frame.partner_snapshot_max_hp.is_some() {
                "Foe focus: — · Threat — (travel)".to_string()
            } else {
                "Party: solo".to_string()
            }
        }
        RunPlaybackFrameKind::Combat(c) => playback_aggro_status_line(c),
    };

    for mut text in &mut aggro {
        if text.sections[0].value != aggro_s {
            text.sections[0].value = aggro_s.clone();
        }
    }
}

fn sync_run_playback_debuff_slots(
    playback: Res<ActiveRunPlayback>,
    mut hero: Query<
        &mut Text,
        (
            With<PlaybackHeroDebuffLine>,
            Without<PlaybackEnemyDebuffLine>,
        ),
    >,
    mut foe: Query<
        &mut Text,
        (
            With<PlaybackEnemyDebuffLine>,
            Without<PlaybackHeroDebuffLine>,
        ),
    >,
) {
    if playback.frames.is_empty() {
        return;
    }
    let idx = playback.display_index.min(playback.frames.len() - 1);
    let frame = &playback.frames[idx];
    const EMPTY_DEBUFF: &str = "—  ·  —  ·  —  ·  —";
    let (hero_line, foe_line) = match &frame.kind {
        RunPlaybackFrameKind::Narration { .. } => {
            (EMPTY_DEBUFF.to_string(), EMPTY_DEBUFF.to_string())
        }
        RunPlaybackFrameKind::Combat(c) => (
            c.hero_debuff_slots.join("  ·  "),
            c.enemy_debuff_slots.join("  ·  "),
        ),
    };
    let hero_text = crate::ui::theme::playback_debuff_status_text(&hero_line);
    let foe_text = crate::ui::theme::playback_debuff_status_text(&foe_line);
    for mut text in &mut hero {
        if crate::ui::theme::text_flatten(&text) != hero_line {
            text.sections = hero_text.sections.clone();
        }
    }
    for mut text in &mut foe {
        if crate::ui::theme::text_flatten(&text) != foe_line {
            text.sections = foe_text.sections.clone();
        }
    }
}

fn sync_playback_delve_progress_bar(
    playback: Res<ActiveRunPlayback>,
    mut fill: Query<&mut Style, With<PlaybackProgressBarFill>>,
    mut label: Query<&mut Text, With<PlaybackProgressLabel>>,
) {
    if playback.frames.is_empty() {
        return;
    }
    let idx = playback.display_index.min(playback.frames.len() - 1);
    let frame = &playback.frames[idx];
    let cap = frame.delve_floors_cap.max(1);
    let cleared = frame.delve_floors_cleared;
    let frac = (cleared as f32 / cap as f32).clamp(0.0, 1.0);
    for mut style in &mut fill {
        style.width = Val::Percent(frac * 100.0);
    }
    let line = format!("Floors cleared: {cleared} / {cap}");
    for mut text in &mut label {
        if text.sections[0].value != line {
            text.sections[0].value = line.clone();
        }
    }
}

fn handle_accept_button(
    mouse: Res<ButtonInput<MouseButton>>,
    press: Res<UiClickPress>,
    buttons: Query<(Entity, &Interaction), With<AcceptRewardsButton>>,
    mut events: EventWriter<AcceptRunRewards>,
) {
    if !mouse.just_released(MouseButton::Left) {
        return;
    }
    let Some(target) = press.0 else {
        return;
    };
    for (entity, interaction) in &buttons {
        if entity == target && ui_click_release_confirms(*interaction) {
            events.send(AcceptRunRewards);
            break;
        }
    }
}

fn handle_equip_buttons(
    mouse: Res<ButtonInput<MouseButton>>,
    press: Res<UiClickPress>,
    buttons: Query<(Entity, &Interaction, &EquipItemButton)>,
    mut events: EventWriter<EquipInventoryItem>,
) {
    if !mouse.just_released(MouseButton::Left) {
        return;
    }
    let Some(target) = press.0 else {
        return;
    };
    for (entity, interaction, button) in &buttons {
        if entity == target && ui_click_release_confirms(*interaction) {
            events.send(EquipInventoryItem {
                item_id: button.item_id,
            });
            break;
        }
    }
}

fn handle_skill_slot_buttons(
    mouse: Res<ButtonInput<MouseButton>>,
    press: Res<UiClickPress>,
    buttons: Query<(Entity, &Interaction, &SkillSlotButton)>,
    mut events: EventWriter<OpenSkillBook>,
    state: Res<State<GameState>>,
) {
    match state.get() {
        GameState::Build | GameState::Upgrades => {}
        _ => return,
    }
    if !mouse.just_released(MouseButton::Left) {
        return;
    }
    let Some(target) = press.0 else {
        return;
    };
    for (entity, interaction, btn) in &buttons {
        if entity == target && ui_click_release_confirms(*interaction) {
            events.send(OpenSkillBook {
                slot: btn.slot,
                kind: btn.kind,
            });
            break;
        }
    }
}

fn hero_display_name_for_slot(profile: &crate::save::SaveProfile, slot: u8) -> String {
    match slot {
        0 => profile.hero.name.clone(),
        1 => profile
            .party_partner
            .as_ref()
            .map(|p| p.name.clone())
            .unwrap_or_default(),
        _ => String::new(),
    }
}

fn handle_hero_name_edit_button(
    mouse: Res<ButtonInput<MouseButton>>,
    press: Res<UiClickPress>,
    buttons: Query<(Entity, &Interaction, &HeroNameEditButton)>,
    mut edit: ResMut<HeroNameEditState>,
    profile: Res<ProfileState>,
    state: Res<State<GameState>>,
) {
    match state.get() {
        GameState::Build | GameState::Upgrades => {}
        _ => return,
    }
    if !mouse.just_released(MouseButton::Left) {
        return;
    }
    let Some(target) = press.0 else {
        return;
    };
    for (entity, interaction, btn) in &buttons {
        if entity != target || !ui_click_release_confirms(*interaction) {
            continue;
        }
        if btn.slot == 1 && profile.profile.party_partner.is_none() {
            return;
        }
        edit.active_slot = Some(btn.slot);
        edit.buffer = hero_display_name_for_slot(&profile.profile, btn.slot);
        break;
    }
}

fn sync_hero_name_labels(
    profile: Res<ProfileState>,
    edit: Res<HeroNameEditState>,
    mut q: Query<(&HeroNameDisplayText, &mut Text)>,
) {
    for (tag, mut text) in &mut q {
        let display = if edit.active_slot == Some(tag.slot) {
            format!("{}▏", edit.buffer)
        } else {
            hero_display_name_for_slot(&profile.profile, tag.slot)
        };
        if text.sections[0].value != display {
            text.sections[0].value = display;
        }
    }
}

#[allow(deprecated)]
fn hero_rename_keyboard(
    mut edit: ResMut<HeroNameEditState>,
    mut profile: ResMut<ProfileState>,
    save_path: Res<ProfileSavePath>,
    keys: Res<ButtonInput<KeyCode>>,
    mut char_ev: EventReader<ReceivedCharacter>,
    state: Res<State<GameState>>,
    settings_modal: Query<(), With<SettingsModalRoot>>,
    skill_book: Query<(), With<SkillBookRoot>>,
) {
    if edit.active_slot.is_none() {
        for _ in char_ev.read() {}
        return;
    }
    match state.get() {
        GameState::Build | GameState::Upgrades => {}
        _ => {
            *edit = HeroNameEditState::default();
            for _ in char_ev.read() {}
            return;
        }
    }
    if !settings_modal.is_empty() || !skill_book.is_empty() {
        for _ in char_ev.read() {}
        return;
    }

    if keys.just_pressed(KeyCode::Escape) {
        *edit = HeroNameEditState::default();
        for _ in char_ev.read() {}
        return;
    }

    if keys.just_pressed(KeyCode::Enter) {
        let Some(slot) = edit.active_slot else {
            return;
        };
        let trimmed = edit.buffer.trim();
        let name: String = if trimmed.is_empty() {
            crate::domain::hero::DEFAULT_HERO_NAME.to_string()
        } else {
            trimmed
                .chars()
                .take(crate::domain::hero::MAX_HERO_NAME_LEN)
                .collect()
        };
        match slot {
            0 => profile.profile.hero.name = name,
            1 => {
                if let Some(p) = profile.profile.party_partner.as_mut() {
                    p.name = name;
                }
            }
            _ => {}
        }
        if let Err(e) = crate::save::save_profile(&save_path.0, &profile.profile) {
            warn!("failed to save after rename: {e}");
        }
        *edit = HeroNameEditState::default();
        for _ in char_ev.read() {}
        return;
    }

    if keys.just_pressed(KeyCode::Backspace) {
        edit.buffer.pop();
    }

    for ev in char_ev.read() {
        for c in ev.char.chars() {
            if c == '\r' || c == '\n' {
                continue;
            }
            if c.is_control() {
                continue;
            }
            if edit.buffer.chars().count() >= crate::domain::hero::MAX_HERO_NAME_LEN {
                continue;
            }
            edit.buffer.push(c);
        }
    }
}

fn handle_salvage_buttons(
    mouse: Res<ButtonInput<MouseButton>>,
    press: Res<UiClickPress>,
    buttons: Query<(Entity, &Interaction, &SalvageItemButton)>,
    mut events: EventWriter<SalvageInventoryItem>,
) {
    if !mouse.just_released(MouseButton::Left) {
        return;
    }
    let Some(target) = press.0 else {
        return;
    };
    for (entity, interaction, button) in &buttons {
        if entity == target && ui_click_release_confirms(*interaction) {
            events.send(SalvageInventoryItem {
                item_id: button.item_id,
            });
            break;
        }
    }
}

fn handle_buy_upgrade_buttons(
    mouse: Res<ButtonInput<MouseButton>>,
    press: Res<UiClickPress>,
    buttons: Query<(Entity, &Interaction, &BuyUpgradeButton)>,
    mut events: EventWriter<BuyUpgrade>,
) {
    if !mouse.just_released(MouseButton::Left) {
        return;
    }
    let Some(target) = press.0 else {
        return;
    };
    for (entity, interaction, button) in &buttons {
        if entity == target && ui_click_release_confirms(*interaction) {
            events.send(BuyUpgrade {
                upgrade: button.upgrade,
            });
            break;
        }
    }
}

fn handle_return_to_build_button(
    mouse: Res<ButtonInput<MouseButton>>,
    press: Res<UiClickPress>,
    buttons: Query<(Entity, &Interaction), With<ReturnToBuildButton>>,
    mut events: EventWriter<ReturnToBuild>,
) {
    if !mouse.just_released(MouseButton::Left) {
        return;
    }
    let Some(target) = press.0 else {
        return;
    };
    for (entity, interaction) in &buttons {
        if entity == target && ui_click_release_confirms(*interaction) {
            events.send(ReturnToBuild);
            break;
        }
    }
}

fn pin_playback_combat_log_scroll(
    playback: Res<ActiveRunPlayback>,
    mut prev_log: Local<String>,
    mut regions: Query<(Entity, &mut UiScrollState, &Node), With<PlaybackLogScrollRegion>>,
    children: Query<&Children>,
    mut content_set: ParamSet<(
        Query<&Node, With<UiScrollContent>>,
        Query<&mut Style, With<UiScrollContent>>,
    )>,
) {
    let body = playback.log_lines.join("\n");
    if body == *prev_log {
        return;
    }
    *prev_log = body.clone();

    for (entity, mut state, viewport_node) in &mut regions {
        let view_h = viewport_node.size().y;
        if view_h <= 0.0 {
            continue;
        }
        let Ok(ch) = children.get(entity) else {
            continue;
        };
        let Some(child) = ch
            .iter()
            .copied()
            .find(|&e| content_set.p0().get(e).is_ok())
        else {
            continue;
        };
        let content_h = content_set
            .p0()
            .get(child)
            .map(|n| n.size().y)
            .unwrap_or(0.0);
        let max_scroll = (content_h - view_h).max(0.0);
        state.offset = max_scroll;
        if let Ok(mut style) = content_set.p1().get_mut(child) {
            style.top = Val::Px(-state.offset);
        }
    }
}

fn apply_ui_scroll(
    mut wheel_events: EventReader<MouseWheel>,
    mut regions: Query<
        (Entity, &RelativeCursorPosition, &mut UiScrollState, &Node),
        With<UiScrollRegion>,
    >,
    children: Query<&Children>,
    mut content_style: Query<&mut Style, With<UiScrollContent>>,
    content_node: Query<&Node, With<UiScrollContent>>,
) {
    let delta: f32 = wheel_events.read().map(|e| e.y * 28.0).sum();
    if delta.abs() < f32::EPSILON {
        return;
    }

    for (entity, rel_pos, mut state, viewport_node) in &mut regions {
        if !rel_pos.mouse_over() {
            continue;
        }
        let view_h = viewport_node.size().y;
        if view_h <= 0.0 {
            continue;
        }
        let Ok(ch) = children.get(entity) else {
            continue;
        };
        let Some(child) = ch.iter().copied().find(|&e| content_node.get(e).is_ok()) else {
            continue;
        };
        let Ok(inner_node) = content_node.get(child) else {
            continue;
        };
        let content_h = inner_node.size().y;
        let max_scroll = (content_h - view_h).max(0.0);
        state.offset = (state.offset + delta).clamp(0.0, max_scroll);
        if let Ok(mut style) = content_style.get_mut(child) {
            style.top = Val::Px(-state.offset);
        }
        break;
    }
}

fn cleanup_ui(mut commands: Commands, roots: Query<Entity, With<UiRoot>>) {
    for root in &roots {
        commands.entity(root).despawn_recursive();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::{
        AcceptRunRewards, GameState, IdleDungeonsPlugin, LatestRunSummary, ProfileSavePath,
        ProfileState, SkipRunPlayback, StartRun,
    };
    use crate::domain::items::{GearSlot, ItemInstance};
    use crate::domain::progression::UpgradeId;
    use crate::ui::mockup_layout::{RightPanelTab, RightTabButton};
    use bevy::input::mouse::MouseButtonInput;
    use bevy::input::ButtonState;
    use bevy::state::app::StatesPlugin;
    use tempfile::tempdir;

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

    fn simulate_primary_click(app: &mut App, button: Entity) {
        // `mouse_button_input_system` clears `just_*` each frame and repopulates from events only,
        // so tests must submit `MouseButtonInput` (manual `press()`/`release()` is not visible as `just_pressed`/`just_released`).
        let window = app.world_mut().spawn_empty().id();
        app.world_mut().send_event(MouseButtonInput {
            button: MouseButton::Left,
            state: ButtonState::Pressed,
            window,
        });
        app.world_mut()
            .entity_mut(button)
            .insert(Interaction::Pressed);
        app.update();
        app.world_mut().send_event(MouseButtonInput {
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

        let button = single_entity::<StartRunButton>(app.world_mut());
        simulate_primary_click(&mut app, button);

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
        app.world_mut().send_event(SkipRunPlayback);
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
        app.world_mut().send_event(StartRun { seed: 1 });

        app.update();
        app.world_mut().send_event(SkipRunPlayback);
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
        app.world_mut().send_event(StartRun { seed: 1 });
        app.update();
        app.world_mut().send_event(SkipRunPlayback);
        app.update();
        app.update();

        let button = single_entity::<AcceptRewardsButton>(app.world_mut());
        simulate_primary_click(&mut app, button);

        assert_eq!(app.world().resource::<Events<AcceptRunRewards>>().len(), 1);
    }

    #[test]
    fn upgrades_screen_has_inventory_and_upgrade_buttons() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(StatesPlugin);
        app.add_plugins(IdleDungeonsPlugin);
        app.add_plugins(UiPlugin);
        app.world_mut()
            .resource_mut::<ProfileState>()
            .profile
            .inventory
            .push(ItemInstance::basic(1, "Iron Sword", GearSlot::Weapon));
        app.world_mut()
            .resource_mut::<NextState<GameState>>()
            .set(GameState::Upgrades);

        app.update();
        app.update();

        assert_eq!(entity_count::<UpgradeScreen>(app.world_mut()), 1);
        assert!(entity_count::<EquipItemButton>(app.world_mut()) >= 1);
        assert!(entity_count::<SalvageItemButton>(app.world_mut()) >= 1);
        press_right_panel_tab(&mut app, RightPanelTab::Upgrades);
        assert!(entity_count::<BuyUpgradeButton>(app.world_mut()) >= 1);
        assert_eq!(entity_count::<ReturnToBuildButton>(app.world_mut()), 1);
    }

    #[test]
    fn pressing_upgrade_button_updates_profile_and_refreshes_screen_text() {
        let dir = tempdir().unwrap();
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(StatesPlugin);
        app.insert_resource(ProfileSavePath(dir.path().join("profile.json")));
        app.add_plugins(IdleDungeonsPlugin);
        app.add_plugins(UiPlugin);
        app.update();

        app.world_mut()
            .resource_mut::<ProfileState>()
            .profile
            .meta
            .gold = 100;
        app.world_mut()
            .resource_mut::<NextState<GameState>>()
            .set(GameState::Upgrades);
        app.update();
        app.update();

        press_right_panel_tab(&mut app, RightPanelTab::Upgrades);

        let button = app
            .world_mut()
            .query_filtered::<(Entity, &BuyUpgradeButton), ()>()
            .iter(app.world())
            .find_map(|(entity, button)| {
                (button.upgrade == UpgradeId::BaseDamage).then_some(entity)
            })
            .unwrap();
        simulate_primary_click(&mut app, button);
        app.update();

        let profile = app.world().resource::<ProfileState>();
        assert_eq!(profile.profile.meta.gold, 90);
        assert_eq!(profile.profile.meta.upgrade_level(UpgradeId::BaseDamage), 1);

        let dumped = all_ui_text(app.world_mut());
        assert!(
            dumped.contains("90"),
            "expected refreshed gold value in UI, got: {dumped}"
        );
        assert!(
            dumped.contains("BaseDamage L1"),
            "expected refreshed upgrade label in UI, got: {dumped}"
        );
    }

    #[test]
    fn pressing_upgrade_button_sends_buy_upgrade_event() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(StatesPlugin);
        app.add_plugins(IdleDungeonsPlugin);
        app.add_plugins(UiPlugin);
        app.world_mut()
            .resource_mut::<NextState<GameState>>()
            .set(GameState::Upgrades);
        app.update();
        app.update();

        press_right_panel_tab(&mut app, RightPanelTab::Upgrades);

        let button = app
            .world_mut()
            .query_filtered::<(Entity, &BuyUpgradeButton), ()>()
            .iter(app.world())
            .find_map(|(entity, button)| {
                (button.upgrade == UpgradeId::BaseDamage).then_some(entity)
            })
            .unwrap();
        simulate_primary_click(&mut app, button);

        assert_eq!(
            app.world()
                .resource::<Events<crate::app::BuyUpgrade>>()
                .len(),
            1
        );
    }

    fn press_right_panel_tab(app: &mut App, tab: RightPanelTab) {
        let entity = app
            .world_mut()
            .query::<(Entity, &RightTabButton)>()
            .iter(app.world())
            .find_map(|(e, b)| (b.0 == tab).then_some(e))
            .expect("tab header button");
        simulate_primary_click(app, entity);
    }

    fn entity_count<T: Component>(world: &mut World) -> usize {
        let mut query = world.query_filtered::<Entity, With<T>>();
        query.iter(world).count()
    }

    fn single_entity<T: Component>(world: &mut World) -> Entity {
        let mut query = world.query_filtered::<Entity, With<T>>();
        query.single(world)
    }

    fn all_ui_text(world: &mut World) -> String {
        world
            .query::<&Text>()
            .iter(world)
            .flat_map(|text| text.sections.iter().map(|s| s.value.as_str()))
            .collect::<Vec<_>>()
            .join(" | ")
    }
}
