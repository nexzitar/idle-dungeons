# Active Remaining Work (Combat & Buildcraft Direction)

**Purpose:** Single strategic backlog for combat feel, buildcraft, and presentation. Historical dated plans live under `plans/specs/obsolete-`*.

**In code today (cross-check):**


| This doc                                    | Code / files                                                                                                             |
| ------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------ |
| Shared GCD vs weapon cadence                | `[skill_triggers_shared_ability_gcd](../../src/domain/skills.rs)`, `[simulate_combat_party](../../src/domain/combat.rs)` |
| Swing vs instant vs next-melee              | `[SkillCombatStyle](../../src/domain/skills.rs)`                                                                         |
| Skill **category** (player-facing taxonomy) | `[SkillCategory](../../src/domain/skills.rs)` + tooltips / book rows                                                     |
| **Skill layering** (mutual exclusivity hints) | `[skill_layering](../../src/domain/skill_layering.rs)` + build panel                                                     |
| Initiative ordering                         | `[combat_timing](../../src/domain/combat_timing.rs)`, `[sorted_strike_actors](../../src/domain/combat_round.rs)`         |
| Fixed-point weapon swing meters             | `[combat_meter](../../src/domain/combat_meter.rs)` + party/foe meters in `[simulate_combat_party](../../src/domain/combat.rs)` |
| Nine-slot gear + loot budgets               | `[GearSlot](../../src/domain/items.rs)`, `[loot](../../src/domain/loot.rs)`                                              |
| Playback timing bars                        | `[CombatPlaybackFrame](../../src/domain/combat.rs)`, theater stacks in `[mockup_layout](../../src/ui/mockup_layout.rs)`  |


---

## Prioritized execution waves (check off in PRs)

Use this as a **sequence**, not parallel pillars—later waves assume earlier ones when noted.

- [x] **Wave 1 — Readability & taxonomy** — `SkillCategory` + skill book labels; floating combat text respects party vs foe anchors; timing semantics in `combat_timing` / `simulate_combat_party` rustdoc.
- [x] **Wave 2 — Skill layering rules** — [`skill_layering`](../../src/domain/skill_layering.rs): layer slots + loadout warnings (Heavy vs Cleave today); tests; build panel surfaces notices. Extend the table as new mutually exclusive pairs land (§A skill layering).
- [x] **Wave 3 — Engine hardening** — Fixed-point meters; poison scheduling decision + tests; scheduler stress / same-tick lethal tests (§B).
- [x] **Wave 4 — Itemization & progression curve** — Nine-slot gear spread + compressed loot budgets; **Rhythm** affix (weave recovery); **encounter score** on `RunSummary`; legacy save slot aliases; §C/D alignment.
- [ ] **Wave 5 — Roles, multi-foe MVP & archetype hints** — Threat decay/transfer/taunt pulse; 2-enemy room + cleave targeting; seed **combat archetype framework** (§E) as tag/heuristic readouts for loot weighting / telemetry — no class locking.
- [ ] **Wave 6 — Telemetry slice** — One run summary metric (e.g. white vs ability damage %) from existing events (§F).
- [ ] **Wave 7+ — Presentation / audio / art** — After combat language is stable (§G).

**Design sections §A–§H** below remain intent-only; scope above is what ships first.

Sections **A–G** below stay as **design intent**; track delivery with the waves above.

---

## Core Design Philosophy

Delvers is evolving away from “idle stat progression” and toward a deterministic combat simulation focused on:

- combat cadence
- readable combat flow
- build-defining itemization
- layered skill interactions
- asynchronous timing
- party role synergy
- long-term buildcraft experimentation

The simulation remains authoritative (`src/domain/`*).  
UI, playback, floating combat text, and telemetry exist to make combat understandable and satisfying — not to drive outcomes.

---

# A. Combat Flow & Readability (HIGH PRIORITY)

