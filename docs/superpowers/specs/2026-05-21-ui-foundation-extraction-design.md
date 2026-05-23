# UI Foundation Extraction — Design Spec (2026-05-21)

## Status

**Approved direction — no production refactor yet.** Implementation follows [`docs/superpowers/plans/2026-05-21-ui-foundation-extraction.md`](../plans/2026-05-21-ui-foundation-extraction.md).

This is **architectural maturation**, not a rewrite. Gameplay simulation, ECS shape, and visual output must remain stable while `ui/` evolves from screen assembly into a reusable toolkit.

---

## Problem

Delvers’ high-level split is already correct:

| Layer | Role |
|-------|------|
| `domain/` | Deterministic simulation truth |
| `presentation/` | Scene composition, JSON tuning, editor, fire/atmosphere |
| `ui/` | Bevy UI: screens, interaction, theme, playback theater |

The weakness is **`ui/` still mixes three concerns**:

1. **Reusable widget behavior** (buttons, scroll, bars, modals) — partially started in `widgets.rs`, duplicated elsewhere.
2. **Interaction plumbing** — release-on-click, palette sync, and ~20 per-button handler systems in `mod.rs` (~3k lines).
3. **Feature composition** — build camp, playback theater, stash, modals — often inlined in `mockup_layout.rs` (~2.8k lines).

Prototype-era names (`mockup_layout`, `placeholder_graphics`) and giant files slow iteration. The presentation editor overlay also reimplements button/panel patterns parallel to gameplay UI.

**Goals**

- Reusable UI primitives with theme integration
- Centralized interaction handling
- Cleaner composition in screen modules
- One widget foundation for gameplay UI, modals, and editor chrome
- Preserved behavior, gameplay flow, and deterministic simulation

**Non-goals**

- Rewriting `domain/` combat/progression
- ECS architecture redesign
- Trait-heavy “UI framework”
- Visual redesign during refactor
- Replacing `presentation/` scene JSON workflow

---

## Constraints (hard)

1. **Behavior preservation** — existing `cargo test` (153+ lib tests) and UI integration tests in `src/ui/mod.rs` must pass after each phase unless a test is updated to match intentional naming-only changes.
2. **Explicit composition** — prefer `spawn_*` helpers and small bundles over generic factories.
3. **No gameplay in primitives** — primitives know `UiTheme`, `Interaction`, scroll math; they do not import `GameState` or emit domain events directly.
4. **Feature markers stay on screens** — `StartRunButton`, `AcceptRewardsButton`, etc. remain `Component` markers on spawned nodes; primitives supply bundles + registration hooks.
5. **Presentation boundary** — `presentation/` keeps scene/anchor/fire/editor; it may **call** `ui::primitives` for editor chrome, not duplicate button styling.

---

## Current inventory (baseline)

| File | ~Lines | Responsibility today |
|------|--------|-------------------|
| `ui/mod.rs` | 3034 | `UiPlugin`, screen spawn, 20+ click handlers, scroll systems, playback sync, item cards |
| `ui/mockup_layout.rs` | 2809 | Shell layout, theater columns, playback bars/portraits, footer, modals scaffolding |
| `ui/components.rs` | 468 | Marker components + `UiButtonPalette` |
| `ui/widgets.rs` | 380 | Atmosphere, top bar, framed panel, scroll viewports, log scroll |
| `ui/theme.rs` | 376 | Colors, typography, item formatting, playback float colors |
| `ui/tooltip.rs` | 189 | Tooltip layer + hover state |
| `ui/placeholder_graphics.rs` | 194 | `UiPlaceholderImages` asset handles |
| Feature panels | 150–580 each | `build_panel`, `title_camp`, `gear_hub`, `skill_book`, etc. |

**Existing partial abstractions**

- **Scroll:** `UiScrollRegion`, `UiScrollContent`, `UiScrollState`; `apply_ui_scroll` in `mod.rs`; spawn helpers in `widgets.rs` and duplicated in `gear_hub.rs`.
- **Buttons:** Bevy `Button` + `UiButtonPalette` + `apply_ui_button_palettes`; release-on-click via `UiClickPress` / `UiPressedButtonEntitiesOnClick` and per-marker `handle_*` systems.
- **Bars:** Inline Node width % fills in `mockup_layout.rs`; sync systems in `mod.rs` (`sync_playback_cast_bars_*`, `sync_run_playback_party_bars`).

---

## Target architecture

