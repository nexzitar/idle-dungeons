# Presentation Editor Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Ship a generalized presentation stack for Delvers — discoverable in-engine editor, mouse layout editing, atmospheric radial fire glow, and compositional `PresentationTrack` curves — with title camp as the first scene, without coupling to simulation.

**Architecture:** Modular `src/presentation/` (scene, element, track, editor, fire) plus a thin `src/ui/scene_tune.rs` adapter loading `assets/tuning/title_scene.json`. Presentation ticks mutate existing entities in place; tracks evaluate deterministically from `(t, seed)`. No real-time lighting.

**Tech Stack:** Rust, Bevy 0.x UI (`Node`, `ImageNode`, `UiTransform`, `Interaction`), serde JSON, existing anchor/pivot helpers.

**Spec:** [`docs/superpowers/specs/2026-05-20-presentation-editor-design.md`](../specs/2026-05-20-presentation-editor-design.md)

**Author workflow (refined):** [`docs/presentation-editor-workflow.md`](../../presentation-editor-workflow.md)

---

## Wave 2 — Atmosphere-first iteration (approved; not started)

**Design-only refinement 2026-05-20.** No code in this section until execution is explicitly scheduled. Supersedes *depth* priority for follow-on PRs; Phases 0–4 below remain the **delivery history**.

**Direction:** Editor evolves from debug transform tooling → **lightweight atmospheric scene composition**. Largest ROI is **atmosphere and responsiveness**, not more gizmo chrome.

### Approved execution order

| # | Task theme | Spec / workflow reference |
|---|------------|---------------------------|
| 1 | Softer radial glow pass | Spec § Fire & atmosphere; workflow § Next-wave priority |
| 2 | Fire breathing / flicker polish | Tracks + frequency bands; stagger phases |
| 3 | Smooth drag interaction | Mouse threshold, hover, optional damped delta |
| 4 | Layered atmosphere (embers, props, washes) | Composable layers only — no monolith entities |
| 5 | Curve tuning UX in overlay | Author `PresentationTrack` without raw JSON-only |
| 6 | Advanced gizmos (`gizmo.rs`) | Phase 5 — **after** 1–4 read well at normal zoom |

### Wave 2 constraints (carry forward)

- **Runtime budget unchanged:** no per-frame allocs, no spawn/despawn per tick, mutate in place, breathing-not-noise.
- **Readability-first:** combat/UI beats atmosphere when they conflict.
- **Layered standard:** new visuals = registry row + `PresentationLayerHost` + JSON tune path.

### Future awareness (no Wave 2 tasks)

| Concept | Spec section | Implementation |
|---------|--------------|----------------|
| Undo / redo / snapshots | § Iteration safety (future) | None |
| Solo-layer preview | § Solo-layer preview (future) | None |
| Presentation states | § Presentation States (future) | None |
| Scene presets | § Presentation scene presets (future) | None |

### Wave 2 documentation checklist (done in refinement pass)

- [x] Workflow philosophy + atmosphere pipeline framing
- [x] Spec: editor direction, Wave 2 priority, layered effects standard
- [x] Spec: future undo + solo-layer sections (clearly separated)
- [x] Plan: this addendum; initial phases unchanged as history

---

## File map (created / modified)

| File | Responsibility |
|------|----------------|
| `src/presentation/element.rs` | `PresentationElementId`, `PresentationElementTune`, serde |
| `src/presentation/layer.rs` | `element:layer` ids, `TITLE_CAMP_LAYER_REGISTRY`, figure/extra layer tunes |
| `src/presentation/scene.rs` | `PresentationScene`, load/save, `layer_tune_mut` routing |
| `src/presentation/track.rs` | `PresentationTrack`, `CurveLayer`, `CurveKind`, `evaluate` |
| `src/presentation/editor/mod.rs` | Plugin registration, resources |
| `src/presentation/editor/overlay.rs` | Hierarchy + inspector + banner UI |
| `src/presentation/editor/selection.rs` | Hover/selection sync with session |
| `src/presentation/editor/mouse.rs` | Drag-to-move offsets |
| `src/presentation/editor/gizmo.rs` | Phase 5 advanced overlays (stub ok) |
| `src/presentation/fire.rs` | Layer spawn + `tick_ambient` (migrate from title-specific) |
| `src/ui/scene_tune.rs` | Type aliases + sync systems delegating to scene/fire |
| `src/ui/components.rs` | `PresentationElementHost`, editor markers |
| `src/ui/title_camp.rs` | Host markers on tunable nodes |
| `src/ui/mockup_layout.rs` | Settings toggle; spawn editor overlay root |
| `src/ui/mod.rs` | Register presentation systems on `GameState::Title` |
| `assets/ui/fire_glow_radial.png` | Soft radial alpha texture |
| `assets/tuning/title_scene.example.json` | Track examples (phase 4) |
| `docs/presentation-editor-workflow.md` | Author workflow |

