# Skillbook & Buildcraft UX — Design Spec (2026-05-20, rev. 2)

## Status

**Design approved (rev. 2 — party workspace pivot).** No production implementation yet. Execute via [`docs/superpowers/plans/2026-05-20-skillbook-buildcraft-ux.md`](../plans/2026-05-20-skillbook-buildcraft-ux.md).

Builds on **UI foundation extraction** (`ui/primitives`, `ui/interaction`, `ui/shell`). This is a **presentation-layer** redesign; combat truth stays in `domain/` except where slot-order priority is explicitly extended (see §6).

---

## 0. Identity pillar

Delvers buildcraft should feel like **assembling a combat rotation**, not equipping passive RPG perks.

- **Party loadouts** are the primary object being edited.
- The **skill library** supports discovery and assignment.
- **Slot position (left → right)** is strategic: when multiple skills are available under the shared ability GCD, **lower-index slots fire first** (see §6).
- UI must be **tactical, stable, and readable** — not spreadsheet-dense, not addon-noisy.

---

## 1. Problem statement

### Player experience gaps

| Issue | Today | Target |
|-------|--------|--------|
| Build identity | Single-hero modal list; slots live elsewhere | **Party composition workspace** — all hero bars visible while browsing |
| Recognition | 22px placeholders + text rows | Large icons; category color language |
| Tooltips | Cursor-following; blocks content | **Fixed inspect panel** on library side (lower-right) |
| Composition | Immediate `AssignHeroSkill` on pick | **Edit session** → Apply / Cancel |
| Tactical clarity | Slot order invisible / cosmetic | **Numbered left→right bars**; order = combat priority |
| Party | One sheet at a time | **Lead + partner** loadout rows always visible |

### Codebase gaps

- `skill_book.rs`: text list modal, per-row `UiTooltip`, immediate commit.
- `shell/layout.rs`: duplicate slot chip UI; opens book for one hero.
- No `skill_icon` / `party_loadout_column` primitives; no cooldown overlay layering.
- Domain: `assign_skill_to_slot` already clears duplicates **within one hero**; cross-hero duplicate is allowed. Slot-order ability priority when multiple ready may need explicit domain work (§6).

### Non-goals (this program)

- Cooldown shaders / radial sweep **implementation** (architecture hooks only)
- Drag/drop in Phase 1 (deprioritized below readability)
- Per-skill PNG art (Phase 1 uses category placeholders)
- Saved loadout presets / weapon-swap bars (future §16)
- Rewriting full combat sim unrelated to slot priority

---

## 2. Design principles

Aligned with [`docs/design-philosophy.md`](../../design-philosophy.md) and [`docs/visual-bible-foundation-v1.md`](../../visual-bible-foundation-v1.md):

1. **Party first** — left column loadouts are the visual anchor; library is secondary.
2. **Readability over density** — curated grid, generous gutters; no MMO inventory cramming.
3. **Stable layout** — bars do not resize, reflow, or shift on hover; only overlays/highlights animate.
4. **Slot order is gameplay** — position communicates rotation priority; show indices subtly.
5. **Per-hero duplicate rule** — same skill twice on one hero forbidden; two heroes may share a skill.
6. **Presentation-only edit** — session mutates pending loadouts until Apply.
7. **Reuse primitives** — `skill_icon`, `skill_bar`, `inspect_panel`, thin `buildcraft/` modules.

**We want:** WoW spellbook *discovery* + action-bar *tactical clarity*, without WoW clutter.

**We reject:** floating tooltip spam, one-hero-only editing, cosmetic slot strips, hover-driven layout motion.

---

## 3. Approaches considered

| Approach | Summary | Verdict |
|----------|---------|---------|
| **A** Incremental list → grid in old modal | Low effort | **Rejected** — still single-hero, library-first |
| **B** Library + one focused loadout bar | Previous rev. 1 | **Superseded** |
| **C** Party workspace (recommended) | Left: party bars; right: library + inspect | **Approved** |
| **D** Inline camp-only editing | No sheet | **Rejected** — cramped; weak focus |