The combat engine now supports asynchronous initiative and attack-speed cadence. The next milestone is turning this into a readable and expressive combat language.

- **Global cooldown system**
  - Certain active abilities trigger shared GCD.
  - Buffs/reactives/passives may bypass GCD.
  - Preserve weapon cadence during most skill activations.
- **Skill category framework**
  - `BasicAttack`
  - `AttackSkill`
  - `Buff`
  - `Reactive`
  - `Passive`
  - `Channel`
  - `Proc`
- **Ability runtime visibility**
  - Cooldown overlays
  - Recharge indicators
  - Buff duration bars
  - Proc highlights
  - GCD pulse visibility
- **Combat readability pass**
  - White damage = weapon/basic attacks
  - Yellow/orange damage = active skill damage
  - Crits use emphasis (`!`, scale pulse, brighter tint)
  - Reduce floating combat text spam
- **Skill layering rules**
  - Example:
    - Poison Weapon + Heavy Strike = allowed
    - Heavy Strike + Whirlwind = mutually exclusive attack actions
  - Define action windows and overlap policies explicitly.
- **Weapon cadence preservation**
  - Basic attacks continue independently of most abilities.
  - Attack skills layer on top of combat rhythm instead of replacing it.
- **Combat event compression**
  - Merge repetitive low-value events
  - Prevent late-game visual overload
- **Combat feel regression tests**
  - Preserve cadence behavior across scheduler refactors.
  - Snapshot representative Swift / Heavy / proc-heavy fights.

---

# B. Combat Engine & Timing

The scheduler exists; this phase hardens it into the permanent combat foundation.

- **Fixed-point weapon meters**
  - Replace remaining float timing state.
  - Integer tick authority only.
- **Poison scheduling review**
  - Decide between:
    - end-of-tick batching
    - independently scheduled DoT slices
  - Prioritize readability and deterministic ordering.
- **Timing semantics documentation**
  - Clarify:
    - attack cadence
    - GCD timing
    - cast timing
    - recharge timing
    - wall-clock interpretation
- **Scheduler stress tests**
  - High APS
  - Multi-proc chains
  - Multi-enemy encounters
  - Same-tick lethals
  - Barrier/reactive ordering
- **Initiative expansion hooks**
  - Leave room for:
    - haste
    - initiative stats
    - encounter modifiers
    - ambush mechanics

---

# C. Build-Defining Itemization

Delvers itemization should shift away from giant stat spikes and toward combat identity.

- **Stat compression**
  - Prevent single-item progression explosions.
  - Smooth early-game scaling heavily.
- **Build-defining affixes**
  - Items should alter:
    - cadence
    - proc behavior
    - cooldown flow
    - targeting
    - threat
    - buff interaction
    - survivability patterns
- **Slot identity**
  - Equipment slots should have distinct gameplay roles.
  - Inspired more by MMO fitting systems than generic ARPG stat sticks.
- **Affix synergy framework**
  - Tags + combat states over isolated stat bonuses.
  - Encourage interaction chains.
- **Legendary identity pass**
  - Legendary items should meaningfully change gameplay patterns.
- **Attack speed ecosystem**
  - Faster attacks should naturally improve:
    - poison application
    - lifesteal frequency
    - proc rate
    - resource generation
    - reactive triggers
- **Item text readability**
  - Explain gameplay behavior clearly.
  - Reduce debug-style affix descriptions.

---

# D. Reward Pacing & Progression

Progression should feel incremental, readable, and long-term.

- **Reward pacing redesign**
  - Fewer jackpot spikes.
  - More incremental growth.
- **Encounter score economy**
  - Explore score/depth reward conversion instead of pure loot explosion scaling.
- **Encounter score / survival progression**
  - Explore progression models beyond pure floor-count scaling.
  - Investigate:
    - enemy kill score
    - survival duration
    - wave pressure
    - encounter efficiency
    - elite kill weighting
  - Goal:
    - smoother reward pacing
    - reduced late-game floor inflation
    - more readable progression scaling
