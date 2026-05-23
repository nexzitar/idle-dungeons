# Skillbook & Buildcraft UX — Implementation Plan (2026-05-20, rev. 2)

**Design:** [`docs/superpowers/specs/2026-05-20-skillbook-buildcraft-ux-design.md`](../specs/2026-05-20-skillbook-buildcraft-ux-design.md) (party workspace pivot, approved)

**Status:** Plan only — **no production code** until implementation kickoff.

**Prerequisites:** UI foundation Phases 1–6 (`ui/primitives`, `ui/interaction`, `ui/shell`).

---

## Revised priority order (global)

Execute in this order across phases:

1. **Party loadout workspace layout** (left column, dual hero rows, fixed 6 slots)
2. **Stable inspect panel** (library side, lower-right, fixed size)
3. **Skill library visual identity** (spacious grid, `SkillPresentation`)
4. **Session-based assignment flow** (party-wide `BuildcraftEditSession`, Apply/Cancel)
5. **Readable slot ordering** (indices, stable positions, inspect copy)
6. **Cooldown overlay architecture hooks** (layered `skill_icon`, hidden overlays)
7. **Domain: slot-order GCD priority** (verify or implement + test)
8. **Drag/drop** (slot reorder first, then library → slot)
9. **Filters / tabs / polish**

Drag/drop is **explicitly below** visual readability, build comprehension, and tactical ordering clarity.

---

## Overview by phase

| Phase | Theme | Tasks |
|-------|--------|-------|
| **1a** | Party column + session + click assign | 1–8 |
| **1b** | Library + inspect + sync | 9–13 |
| **1c** | Overlay hooks + domain priority | 14–16 |
| **2a** | Slot reorder + filters | 17–19 |
| **2b** | Library drag + tooltip suppression | 20–22 |
| **3** | Cooldown sync, art, expansion prep | 23+ (future) |

Each milestone: `cargo test` + manual smoke (build screen → Buildcraft → Apply/Cancel).

---

## Phase 1a — Party workspace foundation

### Task 1: `SkillPresentation` metadata

**Files:** `src/ui/buildcraft/presentation.rs`

- [ ] `icon_for(SkillId) -> Handle<Image>` with category fallback
- [ ] `accent_for_category(SkillCategory) -> Color`
- [ ] Cover all `SkillId::ALL`

---

### Task 2: Layered `skill_icon` primitive (with CD hooks)

**Files:** `src/ui/primitives/skill_icon.rs`, `primitives/mod.rs`

- [ ] `SkillIconConfig { skill, size, selected, disabled, locked, show_slot_index }`
- [ ] Spawn **layer stack** per design spec §10 (art, dim, radial, gcd, charge text, proc, frame)
- [ ] Phase 1: child overlay nodes `Visibility::Hidden` or alpha 0
- [ ] Export `SkillIconOverlayState` type (no sync system yet)
- [ ] Marker component `SkillIconRoot` for future queries

**Do not:** implement shaders or animated sweeps.

---

### Task 3: `skill_bar` primitive (fixed 6 cells)

**Files:** `src/ui/primitives/skill_bar.rs`

- [ ] `SkillBarConfig { slot_count: 6, unlocked, interactive, focused_slot }`
- [ ] **Fixed width** per cell; locked/empty/filled states; **no flex-wrap**
- [ ] `SkillBarSlot { hero: PartyHeroKind, index }` marker
- [ ] Slot index label (1–6) under icon

---

### Task 4: `BuildcraftEditSession` (party-wide)

**Files:** `src/ui/buildcraft/session.rs`

- [ ] `HeroLoadoutEdit` × lead + optional partner
- [ ] `focused: (PartyHeroKind, usize)`
- [ ] `assign_to_focused(skill)` — **intra-hero duplicate eviction only**
- [ ] `is_dirty()`, `commit()` → per-hero `assign_skill_to_slot`
- [ ] `open_from_profile(profile, initial_focus)`

**Tests:**

- [ ] Same skill on P1 slot 0 and P1 slot 2 → slot 0 cleared when assigning to 2
- [ ] Same skill on P1 and P2 → both allowed
- [ ] `commit` matches domain

---

### Task 5: `party_column.rs`

**Files:** `src/ui/buildcraft/party_column.rs`

- [ ] `spawn_party_column(parent, session, ph)` — 35–40% width column
- [ ] Per hero: header (name) + one `skill_bar`
- [ ] Wire focus rings from `session.focused`
- [ ] Partner row omitted or disabled strip when no partner

