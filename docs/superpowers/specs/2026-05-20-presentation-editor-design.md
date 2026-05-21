# Presentation Editor & Atmosphere Framework (2026-05-20)

## Status

**Approved design.** First implementation target: **title camp scene** only. Architecture must generalize for combat theater, dungeon staging, props, VFX, and evolving camp composition without title-specific naming in core modules.

**Initial wave (Phases 0–4 + registry sub-layers):** delivered — discoverable editor, mouse editing, radial glow stack, `PresentationTrack` on fire ambient, `element:layer` hierarchy. See [`docs/presentation-editor-workflow.md`](../../presentation-editor-workflow.md).

**Next wave (approved refinement 2026-05-20):** **atmosphere-first iteration** — polish glow, breathing, drag feel, and layered camp atmosphere **before** advanced gizmos or deep curve UX. Plan addendum: [`docs/superpowers/plans/2026-05-20-presentation-editor.md`](../plans/2026-05-20-presentation-editor.md) § Wave 2.

---

## Problem

Delvers already has a strong presentation foundation: JSON persistence, anchors/pivots, hot reload, layered fire spawn, and an in-engine presentation editor. Remaining gaps are **emotional payoff** and **iteration velocity**: the camp can still read as geometric UI layers, and tooling can still feel like debug layout mode instead of **atmospheric scene composition**.

Goals for the overall initiative (unchanged):

1. **Discoverable in-engine editor** — not hidden behind backtick-only hints.
2. **Mouse-driven scene editing** — complement keyboard nudging.
3. **Atmospheric fire/glow** — environmental softness before advanced gizmos.
4. **Composable presentation curves** — reusable, data-driven motion across systems.
5. **Future-ready boundaries** — presentation states, camp evolution, shader hooks without real-time lighting.

**Follow-on emphasis (next wave):** maximize **immediate visual feedback** so the editor is actively used; defer CAD-like precision tooling.

---

## Editor direction: atmospheric composition workflow

The presentation editor should evolve from **debug transform tooling** into a **lightweight atmospheric scene composition workflow**.

| Optimize for | De-prioritize |
|--------------|----------------|
| Rapid experimentation | CAD-like precision |
| Low friction (toggle, drag, save) | Heavyweight scene authoring |
| Emotional visual iteration | Multi-handle transform suites |
| Safe tweaking (reset, reload; future undo) | Effect spam and busy motion |
| Immediate live feedback | Export/rebuild loops |

Delvers presentation remains **handcrafted, painterly, atmospheric, and organic**. Authoring guidance: [`docs/presentation-editor-workflow.md`](../../presentation-editor-workflow.md) § Workflow philosophy.

---

## Living atmosphere pipeline (conceptual framing)

Delvers is building a **layered atmospheric presentation pipeline** (presentation-only; simulation stays deterministic):

```
Scene composition (anchors, elements, JSON)
    → Layered stacks (element:layer hosts)
    → Runtime modulation (PresentationTrack)
    → Editable transforms (editor)
    → Atmospheric presets (fire, ambient, future props)
    ── separate from ──
Domain / combat / playback truth
```

The editor and tooling exist to support:

- **Iteration velocity** and polish passes
- **Emotional scene building** (camp hub, later theater)
- **Progression visualization** (tents, forge, trophies as layered elements)

Contributors should treat this as **infrastructure for atmosphere**, not a general-purpose level editor.

---

## Layered effects standard (composable over monolithic)

**Prefer composable layered effects over giant monolithic effect entities.**

| Pattern | Example (title camp) |
|---------|----------------------|
| Separate layers per role | `fireplace:stack`, `base`, `flame`, `glow`, `ground` |
| Shared modulation | `PresentationTrack` per property path |
| Registry + spawn | `TITLE_CAMP_LAYER_REGISTRY` + `PresentationLayerHost` |

Future camp/theater assets should add **layers** (glow, ember, fog, lantern halo, boss aura wash) rather than one opaque “effect node.” Monoliths hide tuning, break isolation (see solo-layer preview), and encourage synchronized buzzing.

Reference stack: `src/presentation/fire.rs` + `docs/presentation-scene-composition.md`.

---

## Next execution wave (approved priority)

**Do not** add deep gizmo/handle complexity until atmosphere reads well at normal zoom. Approved order for follow-on implementation:

