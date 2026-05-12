# Roguelike Incremental Bevy Game Design

## Overview

This document defines the MVP design for a roguelike incremental game built with Bevy. The game fantasy is an automated dungeon delver: the player configures a hero, starts a mostly idle run, watches the hero push deeper through a dark fantasy dungeon, then uses the outcome to improve future attempts.

The MVP should prove the core loop with a small vertical slice rather than broad content. It should include one hero, a short dungeon track, automated combat, loot, skill slots, gear synergy, meta-upgrades, and local persistence.

Long term, the game should grow from one configurable hero into a party-building incremental roguelike. Hero identity should emerge from unlocked skill slots, equipped skills, and gear interactions rather than fixed classes.

For the **strategic roadmap** toward buildcraft, party roles, threat, itemization, and telemetry (simulation-first), see the archived snapshot [`obsolete-2026-05-06-buildcraft-party-systems-roadmap.md`](./obsolete-2026-05-06-buildcraft-party-systems-roadmap.md). For current execution tasks, see [`../ACTIVE-REMAINING-WORK.md`](../ACTIVE-REMAINING-WORK.md).

## Design Goals

- Make the automated run readable and satisfying without requiring direct player control.
- Let player decisions happen mostly before and after runs through build choices, gear, and permanent upgrades.
- Keep the MVP small enough to tune while leaving clear paths to more heroes, party formation, more item slots, and deeper skill systems.
- Separate simulation logic from presentation so runs can later support fast-forward, offline progress, deterministic tests, and alternative UI views.
- Build systems in a Bevy-friendly way using plugins, app states, resources, events, and ECS components with clear ownership.

## Core Loop

1. Configure the hero using unlocked skill slots, equipped skills, gear, and permanent upgrades.
2. Start an automated dungeon run.
3. Watch the hero advance room by room, fight monsters, collect resources, and acquire loot.
4. End the run when the hero dies or defeats the milestone boss.
5. Review a run summary showing depth reached, gold earned, loot found, and the main cause of failure or success.
6. Spend resources on permanent upgrades, skill slot progress, and gear decisions.
7. Start another run with a stronger or more specialized build.

The first milestone boss should sit at a fixed depth, such as room 25. Defeating that boss is the MVP's main success condition.

## Bevy Architecture

The game should be organized around small plugins with explicit responsibilities:

- `GameStatePlugin`: owns high-level app states such as loading, main menu, run, summary, upgrades, and build management.
- `RunSimulationPlugin`: advances dungeon rooms, combat ticks, run timers, rewards, deaths, and win conditions.
- `HeroPlugin`: owns hero base stats, unlocked skill slots, equipped skills, gear loadout, and derived stat calculation.
- `SkillPlugin`: defines skill behavior, triggers, tags, scaling, and slot validation.
- `DungeonPlugin`: creates the room sequence, encounter types, depth scaling, elites, treasure rooms, shrines, and boss checkpoints.
- `LootPlugin`: rolls item bases, rarity, affixes, inventory entries, salvage rewards, and equipment validation.
- `MetaProgressionPlugin`: owns permanent upgrades, currencies, unlock conditions, and post-run spending.
- `UiPlugin`: displays run state, build state, combat log, inventory, upgrades, and summaries.
- `SavePlugin`: loads and saves persistent profile data.

The run simulation should be the source of truth. UI systems should read simulation resources and events, not drive combat or progression directly.

## MVP Hero Model

The MVP should not use fixed classes. The player starts with a blank hero chassis whose role emerges from skills and gear.

Initial scope:

- One hero.
- Two unlocked skill slots.
- A small skill pool of about six skills.
- Three gear slots: weapon, armor, and trinket.
- A small set of affixes that visibly interact with the skill pool.
- Meta-upgrades that improve base stats and eventually unlock additional skill slot capacity.

Future expansion should add more item slots, more skill slots, more hero slots, and eventually a full party. At that point, party roles should still be emergent. A hero with taunt, armor scaling, and thorns gear behaves like a tank. A hero with bleed skills, attack speed, and execute effects behaves like a damage dealer. A hero with barriers, healing, and aura effects behaves like support.

## Skills And Gear

Skills define what the hero is trying to do. Gear changes how well that plan works.

Each skill should have:

- A slot type or compatibility rule if needed.
- A trigger, such as on attack, on hit taken, on kill, on room start, on low health, or periodic tick.
- Stat scaling.
- Tags such as `attack`, `defense`, `healing`, `bleed`, `poison`, `barrier`, `minion`, `thorns`, or `execute`.
- A short player-facing description.

Each gear item should have:

- A gear slot.
- Base stats.
- Rarity.
- Optional affixes.
- Tags or modifiers that can interact with skill tags.

Gear should sometimes empower a build and sometimes undermine it. For example:

- A fast weapon is strong with lifesteal, poison, or on-hit effects, but weak with slow heavy-strike skills.
- Cursed armor may grant high mitigation while reducing healing received, making it good for barrier builds and dangerous for sustain builds.
- A trinket that improves bleed duration is valuable only when the current skills can apply bleed.
- Heavy armor that reduces attack speed may protect the hero but weaken builds that depend on frequent hits.

