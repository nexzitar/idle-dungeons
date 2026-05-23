# Delvers — UI Design System

**Status:** Canonical visual source of truth (v1, 2026-05-21)  
**Gold standard implementation:** Party Buildcraft sheet (`src/ui/buildcraft/`, `skill_icon`, `skill_bar`, `inspect_panel`)  
**Related:** [`visual-bible-foundation-v1.md`](visual-bible-foundation-v1.md), [`design-philosophy.md`](design-philosophy.md)

---

## 1. What Delvers UI is

Delvers is a **tactical dark-fantasy buildcraft RPG**. Its UI should feel:

- **Atmospheric but readable** — warm firelight against cold depth
- **MMO-inspired in clarity** — loadout legibility, category language, stable regions
- **Restrained rather than flashy** — crafted, grounded, intentional
- **Information-dense without chaos** — hierarchy does the work, not decoration

The project is **no longer in prototype UI stage**. All new UI must converge on this system.

**We are building:** visual identity, interaction language, long-term UX philosophy — not a one-off cleanup.

---

## 2. Core principles (non-negotiable)

### 2.1 Readability first

If readability conflicts with spectacle, **readability wins**. See visual bible §1.

### 2.2 Stable inspect regions (first-class UX)

**We are moving away from:**

- Cursor tooltip spam
- Overlapping descriptions
- Hover-driven information chaos

**We are moving toward:**

- Dedicated inspect territory
- Anchored information panels
- Calmer reading flow
- Controller-friendly layouts
- Stream/cinematic-readable presentation

**Global rule:** Full skill, item, and hero descriptions live in **inspect panels, fixed info regions, or detail cards** — never in cursor-following tooltips.

### 2.3 Icon-first interaction language

Icons are the primary gameplay vocabulary. Text **supports** icons; it does not replace them.

### 2.4 Mounted / recessed panels

Major UI surfaces feel **framed, inset, layered, slightly recessed** — never “floating debug rectangles.”

### 2.5 Layout stability

Panels do not resize, reflow, or shift on hover. Motion overlays state; it does not move layout.

### 2.6 Category-driven visual language

Skill and ability identity uses **subtle** category tinting on frames, accents, chips, and glows — not full panel recolors.

### 2.7 Layout / readability before animation

Motion guides attention and atmosphere. It never competes with information hierarchy.

---

## 3. Tooltip policy (global Delvers rule)

| Allowed | Forbidden |
|---------|-----------|
| One-line contextual hints (“Click to assign”) | Full skill descriptions |
| Control affordance (“Opens skill library”) | Stat blocks |
| Keyboard/controller hint | Item affix lists |
| Short warning (“Conflicts with Cleave”) | Lore paragraphs |

**Enforcement:** Code review rejects new `UiTooltip` strings longer than ~80 characters or containing `\n` description blocks. Migrate legacy tooltips during screen convergence.

**Replacement:** `inspect_panel`, screen-local fixed info strips, `detail_card` variants.

---

## 4. Inspect philosophy

### Territories

Every screen that shows skills, gear, or heroes should reserve **stable inspect real estate**:

| Context | Inspect territory |
|---------|-------------------|
| Buildcraft sheet | Lower-right inspect panel (canonical) |
| Build camp | Hero column or dedicated strip below loadout |
| Gear hub | Stash column footer or side inspect panel |
| Summary | Reward detail beside loot grid |
| Combat (future) | Theater side panel or focused skill inspect |

### Behavior

- **Hover** updates inspect content; layout does not move
- **Selection/focus** (slot, item, skill) drives primary inspect target
- **No inspect** state shows calm placeholder copy, not empty chrome

### Sync pattern

Inspect content is derived from presentation helpers (`skill_presentation`, item formatters) into `InspectPanelContent`-style structs, then applied to marked text/image entities — never scattered string writes in screen code.

---

## 5. Typography hierarchy

Use `UiTheme` font constants only. Do not invent sizes in screen code.

| Token | Size | Role |
|-------|------|------|
| `FONT_DISPLAY_*` | 34–38 | Title camp hero moments only |
| `FONT_HEADLINE` | 26 | Modal / screen titles |
| `FONT_TITLE` | 22 | Major panel titles |
| `FONT_SECTION` | 17 | Hero identity subtitle, section labels |
| `FONT_BODY` | 15 | Inspect body, primary copy |
| `FONT_CAPTION` | 13 | Hints, warnings, metadata |
| `FONT_LABEL` | 12 | Compact labels |
| `FONT_MICRO` | 11 | Slot indices, chips |

**Text helpers:** `headline_text`, `section_title`, `body_text`, `caption_text` from `ui/theme.rs` / `primitives/text.rs`.

**Hierarchy rule:** One `headline` per major surface; `section_title` for territories; body for reading; caption for receding info.

---

## 6. Spacing scale

| Token | px | Use |
|-------|-----|-----|
| `PAD_ROOT` | 20 | Modal / screen outer padding |
| `PANEL_INSET` | 12 | Standard inner panel padding |
| `PANEL_INSET_SM` | 8 | Compact nested inset |
| `PANEL_INSET_LG` | 14 | Hero cards, inspect panels |
| `PAD_BAR_Y` | 10 | Header/footer vertical rhythm |

