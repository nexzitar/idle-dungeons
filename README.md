# Delvers

Delvers is a Bevy-based **roguelike incremental**: you configure a hero (and emerging party systems), run a **seeded** delve with **automated combat**, then spend **gold** and **loot** on gear, **skill unlocks**, and permanent upgrades. Progress is saved locally (default: `saves/profile.json`).

**Stack:** Rust, **Bevy 0.18**, serde JSON saves. Crate name: `idle_dungeons`.

## Playable loop (today)

1. **Title / campfire** — **Enter camp** opens the briefing when you are ready.
2. **Briefing** — Inspect the hero (and party when unlocked). **Skill book:** click unlocked slots to assign or clear skills. **Skill guild:** spend gold to unlock new skills for the book. Start a run when ready.
3. **Run** — Watch playback or **skip to results**. Combat is simulation-driven and **deterministic** for a given seed and build.
4. **Summary** — **Accept rewards** once; gold and loot merge into your profile.
5. **Camp** — **Gear** hub to equip or salvage stash items; permanent upgrades; adjust loadout and run again.

**UI:** Three-column shell (hero · delve / summary · stash), **settings** (speed toggle, reset progress), **tooltips** on most controls. Footer pills (RUN / CAMP / …) are placeholder chrome.

## Why it is built this way

The **domain simulation** is the source of truth; the UI sends intents via events and reflects `ProfileState`. That keeps runs **replayable**, **testable**, and safe to extend. For the full design rationale (idle × roguelike, buildcraft, itemization, saves), see:

**→ [docs/design-philosophy.md](docs/design-philosophy.md)**

## Project layout

| Path | Role |
|------|------|
| `src/app.rs` | `GameState` (Title / Build / Running / Summary), events, profile resource, save hooks |
| `src/domain/` | Hero, skills, combat, dungeon, loot, run summarization |
| `src/save.rs` | Load/save `SaveProfile` |
| `src/ui/` | `UiPlugin`, layout, theme, tooltips, panels (gear, skill book, skill shop, etc.) |

The run simulation (`simulate_run_with_playback` and friends) drives outcomes; the UI never bypasses it.

## Design documentation

- **Design philosophy (readable overview):** `docs/design-philosophy.md`
- **Visual bible (art direction, Foundation v1):** `docs/visual-bible-foundation-v1.md`
- **Remaining work (active backlog):** `docs/superpowers/ACTIVE-REMAINING-WORK.md`
- Full design: `docs/superpowers/specs/2026-05-06-roguelike-incremental-bevy-design.md`
- **Strategic roadmap (archived snapshot):** `docs/superpowers/specs/obsolete-2026-05-06-buildcraft-party-systems-roadmap.md`
- MVP checklist: `docs/mvp-acceptance.md`
- Skill catalog ↔ combat mapping: `docs/superpowers/plans/2026-05-07-skill-combat-catalog-mapping.md`

## Development

```sh
cargo fmt --check
cargo test
cargo check
cargo run
```

Prefer small, tested changes in `src/domain/` first, then wire through Bevy events and UI.

## Roadmap

Near-term:

- **Combat vs skill catalog** — Extend and tune skills; keep `skill_definition` copy aligned with `simulate_combat` behavior (tests as contract).
- **Stash / inventory** — Sort (recent / rarity / name) is in; filters and richer UX later.
- **Run variety** — More room types and affix interplay; summary already surfaces per-room risk and peak risk.
- **Presentation** — Animation on playback, art pass; keep simulation-driven architecture.

Longer-term (see specs):

- Full party experience and role clarity
- Deeper procedural dungeon structure (beyond linear depth)
- Optional meta layers (prestige, biome unlocks) once the core loop feels rich

Contributions welcome; open an issue or PR with a short note on scope.

## Changelog

See [CHANGELOG.md](CHANGELOG.md) for version history.
