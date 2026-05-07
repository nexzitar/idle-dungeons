use crate::domain::hero::HeroProfile;
use crate::domain::items::ItemInstance;
use crate::domain::loot::salvage_value;
use crate::domain::progression::UpgradeId;
use crate::domain::run::{
    simulate_run_with_playback, RunConfig, RunSimulation, RunSummary, DEFAULT_RUN_MAX_DEPTH,
};
use crate::save::{load_profile, save_profile, SaveProfile};
use crate::ui::UiPlugin;
use bevy::prelude::*;
use bevy::state::app::StatesPlugin;
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, States)]
pub enum GameState {
    #[default]
    Build,
    Running,
    Summary,
    Upgrades,
}

#[derive(Debug, Clone, Copy, Event)]
pub struct StartRun {
    pub seed: u64,
}

#[derive(Debug, Resource)]
pub struct LatestRunSummary {
    pub summary: RunSummary,
    pub rewards_accepted: bool,
}

#[derive(Debug, Clone, Event)]
pub struct AcceptRunRewards;

#[derive(Debug, Clone, Event)]
pub struct EquipInventoryItem {
    pub item_id: u64,
}

#[derive(Debug, Clone, Event)]
pub struct SalvageInventoryItem {
    pub item_id: u64,
}

#[derive(Debug, Clone, Copy, Event)]
pub struct BuyUpgrade {
    pub upgrade: UpgradeId,
}

#[derive(Debug, Clone, Copy, Event)]
pub struct ReturnToBuild;

#[derive(Debug, Clone, Event)]
pub struct SkipRunPlayback;

/// Wipe save to [`SaveProfile::default`] and return to briefing (handled in UI).
#[derive(Debug, Clone, Copy, Event)]
pub struct ResetProgress;

/// Playback timeline for [`GameState::Running`]; advances into [`LatestRunSummary`] unchanged.
#[derive(Debug, Resource)]
pub struct ActiveRunPlayback {
    pub frames: Vec<crate::domain::run::RunPlaybackFrame>,
    pub display_index: usize,
    pub elapsed: f32,
    pub log_lines: Vec<String>,
}

/// Display-only run speed multiplier (delve playback and future live combat).
#[derive(Debug, Resource, Clone, Copy)]
pub struct RunSpeedSetting(pub f32);

impl Default for RunSpeedSetting {
    fn default() -> Self {
        Self(1.0)
    }
}

#[derive(Debug, Resource)]
pub struct ProfileSavePath(pub PathBuf);

impl Default for ProfileSavePath {
    fn default() -> Self {
        Self(PathBuf::from("saves/profile.json"))
    }
}

#[derive(Debug, Resource)]
pub struct ProfileState {
    pub profile: SaveProfile,
}

impl FromWorld for ProfileState {
    fn from_world(world: &mut World) -> Self {
        let path = world
            .get_resource::<ProfileSavePath>()
            .map(|save_path| save_path.0.clone())
            .unwrap_or_else(|| ProfileSavePath::default().0);
        let profile = load_profile(&path).unwrap_or_default();
        Self { profile }
    }
}

impl ProfileState {
    pub fn effective_hero(&self) -> HeroProfile {
        let mut hero = self.profile.hero.clone();
        hero.base_stats = hero.base_stats + self.profile.meta.stat_bonus();
        hero.unlock_skill_slots(self.profile.meta.unlocked_skill_slots);
        hero
    }
}

pub struct IdleDungeonsPlugin;

impl Plugin for IdleDungeonsPlugin {
    fn build(&self, app: &mut App) {
        if !app.is_plugin_added::<StatesPlugin>() {
            app.add_plugins(StatesPlugin);
        }

        app.init_state::<GameState>()
            .init_resource::<ProfileSavePath>()
            .init_resource::<ProfileState>()
            .init_resource::<RunSpeedSetting>()
            .add_event::<StartRun>()
            .add_event::<AcceptRunRewards>()
            .add_event::<EquipInventoryItem>()
            .add_event::<SalvageInventoryItem>()
            .add_event::<BuyUpgrade>()
            .add_event::<ReturnToBuild>()
            .add_event::<SkipRunPlayback>()
            .add_event::<ResetProgress>()
            .add_systems(
                Update,
                (
                    start_run,
                    accept_run_rewards,
                    skip_run_playback,
                    tick_run_playback.run_if(in_state(GameState::Running)),
                    equip_inventory_item,
                    salvage_inventory_item,
                    buy_upgrade,
                    return_to_build,
                ),
            );
    }
}

fn start_run(
    mut commands: Commands,
    mut events: EventReader<StartRun>,
    mut next_state: ResMut<NextState<GameState>>,
    profile: Res<ProfileState>,
) {
    for event in events.read() {
        let hero = profile.effective_hero();
        let RunSimulation { summary, playback } = simulate_run_with_playback(
            &hero,
            RunConfig {
                seed: event.seed,
                max_depth: DEFAULT_RUN_MAX_DEPTH,
            },
        );
        commands.insert_resource(LatestRunSummary {
            summary,
            rewards_accepted: false,
        });
        let mut seed_lines = Vec::new();
        if let Some(first) = playback.frames.first() {
            if let Some(line) = playback_log_line(first) {
                seed_lines.push(line);
            }
        }
        commands.insert_resource(ActiveRunPlayback {
            frames: playback.frames,
            display_index: 0,
            elapsed: 0.0,
            log_lines: seed_lines,
        });
        next_state.set(GameState::Running);
    }
}