| # | Deliverable | Rationale |
|---|-------------|-----------|
| 1 | Softer radial glow system | Largest fix for “UI rectangle” fire read |
| 2 | Better fire breathing / flicker | Emotional ROI; uses tracks + frequency bands |
| 3 | Smooth drag interaction | Editor friction reduction |
| 4 | Layered atmosphere improvements | Embers, props, washes — composable layers |
| 5 | Curve tuning UX | Author tracks in-editor without JSON-only loops |
| 6 | Advanced gizmos + sub-layer handles | Precision after atmosphere rewards iteration |

This **reorders depth work** relative to the original phase table (below): Phases 1–4 established **capability**; Wave 2 optimizes **payoff and usability**.

Original phase table remains the **historical delivery sequence**; Wave 2 is the **active execution priority**.

---

## Architectural placement

Delvers separates concerns deliberately:

| Layer | Responsibility | Must not |
|-------|----------------|----------|
| **Domain / simulation** | Deterministic rules, combat, loot, progression | Import presentation or UI |
| **Event playback** | Replay simulation outcomes for display | Mutate simulation truth |
| **Presentation** | Scene composition, anchors, layers, curves, fake lighting | Drive gameplay outcomes |
| **Presentation editor** | Tooling overlay: select, inspect, drag, save/reload | Ship as required release UI (initially debug-oriented) |

**Approach B (approved):** modular code under `src/presentation/` with thin scene-specific wiring in `src/ui/`. Avoid growing `scene_tune.rs` into a monolith; migrate title camp to consume generic types while keeping `assets/tuning/title_scene.json` as the first on-disk format.

---

## Presentation runtime budget

Presentation must stay **lightweight, stable, and layered** — not an uncontrolled effect system. This applies to the title camp work and all future presentation (particles, lanterns, fog, combat pulses, shrine effects, floating combat text modulation, boss auras, biome overlays).

**Discipline:**

- **Favor low-frequency modulation** — slow breathing beats busy flicker (see frequency bands under curves).
- **Favor layered illusion over expensive rendering** — tinted UI layers, radial textures, alpha stacks; no real-time lights in this era.
- **Minimize allocations during presentation ticks** — evaluate tracks into stack locals; avoid `Vec` growth per frame; reuse buffers where layering grows.
- **Avoid spawn/despawn every frame** — presentation entities are created at scene enter (or explicit hot-reload), then mutated in place.
- **Avoid simulation-coupled presentation updates** — combat/domain never drives per-frame VFX spawn; playback/UI may set presentation *state* tags later, not RNG or damage formulas.
- **Prefer reusable `PresentationTrack`s over bespoke sine in each system** — one evaluator, many properties.
- **Maintain stable frame pacing** as scene complexity grows — cap simultaneous high-frequency layers; defer particles until budget rules exist.

Future subsystems (particles, lanterns, fog, etc.) **must** respect this budget before merging.

---

## Presentation readability

Delvers is becoming more atmospheric, layered, animated, and visually dense. Readability must be **protected intentionally** as camp and combat presentation grow.

**Constraints:**

- **Combat readability overrides atmosphere** — telegraphs, HP, targets, and log/summary clarity win over glow and pulse.
- **UI readability survives dark painterly backdrops** — text, buttons, and panels keep contrast; presentation wash must not swallow interactive chrome.
- **Interactive / tunable elements stay identifiable** — selection outlines, hover, and editor chrome remain distinct from in-world atmosphere.
- **Motion layers rarely compete spatially** — glow, ground wash, and flame occupy related but non-identical regions; avoid duplicate pulsing at the same focal point.
- **Avoid synchronized frequencies** — stagger `frequency_hz` / `phase` across layers on the same element so motion does not “buzz” in unison.
- **Curves favor slow, readable modulation** — prefer environmental drift and fire breathing bands; constant high-frequency activity is visual noise.
- **Effects support “breathing atmosphere,” not noise** — motion should feel like the world is alive, not that the UI is broken.

The editor and track defaults should encode these constraints (conservative frequency bands, modest amplitudes).

---

## Naming & module map (generalized)

Do **not** introduce permanent public names tied to “Title Camp” or “Campfire” in core presentation APIs.

