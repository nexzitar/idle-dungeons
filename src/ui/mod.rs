pub mod build_panel;
pub mod components;
pub mod gear_hub;
pub mod inventory_panel;
pub mod log_panel;
pub mod mockup_layout;
pub mod placeholder_graphics;
pub mod run_panel;
pub mod skill_book;
pub mod skill_shop;
pub mod stash_sort;
pub mod summary_panel;
pub mod theme;
pub mod title_camp;
pub mod tooltip;
pub mod widgets;

use crate::app::{
    AcceptRunRewards, ActiveRunPlayback, AssignHeroSkill, BuySkillUnlock, EquipInventoryItem,
    GameState, LatestRunSummary, OpenGearHub, OpenSkillBook, OpenSkillShop, ProfileSavePath,
    ProfileState, ResetProgress, RunSpeedSetting, SalvageInventoryItem, SkipRunPlayback, StartRun,
};
use crate::domain::items::ItemInstance;
use crate::domain::run::{RunPlaybackFrameKind, RunSummary};
use crate::ui::build_panel::build_panel_text;
use crate::ui::components::{
    AcceptRewardsButton, BuildScreen, CampfireFlame, EquipItemButton, GearHubBackdrop,
    GearHubCloseButton, GearHubOpenButton, GearHubRoot, HeroNameDisplayText, HeroNameEditButton,
    HeroNameEditState, MainCamera, PlaybackAggroArrowLine, PlaybackAggroArrowText,
    PlaybackAllyPortraitBlock,
    PlaybackCaptionText, PlaybackCombatLogPanel, PlaybackCombatLogToggleLabel, PlaybackDepthText,
    PlaybackDmgMeterEnemyFill, PlaybackDmgMeterEnemyValue, PlaybackDmgMeterLeadFill,
    PlaybackDmgMeterLeadValue, PlaybackDmgMeterPartnerFill, PlaybackDmgMeterPartnerRow,
    PlaybackDmgMeterPartnerValue, PlaybackEnemyBarFill, PlaybackEnemyDebuffLine,
    PlaybackEnemyNameText, PlaybackEnemyPortraitBlock, PlaybackHeroBarFill, PlaybackAllyBarFill,
    PlaybackAllyCastFill, PlaybackAllyCdFill, PlaybackLeadCastFill, PlaybackLeadCdFill,
    PlaybackFoeCastFill, PlaybackFoeCdFill,
    PlaybackHeroDebuffLine, PlaybackLogScrollRegion, PlaybackLogText, PlaybackProgressBarFill,
    PlaybackProgressLabel, PlaybackRoomKindText, PlaybackTheaterFloatLayer, ResetProgressButton,
    RunPlaybackScreen, SalvageItemButton, SettingsButton, SettingsModalBackdrop,
    SettingsModalCloseButton, SettingsModalRoot, SettingsModalSpeedButton, SettingsModalSpeedLabel,
    SkillBookBackdrop, SkillBookCloseButton, SkillBookPickButton, SkillBookRoot, SkillShopBackdrop,
    SkillShopBuyButton, SkillShopCloseButton, SkillShopOpenButton, SkillShopRoot, SkillSlotButton,
    SkipPlaybackButton, StartRunButton, StashSortCycleButton, SummaryScreen,
    TitleEnterCampButton, TitleQuitButton, TitleScreen, ToggleCombatLogButton, TopBarField,
    UiButtonPalette, UiRoot, UiScrollContent, UiScrollRegion, UiScrollState, UiTooltip,
    FloatingCombatPopup,
};
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
use bevy::input::{keyboard::KeyboardInput, ButtonState, InputPlugin};
use bevy::prelude::*;
use bevy::transform::prelude::TransformSystems;
use bevy::ui::{ComputedNode, RelativeCursorPosition, UiSystems};
use bevy::text::{TextColor, TextFont};

#[derive(Resource, Default)]
struct GearHubKeepOpen(pub bool);

#[derive(Resource, Default)]
struct PlaybackCombatLogVisible(pub bool);

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
        app.init_resource::<GearHubKeepOpen>();
        app.init_resource::<UiClickPress>();
        app.init_resource::<crate::ui::tooltip::TooltipState>();
        app.init_resource::<HeroNameEditState>();
        app.init_resource::<PlaybackCombatLogVisible>();
        app.add_systems(
            Update,
            crate::ui::tooltip::hide_tooltip_layer_before_pointer_focus.before(UiSystems::Focus),
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
            .add_systems(OnEnter(GameState::Title), spawn_title_screen)
            .add_systems(OnExit(GameState::Title), cleanup_ui)
            .add_systems(OnEnter(GameState::Build), spawn_build_screen)
            .add_systems(OnExit(GameState::Build), cleanup_ui)
            .add_systems(
                OnEnter(GameState::Running),
                (reset_playback_combat_log_visibility, spawn_running_screen).chain(),
            )
            .add_systems(OnExit(GameState::Running), cleanup_running_exit)
            .add_systems(
                Update,
                (
                    (
                        (
                            capture_ui_click_start,
                            apply_ui_button_palettes,
                            handle_title_enter_camp.run_if(in_state(GameState::Title)),
                            handle_title_quit.run_if(in_state(GameState::Title)),
                            handle_start_button.run_if(in_state(GameState::Build)),
                            handle_skip_playback_button.run_if(in_state(GameState::Running)),
                            send_reset_progress_requests,
                            fulfill_reset_progress,
                            open_settings_modal,
                            close_settings_modal,
                            handle_settings_modal_speed,
                        )
                            .chain(),
                        (
                            close_skill_book_modal,
                            close_gear_hub_modal,
                            close_skill_shop_modal,
                            handle_skill_book_pick,
                            handle_stash_sort_button,
                            handle_skill_slot_buttons,
                            request_gear_hub_open,
                            request_skill_shop_open,
                            handle_open_skill_book,
                            open_gear_hub_from_events,
                            open_skill_shop_from_events,
                            handle_skill_shop_purchase,
                            handle_hero_name_edit_button,
                            hero_rename_keyboard,
                        )
                            .chain(),
                    )
                        .chain(),
                    (
                        handle_accept_button.run_if(in_state(GameState::Summary)),
                        handle_equip_buttons.run_if(in_build_or_summary),
                        handle_salvage_buttons.run_if(in_build_or_summary),
                        clear_ui_click_after_release,
                    )
                        .chain()
                        .after(handle_open_skill_book),
                    sync_top_bar,
                    tick_campfire_flames.run_if(in_state(GameState::Title)),
                    sync_hero_name_labels,
                    sync_run_playback_ui.run_if(in_state(GameState::Running)),
                    sync_playback_cast_bars
                        .run_if(in_state(GameState::Running))
                        .after(sync_run_playback_ui),
                    sync_run_playback_party_bars
                        .run_if(in_state(GameState::Running))
                        .after(sync_run_playback_ui),
                    sync_playback_aggro_arrow
                        .run_if(in_state(GameState::Running))
                        .after(sync_run_playback_ui),
                    sync_playback_aggro_arrow_line
                        .run_if(in_state(GameState::Running))
                        .after(sync_run_playback_ui),
                    sync_playback_damage_meters
                        .run_if(in_state(GameState::Running))
                        .after(sync_run_playback_ui),
                    sync_playback_theater_slot_visibility
                        .run_if(in_state(GameState::Running))
                        .after(sync_run_playback_ui),
                    handle_toggle_combat_log_button.run_if(in_state(GameState::Running)),
                    sync_playback_combat_log_panel_visibility.run_if(in_state(GameState::Running)),
                    sync_combat_log_toggle_label.run_if(in_state(GameState::Running)),
                    spawn_playback_floating_combat_text.run_if(in_state(GameState::Running)),
                    tick_floating_combat_popups.run_if(in_state(GameState::Running)),
                    sync_run_playback_debuff_slots
                        .run_if(in_state(GameState::Running))
                        .after(sync_run_playback_ui),
                    sync_playback_delve_progress_bar.run_if(in_state(GameState::Running)),
                ),
            )
            .add_systems(OnEnter(GameState::Summary), spawn_summary_screen)
            .add_systems(OnExit(GameState::Summary), cleanup_ui)
            .add_systems(
                PostUpdate,
                (
                    apply_ui_scroll.after(TransformSystems::Propagate),
                    pin_playback_combat_log_scroll
                        .run_if(in_state(GameState::Running))
                        .after(apply_ui_scroll),
                    refresh_profile_screen_on_profile_change,
                    crate::ui::tooltip::update_tooltip.after(UiSystems::Layout),
                ),
            );
    }
}

