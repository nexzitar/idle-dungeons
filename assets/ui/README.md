# UI art (future)

Placeholder icons are generated in code (`src/ui/assets.rs`) so the game runs without image assets.

To swap in real graphics:

1. Add PNG or SVG-converted PNGs here (e.g. `skill_active.png`, `gold.png`).
2. In `register_ui_placeholder_images`, replace `images.add(gen_…())` with `asset_server.load("ui/skill_active.png")` (after adding `AssetServer` to that system).
3. Keep logical sizes around **32×32** (or 64×64 for `@2x`) for skill/item chips; the UI scales via layout.

**Combat theater:** `dungeon_theater.png` (live delve strip).  
**Title campfire hub:** `campfire_scene.png` (full painting; UI overlays tent label, party slots, and fireplace art on the stone ring).  
**Radial fire glow:** `fire_glow_radial.png` (128×128 RGBA radial gradient, white opaque center smoothing to transparent; tinted warm in `spawn_title_fire_layers` instead of drawing a glowing rectangle).  
**Scene layout (fireplace + party silhouettes):** copy **`assets/tuning/title_scene.example.json`** → **`assets/tuning/title_scene.json`**. Legacy **`title_campfire.json`** is still loaded for `fireplace` only if the scene file is missing. See `src/ui/scene_tune.rs` for fields (`size_basis`, `rotation_deg`, `exposure`, `glow`, `bloom` placeholder, `global_z`). **Debug:** on Title, **`** toggles layout mode; **Tab** picks Fireplace / Lead / Ally; arrows or **IJKL** move the selection; hotkeys run before UI focus so arrows no longer fight directional navigation. **Ctrl+S** / **F5** save or reload JSON.

Suggested filenames (optional convention):

- `skill_active.png`, `skill_passive.png`, `skill_empty.png`, `skill_locked.png`
- `gold.png`, `salvage.png`
- `item_generic.png`
- `dungeon_theater.png`
- `campfire_scene.png`
- `Fireplace.png`
- `fire_glow_radial.png`
