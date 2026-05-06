# MVP Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build the first playable Bevy MVP for Idle Dungeons: one automated hero, skill slots, gear synergy, deterministic dungeon runs, loot, meta-progression, save/load, and a functional UI.

**Architecture:** Keep the core simulation testable without Bevy UI dependencies. Implement domain modules first with Rust unit tests, then connect them to Bevy plugins, resources, events, app states, and UI screens.

**Tech Stack:** Rust, Bevy, `rand`/`rand_chacha` for seeded RNG, `serde`/`serde_json` for local saves, standard Rust unit and integration tests.

---

## Scope Notes

This plan covers the MVP only. It intentionally excludes multiple heroes, full party formation, complex procedural maps, online systems, external content tooling, and asset-heavy visuals.

Because this repository is a Rust application, `Cargo.lock` should be committed. The current `.gitignore` ignores it, so Task 1 fixes that before scaffolding the project.

## File Structure

- `Cargo.toml`: package metadata and dependencies.
- `Cargo.lock`: generated dependency lockfile; commit it for this game application.
- `.gitignore`: update to stop ignoring `Cargo.lock`.
- `src/main.rs`: Bevy app entrypoint.
- `src/lib.rs`: module exports and app plugin assembly.
- `src/app.rs`: top-level Bevy plugin wiring and game states.
- `src/domain/mod.rs`: pure simulation domain module exports.
- `src/domain/stats.rs`: base stats, derived stats, modifiers, and stat math.
- `src/domain/skills.rs`: skill IDs, definitions, tags, triggers, slot rules.
- `src/domain/items.rs`: gear slots, item definitions, rarity, affixes, equip rules.
- `src/domain/hero.rs`: hero profile, loadout, skill slots, gear slots, stat derivation.
- `src/domain/dungeon.rs`: room types, encounter generation, depth scaling.
- `src/domain/combat.rs`: fixed-tick combat resolution and combat events.
- `src/domain/loot.rs`: deterministic loot generation, salvage values, inventory helpers.
- `src/domain/progression.rs`: currencies, permanent upgrades, skill slot unlocks.
- `src/domain/run.rs`: complete run state machine using dungeon, combat, loot, and rewards.
- `src/save.rs`: save profile schema, load/save helpers, missing/corrupt file behavior.
- `src/ui/mod.rs`: UI plugin exports.
- `src/ui/build_panel.rs`: skill, gear, and derived stat display.
- `src/ui/run_panel.rs`: run status display.
- `src/ui/log_panel.rs`: combat and run event display.
- `src/ui/inventory_panel.rs`: inventory, equipment comparison, salvage actions.
- `src/ui/upgrade_panel.rs`: permanent upgrades and skill slot progress.
- `src/ui/summary_panel.rs`: run result display.
- `tests/simulation_mvp.rs`: integration tests for deterministic complete runs.

## Task 1: Project Bootstrap And Lockfile Policy

**Files:**

- Create: `Cargo.toml`
- Create: `src/main.rs`
- Create: `src/lib.rs`
- Create: `src/app.rs`
- Modify: `.gitignore`
- **Step 1: Fix the lockfile ignore rule**

Remove the `Cargo.lock` line from `.gitignore` so the application lockfile can be committed.

Expected `.gitignore` Rust section:

```gitignore
# Rust
/target/
```

- **Step 2: Create `Cargo.toml`**

```toml
[package]
name = "idle_dungeons"
version = "0.1.0"
edition = "2021"

[dependencies]
bevy = "0.14"
rand = "0.8"
rand_chacha = "0.3"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
thiserror = "1"

[dev-dependencies]
tempfile = "3"
```

- **Step 3: Create the minimal app entrypoint**

`src/main.rs`:

```rust
fn main() {
    idle_dungeons::run();
}
```

`src/lib.rs`:

```rust
pub mod app;

pub fn run() {
    app::run();
}
```

`src/app.rs`:

```rust
use bevy::prelude::*;

pub struct IdleDungeonsPlugin;

impl Plugin for IdleDungeonsPlugin {
    fn build(&self, _app: &mut App) {}
}

pub fn run() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(IdleDungeonsPlugin)
        .run();
}
```

- **Step 4: Verify bootstrap compiles**

Run: `cargo check`

Expected: command exits 0 and creates `Cargo.lock`.

- **Step 5: Commit**

```bash
git add .gitignore Cargo.toml Cargo.lock src/main.rs src/lib.rs src/app.rs
git commit -m "Bootstrap Bevy project"
```

## Task 2: Stats, Skills, Items, And Hero Loadouts

**Files:**

- Create: `src/domain/mod.rs`
- Create: `src/domain/stats.rs`
- Create: `src/domain/skills.rs`
- Create: `src/domain/items.rs`
- Create: `src/domain/hero.rs`
- Modify: `src/lib.rs`
- **Step 1: Export the domain module**

Add to `src/lib.rs`:

```rust
pub mod app;
pub mod domain;

pub fn run() {
    app::run();
}
```

Create `src/domain/mod.rs`:

```rust
pub mod hero;
pub mod items;
pub mod skills;
pub mod stats;
```

- **Step 2: Write failing stat derivation tests**

