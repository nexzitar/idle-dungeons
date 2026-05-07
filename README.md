# Idle Dungeons

Idle Dungeons is a Bevy-based roguelike incremental game: you configure a hero, run a seeded delve with automated combat, then spend gold and loot on gear, skills, and permanent upgrades. Progress is saved locally (default: `saves/profile.json`).

**Stack:** Rust, **Bevy 0.14**, serde JSON saves. Crate name: `idle_dungeons`.

## Playable loop (today)

1. **Briefing** — Inspect the hero column; **click unlocked skill slots** to cycle through available skills (build + camp). Start a run when ready.
2. **Run** — Watch playback or **skip to results**.
3. **Summary** — **Accept rewards** once; gold and loot merge into your profile.
4. **Camp / upgrades** — **Equip** or **salvage** stash items; buy **caravan upgrades** with gold (including **Gold Gain**, which scales run gold). **Return to briefing** to change loadout again.

**UI:** Mockup-style three-column shell (hero · delve / summary · stash), settings (speed toggle, reset progress), hover **tooltips** on most controls, and reliable primary-click handling on buttons. Footer “RUN / CAMP / …” pills are decorative for now.

## Project layout

| Path | Role |
|------|------|
| `src/app.rs` | `GameState`, events (start run, rewards, equip, salvage, upgrades, skill cycling), profile resource, save hooks |
| `src/domain/` | Hero, skills, combat simulation, dungeon, loot, run summarization |
| `src/save.rs` | Load/save `SaveProfile` |
| `src/ui/` | `UiPlugin`, mockup layout, theme, tooltips, panels |

The **run simulation** (domain + `simulate_run_with_playback`) is the source of truth; the UI sends intents via events and reflects `ProfileState`.

## Design documentation

- Full design: `docs/superpowers/specs/2026-05-06-roguelike-incremental-bevy-design.md`
- MVP checklist: `docs/mvp-acceptance.md`
- Skill catalog ↔ combat mapping: `docs/superpowers/plans/2026-05-07-skill-combat-catalog-mapping.md`
- Gap / increment plan: `docs/superpowers/plans/2026-05-06-mvp-remaining-work.md`

## Development

```sh
cargo fmt --check
cargo test
cargo check
cargo run
```

Prefer small, tested changes in `src/domain/` first, then wire through Bevy events and UI.

## Roadmap

Near-term goals to deepen the MVP:

- **Combat vs skill catalog** — Most skills have definitions and UI copy; extend `simulate_combat` so Guard, Heavy Strike, Poison, Thorns, Barrier, etc. change outcomes in tested ways (not only Lifesteal).
- **Stash honesty** — Either implement minimal filter/sort for inventory/loot or replace placeholder “Filters | Sort” copy with neutral text.
- **Run / world variety** — More room types, affix interplay, and clarity of risk/reward on the briefing screen.
- **Presentation** — Art pass, clearer typography, animation on playback; keep simulation-driven architecture.

Longer-term (post-MVP direction):

- Multiple heroes / party ideas from the design doc
- Deeper procedural dungeon structure (not only linear depth)
- Optional meta layers (prestige, biome unlocks) once the core loop feels rich

Contributions welcome; open an issue or PR with a short note on scope.
