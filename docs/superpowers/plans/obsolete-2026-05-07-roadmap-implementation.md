# Roadmap Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Execute the README roadmap: deepen combat/skill parity and clarity, make stash UI honest or functional, add run/world variety and briefing clarity, then presentation polish—without breaking the simulation-as-source-of-truth architecture.

**Architecture:** Keep `src/domain/*` as the authority for combat, loot, and dungeon outcomes; expose player intent through Bevy events in `src/app.rs`; reflect state in `src/ui/*`. Each vertical slice lands with `cargo test` green and small commits.

**Tech stack:** Rust, Bevy 0.14, serde JSON, existing `idle_dungeons` crate layout.

**Reality check (do not skip):** `src/domain/combat.rs` `simulate_combat` already applies Guard (flat reduction on incoming hits), Heavy Strike (bonus damage on hero swing), Poison (extra damage chunk on hero swing via `PoisonTick` events), Thorns (reflect after HP loss), Barrier (absorbs before HP), and Lifesteal. The roadmap is **not** “wire skills from zero”—it is **align behavior with `skill_definition` text**, add missing nuance (e.g. poison as damage-over-time vs burst), tune numbers, and prove behavior with tests.

---

## File map (expected touch points)

| Area | Primary files |
|------|----------------|
| Combat depth | `src/domain/combat.rs`, `src/domain/skills.rs`, `src/domain/hero.rs` (if skill hooks need data) |
| Stash honesty | `src/ui/mockup_layout.rs` (~stash caption), optionally `src/ui/mod.rs` / new `src/ui/inventory_panel.rs` for sort state |
| Run variety | `src/domain/dungeon.rs`, `src/domain/run.rs`, `src/app.rs`, `src/ui/mockup_layout.rs` (briefing strings) |
| Presentation | `src/ui/theme.rs`, `src/ui/mockup_layout.rs`, `src/domain/run.rs` (playback timing—careful not to desync tests) |
| Docs | `README.md`, `docs/superpowers/plans/2026-05-06-mvp-remaining-work.md` (mark stale items fixed) |

---

### Phase A — Combat vs skill catalog (deepen + prove)

**Outcome:** Every skill in `SkillId` has behavior that a player can notice and that matches `skill_definition(SkillId).description` within reasonable game-feel. Regressions caught by focused tests.

#### Task A.1: Inventory + spec alignment

**Files:**
- Read: `src/domain/skills.rs` (`SkillDefinition` per skill)
- Read: `src/domain/combat.rs` (`simulate_combat` body ~61–239)
- **`docs/superpowers/plans/2026-05-07-skill-combat-catalog-mapping.md`** — per-skill trigger vs simulation (maintain when combat changes)
- Modify: `docs/superpowers/plans/2026-05-06-mvp-remaining-work.md` (reality table + README link)

- [x] **Step 1:** For each `SkillId`, mapping: **trigger** vs **what code does today** — see catalog mapping doc.

- [x] **Step 2:** Mismatches resolved in Phase A.2–A.4; doc notes future copy-only transparency.

- [x] **Step 3:** Docs committed: `mvp-remaining-work.md` + `skill-combat-catalog-mapping.md` + README pointer.

```bash
git add docs/superpowers/plans/2026-05-06-mvp-remaining-work.md
git commit -m "docs: refresh combat coverage notes vs skill catalog"
```

#### Task A.2: Poison — optional burst vs true DoT (TDD)

**Files:**
- Modify: `src/domain/combat.rs`
- Test: `src/domain/combat.rs` `mod tests`

**Decision gate:** If poison should tick over time without extra hero swings, introduce a **poison stack or enemy poison HP** state inside `simulate_combat` loop (still bounded by `max_clock_ticks` and `MAX_EVENTS`).

- [x] **Step 1: Failing test — poison contributes without a second hero swing**

Implemented as `poison_deals_damage_across_clock_ticks` in `src/domain/combat.rs` (`mod tests`): slow hero (`attack_speed = 0.12`), `PoisonEdge` only, 12 clock ticks, asserts one `HeroAttacked` and at least two `PoisonTick` events.