```
src/ui/
  mod.rs                 # UiPlugin, thin re-exports, screen OnEnter/Exit wiring
  components.rs          # Feature marker components only (shrinking over time)
  theme.rs               # Tokens + text helpers (unchanged role)
  interaction/           # Phase 3: click, focus, keyboard routing
  primitives/
    mod.rs
    button.rs            # UiButtonBundle, variants, palette wiring
    panel.rs             # framed panel, section chrome
    scroll.rs            # scroll viewport + wheel (moved from widgets/mod)
    bar.rs               # track + fill + optional label
    modal.rs             # backdrop + centered card + close affordance
    text.rs              # caption/body/section helpers wrapping theme
    list.rs              # vertical list spacing, card gutters
    spacing.rs           # PAD_* layout constants usage
    tooltip_hook.rs      # optional: register node for tooltip id
  shell/                 # Phase 2 rename from mockup_layout
    mod.rs
    layout.rs            # 3-column shell, header, footer
    theater.rs           # playback column composition
    playback_bars.rs     # bar spawn using primitives::bar
  screens/               # Phase 5 migrations (optional gradual)
    title_camp.rs        # (existing path ok; imports primitives)
    build.rs
    ...
  assets.rs              # Phase 2 rename from placeholder_graphics
  tooltip.rs             # May re-export from primitives or stay sibling
```

**Dependency rule**

```
domain/  ← never imports ui/
presentation/  → may import ui::primitives (editor chrome only)
ui/primitives/  → imports theme, bevy only
ui/screens/     → imports primitives, components, domain types for labels only
ui/mod.rs       → wires systems; delegates spawn to screens/shell
```

---

## Primitive designs

### 1. Button (`primitives/button.rs`)

**Purpose:** One code path for hover/pressed/disabled + release-on-click registration.

```rust
pub enum UiButtonVariant {
    Primary,
    Secondary,
    Danger,
    PanelOutlined,
    Equip,
    Salvage,
    Ghost,
}

pub struct UiButtonBundle {
    pub variant: UiButtonVariant,
    pub label: String,
    pub min_size: Option<(f32, f32)>,
    pub disabled: bool,
}

pub fn spawn_button(parent: &mut ChildSpawnerCommands<'_>, bundle: UiButtonBundle) -> Entity;
```

**Behavior**

- Spawns `Button`, `UiButtonPalette` (from variant), `Interaction`, `RelativeCursorPosition`.
- Feature marker component passed separately: `spawn_button(...).insert(StartRunButton)`.
- Registers with `UiInteractionRegistry` (Phase 3) for release-on-click → `Message` / callback map.

**Migration:** Replace copy-pasted button Node blocks in `widgets.rs` (Settings), `gear_hub.rs`, `mockup_layout.rs`, presentation editor overlay.

### 2. Scroll (`primitives/scroll.rs`)

**Purpose:** Single implementation of clip + wheel + `UiScrollState`.

Move from `widgets.rs` + `gear_hub` duplicate:

- `spawn_scroll_viewport` / `spawn_scrollable_flex_column`
- Document when to use `min_viewport_height_px`

**System ownership:** `apply_ui_scroll` moves from `mod.rs` to `primitives/scroll.rs` (exported fn registered by plugin).

### 3. Bar (`primitives/bar.rs`)

**Purpose:** Unified track + fill for HP, cast, CD, GCD, threat, delve progress.

```rust
pub struct UiBarStyle {
    pub track_color: Color,
    pub fill_color: Color,
    pub height_px: f32,
    pub radius: f32,  // future
}

pub struct UiBarFillMarker; // fill node marker for sync systems

pub fn spawn_bar_track(
    parent: &mut ChildSpawnerCommands<'_>,
    style: UiBarStyle,
    fill_component: impl Component,
) -> (Entity /*track*/, Entity /*fill*/);
```

**Phase 1 scope:** Static width % fills only (current behavior). Delayed fill, gradients, segments → later.

**Migration:** `playback_player0_bar`, cast/cd/gcd fills in `mockup_layout.rs` → thin wrappers calling `spawn_bar_track`.

### 4. Panel & modal (`primitives/panel.rs`, `primitives/modal.rs`)

Extract `spawn_framed_panel`, `spawn_bottom_strip`, modal backdrop pattern from `widgets.rs` / `gear_hub.rs` / `skill_book.rs`.

```rust
pub fn spawn_modal_frame(
    parent: &mut ChildSpawnerCommands<'_>,
    opts: ModalOptions,
    content: impl FnOnce(&mut ChildSpawnerCommands<'_>),
) -> Entity;
```

Modal includes: dim backdrop (click-to-close marker), centered card, optional title row, close button via `spawn_button`.

### 5. Text & spacing (`primitives/text.rs`, `primitives/spacing.rs`)

Thin wrappers over `theme::{caption_text, body_text, section_title}` so feature code does not import theme directly for every label.

`spacing.rs` re-exports `UiTheme::PAD_*` and common `Node` flex presets (column stretch, row gap).

---

## Interaction centralization (Phase 3)

**Today:** Each button has a dedicated system querying `(Entity, Interaction)` + `UiClickPress` + `mouse.just_released`.