| Concept | Preferred name | Notes |
|---------|----------------|-------|
| Composed view | `PresentationScene` | A logical scene instance (title camp today; combat theater later) |
| Tunable object | `PresentationElement` | Host node + metadata (id, layer, anchor ref, transform tune) |
| Visual stack slice | `PresentationLayer` | e.g. base, flame, glow, ground wash — ordered child group |
| Spatial reference | `PresentationAnchor` | Named pose in scene space (replaces ad-hoc anchor map entries) |
| Time modulation | `PresentationTrack` | `base_value` + composable `CurveLayer` list |
| Layer in a track | `CurveLayer` | Kind, frequency, amplitude, phase, blend mode, weight |
| Tooling shell | `PresentationEditor` | Overlay UI, selection session, gizmo flags |
| Sampling | `CurveSampler` / track evaluator | Pure functions of time (+ optional seed); no ECS in core |

**Acceptable scene-specific code (outer layer):**

- `TitleCampPresentationScene` — bootstrap/spawn wiring only
- `title_scene.json` — first persisted scene file
- Markers like `CampfirePresentationRoot` may remain as **spawn markers** until renamed; new APIs use `PresentationElementId` or scene registry keys

**Avoid:** `TitleCampfireEditor`, `TitleSceneOnly`, `CampfireGlowSystem`, and other names that imply a single screen owns the framework.

### Target module layout

```
src/presentation/
  mod.rs
  anchor.rs          # PresentationAnchor, resolve poses (existing logic generalized)
  pivot.rs           # unchanged role
  element.rs         # PresentationElement, PresentationElementTune (serde)
  layer.rs           # PresentationLayer, layer kinds, spawn contracts
  scene.rs           # PresentationScene descriptor + element registry
  track.rs           # PresentationTrack, CurveLayer, blend modes, evaluator
  fire.rs            # camp fire stack spawn (presentation-specific preset, not core editor)
  markers.rs         # spawn markers (scene wiring)
  editor/
    mod.rs           # PresentationEditor plugin/resources
    overlay.rs       # hierarchy + inspector panels
    selection.rs     # pick, hover, session target
    mouse.rs         # drag-to-move, future handles
    gizmo.rs         # optional overlays (phase 5+)
```

`src/ui/scene_tune.rs` becomes a **thin adapter**: loads `title_scene.json` into `PresentationScene` / element tunes, registers editor for `AppState::Title`, delegates sync systems.

---

## PresentationScene & elements

### Scene descriptor

A `PresentationScene` (serialized or built at spawn) contains:

- `scene_id` — e.g. `"title_camp"` (extensible: `"combat_theater"`, `"shrine_room"`)
- `elements: Vec<PresentationElement>` — stable string `id` per element (`fireplace`, `lead_slot`, `ally_slot`, future `tent_01`, `lantern_post`)
- `anchors: HashMap<String, PresentationAnchor>` — named poses
- `layers` — optional grouped layer configs (fire stack, ambient wash) referenced by element id
- `tracks` — optional map of property path → `PresentationTrack` (see curves)

### Element tune (presentation-only transform)

`PresentationElementTune` generalizes current `TitleUiElementTune`:

- `offset_x`, `offset_y`
- `scale_x`, `scale_y`
- `size_basis`, `rotation_deg`
- `exposure`, `glow`, `bloom` (bloom reserved; no post-process requirement)
- `global_z`
- `pivot`, `anchor_ref`
- Future hooks: `tint`, `modulation` (scalar 0..1 for shader-less emissive illusion)

**Gameplay fields never appear here.**

### Camp extensibility (long-term)

Title camp is the emotional hub and should evolve with progression (larger fire, tents, trophies, recruits, forge, lanterns, weapon racks). The scene model must allow:

- **Adding elements** by id without recompiling editor code (hierarchy driven from scene registry + JSON)
- **Anchor growth** — new named anchors (`forge_corner`, `banner_slot`) without breaking old saves (serde defaults)

First implementation still spawns only fireplace + two figure slots; registry and editor hierarchy list **must** be data-driven so new ids are JSON + spawn wiring, not new enum variants in core editor.

Scene preset / variant swapping is specified under **Presentation scene presets (future)** below.

---

## Fake lighting policy (explicit)

**Do not** implement real-time dynamic lighting, shadow casters, or light probes in this initiative.

Delvers’ painterly dark style is served by:

- Layered gradients and radial textures
- Emissive **illusion** (tint + exposure + additive alpha stacks)
- Alpha modulation and subtle scale breathing
- Controlled warm/cool tinting near fake light origins

Real-time lighting is deferred indefinitely unless art direction revisits globally. JSON may store scalar hooks (`emissive_strength`, `heat_distortion` defaulting to 0) for **future** shaders; no shader work in initial phases.

