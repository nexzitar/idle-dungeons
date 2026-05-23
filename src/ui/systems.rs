//! UI lifecycle, top-bar sync, modal open handlers, and hero rename.

use bevy::input::keyboard::KeyboardInput;
use bevy::input::{ButtonState, ButtonInput};
use bevy::prelude::*;

use crate::app::{
    ActiveRunPlayback, GameState, LatestRunSummary, OpenGearHub, OpenSkillBook, OpenSkillShop,
    ProfileSavePath, ProfileState, ResetProgress, RunSpeedSetting,
};
use crate::ui::assets::UiPlaceholderImages;
use crate::ui::components::{
    BuildScreen, GearHubRoot, HeroNameDisplayText, HeroNameEditState, MainCamera,
    PlaybackSpeedValueText, SettingsModalRoot, SkillBookRoot, SkillShopRoot, SummaryScreen,
    TitleScreen, TopBarField, UiButtonPalette, UiRoot,
};
use crate::ui::scene_tune::TitleSceneLayout;
use crate::ui::screens::{spawn_build_screen_root, spawn_summary_screen_root};
use crate::ui::{GearHubKeepOpen, PlaybackCombatLogVisible};

pub(crate) fn reset_playback_combat_log_visibility(mut v: ResMut<PlaybackCombatLogVisible>) {
    v.0 = false;
}


pub(crate) fn spawn_camera(mut commands: Commands) {
    commands.spawn((Camera2d, MainCamera));
}

/// Keep the tooltip layer painted above modals spawned later as additional root children.
pub(crate) fn raise_tooltip_above_modals(
    mut commands: Commands,
    roots: Query<Entity, With<UiRoot>>,
    tooltip: Query<Entity, With<crate::ui::tooltip::TooltipLayer>>,
) {
    let Ok(root) = roots.single() else {
        return;
    };
    let Ok(tip) = tooltip.single() else {
        return;
    };
    commands.entity(root).add_child(tip);
}

pub(crate) fn spawn_title_screen(
    mut commands: Commands,
    profile: Res<ProfileState>,
    speed: Res<RunSpeedSetting>,
    ph: Res<UiPlaceholderImages>,
    tune: Res<TitleSceneLayout>,
) {
    crate::ui::title_camp::spawn_title_screen(
        &mut commands,
        &profile,
        speed.multiplier(),
        &ph,
        &tune,
    );
}

pub(crate) fn cleanup_running_exit(mut commands: Commands, roots: Query<Entity, With<UiRoot>>) {
    for root in &roots {
        commands.entity(root).despawn();
    }
    commands.remove_resource::<ActiveRunPlayback>();
}

/// Re-spawns the gear hub modal after a full UI root rebuild when the player still has it open.
/// Re-spawns the gear hub modal after a full UI root rebuild when the player still has it open.
pub(crate) fn attach_gear_hub_if_kept_open(
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

pub(crate) fn refresh_profile_screen_on_profile_change(
    mut commands: Commands,
    profile: Res<ProfileState>,
    speed: Res<RunSpeedSetting>,
    state: Res<State<GameState>>,
    latest_summary: Option<Res<LatestRunSummary>>,
    build_roots: Query<Entity, With<BuildScreen>>,
    summary_roots: Query<Entity, With<SummaryScreen>>,
    title_roots: Query<Entity, With<TitleScreen>>,
    ph: Res<UiPlaceholderImages>,
    tune: Res<TitleSceneLayout>,
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
            crate::ui::title_camp::spawn_title_screen(
                &mut commands,
                &profile,
                speed.multiplier(),
                &ph,
                &tune,
            );
        }
        GameState::Build => {
            if build_roots.is_empty() {
                return;
            }
            for e in &build_roots {
                commands.entity(e).despawn();
            }
            *name_edit = HeroNameEditState::default();
            let root = spawn_build_screen_root(&mut commands, &profile, speed.multiplier(), &ph);
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
            let root = spawn_summary_screen_root(
                &mut commands,
                &profile,
                &summary,
                speed.multiplier(),
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
        GameState::Running => {}
    }
}

pub(crate) fn sync_playback_speed_label(
    speed: Res<RunSpeedSetting>,
    mut q: Query<&mut Text, With<PlaybackSpeedValueText>>,
) {
    if q.is_empty() {
        return;
    }
    if !speed.is_changed() {
        return;
    }
    let label = crate::ui::shell::fmt_speed_label(speed.multiplier());
    for mut text in &mut q {
        if text.0 != label {
            text.0 = label.clone();
        }
    }
}

pub(crate) fn sync_top_bar(
    state: Res<State<GameState>>,
    profile: Res<ProfileState>,
    latest_summary: Option<Res<LatestRunSummary>>,
    playback: Option<Res<ActiveRunPlayback>>,
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
        };
        if text.0 != value {
            text.0 = value;
        }
    }
}

pub(crate) fn apply_ui_button_palettes(
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

pub(crate) fn fulfill_reset_progress(
    mut events: MessageReader<ResetProgress>,
    mut profile: ResMut<ProfileState>,
    save_path: Res<ProfileSavePath>,
    mut commands: Commands,
    mut next_state: ResMut<NextState<GameState>>,
    state: Res<State<GameState>>,
    speed: Res<RunSpeedSetting>,
    roots: Query<Entity, With<UiRoot>>,
    ph: Res<UiPlaceholderImages>,
    tune: Res<TitleSceneLayout>,
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
            crate::ui::title_camp::spawn_title_screen(
                &mut commands,
                &profile,
                speed.multiplier(),
                &ph,
                &tune,
            );
        } else {
            next_state.set(GameState::Title);
        }
    }
}

pub(crate) fn open_skill_book_from_events(
    mut events: MessageReader<OpenSkillBook>,
    roots: Query<Entity, With<UiRoot>>,
    existing: Query<Entity, With<SkillBookRoot>>,
    mut commands: Commands,
    ph: Res<UiPlaceholderImages>,
    profile: Res<ProfileState>,
    mut session: ResMut<crate::ui::buildcraft::BuildcraftEditSession>,
) {
    for ev in events.read() {
        for e in existing.iter() {
            commands.entity(e).despawn();
        }
        let Ok(root) = roots.single() else {
            continue;
        };
        *session = crate::ui::buildcraft::BuildcraftEditSession::open_from_profile(
            &profile.profile,
            ev.kind,
            ev.slot,
        );
        let unlocked = profile.profile.meta.unlocked_skill_ids.clone();
        commands.entity(root).with_children(|parent| {
            crate::ui::skill_book::spawn_skill_book_modal(
                parent,
                ev.slot,
                ev.kind,
                &unlocked,
                &ph,
                &session,
            );
        });
    }
}

pub(crate) fn open_gear_hub_from_events(
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

pub(crate) fn open_skill_shop_from_events(
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

pub(crate) fn sync_hero_name_labels(
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

pub(crate) fn hero_rename_keyboard(
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

pub(crate) fn cleanup_ui(mut commands: Commands, roots: Query<Entity, With<UiRoot>>) {
    for root in &roots {
        commands.entity(root).despawn();
    }
}
