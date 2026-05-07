# Idle Dungeons

Idle Dungeons is a Bevy-based roguelike incremental game about building an automated dungeon delver. The player configures a hero, starts a mostly idle run, watches the hero push through a dark fantasy dungeon, then spends the results on better gear, skills, and permanent upgrades.

The project ships a playable MVP loop: configure skills on the briefing and camp screens, run the delve, collect rewards, manage stash, and buy upgrades—with progress saved to `saves/profile.json`.

## Game Concept

The hero is not defined by a fixed class. Instead, their role emerges from unlocked skill slots, equipped skills, gear stats, and item affixes. A fast weapon might empower lifesteal or poison builds, while cursed armor might strengthen barrier builds but weaken sustain-heavy ones.

Over time, the game should grow from one configurable hero into a full party-building incremental roguelike. More skill slots, gear slots, heroes, dungeon biomes, bosses, and reset layers can be added after the MVP loop is working.

## MVP Scope

The first playable milestone should include:

- One automated hero.
- Two starting skill slots.
- A small pool of skills.
- Weapon, armor, and trinket gear slots.
- A linear dungeon with normal encounters, elites, treasure, shrines, and a milestone boss.
- Automated fixed-tick combat.
- Loot, salvage, gold, permanent upgrades, and skill slot progression.
- Local save/load.
- Functional Bevy UI for build management, runs, inventory, upgrades, combat log, and run summaries.

Out of scope for the MVP are multiple heroes, full party formation, complex procedural maps, active tactical combat, online systems, and asset-heavy visuals.

## Technical Direction

The game should be organized around focused Bevy plugins:

- `GameStatePlugin` for high-level app states.
- `RunSimulationPlugin` for dungeon progression, combat ticks, deaths, and rewards.
- `HeroPlugin` for base stats, skills, gear, and derived stats.
- `SkillPlugin` for triggers, tags, scaling, and slot validation.
- `DungeonPlugin` for room generation and encounter scaling.
- `LootPlugin` for item rolls, rarity, affixes, equipment, and salvage.
- `MetaProgressionPlugin` for currencies, upgrades, and unlocks.
- `UiPlugin` for readable run, build, inventory, upgrade, and summary screens.
- `SavePlugin` for persistent local profile data.

The run simulation should be the source of truth. UI should observe simulation state and events rather than drive combat directly, which keeps the game easier to test and leaves room for fast-forward, offline progress, and deterministic seeded runs later.

## Design Documentation

The full design document lives at:

`docs/superpowers/specs/2026-05-06-roguelike-incremental-bevy-design.md`

## Development

Common commands:

```sh
cargo fmt --check
cargo test
cargo check
cargo run
```

The MVP should be built one testable slice at a time. Each behavior change should start with a failing test, then minimal implementation, then a passing verification run.

## Current Playable Loop

The MVP opens to the briefing screen. Assign skills by clicking unlocked slots—they cycle through each skill and an empty slot—then start a run. Skip or watch playback, accept rewards on the summary, and at camp equip or salvage loot and buy upgrades. The **Gold Gain** permanent upgrade increases gold earned from delves. Use **Return to briefing** to change skills again. Progress is saved locally to `saves/profile.json`.