---

## Presentation Editor

### Discoverability

- **Settings** (debug builds initially): toggle **“Presentation editor”** — same session flag as backtick layout mode; both entry points stay equivalent.
- **Visible chrome** when active: banner `Presentation Mode · <element id>`; editor panels; save/reload buttons.
- **First visit hint:** one-line toast in editor chrome **and** log line (replace backtick-only discoverability).

### Overlay layout

Full-screen Bevy UI overlay; pointer hits panels; stage passthrough when not dragging.

| Left (~220px) | Center | Right (~280px) |
|---------------|--------|----------------|
| **Hierarchy** — tree from `PresentationScene` element registry | Existing camp stage (unchanged draw order) | **Inspector** — fields for selected `PresentationElementTune` |

Hierarchy for phase 1 (title camp):

- `fireplace`
- `lead_slot`
- `ally_slot`

Sub-layer editing: **delivered** via `element:layer` ids and `TITLE_CAMP_LAYER_REGISTRY` (e.g. `fireplace:glow`, `lead_slot:emoji`). Further hierarchy entries follow the same registry + host pattern for new assets.

### Inspector fields

Mirror element tune: position, scale, rotation, size basis, opacity proxy (exposure), glow, bloom (stored), global Z, pivot dropdown, anchor_ref dropdown from scene anchor keys.

Actions: **Save** (Ctrl+S), **Reload** (F5), duplicate existing disk paths (`assets/tuning/title_scene.json`).

### Session resource

`PresentationEditorSession` (generalized from `TitleSceneTuneSession`):

- `active: bool`
- `selected_element: Option<ElementId>`
- `gizmo_flags: PresentationEditorGizmoFlags` (advanced overlays; mostly off until phase 5)
- Optional: `show_curve_debug` (future)

Keyboard hotkeys remain in debug builds; editor is **additive**, not a replacement.

---

## Mouse editing (phase 2)

| Interaction | Behavior |
|-------------|----------|
| Click host | Select element; sync hierarchy + inspector |
| Drag selected | Delta pointer → `offset_x` / `offset_y` (1 px = 1 unit; shift = 10×) |
| Hover | Light outline (distinct from selection) |

Hit testing: `Interaction` + global UI rect on entities marked as `PresentationElementHost`. No simulation imports.

**Deferred to phase 5 (advanced gizmos):** corner scale handles, rotation handle, pivot crosshair, anchor diamond, glow radius preview ring.

---

## Fire & atmosphere (phase 3 — prioritized before curves in *visible* payoff, after editor + mouse)

### Problem statement

The orange glow issue is **not primarily color** — it is **readable geometry**: UI rectangles with hard bounds. Goal: glow feels **environmental**, not “a UI node.”

### Layer model (unchanged roles, better visuals)

| Layer | Role | Direction |
|-------|------|-----------|
| **A — Static base** | Logs/stones/ember art | Mostly static |
| **B — Flame motion** | Subtle opacity + vertical scale breathe | Low-frequency; avoid rapid loop frames |
| **C — Glow** | Radial soft falloff | **Replace** `BackgroundColor` rect with radial gradient **texture** (`assets/ui/fire_glow_radial.png` or procedurally layered alpha circles); heavy border-radius alone is insufficient |
| **D — Ground wash** | Warm elliptical pool | Soft ellipse, overlapping alpha, no hard top edge; wider than fire base |

### Visual rules

- No fully sharp rectangular silhouettes at normal viewing distance
- Overlapping soft layers; additive-feeling warm tint
- Glow **never fully disappears** (minimum alpha floor)
- Motion: slow pulse, gentle scale breathe — not arcade torch loops

### Tuning surface

`FirePresentationPreset` (or nested under scene JSON `fire_presentation`) holds scalars consumed by layers and tracks:

- `glow_max_alpha`, radial scale, color warm bias
- `ground_max_alpha`, ellipse aspect
- `flame_variants`, crossfade period
- Shader-ready zeros: `heat_distortion`, `bloom_strength` (unused)

`tick_fire_ambient` (renamed from title-specific) reads evaluated tracks when available; until phase 4, may still use legacy scalar sin mapping internally.

---

## PresentationTrack & compositional curves (phase 4)

### Design principle

**Do not** center the framework on `enum CurveKind` alone as the sole driver per property. Long-term presentation needs **layered, composable modulation**: sine + noise + envelope + future state overrides on the same property.