- **Early-game progression smoothing**
  - First elite should feel threatening but eventually conquerable through gradual improvement.
- **Infinite dungeon progression**
  - One endlessly scaling delve becomes the primary progression ladder.
- **Milestone unlocks**
  - Skill slots
  - Party slots
  - Mechanics
  - Systems
  - Build options
- **Inventory pressure review**
  - Reevaluate slot count vs power budget.
  - More slots may allow less extreme item spikes.
- **Equipment slot expansion review**
  - Reevaluate long-term equipment slot count.
  - More slots may allow:
    - smaller individual stat spikes
    - smoother progression curves
    - more specialized affixes
    - more nuanced gearing decisions
  - Current 3-slot structure concentrates too much power per item and risks jackpot progression jumps.
- **Loot pacing pass**
  - Trash/common drops become frequent.
  - Rare/high-impact drops become more curated.
- **Run readability**
  - Fewer but more meaningful reward moments.

---

# E. Party Roles & Encounter Design

Party gameplay should emerge naturally from combat systems instead of hard-coded classes.

- **Threat system expansion**
  - Threat generation
  - Threat decay
  - Threat transfer
  - Taunt windows
- **Tank identity**
  - Tanks survive via mitigation and aggro control.
  - Healers keep tanks alive.
- **Hybrid role viability**
  - Support DPS
  - Self-healing bruisers
  - Off-tanks
  - Reactive support builds
- **Combat archetype framework**
  - Internal combat identities should emerge naturally from mechanics, tags, timing, and itemization.
  - Examples:
    - Burst DPS
    - Sustain DPS
    - Tank
    - Bruiser
    - Proc-based attacker
    - Poison ramp
    - AoE clearer
    - Reactive defender
    - Support/control
  - Archetypes should influence:
    - item generation
    - affix weighting
    - encounter design
    - telemetry
    - future AI/autobuild systems
    - role readability
  - Avoid rigid class-locking; archetypes should emerge from builds rather than fixed hero classes.
- **Multi-enemy encounters**
  - Multiple foes per room
  - Target selection
  - Threat distribution
  - AoE rules
- **AoE skill framework**
  - Cleave
  - Whirlwind
  - Splash damage
  - Multi-target DoTs
- **Party scaling**
  - Unlock additional party slots gradually.
  - Long-term target: 5-character parties.

---

# F. Telemetry & Feedback

The player should be able to understand WHY builds work.

- **Damage meters**
  - DPS
  - damage sources
  - white vs skill damage
  - proc contribution
- **Buff uptime tracking**
  - Poison uptime
  - GCD idle %
  - skill activity %
- **Threat visibility**
  - Current aggro target
  - Threat lead
  - Threat spikes
- **Build analytics**
  - “What actually carried this run?”
- **Run summaries**
  - Top skills
  - Most impactful affixes
  - Buff uptime
  - Kill contribution

---

# G. Presentation & Visual Identity

The current UI proves systems; future work should improve atmosphere and readability.

- **Combat theater polish**
  - Cleaner floating text
  - Better readability hierarchy
  - Stronger timing emphasis
- **Skill visuals**
  - Distinct visual identity per skill category.
- **Animation pass**
  - Weapon cadence readability
  - Attack anticipation
  - Impact timing
- **Pixel-art replacement pass**
  - Replace placeholders gradually.
  - Preserve readability-first philosophy.
- **Combat audio layer**
  - Cadence-driven impact sounds
  - Crit emphasis
  - Buff activation cues

---

# H. Future Expansion Ideas (NOT CURRENT PRIORITY)

These are intentionally deferred until the combat/buildcraft foundation feels complete.

- Prestige systems
- Branching dungeon graphs
- Alternate progression currencies
- Multiplayer / co-op delves
- Raid encounters
- Seasonal ladders
- Async ghost runs