fn skip_run_playback(
    mut events: EventReader<SkipRunPlayback>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    for _ in events.read() {
        next_state.set(GameState::Summary);
    }
}

/// Seconds per playback step at 1x speed; scaled by [`RunSpeedSetting`].
const PLAYBACK_STEP_SECS: f32 = 0.52;

fn tick_run_playback(
    time: Res<Time>,
    speed: Res<RunSpeedSetting>,
    mut playback: ResMut<ActiveRunPlayback>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if playback.frames.is_empty() {
        next_state.set(GameState::Summary);
        return;
    }

    playback.elapsed += time.delta_seconds() * speed.0;
    while playback.elapsed >= PLAYBACK_STEP_SECS {
        playback.elapsed -= PLAYBACK_STEP_SECS;
        if playback.display_index + 1 < playback.frames.len() {
            playback.display_index += 1;
            if let Some(line) = playback_log_line(&playback.frames[playback.display_index]) {
                playback.log_lines.push(line);
            }
        } else {
            next_state.set(GameState::Summary);
            return;
        }
    }
}

fn playback_log_line(frame: &crate::domain::run::RunPlaybackFrame) -> Option<String> {
    Some(match &frame.kind {
        crate::domain::run::RunPlaybackFrameKind::Narration { text } => text.clone(),
        crate::domain::run::RunPlaybackFrameKind::Combat(c) => {
            format!("[Depth {}] {}", frame.depth, c.caption)
        }
    })
}

fn accept_run_rewards(
    mut events: EventReader<AcceptRunRewards>,
    mut latest_summary: Option<ResMut<LatestRunSummary>>,
    mut profile: ResMut<ProfileState>,
    save_path: Res<ProfileSavePath>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    for _ in events.read() {
        let Some(latest_summary) = latest_summary.as_deref_mut() else {
            continue;
        };
        if latest_summary.rewards_accepted {
            continue;
        }

        apply_run_rewards(&mut profile.profile, &latest_summary.summary);
        latest_summary.rewards_accepted = true;
        save_current_profile(&save_path, &profile);
        next_state.set(GameState::Upgrades);
    }
}

fn equip_inventory_item(
    mut events: EventReader<EquipInventoryItem>,
    mut profile: ResMut<ProfileState>,
    save_path: Res<ProfileSavePath>,
) {
    for event in events.read() {
        if let Some(item) = remove_inventory_item(&mut profile.profile.inventory, event.item_id) {
            if let Some(replaced) = profile.profile.hero.equipped_item(item.slot).cloned() {
                profile.profile.inventory.push(replaced);
            }
            let _ = profile.profile.hero.equip_item(item);
            save_current_profile(&save_path, &profile);
        }
    }
}

fn salvage_inventory_item(
    mut events: EventReader<SalvageInventoryItem>,
    mut profile: ResMut<ProfileState>,
    save_path: Res<ProfileSavePath>,
) {
    for event in events.read() {
        if let Some(item) = remove_inventory_item(&mut profile.profile.inventory, event.item_id) {
            profile.profile.meta.salvage += salvage_value(&item);
            save_current_profile(&save_path, &profile);
        }
    }
}

fn buy_upgrade(
    mut events: EventReader<BuyUpgrade>,
    mut profile: ResMut<ProfileState>,
    save_path: Res<ProfileSavePath>,
) {
    for event in events.read() {
        if profile.profile.meta.buy_upgrade(event.upgrade).is_ok() {
            save_current_profile(&save_path, &profile);
        }
    }
}

fn return_to_build(
    mut events: EventReader<ReturnToBuild>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    for _ in events.read() {
        next_state.set(GameState::Build);
    }
}

fn apply_run_rewards(profile: &mut SaveProfile, summary: &RunSummary) {
    profile.meta.gold += summary.gold_earned;
    profile.meta.salvage += summary.salvage_earned;
    profile.meta.add_skill_slot_progress(summary.deepest_depth);
    profile.inventory.extend(summary.loot.iter().cloned());
}

fn remove_inventory_item(inventory: &mut Vec<ItemInstance>, item_id: u64) -> Option<ItemInstance> {
    inventory
        .iter()
        .position(|item| item.id == item_id)
        .map(|index| inventory.remove(index))
}

fn save_current_profile(save_path: &ProfileSavePath, profile: &ProfileState) {
    if let Err(error) = save_profile(&save_path.0, &profile.profile) {
        warn!("failed to save profile: {error}");
    }
}