---

## 4. UX hierarchy (strict)

When the player’s eye tracks the Buildcraft sheet:

1. **Party loadouts** (left) — who brings what rotation
2. **Focused slot** — which cell is being edited (ring + slot index)
3. **Skill library** (right) — browse / pick
4. **Inspect panel** — details for hover/focus target

The library is **supporting material**, not the centerpiece.

---

## 5. Layout specification

### 5.1 Sheet structure (~full overlay)

| Region | Width | Role |
|--------|-------|------|
| **Party column** | **35–40%** | Persistent loadout workspace |
| **Library column** | **60–65%** | Scrollable icon collection + anchored inspect |
| **Header** | Full width | Title, party layering warnings, Apply/Cancel |
| **Footer** | Full width optional | Apply/Cancel duplicate if header crowded |

Minimum sheet width ~**960px**; below that, scale icon cells down slightly before crushing party column.

### 5.2 Party column (left, always visible)

For each active party member (lead always; partner when unlocked):

```
┌─ Player 1 ──────────────── [portrait?] Name ─┐
│  1    2    3    4    5    6                  │
│ [⚔][🛡][ +][░░][░░][░░]   ← fixed 6 cells    │
└──────────────────────────────────────────────┘

┌─ Player 2 ───────────────────────────────────┐
│  1    2    3    4    5    6                  │
│ [↯][ +][░░][░░][░░][░░]                      │
└──────────────────────────────────────────────┘
```

| Element | Rule |
|---------|------|
| Slots | **Always 6 cells** per row (max equip capacity); `unlocked_skill_slots` gates interactable cells |
| Locked cells | Muted frame, lock glyph, non-interactive; still occupy space (no layout shift) |
| Empty unlocked | `+` or ember dot; invites assignment |
| Filled | `skill_icon` with category frame |
| Slot index | Small `1–6` under or in corner — reinforces order |
| Focus | Gold ring on `(hero, slot)`; only one focused cell sheet-wide |
| Header | Hero name + role label; optional 32px portrait later |

**Partner absent:** second row hidden or collapsed to single “locked” strip — never remove the 6-slot grid model.

### 5.3 Library column (right, visually dominant)

| Element | Rule |
|---------|------|
| Grid | **3–4 columns** (not 6+); cell **72–88px** including frame; **12–16px** gutter |
| Scroll | Vertical only; library area takes remaining height above inspect |
| Clear tile | Assigns `None` to focused slot |
| Curated | Show `skill_book_pick_order_for(unlocked)` — no “show 40 icons” density |
| Equipped hint | Badge “on P1” / “on P2” if skill appears on **that** hero’s pending bar (not global hide) |
| Filters | Phase 2+ chips; Phase 1 optional Active/Passive section labels |

Library must feel **spacious and intentional** — fewer visible icons with breathing room beats maximal density.

### 5.4 Inspect panel (library side, fixed)

| Property | Value |
|----------|--------|
| Position | **Lower-right** of library column, **inside** the 60–65% region |
| Size | Fixed e.g. **300×220px** — no auto-height jump |
| Behavior | Content swaps on hover/focus; panel frame **never moves** |
| Empty | “Select a skill or slot” |

**Content stack:** icon 48px → name → kind + category chip → tags → description (clamped) → synergy → (future) GCD/charges/position hint (“Slot 2 — fires before slot 4 when both ready”).

**No `UiTooltip`** on library or loadout cells inside the sheet.

### 5.5 ASCII wireframe (rev. 2)

