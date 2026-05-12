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
| Playback timing bars                        | `[CombatPlaybackFrame](../../src/domain/combat.rs)`, theater stacks in `[mockup_layout](../../src/ui/mockup_layout.rs)`; pack off-target foe timing row when encounter lists multiple foes (`" · "` in playback) |
| Multi-foe pack combat (`Vec` HP, cleave)   | `[simulate_combat_party_foes](../../src/domain/combat.rs)`, `[initiative_ranks_pack](../../src/domain/combat_timing.rs)`, `[sorted_strike_pack_order](../../src/domain/combat_round.rs)`; cap `MAX_COMBAT_FOES` in `[combat.rs](../../src/domain/combat.rs)` |
| Archetype hints (loot bias)                 | `[combat_archetype](../../src/domain/combat_archetype.rs)`, affix weights in `[loot](../../src/domain/loot.rs)`                                                                         |
| Party threat decay / taunt pulse            | `[tick_party_threat_routing](../../src/domain/party.rs)` + calls in `[combat.rs](../../src/domain/combat.rs)`                                                                              |
| Run strike telemetry (white vs ability)     | `[party_strike_damage_white_yellow](../../src/domain/combat.rs)` on [`RunSummary`](../../src/domain/run.rs) |

## Prioritized execution waves (check off in PRs)

Use this as a **sequence**, not parallel pillars—later waves assume earlier ones when noted.

- [x] **Wave 1 — Readability & taxonomy** — `SkillCategory` + skill book labels; floating combat text respects party vs foe anchors; timing semantics in `combat_timing` / `simulate_combat_party` rustdoc.
- [x] **Wave 2 — Skill layering rules** — [`skill_layering`](../../src/domain/skill_layering.rs): layer slots + loadout warnings (Heavy vs Cleave today); tests; build panel surfaces notices. Extend the table as new mutually exclusive pairs land (§A skill layering).
- [x] **Wave 3 — Engine hardening** — Fixed-point meters; poison scheduling decision + tests; scheduler stress / same-tick lethal tests (§B).
- [x] **Wave 4 — Itemization & progression curve** — Nine-slot gear spread + compressed loot budgets; **Rhythm** affix (weave recovery); **encounter score** on `RunSummary`; legacy save slot aliases; §C/D alignment.
- [x] **Wave 5 — Roles, multi-foe MVP & archetype hints** — Party **threat decay** + **taunt pulse** (transfer + burst) when partner is in tank stance; **multi-foe** packs + cleave (incl. elite **twin** rooms); **[`combat_archetype`](../../src/domain/combat_archetype.rs)** hints + **loot affix nudges** from lead loadout; treasure / guided drops use hints. *Deferred (still §E):* focus-fire-only foe targeting for heroes; symmetric lead/partner API rename. *(§E pack cast/CD UI for non-primary foes: Wave 7.)*
- [x] **Wave 6 — Telemetry slice** — [`RunSummary`](../../src/domain/run.rs) **strike** totals (weapon vs ability) from [`CombatEvent::HeroAttacked`](../../src/domain/combat.rs); rewards digest + modal (§F).