pub fn run() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Idle Dungeons".to_string(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(IdleDungeonsPlugin)
        .add_plugins(UiPlugin)
        .run();
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::items::{GearSlot, ItemInstance, ItemRarity};
    use crate::domain::loot::salvage_value;

    #[test]
    fn app_plugin_registers_game_state() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(IdleDungeonsPlugin);

        assert!(app.world().contains_resource::<State<GameState>>());
    }

    #[test]
    fn start_run_event_creates_run_summary_resource() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(IdleDungeonsPlugin);
        app.world_mut().send_event(StartRun { seed: 1 });

        app.update();

        assert!(app.world().contains_resource::<LatestRunSummary>());
    }

    #[test]
    fn app_plugin_loads_missing_profile_resource() {
        let dir = tempfile::tempdir().unwrap();
        let save_path = dir.path().join("profile.json");
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.insert_resource(ProfileSavePath(save_path));
        app.add_plugins(IdleDungeonsPlugin);

        app.update();

        let profile = app.world().resource::<ProfileState>();
        assert_eq!(profile.profile.meta.gold, 0);
        assert!(profile.profile.inventory.is_empty());
    }

    #[test]
    fn accepting_run_rewards_updates_profile_once_and_saves() {
        let dir = tempfile::tempdir().unwrap();
        let save_path = dir.path().join("profile.json");
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.insert_resource(ProfileSavePath(save_path.clone()));
        app.add_plugins(IdleDungeonsPlugin);
        app.world_mut().send_event(StartRun { seed: 1 });
        app.update();
        app.world_mut().send_event(SkipRunPlayback);
        app.update();

        let summary = app.world().resource::<LatestRunSummary>().summary.clone();
        app.world_mut().send_event(AcceptRunRewards);
        app.update();
        app.world_mut().send_event(AcceptRunRewards);
        app.update();

        let profile = app.world().resource::<ProfileState>();
        assert_eq!(profile.profile.meta.gold, summary.gold_earned);
        assert_eq!(profile.profile.meta.salvage, summary.salvage_earned);
        assert_eq!(profile.profile.inventory.len(), summary.loot.len());

        let saved = crate::save::load_profile(&save_path).unwrap();
        assert_eq!(saved.meta.gold, summary.gold_earned);
        assert_eq!(saved.inventory.len(), summary.loot.len());
    }

    #[test]
    fn equip_inventory_item_moves_item_to_hero_and_saves() {
        let dir = tempfile::tempdir().unwrap();
        let save_path = dir.path().join("profile.json");
        let item = ItemInstance::basic(42, "Iron Sword", GearSlot::Weapon);
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.insert_resource(ProfileSavePath(save_path.clone()));
        app.add_plugins(IdleDungeonsPlugin);
        app.world_mut()
            .resource_mut::<ProfileState>()
            .profile
            .inventory
            .push(item.clone());

        app.world_mut()
            .send_event(EquipInventoryItem { item_id: item.id });
        app.update();

        let profile = app.world().resource::<ProfileState>();
        assert_eq!(
            profile.profile.hero.equipped_item(GearSlot::Weapon),
            Some(&item)
        );
        assert!(profile.profile.inventory.is_empty());

        let saved = crate::save::load_profile(&save_path).unwrap();
        assert_eq!(saved.hero.equipped_item(GearSlot::Weapon), Some(&item));
    }

    #[test]
    fn salvage_inventory_item_adds_salvage_and_saves() {
        let dir = tempfile::tempdir().unwrap();
        let save_path = dir.path().join("profile.json");
        let mut item = ItemInstance::basic(7, "Rare Trinket", GearSlot::Trinket);
        item.rarity = ItemRarity::Rare;
        let expected_salvage = salvage_value(&item);
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.insert_resource(ProfileSavePath(save_path.clone()));
        app.add_plugins(IdleDungeonsPlugin);
        app.world_mut()
            .resource_mut::<ProfileState>()
            .profile
            .inventory
            .push(item.clone());

        app.world_mut()
            .send_event(SalvageInventoryItem { item_id: item.id });
        app.update();

        let profile = app.world().resource::<ProfileState>();
        assert_eq!(profile.profile.meta.salvage, expected_salvage);
        assert!(profile.profile.inventory.is_empty());

        let saved = crate::save::load_profile(&save_path).unwrap();
        assert_eq!(saved.meta.salvage, expected_salvage);
    }

    #[test]
    fn buy_upgrade_spends_gold_and_saves() {
        let dir = tempfile::tempdir().unwrap();
        let save_path = dir.path().join("profile.json");
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.insert_resource(ProfileSavePath(save_path.clone()));
        app.add_plugins(IdleDungeonsPlugin);
        app.world_mut()
            .resource_mut::<ProfileState>()
            .profile
            .meta
            .gold = 100;

        app.world_mut().send_event(BuyUpgrade {
            upgrade: UpgradeId::BaseDamage,
        });
        app.update();

        let profile = app.world().resource::<ProfileState>();
        assert_eq!(profile.profile.meta.gold, 90);
        assert_eq!(profile.profile.meta.upgrade_level(UpgradeId::BaseDamage), 1);

        let saved = crate::save::load_profile(&save_path).unwrap();
        assert_eq!(saved.meta.upgrade_level(UpgradeId::BaseDamage), 1);
    }
}