---

## Phase 0 — Presentation types & title JSON adapter

### Task 0: `PresentationElementTune` and scene container

**Files:**
- Create: `src/presentation/element.rs`
- Create: `src/presentation/scene.rs`
- Modify: `src/presentation/mod.rs`
- Modify: `src/ui/scene_tune.rs`

- [ ] **Step 1: Add `element.rs`**

```rust
// src/presentation/element.rs
use crate::presentation::pivot::ScenePivot;
use serde::{Deserialize, Serialize};

pub type PresentationElementId = String;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct PresentationElementTune {
    pub offset_x: f32,
    pub offset_y: f32,
    pub scale_x: f32,
    pub scale_y: f32,
    pub size_basis: f32,
    pub rotation_deg: f32,
    pub exposure: f32,
    pub glow: f32,
    pub bloom: f32,
    pub global_z: i32,
    pub pivot: ScenePivot,
    pub anchor_ref: Option<String>,
}
```

- [ ] **Step 2: Add `scene.rs` with `PresentationScene` mirroring current layout fields**

Keep on-disk JSON shape: top-level `fireplace`, `lead_slot`, `ally_slot`, `anchors`, `fire_presentation`, `ambient`. Internally map to `PresentationScene { scene_id, elements, anchors, fire_presentation, ambient }` or keep struct fields and add `scene_id: "title_camp"` default.

- [ ] **Step 3: Type aliases in `scene_tune.rs` for gradual migration**

```rust
pub type TitleSceneLayout = crate::presentation::scene::TitleCampSceneLayout;
pub use crate::presentation::element::PresentationElementTune as TitleUiElementTune;
```

Reuse existing `TitleCampSceneLayout` name as serde struct in `scene.rs` to avoid breaking `title_scene.json` field names in one step.

- [ ] **Step 4: Run tests**

Run: `cargo test title_scene_layout_serde`
Expected: PASS (existing test in `scene_tune.rs`)

- [ ] **Step 5: Commit**

```bash
git add src/presentation/element.rs src/presentation/scene.rs src/presentation/mod.rs src/ui/scene_tune.rs
git commit -m "refactor: introduce PresentationElementTune and scene container"
```

---

### Task 1: `PresentationEditorSession` resource

**Files:**
- Create: `src/presentation/editor/mod.rs`
- Modify: `src/presentation/mod.rs`
- Modify: `src/ui/scene_tune.rs`

- [ ] **Step 1: Define session + gizmo flags**

```rust
#[derive(Resource, Default)]
pub struct PresentationEditorSession {
    pub active: bool,
    pub selected_element: Option<PresentationElementId>,
    pub gizmo_flags: PresentationEditorGizmoFlags,
}

#[derive(Clone, Default)]
pub struct PresentationEditorGizmoFlags {
    pub selection_outline: bool,
    pub pivot_marker: bool,
    pub anchor_marker: bool,
    pub layer_label: bool,
    pub glow_radius_preview: bool,
}
```

- [ ] **Step 2: Replace `TitleSceneTuneSession` with alias or field migration**

`pub type TitleSceneTuneSession = PresentationEditorSession;` and map `layout_mode` → `active`, `target` → `selected_element` via helper methods or rename with serde only for disk (session is not persisted).

- [ ] **Step 3: `cargo check`**

Run: `cargo check`
Expected: compiles

- [ ] **Step 4: Commit**

```bash
git commit -am "feat: add PresentationEditorSession resource"
```

---

## Phase 1 — Presentation editor overlay

### Task 2: Editor UI shell on Title state