Add tests at the bottom of `src/domain/hero.rs` before implementation:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::items::{GearSlot, ItemAffix, ItemInstance, ItemRarity};
    use crate::domain::skills::SkillId;
    use crate::domain::stats::Stats;

    #[test]
    fn derived_stats_include_base_stats_equipped_skills_and_gear() {
        let mut hero = HeroProfile::new(Stats {
            max_health: 100,
            damage: 10,
            armor: 2,
            attack_speed: 1.0,
            healing_power: 0,
        });

        hero.unlock_skill_slots(2);
        hero.equip_skill(0, SkillId::LifestealStrike).unwrap();
        hero.equip_item(ItemInstance {
            id: 1,
            name: "Vampiric Sword".to_string(),
            slot: GearSlot::Weapon,
            rarity: ItemRarity::Uncommon,
            stats: Stats {
                max_health: 0,
                damage: 4,
                armor: 0,
                attack_speed: 0.2,
                healing_power: 0,
            },
            affixes: vec![ItemAffix::Vampiric],
        })
        .unwrap();

        let derived = hero.derived_stats();

        assert_eq!(derived.max_health, 100);
        assert_eq!(derived.damage, 14);
        assert_eq!(derived.armor, 2);
        assert_eq!(derived.healing_power, 2);
        assert!((derived.attack_speed - 1.2).abs() < f32::EPSILON);
    }

    #[test]
    fn locked_skill_slots_reject_skills() {
        let mut hero = HeroProfile::default();

        let result = hero.equip_skill(1, SkillId::Guard);

        assert_eq!(result, Err(HeroError::SkillSlotLocked { slot: 1 }));
    }

    #[test]
    fn gear_replaces_only_matching_slot() {
        let mut hero = HeroProfile::default();
        let armor = ItemInstance::basic(1, "Iron Armor", GearSlot::Armor);

        hero.equip_item(armor).unwrap();

        assert!(hero.equipped_item(GearSlot::Armor).is_some());
        assert!(hero.equipped_item(GearSlot::Weapon).is_none());
    }
}
```

- **Step 3: Run tests and verify RED**

Run: `cargo test domain::hero --lib`

Expected: fails because `HeroProfile`, `Stats`, `SkillId`, `ItemInstance`, and related APIs do not exist yet.

- **Step 4: Implement `src/domain/stats.rs`**

```rust
use serde::{Deserialize, Serialize};
use std::ops::Add;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Stats {
    pub max_health: i32,
    pub damage: i32,
    pub armor: i32,
    pub attack_speed: f32,
    pub healing_power: i32,
}

impl Default for Stats {
    fn default() -> Self {
        Self {
            max_health: 100,
            damage: 10,
            armor: 0,
            attack_speed: 1.0,
            healing_power: 0,
        }
    }
}

impl Add for Stats {
    type Output = Stats;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            max_health: self.max_health + rhs.max_health,
            damage: self.damage + rhs.damage,
            armor: self.armor + rhs.armor,
            attack_speed: self.attack_speed + rhs.attack_speed,
            healing_power: self.healing_power + rhs.healing_power,
        }
    }
}
```

- **Step 5: Implement `src/domain/skills.rs`**

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SkillId {
    LifestealStrike,
    Guard,
    HeavyStrike,
    PoisonEdge,
    ThornSkin,
    BarrierPulse,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SkillTag {
    Attack,
    Defense,
    Healing,
    Poison,
    Barrier,
    Thorns,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SkillTrigger {
    OnAttack,
    OnHitTaken,
    OnRoomStart,
    PeriodicTick,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SkillDefinition {
    pub id: SkillId,
    pub name: &'static str,
    pub trigger: SkillTrigger,
    pub tags: &'static [SkillTag],
    pub description: &'static str,
}

pub fn skill_definition(id: SkillId) -> SkillDefinition {
    match id {
        SkillId::LifestealStrike => SkillDefinition {
            id,
            name: "Lifesteal Strike",
            trigger: SkillTrigger::OnAttack,
            tags: &[SkillTag::Attack, SkillTag::Healing],
            description: "Attacks restore a small amount of health.",
        },
        SkillId::Guard => SkillDefinition {
            id,
            name: "Guard",
            trigger: SkillTrigger::OnHitTaken,
            tags: &[SkillTag::Defense],
            description: "Reduces incoming damage.",
        },
        SkillId::HeavyStrike => SkillDefinition {
            id,
            name: "Heavy Strike",
            trigger: SkillTrigger::OnAttack,
            tags: &[SkillTag::Attack],
            description: "Slow attacks hit harder.",
        },
        SkillId::PoisonEdge => SkillDefinition {
            id,
            name: "Poison Edge",
            trigger: SkillTrigger::OnAttack,
            tags: &[SkillTag::Attack, SkillTag::Poison],
            description: "Attacks apply poison.",
        },
        SkillId::ThornSkin => SkillDefinition {
            id,
            name: "Thorn Skin",
            trigger: SkillTrigger::OnHitTaken,
            tags: &[SkillTag::Defense, SkillTag::Thorns],
            description: "Reflects damage when hit.",
        },
        SkillId::BarrierPulse => SkillDefinition {
            id,
            name: "Barrier Pulse",
            trigger: SkillTrigger::OnRoomStart,
            tags: &[SkillTag::Defense, SkillTag::Barrier],
            description: "Starts each room with a barrier.",
        },
    }
}
```

- **Step 6: Implement `src/domain/items.rs`**