```
┌────────────────────────────────────────────────────────────────────────────────┐
│  Party Buildcraft                    [ Layering warnings per hero ]  [Apply][X]│
├──────────────────────────────┬─────────────────────────────────────────────────┤
│  PARTY (35–40%)              │  LIBRARY (60–65%)                               │
│                              │  ┌────┬────┬────┬────┐                        │
│  Player 1 — Aldric           │  │ ⚔  │ 🛡 │ ↯  │ ·  │   spacious grid        │
│  1   2   3   4   5   6      │  ├────┼────┼────┼────┤                        │
│ [■][■][*+][░][░][░]          │  │    │    │    │    │   ← scroll             │
│                              │  └────┴────┴────┴────┘                        │
│  Player 2 — Mira             │                    ┌─────────────────────────┐  │
│  1   2   3   4   5   6      │                    │ Inspect (fixed)         │  │
│ [■][+][░][░][░][░]          │                    │ [icon] Heavy Strike     │  │
│                              │                    │ Slot 3 · fires before 5 │  │
│  * = focused slot            │                    │ …                       │  │
│                              │                    └─────────────────────────┘  │
└──────────────────────────────┴─────────────────────────────────────────────────┘
```

### 5.6 Camp hero column (launcher)

Build screen hero panels keep compact read-only bars (or icons). Clicking a slot opens Buildcraft with focus `(that hero, that slot)`. Sheet shows **full party**, not only clicked hero.

---

## 6. Skill rules

### 6.1 Duplicate equipment

| Scope | Rule |
|-------|------|
| **Within one hero** | Same `SkillId` **cannot** occupy two slots. Assigning to slot *j* removes that id from other slots on **that hero** in `pending` and on commit. |
| **Across heroes** | Player 1 and Player 2 **may** both equip Guard (or any same skill). No cross-hero eviction. |

**Session:** `BuildcraftEditSession` holds **per-hero** `pending: [Option<SkillId>; 6]`. `assign_to_focused(hero, skill)` scans only that hero’s array.

**Domain:** Matches `HeroProfile::assign_skill_to_slot` today (`hero.rs` clears duplicate on same hero only). Commit calls assign per changed `(hero, slot)`.

**Tests required:** cross-hero same skill allowed; intra-hero duplicate evicted in pending preview and commit.

### 6.2 Slot order = combat priority (design contract)

**Player-facing rule:** Loadout bar order is **left → right** = **earlier → later** in ability priority when multiple skills are off cooldown and eligible under the **shared ability GCD**.

**Implications:**

- Reordering slots (Phase 2 drag) changes combat behavior — UI must make order legible.
- Inspect panel should eventually state priority relative to neighbors.
- Empty slots are skipped; locked slots never fire.

**Domain / combat note:** Today combat resolves weapon/skill logic through hero build and timing pulses; **explicit left-to-right multi-skill GCD priority** should be treated as a **design contract** to implement or verify in `domain/combat.rs` (may require a dedicated scheduling pass over `equipped_skills` indices). UI and docs must not imply order matters until domain implements it — **Phase 1 plan includes a domain audit + test** for slot-index priority when multiple actives ready.

**UI obligation (immediate):** Never shuffle slot positions on assign; display stable indices; Phase 2 reorder = swap indices in `pending` array.

---

## 7. Interaction model

### 7.1 Open

1. Click `SkillSlotButton` on camp (or future entry point).
2. `OpenSkillBook { slot, kind }` → spawn Buildcraft sheet.
3. Session snapshots **all party heroes** present in profile (lead + partner if any).
4. Focus ← event’s `(kind, slot)`.

### 7.2 Edit (Phase 1 — click)

| Action | Effect |
|--------|--------|
| Click party slot `(H, i)` | `focused = (H, i)`; ring slot; inspect shows slot contents |
| Click library skill `S` | `pending[H][i] = Some(S)` with **intra-hero** duplicate eviction |
| Click clear tile | `pending[H][i] = None` |
| Hover library / slot | Updates `hover_inspect` only |

### 7.3 Apply / Cancel

| Control | Effect |
|---------|--------|
| Cancel / backdrop | Drop session; despawn |
| Apply | For each hero, diff `pending` vs `snapshot`; `assign_skill_to_slot` per changed index; save; close |

