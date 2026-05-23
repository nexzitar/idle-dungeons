# UI Design Convergence — Implementation Plan (2026-05-21)

**Design:** [`docs/superpowers/specs/2026-05-21-ui-design-convergence-design.md`](../specs/2026-05-21-ui-design-convergence-design.md)  
**Law:** [`docs/ui-design-system.md`](../../ui-design-system.md)  
**Status:** Plan only — **no production code** until kickoff commit.

**Prerequisites:** UI foundation Phases 1–6; Buildcraft sheet Phase 1 (party workspace, inspect panel, session).

---

## Program ethos

Each phase is **one cohesive commit** (or two if primitive + migration must split). Always:

```bash
cargo test
```

Manual smoke: affected screen(s) + open/close modals + verify underlying buttons blocked.

**Do not** batch unrelated screens in one PR.

---

## Revised roadmap (approved order)

| Step | Phase | Theme |
|------|-------|--------|
| 1 | **0** | Theme / tokens / design system doc |
| 2 | **1** | `mounted_panel` + `framed_section_header` |
| 3 | **2** | `loadout_row` everywhere |
| 4 | **3** | **`hero_identity_card`** |
| 5 | **4** | Build screen convergence |
| 6 | **5** | Inspect ecosystem (policy + migrations) |
| 7 | **6** | Gear hub |
| 8 | **7** | Summary rewards |
| 9 | **8** | Combat theater |
| 10 | **9** | Motion / overlay hooks |
| 11 | **10** | Skill shop |

**Rationale for hero card at step 4:** Emotional anchor removes dev-tool feel before full build column refactor; reusable across later phases.

**Inspect ecosystem at step 5:** Cross-cutting — apply while build screen is fresh; gear/summary inherit policy.

---

## Phase 0 — Theme, tokens, design system

**Goal:** Single source of truth; no visual change required yet.

### Task 0.1: Publish design system

**Files:** `docs/ui-design-system.md` (created)

- [x] Spacing, icon tiers, borders, panel recipes, typography
- [x] Density zones, category map, inspect + tooltip rules
- [x] Motion, fake-lit UI, layering, combat sequencing (future)
- [x] PR checklist

### Task 0.2: Theme presets

**Files:** `src/ui/theme.rs`

- [ ] `UiDensity` enum + gutter/icon accessors (Buildcraft, Camp, Combat, Gear, Summary)
- [ ] `UiIconSize` constants (72, 52, 40, 28) — document deprecation of 22
- [ ] `UiFrame` helpers or `mounted_panel_styles()` returning `(bg, border, padding)`
- [ ] `category_display_family(SkillCategory) -> DisplayFamily` in `skill_presentation.rs`
- [ ] `accent_for_display_family()` aligned with design system §11
- [ ] Unit tests for mapping stability

### Task 0.3: Cross-link docs

**Files:** `docs/visual-bible-foundation-v1.md`, `docs/design-philosophy.md`, `README.md` (one line)

- [ ] Link to `ui-design-system.md` as UI canonical doc

**Verify:** `cargo test` — no regressions.

---

## Phase 1 — Mounted panel + section header

**Goal:** Dogfood buildcraft; extract recipes without changing player-visible layout (or minimal).

### Task 1.1: `mounted_panel` primitive

**Files:** `src/ui/primitives/panel.rs` (extend), `primitives/mod.rs`

- [ ] `MountedPanelStyle { Recessed, Deep, OrnatePrimary }`
- [ ] `spawn_mounted_panel(parent, style) -> Entity`
- [ ] Uses `UiDensity` + design system recipes

### Task 1.2: `framed_section_header`

**Files:** `src/ui/primitives/section.rs` (new) or extend `text.rs`

- [ ] `spawn_framed_section_header(parent, title, optional: subtitle)`
- [ ] Replaces ad-hoc `section_title` + spacer patterns

### Task 1.3: Refactor buildcraft to use primitives

**Files:** `buildcraft/party_column.rs`, `buildcraft/library.rs`, `buildcraft/sheet.rs`

- [ ] Party column → `mounted_panel(OrnatePrimary)`
- [ ] Library frame → `mounted_panel(Recessed)`
- [ ] Section headers → `framed_section_header`

**Verify:** Visual parity with current buildcraft; `cargo test`.

---

## Phase 2 — Loadout row everywhere

**Goal:** One combat-script row language; retire `skill_slot_row` implementation.

### Task 2.1: `loadout_row` primitive

**Files:** `src/ui/primitives/loadout.rs` (new)

- [ ] `LoadoutRowConfig { hero, slots, unlocked, focused, interactive, cell_px, show_indices }`
- [ ] `spawn_loadout_row` wraps hero label + `skill_bar`
- [ ] Optional layering warning caption hook

### Task 2.2: Extract shell skill slots

**Files:** `src/ui/shell/skill_slots.rs` (new, from `layout.rs`), `shell/layout.rs`