### Naming note (`CurveKind`)

No rename in initial implementation. Acknowledge that `CurveKind` may later evolve semantically to **`ModulationKind`** or **`WaveformKind`** if the system grows beyond classic mathematical curves (envelopes, noise fields, state-driven pulses). Plan serializers with stable string tags so a rename is non-breaking.

### Recommended frequency bands

Protect Delvers’ **heavy, slow, grounded, breathing** feel — not hyperactive or arcade-like. Authoring guidance for `CurveLayer.frequency_hz`:

| Band (Hz) | Typical use | Notes |
|-----------|-------------|--------|
| **0.03 – 0.12** | Environmental drift | Fog bias, distant ambient, camp “air” |
| **0.15 – 0.35** | Fire / light breathing | Glow alpha, ground wash, flame crossfade |
| **0.5 – 1.2** | UI emphasis / reactive pulses | Lantern hitch, low-health hint, boss intro accent |
| **2+** | Avoid except special effects | Rare; never default for camp fire or idle atmosphere |

When stacking layers on one property, **stagger frequencies and phases** (readability: avoid synchronized buzzing). Defaults and editor examples should sit in the fire/light band unless explicitly marked reactive.

### Core types

```rust
/// Evaluates to a single f32 at time t.
pub struct PresentationTrack {
    pub base_value: f32,
    pub layers: Vec<CurveLayer>,
}

pub struct CurveLayer {
    pub kind: CurveKind,           // sine, triangle, saw, smooth_noise, random_pulse, ease, envelope, ...
    pub frequency_hz: f32,
    pub amplitude: f32,
    pub phase: f32,
    pub weight: f32,
    pub blend: CurveBlendMode,     // Additive | Multiplicative
}

pub enum CurveBlendMode {
    Additive,
    Multiplicative,
}
```

### Evaluation order (deterministic)

1. Start with `base_value`.
2. For each layer in order:
   - Sample `layer.kind` at `t` with optional **scene seed** (integer from scene or element; no per-frame `thread_rng`).
   - Apply **additive**: `value += sample * amplitude * weight`
   - Apply **multiplicative**: `value *= 1.0 + (sample * amplitude * weight)` (exact formula locked in implementation plan; must be stable and tested).

3. Clamp or map to property range (document per property: alpha 0..1, scale ≥ 0, etc.).

`CurveSampler` is a thin wrapper; logic lives on `PresentationTrack::evaluate(t, seed)`.

### JSON (backward compatible)

Existing `title_scene.json` scalar fields (`glow_pulse_hz`, `ground_flicker_hz`, …) **remain valid**. Loader:

- If `tracks` / nested track objects absent → synthesize default `PresentationTrack` from legacy scalars.
- If present → use compositional tracks.

Example (illustrative):

```json
"fire_presentation": {
  "enabled": true,
  "glow_alpha": {
    "base_value": 0.22,
    "layers": [
      { "kind": "sine", "frequency_hz": 0.22, "amplitude": 0.04, "weight": 1.0, "blend": "additive" },
      { "kind": "smooth_noise", "frequency_hz": 0.15, "amplitude": 0.03, "weight": 0.6, "blend": "additive" }
    ]
  }
}
```

### Property paths

Tracks attach to **logical property paths** (string keys or typed enum in Rust):

- `fire.glow.alpha`
- `fire.ground.alpha`
- `fire.flame.crossfade`
- Future: `lantern.emissive`, `boss_aura.scale`, `floating_text.modulation`

Avoid one-track-per-enum-variant coupling to a single screen; use a map `HashMap<String, PresentationTrack>` on scene or element.

### Consumers (ordered adoption)

1. Fire glow alpha + ground wash (title camp)
2. Flame crossfade weighting
3. UI pulse / lantern props (later scenes)
4. Combat theater ambient (later)

---

## Presentation States (future — spec awareness only)

**Not implemented** in initial phases. Architecture must **not** prevent blendable, state-driven presentation later.

### Concept

`PresentationState` is a set of active tags with optional blend weights:

| State (examples) | Typical use |
|------------------|-------------|
| `idle` | Camp default |
| `hover` | UI/prop emphasis |
| `combat` | Theater active |
| `boss` | Boss encounter pulse overrides |
| `low_health` | Hero vignette / pulse |
| `victory` | Summary beat |
| `storm` / `night` | Biome variant modifiers |

### Future override model (design sketch)

