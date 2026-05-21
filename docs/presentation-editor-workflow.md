# Presentation editor workflow (Title camp)

In **debug builds**, Delvers exposes an in-engine **presentation editor** on the Title game state: a fullscreen overlay beside the unchanged title stage so you can select elements, tweak `PresentationElementTune` fields, and save JSON without relying on obscure hotkeys alone. Domain and combat code never depend on this tool.

**Related docs:** conceptual stack in [`presentation-scene-composition.md`](presentation-scene-composition.md); architecture and priorities in [`superpowers/specs/2026-05-20-presentation-editor-design.md`](superpowers/specs/2026-05-20-presentation-editor-design.md); implementation history in [`superpowers/plans/2026-05-20-presentation-editor.md`](superpowers/plans/2026-05-20-presentation-editor.md).

---

## Direction: atmospheric composition, not CAD

The presentation stack is **architecturally sound** (layered hosts, `element:layer` ids, tracks, JSON persistence, simulation separation). The **next execution priority** is not more transform precision — it is making the camp **feel alive** while keeping the editor **rewarding to use**.

| From | Toward |
|------|--------|
| Debug transform tooling | Lightweight **atmospheric scene composition** |
| Precision-first | **Emotional visual iteration** first |
| Monolithic effects | **Composable layered** atmosphere (glow, flame, ember, ground wash, fog, …) |

If editing feels visually responsive, the tool gets used; if it feels like layout homework, it gets avoided.

---

## Workflow philosophy

The editor optimizes for:

