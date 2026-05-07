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
| **In-game skill loadout** | **Missing** — no `equip_skill` from UI/app; saves can have empty slots forever |
| **Combat uses skill catalog** | **Partial** — `simulate_combat` only special-cases `LifestealStrike`; other skills are definitions + UI labels |
| **Gold gain upgrade** | **Likely ineffectual** — `UpgradeId::GoldGain` exists in UI/meta; `simulate_run_with_playback` does not apply a gold multiplier |
| Stash “Filters / Sort” | Decorative copy only |
| README “early planning” | Out of date vs playable loop |
| README plugin split (`HeroPlugin`, etc.) | Not reflected in crate layout (acceptable for MVP; optional cleanup later) |

---

### Priority A — Skill loadout (player-facing core)

**Problem:** The design promises “configure the hero” with unlocked slots and a six-skill pool; today slots unlock in meta but assignments are not editable in-game.

**Files (typical):**
- `src/app.rs` — e.g. `ChangeHeroSkill { slot: usize, skill: Option<SkillId> }` or `EquipSkill` / `ClearSkillSlot`; validate with `HeroProfile::equip_skill`; save after change.
- `src/save.rs` — no schema change if using existing `HeroProfile` fields.
- `src/ui/mod.rs` / `src/ui/mockup_layout.rs` — interactive skill row: click slot → picker (dropdown or modal) listing `SkillId::iter()` or static list; respect `unlocked_skill_slots`.
- Tests: app or domain integration test that event updates profile and persists (optional).

**Suggested order:**
1. Domain/event API + unit test (invalid slot, locked slot, duplicate skill policy — decide: allow duplicate or one-of-each).
2. Minimal UI: assign/clear for slot 0–1 on Build (and optionally Camp).
3. Save on change (same pattern as equip/salvage).

---

### Priority B — Combat reflects the skill roster (depth, not just labels)

**Problem:** `src/domain/combat.rs` `simulate_combat` only branches on Lifesteal; Guard, Heavy Strike, Poison, Thorn Skin, Barrier Pulse do not alter outcomes, so “build” choices barely matter.

**Approach (incremental):**
1. Inventory desired behaviors from `skill_definition` (triggers/tags).
2. Add focused tests per skill (similar to `lifesteal_skill_restores_health_on_attack`).
3. Extend `simulate_combat` in small steps: e.g. Guard → flat damage reduction on hit; Barrier → temporary shield at room start; Poison → DoT ticks; Thorn → reflect on hit taken; Heavy → trade attack speed or burst damage — **align with design doc** and keep ticks bounded.

**Files:** `src/domain/combat.rs`, possibly `src/domain/skills.rs` if shared helpers.

---

### Priority C — Gold gain upgrade does something

**Problem:** Players can buy `GoldGain` but run gold may ignore it.

**Options:**
- Pass `meta` or a `gold_multiplier: f32` into `simulate_run_with_playback` / `RunConfig`, or
- Apply multiplier only when merging rewards in `accept_run_rewards` (document which matches UI copy).

**Files:** `src/domain/run.rs`, `src/app.rs` (`start_run` / reward acceptance), tests for deterministic scaling with upgrade level.

---

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

1. **MVP “feels playable”:** Priority A + C + README (skill choices + gold upgrade meaning + honest docs).
2. **MVP “matches fantasy”:** Priority B (at least 3–4 skills materially change fights or logs).
3. **MVP “polished shell”:** Priority D.

---

## Verification

After each slice: `cargo fmt --check`, `cargo test`, `cargo check`, manual run: new game / existing save, assign skills, verify combat log and summary change as expected.
