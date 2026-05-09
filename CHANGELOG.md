# Changelog

All notable changes to **Delvers** (crate `idle_dungeons`) are recorded here. **Patch** bumps (0.N.x) are used for small UI and iteration tweaks; **minor** (0.N.0) for larger feature slices; **major** (N.0.0) for big structural releases.

## 0.2.4 — 2026-05-08

### Combat & pacing

- **Skill cadence**: **OnAttack** actives use **cast** and **cooldown** simulation ticks (see `skill_timings`); spammy meter-only swings only apply when you have **no** attack actives equipped.
- **Playback**: **Cast** and **cooldown** micro-bars under each hero/enemy HP bar during combat playback.
- **Elite / boss tuning**: Depth-10 elite slightly trimmed; Gate Warden stats lowered; longer combat tick budget for simulated fights.

### Progression

- **Starter skills**: New saves only **four** skills in the book until you purchase more (migrating saves without the field still unlock the **full** roster).
- **Skill guild** (briefing footer): spend **gold** to permanently add skills to your library; **Assign hero skill** refuses locked IDs.

### Run rewards

- **First blood** loot: the **first** combat win before depth **10** grants **one** rolled item (intended “first-run item” hook).

## 0.2.3 — 2026-05-08

### Engine / build

- **Bevy 0.18**: ECS uses **messages** instead of legacy events (`add_message`, `MessageReader`, etc.); **UI** spawns **`Node`** / **`Text`** components instead of bundles; input such as hero rename listens for **`KeyboardInput`** instead of **`ReceivedCharacter`**. Playback and game logic crates are unchanged.
- **Lean Bevy deps**: `default-features = false` with `features = ["2d"]` drops the bundled **3D** stack (`bevy_pbr`, `bevy_gltf`, etc.) — faster builds and smaller binaries for this UI-first game.

### Maintenance

- Cleaned **`unused_parens`** warnings from UI spawn sites after the automated bundle-to-component refactors.

## 0.2.2 — 2026-05-06

### Brand / flow

- **Delvers**: Window title and in-game header copy use the new name.
- **`GameState::Title`**: The game now boots to a **title / campfire hub** screen (`title_camp.rs`): animated fire, progression-based silhouettes at the fire (second figure when `party_slots_unlocked() >= 2`, i.e. depth 75+), optional **tent** after depth 15. **Enter camp** moves to the existing briefing (`Build`); **Quit** exits the app. Settings and reset behave as before; reset returns to the title screen.

## 0.2.1 — 2026-05-06

### Combat theatre (run playback)

- **Backdrop**: Procedural **pixel stone wall** texture (`dungeon_theater` in `UiPlaceholderImages`) tiled behind the live combat strip; swap with `assets/ui/dungeon_theater.png` later if you want hand-drawn art.
- **Threat focus**: Red **horizontal bar** overlay points at the focused party row (`sync_playback_aggro_arrow_line`), driven by the same threat / target rules as the existing “→ You / Ally” caption on the enemy plate.
- **Damage meters**: **DAMAGE (THIS ENCOUNTER)** block with bars for **Hero** (lead attacks + poison/thorns to the enemy), **Ally** (partner attacks only; row hidden when solo), and **Foe** (total damage dealt by the enemy to the party). Bar width is relative to the max of the three totals on the current frame.

### Data / telemetry

- `CombatPlaybackFrame` now carries **`damage_meter_party_0`**, **`damage_meter_party_1`**, and **`damage_meter_foe`**, accumulated while building playback from combat events.

## 0.2.0 — 2026-05-06

### UI / UX

- **Gear hub**: Equip and salvage no longer imply closing the modal; the hub stays open until **Close** or the backdrop dismiss control. Opening sets an internal “keep open” flag; full UI rebuilds (e.g. after stash sort or profile refresh) re-spawn the hub when that flag is set.
- **Run summary**: Loot and **Accept rewards** live in a centered **Run rewards** modal instead of competing with the footer **Start run** area on other flows.
- **Layout**: Removed the permanent right-hand **stash / management** column from briefing, run playback, and summary; inventory and paper doll are handled in the **Gear** hub modal.
- **Footer**: Gold meta **Upgrades** screen and related navigation removed entirely.

### Gameplay / meta

- **Gold upgrades**: Removed the infinite gold-purchased stat upgrade system. Progression stats come from **gear** (and skills), not purchased meta lines. `MetaProgression` retains gold, salvage, skill slot unlocks, and depth-related progress only.

### Content / naming

- **Legendary (and multi-affix) item names**: Two-affix items use a **prefix + gear kind + “of” suffix** pattern from both affixes (e.g. Virulent + Spiked trinket → `Legendary Virulent Charm of Thorns`), instead of only reflecting the first affix.

### Maintenance

- Deleted unused `src/ui/upgrade_panel.rs`.
- Fixed summary rewards modal dimmer: `FocusPolicy` set on `NodeBundle` only (avoids duplicate-component panic on Bevy 0.14).

---

*When you ship new work, add a dated subsection under a new version in `Cargo.toml`, keeping one bullet per user-facing or save-format change.*