**Files:**
- Create: `src/presentation/editor/overlay.rs`
- Modify: `src/presentation/editor/mod.rs`
- Modify: `src/ui/components.rs`
- Modify: `src/ui/mockup_layout.rs`
- Modify: `src/ui/title_camp.rs` or title spawn path
- Modify: `src/ui/mod.rs`

- [ ] **Step 1: Add components**

```rust
#[derive(Component)]
pub struct PresentationEditorRoot;

#[derive(Component)]
pub struct PresentationEditorHierarchyButton(pub PresentationElementId);
```

- [ ] **Step 2: `spawn_presentation_editor_overlay`**

Full-screen `Node` with `GlobalZIndex` high; children:
- Top banner: `Presentation Mode · {selected_id}`
- Left column: buttons/labels for `fireplace`, `lead_slot`, `ally_slot` (read ids from `TitleCampSceneLayout` / scene registry)
- Right column: read-only or button-step fields for selected `PresentationElementTune` (mirror hotkey deltas: ±1 position, ±0.01 scale)
- Footer: Save, Reload buttons (call existing `try_save_to_disk` / `try_load_from_disk`)

Spawn on `OnEnter(GameState::Title)` when `#[cfg(debug_assertions)]` or always with visibility tied to `session.active`.

- [ ] **Step 3: System `sync_presentation_editor_inspector`**

When `PresentationEditorSession` or `TitleSceneLayout` changes, update banner text and inspector labels from `layout.tune(selected)`.

- [ ] **Step 4: Settings modal toggle**

In `spawn_settings_modal` (`mockup_layout.rs`), add debug-only row:

```text
[ ] Presentation editor
```

Toggle `PresentationEditorSession.active` on click; when enabling, show overlay root (`Visibility::Visible`).

Wire `title_scene_tune_hotkeys` backtick to flip `session.active` (same flag).

- [ ] **Step 5: First-visit toast in banner**

Replace log-only hint: when `TitleSceneTuneHintLogged` is false, set banner subtext to hotkey summary once.

- [ ] **Step 6: Manual verify**

Run: `cargo run`
On Title: open Settings → enable Presentation editor → see panels; Tab/backtick still cycle targets.

- [ ] **Step 7: Commit**

```bash
git commit -am "feat: presentation editor overlay and settings toggle"
```

---

## Phase 2 — Mouse select and drag

### Task 3: `PresentationElementHost` markers

**Files:**
- Modify: `src/ui/components.rs`
- Modify: `src/ui/title_camp.rs`
- Create: `src/presentation/editor/selection.rs`
- Create: `src/presentation/editor/mouse.rs`

- [ ] **Step 1: Marker component**

```rust
#[derive(Component)]
pub struct PresentationElementHost(pub PresentationElementId);
```

Attach to fireplace host and figure roots in `title_camp.rs` (`"fireplace"`, `"lead_slot"`, `"ally_slot"`).

- [ ] **Step 2: `presentation_editor_pick`**

On `Pointer<Click>` / `Interaction::Pressed` when `session.active`, if hit entity has `PresentationElementHost`, set `selected_element` and refresh hierarchy highlight.

- [ ] **Step 3: `presentation_editor_drag`**

When active and pointer down on selected host:
- Store drag start in session or local resource
- Each move: `offset_x/y += delta` (shift ×10)
- Mark `TitleSceneLayout` changed; existing `sync_title_scene_elements` applies

Use `UiGlobalTransform` or pointer delta in screen space; do not spawn entities per frame.

- [ ] **Step 4: Hover outline**

In `title_scene_tune_selection_gizmo` or `selection.rs`, light border when `Interaction::Hovered` and editor active.

- [ ] **Step 5: Register systems in `ui/mod.rs`**

```rust
.run_if(in_state(GameState::Title))
.before(UiSystems::Focus)
```

Chain: pick → drag → sync → gizmo.

- [ ] **Step 6: `cargo test` + manual drag test**

- [ ] **Step 7: Commit**

```bash
git commit -am "feat: mouse pick and drag for presentation elements"
```

---

## Phase 3 — Atmospheric fire (radial glow + ground wash)

### Task 4: Radial glow asset

**Files:**
- Create: `assets/ui/fire_glow_radial.png`
- Modify: `assets/ui/README.md`

- [ ] **Step 1: Add PNG**