```rust
// Future — not built now
pub struct PresentationStateOverride {
    pub when: PresentationStateMask,
    pub track_overrides: HashMap<String, PresentationTrack>,
    pub blend_secs: f32,
}
```

Scene may define `state_overrides: Vec<...>` in JSON. Editor shows read-only “States (future)” section. Track evaluator accepts optional `active_states: &[PresentationState]` later to merge override tracks without breaking deterministic simulation (states come from UI/playback, not combat RNG).

**Invariant today:** evaluator API should accept `(t, seed)` only; add optional state slice in a backward-compatible extension.

---

## Presentation scene presets (future — spec awareness only)

**Not implemented** in initial phases. Camp evolution, biome progression, and encounter context should eventually swap or blend whole presentation configurations without rewriting element ids.

### Concept

`PresentationScenePreset` — a named bundle applied to a `PresentationScene`:

- `preset_id` — e.g. `camp_stage_1`, `camp_stage_3_winter`, `shrine_corruption`, `boss_gate`
- **Element overrides** — visibility, alternate asset paths, transform deltas per `element_id`
- **Track overrides** — optional map of property path → `PresentationTrack` replacements
- **Anchor deltas** — optional named anchor adjustments for biome layout
- **Ambient / fire presets** — nested configs (e.g. larger fire, dimmer lanterns)

### Intended uses (later)

- Upgraded camp variants (more tents, forge, trophies) as progression unlocks
- Season / biome themes: winter, corruption, shrine blessing
- Boss encounter presentation variants (theater framing, aura tracks)
- Biome overlays on shared combat theater scenes
- Progression-driven **scene swaps** — `active_preset` on scene load or cross-fade between presets

### Integration sketch (future)

```rust
// Future — not built now
pub struct PresentationScenePreset {
    pub preset_id: String,
    pub element_overrides: HashMap<String, PresentationElementPresetPatch>,
    pub track_overrides: HashMap<String, PresentationTrack>,
}
```

JSON may add `presets: { ... }` and `active_preset: "camp_stage_2"` on the scene file. Editor: read-only preset list until camp evolution ships. Initial title camp uses implicit default preset only.

---

## Advanced gizmos (phase 5 — lowest priority)

`PresentationEditorGizmoFlags` (mostly off until atmosphere wave completes):

- `selection_outline` (default on)
- `pivot_marker`
- `anchor_marker`
- `layer_label`
- `glow_radius_preview`

Corner scale handles, rotation handles, anchor diamonds, and glow-radius rings remain **Wave 2 item #6** — after softer glow, breathing, drag polish, and layered atmosphere.

Sub-layer **selection** in hierarchy is **delivered**; sub-layer **gizmo precision** is not.

---

## Iteration safety (future — not implemented)

Once mouse drag and inspector typing are daily drivers, authors need **safe iteration**:

| Capability | Intent |
|------------|--------|
| **Undo / redo** | Revert last nudge, field edit, or drag delta |
| **Temporary edit snapshots** | Session buffer before commit-to-JSON |
| **Revert-to-last-save** | One action to disk-known-good without hand-editing JSON |

**Architecture awareness only** — no storage format or command stack in the current wave. Session and layout resources should remain structured so a command history can wrap `layer_tune_mut` / `apply_editor_tune_delta` later without domain coupling.

---

## Solo-layer preview (future — not implemented)

As presentation complexity grows (combat theater, particles, fog, UI overlays), tuning one layer in a stack becomes painful. Planned **solo-layer preview** modes:

| Isolate | Use when |
|---------|----------|
| Glow only | Radial falloff and alpha tracks |
| Flame group | Crossfade and tint stacks |
| Particles / embers | Density without wash drowning |
| Ground wash | Ellipse shape vs fire base |
| UI overlays | Theater chrome vs stage atmosphere |

Editor would temporarily hide or dim non-selected layers while keeping host transforms editable. Critical for **combat theater** and **VFX stacks** where everything shares the same focal point.

**Not implemented** — flags or session field reserved in design only (`solo_preview_layer: Option<ElementLayerId>` sketch). No UI in Wave 2 unless explicitly scoped.

---

## Implementation phases (initial delivery — complete)

