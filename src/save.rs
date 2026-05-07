use crate::domain::hero::HeroProfile;
use crate::domain::items::ItemInstance;
use crate::domain::progression::MetaProgression;
use serde::{Deserialize, Serialize};
use std::path::Path;

/// Persisted stash / loot list ordering in the profile JSON.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum StashSortOrder {
    /// Same order as [`SaveProfile::inventory`] in the vec: last element is most recently appended.
    /// Display **newest first** in the UI.
    #[default]
    Recent,
    /// Rare → Uncommon → Common, then item name (`str`), then `id`.
    RarityName,
}

impl StashSortOrder {
    pub fn toggle(self) -> Self {
        match self {
            Self::Recent => Self::RarityName,
            Self::RarityName => Self::Recent,
        }
    }

    pub fn button_label(self) -> &'static str {
        match self {
            Self::Recent => "Order: · newest ·",
            Self::RarityName => "Order: · rarity ·",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SaveProfile {
    pub hero: HeroProfile,
    pub inventory: Vec<ItemInstance>,
    pub meta: MetaProgression,
    #[serde(default)]
    pub stash_sort: StashSortOrder,
}

impl SaveProfile {
    /// Keep [`HeroProfile::unlocked_skill_slots`] aligned with meta progression (source of truth).
    pub fn sync_skill_slot_unlocks(&mut self) {
        self.hero.unlock_skill_slots(self.meta.unlocked_skill_slots);
    }
}

impl Default for SaveProfile {
    fn default() -> Self {
        let mut s = Self {
            hero: HeroProfile::default(),
            inventory: vec![],
            meta: MetaProgression::default(),
            stash_sort: StashSortOrder::default(),
        };
        s.sync_skill_slot_unlocks();
        s
    }
}

#[derive(Debug, thiserror::Error)]
pub enum SaveError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("parse error: {0}")]
    Parse(#[from] serde_json::Error),
}

pub fn load_profile(path: &Path) -> Result<SaveProfile, SaveError> {
    if !path.exists() {
        return Ok(SaveProfile::default());
    }
    let contents = std::fs::read_to_string(path)?;
    let profile = serde_json::from_str(&contents)?;
    Ok(profile)
}

pub fn save_profile(path: &Path, profile: &SaveProfile) -> Result<(), SaveError> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let contents = serde_json::to_string_pretty(profile)?;
    std::fs::write(path, contents)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::progression::MetaProgression;

    #[test]
    fn missing_save_returns_fresh_profile() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("profile.json");

        let profile = load_profile(&path).unwrap();

        assert_eq!(profile.meta, MetaProgression::default());
    }

    #[test]
    fn save_round_trip_preserves_profile() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("profile.json");
        let mut profile = SaveProfile::default();
        profile.meta.gold = 55;

        save_profile(&path, &profile).unwrap();
        let loaded = load_profile(&path).unwrap();

        assert_eq!(loaded, profile);
    }

    #[test]
    fn stash_sort_round_trips_in_save() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("profile.json");
        let mut profile = SaveProfile::default();
        profile.stash_sort = StashSortOrder::RarityName;

        save_profile(&path, &profile).unwrap();
        let loaded = load_profile(&path).unwrap();

        assert_eq!(loaded.stash_sort, StashSortOrder::RarityName);
    }

    #[test]
    fn stash_sort_defaults_when_json_field_removed() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("profile.json");
        let mut profile = SaveProfile::default();
        profile.stash_sort = StashSortOrder::RarityName;
        save_profile(&path, &profile).unwrap();

        let mut value: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
        value
            .as_object_mut()
            .expect("profile object")
            .remove("stash_sort");
        std::fs::write(&path, serde_json::to_string(&value).unwrap()).unwrap();

        let loaded = load_profile(&path).unwrap();
        assert_eq!(loaded.stash_sort, StashSortOrder::Recent);
    }

    #[test]
    fn corrupt_save_returns_error() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("profile.json");
        std::fs::write(&path, "{not-json").unwrap();

        let result = load_profile(&path);

        assert!(matches!(result, Err(SaveError::Parse(_))));
    }
}