- [x] **Step 2: Run test — expect FAIL** (done during TDD; now passes).

```bash
cargo test poison_deals_damage_across_clock_ticks -- --nocapture
```

- [x] **Step 3: Implement minimal poison model** in `simulate_combat`: `poison_stacks` on hero hit (+2, cap 40); end of each outer iteration applies one `PoisonTick` (`poison_tick` damage), decrements stack, respects `MAX_EVENTS`.

- [x] **Step 4: Run full suite** — PASS (`cargo test`).

- [x] **Step 5: Commit** — `feat(combat): poison ticks across clock iterations`

#### Task A.3: Guard / Heavy — scaling + readability

**Files:**
- Modify: `src/domain/combat.rs`

**Goal:** Tie flat Guard reduction to `healing_power` or armor stat (small, testable), and document Heavy tradeoff if design calls for **attack speed penalty** (add test that hero swing count drops when Heavy equipped).

- [x] **Step 1:** Add test `guard_reduction_scales_with_healing_power` (or reuses existing test name if present — extend it) asserting higher `healing_power` lowers damage taken from a fixed enemy hit when Guard equipped.

- [x] **Step 2:** Implement: replace hardcoded `3` in `enemy_damage = (enemy_damage - 3).max(1)` with formula using `stats.healing_power` (floor/clamp so minimum 1 damage still possible).

- [x] **Step 3:** Optional: If Heavy gets ASPD tradeoff, adjust `hero_as` when `has_heavy` and add test on swing count or time-to-kill. (`hero_as *= 0.75` then `.max(0.12)`; test `heavy_strike_slows_attack_pacing`.)

- [x] **Step 4:** `cargo test && cargo fmt`

- [x] **Step 5: Commit** `feat(combat): tune guard scaling (+ optional heavy ASPD tradeoff)`

#### Task A.4: Barrier / Thorns — edge cases

**Files:**
- `src/domain/combat.rs`

- [x] **Step 1:** Tests: barrier fully absorbs lethal strike; thorns kill enemy after reflect; affix + skill synergy order documented in test names.

- [x] **Step 2:** Fix any order-of-operations bugs found. (None; `hp_loss == 0` correctly skips thorns.)

- [x] **Step 3:** Commit `test(combat): barrier and thorns edge cases`

---

### Phase B — Stash honesty (quick win vs real feature)

Pick **one** path; do not leave misleading UI.

#### Task B.1 (Path 1 — recommended minimal): Honest copy

**Superseded for filter line by B.2:** stash column now shows `Stash filters: —` plus a real sort control. Remaining “filters” work is still future scope.

- [x] **Step 1–3:** No further copy required; stash sort (B.2) superseded filter line. **Done / N/A.**

#### Task B.2 (Path 2 — functional minimal): Sort only

**Files:**
- Modify: `src/ui/mockup_layout.rs` / `src/ui/mod.rs`
- Optional new: small `StashSortOrder` resource + toggle control

- [x] **Step 1:** Add resource `StashSort: enum { Recent, RarityName }` in `src/ui/mod.rs` or `components.rs`.

Implemented as `StashSortOrder` in `src/save.rs` on `SaveProfile` (`Recent` = reverse profile vec order / newest appended last; `RarityName` = rare first, name A–Z, `id` tie-break). Session `Resource` removed; UI reads `profile.profile.stash_sort`, toggles persist via `save_profile`.

- [x] **Step 2:** When spawning inventory rows OR in a sync system, sort `inventory: &[ItemInstance]` by chosen key before `spawn_item_card` (define “recent” as **file order** or `id` if no timestamp — document in comment).

`stash_display_indices` used for Inventory (first 14), Loot tab, and non-interactive list rows.

- [x] **Step 3:** Add tiny UI control “Sort: ···” that toggles sort (only if you commit to Path 2 fully).

