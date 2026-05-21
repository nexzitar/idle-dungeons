# Presentation editor workflow (Title camp)

In **debug builds**, Delvers exposes an in-engine **presentation editor** on the Title game state: a fullscreen overlay beside the unchanged title stage so you can select elements, tweak `PresentationElementTune` fields, and save JSON without relying on obscure hotkeys alone. Domain and combat code never depend on this tool.

**Related docs:** conceptual stack in [`presentation-scene-composition.md`](presentation-scene-composition.md); architecture in [`superpowers/specs/2026-05-20-presentation-editor-design.md`](superpowers/specs/2026-05-20-presentation-editor-design.md).

---

## Settings toggle (debug)

1. From the title hub, open **Settings**.
2. Under the **Debug** section (only when built with debug assertions), use **Presentation editor · OFF / ON**.
3. Flipping ON sets the same **`PresentationEditorSession::active`** flag as **`**`** (backtick) layout mode: the dimmed fullscreen chrome appears over the campfire.

The row tooltip mirrors the overlay: toggling equals “layout mode” from the legacy backtick workflow (`Open the fullscreen presentation editor overlay (same as layout mode)`).

---

## Overlay: hierarchy, inspector, save / reload, backtick

When the session is active:

| Area | Role |
|------|------|
| **Banner** | `Presentation Mode · {element id}` plus a short hint pointing at backtick / Tab cycling. |
| **Left — Hierarchy** | Buttons **Fireplace**, **Lead slot**, **Ally slot** (ids `fireplace`, `lead_slot`, `ally_slot`). Sets **`selected_element`** for the inspector. |
| **Right — Inspector** | Stepped edits for offsets, scales, size basis, rotation, exposure / glow / bloom, global Z; **Pivot** / **Anchor** summary is read-only. **− / +** mirror the debug hotkey step sizes in **`TITLE_SCENE_TUNE_HINT`** in `src/ui/scene_tune.rs`. |
| **Footer** | **Save to disk** (same as **Ctrl+S**) and **Reload from disk** (same as **F5**). |

**Backtick** toggles layout mode globally. Persistence uses **`TitleSceneLayout::try_save_to_disk`** / **`try_load_from_disk`**, wired to **`assets/tuning/title_scene.json`** (see below).

Implementation reference: `src/presentation/editor/overlay.rs` (spawn), `src/ui/mod.rs` (`handle_presentation_editor_overlay_buttons`).

---

## Mouse pick and drag

With the editor active:

- **Pick:** left **mouse down** on a tunable stage host selects that element (**topmost by `GlobalZIndex`**, ties broken by stable entity ordering). Hosts carry `PresentationElementHost` (`src/ui/components.rs`).
- **Drag:** after pressing on a host, **move** while holding the button to accumulate motion into **`offset_x`** / **`offset_y`**. Hold **Shift** for a **×10** step on drag deltas.

Panel hits use picking / focus so hierarchy and inspector do not accidentally drive stage selection. Logic: `src/presentation/editor/mouse.rs`.

---

## Frequency band guidance (from spec)

Authoring modulation for **`CurveLayer`** / **`PresentationTrack`** (e.g. `fire_presentation` in JSON) should follow the recommended bands so camp atmosphere stays heavy and readable:

| Band (Hz) | Typical use |
|-----------|-------------|
| **0.03 – 0.12** | Environmental drift (ambient bias, distant “air”) |
| **0.15 – 0.35** | Fire / light breathing (glow alpha, ground wash, flame crossfade) |
| **0.5 – 1.2** | UI emphasis / reactive pulses |
| **2+** | Avoid as defaults; reserve for rare FX |

Stack layers should **stagger `frequency_hz` and `phase`** so co-located effects do not **buzz in sync**.

Canonical table: **`### Recommended frequency bands`** in [`docs/superpowers/specs/2026-05-20-presentation-editor-design.md`](superpowers/specs/2026-05-20-presentation-editor-design.md).

---

## Runtime budget summary (presentation layer)

Keep work aligned with **`### Presentation runtime budget`** and **`### Presentation readability`** in that spec:

- Prefer **slow, layered illusion** (tints, alpha stacks, textures) over busy flicker and expensive rendering tricks.
- **Low allocations** during presentation ticks: evaluate tracks into **stack locals**, avoid growing buffers every frame (`src/presentation/track.rs`).
- **No spawn/despawn every frame**: entities spawn with the scene or on reload; sync systems **mutate in place**.
- **Decouple domain combat** from decorative per-frame pulses; playback may someday set presentation *state* tags — not RNG or combat formulas.
- **Reuse `PresentationTrack`** evaluators rather than scattering bespoke sine helpers.

Combat readability overrides atmosphere when they conflict.

---

## Layout file path

The editor reads and writes:

**`assets/tuning/title_scene.json`**

- Example / scaffold: **`assets/tuning/title_scene.example.json`**
- Legacy fireplace-only fallback: **`assets/tuning/title_campfire.json`**

Rust constant: **`SCENE_PATH`** in `src/presentation/scene.rs`.

---

## Phase 5 (not delivered here)

Advanced **canvas gizmos** (handles, pivot/anchor markers, glow-radius preview flags) remain **future** (`presentation/editor/gizmo.rs`). **Phases 0–4** cover typed JSON, overlay + debug settings entry, mouse editing, **`fire_glow_radial.png`** glow stacking, and **track** curves.