### 7.4 Phase 2 — reorder (before drag-from-library)

- **Slot ↔ slot drag** within same hero bar swaps `pending[i]` / `pending[j]` — teaches rotation tuning.
- Library → slot drag comes **after** bar readability is proven.

### 7.5 `UiClickAction` (centralized)

`BuildcraftFocusSlot { hero, index }`, `BuildcraftPickSkill(SkillId)`, `BuildcraftClearSlot`, `BuildcraftApply`, `BuildcraftCancel`, `BuildcraftClose`.

---

## 8. `BuildcraftEditSession` (revised)

```rust
/// Per-hero editable loadout (fixed 6 slots).
pub struct HeroLoadoutEdit {
    pub hero: PartyHeroKind,
    pub unlocked: u8,
    pub snapshot: [Option<SkillId>; 6],
    pub pending: [Option<SkillId>; 6],
}

pub struct BuildcraftEditSession {
    pub lead: HeroLoadoutEdit,
    pub partner: Option<HeroLoadoutEdit>,
    pub focused: (PartyHeroKind, usize),
    pub hover_inspect: InspectTarget,
}

impl BuildcraftEditSession {
    pub fn loadout_mut(&mut self, hero: PartyHeroKind) -> &mut HeroLoadoutEdit;
    pub fn assign_to_focused(&mut self, skill: Option<SkillId>);
    // Evict `skill` from other slots on SAME hero only.
    pub fn is_dirty(&self) -> bool;
    pub fn commit(&self, profile: &mut SaveProfile) -> Result<(), _>;
}
```

No combat stats in session — assignments only.

---

## 9. Visual identity (`SkillPresentation`)

Phase 1: category-colored frames + placeholder handles (`UiPlaceholderImages` / per-category).

| Signal | Presentation |
|--------|----------------|
| Active / Passive | Frame shape |
| `SkillCategory` | Border accent (`skill_category_chip_colors`) |
| Attack / Reactive / Passive / Heal | Warm / blue / purple / green language (spec rev.1 table) |

**Recognition target:** Heavy Strike, Guard, Lifesteal Strike identifiable without reading name.

---

## 10. Cooldown overlay architecture (hooks only — no VFX yet)

`skill_icon` must be a **layered stack**, not a flat `ImageNode`, so combat and camp can reuse it later.

### 10.1 Entity layering (bottom → top)

```
SkillIconRoot
├── SkillIconArt          (base image)
├── SkillIconDimOverlay   (cooldown darken, alpha)
├── SkillIconRadialSweep  (future: material/mask; hidden in Ph.1)
├── SkillIconGcdSweep     (shared GCD wedge; hidden in Ph.1)
├── SkillIconChargeText   (optional countdown; hidden in Ph.1)
├── SkillIconProcGlow     (future; hidden)
└── SkillIconFrame        (category border — always on top)
```

### 10.2 Presentation state (sync-driven later)

```rust
pub struct SkillIconOverlayState {
    pub cooldown_frac: f32,      // 0 = ready, 1 = full CD
    pub gcd_frac: f32,
    pub ready_pulse: bool,
    pub disabled: bool,
    pub desaturate: bool,
}
```

**Systems (future):** `sync_skill_bar_overlays` in playback/build — **not Phase 1**.

### 10.3 Motion rules (loadout bars)

| Allowed | Forbidden |
|---------|-----------|
| Cooldown radial sweep | Bar width change on hover |
| Ready pulse (subtle) | Reflowing wrapped slots |
| Border focus ring | Icons jumping position |
| Dim/desaturate | Tooltip following cursor |

---

## 11. Architectural modules

```
src/ui/
  primitives/
    skill_icon.rs       # layered icon + SkillIconConfig
    skill_bar.rs        # fixed 6-cell horizontal bar
    inspect_panel.rs    # fixed inspect region
    drag_drop.rs        # Phase 2b — after reorder polish
  buildcraft/
    mod.rs
    sheet.rs            # two-column layout
    party_column.rs     # spawn all hero rows
    library.rs          # grid + scroll
    session.rs
    presentation.rs     # SkillId → art/colors
    sync.rs
  skill_book.rs         # thin shim → buildcraft
```