- **Rapid experimentation** — tweak, see, tweak again without ceremony
- **Low friction** — Settings toggle, hierarchy, mouse drag, save/reload
- **Emotional visual iteration** — atmosphere and warmth before pixel-perfect placement
- **Safe tweaking** — reset-to-center, reset-all, reload from disk (see [Future: iteration safety](#future-iteration-safety-not-implemented))
- **Immediate feedback** — live stage behind the overlay; no export/rebuild loop

It does **not** optimize for:

- CAD-like precision or multi-handle transform suites
- Heavyweight scene authoring or node-graph complexity
- Overcomplicated gizmo chrome that competes with the camp painting

Delvers presentation stays **handcrafted, painterly, atmospheric, and organic** — the tool serves that identity.

---

## Next-wave priority (atmosphere before editor depth)

**Approved order for follow-on work** (largest emotional ROI first):

| Priority | Focus | Why |
|----------|--------|-----|
| 1 | Softer radial glow system | Removes “UI rectangle” read; environmental warmth |
| 2 | Better fire breathing / flicker | Camp feels alive; uses existing track stack |
| 3 | Smooth drag interaction | Reduces friction during layout passes |
| 4 | Layered atmosphere improvements | Embers, wash, props, progression hooks — **layers**, not monoliths |
| 5 | Curve tuning UX | Inspector/helpers for `PresentationTrack` without leaving the overlay |
| 6 | Advanced gizmos / handles | Pivot rings, scale handles, glow-radius preview — **after** atmosphere pays off |

Do **not** jump to Phase 5-style gizmo depth until 1–4 read well at normal zoom. See spec **§ Next execution wave**.

---

## Living atmosphere pipeline (how the pieces fit)

Delvers is evolving a **layered atmospheric presentation pipeline** (presentation-only; domain stays deterministic):

| Piece | Role |
|-------|------|
| **Scene composition** | Anchors, pivots, element hosts, `title_scene.json` |
| **Layered presentation** | `element:layer` stacks (fire stack, figure emoji, future tent/fog) |
| **Runtime modulation** | `PresentationTrack` + `CurveLayer` on alpha, scale, tint proxies |
| **Editable transforms** | Host + sub-layer offsets/scales via editor |
| **Atmospheric tuning** | Fire preset, ambient scalars, frequency bands |
| **Simulation separation** | Combat/domain never spawns presentation per frame |

The editor exists to support **iteration velocity**, **polish**, **emotional scene building**, and **progression visualization** (camp upgrades, theater framing later) — not to replace art tools.

**Standard pattern:** prefer **composable layers** over one giant effect entity:

- glow layer · flame layer · ember layer · ground wash layer · fog layer · …

The title **fire stack** is the reference implementation; new camp/theater assets should follow the same split + registry entry pattern ([`TITLE_CAMP_LAYER_REGISTRY`](../../src/presentation/layer.rs)).

---

## Settings toggle (debug)

1. From the title hub, open **Settings**.
2. Under the **Debug** section (only when built with debug assertions), use **Presentation editor · OFF / ON**.
3. Flipping ON sets the same **`PresentationEditorSession::active`** flag as **`**`** (backtick) layout mode: the dimmed fullscreen chrome appears over the campfire.

The row tooltip mirrors the overlay: toggling equals “layout mode” from the legacy backtick workflow.

---

## Overlay: hierarchy, inspector, save / reload

When the session is active:

| Area | Role |
|------|------|
| **Banner** | `Presentation Mode · {element id}` — sub-layers show `Layer · fireplace:glow` style ids |
| **Left — Hierarchy** | From [`TITLE_CAMP_LAYER_REGISTRY`](../../src/presentation/layer.rs): hosts + sub-layers (`fireplace:glow`, `lead_slot:emoji`, …). New assets: registry row + [`PresentationLayerHost`](../../src/presentation/markers.rs) on the child node |
| **Right — Inspector** | Click value to type (**Enter** / **Esc**). **− / +** fine steps; **Shift** = 10×. Pivot/anchor summary read-only on hosts |
| **Footer** | **Reset to center**, **Reset all**, **Save** (Ctrl+S), **Reload** (F5) |

Sub-layer tunes: `fire_presentation.layers` (fireplace), `figure_layers` (slot emoji), `extra_layers` (`"tent:panel"` keys for future composites).

**Backtick** toggles layout mode. Persistence: **`assets/tuning/title_scene.json`** (`TitleSceneLayout::try_save_to_disk` / `try_load_from_disk`).

Implementation: `src/presentation/editor/overlay.rs`, `src/ui/mod.rs` (`handle_presentation_editor_overlay_buttons`).

---

## Mouse pick and drag

- **Pick:** left press on a host or sub-layer; topmost by `GlobalZIndex` (`PresentationElementHost`, `PresentationLayerHost`).
- **Drag:** move ≥ few px while held to nudge offsets; **Shift** = 10×. Goal for next wave: **smoother** feel (less threshold jitter, clearer hover) — see plan Wave 2.

Panel picking stays separate from stage hits (`src/presentation/editor/mouse.rs`).

---

## Curve authoring (frequency bands)

Modulation for **`PresentationTrack`** / **`CurveLayer`** should stay in spec bands so camp stays heavy and readable:

| Band (Hz) | Typical use |
|-----------|-------------|
| **0.03 – 0.12** | Environmental drift |
| **0.15 – 0.35** | Fire / light breathing |
| **0.5 – 1.2** | UI emphasis / reactive pulses |
| **2+** | Avoid as defaults |

Stagger **`frequency_hz`** and **`phase`** across stacked layers. Canonical table: spec **§ Recommended frequency bands**.

---

## Runtime budget (non-negotiable)

Core Delvers presentation identity — do not compromise for “more FX”:

- **No per-frame allocations** on presentation ticks — evaluate tracks into stack locals
- **No effect spam** — few layers, slow modulation, breathing not noise
- **Mutate in place** — no spawn/despawn every frame; sync on layout change + ambient tick
- **Readability-first** — combat/UI clarity beats atmosphere when they conflict
- **Slow layered modulation** — environmental illusion over busy flicker
- **Reuse `PresentationTrack`** — one evaluator, many properties

Details: spec **§ Presentation runtime budget** and **§ Presentation readability**.

---

## Layout file path

**`assets/tuning/title_scene.json`**

- Example: **`assets/tuning/title_scene.example.json`**
- Legacy fallback: **`assets/tuning/title_campfire.json`**
- Constant: **`SCENE_PATH`** in `src/presentation/scene.rs`

---

## Future: iteration safety (not implemented)

Architecture should eventually support:

- **Undo / redo** history for inspector and drag edits
- **Temporary edit snapshots** (session-only buffers)
- **Revert-to-last-save** distinct from reload-from-disk

No implementation in the current wave — awareness only so mouse/inspector iteration does not feel risky long-term.

---

## Future: solo-layer preview (not implemented)

As stacks grow (combat theater, VFX, overlays), tuning one layer in isolation becomes necessary. Planned concept:

- Isolate **glow**, **flame**, **particles**, **ground wash**, **UI overlays**, etc.
- Solo preview in editor without hiding the whole scene permanently

Especially relevant for **combat theater** and **atmospheric overlays**. Spec **§ Solo-layer preview (future)**.

---

## Delivered vs deferred (editor tooling)

| Delivered (initial wave) | Deferred / next wave |
|--------------------------|----------------------|
| Overlay, Settings toggle, hierarchy, inspector | Curve tuning UX in overlay |
| Mouse pick/drag, per-layer hosts | Smoother drag polish |
| Radial glow asset + layered fire | Softer glow + breathing pass |
| `PresentationTrack` on fire alpha | Layered camp props / embers |
| Registry-driven `element:layer` tree | Advanced gizmos (`gizmo.rs`) |
| | Solo-layer preview, undo/redo |

---

## Self-review (this doc)

- **Workflow + philosophy** live here; **types, budget, future APIs** live in the spec.
- **Task checkboxes and file map** live in the plan.
- **Future-only** sections are labeled and omit implementation steps.