```rust
use crate::domain::stats::Stats;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum GearSlot {
    Weapon,
    Armor,
    Trinket,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ItemRarity {
    Common,
    Uncommon,
    Rare,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ItemAffix {
    Vampiric,
    Heavy,
    Cursed,
    Spiked,
    Relentless,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ItemInstance {
    pub id: u64,
    pub name: String,
    pub slot: GearSlot,
    pub rarity: ItemRarity,
    pub stats: Stats,
    pub affixes: Vec<ItemAffix>,
}

impl ItemInstance {
    pub fn basic(id: u64, name: impl Into<String>, slot: GearSlot) -> Self {
        Self {
            id,
            name: name.into(),
            slot,
            rarity: ItemRarity::Common,
            stats: Stats {
                max_health: 0,
                damage: 0,
                armor: 0,
                attack_speed: 0.0,
                healing_power: 0,
            },
            affixes: Vec::new(),
        }
    }

    pub fn affix_stats(&self) -> Stats {
        self.affixes.iter().fold(
            Stats {
                max_health: 0,
                damage: 0,
                armor: 0,
                attack_speed: 0.0,
                healing_power: 0,
            },
            |stats, affix| {
                stats
                    + match affix {
                        ItemAffix::Vampiric => Stats { healing_power: 2, ..Stats::default_zero() },
                        ItemAffix::Heavy => Stats { damage: 4, attack_speed: -0.2, ..Stats::default_zero() },
                        ItemAffix::Cursed => Stats { armor: 5, healing_power: -2, ..Stats::default_zero() },
                        ItemAffix::Spiked => Stats { armor: 2, damage: 1, ..Stats::default_zero() },
                        ItemAffix::Relentless => Stats { attack_speed: 0.3, ..Stats::default_zero() },
                    }
            },
        )
    }
}

impl Stats {
    pub fn default_zero() -> Self {
        Self {
            max_health: 0,
            damage: 0,
            armor: 0,
            attack_speed: 0.0,
            healing_power: 0,
        }
    }
}
```

- **Step 7: Implement `src/domain/hero.rs`**

```rust
use crate::domain::items::{GearSlot, ItemInstance};
use crate::domain::skills::SkillId;
use crate::domain::stats::Stats;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum HeroError {
    #[error("skill slot {slot} is locked")]
    SkillSlotLocked { slot: usize },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HeroProfile {
    pub base_stats: Stats,
    pub unlocked_skill_slots: usize,
    pub equipped_skills: Vec<Option<SkillId>>,
    pub equipped_items: HashMap<GearSlot, ItemInstance>,
}

impl Default for HeroProfile {
    fn default() -> Self {
        Self::new(Stats::default())
    }
}

impl HeroProfile {
    pub fn new(base_stats: Stats) -> Self {
        Self {
            base_stats,
            unlocked_skill_slots: 0,
            equipped_skills: vec![None, None, None, None, None, None],
            equipped_items: HashMap::new(),
        }
    }

    pub fn unlock_skill_slots(&mut self, count: usize) {
        self.unlocked_skill_slots = count.min(self.equipped_skills.len());
    }

    pub fn equip_skill(&mut self, slot: usize, skill: SkillId) -> Result<(), HeroError> {
        if slot >= self.unlocked_skill_slots {
            return Err(HeroError::SkillSlotLocked { slot });
        }
        self.equipped_skills[slot] = Some(skill);
        Ok(())
    }

    pub fn equip_item(&mut self, item: ItemInstance) -> Result<(), HeroError> {
        self.equipped_items.insert(item.slot, item);
        Ok(())
    }

    pub fn equipped_item(&self, slot: GearSlot) -> Option<&ItemInstance> {
        self.equipped_items.get(&slot)
    }

    pub fn equipped_skill_ids(&self) -> impl Iterator<Item = SkillId> + '_ {
        self.equipped_skills.iter().filter_map(|skill| *skill)
    }

    pub fn derived_stats(&self) -> Stats {
        self.equipped_items
            .values()
            .fold(self.base_stats, |stats, item| stats + item.stats + item.affix_stats())
    }
}
```

- **Step 8: Run tests and verify GREEN**

Run: `cargo test domain::hero --lib`

Expected: all hero tests pass.

- **Step 9: Run format and full tests**

Run: `cargo fmt --check && cargo test`

Expected: both commands exit 0.

- **Step 10: Commit**

```bash
git add src/lib.rs src/domain
git commit -m "Add hero build domain"
```

## Task 3: Deterministic Dungeon Generation

**Files:**

- Create: `src/domain/dungeon.rs`
- Modify: `src/domain/mod.rs`
- **Step 1: Export dungeon module**

Add to `src/domain/mod.rs`:

```rust
pub mod dungeon;
pub mod hero;
pub mod items;
pub mod skills;
pub mod stats;
```

- **Step 2: Write failing dungeon tests**