**Gutter presets (canonical):**

| Name | px | Use |
|------|-----|-----|
| `GUTTER_SECTION` | 14 | Between major sections (buildcraft) |
| `GUTTER_ROW` | 10 | Between rows in a section |
| `GUTTER_SLOT` | 8 | Between skill bar cells |
| `GUTTER_GRID` | 14 | Library / loot icon grids |

**Rule:** Pick a density zone (§8), then use its gutter preset — do not hardcode one-off margins in screen files.

---

## 7. Icon size tiers

| Tier | px | Density zone | Use |
|------|-----|--------------|-----|
| **Library** | 72 | Spacious | Buildcraft library, shop catalogue |
| **Bar** | 52 | Strategic | Loadout rows, build screen slots |
| **Compact** | 40 | Tactical | Combat skill row (future) |
| **Chip** | 28 | Dense | Inline references, synergy icons |
| **Micro** | 22 | Legacy — **deprecate** | Do not use in new UI |

All tiers use `skill_icon` / `icon_frame` primitives with shared overlay hooks.

---

## 8. Density zones

Zones share **framing, typography, colors, and spacing DNA** but differ in breathing room and cadence.

| Zone | Feel | Icon tier | Gutters | Focus |
|------|------|-----------|---------|-------|
| **Buildcraft** | Spacious / strategic | 52 + 72 | Wide (14) | Rotation planning, party composition |
| **Camp (build)** | Atmospheric / preparatory | 52 | Medium | Hero identity, delve briefing |
| **Combat** | Compact / tactical | 40 | Tight (6–8) | HP, casts, sequencing |
| **Gear** | Inventory-dense | 52 items | Medium-tight | Stash scan, equip decisions |
| **Summary** | Celebratory | 52–72 loot | Medium | Rewards, outcome, pride |
| **Camp (title)** | Atmospheric | N/A | Loose | Firelight, invitation |

Implement via `UiDensity` presets in `theme.rs` (planned) — screens select a zone, primitives apply numbers.

---

## 9. Panel recipes (mounted surfaces)

### Recessed panel (default)

```
Background: panel_bg
Border: 1px panel_border_inner
Padding: PANEL_INSET
```

### Deep mounted card

```
Background: panel_bg_deep
Border: 1px panel_border_inner (or rarity-tinted for items)
Padding: PANEL_INSET
Optional: inner column gap GUTTER_ROW
```

### Ornate primary column (buildcraft party column)

```
Background: panel_bg
Border: 1px ornate_gold (primary territories only — use sparingly)
Padding: PANEL_INSET
```

### Modal shell (canonical)

```
ModalShellRoot: FocusPolicy::Block, fullscreen
Backdrop: modal dim + optional click-to-close
Content: centered, PAD_ROOT, ornate gold 2px frame on primary dialog
```

**Anti-pattern:** Raw `Node` + random `BackgroundColor` in screen code without a recipe.

---

## 10. Border rules

| Border | Color | When |
|--------|-------|------|
| Inner | `panel_border_inner` | Default inset panels |
| Structural | `panel_border` | Dividers, bar tracks |
| Ornate | `ornate_gold` | Primary column/frame — max 1–2 per screen |
| Focus / selection | `ornate_gold` 2px | Focused loadout slot |
| Category | `accent_for_skill` mixed 35% void | Skill icon frames |
| Rarity | `rarity_color` mixed 45% black | Item cards |

Never stack more than two border emphasis levels in one viewport region.

---

## 11. Category color language (display families)

Presentation mapping in `skill_presentation.rs` — **domain categories unchanged**.

| Display family | Tone | Domain map (v1) | Use |
|----------------|------|-----------------|-----|
| **Attack** | Ember / orange | `BasicAttack`, `AttackSkill`, `Proc` | Frames, chips, subtle glow |
| **Reaction** | Steel / blue | `Reactive` | Defensive/readiness accent |
| **Passive** | Violet | `Passive`, `Channel` | Always-on abilities |
| **Sustain** | Green / gold | `Buff` (support/heal) | Recovery, buffs |
| **Utility** | Pale cyan | Reserved / future tags | Control, mobility (when tagged) |

**Application:** icon frame borders, category chips, inspect accent line, synergy highlights — **not** full panel backgrounds.

---

## 12. Combat sequencing language (gameplay + UI)

**Slot order left → right = combat script priority** when multiple skills are ready under GCD.

The loadout row is **the combat script of the hero** — a unique Delvers identity opportunity.

### Current (implemented / planned near-term)

- Subtle slot indices (1–6) on bar cells
- Inspect copy explaining priority
- Stable left-to-right bar layout

### Future-facing (document only — do not implement until combat UX phase)

- Slot sequencing cues during playback
- Left-to-right tactical flow animation
- Combo / chain indicators
- Timing chain readability
- GCD pulse directionality (L→R sweep)
- Reactive linkage visuals (guard → riposte lines)