- [ ] Move `skill_slot_row`, `skill_slot_placeholder_handle` → use `loadout_row` + `skill_presentation`
- [ ] Interactive build: open buildcraft on slot click (unchanged behavior)
- [ ] Summary: read-only row (`interactive: false`)
- [ ] Remove 22px icon + text chip layout
- [ ] Remove per-slot long `UiTooltip` (replace with one-line hint on row or defer to Phase 5)

### Task 2.3: Sync buildcraft party column

**Files:** `buildcraft/party_column.rs`

- [ ] Use shared `loadout_row` if not already via thin wrapper

**Verify:** Build + summary hero slots match buildcraft bar visually; open buildcraft from build slot.

---

## Phase 3 — Hero identity card

**Goal:** Emotional anchor primitive; deploy on build screen hero column first.

### Task 3.1: `hero_identity_card` primitive

**Files:** `src/ui/primitives/hero_card.rs` (new)

- [ ] `HeroIdentityConfig { name, subtitle, portrait: Handle<Image>, tint, show_stat_strip }`
- [ ] Portrait frame 64–80px, deep mounted panel
- [ ] Future-hook components/markers (stance, relationship) — empty in v1
- [ ] `HeroIdentityCard` component for sync systems later

### Task 3.2: Build screen — hero column header

**Files:** `shell/layout.rs` or `shell/hero_column.rs`, `screens/build.rs`

- [ ] Replace plain name row with `hero_identity_card` for P1 (+ P2 when unlocked)
- [ ] Subtitle: class/role from domain (existing labels)
- [ ] Subtle warm edge gradient optional (fake-lit v1)

### Task 3.3: Summary screen — hero column header

**Files:** `screens/summary.rs`, shared hero column spawn

- [ ] Same card, read-only variant

**Verify:** Build + summary feel anchored; partner slot locked state graceful.

---

## Phase 4 — Build screen convergence

**Goal:** Full camp build column matches buildcraft language.

### Task 4.1: Retire `build_panel_text` wall

**Files:** `build_panel.rs`, `shell/layout.rs`

- [ ] Stats as compact chip row (HP/DMG/ARM) under identity card
- [ ] Layering warnings as gold caption (buildcraft pattern)
- [ ] Keep domain text helpers for tests or move to structured formatters

### Task 4.2: Column framing

**Files:** `shell/layout.rs`

- [ ] Hero column: ornate or deep mounted panel wrapping identity + loadout
- [ ] Briefing column: recessed panel consistency
- [ ] Spacing: `UiDensity::Camp` gutters

### Task 4.3: Footer dock

**Files:** `shell/footer.rs`

- [ ] Skill shop / gear / start run → `spawn_button` Primary / PanelOutlined
- [ ] Demote long tooltips to one-line hints

### Task 4.4: Extract `hero_column.rs`

**Files:** `shell/hero_column.rs`, slim `layout.rs`

- [ ] Shared spawn for build + summary hero column

**Verify:** Full build screen smoke; start run; open gear/buildcraft.

---

## Phase 5 — Inspect ecosystem

**Goal:** Enforce global tooltip rule; fixed regions on camp screens.

### Task 5.1: Inspect policy tooling

**Files:** `docs/ui-design-system.md` (done), optional `ui/inspect.rs` module

- [ ] `InspectRegion` marker component
- [ ] Shared sync pattern doc in module comment (mirror buildcraft `sync.rs`)

### Task 5.2: Build screen inspect strip

**Files:** `shell/hero_column.rs`, `primitives/inspect_panel.rs`

- [ ] `spawn_inspect_panel_compact` (~120–160px height)
- [ ] Hover/focus on loadout slot updates inspect (reuse `InspectPanelContent` builders)
- [ ] Remove remaining skill slot description tooltips

### Task 5.3: Tooltip audit

**Files:** `shell/layout.rs`, `footer.rs`, `gear_hub.rs`, …

- [ ] Grep `UiTooltip` — shorten or remove per policy
- [ ] CHANGELOG note: tooltip policy migration

**Verify:** No `\n` in tooltip strings on build screen; inspect shows skill detail.

---

## Phase 6 — Gear hub

**Goal:** Icon-first items + fixed stash inspect.

### Task 6.1: `rarity_frame` + item card v2

**Files:** `primitives/card.rs`, optional `icon_frame.rs`

- [ ] Icon-first header (larger placeholder gear art)
- [ ] Rarity border via shared helper
- [ ] Actions secondary (smaller buttons)

### Task 6.2: Stash inspect panel

**Files:** `gear_hub.rs`

- [ ] Fixed inspect at bottom of stash column
- [ ] Hover/selection on item card updates inspect (affixes, stats)

### Task 6.3: Loadout header

**Files:** `gear_hub.rs`

- [ ] `hero_identity_card` above equipped scroll

**Verify:** Gear hub equip/salvage; inspect shows item detail without cursor tooltip.

---

## Phase 7 — Summary rewards

**Goal:** Celebratory density zone.

### Task 7.1: `reward_card`

**Files:** `primitives/reward_card.rs`

- [ ] Larger icon tier (72), rarity glow, “NEW” optional caption
- [ ] Used in summary loot grid + rewards modal

### Task 7.2: Outcome hierarchy

**Files:** `summary_panel.rs`, `shell/layout.rs` (dungeon summary column)

