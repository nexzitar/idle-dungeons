pub mod assets;
pub mod buildcraft;
pub mod build_panel;
pub mod components;
pub mod gear_hub;
pub mod interaction;
pub mod inventory_panel;
pub mod log_panel;
pub mod playback_sync;
pub mod primitives;
pub mod run_panel;
pub mod scene_tune;
pub mod screens;
pub mod shell;
pub mod skill_book;
pub mod skill_presentation;
pub mod skill_shop;
pub mod stash_sort;
pub mod summary_panel;
pub mod systems;
pub mod theme;
pub mod title_camp;
pub mod tooltip;

#[cfg(test)]
mod plugin_tests;

pub(crate) use interaction::{ui_click_release_confirms, UiClickPress};
pub(crate) use screens::{
    spawn_build_screen_root, spawn_running_screen_root, spawn_summary_screen_root,
};
pub(crate) use systems::attach_gear_hub_if_kept_open;

use crate::app::GameState;
use crate::presentation::editor::{
    presentation_editor_tune_field_keyboard, PresentationEditorFieldEditState,
};
use crate::presentation::PresentationEditorSession;
use crate::ui::components::HeroNameEditState;
use bevy::app::MainScheduleOrder;
use bevy::asset::AssetPlugin;
use bevy::ecs::schedule::ScheduleLabel;
use bevy::input::InputPlugin;
use bevy::prelude::*;
use bevy::transform::prelude::TransformSystems;
use bevy::ui::UiSystems;

use playback_sync::{
    spawn_playback_floating_combat_text, sync_combat_log_toggle_label,
    sync_playback_aggro_arrow, sync_playback_aggro_arrow_line, sync_playback_cast_bars_foe,
    sync_playback_cast_bars_party, sync_playback_combat_log_panel_visibility,
    sync_playback_damage_meters, sync_playback_delve_progress_bar,
    sync_playback_theater_slot_visibility, sync_run_playback_debuff_slots,
    sync_run_playback_party_bars, sync_run_playback_ui, tick_floating_combat_popups,
};
use screens::{spawn_build_screen, spawn_running_screen, spawn_summary_screen};
use systems::{
    apply_ui_button_palettes, cleanup_running_exit, cleanup_ui, fulfill_reset_progress,
    hero_rename_keyboard, open_gear_hub_from_events, open_skill_book_from_events,
    open_skill_shop_from_events, raise_tooltip_above_modals,
    refresh_profile_screen_on_profile_change, reset_playback_combat_log_visibility,
    spawn_camera, spawn_title_screen, sync_hero_name_labels, sync_playback_speed_label,
    sync_top_bar,
};

#[derive(Resource, Default)]
pub(crate) struct GearHubKeepOpen(pub bool);

#[derive(Resource, Default)]
pub(crate) struct PlaybackCombatLogVisible(pub bool);