Add tests to `src/domain/dungeon.rs` before implementation:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generated_dungeon_is_repeatable_for_seed() {
        let first = generate_dungeon(25, 123);
        let second = generate_dungeon(25, 123);

        assert_eq!(first, second);
    }

    #[test]
    fn milestone_boss_is_at_final_depth() {
        let rooms = generate_dungeon(25, 123);

        assert_eq!(rooms.last().unwrap().depth, 25);
        assert_eq!(rooms.last().unwrap().kind, RoomKind::Boss);
    }

    #[test]
    fn depth_scaling_increases_enemy_strength() {
        let shallow = Encounter::monster_for_depth(1, false);
        let deep = Encounter::monster_for_depth(20, false);

        assert!(deep.enemy.max_health > shallow.enemy.max_health);
        assert!(deep.enemy.damage > shallow.enemy.damage);
    }
}
```

- **Step 3: Run tests and verify RED**

Run: `cargo test domain::dungeon --lib`

Expected: fails because dungeon APIs do not exist.

- **Step 4: Implement `src/domain/dungeon.rs`**

```rust
use crate::domain::stats::Stats;
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RoomKind {
    Monster,
    Elite,
    Treasure,
    Shrine,
    Boss,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Enemy {
    pub name: String,
    pub max_health: i32,
    pub damage: i32,
    pub armor: i32,
    pub attack_speed: f32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Encounter {
    pub enemy: Enemy,
}

impl Encounter {
    pub fn monster_for_depth(depth: u32, elite: bool) -> Self {
        let multiplier = if elite { 2 } else { 1 };
        Self {
            enemy: Enemy {
                name: if elite { "Elite Hollow".into() } else { "Hollow".into() },
                max_health: (24 + depth as i32 * 6) * multiplier,
                damage: (3 + depth as i32 / 2) * multiplier,
                armor: depth as i32 / 5,
                attack_speed: 1.0,
            },
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DungeonRoom {
    pub depth: u32,
    pub kind: RoomKind,
    pub encounter: Option<Encounter>,
}

pub fn generate_dungeon(depth_count: u32, seed: u64) -> Vec<DungeonRoom> {
    let mut rng = ChaCha8Rng::seed_from_u64(seed);
    (1..=depth_count)
        .map(|depth| {
            let kind = if depth == depth_count {
                RoomKind::Boss
            } else if depth % 10 == 0 {
                RoomKind::Elite
            } else {
                match rng.gen_range(0..10) {
                    0 => RoomKind::Treasure,
                    1 => RoomKind::Shrine,
                    _ => RoomKind::Monster,
                }
            };

            let encounter = match kind {
                RoomKind::Monster => Some(Encounter::monster_for_depth(depth, false)),
                RoomKind::Elite => Some(Encounter::monster_for_depth(depth, true)),
                RoomKind::Boss => Some(Encounter {
                    enemy: Enemy {
                        name: "Gate Warden".into(),
                        max_health: 250,
                        damage: 18,
                        armor: 4,
                        attack_speed: 0.8,
                    },
                }),
                RoomKind::Treasure | RoomKind::Shrine => None,
            };

            DungeonRoom { depth, kind, encounter }
        })
        .collect()
}

impl From<&Enemy> for Stats {
    fn from(enemy: &Enemy) -> Self {
        Self {
            max_health: enemy.max_health,
            damage: enemy.damage,
            armor: enemy.armor,
            attack_speed: enemy.attack_speed,
            healing_power: 0,
        }
    }
}
```

- **Step 5: Run tests and verify GREEN**

Run: `cargo test domain::dungeon --lib`

Expected: all dungeon tests pass.

- **Step 6: Run full verification**

Run: `cargo fmt --check && cargo test`

Expected: all checks pass.

- **Step 7: Commit**

```bash
git add src/domain/mod.rs src/domain/dungeon.rs
git commit -m "Add deterministic dungeon generation"
```

## Task 4: Fixed-Tick Combat Simulation

**Files:**

- Create: `src/domain/combat.rs`
- Modify: `src/domain/mod.rs`
- Modify: `src/domain/skills.rs`
- **Step 1: Export combat module**

Add `pub mod combat;` to `src/domain/mod.rs`.

- **Step 2: Write failing combat tests**

Add tests to `src/domain/combat.rs` before implementation:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::dungeon::Enemy;
    use crate::domain::hero::HeroProfile;
    use crate::domain::skills::SkillId;
    use crate::domain::stats::Stats;

    #[test]
    fn hero_defeats_weaker_enemy() {
        let mut hero = HeroProfile::new(Stats { damage: 12, ..Stats::default() });
        hero.unlock_skill_slots(1);

        let enemy = Enemy {
            name: "Training Hollow".into(),
            max_health: 20,
            damage: 1,
            armor: 0,
            attack_speed: 1.0,
        };

        let result = simulate_combat(&hero, &enemy, 100);

        assert_eq!(result.outcome, CombatOutcome::HeroWon);
        assert!(result.events.iter().any(|event| matches!(event, CombatEvent::EnemyDefeated)));
    }

    #[test]
    fn lifesteal_skill_restores_health_on_attack() {
        let mut hero = HeroProfile::default();
        hero.unlock_skill_slots(1);
        hero.equip_skill(0, SkillId::LifestealStrike).unwrap();

        let enemy = Enemy {
            name: "Durable Hollow".into(),
            max_health: 60,
            damage: 10,
            armor: 0,
            attack_speed: 1.0,
        };

        let result = simulate_combat(&hero, &enemy, 5);

        assert!(result.events.iter().any(|event| matches!(event, CombatEvent::HeroHealed { amount } if *amount > 0)));
    }

    #[test]
    fn armor_reduces_incoming_damage_but_damage_is_at_least_one() {
        let hero = HeroProfile::new(Stats { armor: 99, ..Stats::default() });
        let enemy = Enemy {
            name: "Weak Hollow".into(),
            max_health: 999,
            damage: 5,
            armor: 0,
            attack_speed: 1.0,
        };

        let result = simulate_combat(&hero, &enemy, 1);

        assert!(result.hero_health < hero.derived_stats().max_health);
    }
}
```

- **Step 3: Run tests and verify RED**

Run: `cargo test domain::combat --lib`

Expected: fails because combat APIs do not exist.

- **Step 4: Implement `src/domain/combat.rs`**

Implement a simple deterministic tick model:

```rust
use crate::domain::dungeon::Enemy;
use crate::domain::hero::HeroProfile;
use crate::domain::skills::SkillId;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CombatOutcome {
    HeroWon,
    EnemyWon,
    TimedOut,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CombatEvent {
    HeroAttacked { damage: i32 },
    EnemyAttacked { damage: i32 },
    HeroHealed { amount: i32 },
    EnemyDefeated,
    HeroDefeated,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CombatResult {
    pub outcome: CombatOutcome,
    pub hero_health: i32,
    pub enemy_health: i32,
    pub events: Vec<CombatEvent>,
}

pub fn simulate_combat(hero: &HeroProfile, enemy: &Enemy, max_ticks: u32) -> CombatResult {
    let stats = hero.derived_stats();
    let mut hero_health = stats.max_health;
    let mut enemy_health = enemy.max_health;
    let has_lifesteal = hero.equipped_skill_ids().any(|skill| skill == SkillId::LifestealStrike);
    let mut events = Vec::new();

    for _ in 0..max_ticks {
        let hero_damage = (stats.damage - enemy.armor).max(1);
        enemy_health -= hero_damage;
        events.push(CombatEvent::HeroAttacked { damage: hero_damage });

        if has_lifesteal {
            let amount = (hero_damage / 4).max(1);
            hero_health = (hero_health + amount).min(stats.max_health);
            events.push(CombatEvent::HeroHealed { amount });
        }

        if enemy_health <= 0 {
            events.push(CombatEvent::EnemyDefeated);
            return CombatResult { outcome: CombatOutcome::HeroWon, hero_health, enemy_health, events };
        }

        let enemy_damage = (enemy.damage - stats.armor).max(1);
        hero_health -= enemy_damage;
        events.push(CombatEvent::EnemyAttacked { damage: enemy_damage });

        if hero_health <= 0 {
            events.push(CombatEvent::HeroDefeated);
            return CombatResult { outcome: CombatOutcome::EnemyWon, hero_health, enemy_health, events };
        }
    }

    CombatResult { outcome: CombatOutcome::TimedOut, hero_health, enemy_health, events }
}
```

- **Step 5: Run tests and verify GREEN**

Run: `cargo test domain::combat --lib`

Expected: all combat tests pass.

- **Step 6: Run full verification**

Run: `cargo fmt --check && cargo test`

Expected: all checks pass.

- **Step 7: Commit**

```bash
git add src/domain/mod.rs src/domain/skills.rs src/domain/combat.rs
git commit -m "Add fixed tick combat simulation"
```

## Task 5: Loot, Inventory, And Salvage

**Files:**

- Create: `src/domain/loot.rs`
- Modify: `src/domain/mod.rs`
- Modify: `src/domain/items.rs`
- **Step 1: Export loot module**

Add `pub mod loot;` to `src/domain/mod.rs`.

- **Step 2: Write failing loot tests**

Add tests to `src/domain/loot.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::items::GearSlot;

    #[test]
    fn loot_rolls_are_repeatable_for_seed_and_depth() {
        let first = roll_loot(10, 99);
        let second = roll_loot(10, 99);

        assert_eq!(first, second);
    }

    #[test]
    fn deeper_loot_has_at_least_as_much_stat_budget() {
        let shallow = roll_loot(1, 42);
        let deep = roll_loot(20, 42);

        assert!(deep.stats.damage + deep.stats.armor + deep.stats.max_health >= shallow.stats.damage + shallow.stats.armor + shallow.stats.max_health);
    }

    #[test]
    fn salvage_value_scales_by_rarity() {
        let item = roll_loot(10, 123);

        assert!(salvage_value(&item) > 0);
    }

    #[test]
    fn rolled_items_use_mvp_gear_slots() {
        let item = roll_loot(3, 5);

        assert!(matches!(item.slot, GearSlot::Weapon | GearSlot::Armor | GearSlot::Trinket));
    }
}
```

- **Step 3: Run tests and verify RED**

Run: `cargo test domain::loot --lib`

Expected: fails because loot APIs do not exist.

- **Step 4: Implement `src/domain/loot.rs`**

```rust
use crate::domain::items::{GearSlot, ItemAffix, ItemInstance, ItemRarity};
use crate::domain::stats::Stats;
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;

pub fn roll_loot(depth: u32, seed: u64) -> ItemInstance {
    let mut rng = ChaCha8Rng::seed_from_u64(seed ^ depth as u64);
    let slot = match rng.gen_range(0..3) {
        0 => GearSlot::Weapon,
        1 => GearSlot::Armor,
        _ => GearSlot::Trinket,
    };
    let rarity = if depth >= 18 {
        ItemRarity::Rare
    } else if depth >= 8 {
        ItemRarity::Uncommon
    } else {
        ItemRarity::Common
    };
    let budget = depth as i32 + rarity_bonus(rarity);
    let affix = match rng.gen_range(0..5) {
        0 => ItemAffix::Vampiric,
        1 => ItemAffix::Heavy,
        2 => ItemAffix::Cursed,
        3 => ItemAffix::Spiked,
        _ => ItemAffix::Relentless,
    };

    ItemInstance {
        id: seed ^ ((depth as u64) << 32),
        name: format!("{rarity:?} {slot:?}"),
        slot,
        rarity,
        stats: match slot {
            GearSlot::Weapon => Stats { damage: budget, ..Stats::default_zero() },
            GearSlot::Armor => Stats { armor: budget / 2, max_health: budget * 4, ..Stats::default_zero() },
            GearSlot::Trinket => Stats { healing_power: budget / 2, attack_speed: 0.1, ..Stats::default_zero() },
        },
        affixes: vec![affix],
    }
}

pub fn salvage_value(item: &ItemInstance) -> u32 {
    match item.rarity {
        ItemRarity::Common => 5,
        ItemRarity::Uncommon => 15,
        ItemRarity::Rare => 40,
    }
}

fn rarity_bonus(rarity: ItemRarity) -> i32 {
    match rarity {
        ItemRarity::Common => 0,
        ItemRarity::Uncommon => 4,
        ItemRarity::Rare => 10,
    }
}
```

- **Step 5: Run tests and verify GREEN**

Run: `cargo test domain::loot --lib`

Expected: all loot tests pass.

- **Step 6: Run full verification**

Run: `cargo fmt --check && cargo test`

Expected: all checks pass.

- **Step 7: Commit**

```bash
git add src/domain/mod.rs src/domain/items.rs src/domain/loot.rs
git commit -m "Add deterministic loot generation"
```

## Task 6: Meta-Progression

**Files:**

- Create: `src/domain/progression.rs`
- Modify: `src/domain/mod.rs`
- **Step 1: Export progression module**

Add `pub mod progression;` to `src/domain/mod.rs`.

- **Step 2: Write failing progression tests**

Add tests to `src/domain/progression.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn upgrades_spend_gold_and_increase_level() {
        let mut profile = MetaProgression::default();
        profile.gold = 100;

        profile.buy_upgrade(UpgradeId::BaseDamage).unwrap();

        assert_eq!(profile.gold, 90);
        assert_eq!(profile.upgrade_level(UpgradeId::BaseDamage), 1);
    }

    #[test]
    fn cannot_buy_upgrade_without_enough_gold() {
        let mut profile = MetaProgression::default();

        let result = profile.buy_upgrade(UpgradeId::BaseDamage);

        assert_eq!(result, Err(ProgressionError::NotEnoughGold { required: 10, available: 0 }));
    }

    #[test]
    fn skill_slot_unlocks_after_progress_threshold() {
        let mut profile = MetaProgression::default();

        profile.add_skill_slot_progress(100);

        assert_eq!(profile.unlocked_skill_slots, 3);
    }
}
```

- **Step 3: Run tests and verify RED**

Run: `cargo test domain::progression --lib`

Expected: fails because progression APIs do not exist.

- **Step 4: Implement progression**

```rust
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum UpgradeId {
    MaxHealth,
    BaseDamage,
    Armor,
    HealingPower,
    GoldGain,
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ProgressionError {
    #[error("not enough gold: required {required}, available {available}")]
    NotEnoughGold { required: u32, available: u32 },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MetaProgression {
    pub gold: u32,
    pub salvage: u32,
    pub unlocked_skill_slots: usize,
    pub skill_slot_progress: u32,
    upgrades: HashMap<UpgradeId, u32>,
}

impl Default for MetaProgression {
    fn default() -> Self {
        Self {
            gold: 0,
            salvage: 0,
            unlocked_skill_slots: 2,
            skill_slot_progress: 0,
            upgrades: HashMap::new(),
        }
    }
}

impl MetaProgression {
    pub fn upgrade_level(&self, upgrade: UpgradeId) -> u32 {
        *self.upgrades.get(&upgrade).unwrap_or(&0)
    }

    pub fn buy_upgrade(&mut self, upgrade: UpgradeId) -> Result<(), ProgressionError> {
        let cost = self.upgrade_cost(upgrade);
        if self.gold < cost {
            return Err(ProgressionError::NotEnoughGold { required: cost, available: self.gold });
        }
        self.gold -= cost;
        *self.upgrades.entry(upgrade).or_insert(0) += 1;
        Ok(())
    }

    pub fn add_skill_slot_progress(&mut self, amount: u32) {
        self.skill_slot_progress += amount;
        if self.skill_slot_progress >= 100 {
            self.unlocked_skill_slots = self.unlocked_skill_slots.max(3);
        }
    }

    fn upgrade_cost(&self, upgrade: UpgradeId) -> u32 {
        10 + self.upgrade_level(upgrade) * 5
    }
}
```

- **Step 5: Run tests and verify GREEN**

Run: `cargo test domain::progression --lib`

Expected: all progression tests pass.

- **Step 6: Run full verification**

Run: `cargo fmt --check && cargo test`

Expected: all checks pass.

- **Step 7: Commit**

```bash
git add src/domain/mod.rs src/domain/progression.rs
git commit -m "Add meta progression"
```

## Task 7: Complete Run State Machine

**Files:**

- Create: `src/domain/run.rs`
- Create: `tests/simulation_mvp.rs`
- Modify: `src/domain/mod.rs`
- **Step 1: Export run module**

Add `pub mod run;` to `src/domain/mod.rs`.

- **Step 2: Write failing integration test**

Create `tests/simulation_mvp.rs`:

```rust
use idle_dungeons::domain::hero::HeroProfile;
use idle_dungeons::domain::run::{RunConfig, RunOutcome, simulate_run};

#[test]
fn seeded_run_reaches_same_result_every_time() {
    let hero = HeroProfile::default();
    let config = RunConfig { seed: 7, max_depth: 25 };

    let first = simulate_run(&hero, config);
    let second = simulate_run(&hero, config);

    assert_eq!(first, second);
}

#[test]
fn run_summary_reports_depth_gold_and_outcome() {
    let hero = HeroProfile::default();
    let result = simulate_run(&hero, RunConfig { seed: 5, max_depth: 25 });

    assert!(result.deepest_depth >= 1);
    assert!(result.gold_earned > 0);
    assert!(matches!(result.outcome, RunOutcome::HeroDied | RunOutcome::BossDefeated));
}
```

- **Step 3: Run tests and verify RED**

Run: `cargo test --test simulation_mvp`

Expected: fails because run APIs do not exist.

- **Step 4: Implement `src/domain/run.rs`**

```rust
use crate::domain::combat::{simulate_combat, CombatOutcome};
use crate::domain::dungeon::{generate_dungeon, RoomKind};
use crate::domain::hero::HeroProfile;
use crate::domain::loot::{roll_loot, salvage_value};
use crate::domain::items::ItemInstance;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RunConfig {
    pub seed: u64,
    pub max_depth: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RunOutcome {
    HeroDied,
    BossDefeated,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RunSummary {
    pub outcome: RunOutcome,
    pub deepest_depth: u32,
    pub gold_earned: u32,
    pub salvage_earned: u32,
    pub loot: Vec<ItemInstance>,
    pub death_reason: Option<String>,
}

pub fn simulate_run(hero: &HeroProfile, config: RunConfig) -> RunSummary {
    let rooms = generate_dungeon(config.max_depth, config.seed);
    let mut deepest_depth = 0;
    let mut gold_earned = 0;
    let mut salvage_earned = 0;
    let mut loot = Vec::new();

    for room in rooms {
        deepest_depth = room.depth;
        match room.kind {
            RoomKind::Monster | RoomKind::Elite | RoomKind::Boss => {
                let enemy = room.encounter.as_ref().unwrap().enemy.clone();
                let combat = simulate_combat(hero, &enemy, 100);
                if combat.outcome == CombatOutcome::EnemyWon {
                    return RunSummary {
                        outcome: RunOutcome::HeroDied,
                        deepest_depth,
                        gold_earned,
                        salvage_earned,
                        loot,
                        death_reason: Some(format!("Defeated by {}", enemy.name)),
                    };
                }
                gold_earned += room.depth * 3;
                if room.kind == RoomKind::Boss {
                    return RunSummary {
                        outcome: RunOutcome::BossDefeated,
                        deepest_depth,
                        gold_earned,
                        salvage_earned,
                        loot,
                        death_reason: None,
                    };
                }
            }
            RoomKind::Treasure => {
                let item = roll_loot(room.depth, config.seed);
                salvage_earned += salvage_value(&item);
                loot.push(item);
                gold_earned += room.depth * 2;
            }
            RoomKind::Shrine => {
                gold_earned += room.depth;
            }
        }
    }

    RunSummary {
        outcome: RunOutcome::BossDefeated,
        deepest_depth,
        gold_earned,
        salvage_earned,
        loot,
        death_reason: None,
    }
}
```

- **Step 5: Run tests and verify GREEN**

Run: `cargo test --test simulation_mvp`

Expected: integration tests pass.

- **Step 6: Run full verification**

Run: `cargo fmt --check && cargo test`

Expected: all checks pass.

- **Step 7: Commit**

```bash
git add src/domain/mod.rs src/domain/run.rs tests/simulation_mvp.rs
git commit -m "Add deterministic run simulation"
```

## Task 8: Save And Load Profiles

**Files:**

- Create: `src/save.rs`
- Modify: `src/lib.rs`
- **Step 1: Export save module**

Add `pub mod save;` to `src/lib.rs`.

- **Step 2: Write failing save tests**

Add tests to `src/save.rs`:

```rust
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
    fn corrupt_save_returns_error() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("profile.json");
        std::fs::write(&path, "{not-json").unwrap();

        let result = load_profile(&path);

        assert!(matches!(result, Err(SaveError::Parse(_))));
    }
}
```

- **Step 3: Run tests and verify RED**

Run: `cargo test save --lib`

Expected: fails because save APIs do not exist.

- **Step 4: Implement `src/save.rs`**

```rust
use crate::domain::hero::HeroProfile;
use crate::domain::items::ItemInstance;
use crate::domain::progression::MetaProgression;
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SaveProfile {
    pub hero: HeroProfile,
    pub inventory: Vec<ItemInstance>,
    pub meta: MetaProgression,
}

impl Default for SaveProfile {
    fn default() -> Self {
        Self {
            hero: HeroProfile::default(),
            inventory: Vec::new(),
            meta: MetaProgression::default(),
        }
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
```

- **Step 5: Run tests and verify GREEN**

Run: `cargo test save --lib`

Expected: all save tests pass.

- **Step 6: Run full verification**

Run: `cargo fmt --check && cargo test`

Expected: all checks pass.

- **Step 7: Commit**

```bash
git add src/lib.rs src/save.rs
git commit -m "Add profile save loading"
```

## Task 9: Bevy App States And Simulation Resource

**Files:**

- Modify: `src/app.rs`
- **Step 1: Write failing app state tests**

Add tests to `src/app.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

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
}
```

- **Step 2: Run tests and verify RED**

Run: `cargo test app --lib`

Expected: fails because `GameState`, `StartRun`, and `LatestRunSummary` do not exist.

- **Step 3: Implement app resources and systems**

Update `src/app.rs`:

```rust
use crate::domain::hero::HeroProfile;
use crate::domain::run::{RunConfig, RunSummary, simulate_run};
use bevy::prelude::*;

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
pub struct LatestRunSummary(pub RunSummary);

pub struct IdleDungeonsPlugin;

impl Plugin for IdleDungeonsPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<GameState>()
            .add_event::<StartRun>()
            .add_systems(Update, start_run);
    }
}

fn start_run(mut commands: Commands, mut events: EventReader<StartRun>, mut next_state: ResMut<NextState<GameState>>) {
    for event in events.read() {
        let hero = HeroProfile::default();
        let summary = simulate_run(&hero, RunConfig { seed: event.seed, max_depth: 25 });
        commands.insert_resource(LatestRunSummary(summary));
        next_state.set(GameState::Summary);
    }
}

pub fn run() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(IdleDungeonsPlugin)
        .run();
}
```

- **Step 4: Run tests and verify GREEN**

Run: `cargo test app --lib`

Expected: app state tests pass.

- **Step 5: Run full verification**

Run: `cargo fmt --check && cargo test`

Expected: all checks pass.

- **Step 6: Commit**

```bash
git add src/app.rs
git commit -m "Wire run simulation into Bevy app"
```

## Task 10: Functional MVP UI

**Files:**

- Create: `src/ui/mod.rs`
- Create: `src/ui/build_panel.rs`
- Create: `src/ui/run_panel.rs`
- Create: `src/ui/log_panel.rs`
- Create: `src/ui/inventory_panel.rs`
- Create: `src/ui/upgrade_panel.rs`
- Create: `src/ui/summary_panel.rs`
- Modify: `src/lib.rs`
- Modify: `src/app.rs`
- **Step 1: Export UI module**

Add `pub mod ui;` to `src/lib.rs`.

- **Step 2: Write failing UI text model tests**

Use testable text model functions before building visual Bevy nodes. Add to `src/ui/build_panel.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::hero::HeroProfile;

    #[test]
    fn build_panel_text_lists_skill_slots_and_stats() {
        let hero = HeroProfile::default();

        let text = build_panel_text(&hero);

        assert!(text.contains("Skill Slots"));
        assert!(text.contains("Max Health"));
    }
}
```

Add to `src/ui/summary_panel.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::run::{RunOutcome, RunSummary};

    #[test]
    fn summary_text_reports_depth_gold_and_outcome() {
        let summary = RunSummary {
            outcome: RunOutcome::HeroDied,
            deepest_depth: 8,
            gold_earned: 30,
            salvage_earned: 5,
            loot: Vec::new(),
            death_reason: Some("Defeated by Hollow".into()),
        };

        let text = summary_panel_text(&summary);

        assert!(text.contains("Depth 8"));
        assert!(text.contains("Gold 30"));
        assert!(text.contains("Defeated by Hollow"));
    }
}
```

- **Step 3: Run tests and verify RED**

Run: `cargo test ui --lib`

Expected: fails because UI modules and text functions do not exist.

- **Step 4: Implement UI text model functions**

`src/ui/mod.rs`:

```rust
pub mod build_panel;
pub mod inventory_panel;
pub mod log_panel;
pub mod run_panel;
pub mod summary_panel;
pub mod upgrade_panel;

use bevy::prelude::*;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, _app: &mut App) {}
}
```

`src/ui/build_panel.rs`:

```rust
use crate::domain::hero::HeroProfile;

pub fn build_panel_text(hero: &HeroProfile) -> String {
    let stats = hero.derived_stats();
    format!(
        "Skill Slots: {}/{}\nMax Health: {}\nDamage: {}\nArmor: {}",
        hero.unlocked_skill_slots,
        hero.equipped_skills.len(),
        stats.max_health,
        stats.damage,
        stats.armor
    )
}
```

`src/ui/summary_panel.rs`:

```rust
use crate::domain::run::RunSummary;

pub fn summary_panel_text(summary: &RunSummary) -> String {
    format!(
        "Outcome: {:?}\nDepth {}\nGold {}\nSalvage {}\n{}",
        summary.outcome,
        summary.deepest_depth,
        summary.gold_earned,
        summary.salvage_earned,
        summary.death_reason.clone().unwrap_or_else(|| "Victory".to_string())
    )
}
```

Create the remaining panel modules with focused title functions that compile:

```rust
pub fn panel_title() -> &'static str {
    "Panel"
}
```

Use that function in `inventory_panel.rs`, `log_panel.rs`, `run_panel.rs`, and `upgrade_panel.rs`, changing the returned string to the panel name.

- **Step 5: Wire `UiPlugin` into app**

In `src/app.rs`, import and add the plugin:

```rust
use crate::ui::UiPlugin;

// inside run()
.add_plugins(UiPlugin)
```

- **Step 6: Run tests and verify GREEN**

Run: `cargo test ui --lib`

Expected: UI text model tests pass.

- **Step 7: Run full verification**

Run: `cargo fmt --check && cargo test && cargo check`

Expected: all checks pass.

- **Step 8: Commit**

```bash
git add src/lib.rs src/app.rs src/ui
git commit -m "Add functional MVP UI models"
```

## Task 11: MVP Acceptance Pass

**Files:**

- Modify: `README.md`
- Create: `docs/mvp-acceptance.md`
- **Step 1: Write acceptance checklist**

Create `docs/mvp-acceptance.md`:

```markdown
# MVP Acceptance Checklist

- [ ] `cargo fmt --check` passes.
- [ ] `cargo test` passes.
- [ ] `cargo check` passes.
- [ ] A seeded run returns the same summary on repeated executions.
- [ ] Hero builds derive stats from base stats, skills, gear, and affixes.
- [ ] Locked skill slots reject equipped skills.
- [ ] Gear only equips into matching gear slots.
- [ ] Dungeon generation places the milestone boss at depth 25.
- [ ] Combat resolves to hero victory, hero death, or timeout.
- [ ] Loot rolls are deterministic under seed and depth.
- [ ] Permanent upgrades spend gold and update levels.
- [ ] Save/load round trips preserve profile data.
- [ ] UI text models expose build and summary information.
```

- **Step 2: Add README development commands**

Append to `README.md`:

```markdown
## Development

Common commands:

```sh
cargo fmt --check
cargo test
cargo check
```

The MVP should be built one testable slice at a time. Each behavior change should start with a failing test, then minimal implementation, then a passing verification run.

```

- [ ] **Step 3: Run full MVP verification**

Run: `cargo fmt --check && cargo test && cargo check`

Expected: all commands exit 0.

- [ ] **Step 4: Commit**

```bash
git add README.md docs/mvp-acceptance.md
git commit -m "Document MVP acceptance checks"
```

## Execution Rules

- Every production-code task starts with a failing test.
- If a test passes before implementation, rewrite the test because it did not prove missing behavior.
- After each task, run the targeted test, then `cargo fmt --check && cargo test`.
- Commit after each task while the working tree is clean.
- Do not add UI polish before the simulation behavior is tested.
- Do not add party systems, extra heroes, or complex map generation during this MVP plan.

## Self-Review

- Spec coverage: This plan covers project bootstrap, plugin architecture, hero skill slots, gear/stat synergy, deterministic dungeon generation, fixed-tick combat, loot, meta-progression, save/load, UI panels, and MVP acceptance checks.
- Intentional gaps: Full party formation, multiple heroes, advanced procedural maps, online systems, external content tooling, and asset-heavy visuals remain out of scope, matching the design document.
- Test strategy: Every behavior subsystem has a red-green test gate and a full verification gate before commit.