---

### Task 6: Sheet shell — two-column layout

**Files:** `src/ui/buildcraft/sheet.rs`

- [ ] Modal backdrop + `BuildcraftRoot`
- [ ] Header: title “Party Buildcraft”, Apply/Cancel
- [ ] Row: `party_column` (left) | library placeholder (right)
- [ ] **No layout shift** on hover (fixed flex basis)

---

### Task 7: Interaction — focus + assign (no library yet)

**Files:** `interaction/registry.rs`, `dispatch.rs`

- [ ] `BuildcraftFocusSlot { hero, index }`
- [ ] `BuildcraftApply`, `BuildcraftCancel`, `BuildcraftClose`
- [ ] Dispatch mutates session only; Apply commits

---

### Task 8: Wire open flow + deprecate list modal

**Files:** `systems.rs`, `skill_book.rs`

- [ ] `open_skill_book_from_events` → `spawn_buildcraft_sheet`
- [ ] Init session from full party + `OpenSkillBook { kind, slot }` focus
- [ ] Remove `spawn_pick_row` / list UI
- [ ] **Remove `#[cfg(debug_assertions)]` gate** on open handler when ready for release

**Manual:** open sheet, click slots on P1/P2, Cancel closes without save.

```bash
git commit -m "feat(ui): buildcraft party column and edit session"
```

---

## Phase 1b — Library + inspect

### Task 9: `inspect_panel` primitive

**Files:** `src/ui/primitives/inspect_panel.rs`

- [ ] Fixed size panel; markers for icon/title/meta/body/synergy
- [ ] `inspect_content_for_skill`, `inspect_content_for_slot(hero, index, skill)`

---

### Task 10: Library grid (spacious)

**Files:** `src/ui/buildcraft/library.rs`

- [ ] Right column ~60–65%; scroll viewport
- [ ] Grid **3–4 columns**, 72–88px cells, 12–16px gap
- [ ] Clear tile + unlocked skills via `skill_book_pick_order_for`
- [ ] `BuildcraftLibrarySkill(SkillId)` marker
- [ ] Optional “equipped on P1” badge (same hero only)

**Do not:** squeeze 6+ columns or shrink below readability targets.

---

### Task 11: Integrate library + inspect into sheet

**Files:** `buildcraft/sheet.rs`

- [ ] Library column fills space **above** inspect panel
- [ ] Inspect **anchored bottom-right** inside library column
- [ ] `BuildcraftPickSkill`, `BuildcraftClearSlot` actions

---

### Task 12: Sync systems

**Files:** `buildcraft/sync.rs`, `ui/mod.rs`

- [ ] `sync_buildcraft_party_bars` — `pending` → icons per hero
- [ ] `sync_buildcraft_focus_ring` — focused slot highlight
- [ ] `sync_buildcraft_inspect` — hover/focus → panel text (include slot index + priority hint copy)
- [ ] `sync_buildcraft_apply_enabled`

Register while `BuildcraftRoot` exists.

---

### Task 13: Layering warnings in header

**Files:** `buildcraft/sheet.rs`

- [ ] Per-hero `skill_layering` warning from **pending** loadouts
- [ ] Compact caption in header (not modal spam)

**Manual:** pick skills, inspect updates, Apply saves both heroes, cross-hero duplicate works.

```bash
git commit -m "feat(ui): buildcraft library grid and inspect panel"
```

---

## Phase 1c — Overlay hooks + domain priority

### Task 14: Overlay spawn verification

**Files:** `skill_icon.rs`, docs

- [ ] Debug-only toggle to show dim overlay at 50% alpha for one frame (dev screenshot) — optional
- [ ] Document `SkillIconOverlayState` fields for playback team

---

### Task 15: Domain — left→right ability priority

**Files:** `src/domain/combat.rs` (or new `skill_priority.rs`), tests

- [ ] **Audit** current behavior when multiple actives ready under shared GCD
- [ ] **Implement** if missing: iterate `equipped_skills` indices `0..unlocked` left→right for eligible skill fire
- [ ] Unit test: two actives ready → lower index fires first
- [ ] Update CHANGELOG / design spec if behavior was already correct

**Blocks:** marketing copy in inspect panel claiming priority — enable after Task 15 passes.

---

### Task 16: Phase 1 completion

- [ ] `cargo test` full suite
- [ ] Manual smoke checklist (design spec §17)
- [ ] `CHANGELOG` Unreleased entry
- [ ] Camp `skill_slot_row` still opens sheet (unchanged launcher OK for Phase 1)