This relationship should be central to the long-term game. The player should learn to evaluate gear by comparing its stats and traits against the equipped skills, not just by checking whether the item has a higher number.

## Dungeon And Runs

The MVP dungeon is a linear sequence of rooms by depth. Each room contains one encounter:

- Normal monster.
- Elite monster.
- Treasure cache.
- Shrine or temporary modifier.
- Boss checkpoint.

Combat should run on fixed simulation ticks. The hero and enemy act according to derived stats such as attack speed, damage, armor, hit chance, crit chance, healing, barriers, and triggered skill effects. The player does not directly control combat during the MVP run.

Depth scaling should increase monster stats and rewards. The dungeon should be deterministic enough under a seeded RNG to support repeatable tests and tuning.

## Progression

The MVP should include two progression layers:

- Run progression: room depth, temporary loot, current health, current encounter, and run-specific rewards.
- Meta-progression: permanent upgrades, persistent gear collection, unlocked skill slots, and profile-level currencies.

Permanent upgrades should start simple:

- Maximum health.
- Base damage.
- Armor.
- Recovery or healing effectiveness.
- Gold gain.
- Starting resources or early-run advantage.
- Skill slot unlock progress.

Skill slot unlocks should be treated as major milestones because they change how hero identity is built. Early runs may only support two skill slots, while later progression expands build complexity by unlocking more slots.

## User Interface

The MVP UI should be functional and information-rich rather than visually complex.

Required screens and panels:

- Build panel: equipped skills, locked and unlocked skill slots, gear slots, and derived stats.
- Run panel: current depth, room type, hero health, enemy health, combat speed, and current rewards.
- Combat log: major events such as skill triggers, loot drops, deaths, elite kills, and boss outcomes.
- Inventory panel: loot found, equipped gear, comparison details, and salvage actions.
- Upgrade panel: permanent upgrades, costs, available currencies, and skill slot unlock progress.
- Summary panel: run duration, deepest room, gold earned, loot found, and cause of death or victory.

The UI should make gear-skill relationships visible. If an item modifies a tag that the current build uses, the comparison should make that interaction clear.

## Data Flow

Core data should include:

- Hero base stats.
- Permanent upgrade levels.
- Unlocked skill slot count.
- Equipped skill IDs.
- Equipped gear item IDs.
- Gear inventory.
- Current dungeon depth.
- Current encounter.
- Run resources and temporary state.
- Persistent profile resources.
- Run history summary.

Derived stats should be recalculated from base stats, upgrades, skills, gear, and affixes. Derived values should not be stored as authoritative state. This keeps build interactions easier to reason about and reduces save compatibility risk.

Skill and item definitions can start as Rust constants for the MVP, but they should be shaped like data records. That will make it easier to move them to asset files later if content grows.

## Error Handling And Constraints

Invalid builds should be impossible where practical:

- Locked skill slots cannot hold skills.
- Gear can only be equipped in matching gear slots.
- Duplicate skill rules should be explicit.
- Missing item or skill definitions should fail loudly in development.
- Save loading should create a fresh profile only when no save exists or the save is clearly unreadable.

Debug-visible simulation logs should help diagnose balance and rules issues. For example, a death summary should identify the final room, enemy, damage source, and any major skill or gear modifiers involved.

## Testing Strategy

Testing should focus on simulation correctness before UI polish.

Priority tests:

- Derived stat calculation from base stats, upgrades, skills, gear, and affixes.
- Skill slot validation and unlock rules.
- Skill trigger behavior.
- Combat tick outcomes.
- Loot roll boundaries and rarity weighting.
- Gear equip and salvage rules.
- Seeded run determinism.
- Save/load round trips for persistent profile data.

UI testing can stay lighter during the MVP, but the data powering UI panels should be covered by simulation and state tests.

## MVP Scope

In scope:

- One automated hero.
- Two starting skill slots.
- Around six skills.
- Weapon, armor, and trinket slots.
- A single linear dungeon with a fixed milestone boss.
- Normal monsters, elites, treasure, shrines, and one boss.
- Gold, loot, salvage, permanent upgrades, and skill slot progression.
- Local save/load.
- Functional Bevy UI for build, run, inventory, upgrades, log, and summary.

Out of scope for MVP:

- Multiple heroes.
- Full party formation.
- Complex procedural maps.
- Active tactical combat input.
- Online systems.
- Deep content volume.
- Asset-heavy visuals.
- External content editors.

## Long-Term Expansion

After the MVP proves the loop, the design should expand in this order:

1. More skills and gear affixes that deepen skill-tag synergy.
2. More gear slots and item types.
3. Additional skill slot unlocks and build complexity.
4. Multiple hero slots.
5. Party formation, roles, and inter-hero synergies.
6. More dungeon biomes, bosses, and reset layers.
7. Offline progress and faster simulation modes.
8. Data-driven content files or internal tooling if content volume justifies it.

The long-term game should keep the same principle as the MVP: roles are not selected from a class list. Roles emerge from skill slots, gear, stats, and party composition.