> **Wave 6 is complete** — shipped **`idle_dungeons` v0.2.30** ([`CHANGELOG.md`](../../CHANGELOG.md)); merged as [PR #41](https://github.com/nexzitar/idle-dungeons/pull/41). If you still see an unchecked box, refresh from **`master`** — look for the **`[x]`** on the Wave 6 line above.

- [x] **Wave 7 — Presentation slice (§G, first tranche)** — **Readability-first combat UI:** stronger floating-text hierarchy (white vs ability vs crit) and less spam where [`playback_float_text_color`](../../src/ui/theme.rs) / theater captions allow; **multi-foe:** surface cast/CD or telegraph for **non-primary** foes where pack fights need it (extends §E deferral); **skill category** affordance in book/build (reuse `SkillCategory` colors or chips). *Out of Wave 7 scope:* new art assets, full animation pass, audio bank.
- [ ] **Wave 8+ — Audio, motion, and art** — Impact/crit/buff SFX, cadence-driven audio, pixel-art swap, anticipation/impact timing (remaining §G).

> **Wave 7 is complete** — shipped **`idle_dungeons` v0.2.31** ([`CHANGELOG.md`](../../CHANGELOG.md)).

**Design sections §A–§H** below remain intent-only; scope above is what ships first.

Sections **A–G** below stay as **design intent**; track delivery with the waves above.

### Wave 7 — suggested execution order (draft)

1. **Trace playback → float text** — Follow combat events into the theater (e.g. [`mockup_layout`](../../src/ui/mockup_layout.rs)), [`playback_float_text_color`](../../src/ui/theme.rs), and any caption/float spawn helpers; list all code paths that emit floats today.
2. **Floating combat text pass** — Align with §A / §G: clearer hierarchy (weapon vs ability vs crit vs DoT), optional merge/cap for spam; add or extend tests only where behavior is easy to lock (e.g. color mapping).
3. **Multi-foe cast/CD parity** — If [`CombatPlaybackFrame`](../../src/domain/combat.rs) / timing pulses only drive one foe row, extend data + UI so secondary living foes show wind-up/recovery when it affects readability (per §E “Playback / UI parity for packs”); respect existing Bevy `Query` disjointness notes in UI code.
4. **Skill category affordance** — Reuse [`SkillCategory`](../../src/domain/skills.rs): tint, chip, or icon column in [`skill_book`](../../src/ui/skill_book.rs) and/or [`build_panel`](../../src/ui/build_panel.rs) so taxonomy is visible at a glance.
5. **Verify** — `cargo test`, manual playback on a seed that hits **multi-foe** (e.g. elite twin) and a long single-target fight.

*(Steps 2–4 can be separate PRs; order 2 → 4 → 3 is fine if multi-foe work needs more design time.)*

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

- **Symmetric actors (intent)**  
  The **party is heroes**, not a privileged “lead” plus sidekick. Any hero slot should be able to function as the one playable character today, with **additional heroes added as peers**—same rules, same dignity in APIs and UI copy. The same principle applies to **enemies**: scalable packs, not a special “main foe + extras” except where pacing/content demands it. Implementation may still use `lead`/`partner` names in code paths for now; refactoring toward **slot-indexed heroes** should preserve this equality so co-op or AI teammates do not bake in hierarchy. **Mind control** (below) leans on this: temporarily moving a unit between sides is easier if both sides are “party lists” with shared action resolution.

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

Party gameplay should emerge naturally from combat systems instead of hard-coded classes. **Implementation stance:** model **heroes and foes as ordered, capped slot lists** (equal heroes in intent—Core Design Philosophy, **Symmetric actors**); refactor away from “lead/partner” semantics in public APIs over time.

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
  - Target selection **beyond focus-fire** on lowest-index living foe: e.g. **threat tables on the foe side**, player/indirect control, or encounter scripts
  - Threat distribution
  - AoE rules
- **Playback / UI parity for packs**
  - Today timing pulses emphasize one “primary” foe’s cast/CD; **surface secondary (and further) foes’ wind-up and recovery** where it affects readability (stacked bars, compact row, or tooltip), not only the focused target
- **AoE skill framework**
  - Cleave, whirlwind, splash, multi-target DoTs
  - **Configurable splash count:** support strikes that hit the primary target plus **N additional enemies** (e.g. 1–2), not strictly “every other living foe.” *Current sim:* Cleave applies to **all** other living foes in the pack—add a numeric cap / targeting predicate when skills need stricter limits
- **Mind control (future mechanic)**
  - Ability usable by **players and enemies**: temporarily **control one unit on the opposing side** so they act for your side’s benefit (to the best of their AI/abilities). Requires clear ownership of initiative, threat, and “which party’s” buff/debuff state for the controlled unit; fits the symmetric party/enemy list model above
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
  - **Multi-foe packs:** surface cast/CD (and related telegraphs) for more than one foe when useful—see §E “Playback / UI parity for packs”
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

- **Mind control** — cross-side unit control (see §E); also listed there for combat design
- Prestige systems
- Branching dungeon graphs
- Alternate progression currencies
- Multiplayer / co-op delves
- Raid encounters
- Seasonal ladders
- Async ghost runs

