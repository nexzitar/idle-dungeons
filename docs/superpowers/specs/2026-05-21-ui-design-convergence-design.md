# UI Design Convergence — Design Spec (2026-05-21)

## Status

**Approved direction (rev. 1).** Strategic identity pass — not a cleanup chore.  
**Canonical reference:** Party Buildcraft sheet + [`docs/ui-design-system.md`](../../ui-design-system.md)  
**Implementation:** [`docs/superpowers/plans/2026-05-21-ui-design-convergence.md`](../plans/2026-05-21-ui-design-convergence.md) — **no code until plan kickoff.**

Builds on UI foundation extraction (Phases 1–6), completed Buildcraft Phase 1 sheet, and [`docs/visual-bible-foundation-v1.md`](../../visual-bible-foundation-v1.md).

---

## 0. Strategic framing

Delvers is entering **cohesive visual product** maturity:

| From | To |
|------|-----|
| Prototype / dev-tool UI | Crafted tactical dark-fantasy product |
| Per-screen one-offs | Shared interaction language |
| Tooltip-driven discovery | Stable inspect territories |
| Text-first loadouts | Icon-first combat script rows |
| Architectural modularity without visual governance | Module tree + **design system law** |

This program defines **Delvers’ visual identity, interaction language, and long-term UX philosophy** for all camp, build, gear, summary, and combat surfaces.

**Non-goals:** Gameplay redesign, domain changes, shaders, heavy VFX, title-camp presentation editor, skill PNG art pass.

---

## 1. Canonical direction (Buildcraft gold standard)

The Buildcraft sheet is the **reference implementation** for:

- Spacing and panel composition
- Icon sizing (52 bar / 72 library)
- Typography hierarchy
- Ornate framing (sparing gold)
- Fixed inspect panel structure
- Apply/Cancel session chrome
- Slot styling and category accents
- Overall breathable density

All future UI converges toward:

1. **Stable information regions**
2. **Mounted / recessed panels**
3. **Icon-first readability**
4. **Category-driven visual language**
5. **Atmospheric warmth**
6. **Deliberate spacing hierarchy**

---

## 2. First-class principles (elevated from implementation detail)

### 2.1 Stable inspect regions

**Major UX principle** — not an optional polish item.

We intentionally abandon cursor tooltip spam, overlapping descriptions, and hover chaos.

We adopt:

- Dedicated inspect territory per screen
- Anchored information panels
- Calmer reading flow
- Controller-friendly layouts
- Stream/cinematic-readable presentation

**Global rule:** Tooltips = short hints only. Full descriptions live in inspect panels, fixed info regions, or detail cards. See [`ui-design-system.md` §3–4](../../ui-design-system.md).

### 2.2 Hero identity as emotional anchor

The **hero identity card** is a primary primitive — not a late polish item.

It anchors party UI emotionally across build, summary, gear, combat, and future roster/camp systems.

See §5.

### 2.3 Loadout row as combat script

Slot order left → right is **gameplay language** and **brand identity**.

The loadout row is *the combat script of the hero*. Visual reinforcement is a long-term program; near-term: indices, stable layout, inspect copy, shared `skill_bar` everywhere.

Future-facing ideas (document only): sequencing cues, combo indicators, GCD pulse directionality, reactive linkage visuals — see design system §12.

### 2.4 Fake-lit UI

UI feels illuminated by campfires, braziers, magic, dungeon warmth/cold — via gradients, glow placement, contrast hierarchy — **not** real-time lighting.

### 2.5 Layout before motion

Readability and stable layout precede animation. Motion is subtle, low-frequency, attention-guiding. No bounce, scale-on-hover, or constant movement.

---

## 3. Problem statement (audit summary)

### Cohesive today

- Buildcraft sheet, session, modal blocking
- `skill_icon`, `skill_bar`, `inspect_panel`, `skill_presentation`
- `ui/primitives/` scaffold, `ui/interaction/` dispatch
- Visual bible alignment (warm vs cold, restraint)

### Fragmented today

| Surface | Gap |
|---------|-----|
| **Build screen** | 22px text slots vs 52px buildcraft bar; `build_panel_text` wall; tooltip-heavy |
| **Gear hub** | Text-first item cards; no inspect region; tooltip actions |
| **Summary** | Plaintext rewards; weak loot hierarchy |
| **Combat theater** | Duplicated bar code; no shared skill row; ad-hoc cast stacks |
| **Skill shop** | Non-canonical modal; text catalogue |
| **Cross-cutting** | Two panel families; micro icon tier legacy; inspect vs tooltip split |

### Architectural risk (elevated)

**Visual fragmentation despite modular architecture.**

Healthy module tree:

```
presentation/  primitives/  shell/  buildcraft/  theme/  skill_presentation/
```

…becomes **dangerous** without:

- [`docs/ui-design-system.md`](../../ui-design-system.md) as law
- Shared presets (`UiDensity`, panel recipes)
- Strict primitive reuse
- PR checklist (design system §19)

---

## 4. Density zones

Formal concept — shared DNA, different cadence. Full table in design system §8.

