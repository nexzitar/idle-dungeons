# UI Foundation Extraction Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Extract reusable `ui/primitives` and `ui/interaction` layers from prototype-era assembly code without changing gameplay, simulation, or visual output.

**Architecture:** Keep `domain/` / `presentation/` / `ui/` split. Move scroll, button, bar, panel, and modal spawn logic into `ui/primitives/`. Centralize release-on-click in `ui/interaction/`. Split `mockup_layout.rs` into `ui/shell/`. Migrate screens incrementally.

**Tech Stack:** Rust, Bevy 0.16 UI (`Button`, `Node`, `Interaction`), existing `UiTheme` / `UiButtonPalette`, `cargo test`.

**Design spec:** [`docs/superpowers/specs/2026-05-21-ui-foundation-extraction-design.md`](../specs/2026-05-21-ui-foundation-extraction-design.md)

---

## Prerequisites

- [ ] Read design spec (sections: Constraints, Target architecture, Phased delivery)
- [ ] Branch from `master` or continue `feat/wave7-presentation` if presentation work must land first
- [ ] `cargo test` green before Phase 1

```bash
cd /Users/mattias/prog/Best_Game && cargo test
```

Expected: all tests pass (153+ lib, 5 integration).

---

## Phase 1 — Extract primitives (behavior-neutral)

### Task 1: Scaffold `ui/primitives` module

**Files:**
- Create: `src/ui/primitives/mod.rs`
- Create: `src/ui/primitives/spacing.rs`
- Modify: `src/ui/mod.rs` (add `pub mod primitives;`)

- [ ] **Step 1: Create module tree**

`src/ui/primitives/mod.rs`:

```rust
pub mod bar;
pub mod button;
pub mod panel;
pub mod scroll;
pub mod spacing;
pub mod text;

pub use bar::*;
pub use button::*;
pub use panel::*;
pub use scroll::*;
pub use spacing::*;
pub use text::*;
```

`src/ui/primitives/spacing.rs`:

```rust
use bevy::prelude::*;
use crate::ui::theme::UiTheme;

pub fn column_stretch() -> Node {
    Node {
        box_sizing: BoxSizing::BorderBox,
        width: Val::Percent(100.0),
        flex_direction: FlexDirection::Column,
        align_items: AlignItems::Stretch,
        ..default()
    }
}

pub fn row_gap(gap_px: f32) -> Node {
    Node {
        flex_direction: FlexDirection::Row,
        column_gap: Val::Px(gap_px),
        align_items: AlignItems::Center,
        ..default()
    }
}

pub const ROOT_PAD_X: f32 = UiTheme::PAD_ROOT;
pub const ROOT_PAD_Y: f32 = UiTheme::PAD_BAR_Y;
```

- [ ] **Step 2: Wire into `ui/mod.rs`**

Add after existing modules:

```rust
pub mod primitives;
```

- [ ] **Step 3: Verify compile**

```bash
cargo check
```

Expected: success (empty bar/button modules added in Task 2–4).

- [ ] **Step 4: Commit**

```bash
git add src/ui/primitives/ src/ui/mod.rs
git commit -m "refactor(ui): scaffold primitives module"
```

---

### Task 2: Move scroll primitives from `widgets.rs`

**Files:**
- Create: `src/ui/primitives/scroll.rs`
- Modify: `src/ui/widgets.rs` (re-export or delegate)
- Modify: `src/ui/mod.rs` (`apply_ui_scroll` import path)

- [ ] **Step 1: Move scroll spawn + components usage**

Cut from `widgets.rs` into `primitives/scroll.rs`:

- `spawn_panel_scroll_viewport` (rename to `spawn_scroll_viewport`)
- `spawn_scrollable_flex_column`
- `spawn_scrollable_log`

Keep public signatures identical. Import `UiScrollRegion`, `UiScrollContent`, `UiScrollState` from `crate::ui::components`.

- [ ] **Step 2: Move `apply_ui_scroll` + `pin_playback_combat_log_scroll`**

Cut systems from `ui/mod.rs` into `primitives/scroll.rs` as `pub fn apply_ui_scroll(...)` and `pub fn pin_playback_combat_log_scroll(...)`.