**Target:** `ui/interaction/`

| Piece | Responsibility |
|-------|----------------|
| `click.rs` | `capture_ui_click_start`, `clear_ui_click_after_release`, `ui_click_release_confirms` |
| `registry.rs` | Map `Entity` → `UiClickAction` enum or typed messages |
| `dispatch.rs` | Single `dispatch_ui_click_actions` system reading registry on release |
| `focus.rs` | Stub for future keyboard/controller focus |
| `hover.rs` | Optional centralized hover sound/tooltip trigger hooks |

**Registration pattern**

```rust
// On spawn:
commands.entity(btn).insert(UiClickAction::StartRun);

// dispatch.rs matches UiClickAction → MessageWriter<StartRun>
```

**Migration strategy:** Convert handlers in batches (title → build → summary → modals → editor). Keep old systems until batch passes tests, then delete.

**Do not** build a fully generic event bus — explicit `enum UiClickAction` with ~25 variants is fine and readable.

---

## Naming cleanup (Phase 2)

| Current | Proposed | Rationale |
|---------|----------|-----------|
| `mockup_layout.rs` | `shell/layout.rs` + splits | No longer a mockup; owns game shell |
| `placeholder_graphics.rs` | `assets.rs` | Real production assets |
| `widgets.rs` | Deprecate → `primitives/*` | Name collision with goal |
| `components.rs` | Keep; shrink to markers | Palettes move to `primitives/button` |

Avoid drive-by renames of `scene_tune.rs` (title-specific tuning) or `presentation/` modules in this initiative.

---

## File splitting targets

| Source | Split |
|--------|-------|
| `mockup_layout.rs` | `shell/layout.rs`, `shell/theater.rs`, `shell/playback_bars.rs`, `shell/footer.rs` |
| `mod.rs` | `interaction/*`, `screens/spawn_*.rs`, keep plugin registration only |
| `gear_hub.rs` | Use primitives scroll/modal; file stays feature-focused |

**Rule of thumb:** No file > ~800 lines after Phase 5; `mod.rs` < ~400 lines.

---

## Editor / tooling alignment

Presentation editor (`presentation/editor/overlay.rs`) should:

- Use `ui::primitives::button` for Save/Reload/Reset/hierarchy rows
- Use `ui::primitives::panel` for inspector blocks
- **Not** introduce a parallel button palette system

Editor-specific markers (`PresentationEditorSaveButton`, etc.) stay in `presentation::editor` or `ui::components` — TBD in plan Task 1 (prefer `presentation::editor::components` for editor-only markers).

---

## Testing & verification

| Check | When |
|-------|------|
| `cargo test` | Every task / PR slice |
| `src/ui/mod.rs` integration tests | After interaction changes |
| Manual smoke script | Title → Build → Run → Summary → Accept → Title |
| Visual diff | Optional screenshots; not CI-gated initially |

**No new screenshot test infrastructure in Phase 1.**

Regression signals:

- Button clicks still fire events (start run, accept rewards, equip, skill pick)
- Scroll regions: gear hub stash, combat log
- Playback bars track `CombatPlaybackFrame` fields (`player0_cast`, etc.)

---

## Phased delivery (summary)

| Phase | Deliverable | Risk |
|-------|-------------|------|
| **1** | `ui/primitives/*` extracted; behavior-neutral moves | Low |
| **2** | Renames `mockup_layout` → `shell/`, `placeholder_graphics` → `assets` | Low (import paths) |
| **3** | `ui/interaction/` + `UiClickAction` dispatch | Medium |
| **4** | Theme composition helpers (`UiTextStyle`, layout presets) | Low |
| **5** | Migrate feature panels + shrink `mod.rs` | Medium, incremental |

Each phase merges independently; no long-lived branch required.

---

## Success criteria

- [ ] New button needs ≤3 lines: `spawn_button(...).insert(MyMarker)`
- [ ] Scroll regions use one primitive API (gear hub, log, stash)
- [ ] Playback HP/cast/CD bars use `primitives::bar`
- [ ] `mod.rs` under 500 lines (stretch goal 400)
- [ ] No `mockup_*` or `placeholder_*` in `src/ui/` paths
- [ ] Presentation editor buttons use shared primitives
- [ ] All existing tests green; manual smoke passes
- [ ] `domain/` and `presentation/` scene JSON unchanged in Phase 1–3

---

## Related docs

- [`docs/design-philosophy.md`](../../design-philosophy.md) — simulation-first, UX principles
- [`docs/presentation-editor-workflow.md`](../../presentation-editor-workflow.md) — editor consumes shared UI where possible
- [`docs/superpowers/specs/2026-05-20-presentation-editor-design.md`](2026-05-20-presentation-editor-design.md) — presentation layer (parallel, not merged into ui)