| Zone | Character |
|------|-----------|
| Buildcraft | Spacious / strategic |
| Camp (build) | Atmospheric / preparatory |
| Combat | Compact / tactical |
| Gear | Inventory-dense |
| Summary | Celebratory |
| Title camp | Atmospheric (separate program) |

Screens declare a zone; primitives apply tier + gutters.

---

## 5. Hero identity card (primitive spec)

### Purpose

Emotional anchor of party UI — reduces “debug tool” feeling immediately.

### Required support (v1)

- Portrait / silhouette placeholder (`UiPlaceholderImages`)
- Role / class identity line
- Display name (title) + subtitle (class tag or role)
- Subtle category or party warm tint on frame edge
- Compact stat strip hook (optional v1)

### Future hooks (structure only — do not implement in convergence v1)

- Stance / status badge
- Relationship indicators (party synergy)
- Campfire-lighting variant (fake-lit warm gradient)
- Partner row linking (P1/P2)

### Reuse targets

- Build screen hero column
- Summary hero column
- Gear hub loadout header
- Combat theater party header
- Future roster / camp expansion

### Visual recipe

```
hero_identity_card:
  mounted deep panel OR ornate left accent strip
  64–80px portrait frame (square, inner border)
  title: FONT_SECTION + muted_cream
  subtitle: FONT_CAPTION + body_dim
  optional: category_chip row for dominant build theme
  padding: PANEL_INSET_LG
  density: Camp zone
```

Extract to `ui/primitives/hero_card.rs` (or `hero_identity.rs`).

---

## 6. Primitive roadmap (convergence scope)

| Primitive | Priority | Notes |
|-----------|----------|-------|
| Theme tokens + `UiDensity` | P0 | `theme.rs` |
| `mounted_panel` | P0 | Dogfood on buildcraft column |
| `framed_section_header` | P0 | Extend `spawn_section_header` |
| `loadout_row` | P1 | Wrap `skill_bar` + label |
| **`hero_identity_card`** | **P1 (elevated)** | Before full build screen |
| `inspect_panel` variants | P2 | compact, loot |
| `category_chip` | P2 | Display family badge |
| `rarity_frame` / item card v2 | P3 | Gear + summary |
| `reward_card` | P3 | Summary |
| Combat bar presets | P4 | Theater |
| `cooldown_overlay` / `selection_glow` | P5 | Hooks + subtle motion |
| Skill shop icon grid | P6 | Last major screen |

---

## 7. Screen convergence targets (outcomes)

### Build screen

- Hero identity card + `loadout_row` (shared `skill_bar`)
- Retire text loadout dump; stats as compact chips
- Footer buttons match buildcraft variants
- Fixed inspect strip; demote slot tooltips

### Inspect ecosystem (cross-cutting phase)

- Policy enforced in code review
- Per-screen inspect territory map (design system §4)
- Tooltip audit + migration checklist

### Gear hub

- Item cards icon-first + `rarity_frame`
- Stash inspect panel (fixed)
- Loadout header uses hero identity card

### Summary

- Celebratory density zone
- Outcome hierarchy + `reward_card` grid
- Rewards modal uses canonical modal shell

### Combat theater

- Unified `bar.rs` presets
- Compact skill row (static icons, overlay hooks)
- Document future sequencing visuals (no implementation)

### Skill shop

- Modal shell + library grid pattern (lowest priority — functional today)

---

## 8. Category display mapping

Presentation-only in `skill_presentation.rs`:

| Display | Domain |
|---------|--------|
| Attack (ember/orange) | `BasicAttack`, `AttackSkill`, `Proc` |
| Reaction (steel/blue) | `Reactive` |
| Passive (violet) | `Passive`, `Channel` |
| Sustain (green/gold) | `Buff` |
| Utility (pale cyan) | Reserved |

---

## 9. Approaches considered

| Approach | Verdict |
|----------|---------|
| **A** Foundation-first + screen migration | **Approved** |
| **B** Screen-first without tokens | Rejected — drift risk |
| **C** Token doc only | Insufficient — must migrate surfaces |

---

## 10. Success criteria (program complete)

- [`docs/ui-design-system.md`](../../ui-design-system.md) exists and is referenced by new UI PRs
- Build, gear, summary share framing, typography, icon tiers
- All camp loadout slots use `skill_bar` / `skill_icon`
- Hero identity card on build + summary (+ gear header)
- No full descriptions on cursor tooltips on migrated screens
- Inspect regions on build, buildcraft, gear, summary
- `shell/layout.rs` reduced via extraction; no new ad-hoc panel spawns
- Combat bars unified; skill row scaffold present
- Visual fragmentation called out in reviews when presets skipped

---

## 11. References

- [`docs/ui-design-system.md`](../../ui-design-system.md)
- [`docs/visual-bible-foundation-v1.md`](../../visual-bible-foundation-v1.md)
- [`docs/design-philosophy.md`](../../design-philosophy.md)
- Buildcraft spec: [`2026-05-20-skillbook-buildcraft-ux-design.md`](2026-05-20-skillbook-buildcraft-ux-design.md)