fn reset_playback_combat_log_visibility(mut v: ResMut<PlaybackCombatLogVisible>) {
    v.0 = false;
}

fn in_build_or_summary(state: Res<State<GameState>>) -> bool {
    matches!(*state.get(), GameState::Build | GameState::Summary)
}

fn spawn_camera(mut commands: Commands) {
    commands.spawn((Camera2d, MainCamera));
}

fn spawn_title_screen(
    mut commands: Commands,
    profile: Res<ProfileState>,
    speed: Res<RunSpeedSetting>,
    ph: Res<UiPlaceholderImages>,
) {
    crate::ui::title_camp::spawn_title_screen(&mut commands, &profile, speed.0, &ph);
}

fn tick_campfire_flames(time: Res<Time>, mut q: Query<(&CampfireFlame, &mut BackgroundColor)>) {
    let t = time.elapsed_secs();
    for (flame, mut bg) in &mut q {
        let w = ((t * flame.speed + flame.phase_offset).sin() * 0.5 + 0.5).clamp(0.0, 1.0);
        let c = flame.base.mix(&flame.peak, w);
        *bg = c.into();
    }
}

fn handle_title_enter_camp(
    mouse: Res<ButtonInput<MouseButton>>,
    press: Res<UiClickPress>,
    buttons: Query<(Entity, &Interaction), With<TitleEnterCampButton>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if !mouse.just_released(MouseButton::Left) {
        return;
    }
    let Some(target) = press.0 else {
        return;
    };
    for (entity, interaction) in &buttons {
        if entity == target && ui_click_release_confirms(*interaction) {
            next_state.set(GameState::Build);
            break;
        }
    }
}