Register in `UiPlugin` via `crate::ui::primitives::scroll::apply_ui_scroll` (same schedule ordering).

- [ ] **Step 3: Update call sites**

Replace `crate::ui::widgets::spawn_scrollable_flex_column` → `crate::ui::primitives::spawn_scrollable_flex_column` in:

- `src/ui/gear_hub.rs`
- `src/ui/mockup_layout.rs`
- `src/ui/widgets.rs` (delegate: `pub use crate::ui::primitives::scroll::spawn_scrollable_flex_column;` for one commit of compatibility)

- [ ] **Step 4: Run tests**

```bash
cargo test
```

Expected: PASS (no behavior change).

- [ ] **Step 5: Commit**

```bash
git commit -m "refactor(ui): extract scroll primitives"
```

---

### Task 3: Extract button primitive

**Files:**
- Create: `src/ui/primitives/button.rs`
- Modify: `src/ui/components.rs` (keep `UiButtonPalette`; optional `UiButtonDisabled` marker)
- Modify: `src/ui/widgets.rs` (Settings button uses primitive)

- [ ] **Step 1: Define variant → palette mapping**

`src/ui/primitives/button.rs`:

```rust
use bevy::prelude::*;
use bevy::text::{TextColor, TextFont};
use bevy::ui::RelativeCursorPosition;
use crate::ui::components::UiButtonPalette;
use crate::ui::theme::UiTheme;

#[derive(Clone, Copy, Debug)]
pub enum UiButtonVariant {
    Primary,
    Secondary,
    Danger,
    PanelOutlined,
    Equip,
    Salvage,
}

impl UiButtonVariant {
    pub fn palette(self) -> UiButtonPalette {
        match self {
            Self::Primary => UiButtonPalette::primary_cta(),
            Self::Secondary => UiButtonPalette::panel_outlined(),
            Self::Danger => UiButtonPalette::salvage(), // reuse until dedicated danger palette exists
            Self::PanelOutlined => UiButtonPalette::panel_outlined(),
            Self::Equip => UiButtonPalette::equip(),
            Self::Salvage => UiButtonPalette::salvage(),
        }
    }
}

pub struct UiButtonConfig<'a> {
    pub label: &'a str,
    pub variant: UiButtonVariant,
    pub width: Val,
    pub height: Val,
    pub font_size: f32,
}

pub fn spawn_button(
    parent: &mut ChildSpawnerCommands<'_>,
    config: UiButtonConfig<'_>,
) -> Entity {
    let pal = config.variant.palette();
    parent
        .spawn((
            Node {
                box_sizing: BoxSizing::BorderBox,
                width: config.width,
                height: config.height,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                border: UiRect::all(Val::Px(1.0)),
                ..default()
            },
            Button,
            BackgroundColor(pal.idle_bg),
            BorderColor::from(pal.idle_border),
            pal,
            Interaction::default(),
            RelativeCursorPosition::default(),
        ))
        .with_children(|btn| {
            btn.spawn((
                Text::new(config.label),
                TextFont::from_font_size(config.font_size),
                TextColor(Color::WHITE),
            ));
        })
        .id()
}
```

Add `UiButtonPalette::primary/secondary/danger` helpers in `components.rs` if missing (mirror existing equip/salvage/panel_outlined patterns).

- [ ] **Step 2: Migrate Settings button in `spawn_top_resource_bar`**

Replace inline spawn block with:

```rust
let settings = spawn_button(row, UiButtonConfig {
    label: "Settings",
    variant: UiButtonVariant::PanelOutlined,
    width: Val::Px(96.0),
    height: Val::Px(34.0),
    font_size: UiTheme::FONT_COMPACT,
});
// row.get_entity(settings).insert(SettingsButton); — or return entity and insert in caller
```

- [ ] **Step 3: Run tests + manual click Settings on Title/Build**

```bash
cargo test
```

- [ ] **Step 4: Commit**

```bash
git commit -m "refactor(ui): add spawn_button primitive and migrate Settings"
```

---

### Task 4: Extract bar primitive

**Files:**
- Create: `src/ui/primitives/bar.rs`
- Modify: `src/ui/mockup_layout.rs` (`playback_player0_bar`, ally bar helpers)

