# MVP Remaining Work — Gap Plan

> **For agentic workers:** Use superpowers:subagent-driven-development or superpowers:executing-plans for task-by-task execution. Track with `- [ ]` checkboxes.

**Goal:** Close the gap between “checklist MVP” (tests + loop + persistence) and the playable, build-centric MVP described in README and `docs/superpowers/specs/2026-05-06-roguelike-incremental-bevy-design.md`.

**Architecture:** Keep the run simulation as source of truth; extend domain with tests first; wire new player intent through Bevy events and `ProfileState` / `SaveProfile`, then UI. Prefer small vertical slices (e.g. skill picker before full combat rework).

**Tech stack:** Rust, Bevy, existing `src/domain`, `src/app`, `src/ui` modules.

---

## Reality check: where the project is

| Area | Status |
|------|--------|
| Loop (build → run → summary → camp → save) | Working |
| Gear equip/salvage, upgrades, deterministic runs | Working |
| `docs/mvp-acceptance.md` | All items checked |
| **In-game skill loadout** | **Working** — `CycleHeroSkillSlot` / skill slot buttons on Build & Camp; see `src/app.rs`, `src/ui/mod.rs` |
| **Combat uses skill catalog** | **Partial (implementation exists; deepen & align)** — `simulate_combat` applies Lifesteal, Guard (flat reduction on hits), Heavy (+damage on swing), Poison (`PoisonTick` on hero swing), Thorns (reflect), Barrier (shield). See mapping table below; poison-as-DoT-over-ticks and Heavy “slow” tradeoff are roadmap items in `docs/superpowers/plans/2026-05-07-roadmap-implementation.md`. |
| **Gold gain upgrade** | **Working** — `gold_gain_multiplier` on `RunConfig` from meta in `src/app.rs`; applied in `src/domain/run.rs` |
| Stash “Filters / Sort” | Decorative copy only |
| README | Updated (`README.md`); roadmap + layout |

### Skill definitions vs `simulate_combat` (authoritative as of 2026-05)

| Skill | `SkillTrigger` (catalog) | Behavior in `simulate_combat` |
|-------|--------------------------|--------------------------------|
| LifestealStrike | OnAttack | Heal on hero attack (`HeroHealed`). |
| Guard | OnHitTaken | Reduces enemy hit damage by flat amount (currently hardcoded offset). |
| HeavyStrike | OnAttack | Bonus damage on hero swing; **no** attack-speed penalty yet (description says “slow”). |
| PoisonEdge | OnAttack | Extra damage via `PoisonTick` on each hero swing — **burst-on-swing**, not periodic room DoT yet. |
| ThornSkin | OnHitTaken | Reflect after hero takes HP loss from enemy hit. |
| BarrierPulse | OnRoomStart | Shield at combat start, absorbs before HP loss. |

**Execution plan:** `docs/superpowers/plans/2026-05-07-roadmap-implementation.md` (Phase A–E).

---

### Priority A — Skill loadout (player-facing core) — **DONE**

The interactive loadout described below is implemented (cycle slots, `equip_skill`, persistence). This section is kept for history.

<details>
<summary>Original gap text (archived)</summary>

**Problem (resolved):** The design promises “configure the hero” with unlocked slots and a six-skill pool; today slots unlock in meta but assignments are not editable in-game.

**Files (typical):**
- `src/app.rs` — e.g. `ChangeHeroSkill { slot: usize, skill: Option<SkillId> }` or `EquipSkill` / `ClearSkillSlot`; validate with `HeroProfile::equip_skill`; save after change.
- `src/save.rs` — no schema change if using existing `HeroProfile` fields.
- `src/ui/mod.rs` / `src/ui/mockup_layout.rs` — interactive skill row: click slot → picker (dropdown or modal) listing `SkillId::iter()` or static list; respect `unlocked_skill_slots`.
- Tests: app or domain integration test that event updates profile and persists (optional).

**Suggested order:**
1. Domain/event API + unit test (invalid slot, locked slot, duplicate skill policy — decide: allow duplicate or one-of-each).
2. Minimal UI: assign/clear for slot 0–1 on Build (and optionally Camp).
3. Save on change (same pattern as equip/salvage).

</details>

---

### Priority B — Combat reflects the skill roster (depth, not just labels)

**Status:** Baseline behaviors exist; **deepen** per roadmap Phase A (poison over ticks, guard scaling, heavy tradeoff, tests).

**Problem (narrowed):** Some catalog text (DoT, “slow” heavy) does not fully match tick-level simulation; add tests and tune.

**Approach (incremental):**
1. ~~Inventory desired behaviors~~ — see table above.
2. Add focused tests per skill (**especially** poison across clock ticks without extra swings).
3. Extend `simulate_combat` in small steps — follow `2026-05-07-roadmap-implementation.md`.

**Files:** `src/domain/combat.rs`, possibly `src/domain/skills.rs` if shared helpers.

---

### Priority C — Gold gain upgrade does something — **DONE**

Multiplier is threaded into run simulation. This section is kept for history.

<details>
<summary>Original gap text (archived)</summary>

**Problem (resolved):** Players can buy `GoldGain` but run gold may ignore it.

</details>

### Priority D — UX honesty and polish

- **Stash filters:** Implement minimal sort (rarity, name) and/or filter toggles, **or** replace static line with neutral placeholder so UI doesn’t promise controls that don’t exist.
- **Skill row:** After Priority A, show full skill names in tooltips or secondary text (first-letter chips are cute but opaque).
- **README:** Update “early planning” + point at `mvp-acceptance.md` and current loop.

---

### Priority E — Optional / post-MVP

- Extract plugins per README (`GameStatePlugin`, …) when files grow painful.
- Use `attack_speed` in combat pacing if design requires it (currently alternating fixed steps per tick).
- More affix ↔ skill synergy if combat supports those tags.

---

## Suggested milestone ordering

1. **Combat depth + honesty:** Roadmap Phase A (`simulate_combat` alignment) + Phase B stash copy or sort (`2026-05-07-roadmap-implementation.md`).
2. **World + shell:** Roadmap Phase C–D (dungeon/briefing, presentation).
3. **Post-MVP:** Priority E + roadmap Phase E.

## Verification

After each slice: `cargo fmt --check`, `cargo test`, `cargo check`, manual run: new game / existing save, assign skills, verify combat log and summary change as expected.