`StashSortCycleButton` in stash column; toggles order and rebuilds the active menu screen.

- [x] **Step 4:** Test: unit test sort helper in `src/domain/items` or `src/ui` with fake items.

Tests in `src/ui/stash_sort.rs`.

- [x] **Step 5:** Commit `feat(ui): stash sort by rarity/name` (preference persisted in `SaveProfile.stash_sort` as of follow-up save change).

---

### Phase C — Run / world variety + briefing clarity

#### Task C.1: Dungeon variety (domain-first)

**Files:**
- `src/domain/dungeon.rs`, `src/domain/run.rs`, tests under `src/domain/`

- [x] **Step 1:** Read `RoomKind` and generation; list which kinds appear at which depths today.

Documented in `generate_dungeon` rustdoc on `src/domain/dungeon.rs`.

- [x] **Step 2:** Add **one** new encounter pattern or weight tweak with deterministic seed test (e.g. depth 10 always sees X under seed Y unless design forbids).

Depth **11** is fixed **Treasure** when `seed % 97 == 11` (e.g. seed **108**). Test: `seeded_depth_11_treasure_when_seed_mod_97_eq_11`.

- [x] **Step 3:** **`RunPlaybackFrame.risk_hint`** (per-room label from `room_risk_hint` in `src/domain/dungeon.rs`) on playback “Type” line; **`RunSummary.peak_risk_note`** (max danger seen) on summary panel + text report. Integration test: `tests/simulation_mvp.rs`.

- [x] **Step 4:** Commit `feat(dungeon): tweak room table + test`

#### Task C.2: Briefing UI

**Files:**
- `src/ui/mockup_layout.rs` (`spawn_dungeon_briefing_column`)

- [x] **Step 1:** Pull next-run info from `ProfileState` / default depth cap strings (e.g. `DEFAULT_RUN_MAX_DEPTH`).

Uses `DEFAULT_RUN_MAX_DEPTH` and `DEFAULT_RUN_SEED` from `run.rs` (single source with Start button).

- [x] **Step 2:** Show “Target depth”, “Boss at depth N”, “Seed (if fixed)” consistently.

Briefing column captions updated; seed labeled as MVP fixed value until picker exists.

- [x] **Step 3:** Commit `feat(ui): clearer briefing stats`

---

### Phase D — Presentation (incremental)

#### Task D.1: Typography + spacing pass (no new assets)

**Files:**
- `src/ui/theme.rs`, `src/ui/mockup_layout.rs`

- [x] **Step 1:** Normalize font sizes / panel padding against a short style table in `theme.rs` comments.

- [x] **Step 2:** Subtle color contrast fixes for `body_dim` vs backgrounds (manual playtest).

- [x] **Step 3:** Commit `style(ui): theme consistency pass`

#### Task D.2: Playback motion (optional)

**Status:** **Deferred.** Bars and log advance on fixed `PLAYBACK_STEP_SECS` in `src/app.rs`; adding interpolation would require smoothing state in `UiPlugin` and was skipped to keep playback timing simple and tests stable.

- [ ] **Step 1:** If adding interpolation, keep **simulation indices** authoritative; only ease displayed bars/text.

- [ ] **Step 2:** Do not break `cargo test` timing assumptions—prefer visual-only changes.

---

### Phase E — Longer term (separate future plans)

Defer until Phases A–D feel good in playtests:

- Multi-hero / party systems (new spec + migration story for saves)
- Non-linear dungeon graph / branching routes
- Prestige / meta currencies beyond gold & salvage

**Do not start Phase E inside this plan’s execution batch.**

---

## Self-review (plan author)

1. **Spec coverage:** README bullets mapped — A=combat depth, B=stash, C=variety+briefing, D=presentation, E=deferred.
2. **Placeholders:** No `TBD`; poison DoT work landed with real tests (no `todo!` scaffolds).
3. **Consistency:** Task A.2 state variables must match `CombatPlaybackFrame` / `combat_playback_frames_from_result` (update if new event ordering changes HP snapshots).