- [ ] **Step 1: Implement `spawn_horizontal_bar`**

```rust
pub struct UiBarStyle {
    pub track: Color,
    pub fill: Color,
    pub height_px: f32,
}

pub fn spawn_horizontal_bar<M: Component>(
    parent: &mut ChildSpawnerCommands<'_>,
    style: UiBarStyle,
    fill_marker: M,
    initial_fill_pct: f32,
) -> Entity {
    parent
        .spawn((
            Node {
                box_sizing: BoxSizing::BorderBox,
                width: Val::Percent(100.0),
                height: Val::Px(style.height_px),
                ..default()
            },
            BackgroundColor(style.track.into()),
        ))
        .with_children(|track| {
            track.spawn((
                Node {
                    width: Val::Percent(initial_fill_pct.clamp(0.0, 100.0)),
                    height: Val::Percent(100.0),
                    ..default()
                },
                BackgroundColor(style.fill.into()),
                fill_marker,
            ));
        })
        .id()
}
```

- [ ] **Step 2: Refactor `playback_player0_bar` / `playback_player1_bar` in mockup_layout**

Replace duplicated Node trees with `spawn_horizontal_bar` calls; keep existing `PlaybackPlayer0BarFill` / `PlaybackPlayer1BarFill` markers.

- [ ] **Step 3: Run tests**

```bash
cargo test
```

- [ ] **Step 4: Commit**

```bash
git commit -m "refactor(ui): extract horizontal bar primitive for playback"
```

---

### Task 5: Extract panel + modal primitives

**Files:**
- Create: `src/ui/primitives/panel.rs`
- Create: `src/ui/primitives/modal.rs`
- Modify: `src/ui/gear_hub.rs`
- Modify: `src/ui/widgets.rs`

- [ ] **Step 1: Move `spawn_framed_panel`, `spawn_bottom_strip` → `panel.rs`**

- [ ] **Step 2: Add `spawn_modal_shell` in `modal.rs`**

Extract pattern from `gear_hub.rs`:

- Full-screen `Node` + `GearHubBackdrop` marker
- Center card with border/background from `UiTheme`
- Optional `on_backdrop_click` marker component

- [ ] **Step 3: Refactor `spawn_gear_hub_modal` to use modal + scroll + button primitives**

- [ ] **Step 4: Run tests**

```bash
cargo test
```

- [ ] **Step 5: Commit**

```bash
git commit -m "refactor(ui): extract panel and modal primitives"
```

---

### Task 6: Text primitive wrappers

**Files:**
- Create: `src/ui/primitives/text.rs`

- [ ] **Step 1: Re-export theme text spawns**

```rust
pub use crate::ui::theme::{body_text, caption_text, section_title};
```

- [ ] **Step 2: Optional `spawn_section_header` helper** (title + bottom margin)

- [ ] **Step 3: Commit**

```bash
git commit -m "refactor(ui): add text primitive wrappers"
```

---

### Task 7: Phase 1 changelog + doc pointer

**Files:**
- Modify: `CHANGELOG.md` (Unreleased)
- Modify: `docs/design-philosophy.md` (one line linking ui primitives spec)

- [ ] **Step 1: Add Unreleased entry**

```markdown
### UI foundation (Phase 1)
- Extracted `ui/primitives` (button, scroll, bar, panel, modal) — behavior-neutral moves from `widgets` / `mockup_layout`.
```

- [ ] **Step 2: Commit**

```bash
git commit -m "docs: note UI primitives Phase 1 extraction"
```

---

## Phase 2 — Naming & module layout

### Task 8: Rename `placeholder_graphics` → `assets`

**Files:**
- Rename: `src/ui/placeholder_graphics.rs` → `src/ui/assets.rs`
- Modify: all imports (`UiPlaceholderImages`, `register_ui_placeholder_images`)

- [ ] **Step 1: `git mv` file**

```bash
git mv src/ui/placeholder_graphics.rs src/ui/assets.rs
```

- [ ] **Step 2: Update `mod.rs`**

```rust
pub mod assets;
// remove pub mod placeholder_graphics;
```