**Domain touch on Apply only** (+ optional **domain task** for slot-order GCD priority).

---

## 12. Phased rollout (summary)

| Phase | Focus |
|-------|--------|
| **1** | Party column + library + inspect + session + click assign + slot labels + overlay **spawn hooks** (hidden layers) |
| **1b** | Domain: verify/implement left→right ready-skill priority + test |
| **2a** | Slot reorder (swap) + filters |
| **2b** | Library drag + global tooltip suppression |
| **3** | Cooldown overlay sync in playback, per-skill art, presets |

Detail: implementation plan.

---

## 13. Migration from `skill_book.rs`

| Old | New |
|-----|-----|
| `spawn_skill_book_modal` | `buildcraft::spawn_sheet` |
| `spawn_pick_row` | removed |
| `SkillBookPick` immediate | session + Apply |
| `OpenSkillBook.kind` | sets initial `focused` hero only |
| Single-hero assumption | `party_column.rs` |

Enable `open_skill_book_from_events` in **release** when shipping.

---

## 14. Risks & tradeoffs

| Risk | Mitigation |
|------|------------|
| Domain slot priority not implemented yet | Phase 1b domain task; inspect copy qualified until verified |
| Two-hero sheet height | Scroll party column if needed; fixed row heights |
| Session size | Two `[Option; 6]` arrays — trivial |
| Player confuses cross-hero duplicate | Inspect + badge shows which heroes wear skill |
| Layout shift | Fixed 6 cells always |

---

## 15. Defer (unchanged / extended)

| Item | Phase |
|------|-------|
| Radial CD shaders | 3 |
| Drag library → slot | 2b |
| Search | 2+ |
| Keybind overlays | 3 |
| §16 expansion bars | 4+ |

---

## 16. Future expansion (architecture must not block)

Design-only — **do not implement** now. Structure party column as **stack of loadout bands**, each band a typed row:

| Future band type | Example | Notes |
|------------------|---------|-------|
| **Core rotation** | Current 6-slot bar | Default |
| **Weapon swap** | Second 6-slot bar, gated by meta | Shares hero header |
| **Stance / context** | 3–4 icon strip toggling modifiers | Mutually exclusive highlight |
| **Companion / summon** | 4-slot bar under owner | Linked to hero |
| **Encounter temp** | Gray-bordered 2-slot row | Cleared after run |

**Architectural hooks:**

- `PartyLoadoutColumn` spawns `Vec<LoadoutBandConfig>` not one hardcoded bar.
- `HeroLoadoutEdit` may become `HashMap<LoadoutBandId, [Option<SkillId>; N]>` later — Phase 1 uses `BandId::Core` only.
- `skill_bar` accepts `slot_count` (max 6) and `band_label`.
- Session commit iterates bands for that hero.

This prevents painting into a corner when adding weapon swap or stance bars.

---

## 17. Success criteria

- Both party rows visible while browsing library; focus moves between heroes/slots.
- Intra-hero duplicate blocked in pending; cross-hero duplicate allowed.
- Inspect panel fixed on library side; no layout jump.
- Slot indices 1–6 visible; order stable on assign.
- `skill_icon` spawns overlay child nodes (hidden) for future CD/GCD.
- `cargo test` green + session unit tests for duplicate rules.
- Domain test documents or implements left→right ready priority.

---

## References

- `src/ui/skill_book.rs`, `src/ui/shell/layout.rs` (`skill_slot_row`)
- `src/domain/hero.rs` (`assign_skill_to_slot`)
- `src/domain/skills.rs`, `src/ui/theme.rs`
- UI foundation spec: `2026-05-21-ui-foundation-extraction-design.md`