#[derive(Resource, Default)]
pub(crate) struct FloatingCombatPopupSeq(u32);

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
        app.init_resource::<interaction::click::UiPressedButtonEntitiesOnClick>();
        app.init_resource::<crate::ui::tooltip::TooltipState>();
        app.init_resource::<HeroNameEditState>();
        app.init_resource::<PlaybackCombatLogVisible>();
        app.init_resource::<FloatingCombatPopupSeq>();
        app.init_resource::<buildcraft::BuildcraftEditSession>();
        app.insert_resource(crate::ui::scene_tune::TitleSceneLayout::try_load_from_disk());
        app.init_resource::<PresentationEditorSession>();
        #[cfg(debug_assertions)]
        app.init_resource::<crate::presentation::editor::PresentationEditorDragState>();
        app.init_resource::<PresentationEditorFieldEditState>();
        #[cfg(debug_assertions)]
        app.init_resource::<crate::ui::scene_tune::TitleSceneTuneHintLogged>();
        app.add_systems(PreUpdate, raise_tooltip_above_modals);
        app.add_systems(
            Update,
            crate::ui::tooltip::hide_tooltip_layer_before_pointer_focus.before(UiSystems::Focus),
        );
        app.add_schedule(Schedule::new(RegisterUiPlaceholderImages));
        app.add_systems(
            RegisterUiPlaceholderImages,
            crate::ui::assets::register_ui_placeholder_images,
        );
        app.world_mut()
            .resource_mut::<MainScheduleOrder>()
            .insert_startup_before(StateTransition, RegisterUiPlaceholderImages);
        app.add_systems(Startup, spawn_camera);
        #[cfg(debug_assertions)]
        {
            app.add_systems(
                OnEnter(GameState::Title),
                (
                    crate::ui::scene_tune::log_title_scene_tune_hint_on_first_title_visit,
                    spawn_title_screen,
                )
                    .chain(),
            );
        }
        #[cfg(not(debug_assertions))]
        {
            app.add_systems(OnEnter(GameState::Title), spawn_title_screen);
        }
        app.add_systems(OnExit(GameState::Title), cleanup_ui)
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
                            interaction::click::capture_ui_pressed_button_entities,
                            interaction::click::capture_ui_click_start,
                            interaction::dispatch::dispatch_ui_clicks,
                        )
                            .chain(),
                        (
                            apply_ui_button_palettes,
                            fulfill_reset_progress,
                            open_skill_book_from_events,
                            open_gear_hub_from_events,
                            open_skill_shop_from_events,
                            hero_rename_keyboard,
                            buildcraft::sync::sync_buildcraft_hover,
                            buildcraft::sync::sync_buildcraft_inspect,
                            buildcraft::sync::sync_buildcraft_apply_enabled,
                            buildcraft::sync::sync_buildcraft_party_bars,
                            interaction::click::clear_ui_click_after_release,
                        ),
                    )
                        .chain(),
                    sync_top_bar,
                    sync_playback_speed_label,
                    sync_hero_name_labels,
                    sync_run_playback_ui.run_if(in_state(GameState::Running)),
                    sync_playback_cast_bars_party
                        .run_if(in_state(GameState::Running))
                        .after(sync_run_playback_ui),
                    sync_playback_cast_bars_foe
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
                    crate::ui::primitives::scroll::apply_ui_scroll
                        .after(TransformSystems::Propagate),
                    crate::ui::primitives::scroll::pin_playback_combat_log_scroll
                        .run_if(in_state(GameState::Running))
                        .after(crate::ui::primitives::scroll::apply_ui_scroll),
                    refresh_profile_screen_on_profile_change,
                    crate::ui::tooltip::update_tooltip.after(UiSystems::Layout),
                ),
            );
        #[cfg(debug_assertions)]
        app.add_systems(
            Update,
            interaction::dispatch::dispatch_editor_ui_clicks
                .after(interaction::dispatch::dispatch_ui_clicks),
        );
        #[cfg(debug_assertions)]
        app.add_systems(
            Update,
            crate::ui::scene_tune::title_scene_tune_hotkeys
                .run_if(in_state(GameState::Title))
                .after(apply_ui_button_palettes)
                .before(UiSystems::Focus),
        );
        #[cfg(debug_assertions)]
        app.add_systems(
            Update,
            (
                presentation_editor_tune_field_keyboard,
                crate::presentation::editor::toggle_presentation_editor_visibility,
                crate::presentation::editor::presentation_editor_pick,
                crate::presentation::editor::presentation_editor_drag,
                crate::presentation::editor::sync_presentation_editor_ui,
                crate::ui::scene_tune::sync_title_scene_elements,
                crate::ui::scene_tune::sync_presentation_layer_transforms,
                crate::ui::scene_tune::sync_title_fire_presentation_from_layout,
                crate::ui::scene_tune::title_scene_tune_selection_gizmo,
                crate::presentation::editor::presentation_editor_hover_outline,
                crate::ui::scene_tune::tick_title_fire_ambient,
            )
                .chain()
                .run_if(in_state(GameState::Title))
                .after(apply_ui_button_palettes)
                .after(UiSystems::Focus),
        );
        #[cfg(not(debug_assertions))]
        app.add_systems(
            Update,
            (
                crate::ui::scene_tune::sync_title_scene_elements,
                crate::ui::scene_tune::sync_presentation_layer_transforms,
                crate::ui::scene_tune::sync_title_fire_presentation_from_layout,
                crate::ui::scene_tune::tick_title_fire_ambient,
            )
                .run_if(in_state(GameState::Title)),
        );
    }
}