Update `RegisterUiPlaceholderImages` system path to `assets::register_ui_placeholder_images`.

- [ ] **Step 3: Global replace imports**

`crate::ui::placeholder_graphics` → `crate::ui::assets`

- [ ] **Step 4: `cargo test` + commit**

```bash
git commit -m "refactor(ui): rename placeholder_graphics to assets"
```

---

### Task 9: Split `mockup_layout` → `ui/shell/`

**Files:**
- Create: `src/ui/shell/mod.rs`, `layout.rs`, `theater.rs`, `playback_bars.rs`, `footer.rs`
- Rename/remove: `src/ui/mockup_layout.rs`

- [ ] **Step 1: Create `shell/mod.rs` re-exporting public API**

Preserve current public functions used elsewhere:

- `spawn_mockup_header`
- `spawn_running_screen_content` (or equivalent)
- `FooterMode`
- `spawn_dungeon_summary_column`
- etc. (grep `mockup_layout::` before cut)

- [ ] **Step 2: Move theater/playback sections to `theater.rs` + `playback_bars.rs`**

- [ ] **Step 3: Move footer/chrome to `footer.rs` + `layout.rs`**

- [ ] **Step 4: Update `mod.rs` and feature imports**

- [ ] **Step 5: `cargo test` + commit**

```bash
git commit -m "refactor(ui): split mockup_layout into shell module"
```

---

### Task 10: Deprecate `widgets.rs` surface

**Files:**
- Modify: `src/ui/widgets.rs` → thin re-exports or delete

- [ ] **Step 1: Replace `widgets` usages with `primitives` + `shell`**

- [ ] **Step 2: Keep `widgets.rs` as compatibility shim for one release OR remove and fix imports**

- [ ] **Step 3: Commit**

```bash
git commit -m "refactor(ui): retire widgets module in favor of primitives"
```

---

## Phase 3 — Interaction centralization

### Task 11: Scaffold `ui/interaction`

**Files:**
- Create: `src/ui/interaction/mod.rs`, `click.rs`, `registry.rs`, `dispatch.rs`
- Modify: `src/ui/mod.rs`

- [ ] **Step 1: Move click resources**

Move to `interaction/click.rs`:

- `UiClickPress`
- `UiPressedButtonEntitiesOnClick`
- `capture_ui_pressed_button_entities`
- `capture_ui_click_start`
- `clear_ui_click_after_release`
- `ui_click_release_confirms` (pub(crate))

- [ ] **Step 2: Define `UiClickAction` enum**

`src/ui/interaction/registry.rs`:

```rust
#[derive(Component, Clone, Copy, Debug)]
pub enum UiClickAction {
    TitleEnterCamp,
    TitleQuit,
    StartRun,
    SkipPlayback,
    AcceptRewards,
    ResetProgress,
    OpenSettings,
    CloseSettings,
    // ... one variant per interactive control
}
```

- [ ] **Step 3: `dispatch_ui_clicks` system**

On mouse release, if `UiClickPress` entity has `UiClickAction`, write corresponding `Message`.

- [ ] **Step 4: Commit scaffold**

```bash
git commit -m "refactor(ui): scaffold interaction dispatch"
```

---

### Task 12: Migrate title + build buttons to `UiClickAction`

**Files:**
- Modify: `src/ui/title_camp.rs`, `src/ui/mod.rs` spawn sites, remove `handle_title_*` / `handle_start_button`

- [ ] **Step 1: Insert `UiClickAction` on spawn for `TitleEnterCampButton`, `TitleQuitButton`, `StartRunButton`**

- [ ] **Step 2: Implement dispatch arms; delete old handler systems**

- [ ] **Step 3: Run UI tests**

```bash
cargo test ui_plugin_starts_on_title pressing_start_button
```

Expected: PASS.

- [ ] **Step 4: Commit**

```bash
git commit -m "refactor(ui): centralize title and build click dispatch"
```

---

### Task 13: Migrate summary, stash, modals, editor buttons

**Files:**
- Modify: `src/ui/mod.rs`, `gear_hub.rs`, `skill_book.rs`, `skill_shop.rs`, `presentation/editor/overlay.rs`

