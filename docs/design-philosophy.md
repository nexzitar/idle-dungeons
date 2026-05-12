# Delvers — design philosophy

This document captures how **Delvers** is meant to feel and how technical choices support that goal. It is descriptive of the current codebase and direction, not a contract for every future feature.

## What kind of game this is

Delvers sits in the overlap of **idle / incremental** games and **roguelike runs**: you prepare a build, send the hero (eventually a party) into a **seeded delve**, and watch **automated combat** resolve. You do not steer every swing in real time. The skill is in **loadout, economy, and risk choices**—what you bring, what you buy, what you keep, and when you push deeper.

That split is intentional: runs are **short, legible episodes** in a **longer persistence loop**. Gold, stash gear, unlocked skills, and permanent upgrades accumulate across delves so each run is both a self-contained story (win, lose, or time out) and a step in a broader arc.

## Simulation-first architecture

The **domain layer** (`src/domain/`) owns the truth: hero stats, dungeon generation, encounter resolution, loot rolls, and run summarization. The Bevy app (`src/app.rs`, `src/ui/`) interprets player intent as **events**, updates **`ProfileState`**, and reflects outcomes on screen.

**Why it matters**

- **Fairness and debugging:** The same inputs should yield the same results. Seeded RNG (e.g. ChaCha8) is used deliberately so runs, rooms, and loot are **reproducible** for a given seed and depth—essential for tests and for player trust.
- **Iteration speed:** Game designers (and the codebase) can change rules in Rust without fighting presentation details. Automated tests guard combat, loot, saves, and progression invariants.
- **Future flexibility:** A richer visual layer or alternate front ends can sit on top without rewriting combat math.

The UI’s job is to **surface** simulation outputs—playback, summaries, risk hints—not to secretly alter them.

## Buildcraft over micromanagement

Progression is expressed through **build decisions**:

- **Gear** — Three slots (weapon, armor, trinket), stat budgets tied to depth and rarity, and **affixes** that change how skills and defenses behave in combat.
- **Skills** — A growing catalog; **starter** skills are always available, while others unlock via the **skill shop** (gold sink) and are assigned in the **skill book** before a run. Slots unlock through meta milestones so the problem space widens over time rather than overwhelming new players.
- **Party (emerging)** — Second-hero ideas and party targeting already appear in domain code; the long-term vision is coordinated roles, not a single blob of stats.

The fantasy is **“I tuned this hero to survive that floor”**, not **“I clicked faster than the spawn timer.”**

## Runs, risk, and readability

A delve is a **linear depth ladder** with **varied room types** (monster, elite, treasure, shrine, boss). Seeds bias certain moments (e.g. deterministic “lucky vein” patterns) while keeping most room rolls understandable: monsters scale with depth, elites and bosses punch harder, and summaries report **peak risk** so players learn what went wrong without reading a combat log.

Combat itself ticks on a **clock** with discrete turns/pulses—clear enough to test, terse enough to **skip playback** and still get a fair summary. Skills are meant to have **readable identities** (guard, poison, thorns, barrier, lifesteal, etc.) that interact with gear affixes in ways players can anticipate after a few runs.

## Itemization and long-tail excitement

Loot uses **depth-scaled stat budgets** plus **rarity**. Rarity is intentionally **probabilistic** for most of the climb: very good rolls are possible early (rare) but not **guaranteed** by depth alone; only at extreme depth does the top tier become a **certainty** in the current design. That preserves **hope** on shallow delves without making every deep run feel identical, while still tying power to how far you push.

**Salvage** converts unwanted items into econ; **stash** and **sort** options keep inventory manageable as the idle loop produces volume.

## Meta progression and saves

Gold flows from runs into **skill purchases**, **permanent upgrades**, and preparation. Progress lives in a **local JSON save** (`SaveProfile`): hero, inventory, meta (gold, unlocks, upgrade levels). Migrations are handled carefully so early adopters do not lose profiles when new fields appear.

The philosophy is **player-owned progression** (single-player, offline-friendly) with room to add cloud or prestige layers later once the core loop feels complete.

## UX and presentation

The current UI is a deliberate **structured shell**: columns for hero, delve/summary, and stash; modals for gear and skills; **tooltips** for clarity; and **release-on-click** handling tuned so purchases and assignments feel reliable on real hardware.

Visual polish (animation, art pass) is expected to trail **mechanical depth**—the product should remain playable and testable at every step, with presentation catching up rather than the reverse.

## Testing and change discipline

**Domain tests** encode design contracts: repeatable dungeons, deterministic loot for seed+depth, combat edge cases (armor, barriers, poison ticks, thorns, etc.), save round-trips. When behavior changes, tests and **CHANGELOG** entries should move together so the game’s “physics” stay legible to contributors.

## Where this can grow

**Active backlog:** `docs/superpowers/ACTIVE-REMAINING-WORK.md` (what is left to implement). Archived strategic specs live under `docs/superpowers/specs/obsolete-*`. This philosophy doc should stay short enough to read once; superpowers docs drill into specifics.

In one line: **Delvers rewards preparation and systems knowledge in a fair, reproducible simulation, with an incremental meta loop that makes every delve matter.**
