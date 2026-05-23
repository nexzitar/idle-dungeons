# Delvers

Delvers is a Bevy-based **roguelike incremental**: you configure a hero (and emerging party systems), run a **seeded** delve with **automated combat**, then spend **gold** and **loot** on gear, **skill unlocks**, and permanent upgrades. Progress is saved locally (default: `saves/profile.json`).

**Stack:** Rust, **Bevy 0.18**, serde JSON saves. Crate name: `idle_dungeons`.

## Playable loop (today)

1. **Title / campfire** — **Enter camp** opens the briefing when you are ready.
2. **Briefing (Build)** — Hero identity cards and icon loadout rows. Click skill slots to open **Party Buildcraft** (Apply/Cancel edit session). **Skill guild** (footer) spends gold to unlock catalogue skills. **Gear** hub equips or salvages stash items. Start a run when ready.
3. **Run** — Live delve theater with cast/CD/GCD bars, damage meters, and a combat skill inspect strip. Watch playback or **skip to results**. Combat is simulation-driven and **deterministic** for a given seed and build.
4. **Summary** — Outcome headline, treasure stats, icon loot grid, recessed chronicle. **Accept rewards** once; gold and loot merge into your profile.
5. **Camp** — Adjust loadout, gear, and permanent upgrades; run again.

**UI:** Three-column shell (hero · delve / summary · briefing), **settings** (speed toggle, reset progress), and **fixed inspect strips** on camp surfaces and modals (short one-line hints on controls; no long cursor tooltips). Design law: [`docs/ui-design-system.md`](docs/ui-design-system.md).

## Why it is built this way

The **domain simulation** is the source of truth; the UI sends intents via events and reflects `ProfileState`. That keeps runs **replayable**, **testable**, and safe to extend. For the full design rationale (idle × roguelike, buildcraft, itemization, saves), see:

**→ [docs/design-philosophy.md](docs/design-philosophy.md)**

## Project layout

| Path | Role |
|------|------|
| `src/app.rs` | `GameState` (Title / Build / Running / Summary), events, profile resource, save hooks |
| `src/domain/` | Hero, skills, combat, dungeon, loot, run summarization |
| `src/save.rs` | Load/save `SaveProfile` |
| `src/ui/` | `UiPlugin` — `primitives/`, `shell/`, `screens/`, `interaction/`, `buildcraft/`, modals |
| `src/presentation/` | Title camp atmosphere, scene tuning, presentation editor (debug) |

The run simulation (`simulate_run_with_playback` and friends) drives outcomes; the UI never bypasses it.

## Design documentation

- **Design philosophy (readable overview):** `docs/design-philosophy.md`
- **Visual bible (art direction, Foundation v1):** `docs/visual-bible-foundation-v1.md`
- **UI design system (tokens, inspect policy, primitives):** `docs/ui-design-system.md`
- **UI convergence plan (Phases 0–10, complete):** `docs/superpowers/plans/2026-05-21-ui-design-convergence.md`
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

**Recently shipped**

- **UI design convergence (Phases 0–10):** Shared primitives, inspect ecosystem, gear hub, summary rewards, combat theater scaffold, skill shop icon catalogue — see [CHANGELOG.md](CHANGELOG.md).
- **Party Buildcraft:** Full-screen skill workspace with library grid and edit session.
- **Presentation Wave 7:** Combat readability (float text, multi-foe timing bars, category chips).

**Near-term (post-convergence)**

- **Combat UX (Wave 8):** Playback inspect (started), slot sequencing cues, per-skill cooldown overlays when playback frames carry slot timing.
- **Combat vs skill catalog** — Extend and tune skills; keep `skill_definition` copy aligned with `simulate_combat` behavior (tests as contract).
- **Stash / inventory** — Sort (recent / rarity / name) is in; filters and richer UX later.
- **Run variety** — More room types and affix interplay; summary already surfaces per-room risk and peak risk.

**Longer-term** (see specs and `ACTIVE-REMAINING-WORK.md`)

- Audio layer, pixel-art pass, animation timing
- Full party experience and role clarity
- Deeper procedural dungeon structure (beyond linear depth)
- Optional meta layers (prestige, biome unlocks) once the core loop feels rich

Contributions welcome; open an issue or PR with a short note on scope.

## Changelog

See [CHANGELOG.md](CHANGELOG.md) for version history.