- [ ] Headline outcome → treasure stat row → loot grid
- [ ] Chronicle in recessed scroll panel

### Task 7.3: Rewards modal chrome

**Files:** `shell/footer.rs`

- [ ] Align with `spawn_modal_shell` + buildcraft framing

**Verify:** Summary flow accept rewards; loot readable.

---

## Phase 8 — Combat theater

**Goal:** Shared bars + combat script row scaffold.

### Task 8.1: Bar preset unification

**Files:** `playback_bars.rs`, `primitives/bar.rs`

- [ ] Enemy bar → `spawn_horizontal_bar`
- [ ] Cast/CD/GCD preset styles in `UiBarStyle` (height, color tokens)
- [ ] Preserve all `Playback*Fill` marker components

### Task 8.2: Compact skill row (static)

**Files:** `shell/theater.rs`, `skill_bar.rs`

- [ ] 40px `skill_bar` under party portraits
- [ ] Overlays hidden; hooks present

### Task 8.3: Document sequencing visuals

**Files:** `docs/ui-design-system.md` (already §12)

- [ ] Add “Combat UX backlog” stub in `docs/superpowers/ACTIVE-REMAINING-WORK.md` if exists

**Verify:** Running playback sync tests; bars update; no marker regression.

---

## Phase 9 — Motion / overlay hooks

**Goal:** Subtle, optional; no layout motion.

### Task 9.1: Selection glow

**Files:** `skill_icon.rs`

- [ ] Focused slot soft gold glow (border or overlay layer)

### Task 9.2: Inspect fade

**Files:** `buildcraft/sync.rs`, inspect sync pattern

- [ ] Debounced content update; optional 150ms alpha (if Bevy UI supports without layout cost)

### Task 9.3: Cooldown overlay (hidden default)

**Files:** `skill_icon.rs`, `playback_sync.rs` (stub)

- [ ] Wire `SkillIconOverlayState` from playback when ready — **no sweep art required**

**Defer:** Ember pulse, atmospheric breathing beyond title camp.

---

## Phase 10 — Skill shop

**Goal:** Lowest priority; align when other camp UI stable.

### Task 10.1: Modal shell migration

**Files:** `skill_shop.rs`

- [ ] `spawn_modal_shell` + canonical header/footer

### Task 10.2: Icon library grid

**Files:** `skill_shop.rs`

- [ ] Reuse library grid pattern from buildcraft (smaller scope)
- [ ] Fixed inspect for skill purchase detail

**Verify:** Buy skill flow; gold deduction.

---

## Risk register

| Risk | Level | Mitigation |
|------|-------|------------|
| **Visual fragmentation despite modularity** | **High** | Design system law; Phase 0 presets; PR checklist |
| `shell/layout.rs` churn | High | Extract `hero_column`, `skill_slots` per phase |
| Playback marker breakage | High | Preserve components; test running state |
| Tooltip removal user confusion | Medium | Inspect first, then strip tooltips |
| Hero card scope creep | Medium | v1 = portrait + title + subtitle only |
| Animation over-build | Medium | Phase 9 last; layout-first rule |
| Title camp scope bleed | Low | Explicit exclusion from this plan |

---

## Architectural concerns (pre-implementation)

1. **`shell/layout.rs` god file** — Must shrink during Phases 2–4. Do not add new spawn logic there; only delegate to `shell/hero_column.rs`, `shell/skill_slots.rs`, `shell/header.rs`.

2. **Inspect sync proliferation** — Risk of N copies of buildcraft `sync.rs`. Consider thin `ui/inspect/sync_inspect_panel.rs` shared helper in Phase 5 before gear inspect.

3. **Category two-layer model** — Domain `SkillCategory` vs display `DisplayFamily` must live only in `skill_presentation.rs`. Document in design system; test mapping.

4. **Density presets vs one-off modals** — Gear hub and skill shop use different modal origins; Phase 6/10 must converge on `modal.rs`, not fork.

5. **Tooltip layer z-order** — As inspect grows, tooltip layer should only show hints; consider `TooltipState` max length guard (optional Phase 5).

6. **Hero card + rename flow** — `HeroNameEditButton` must remain on card; identity primitive wraps existing rename behavior.

7. **Combat skill row + playback** — Static row in Phase 8 avoids sync complexity; dynamic overlays are Phase 9+ with explicit playback tasks.

8. **No screenshot CI** — Manual smoke checklist per phase remains necessary until visual regression tooling exists.

---

## Commit message convention

```
feat(ui): <phase> — <short outcome>

Examples:
feat(ui): phase 0 — design system tokens and UiDensity presets
feat(ui): phase 3 — hero identity card on build and summary
refactor(ui): phase 2 — loadout_row replaces text skill chips
```

---

## Kickoff checklist

Before Phase 0 code:

- [ ] User approved this plan (rev. 2026-05-21)
- [ ] `docs/ui-design-system.md` reviewed
- [ ] Branch strategy agreed (e.g. `feat/ui-design-convergence` off current feature branch)

**First code commit:** Phase 0 Task 0.2 (theme presets) — doc already landed in Task 0.1.