- [ ] **Step 1: Batch-convert Accept/Equip/Salvage/StashSort/SkillPick/Close* buttons**

- [ ] **Step 2: Batch-convert presentation editor overlay buttons**

- [ ] **Step 3: Remove redundant `handle_*` systems; keep `apply_ui_button_palettes`**

- [ ] **Step 4: Full test suite**

```bash
cargo test
```

- [ ] **Step 5: Commit**

```bash
git commit -m "refactor(ui): complete UiClickAction dispatch migration"
```

---

## Phase 4 — Theme composition helpers

### Task 14: Layout presets + item card helper

**Files:**
- Modify: `src/ui/theme.rs` or `src/ui/primitives/spacing.rs`
- Modify: `src/ui/mod.rs` (`spawn_item_card`)

- [ ] **Step 1: Add `UiPanelStyle`, `UiModalStyle` structs wrapping colors/padding**

- [ ] **Step 2: Move `spawn_item_card` / `spawn_item_card_preview` to `primitives/list.rs` or `primitives/card.rs`**

- [ ] **Step 3: Tests + commit**

```bash
git commit -m "refactor(ui): theme layout presets and item card primitive"
```

---

## Phase 5 — Screen migration & `mod.rs` slimming

### Task 15: Extract screen spawn from `mod.rs`

**Files:**
- Create: `src/ui/screens/mod.rs`, `build.rs`, `summary.rs`, `running.rs`
- Modify: `src/ui/mod.rs`

- [ ] **Step 1: Move `spawn_build_screen*` → `screens/build.rs`**

- [ ] **Step 2: Move `spawn_summary_screen*` → `screens/summary.rs`**

- [ ] **Step 3: Move `spawn_running_screen*` → `screens/running.rs`**

- [ ] **Step 4: Leave `UiPlugin::build` registration in `mod.rs` only**

- [ ] **Step 5: `cargo test` + commit**

```bash
git commit -m "refactor(ui): extract screen spawn modules from mod.rs"
```

---

### Task 16: Presentation editor uses primitives

**Files:**
- Modify: `src/presentation/editor/overlay.rs`

- [ ] **Step 1: Replace inline button Nodes with `spawn_button`**

- [ ] **Step 2: Replace inspector section frames with `spawn_framed_panel`**

- [ ] **Step 3: Manual test: Title → ` toggle editor → Save/Reload**

- [ ] **Step 4: Commit**

```bash
git commit -m "refactor(editor): use ui primitives for overlay chrome"
```

---

### Task 17: Final cleanup & success metrics

- [ ] **Step 1: `rg 'mockup_layout|placeholder_graphics' src/`** — zero hits

- [ ] **Step 2: `wc -l src/ui/mod.rs`** — target < 500 lines

- [ ] **Step 3: Full `cargo test`**

- [ ] **Step 4: Manual smoke**

1. Boot → Title campfire visible
2. Enter camp → Build → Start run
3. Skip playback → Summary → Accept → Title
4. Open Gear hub, scroll stash
5. Open Skill book, pick skill
6. (debug) Presentation editor save/reload

- [ ] **Step 5: CHANGELOG final entry + commit**

```bash
git commit -m "docs: complete UI foundation extraction Phase 1-5"
```

---

## Self-review (spec coverage)

| Spec section | Plan tasks |
|--------------|------------|
| UI primitives layer | Tasks 1–7 |
| Button extraction | Task 3, 12–13 |
| Scroll abstraction | Task 2 |
| Bar system | Task 4 |
| Naming cleanup | Tasks 8–10 |
| Split oversized files | Tasks 9, 15 |
| Interaction centralization | Tasks 11–13 |
| Explicit composition | Enforced in primitive APIs |
| Preserve architecture | Constraints in spec; no domain edits |
| Editor compatibility | Task 16 |
| Execution order | Phases 1–5 task order |

---

## Execution handoff

Plan complete and saved to `docs/superpowers/plans/2026-05-21-ui-foundation-extraction.md`.

**Two execution options:**

1. **Subagent-Driven (recommended)** — fresh subagent per task, review between tasks  
2. **Inline Execution** — run phases in this session with `executing-plans` checkpoints  

Which approach do you want?