All future combat presentation must reinforce sequencing without adding clutter.

---

## 13. Fake-lit UI philosophy

UI should feel **illuminated by the world**, not by a flat HUD layer.

**Sources (conceptual):** campfires, braziers, magic, dungeon warmth/cold, enemy danger.

**Achieve via:**

- Gradients (warm top-left bias on camp screens)
- Glow placement (gold accents near “safe” regions)
- Warmth/coolness in panel backgrounds
- Contrast hierarchy (hero = warm, depth briefing = cooler)
- Atmospheric framing (ornate gold = firelight catch)

**Not via:** real-time lighting, shaders, or dynamic shadow systems in UI v1.

**Future hooks:** `hero_identity_card` optional campfire-lighting variant; inspect panel subtle warm edge gradient.

---

## 14. Motion philosophy

### Allowed (low frequency)

- Cooldown radial sweeps
- GCD overlay fills
- Ember pulse on category accents (very subtle)
- Inspect content crossfade (150–250ms)
- Selection glow fade-in
- Atmospheric glow breathing on title camp (existing)

### Forbidden

- Bounce
- Scale-on-hover
- Flashy transitions
- Constant movement
- Excessive VFX on UI chrome
- Layout animation

### Animation frequency guidelines

| Type | Max rate |
|------|----------|
| State overlays (CD/GCD) | Every frame during combat only |
| Atmospheric pulse | ≤ 0.5 Hz |
| Hover glow | Instant on/off or ≤ 200ms fade |
| Inspect update | Debounced; no flicker on fast hover |

**Rule:** If motion does not explain state or atmosphere, delete it.

---

## 15. Interaction rules

1. **Modals block** underlying UI (`FocusPolicy::Block` + subtree click filter).
2. **Primary actions** use `UiButtonVariant::Primary`; secondary use `PanelOutlined`.
3. **Edit sessions** (buildcraft pattern): pending state → Apply/Cancel — use for multi-hero mutations.
4. **Icon clicks** prefer focus-then-assign over immediate commit (except trivial toggles).
5. **Controller-ready:** every inspect region reachable without cursor precision; no required hover for critical info.

---

## 16. Layering principles

**Z-order (bottom → top):**

1. Atmosphere / background bands
2. Screen columns (mounted panels)
3. Modals (`ModalShellRoot`)
4. Tooltip layer (hints only — demote over time)
5. Presentation editor (debug only)

**Within panels:** background → content → overlays (CD/GCD/selection) → labels.

**Visual fragmentation guard:** Growing module tree (`presentation/`, `primitives/`, `shell/`, `buildcraft/`, `theme/`, `skill_presentation/`) is healthy **only** if every screen composes from this document’s presets.

---

## 17. Primitive catalog (target)

| Primitive | Status | Purpose |
|-----------|--------|---------|
| `skill_icon` | Exists | Layered icon + overlay hooks |
| `skill_bar` | Exists | 6-cell loadout row |
| `inspect_panel` | Exists | Fixed inspect territory |
| `mounted_panel` | Planned | Recessed/ornate panel recipe |
| `framed_section_header` | Planned | Section title + rhythm |
| `loadout_row` | Planned | Hero label + skill_bar |
| `hero_identity_card` | Planned | Emotional party anchor |
| `category_chip` | Planned | Display family badge |
| `icon_frame` / `rarity_frame` | Planned | Shared border treatment |
| `reward_card` | Planned | Summary loot tile |
| `cooldown_overlay` | Hooks only | Combat/build overlays |
| `selection_glow` | Planned | Focus ring beyond border |

**Rule:** Appears twice → extract before third copy.

---

## 18. Module ownership

| Concern | Owner module |
|---------|--------------|
| Tokens, colors, typography | `ui/theme.rs` |
| Skill icon/accent mapping | `ui/skill_presentation.rs` |
| Reusable widgets | `ui/primitives/` |
| Screen composition | `ui/screens/`, `ui/shell/` |
| Feature workspaces | `ui/buildcraft/`, `ui/gear_hub.rs`, … |
| Click / modal policy | `ui/interaction/` |
| Atmosphere / title | `ui/title_camp.rs`, `presentation/` |

Screen files **orchestrate**; they do not invent visual language.

---

## 19. Success checklist (per PR)

- [ ] Uses spacing/icon tier from density zone
- [ ] Uses panel recipe, not ad-hoc colors
- [ ] No long tooltips added
- [ ] Inspect or fixed region for primary info
- [ ] Primitives reused or extended
- [ ] `cargo test` green
- [ ] Manual smoke on affected screen

---

## 20. References

- Buildcraft spec: [`superpowers/specs/2026-05-20-skillbook-buildcraft-ux-design.md`](superpowers/specs/2026-05-20-skillbook-buildcraft-ux-design.md)
- Convergence spec: [`superpowers/specs/2026-05-21-ui-design-convergence-design.md`](superpowers/specs/2026-05-21-ui-design-convergence-design.md)
- Implementation plan: [`superpowers/plans/2026-05-21-ui-design-convergence.md`](superpowers/plans/2026-05-21-ui-design-convergence.md)