128×128 (or 256×256) RGBA: white center alpha ~0.9, smooth falloff to transparent edges. No hard rectangle edges. Can be authored in any paint tool or generated once via a small script committed separately.

- [ ] **Step 2: Load in `placeholder_graphics` or title asset load path**

Ensure `AssetServer::load("ui/fire_glow_radial.png")` resolves (same pattern as `Fireplace.png`).

- [ ] **Step 3: Commit asset**

```bash
git add assets/ui/fire_glow_radial.png assets/ui/README.md
git commit -m "assets: add radial fire glow texture"
```

---

### Task 5: Rewire fire layers C + D

**Files:**
- Modify: `src/presentation/fire.rs`
- Modify: `src/ui/scene_tune.rs` (`sync_title_fire_presentation_from_layout`, `tick_title_fire_ambient`)

- [ ] **Step 1: Replace glow `BackgroundColor` rect with `ImageNode`**

Spawn glow child using `fire_glow_radial.png`, `NodeImageMode::Auto`, warm tint `Color::srgba(1.0, 0.55, 0.18, alpha)`, large negative margins for soft falloff, `BorderRadius` not relied upon for shape.

- [ ] **Step 2: Ground wash — wider ellipse, lower contrast edge**

Increase width ratio (~1.5× base), reduce height, use elliptical `BorderRadius` + lower peak alpha; avoid flat top edge (gradient-only feel).

- [ ] **Step 3: Enforce minimum glow alpha floor**

In `tick_ambient`, clamp glow alpha ≥ `glow_min_alpha` (new field default ~0.08) per spec.

- [ ] **Step 4: Tune defaults in `TitleFirePresentationTune`**

Frequencies in **0.15–0.35 Hz** band per spec; reduce `ground_flicker_hz` if currently ~1.65 until phase 4 tracks (or lower to ~0.25 for breathing).

- [ ] **Step 5: Visual verify**

Run game, Title camp: glow should not read as a sharp orange rectangle.

- [ ] **Step 6: Commit**

```bash
git commit -am "feat: radial glow and softer ground wash for campfire"
```

---

## Phase 4 — `PresentationTrack` framework

### Task 6: Track evaluator + tests

**Files:**
- Create: `src/presentation/track.rs`
- Modify: `src/presentation/mod.rs`

- [ ] **Step 1: Implement types** (see spec `CurveBlendMode`, `CurveKind` enum: `Sine`, `Triangle`, `Saw`, `SmoothNoise`, `RandomPulse`, `Ease`, `Envelope`)

- [ ] **Step 2: `PresentationTrack::evaluate(t_secs, seed) -> f32`**

Order: start `base_value`; foreach layer sample ∈ [-1,1] or [0,1] per kind; apply additive/multiplicative per spec; clamp.

`SmoothNoise`: deterministic hash noise from `floor(t * freq)` and seed — no `thread_rng`.

- [ ] **Step 3: Unit tests in `track.rs`**

```rust
#[test]
fn track_additive_layers_are_deterministic() {
    let track = PresentationTrack { base_value: 0.2, layers: vec![...] };
    let a = track.evaluate(1.25, 42);
    let b = track.evaluate(1.25, 42);
    assert_eq!(a, b);
}

#[test]
fn legacy_scalar_maps_to_default_track() {
    // glow_pulse_hz 0.22 → sine layer in 0.15-0.35 band
}
```

Run: `cargo test presentation::track` or `cargo test track_`
Expected: PASS

- [ ] **Step 4: Commit**

```bash
git add src/presentation/track.rs
git commit -m "feat: PresentationTrack evaluator with deterministic tests"
```

---

### Task 7: JSON tracks + fire tick rewire

**Files:**
- Modify: `src/presentation/scene.rs` (fire_presentation serde)
- Modify: `src/presentation/fire.rs`
- Modify: `assets/tuning/title_scene.example.json`

- [ ] **Step 1: Extend `TitleFirePresentationTune`**

Optional fields:

```json
"glow_alpha": { "base_value": 0.22, "layers": [...] }
```

Keep legacy `glow_pulse_hz`, `glow_pulse_scale`, etc.; on load, if `glow_alpha` absent, synthesize track from scalars.

- [ ] **Step 2: Rename `tick_title_fire_ambient` → `tick_fire_ambient`**