```bash
git commit -m "feat(ui): buildcraft phase 1 complete with slot priority"
```

---

## Phase 2a — Reorder + filters (before library drag)

### Task 17: Slot ↔ slot reorder

**Files:** `buildcraft/session.rs`, `party_column.rs`, minimal pointer handling or click-swap fallback

- [ ] Swap `pending[hero][i/j]` — **preferred first drag milestone**
- [ ] UI: drag handle or “move left/right” — choose simplest shippable
- [ ] Inspect shows new order indices

**Validates:** rotation tuning without library drag complexity.

---

### Task 18: Category filter chips

**Files:** `buildcraft/library.rs`, `session.rs`

- [ ] `LibraryFilter` in session; visibility toggle on grid cells (avoid respawn)

---

### Task 19: Camp column read-only `skill_bar`

**Files:** `shell/layout.rs`

- [ ] Replace `skill_slot_row` internals with `skill_bar` (interactive=false)
- [ ] Click opens Buildcraft with focus

```bash
git commit -m "feat(ui): buildcraft slot reorder and filters"
```

---

## Phase 2b — Drag/drop (lower priority)

### Task 20: `drag_drop` primitive scaffold

**Files:** `src/ui/primitives/drag_drop.rs`

- [ ] Ghost icon, drag threshold, drop highlight
- [ ] Does not mutate profile — session only

---

### Task 21: Library → slot drag

**Files:** `library.rs`, `party_column.rs`

- [ ] Drop on `SkillBarSlot` assigns to `(hero, index)`
- [ ] Suppress click if drag consumed gesture

---

### Task 22: Tooltip suppression + Phase 2 QA

- [ ] No `UiTooltip` on sheet cells
- [ ] Optional: hide global `TooltipLayer` while `BuildcraftRoot` active

```bash
git commit -m "feat(ui): buildcraft drag-drop"
```

---

## Phase 3 — Future (outline only)

| Task | Item |
|------|------|
| 23 | `sync_skill_bar_overlays` during `GameState::Running` |
| 24 | Per-skill PNG manifest |
| 25 | `LoadoutBandId` second bar (weapon swap) — needs domain spec |
| 26 | Saved loadout presets |

---

## Migration checklist

| Current | Destination |
|---------|-------------|
| `spawn_skill_book_modal` | `buildcraft::sheet::spawn_buildcraft_sheet` |
| `spawn_pick_row` | deleted |
| `SkillBookPickButton` | `BuildcraftLibrarySkill` + session |
| `SkillBookRoot` | `BuildcraftRoot` |
| Single-hero session (rev.1 plan) | `HeroLoadoutEdit` × 2 |
| `OpenSkillBook.kind` | initial `focused.0` only |

---

## `UiPlugin` registration

```text
Update (after dispatch, when BuildcraftRoot exists):
  sync_buildcraft_party_bars
  sync_buildcraft_focus_ring
  sync_buildcraft_inspect
  sync_buildcraft_apply_enabled
```

---

## Testing strategy

| Layer | Coverage |
|-------|----------|
| **Session** | Intra-hero duplicate; cross-hero allow; dirty; commit |
| **Domain** | Slot-order priority when multi-ready (Task 15) |
| **UI** | Open sheet → assign P2 slot → Apply → save persists |
| **Manual** | Party column always visible; inspect fixed; no bar resize on hover |

---

## What NOT to do in Phase 1

- Drag/drop
- Cooldown animation / radial shaders
- Filters (optional section headers OK)
- Weapon swap / stance bands (§16 future)
- Shrinking icons to fit “more skills on screen”
- Cross-hero duplicate prevention
- Merging inspect into global cursor tooltip

---

## Risks (implementation)

| Risk | Mitigation |
|------|------------|
| Sheet too tall for 720p | Scroll party column; fixed row height |
| Domain priority lagging UI | Task 15 before priority marketing in inspect |
| Layered icon perf | 6–12 icons × ~5 hidden nodes — acceptable |
| `open_skill_book` debug-only today | Task 8 enables release |

---

## Execution handoff

1. ~~Design rev. 2 approval~~ — **done**.
2. Implement **Phase 1a → 1b → 1c** sequentially; do not start Phase 2b drag until 1c manual sign-off.
3. Treat **Task 15 (domain priority)** as part of Phase 1 completeness, not optional polish.

**No production code until implementation kickoff.**
