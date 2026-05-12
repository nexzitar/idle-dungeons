# Combat scheduler Phase 1 — Implementation Plan

> **OBSOLETE (archived).** Current execution backlog: [`../ACTIVE-REMAINING-WORK.md`](../ACTIVE-REMAINING-WORK.md). Many steps landed in code; checkboxes below are not maintained.

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace phase-locked per-tick resolution in `simulate_combat_party` with a **dense 100 ms tick** loop that **batches and sorts** all actions due on the same tick using **stable initiative** (spec: `docs/superpowers/specs/obsolete-2026-05-09-combat-feel-timing-skill-flow.md`), preserving **determinism** and passing the existing combat regression suite.

**Architecture:** Keep **`CombatResult` / `CombatEvent` / `combat_playback_frames_from_result`** as the Player-facing API. Introduce a small **`combat_timing`** module for `COMBAT_TICK_MS`, deterministic **initiative ranks** per encounter, and a **total order key** `(tick, initiative_rank, action_lane, sequence)`. Each outer tick: advance weapon meters and cast/CD machinery into **pending actions** for this tick, append scheduled housekeeping (poison tick, `TimingPulse`, threat drips if still tick-scoped), **sort**, execute in order, then advance. **Float meters** stay for Phase 1 (parity with current math); fixed-point can be a follow-up.

**Tech Stack:** Rust 2021, crate `idle_dungeons`, tests via `cargo test`. Primary files: `src/domain/combat.rs`, new `src/domain/combat_timing.rs`, `src/domain/mod.rs`.

---

## File map (Phase 1)

| File | Responsibility |
|------|------------------|
| `src/domain/mod.rs` | `pub mod combat_timing;` |
| `src/domain/combat_timing.rs` | `COMBAT_TICK_MS`, `mix64` / initiative from seed, `ActionLane`, `ActionSortKey`, `Ord` impl, unit tests |
| `src/domain/combat.rs` | Refactor `simulate_combat_party` inner loop to batch+sort; may add private helper types `PendingSwing`, `TickExecutor`; keep `simulate_combat`, `pick_party_enemy_target`, playback helpers |

**Do not** change `RunSummary`, save format, or UI event wiring in Phase 1 unless a `CombatEvent` variant is strictly necessary (prefer **no** new variants until required).

---

### Task 1: Add `combat_timing` module (constants + initiative)

**Files:**
- Create: `src/domain/combat_timing.rs`
- Modify: `src/domain/mod.rs` (add `pub mod combat_timing;`)

- [ ] **Step 1: Create `combat_timing.rs`**

```rust
//! Deterministic combat tick and initiative (see combat feel spec).

/// In-fiction duration of one simulation tick (milliseconds).
pub const COMBAT_TICK_MS: u32 = 100;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ActionLane {
    /// Weapon swings / cast damage resolutions scheduled this tick.
    Proactive,
    /// Poison DoT slice and similar end-of-window effects (after proactive unless spec says otherwise).
    Dot,
    /// Reserved: barrier react, on-hit procs (Phase 3+).
    Reactive,
}

/// Mix seed with a tag; stable, dependency-free (no floats).
#[inline]
pub fn mix64(seed: u64, tag: u64) -> u64 {
    seed.wrapping_add(tag).rotate_left(23).wrapping_mul(0x9E37_79B97F4A7C15)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InitiativeActor {
    Lead,
    Partner,
    Foe,
}

/// Returns **unique** ranks `0..N-1` for actors in this encounter: **lower = earlier** when `ready_at_tick` ties.
/// Index: `0` = lead, `1` = partner, `2` = foe. Unused slots stay `255`.
/// `party_slots`: `1` = solo (lead + foe only), `2` = lead + partner + foe.
pub fn initiative_ranks(encounter_seed: u64, party_slots: u8) -> [u8; 3] {
    let mut entries: Vec<(u8, u64)> = Vec::with_capacity(3);
    entries.push((0u8, mix64(encounter_seed, 0x4C454144u64)));
    if party_slots >= 2 {
        entries.push((1u8, mix64(encounter_seed, 0x50415254u64)));
    }
    entries.push((2u8, mix64(encounter_seed, 0x464F4520u64)));
    entries.sort_by_key(|&(_, k)| k);
    let mut out = [255u8; 3];
    for (rank, (actor_id, _)) in entries.iter().enumerate() {
        out[*actor_id as usize] = rank as u8;
    }
    out
}

#[inline]
pub fn initiative_rank_for(actor: InitiativeActor, encounter_seed: u64, party_slots: u8) -> u8 {
    let idx = match actor {
        InitiativeActor::Lead => 0usize,
        InitiativeActor::Partner => 1usize,
        InitiativeActor::Foe => 2usize,
    };
    initiative_ranks(encounter_seed, party_slots)[idx]
}

/// Total order for pending actions on one tick.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ActionSortKey {
    pub tick: u32,
    pub initiative: u8,
    pub lane: ActionLane,
    pub seq: u32,
}

impl Ord for ActionSortKey {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        (self.tick, self.initiative, self.lane, self.seq).cmp(&(
            other.tick,
            other.initiative,
            other.lane,
            other.seq,
        ))
    }
}

impl PartialOrd for ActionSortKey {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
```

