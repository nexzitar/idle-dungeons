# UI art (future)

Placeholder icons are generated in code (`src/ui/placeholder_graphics.rs`) so the game runs without image assets.

To swap in real graphics:

1. Add PNG or SVG-converted PNGs here (e.g. `skill_active.png`, `gold.png`).
2. In `register_ui_placeholder_images`, replace `images.add(gen_…())` with `asset_server.load("ui/skill_active.png")` (after adding `AssetServer` to that system).
3. Keep logical sizes around **32×32** (or 64×64 for `@2x`) for skill/item chips; the UI scales via layout.

Suggested filenames (optional convention):

- `skill_active.png`, `skill_passive.png`, `skill_empty.png`, `skill_locked.png`
- `gold.png`, `salvage.png`
- `item_generic.png`
- `dungeon_theater.png` — wide strip (~256×96 or larger) for combat backdrop; loaded in place of `gen_dungeon_theater()` when you wire `AssetServer`.