fn handle_title_quit(
    mouse: Res<ButtonInput<MouseButton>>,
    press: Res<UiClickPress>,
    buttons: Query<(Entity, &Interaction), With<TitleQuitButton>>,
    mut exit: MessageWriter<AppExit>,
) {
    if !mouse.just_released(MouseButton::Left) {
        return;
    }
    let Some(target) = press.0 else {
        return;
    };
    for (entity, interaction) in &buttons {
        if entity == target && ui_click_release_confirms(*interaction) {
            exit.write(AppExit::Success);
            break;
        }
    }
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

fn root_shell() -> impl Bundle {
    (
        Node {
            box_sizing: BoxSizing::BorderBox,
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            position_type: PositionType::Relative,
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Stretch,
            ..default()
        },
        BackgroundColor(Color::NONE),
    )
}

fn content_column_bundle() -> impl Bundle {
    (
        Node {
            box_sizing: BoxSizing::BorderBox,
            width: Val::Percent(100.0),
            flex_grow: 1.0,
            min_height: Val::Px(0.0),
            flex_direction: FlexDirection::Column,
            padding: UiRect::axes(Val::Px(18.0), Val::Px(14.0)),
            row_gap: Val::Px(12.0),
            align_items: AlignItems::Stretch,
            ..default()
        },
    )
}

fn cleanup_running_exit(mut commands: Commands, roots: Query<Entity, With<UiRoot>>) {
    for root in &roots {
        commands.entity(root).despawn();
    }
    commands.remove_resource::<ActiveRunPlayback>();
}

fn spawn_running_screen(
    mut commands: Commands,
    profile: Res<ProfileState>,
    speed: Res<RunSpeedSetting>,
    ph: Res<UiPlaceholderImages>,
) {
    spawn_running_screen_root(&mut commands, &profile, speed.0, &ph);
}

fn spawn_running_screen_root(
    commands: &mut Commands,
    profile: &ProfileState,
    speed_mult: f32,
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
                    crate::ui::mockup_layout::spawn_ornate_column(row, 1.0, |panel| {
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
                    crate::ui::mockup_layout::spawn_ornate_column(row, 1.25, |panel| {
                        crate::ui::mockup_layout::spawn_run_playback_middle_column(panel, ph);
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
    ph: Res<UiPlaceholderImages>,
) {
    spawn_build_screen_root(&mut commands, &profile, speed.0, &ph);
}

fn spawn_build_screen_root(
    commands: &mut Commands,
    profile: &ProfileState,
    speed_mult: f32,
    ph: &UiPlaceholderImages,
) -> Entity {
    let lead = profile.effective_hero();
    let partner = profile.effective_party_partner();
    let party_slots = profile.profile.meta.party_slots_unlocked();
    let meta = &profile.profile.meta;
    let loadout_lines: Vec<String> = build_panel_text(&lead)
        .lines()
        .map(|s| s.to_string())
        .collect();
    let stash = profile.profile.inventory.len();

    let root_entity = commands.spawn((root_shell(), UiRoot, BuildScreen)).id();
    commands.entity(root_entity).with_children(|root| {
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
                    crate::ui::mockup_layout::spawn_ornate_column(row, 1.0, |panel| {
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
                    crate::ui::mockup_layout::spawn_ornate_column(row, 1.25, |panel| {
                        crate::ui::mockup_layout::spawn_dungeon_briefing_column(
                            panel, stash, meta,
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
    root_entity
}

fn spawn_summary_screen(
    mut commands: Commands,
    profile: Res<ProfileState>,
    latest_summary: Option<Res<LatestRunSummary>>,
    speed: Res<RunSpeedSetting>,
    ph: Res<UiPlaceholderImages>,
) {
    let summary = latest_summary
        .as_deref()
        .map(|s| s.summary.clone())
        .unwrap_or_else(crate::ui::summary_panel::empty_run_summary);
    spawn_summary_screen_root(&mut commands, &profile, &summary, speed.0, &ph);
}

fn spawn_summary_screen_root(
    commands: &mut Commands,
    profile: &ProfileState,
    summary: &RunSummary,
    speed_mult: f32,
    ph: &UiPlaceholderImages,
) -> Entity {
    let meta = &profile.profile.meta;
    let lead = profile.effective_hero();
    let partner = profile.effective_party_partner();
    let party_slots = profile.profile.meta.party_slots_unlocked();
    let loadout_lines: Vec<String> = build_panel_text(&lead)
        .lines()
        .map(|s| s.to_string())
        .collect();

    let root_entity = commands.spawn((root_shell(), UiRoot, SummaryScreen)).id();
    commands.entity(root_entity).with_children(|root| {
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
                    crate::ui::mockup_layout::spawn_ornate_column(row, 1.0, |panel| {
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
                    crate::ui::mockup_layout::spawn_ornate_column(row, 1.25, |panel| {
                        crate::ui::mockup_layout::spawn_dungeon_summary_column(panel, summary);
                    });
                });
                crate::ui::mockup_layout::spawn_mockup_footer(
                    col,
                    crate::ui::mockup_layout::FooterMode::Summary,
                );
            });
            crate::ui::mockup_layout::spawn_summary_rewards_modal(
                root,
                summary,
                profile.profile.stash_sort,
            );
            crate::ui::tooltip::spawn_tooltip_layer(root);
        });
    root_entity
}

/// Re-spawns the gear hub modal after a full UI root rebuild when the player still has it open.
fn attach_gear_hub_if_kept_open(
    commands: &mut Commands,
    root: Entity,
    gear_keep: &GearHubKeepOpen,
    profile: &ProfileState,
    game_state: GameState,
    latest: Option<&LatestRunSummary>,
    ph: &UiPlaceholderImages,
) {
    if !gear_keep.0 {
        return;
    }
    let interactive = matches!(game_state, GameState::Build | GameState::Summary);
    let summary_loot = if matches!(game_state, GameState::Summary) {
        latest.map(|s| s.summary.loot.as_slice())
    } else {
        None
    };
    commands.entity(root).with_children(|parent| {
        crate::ui::gear_hub::spawn_gear_hub_modal(
            parent,
            profile,
            &profile.profile.inventory,
            summary_loot,
            interactive,
            ph,
        );
    });
}

fn handle_stash_sort_button(
    mouse: Res<ButtonInput<MouseButton>>,
    press: Res<UiClickPress>,
    sort_buttons: Query<(Entity, &Interaction), With<StashSortCycleButton>>,
    mut profile: ResMut<ProfileState>,
    save_path: Res<ProfileSavePath>,
    mut commands: Commands,
    speed: Res<RunSpeedSetting>,
    latest_summary: Option<Res<LatestRunSummary>>,
    state: Res<State<GameState>>,
    build_roots: Query<Entity, With<BuildScreen>>,
    summary_roots: Query<Entity, With<SummaryScreen>>,
    running_roots: Query<Entity, With<RunPlaybackScreen>>,
    ph: Res<UiPlaceholderImages>,
    mut name_edit: ResMut<HeroNameEditState>,
    gear_keep: Res<GearHubKeepOpen>,
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
                    commands.entity(e).despawn();
                }
                *name_edit = HeroNameEditState::default();
                let root = spawn_build_screen_root(&mut commands, &profile, speed.0, &ph);
                attach_gear_hub_if_kept_open(
                    &mut commands,
                    root,
                    &*gear_keep,
                    &profile,
                    GameState::Build,
                    latest_summary.as_deref(),
                    &ph,
                );
            }
            GameState::Summary => {
                let summary = latest_summary
                    .as_deref()
                    .map(|s| s.summary.clone())
                    .unwrap_or_else(crate::ui::summary_panel::empty_run_summary);
                for e in &summary_roots {
                    commands.entity(e).despawn();
                }
                *name_edit = HeroNameEditState::default();
                let root = spawn_summary_screen_root(
                    &mut commands,
                    &profile,
                    &summary,
                    speed.0,
                    &ph,
                );
                attach_gear_hub_if_kept_open(
                    &mut commands,
                    root,
                    &*gear_keep,
                    &profile,
                    GameState::Summary,
                    latest_summary.as_deref(),
                    &ph,
                );
            }
            GameState::Running => {
                for e in &running_roots {
                    commands.entity(e).despawn();
                }
                spawn_running_screen_root(&mut commands, &profile, speed.0, &ph);
            }
            GameState::Title => {}
        }
        break;
    }
}

fn refresh_profile_screen_on_profile_change(
    mut commands: Commands,
    profile: Res<ProfileState>,
    speed: Res<RunSpeedSetting>,
    state: Res<State<GameState>>,
    latest_summary: Option<Res<LatestRunSummary>>,
    build_roots: Query<Entity, With<BuildScreen>>,
    summary_roots: Query<Entity, With<SummaryScreen>>,
    title_roots: Query<Entity, With<TitleScreen>>,
    ph: Res<UiPlaceholderImages>,
    mut name_edit: ResMut<HeroNameEditState>,
    gear_keep: Res<GearHubKeepOpen>,
) {
    if !profile.is_changed() {
        return;
    }
    match state.get() {
        GameState::Title => {
            if title_roots.is_empty() {
                return;
            }
            for e in &title_roots {
                commands.entity(e).despawn();
            }
            crate::ui::title_camp::spawn_title_screen(&mut commands, &profile, speed.0, &ph);
        }
        GameState::Build => {
            if build_roots.is_empty() {
                return;
            }
            for e in &build_roots {
                commands.entity(e).despawn();
            }
            *name_edit = HeroNameEditState::default();
            let root = spawn_build_screen_root(&mut commands, &profile, speed.0, &ph);
            attach_gear_hub_if_kept_open(
                &mut commands,
                root,
                &*gear_keep,
                &profile,
                GameState::Build,
                latest_summary.as_deref(),
                &ph,
            );
        }
        GameState::Summary => {
            if summary_roots.is_empty() {
                return;
            }
            let summary = latest_summary
                .as_deref()
                .map(|s| s.summary.clone())
                .unwrap_or_else(crate::ui::summary_panel::empty_run_summary);
            for e in &summary_roots {
                commands.entity(e).despawn();
            }
            *name_edit = HeroNameEditState::default();
            let root =
                spawn_summary_screen_root(&mut commands, &profile, &summary, speed.0, &ph);
            attach_gear_hub_if_kept_open(
                &mut commands,
                root,
                &*gear_keep,
                &profile,
                GameState::Summary,
                latest_summary.as_deref(),
                &ph,
            );
        }
        GameState::Running => {}
    }
}

pub(crate) fn spawn_item_card(
    parent: &mut ChildSpawnerCommands<'_>,
    item: &ItemInstance,
    ph: &UiPlaceholderImages,
) {
    parent
        .spawn((
            Node {
                box_sizing: BoxSizing::BorderBox,
                width: Val::Percent(100.0),
                padding: UiRect::all(Val::Px(UiTheme::PANEL_INSET)),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::FlexStart,
                row_gap: Val::Px(8.0),
                border: UiRect::all(Val::Px(1.0)),
                ..default()
            },
            BackgroundColor(UiTheme::panel_bg_deep()),
            BorderColor::from(rarity_color(item.rarity).mix(&Color::BLACK, 0.45)),
        ))
        .with_children(|card| {
            card.spawn((
                Node {
                    box_sizing: BoxSizing::BorderBox,
                    width: Val::Percent(100.0),
                    flex_direction: FlexDirection::Row,
                    column_gap: Val::Px(10.0),
                    align_items: AlignItems::FlexStart,
                    ..default()
                },
            ))
            .with_children(|head| {
                head.spawn((
                    Node {
                        box_sizing: BoxSizing::BorderBox,
                        width: Val::Px(40.0),
                        height: Val::Px(40.0),
                        flex_shrink: 0.0,
                        ..default()
                    },
                    ImageNode {
                        image: ph.item_generic.clone(),
                        color: rarity_color(item.rarity).mix(&Color::WHITE, 0.35),
                        ..default()
                    },
                ));
                head.spawn((
                    Node {
                        box_sizing: BoxSizing::BorderBox,
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::FlexStart,
                        row_gap: Val::Px(4.0),
                        flex_grow: 1.0,
                        min_width: Val::Px(0.0),
                        ..default()
                    },
                ))
                .with_children(|txt| {
                    txt.spawn((
                        Text::new(item.name.clone()),
                        TextFont::from_font_size(UiTheme::FONT_SECTION),
                        TextColor(rarity_color(item.rarity)),
                    ));
                    txt.spawn(caption_text(format!("{:?} · {:?}", item.rarity, item.slot)));
                    txt.spawn(body_text(format_item_stat_summary(item)));
                    let aff = format_item_affix_lines(item);
                    if !aff.is_empty() {
                        txt.spawn(caption_text(aff));
                    }
                });
            });
            card.spawn((
                Node {
                    box_sizing: BoxSizing::BorderBox,
                    flex_direction: FlexDirection::Row,
                    column_gap: Val::Px(10.0),
                    align_items: AlignItems::Center,
                    ..default()
                },
            ))
            .with_children(|row| {
                let equip_pal = UiButtonPalette::equip();
                row.spawn((
                    Node {
                        box_sizing: BoxSizing::BorderBox,
                        width: Val::Px(108.0),
                        height: Val::Px(36.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        border: UiRect::all(Val::Px(1.0)),
                        ..default()
                    },
                    Button,
                    BackgroundColor(equip_pal.idle_bg),
                    BorderColor::from(equip_pal.idle_border),
                    EquipItemButton { item_id: item.id },
                    equip_pal,
                    UiTooltip::txt(
                        "Equip this item on your hero. It replaces whatever is currently in this gear slot.",
                    ),
                ))
                .with_children(|b| {
                    b.spawn((
                        Text::new("Equip"),
                        TextFont::from_font_size(UiTheme::FONT_BODY),
                        TextColor(Color::WHITE),
                    ));
                });
                let salvage_pal = UiButtonPalette::salvage();
                row.spawn((
                    Node {
                        box_sizing: BoxSizing::BorderBox,
                        width: Val::Px(108.0),
                        height: Val::Px(36.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        border: UiRect::all(Val::Px(1.0)),
                        ..default()
                    },
                    Button,
                    BackgroundColor(salvage_pal.idle_bg),
                    BorderColor::from(salvage_pal.idle_border),
                    SalvageItemButton { item_id: item.id },
                    salvage_pal,
                    UiTooltip::txt(
                        "Salvage this item for currency. The item is removed from your stash permanently.",
                    ),
                ))
                .with_children(|b| {
                    b.spawn((
                        Text::new("Salvage"),
                        TextFont::from_font_size(UiTheme::FONT_BODY),
                        TextColor(UiTheme::body()),
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
        GameState::Title | GameState::Build => meta.deepest_floor_reached.to_string(),
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
        if text.0 != value {
            text.0 = value;
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
        *border = bo.into();
    }
}

fn send_reset_progress_requests(
    mouse: Res<ButtonInput<MouseButton>>,
    press: Res<UiClickPress>,
    interactions: Query<(Entity, &Interaction), With<ResetProgressButton>>,
    mut events: MessageWriter<ResetProgress>,
) {
    if !mouse.just_released(MouseButton::Left) {
        return;
    }
    let Some(target) = press.0 else {
        return;
    };
    for (entity, interaction) in &interactions {
        if entity == target && ui_click_release_confirms(*interaction) {
            events.write(ResetProgress);
            break;
        }
    }
}

fn fulfill_reset_progress(
    mut events: MessageReader<ResetProgress>,
    mut profile: ResMut<ProfileState>,
    save_path: Res<ProfileSavePath>,
    mut commands: Commands,
    mut next_state: ResMut<NextState<GameState>>,
    state: Res<State<GameState>>,
    speed: Res<RunSpeedSetting>,
    roots: Query<Entity, With<UiRoot>>,
    ph: Res<UiPlaceholderImages>,
    mut name_edit: ResMut<HeroNameEditState>,
    mut gear_keep: ResMut<GearHubKeepOpen>,
) {
    for _ in events.read() {
        profile.profile = crate::save::SaveProfile::default();
        if let Err(e) = crate::save::save_profile(&save_path.0, &profile.profile) {
            warn!("failed to save profile after reset: {e}");
        }
        commands.remove_resource::<LatestRunSummary>();
        commands.remove_resource::<ActiveRunPlayback>();
        *name_edit = HeroNameEditState::default();
        gear_keep.0 = false;

        if *state.get() == GameState::Title {
            for e in &roots {
                commands.entity(e).despawn();
            }
            crate::ui::title_camp::spawn_title_screen(&mut commands, &profile, speed.0, &ph);
        } else {
            next_state.set(GameState::Title);
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
        let Ok(root) = roots.single() else {
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
            commands.entity(entity).despawn();
        }
    }
}

fn handle_open_skill_book(
    mut events: MessageReader<OpenSkillBook>,
    roots: Query<Entity, With<UiRoot>>,
    existing: Query<(), With<SkillBookRoot>>,
    mut commands: Commands,
    ph: Res<UiPlaceholderImages>,
    profile: Res<ProfileState>,
) {
    for ev in events.read() {
        if !existing.is_empty() {
            continue;
        }
        let Ok(root) = roots.single() else {
            continue;
        };
        let unlocked = profile.profile.meta.unlocked_skill_ids.clone();
        commands.entity(root).with_children(|parent| {
            crate::ui::skill_book::spawn_skill_book_modal(
                parent,
                ev.slot,
                ev.kind,
                &unlocked,
                &ph,
            );
        });
    }
}

fn request_gear_hub_open(
    mouse: Res<ButtonInput<MouseButton>>,
    press: Res<UiClickPress>,
    open_btns: Query<(Entity, &Interaction), With<GearHubOpenButton>>,
    mut writer: MessageWriter<OpenGearHub>,
) {
    if !mouse.just_released(MouseButton::Left) {
        return;
    }
    let Some(target) = press.0 else {
        return;
    };
    for (entity, interaction) in &open_btns {
        if entity == target && ui_click_release_confirms(*interaction) {
            writer.write(OpenGearHub);
            return;
        }
    }
}

fn open_gear_hub_from_events(
    mut events: MessageReader<OpenGearHub>,
    roots: Query<Entity, With<UiRoot>>,
    existing: Query<Entity, With<GearHubRoot>>,
    mut commands: Commands,
    ph: Res<UiPlaceholderImages>,
    profile: Res<ProfileState>,
    state: Res<State<GameState>>,
    latest: Option<Res<LatestRunSummary>>,
    mut gear_keep: ResMut<GearHubKeepOpen>,
) {
    for _ in events.read() {
        gear_keep.0 = true;
        for e in existing.iter() {
            commands.entity(e).despawn();
        }
        let Ok(root) = roots.single() else {
            continue;
        };
        let interactive = matches!(*state.get(), GameState::Build | GameState::Summary);
        let summary_loot = if matches!(*state.get(), GameState::Summary) {
            latest.as_ref().map(|s| s.summary.loot.as_slice())
        } else {
            None
        };
        commands.entity(root).with_children(|parent| {
            crate::ui::gear_hub::spawn_gear_hub_modal(
                parent,
                &profile,
                &profile.profile.inventory,
                summary_loot,
                interactive,
                &ph,
            );
        });
    }
}

fn close_gear_hub_modal(
    mouse: Res<ButtonInput<MouseButton>>,
    press: Res<UiClickPress>,
    backdrop: Query<(Entity, &Interaction), With<GearHubBackdrop>>,
    close_btn: Query<(Entity, &Interaction), With<GearHubCloseButton>>,
    modal: Query<Entity, With<GearHubRoot>>,
    mut commands: Commands,
    mut gear_keep: ResMut<GearHubKeepOpen>,
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
        gear_keep.0 = false;
        for entity in &modal {
            commands.entity(entity).despawn();
        }
    }
}

fn request_skill_shop_open(
    mouse: Res<ButtonInput<MouseButton>>,
    press: Res<UiClickPress>,
    open_btns: Query<(Entity, &Interaction), With<SkillShopOpenButton>>,
    mut writer: MessageWriter<OpenSkillShop>,
) {
    if !mouse.just_released(MouseButton::Left) {
        return;
    }
    let Some(target) = press.0 else {
        return;
    };
    for (entity, interaction) in &open_btns {
        if entity == target && ui_click_release_confirms(*interaction) {
            writer.write(OpenSkillShop);
            return;
        }
    }
}

fn open_skill_shop_from_events(
    mut events: MessageReader<OpenSkillShop>,
    roots: Query<Entity, With<UiRoot>>,
    existing: Query<Entity, With<SkillShopRoot>>,
    mut commands: Commands,
    profile: Res<ProfileState>,
    state: Res<State<GameState>>,
) {
    for _ in events.read() {
        if !matches!(*state.get(), GameState::Build) {
            continue;
        }
        for e in existing.iter() {
            commands.entity(e).despawn();
        }
        let Ok(root) = roots.single() else {
            continue;
        };
        let unlocked = profile.profile.meta.unlocked_skill_ids.clone();
        let gold = profile.profile.meta.gold;
        commands.entity(root).with_children(|parent| {
            crate::ui::skill_shop::spawn_skill_shop_modal(parent, &unlocked, gold);
        });
    }
}

fn close_skill_shop_modal(
    mouse: Res<ButtonInput<MouseButton>>,
    press: Res<UiClickPress>,
    backdrop: Query<(Entity, &Interaction), With<SkillShopBackdrop>>,
    close_btn: Query<(Entity, &Interaction), With<SkillShopCloseButton>>,
    modal: Query<Entity, With<SkillShopRoot>>,
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
            commands.entity(entity).despawn();
        }
    }
}

fn handle_skill_shop_purchase(
    mouse: Res<ButtonInput<MouseButton>>,
    press: Res<UiClickPress>,
    buttons: Query<(Entity, &Interaction, &SkillShopBuyButton)>,
    mut writer: MessageWriter<BuySkillUnlock>,
) {
    if !mouse.just_released(MouseButton::Left) {
        return;
    }
    let Some(target) = press.0 else {
        return;
    };
    for (entity, interaction, btn) in &buttons {
        if entity == target && ui_click_release_confirms(*interaction) {
            writer.write(BuySkillUnlock { skill: btn.skill });
            return;
        }
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
            commands.entity(entity).despawn();
        }
    }
}

fn handle_skill_book_pick(
    mouse: Res<ButtonInput<MouseButton>>,
    press: Res<UiClickPress>,
    buttons: Query<(Entity, &Interaction, &SkillBookPickButton)>,
    mut writer: MessageWriter<AssignHeroSkill>,
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
            writer.write(AssignHeroSkill {
                slot: pick.slot,
                skill: pick.skill,
                kind: pick.kind,
            });
            for e in &modal {
                commands.entity(e).despawn();
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
            text.0 = format!("Speed: {speed_label} (click to toggle)");
        }
        break;
    }
}

fn handle_start_button(
    mouse: Res<ButtonInput<MouseButton>>,
    press: Res<UiClickPress>,
    buttons: Query<(Entity, &Interaction), With<StartRunButton>>,
    mut start_run_events: MessageWriter<StartRun>,
) {
    if !mouse.just_released(MouseButton::Left) {
        return;
    }
    let Some(target) = press.0 else {
        return;
    };
    for (entity, interaction) in &buttons {
        if entity == target && ui_click_release_confirms(*interaction) {
            start_run_events.write(StartRun {
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
    mut events: MessageWriter<SkipRunPlayback>,
) {
    if !mouse.just_released(MouseButton::Left) {
        return;
    }
    let Some(target) = press.0 else {
        return;
    };
    for (entity, interaction) in &buttons {
        if entity == target && ui_click_release_confirms(*interaction) {
            events.write(SkipRunPlayback);
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
        Query<&mut Node, With<PlaybackHeroBarFill>>,
        Query<&mut Node, With<PlaybackEnemyBarFill>>,
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

    let log_plain = crate::ui::theme::playback_log_plain(&playback.log_lines);

    for mut text in params.p0().iter_mut() {
        if text.0 != depth_s {
            text.0 = depth_s.clone();
        }
    }
    for mut text in params.p1().iter_mut() {
        if text.0 != kind_s {
            text.0 = kind_s.clone();
        }
    }
    for mut text in params.p2().iter_mut() {
        if text.0 != enemy_name {
            text.0 = enemy_name.clone();
        }
    }
    for mut text in params.p3().iter_mut() {
        if text.0 != caption {
            text.0 = caption.clone();
        }
    }
    for mut text in params.p4().iter_mut() {
        if text.0 != log_plain {
            text.0.clone_from(&log_plain);
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

fn sync_playback_cast_bars(
    playback: Res<ActiveRunPlayback>,
    mut params: ParamSet<(
        Query<&mut Node, With<PlaybackLeadCastFill>>,
        Query<&mut Node, With<PlaybackLeadCdFill>>,
        Query<&mut Node, With<PlaybackAllyCastFill>>,
        Query<&mut Node, With<PlaybackAllyCdFill>>,
        Query<&mut Node, With<PlaybackFoeCastFill>>,
        Query<&mut Node, With<PlaybackFoeCdFill>>,
    )>,
) {
    if playback.frames.is_empty() {
        return;
    }
    let idx = playback.display_index.min(playback.frames.len() - 1);
    let f = &playback.frames[idx];
    let crate::domain::run::RunPlaybackFrameKind::Combat(c) = &f.kind else {
        return;
    };
    let lc = Val::Percent((c.lead_cast * 100.0).clamp(0.0, 100.0));
    let lcdn = Val::Percent((c.lead_cd * 100.0).clamp(0.0, 100.0));
    let ac = Val::Percent((c.ally_cast * 100.0).clamp(0.0, 100.0));
    let acdn = Val::Percent((c.ally_cd * 100.0).clamp(0.0, 100.0));
    let fc = Val::Percent((c.foe_cast * 100.0).clamp(0.0, 100.0));
    let fcdn = Val::Percent((c.foe_cd * 100.0).clamp(0.0, 100.0));
    for mut n in params.p0().iter_mut() {
        n.width = lc;
    }
    for mut n in params.p1().iter_mut() {
        n.width = lcdn;
    }
    for mut n in params.p2().iter_mut() {
        n.width = ac;
    }
    for mut n in params.p3().iter_mut() {
        n.width = acdn;
    }
    for mut n in params.p4().iter_mut() {
        n.width = fc;
    }
    for mut n in params.p5().iter_mut() {
        n.width = fcdn;
    }
}

fn sync_run_playback_party_bars(
    playback: Res<ActiveRunPlayback>,
    mut ally_bar: Query<&mut Node, With<PlaybackAllyBarFill>>,
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
}

fn sync_playback_aggro_arrow(
    playback: Res<ActiveRunPlayback>,
    mut aggro: Query<&mut Text, With<PlaybackAggroArrowText>>,
) {
    if playback.frames.is_empty() {
        return;
    }
    let idx = playback.display_index.min(playback.frames.len() - 1);
    let frame = &playback.frames[idx];
    let arrow_s = match &frame.kind {
        RunPlaybackFrameKind::Narration { .. } => "\u{2014}".to_string(),
        RunPlaybackFrameKind::Combat(c) => {
            let has_partner = frame.partner_snapshot_max_hp.is_some();
            let t0 = c.threat_slot0.unwrap_or(0);
            let t1 = c.threat_slot1.unwrap_or(0);
            let label = crate::domain::combat::aggro_arrow_target_label(
                [t0, t1],
                frame.hero_snapshot_hp,
                frame.partner_snapshot_hp.unwrap_or(0),
                has_partner,
                c.foe_last_target,
            );
            format!("\u{2192} {label}")
        }
    };
    for mut text in &mut aggro {
        if text.0 != arrow_s {
            text.0 = arrow_s.clone();
        }
    }
}

fn sync_playback_theater_slot_visibility(
    playback: Res<ActiveRunPlayback>,
    mut ally: Query<
        &mut Visibility,
        (
            With<PlaybackAllyPortraitBlock>,
            Without<PlaybackEnemyPortraitBlock>,
        ),
    >,
    mut enemy: Query<
        &mut Visibility,
        (
            With<PlaybackEnemyPortraitBlock>,
            Without<PlaybackAllyPortraitBlock>,
        ),
    >,
) {
    if playback.frames.is_empty() {
        return;
    }
    let idx = playback.display_index.min(playback.frames.len() - 1);
    let frame = &playback.frames[idx];
    let show_ally = frame.partner_snapshot_max_hp.is_some();
    let show_enemy = matches!(frame.kind, RunPlaybackFrameKind::Combat(_));
    for mut v in &mut ally {
        *v = if show_ally {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
    for mut v in &mut enemy {
        *v = if show_enemy {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
}

fn sync_playback_aggro_arrow_line(
    playback: Res<ActiveRunPlayback>,
    mut q: Query<(&mut Node, &mut Visibility), With<PlaybackAggroArrowLine>>,
) {
    if playback.frames.is_empty() {
        return;
    }
    let idx = playback.display_index.min(playback.frames.len() - 1);
    let frame = &playback.frames[idx];
    let Ok((mut style, mut vis)) = q.single_mut() else {
        return;
    };
    match &frame.kind {
        RunPlaybackFrameKind::Narration { .. } => {
            *vis = Visibility::Hidden;
        }
        RunPlaybackFrameKind::Combat(c) => {
            let has_partner = frame.partner_snapshot_max_hp.is_some();
            let t0 = c.threat_slot0.unwrap_or(0);
            let t1 = c.threat_slot1.unwrap_or(0);
            let slot = crate::domain::combat::pick_party_enemy_target(
                0,
                [t0, t1],
                frame.hero_snapshot_hp,
                frame.partner_snapshot_hp.unwrap_or(0),
                has_partner,
                c.foe_last_target,
            );
            let top = if slot == 0 { 30.0 } else { 58.0 };
            *vis = Visibility::Visible;
            style.position_type = PositionType::Absolute;
            style.top = Val::Percent(top);
            style.right = Val::Percent(14.0);
            style.width = Val::Percent(44.0);
            style.height = Val::Px(4.0);
            style.left = Val::Auto;
            style.bottom = Val::Auto;
        }
    }
}

fn sync_playback_damage_meters(
    playback: Res<ActiveRunPlayback>,
    mut partner_row: Query<&mut Visibility, With<PlaybackDmgMeterPartnerRow>>,
    mut fills: ParamSet<(
        Query<&mut Node, With<PlaybackDmgMeterLeadFill>>,
        Query<&mut Node, With<PlaybackDmgMeterPartnerFill>>,
        Query<&mut Node, With<PlaybackDmgMeterEnemyFill>>,
    )>,
    mut vals: ParamSet<(
        Query<&mut Text, With<PlaybackDmgMeterLeadValue>>,
        Query<&mut Text, With<PlaybackDmgMeterPartnerValue>>,
        Query<&mut Text, With<PlaybackDmgMeterEnemyValue>>,
    )>,
) {
    if playback.frames.is_empty() {
        return;
    }
    let idx = playback.display_index.min(playback.frames.len() - 1);
    let frame = &playback.frames[idx];
    let has_partner = frame.partner_snapshot_max_hp.is_some();
    let (p0, p1, fe) = match &frame.kind {
        RunPlaybackFrameKind::Combat(c) => (
            c.damage_meter_party_0,
            c.damage_meter_party_1,
            c.damage_meter_foe,
        ),
        _ => (0u32, 0u32, 0u32),
    };
    let max = p0.max(p1).max(fe).max(1) as f32;
    let w0 = (p0 as f32 / max * 100.0).clamp(0.0, 100.0);
    let w1 = (p1 as f32 / max * 100.0).clamp(0.0, 100.0);
    let wf = (fe as f32 / max * 100.0).clamp(0.0, 100.0);
    for mut s in fills.p0().iter_mut() {
        s.width = Val::Percent(w0);
    }
    for mut s in fills.p1().iter_mut() {
        s.width = Val::Percent(w1);
    }
    for mut s in fills.p2().iter_mut() {
        s.width = Val::Percent(wf);
    }
    let s0 = p0.to_string();
    let s1 = p1.to_string();
    let sf = fe.to_string();
    for mut t in vals.p0().iter_mut() {
        if t.0 != s0 {
            t.0.clone_from(&s0);
        }
    }
    for mut t in vals.p1().iter_mut() {
        if t.0 != s1 {
            t.0.clone_from(&s1);
        }
    }
    for mut t in vals.p2().iter_mut() {
        if t.0 != sf {
            t.0.clone_from(&sf);
        }
    }
    for mut v in &mut partner_row {
        *v = if has_partner {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
}

fn handle_toggle_combat_log_button(
    mouse: Res<ButtonInput<MouseButton>>,
    press: Res<UiClickPress>,
    buttons: Query<(Entity, &Interaction), With<ToggleCombatLogButton>>,
    mut vis: ResMut<PlaybackCombatLogVisible>,
) {
    if !mouse.just_released(MouseButton::Left) {
        return;
    }
    let Some(target) = press.0 else {
        return;
    };
    for (entity, interaction) in &buttons {
        if entity == target && ui_click_release_confirms(*interaction) {
            vis.0 = !vis.0;
            break;
        }
    }
}

fn sync_playback_combat_log_panel_visibility(
    vis: Res<PlaybackCombatLogVisible>,
    mut panel: Query<&mut Visibility, With<PlaybackCombatLogPanel>>,
) {
    let v = if vis.0 {
        Visibility::Visible
    } else {
        Visibility::Hidden
    };
    for mut pv in &mut panel {
        *pv = v;
    }
}

fn sync_combat_log_toggle_label(
    vis: Res<PlaybackCombatLogVisible>,
    mut labels: Query<&mut Text, With<PlaybackCombatLogToggleLabel>>,
) {
    let s = if vis.0 {
        "Hide log"
    } else {
        "Show log"
    };
    for mut text in &mut labels {
        if text.0 != s {
            text.0 = s.to_string();
        }
    }
}

fn spawn_playback_floating_combat_text(
    playback: Res<ActiveRunPlayback>,
    mut last_idx: Local<Option<usize>>,
    float_layer: Query<Entity, With<PlaybackTheaterFloatLayer>>,
    mut commands: Commands,
) {
    if playback.frames.is_empty() {
        return;
    }
    let idx = playback.display_index.min(playback.frames.len() - 1);
    if *last_idx == Some(idx) {
        return;
    }
    *last_idx = Some(idx);

    let frame = &playback.frames[idx];
    let (caption, anchor) = match &frame.kind {
        RunPlaybackFrameKind::Combat(c) => (c.caption.clone(), c.sfx_anchor),
        _ => return,
    };
    if matches!(anchor, crate::domain::combat::CombatSfxAnchor::Neutral) {
        return;
    }
    let Ok(parent) = float_layer.single() else {
        return;
    };
    let color = crate::ui::theme::playback_float_text_color(&caption);
    let font_size = UiTheme::FONT_COMPACT;
    let mut pos = Node {
                box_sizing: BoxSizing::BorderBox,
                position_type: PositionType::Absolute,
            max_width: Val::Px(200.0),
            padding: UiRect::axes(Val::Px(6.0), Val::Px(2.0)),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
            };
    match anchor {
        crate::domain::combat::CombatSfxAnchor::Lead => {
            pos.left = Val::Percent(4.0);
            pos.right = Val::Auto;
            pos.top = Val::Percent(10.0);
        }
        crate::domain::combat::CombatSfxAnchor::Ally => {
            pos.left = Val::Percent(4.0);
            pos.right = Val::Auto;
            pos.top = Val::Percent(52.0);
        }
        crate::domain::combat::CombatSfxAnchor::Enemy => {
            pos.right = Val::Percent(4.0);
            pos.left = Val::Auto;
            pos.top = Val::Percent(28.0);
        }
        crate::domain::combat::CombatSfxAnchor::Neutral => return,
    }

    commands.entity(parent).with_children(|layer| {
        layer
            .spawn((pos, FloatingCombatPopup { ttl: 0.95 }))
            .with_children(|pop| {
                pop.spawn((
                    Text::new(caption),
                    TextFont::from_font_size(font_size),
                    TextColor(color),
                ));
            });
    });
}

fn tick_floating_combat_popups(
    time: Res<Time>,
    mut commands: Commands,
    mut q: Query<(Entity, &mut FloatingCombatPopup)>,
) {
    let dt = time.delta_secs();
    for (entity, mut pop) in &mut q {
        pop.ttl -= dt;
        if pop.ttl <= 0.0 {
            commands.entity(entity).despawn();
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
    let hs = crate::ui::theme::playback_debuff_line_string(&hero_line);
    let fs = crate::ui::theme::playback_debuff_line_string(&foe_line);
    for mut text in &mut hero {
        if text.0 != hs {
            text.0.clone_from(&hs);
        }
    }
    for mut text in &mut foe {
        if text.0 != fs {
            text.0.clone_from(&fs);
        }
    }
}

fn sync_playback_delve_progress_bar(
    playback: Res<ActiveRunPlayback>,
    mut fill: Query<&mut Node, With<PlaybackProgressBarFill>>,
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
        if text.0 != line {
            text.0 = line.clone();
        }
    }
}

fn handle_accept_button(
    mouse: Res<ButtonInput<MouseButton>>,
    press: Res<UiClickPress>,
    buttons: Query<(Entity, &Interaction), With<AcceptRewardsButton>>,
    mut events: MessageWriter<AcceptRunRewards>,
) {
    if !mouse.just_released(MouseButton::Left) {
        return;
    }
    let Some(target) = press.0 else {
        return;
    };
    for (entity, interaction) in &buttons {
        if entity == target && ui_click_release_confirms(*interaction) {
            events.write(AcceptRunRewards);
            break;
        }
    }
}

fn handle_equip_buttons(
    mouse: Res<ButtonInput<MouseButton>>,
    press: Res<UiClickPress>,
    buttons: Query<(Entity, &Interaction, &EquipItemButton)>,
    mut events: MessageWriter<EquipInventoryItem>,
) {
    if !mouse.just_released(MouseButton::Left) {
        return;
    }
    let Some(target) = press.0 else {
        return;
    };
    for (entity, interaction, button) in &buttons {
        if entity == target && ui_click_release_confirms(*interaction) {
            events.write(EquipInventoryItem {
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
    mut events: MessageWriter<OpenSkillBook>,
    state: Res<State<GameState>>,
) {
    if *state.get() != GameState::Build {
        return;
    }
    if !mouse.just_released(MouseButton::Left) {
        return;
    }
    let Some(target) = press.0 else {
        return;
    };
    for (entity, interaction, btn) in &buttons {
        if entity == target && ui_click_release_confirms(*interaction) {
            events.write(OpenSkillBook {
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
    if *state.get() != GameState::Build {
        return;
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
        if text.0 != display {
            text.0 = display;
        }
    }
}

fn hero_rename_keyboard(
    mut edit: ResMut<HeroNameEditState>,
    mut profile: ResMut<ProfileState>,
    save_path: Res<ProfileSavePath>,
    keys: Res<ButtonInput<KeyCode>>,
    mut kb: MessageReader<KeyboardInput>,
    state: Res<State<GameState>>,
    settings_modal: Query<(), With<SettingsModalRoot>>,
    skill_book: Query<(), With<SkillBookRoot>>,
) {
    if edit.active_slot.is_none() {
        for _ in kb.read() {}
        return;
    }
    if *state.get() != GameState::Build {
        *edit = HeroNameEditState::default();
        for _ in kb.read() {}
        return;
    }
    if !settings_modal.is_empty() || !skill_book.is_empty() {
        for _ in kb.read() {}
        return;
    }

    if keys.just_pressed(KeyCode::Escape) {
        *edit = HeroNameEditState::default();
        for _ in kb.read() {}
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
        for _ in kb.read() {}
        return;
    }

    if keys.just_pressed(KeyCode::Backspace) {
        edit.buffer.pop();
    }

    for ev in kb.read() {
        if ev.state != ButtonState::Pressed || ev.repeat {
            continue;
        }
        let Some(t) = ev.text.as_ref() else {
            continue;
        };
        for c in t.chars() {
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
    mut events: MessageWriter<SalvageInventoryItem>,
) {
    if !mouse.just_released(MouseButton::Left) {
        return;
    }
    let Some(target) = press.0 else {
        return;
    };
    for (entity, interaction, button) in &buttons {
        if entity == target && ui_click_release_confirms(*interaction) {
            events.write(SalvageInventoryItem {
                item_id: button.item_id,
            });
            break;
        }
    }
}

fn pin_playback_combat_log_scroll(
    playback: Res<ActiveRunPlayback>,
    vis: Res<PlaybackCombatLogVisible>,
    mut prev_log: Local<String>,
    mut regions: Query<(Entity, &mut UiScrollState, &ComputedNode), With<PlaybackLogScrollRegion>>,
    children: Query<&Children>,
    mut content_set: ParamSet<(
        Query<&ComputedNode, With<UiScrollContent>>,
        Query<&mut Node, With<UiScrollContent>>,
    )>,
) {
    if !vis.0 {
        return;
    }
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
        let mut scroll_child = None;
        for e in ch.iter() {
            if content_set.p0().get(e).is_ok() {
                scroll_child = Some(e);
                break;
            }
        }
        let Some(child) = scroll_child else {
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
    mut wheel_events: MessageReader<MouseWheel>,
    mut regions: Query<
        (Entity, &RelativeCursorPosition, &mut UiScrollState, &ComputedNode),
        With<UiScrollRegion>,
    >,
    children: Query<&Children>,
    mut content_style: Query<&mut Node, With<UiScrollContent>>,
    content_node: Query<&ComputedNode, With<UiScrollContent>>,
) {
    let delta: f32 = wheel_events.read().map(|e| e.y * 28.0).sum();
    if delta.abs() < f32::EPSILON {
        return;
    }

    for (entity, rel_pos, mut state, viewport_node) in &mut regions {
        if !rel_pos.cursor_over() {
            continue;
        }
        let view_h = viewport_node.size().y;
        if view_h <= 0.0 {
            continue;
        }
        let Ok(ch) = children.get(entity) else {
            continue;
        };
        let mut scroll_child = None;
        for e in ch.iter() {
            if content_node.get(e).is_ok() {
                scroll_child = Some(e);
                break;
            }
        }
        let Some(child) = scroll_child else {
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
        commands.entity(root).despawn();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::{
        AcceptRunRewards, GameState, IdleDungeonsPlugin, LatestRunSummary, OpenGearHub,
        SkipRunPlayback, StartRun,
    };
    use bevy::input::mouse::MouseButtonInput;
    use bevy::input::ButtonState;
    use bevy::state::app::StatesPlugin;
    use bevy::ecs::message::Messages;

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
        // `mouse_button_input_system` clears `just_*` each frame and repopulates from events only,
        // so tests must submit `MouseButtonInput` (manual `press()`/`release()` is not visible as `just_pressed`/`just_released`).
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

        assert_eq!(app.world().resource::<Messages<AcceptRunRewards>>().len(), 1);
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
        query.single(world).expect("expected exactly one matching entity")
    }
}