Evaluate `glow_alpha` and `ground_alpha` tracks; stagger flame layer phases; respect frequency band table in spec.

- [ ] **Step 3: Update example JSON** with staggered sine + smooth_noise layers (0.22 Hz and 0.17 Hz, different phases).

- [ ] **Step 4: `cargo test` full suite**

Run: `cargo test`
Expected: all pass; domain tests unchanged

- [ ] **Step 5: Commit**

```bash
git commit -am "feat: compositional presentation tracks for fire ambient"
```

---

## Phase 5 — Advanced gizmos (deferred → Wave 2 #6)

### Task 8: Gizmo flags + precision handles (optional follow-up PR)

**Sub-layer hierarchy selection:** largely **delivered** via `TITLE_CAMP_LAYER_REGISTRY`, `PresentationLayerHost`, and `layer_tune_mut` (see `src/presentation/layer.rs`). Remaining Phase 5 work is **precision gizmos only**.

**Files:**
- Modify: `src/presentation/editor/gizmo.rs`
- Modify: `src/presentation/editor/overlay.rs`

- [ ] **Step 1: Implement pivot/anchor markers when flags enabled**

- [ ] **Step 2: Glow-radius preview ring tied to fireplace glow layer**

- [ ] **Step 3: Corner scale / rotation handles (if scoped)**

**Gate:** Wave 2 items **1–4** (atmosphere + drag) stable at normal viewing distance. Do not start Task 8 before that gate.

---

## Documentation & changelog

### Task 9: Docs

**Files:**
- Create: `docs/presentation-editor-workflow.md`
- Modify: `docs/presentation-scene-composition.md`
- Modify: `CHANGELOG.md`

- [ ] **Step 1: Workflow doc** — Settings toggle, hierarchy, inspector, mouse drag, save/reload, frequency band guidance, runtime budget summary.

- [ ] **Step 2: Update composition doc** — link editor, tracks, radial glow.

- [ ] **Step 3: CHANGELOG entry** under Unreleased.

- [ ] **Step 4: Commit**

```bash
git add docs/ CHANGELOG.md
git commit -m "docs: presentation editor workflow and composition updates"
```

---

## Plan self-review (spec coverage)

### Initial wave (Phases 0–4 + registry)

| Spec section | Task(s) |
|--------------|---------|
| Runtime budget | Tasks 5–7 (in-place tick, no spawn/frame, tracks) |
| Readability | Task 5 defaults + Task 7 staggered frequencies |
| Frequency bands | Task 6–7 tests + example JSON |
| Editor overlay | Task 2 |
| Mouse editing | Task 3 |
| Fire atmosphere (baseline) | Tasks 4–5 |
| PresentationTrack | Tasks 6–7 |
| Sub-layer selection | Registry + hosts (post-plan; not Task 8) |
| Presentation States (future) | Spec only |
| Scene presets (future) | Spec only |
| Phase order 0→4 | Delivered per plan |
| Phase 5 gizmos | Deferred → **Wave 2 #6** |

### Wave 2 refinement (2026-05-20)

| Spec section | Plan location |
|--------------|---------------|
| Editor direction / philosophy | Workflow doc + spec § Editor direction |
| Next execution priority 1→6 | § Wave 2 above |
| Living atmosphere pipeline | Workflow + spec § Living atmosphere pipeline |
| Layered effects standard | Spec § Layered effects standard |
| Undo / redo (future) | Spec only — no tasks |
| Solo-layer preview (future) | Spec only — no tasks |
| Runtime budget preserved | Wave 2 constraints |

**Overlap control:** Sub-layer **tree** is not Phase 5 anymore; Phase 5 = gizmo precision only. Atmosphere polish is **not** a duplicate of Tasks 4–5 — Wave 2 is a **quality pass** on top of delivered baseline.

**Placeholder scan:** None.

---

## Execution handoff

**Initial plan:** Phases 0–4 executed per task list above; Phase 5 gizmos remain open.

**Approved next work:** **Wave 2** (atmosphere-first, spec-aligned order 1→6). Use subagent-driven or inline execution when starting implementation — begin with **#1 softer radial glow**, not Task 8 gizmos.

Refinement docs ready for execution; **no implementation** in the refinement pass itself.
