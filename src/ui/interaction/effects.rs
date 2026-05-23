//! Click side-effects that need screen rebuilds or multi-system state.

use bevy::prelude::*;

use crate::app::{GameState, LatestRunSummary, ProfileSavePath, ProfileState};
use crate::ui::assets::UiPlaceholderImages;
use crate::ui::components::HeroNameEditState;
use crate::ui::GearHubKeepOpen;

pub(crate) fn apply_cycle_stash_sort(
    profile: &mut ProfileState,
    save_path: &ProfileSavePath,
    commands: &mut Commands,
    speed_mult: f32,
    game_state: GameState,
    latest_summary: Option<&LatestRunSummary>,
    build_roots: &[Entity],
    summary_roots: &[Entity],
    running_roots: &[Entity],
    ph: &UiPlaceholderImages,
    name_edit: &mut HeroNameEditState,
    gear_keep: &GearHubKeepOpen,
) {
    profile.profile.stash_sort = profile.profile.stash_sort.toggle();
    if let Err(e) = crate::save::save_profile(&save_path.0, &profile.profile) {
        warn!("failed to save stash sort preference: {e}");
    }

    match game_state {
        GameState::Build => {
            for e in build_roots {
                commands.entity(*e).despawn();
            }
            *name_edit = HeroNameEditState::default();
            let root = crate::ui::spawn_build_screen_root(commands, profile, speed_mult, ph);
            crate::ui::attach_gear_hub_if_kept_open(
                commands,
                root,
                gear_keep,
                profile,
                GameState::Build,
                latest_summary,
                ph,
            );
        }
        GameState::Summary => {
            let summary = latest_summary
                .map(|s| s.summary.clone())
                .unwrap_or_else(crate::ui::summary_panel::empty_run_summary);
            for e in summary_roots {
                commands.entity(*e).despawn();
            }
            *name_edit = HeroNameEditState::default();
            let root =
                crate::ui::spawn_summary_screen_root(commands, profile, &summary, speed_mult, ph);
            crate::ui::attach_gear_hub_if_kept_open(
                commands,
                root,
                gear_keep,
                profile,
                GameState::Summary,
                latest_summary,
                ph,
            );
        }
        GameState::Running => {
            for e in running_roots {
                commands.entity(*e).despawn();
            }
            crate::ui::spawn_running_screen_root(commands, profile, speed_mult, ph);
        }
        GameState::Title => {}
    }
}
