# Combat feel, timing & skill flow — design spec

**Status:** draft for review  
**Anchors:** simulation-first (`src/domain/combat.rs`), deterministic playback, Bevy UI consumes events only.

## Decisions (locked for Phase 1 planning)

| Topic | Decision |
|-------|----------|
| Timing engine | **A — shared discrete-event scheduler** (single timeline; next action pops from a priority queue / sorted structure). |
| Time quantum | Each simulation **tick** represents **100 ms** of fictional time. All durations (GCD, casts, buffs, cooldowns) are expressed as **integer tick counts** (e.g. 6 ticks = 600 ms). Display and copy may say “0.6s” but **authority is ticks**. |
| Determinism | Same `(seed, hero, enemy, max_ticks, …)` ⇒ **identical ordered event list**. Tie-breaking when two actions share the same tick **must be explicit and stable** (see below). |
| Same-tick order | **Stable initiative** — not a fixed “enemy always first” rule. |

---

## Goals

Refactor combat from a **phase-locked** loop (heroes block → enemy block → DoTs) into a **layered, readable, asynchronous** simulation:

- **Meaningful attack speed** (frequency and carryover).
- **Visible** skill usage, buffs, cooldowns, and pacing (playback + tooltips).
- **Build identity** via categories, GCD layering, and timed windows.
- **No compromise:** domain remains authoritative; **replayability** and **tests** guard behavior.

Non-goals in v1 of this spec: final art, pixel-perfect VFX; **placeholder** icons/bars/text are enough if they reflect true state.

---

## Current codebase (baseline)

Today, `simulate_combat_party` advances a **global clock** `tick = 0..max_clock_ticks`. Within each tick:

- Lead and partner can accumulate **weapon meters** (`+= attack_speed`, fire while `>= 1.0`) — **carryover already exists** per actor.
- An alternate path uses **cast + cooldown ticks** for OnAttack actives.
- Enemy uses **`enemy_meter`** / cast / CD similarly.
- **Order is roughly:** lead actions → partner → enemy → poison tick (and other end-of-tick housekeeping).

**Problem:** actions that “occur” in the same 100 ms window are **batched by phase**, not **interleaved** by true readiness time. That reads as synchronized and hides rhythm.

---

## Time model

- **Tick:** `u32` (or `u64` if needed for long fights), `tick_duration_ms = 100` **constant**.
- **Simulation time:** `sim_time_ms = tick * 100` (derivable; do not duplicate as floats in domain).
- **Scheduling:** every schedulable action has **`ready_at_tick: u32`** (or `ready_at_tick` + **sub-order** for ties). The main loop:

  1. Advance **global tick** to the **minimum** `ready_at_tick` among pending actions (or step tick-by-tick if we require dense telemetry every 100 ms — see “Dense vs sparse clock”).
  2. Execute **all** actions tied to that tick in **tie-break order**.
  3. Re-queue next occurrences (swing, dot tick, buff expiry, etc.).

### Dense vs sparse global clock

Two valid patterns:

1. **Dense:** increment `tick` by 1 each iteration; each iteration processes **only** events scheduled for **this** tick (may be zero or many). Simple for “every 100 ms snapshot” playback (`TimingPulse` every tick).
2. **Sparse:** jump `tick` to next event time. Fewer iterations; must still **emit** playback snapshots if UI assumes per-tick updates (either interpolate in UI or emit **synthetic** pulse events when jumping — prefer **dense** early to avoid playback gaps unless profiling proves need).

**Recommendation for Phase 1:** **dense clock** up to `max_clock_ticks`, matching current loop shape and keeping `TimingPulse` semantics easy.

### Tie-breaking / initiative (determinism contract)

When multiple actions resolve on the **same tick**, use **stable initiative ordering**:

- **Why not “enemy first always”:** hard-coding foe-before-hero (or the reverse) makes **same-tick lethal** outcomes, **simultaneous damage**, **interrupts**, and **reactive shields/barriers** feel arbitrary or unfair—players read it as the engine picking a favorite. A **declared, stable** initiative order keeps outcomes **predictable and reviewable** while still deterministic.
- **Initiative is stable:** derive a total priority **once per encounter** from fixed inputs (e.g. encounter seed + participant roster + a small deterministic tie-break table). Same inputs ⇒ same initiative stack for the whole fight. Optional future knobs (DEX, talents) can **recompute** initiative only when those stats change mid-fight, still from explicit rules.

**Resolution sorting key** (conceptual; exact encoding in implementation plan):

