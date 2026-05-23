# Presentation scene composition (Delvers)

Presentation-only building blocks for title/camp screens and future cinematic layers. Gameplay and simulation stay in `domain/`; this stack is **data-driven**, **hot-reloadable**, and safe to extend with assets and VFX without entangling rules.

## Authoring & editor workflow

Discoverable tooling (settings toggle, fullscreen overlay hierarchy/inspector, mouse pick-drag, save/reload) is documented in [**`presentation-editor-workflow.md`**](presentation-editor-workflow.md) (debug Title state).

**Composable time curves:** scalar **`PresentationTrack`** / **`CurveLayer`** types and evaluation live in [`src/presentation/track.rs`](../src/presentation/track.rs); optional JSON hooks on fire presentation tune are illustrated in **`assets/tuning/title_scene.example.json`** and specified under *PresentationTrack & compositional curves* in [`superpowers/specs/2026-05-20-presentation-editor-design.md`](superpowers/specs/2026-05-20-presentation-editor-design.md).

**Radial campfire glow asset:** **`assets/ui/fire_glow_radial.png`** — soft radial alpha tinted warm in **`spawn_title_fire_layers`** (`src/presentation/fire.rs`, handle loaded as `ui/fire_glow_radial.png`), replacing a flat **`BackgroundColor`** rectangle for atmospheric falloff.

## Layout file

- Primary: `assets/tuning/title_scene.json`
- Example / template: `assets/tuning/title_scene.example.json`
- Legacy fallback: `assets/tuning/title_campfire.json` (migrated into `TitleSceneLayout::fireplace`)

## Core concepts

| Concept | Rust | Role |
|--------|------|------|
| Scene anchors | `SceneAnchorPose`, `resolve_element_translation_px` | Named points in **stage pixel space**; elements add their own `offset_*` on top. |
| Pivot | `ScenePivot`, `pivot_translation_compensation_px` | Keeps **ground contact** and scaling stable (center vs bottom-center vs custom normalized pivot). |
| Elements | `TitleUiElementTune` | Generic transform + exposure/glow + optional `anchor_ref` + `pivot`. |
| Fire layers | `PresentationFirePart`, `spawn_title_fire_layers` | Host node in UI; children = ground wash, clipped stack (base + flame variants), radial **`ImageNode`** using [`fire_glow_radial.png`](../assets/ui/fire_glow_radial.png). |
| Ambient (fake light) | `TitleAmbientPresentationTune` | Scalar hooks for future prop/hero tint near fire (no real light engine). |

## Campfire host vs layers

`TitleCampfireTuneMarker` sits on a **host** `Node` (no `ImageNode`). `spawn_title_fire_layers` runs under `with_children` of that host so z-order, clip rect, and tuning stay coherent. `sync_title_scene_elements` moves the host; `sync_title_fire_presentation_from_layout` resizes the inner stack and updates base tint after JSON reload.

## Runtime systems (Title state)

- `sync_title_scene_elements` — host + figure roots + emoji (uses `ParamSet` for disjoint `Node` access).
- `sync_title_fire_presentation_from_layout` — stack size, glow/ground geometry, base image tint.
- `tick_title_fire_ambient` — slow cross-fade on flame alphas, glow breathe, ground flicker (`ParamSet` for glow vs ground `BackgroundColor`).

## JSON: pivots

`pivot` is a snake_case string: `center`, `top_center`, `bottom_center`, `bottom_left`, `bottom_right`, or `custom` with a nested normalized pivot (see `NormalizedPivot` in `src/presentation/pivot.rs`).

## JSON: anchors

`anchors` is a map of names (e.g. `fire_center`, `lead_slot`, `loot_spawn`, `camera_focus`) to `SceneAnchorPose`. Stage elements reference a name via `anchor_ref`; missing names resolve to `(0, 0)`.

## Fire presentation tuning

`fire_presentation` controls the **illusion** only: variant count, cross-fade period, glow pulse rate/amplitude, and ground ellipse flicker. Disable with `"enabled": false` to fall back to a static stack.

## Debug layout mode

On Title (debug builds): backtick toggles layout mode (same flag as Settings → Debug → **Presentation editor**); Tab cycles targets; hotkeys in `TITLE_SCENE_TUNE_HINT` in `scene_tune.rs`. Ctrl+S / F5 save and reload. Full workflow: [`presentation-editor-workflow.md`](presentation-editor-workflow.md).

## Module paths

- `src/presentation/` — anchors, pivots, fire layer spawn, markers re-exported for UI.
- `src/ui/scene_tune.rs` — serde layout, sync systems, hotkeys, gizmos.