- [ ] **Step 2: Wire module**

Add to `src/domain/mod.rs` after other mods:

```rust
pub mod combat_timing;
```

- [ ] **Step 3: Compile**

Run:

```bash
cd /Users/mattias/prog/Best_Game && cargo check -q
```

Expected: success.

- [ ] **Step 4: Commit**

```bash
git add src/domain/combat_timing.rs src/domain/mod.rs
git commit -m "feat(combat): add combat_timing module (tick ms, initiative, sort key)"
```

---

### Task 2: Unit tests — initiative and sort key stability

**Files:**
- Modify: `src/domain/combat_timing.rs` (`#[cfg(test)]` at bottom)

- [ ] **Step 1: Append tests**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn initiative_deterministic_for_seed() {
        let a = initiative_rank_for(InitiativeActor::Lead, 42, 2);
        let b = initiative_rank_for(InitiativeActor::Lead, 42, 2);
        let c = initiative_rank_for(InitiativeActor::Foe, 42, 2);
        assert_eq!(a, b);
        assert_ne!(a, c, "lead and foe should not always tie on mixed keys");
    }

    #[test]
    fn initiative_ranks_are_permutation_no_collisions_solo() {
        let r = initiative_ranks(99, 1);
        assert_eq!(r[1], 255, "partner absent");
        let mut seen = [false, false];
        for slot in [0usize, 2usize] {
            let k = r[slot] as usize;
            assert!(k < 2, "rank must be 0..2 for solo");
            assert!(!seen[k], "duplicate rank");
            seen[k] = true;
        }
    }

    #[test]
    fn sort_key_orders_lane_then_seq() {
        let a = ActionSortKey {
            tick: 1,
            initiative: 0,
            lane: ActionLane::Proactive,
            seq: 1,
        };
        let b = ActionSortKey {
            tick: 1,
            initiative: 0,
            lane: ActionLane::Dot,
            seq: 0,
        };
        assert!(a < b, "Proactive before Dot at same initiative");
    }
}
```

- [ ] **Step 2: Run tests**

```bash
cd /Users/mattias/prog/Best_Game && cargo test combat_timing -q
```

Expected: all tests pass.

- [ ] **Step 3: Commit**

```bash
git add src/domain/combat_timing.rs
git commit -m "test(combat_timing): initiative and ActionSortKey ordering"
```

---

### Task 3: Encounter seed helper (single source per combat)

**Files:**
- Modify: `src/domain/combat_timing.rs` (add function)
- Modify: `src/domain/combat.rs` (call site later)

- [ ] **Step 1: Add helper**

```rust
/// Derives a per-encounter RNG-neutral seed for initiative and future scheduling.
/// Keep stable fields only: lead identity is NOT required for Phase 1 — use enemy + depth-less fight id.
#[inline]
pub fn encounter_initiative_seed(run_seed: u64, enemy_id_mix: u64) -> u64 {
    mix64(run_seed, enemy_id_mix)
}
```

Document in comment that **Phase 1** uses `run_seed` from caller: thread `0u64` until `simulate_combat_party` gains an explicit `combat_rng_seed` parameter in a later patch, **or** derive from existing arguments:

**Recommended for this task:** In `combat.rs`, when calling initiative, use:

`let enc_seed = combat_timing::encounter_initiative_seed(0, enemy.max_health as u64 ^ enemy.damage as u64);`

This stays **deterministic per enemy shape** (no new API). A follow-up PR may pass real `run_seed` from `run.rs`.

- [ ] **Step 2: Commit**

```bash
git add src/domain/combat_timing.rs
git commit -m "feat(combat_timing): encounter_initiative_seed helper"
```

---

### Task 4: Refactor `simulate_combat_party` — per-tick batch + sort

**Files:**
- Modify: `src/domain/combat.rs` (large refactor inside `simulate_combat_party` only)

**Behavioral goals:**

- Outer loop remains **`for tick in 0..max_clock_ticks`** with `combat_clock = tick + 1` as today.
- **Each tick**, instead of “lead block → partner block → enemy block → poison”:
  1. Increment `seq` counter for this tick (starts 0 each tick, increments for each enqueued action).
  2. **Enqueue** proactive swings: for each hero and foe, run the **same meter / cast / CD math** as today but **do not apply damage** immediately; push structs like `PendingHeroSwing { attacker: u8, sort: ActionSortKey }` into `Vec`.
  3. If poison should tick this iteration (same rules as current “end of tick” poison), enqueue with `ActionLane::Dot` and initiative = **last** (e.g. `initiative_rank_for(Foe, ...) + 128` clamp — better: use `seq` only after proactive by using `lane=Dot` and tie-break with **higher initiative number means later** — already `Dot > Proactive` in enum order? **Fix:** `ActionLane` order has `Proactive < Dot < Reactive`; keep proactive sorted by initiative, then append dots with `initiative = 252` constant **DotPhase** rank).
  4. **Sort** `Vec` by `ActionSortKey`.
  5. **Drain** actions: apply damage/heal/threat/thorns exactly as current inline code does for each swing type.
  6. Emit **`CombatEvent::TimingPulse`** as today (after actions for this tick or preserved order — match current tests).

**Concrete sub-steps:**

- [ ] **Step 1: Read full `simulate_combat_party`** body (`src/domain/combat.rs` ~478–1173) and list all branches that emit `HeroAttacked`, `EnemyAttacked`, `PoisonTick`, `ThornsReflect`, `HeroHealed`, win/lose early returns.

- [ ] **Step 2: Introduce private enum in `combat.rs`**

```rust
enum PendingAction<'a> {
    HeroSwing { attacker: u8, key: crate::domain::combat_timing::ActionSortKey },
    EnemySwing { key: crate::domain::combat_timing::ActionSortKey },
    PoisonSlice { key: crate::domain::combat_timing::ActionSortKey },
}
```

Executor matches on `PendingAction` and invokes **existing** helper closures or copied blocks (avoid duplicating damage formulas — extract `fn apply_hero_swing(...)` if needed).

- [ ] **Step 3: Initiative ranks per swing**

When enqueueing `HeroSwing` for lead, `initiative = initiative_rank_for(Lead, enc_seed, party_slots)`; partner uses `Partner`; foe uses `Foe`. **`seq`** increases per enqueue on that tick.

- [ ] **Step 4: Poison enqueue**

After proactive queue built from meters, if poison rules say tick fires **this** tick, enqueue `PoisonSlice` with `lane: Dot` and `initiative: 255` (or fixed “after all proactive” rank).

- [ ] **Step 5: Run full test suite**

```bash
cd /Users/mattias/prog/Best_Game && cargo test -q
```

Expected: all pass. If ordering changes break party/threat tests, **inspect** diff; either fix ordering bug or, if new order is spec-correct, update test expectations with a **comment** citing initiative spec.

- [ ] **Step 6: Commit**

```bash
git add src/domain/combat.rs
git commit -m "refactor(combat): batch per-tick actions with initiative sort (Phase 1)"
```

---

### Task 5: Targeted regression — same-tick trade fairness

**Files:**
- Modify: `src/domain/combat.rs` (`tests` module)

- [ ] **Step 1: Add test** `same_tick_race_determined_by_initiative`

Construct a minimal `HeroProfile` and `Enemy` where **both** can kill the other on the **first** swing (e.g. hero HP 1, enemy HP 1, hero damage ≥ 1, enemy damage ≥ 1, both `attack_speed` high enough first swing tick 1 — may require tuning `max_clock_ticks` and meters precharge). **Assert** `outcome` matches **exactly** what the engine returns **after** refactor (record once from `dbg!`).

Because numbers are data-dependent, first run:

```bash
cargo test same_tick_race_determined_by_initiative -- --nocapture
```

paste outcome into `assert_eq!(r.outcome, CombatOutcome::HeroWon);` or `EnemyWon` as appropriate.

**Template:**

```rust
#[test]
fn same_tick_race_determined_by_initiative() {
    let mut hero = HeroProfile::new(Stats {
        damage: 5,
        max_health: 1,
        attack_speed: 100.0,
        ..Stats::default()
    });
    hero.unlock_skill_slots(6);
    let enemy = Enemy {
        name: "Straw".into(),
        max_health: 1,
        damage: 5,
        armor: 0,
        attack_speed: 100.0,
        cast_ticks: 0,
        cooldown_ticks: 0,
    };
    let r = simulate_combat_party(
        &hero,
        &enemy,
        50,
        hero.derived_stats().max_health,
        None,
    );
    assert!(
        matches!(r.outcome, CombatOutcome::HeroWon | CombatOutcome::EnemyWon),
        "{:?}",
        r.outcome
    );
    // After first run, lock exact winner:
    // assert_eq!(r.outcome, CombatOutcome::HeroWon);
}
```

- [ ] **Step 2: Replace comment with locked outcome** once observed.

- [ ] **Step 3: Commit**

```bash
git add src/domain/combat.rs
git commit -m "test(combat): same-tick race outcome stable under initiative"
```

---

### Task 6: Docs + changelog

**Files:**
- Modify: `CHANGELOG.md`
- Modify: `docs/superpowers/specs/obsolete-2026-05-09-combat-feel-timing-skill-flow.md` (link to this plan at bottom)

- [ ] **Step 1: Changelog entry** under next patch version:

```markdown
### Combat