| Phase | Deliverable | Outcome |
|-------|-------------|---------|
| **0** | Types + `title_scene.json` adapter | Generalized presentation model |
| **1** | `PresentationEditor` overlay, Settings toggle, hierarchy, inspector, save/reload | Discoverable tooling |
| **2** | Mouse select + drag on hosts | Faster layout iteration |
| **3** | Radial glow texture + ground wash softness | Visual quality baseline |
| **4** | `PresentationTrack` + fire tick rewire | Composable motion |
| **5** | Advanced gizmos | **Deferred** — see Wave 2 #6 |

**Active priority:** [Next execution wave](#next-execution-wave-approved-priority) (atmosphere and usability before Phase 5 depth).

Phases 1–3 were allowed to land before curve completeness; that rule still applies to Wave 2 atmosphere tasks (do not block glow polish on curve UX).

---

## Migration from current code

| Current | Target |
|---------|--------|
| `TitleSceneLayout` | `PresentationScene` + title adapter / type alias during transition |
| `TitleUiElementTune` | `PresentationElementTune` |
| `TitleSceneTuneSession` | `PresentationEditorSession` |
| `TitleSceneTuneTarget` enum | `ElementId` string or registry index |
| `scene_tune.rs` sync/hotkeys | Split: generic sync in `presentation/scene.rs`, title wiring in ui |
| `tick_title_fire_ambient` | `presentation::fire::tick_ambient` using tracks when present |
| `docs/presentation-scene-composition.md` | Update + new `docs/presentation-editor-workflow.md` |

Serde aliases on JSON fields preserve existing `title_scene.json` files on disk.

---

## Testing & verification

| Area | Approach |
|------|----------|
| Track evaluation | Unit tests: deterministic samples at fixed `t` + `seed`; additive/multiplicative order |
| JSON load | Round-trip + legacy scalar → synthesized track equivalence (within epsilon) |
| Editor | Manual test checklist on Title state (toggle, select, drag, save, reload, F5) |
| Domain | No new domain tests; presentation must not alter combat tests |

---

## Documentation deliverables (with implementation)

- Update [`docs/presentation-scene-composition.md`](../../presentation-scene-composition.md)
- Add `docs/presentation-editor-workflow.md`
- Extend `assets/tuning/title_scene.example.json` with track examples (phase 4)
- CHANGELOG entry when phases ship

---

## Non-goals (this initiative)

- Real-time lighting, shadows, or light probes
- Full shader pipeline (bloom, heat distortion implementation)
- Combat theater editor (architecture only)
- Asset hot-swap UI (fields may be reserved)
- Presentation state blending implementation
- External editor tools

---

## Success criteria

### Initial wave (delivered / verifying)

1. Artist/designer can open Presentation editor from Settings without knowing backtick.
2. Click-drag moves fireplace, figure slots, and registered sub-layers; JSON persists correctly.
3. `PresentationTrack` can combine sine + noise layers on one property; tests prove determinism.
4. New scene ids and elements can be added via registry + JSON + spawn wiring without renaming core editor types.
5. Domain layer remains free of presentation imports.
6. Presentation ticks do not allocate or spawn/despawn per frame on the title scene path.

### Wave 2 (next execution — atmosphere-first)

7. Fire glow reads **environmental** at normal zoom (soft radial, minimum alpha floor, no obvious UI rect).
8. Fire motion feels like **breathing**, not arcade flicker (bands respected, phases staggered).
9. Drag/layout interaction feels **smooth** enough for repeated tuning sessions.
10. New atmosphere work uses **layered** spawn pattern, not monolithic effect nodes.
11. Default modulation stays in spec frequency bands; readability constraints unchanged.

---

## Spec self-review (refinement 2026-05-20)

| Topic | Location | Notes |
|-------|----------|-------|
| Day-to-day authoring | `presentation-editor-workflow.md` | Philosophy, Wave 2 table, file paths |
| Types, budget, future APIs | This spec | Runtime budget unchanged; future sections labeled |
| Task checkboxes | Implementation plan | Wave 2 addendum only |
| Overlap removed | Phase 5 vs sub-layers | Selection delivered; gizmos still deferred |
| Future vs now | Undo, solo preview, states, presets | Clearly **not implemented** |

---

## References

- Existing: [`docs/presentation-scene-composition.md`](../../presentation-scene-composition.md)
- Code: `src/presentation/`, `src/ui/scene_tune.rs`, `assets/tuning/title_scene.json`
- Visual direction: [`docs/visual-bible-foundation-v1.md`](../../visual-bible-foundation-v1.md)
- Philosophy: [`docs/design-philosophy.md`](../../design-philosophy.md)
