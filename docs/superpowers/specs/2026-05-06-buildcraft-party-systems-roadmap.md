# Idle Dungeons — Buildcraft & Party Systems Roadmap

**Goal:** Expand Idle Dungeons from a single-hero stat-driven MVP into a deeper buildcraft-focused automated RPG with meaningful roles, party composition, skill identity, threat systems, and build-defining equipment.

This roadmap intentionally moves away from:

- infinite raw-stat scaling
- generic stat-stick loot
- linear incremental progression

and toward:

- build identity
- role specialization
- item-driven gameplay
- party synergy
- tactical automation
- replayable buildcraft

The **simulation-first architecture remains authoritative**.

All gameplay systems must continue to flow through:

- deterministic simulation
- event-driven combat
- replay-safe logic
- UI observing simulation state

**Do not compromise the simulation-as-source-of-truth architecture.**

---

## Core Design Direction

The long-term gameplay loop should focus on:

- creating builds
- assembling synergies
- solving encounters through automation logic
- defining party roles
- experimenting with skill/item combinations

The player should feel like:

- a strategist
- a party architect
- a buildcrafter

**Not:**

- a player infinitely buying raw stats

---

## Phase 1 — Skill System Redesign

### Goals

Replace the temporary “click slot to rotate skill” interaction with a proper skill selection system.

Skills should feel:

- collectible
- unlockable
- build-defining
- readable
- expandable through future patches

### Skill Structure

**Maximum equipped skill slots:** 6 total.

**Planned launch skill count:** approximately 20 skills.

Future patches should expand skill count, archetypes, tags, and interactions **without** increasing total equipped slots significantly. Scarcity of slots is important for build identity.

### Active vs Passive Skills

Split skills into two categories.

#### Active Skills

Skills that directly participate in combat behavior.

**Examples:** Heavy Strike, Poison Edge, Cleave, Guard, Barrier Pulse, Taunt.

These affect:

- attack behavior
- cooldowns
- targeting
- triggers
- combat pacing

#### Passive Skills

Always-on modifiers or conditional effects.

**Examples:** Toxic Mastery, Vampiric Aura, Thick Hide, Threat Generation, Arcane Overflow, Berserker.

These affect:

- scaling
- synergies
- archetypes
- role specialization

Passive skills should heavily influence build identity.

### Skill Book UI

Implement a proper skill selection interface.

Clicking a skill slot should:

- open a skillbook/modal/panel
- show available skills
- allow selecting a skill into the slot

Each skill entry should display:

- name
- type (active/passive)
- tags
- short description
- synergy hints

**Optional:** rarity, unlock source, mastery progression.

### Skill Tags

Introduce gameplay tags.

**Examples:** Poison, Barrier, Melee, Ranged, AoE, Threat, Healing, Summon, Reactive, Curse, Critical, Sustain.

Tags are critical for:

- future scaling
- synergy systems
- item interactions
- passive effects
- party composition

Avoid hardcoding interactions where possible. Prefer:

- tag-driven logic
- event-driven triggers
- extensible effect pipelines

---

## Phase 2 — Itemization Redesign

### Goals

Items should become:

- build-defining
- role-defining
- mechanically interesting

**Not merely:** raw stat sticks.

Common/trash gear may remain relatively simple. Higher rarity items should:

- meaningfully alter gameplay
- change combat behavior
- enable archetypes
- introduce synergies

### Equipment Philosophy

**Inspiration:** Eve Echoes fitting philosophy, ARPG legendary effects, roguelike synergy systems.

Equipment slots should:

- have specialized roles
- support archetype building
- encourage tradeoffs

**Examples:** threat generation relic, poison amplifier trinket, barrier-conversion shield, AoE chain weapon, lifesteal dagger, summon-enhancing idol.

### Stats vs Effects

Items may include HP, damage, armor, attack speed, crit chance — **but** mechanical effects should dominate higher rarities.

**Examples:** poison spreads on kill, barrier reflects damage, cleave hits additional enemies, healing overheals into shields, taunts pulse periodically, crits apply bleed.

**Goal:** “This item changes how the build works.” **Not:** “This item is +7% stronger.”

### Item Rarity Philosophy

| Rarity   | Role                                      |
|----------|-------------------------------------------|
| Common   | Mostly raw stats                          |
| Uncommon | Minor mechanics                           |
| Rare     | Build-enabling effects                    |
| Epic     | Strong synergy pieces                     |
| Legendary| Run-defining mechanics                    |

---

## Phase 3 — Party Combat Foundations

### Goals

Expand combat from **one hero vs one enemy** toward **role-based party combat** while preserving deterministic simulation.

### Threat / Aggro System

Implement threat generation. Enemies should prefer targets based on:

- threat values
- taunt effects
- role modifiers
- reactive mechanics

This allows tanks to function, healers to matter, and DPS to require protection. Without threat, tank builds become redundant.

### Role Identity

- **Tanks:** Generate threat and survive.
- **Healers:** Maintain party survival.
- **DPS:** Deal damage efficiently but risk pulling aggro.
- **Supports:** Buff, debuff, manipulate encounters.

### Threat Design Goals

Threat should:

- be visible in telemetry/debug tools
- support passive/item/skill modification
- allow specialization

**Examples:** +30% threat generation, threat decay reduction, taunt pulse, threat transfer, stealth/threat drop.

---

## Phase 4 — Multi-Enemy Combat

### Goals

Allow encounters with multiple enemies, enemy formations, AoE skills, and target prioritization. Combat should remain deterministic, readable, and simulation-driven.

### AoE Systems

Support cleaves, chain attacks, splash damage, cones, room-wide effects. Both heroes and enemies should eventually support AoE.

### Encounter Variety

Possible room types: swarm rooms, elite packs, summoner encounters, support enemy groups, boss + adds, glass cannon formations — creating build pressure, specialization, and tactical strengths/weaknesses.

---

## Phase 5 — Combat Telemetry & Analysis

### Goals

Help players understand why builds succeed or fail and each party member’s contribution. Telemetry should support balancing, theorycrafting, and party optimization.

### Planned Metrics

**Examples:** DPS per hero, healing done, damage prevented, threat generated, poison contribution, crit rate, uptime metrics, barrier absorption, overkill waste, deaths prevented.

**Eventually:** encounter replay analysis, per-room summaries, build comparison.

---

## Long-Term Design Philosophy

Idle Dungeons should evolve toward:

- automated tactical RPG
- buildcraft sandbox
- deterministic simulation roguelike
- party synergy game

The game should reward experimentation, synergy discovery, creative builds, role composition, and automation strategy — **not** infinite stat inflation, passive idle waiting, or raw number scaling alone.

Every new system should strengthen build identity, item excitement, role specialization, replayability, and emergent gameplay.