- **Phase 1 scheduler:** per-tick combat actions are **sorted** by **stable initiative** (see `combat_timing` + spec) instead of a fixed hero-then-foe phase lock. Tick quantum remains aligned with **100 ms** fiction (`COMBAT_TICK_MS`).
```

- [ ] **Step 2: Spec footer** — add:

```markdown
## Implementation

- Phase 1 plan: [`docs/superpowers/plans/obsolete-2026-05-09-combat-scheduler-phase1.md`](../plans/obsolete-2026-05-09-combat-scheduler-phase1.md)
```

- [ ] **Step 3: Commit**

```bash
git add CHANGELOG.md docs/superpowers/specs/obsolete-2026-05-09-combat-feel-timing-skill-flow.md
git commit -m "docs: link combat Phase 1 plan and changelog scheduler note"
```

---

## Self-review (plan author)

| Spec section | Task coverage |
|--------------|----------------|
| Dense 100 ms tick | Task 4 keeps dense loop; `COMBAT_TICK_MS` documents quantum |
| Scheduler A (batch same tick) | Task 4 |
| Stable initiative ties | Tasks 1–2, 4–5 |
| Carryover meters | Task 4 — preserve existing float meter logic |
| Parity / tests | Task 4–5 + full `cargo test` |
| Out of scope Phase 2+ | No skill categories / buff events added |

**Placeholder scan:** No `TBD` in executable steps; known follow-up: pass real `run_seed` into `encounter_initiative_seed` (Task 3 comment).

**Type consistency:** `ActionSortKey`, `ActionLane`, `InitiativeActor` used consistently; `party_slots` `u8` `1` or `2`.

---

## Execution handoff

**Plan complete and saved to `docs/superpowers/plans/obsolete-2026-05-09-combat-scheduler-phase1.md`. Two execution options:**

1. **Subagent-Driven (recommended)** — Dispatch a fresh subagent per task, review between tasks, fast iteration (**superpowers:subagent-driven-development**).

2. **Inline execution** — Run tasks in this session using **superpowers:executing-plans**, batch execution with checkpoints.

**Which approach do you want?**