1. **Primary:** `ready_at_tick` (must match for same-tick batch).
2. **Secondary:** **initiative rank** among actors (lead, partner, enemy, …) from the stable encounter list—**not** “enemy wins ties by default.”
3. **Tertiary:** action **class** (e.g. proactive weapon/skill vs scheduled DoT vs reactive proc) so systems like “react after damage resolves” stay well-defined—reactives still need **acyclic** rules; if a reactive fires in the same tick as the hit, its **sub-order** must be after the triggering damage unless a skill says otherwise.
4. **Quaternary:** monotonic **sequence** / action id for absolute uniqueness.

Document the final sort tuple in code (`const` + comment). Add **unit tests** for tie scenarios (same-tick trade, shield absorb edge, double KO) so order regressions are caught.

### Carryover / attack speed

Weapon **period** in ticks: e.g. `period_ticks = max(1, round(1.0 / attack_speed in “swings per tick”))` — **exact formula TBD in implementation plan**; must preserve current power curve approximately or be called out as rebalance.

**Accumulator pattern (preferred):** keep fractional progress **in fixed-point** (e.g. `u32` thousandths) per actor:

- Each tick: `progress += attack_speed_fixed;` when `progress >= threshold`, emit swing(s), `progress -= threshold`, allow **multiple** swings in one tick if speed extreme (cap per tick if spam becomes unreadable — if capped, document as **game design** limit, not float noise).

This matches current float meter behavior but removes cross-platform float risk if we ever care.

---

## Phase 1 — Asynchronous attack timing (implementation focus)

**Deliverables:**

- Replace phase-locked “all lead, then partner, then foe” with **scheduler-driven** resolution on a **dense 100 ms tick**.
- **One** authoritative ordering policy and **golden / parity tests** vs current outcomes for a **small suite** of scenarios (or deliberate “known divergences” list if parity impossible).
- Playback still driven by `Vec<CombatEvent>`; extend variants only if needed for Phase 1 (e.g. finer-grained swing reason).

**Out of scope for Phase 1:** full buff system, GCD skill categories — only hook **scheduler shapes** so later phases attach.

---

## Phase 2 — Skill category model

Introduce **`SkillCategory`** (names indicative):

| Category | Consumes | GCD | Examples (rename/map over time) |
|----------|----------|-----|----------------------------------|
| **Core attack** | Weapon / primary action slot | Yes (default) | Heavy Strike, Cleave-style actives |
| **Buff / enhancement** | Off-GCD or own budget | Optional / short | “Poison Coating” windows |
| **Reactive** | Triggered | Usually no | Barrier Pulse, revenge-style |
| **Passive** | — | — | Thick Hide, Toxic Mastery |

**Data-driven:** `SkillDefinition` gains category + gcd flags + `tick` costs.

---

## Phase 3 — Buff state system

- **Duration buffs**, **charge buffs**, **stacks**, **aura** hooks — all **tick-indexed** (`expires_at_tick`, `charges_remaining`).
- Refactors called out in product brief (Poison Strike → Poison **Coating**; Lifesteal Strike → **Blood Frenzy** window) become **content** changes once the buff runtime exists.
- Events: `BuffApplied`, `BuffTick`, `BuffExpired`, `BuffChargeConsumed` (exact names TBD) for playback.

---

## Phase 4 — Cooldown & GCD

- **GCD** as a **shared lockout** on **core attack** category (duration in ticks); buffs/reactions can be excluded.
- Per-skill **cooldown** timers separate from GCD where needed.
- Prevent “proc soup”: rate limits, shared buckets, or **event coalescing** in UI — domain may still emit truth; UI samples.

---

## Phase 5 — Combat readability

- **UI:** skill highlight, CD overlay, buff row, status icons — fed by **events** + **final state snapshots** per tick.
- Logs supplement visuals, not replace them.

---

## Testing & migration

- **Property:** same inputs ⇒ same `events` sequence (or same **normalized** playback if adding purely cosmetic events — avoid that early).
- **Regression suite:** snapshot tests for representative combats before/after Phase 1 scheduler.
- **Fuzz / invariants:** HP never NaN, damage ≥ 0, scheduler queue empty ⇒ combat ended or timed out.

---

## Open points (for implementation plan)

- Exact **fixed-point** format for meters vs `f32` parity with today.
- **max_clock_ticks** meaning: still hard cap wall-clock equivalent (`ticks * 100ms`) or separate timeout?
- Whether **poison** remains end-of-tick batch or becomes **scheduled** slices for readability.
- Party + threat interaction order when scheduler interleaves.

---

## Next step (process)

After this spec is reviewed, use **writing-plans** to break Phase 1 into PR-sized tasks (scheduler core → parity tests → playback verification → UI pulse if needed).
