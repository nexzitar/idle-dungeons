# Changelog

All notable changes to **Delvers** (crate `idle_dungeons`) are recorded here. **Patch** bumps (0.N.x) are used for small UI and iteration tweaks; **minor** (0.N.0) for larger feature slices; **major** (N.0.0) for big structural releases.

## Unreleased

### UI design convergence (in progress)

- **Phase 0–1:** Design system doc, `UiDensity` / `MountedPanelStyle` / `SkillDisplayFamily`, `spawn_mounted_panel`, `spawn_framed_section_header`.
- **Phase 2:** Shared `loadout_row` + `skill_bar` on build, summary, and running camp columns; text chip slots removed (~170 lines from `shell/layout.rs`).
- **Phase 3:** `hero_identity_card` primitive — portrait frame, name/rename, role subtitle, HP/DMG/ARM stat strip; camp party column uses it for P1/P2 instead of separate title + vitals rows.
- **Phase 4:** Build screen convergence — `shell/hero_column.rs` (deep mounted panel, framed headers, gold layering captions); briefing column recessed panel; footer dock uses `spawn_button` with one-line hints; `build_panel_text` wall retired.
- **Phase 5:** Inspect ecosystem — `ui/inspect.rs` (`InspectRegion`, scoped strips, unified hover/sync); compact inspect on camp, gear hub, and skill shop; floating tooltips removed from camp surfaces (inspect strip replaces them).

### Party buildcraft sheet (Phase 1)

- **Party-first skill workspace:** Replaces the list-style skill book with a full **Party Buildcraft** sheet — party loadout column (P1/P2, six slots each), icon library grid, and fixed inspect panel.
- **Edit session:** Pending loadout changes live in `BuildcraftEditSession`; **Apply** commits all heroes and saves, **Cancel** discards. Same skill forbidden within one hero, allowed across heroes.
- **Primitives:** `skill_icon`, `skill_bar`, and `inspect_panel` under `ui/primitives/`; cooldown overlay hooks reserved for future playback VFX.
- **Spec:** `docs/superpowers/specs/2026-05-20-skillbook-buildcraft-ux-design.md` (rev. 2).

### UI foundation extraction (Phases 1–6, complete)

- **Phase 1:** Extracted `ui/primitives` (button, scroll, bar, panel, modal, text) — behavior-neutral moves from `widgets` / shell layout / `gear_hub`.
- **Phase 2:** Renamed `placeholder_graphics` → `assets`; split legacy shell layout into `ui/shell/` (`layout`, `theater`, `playback_bars`, `footer`); retired `widgets` (`spawn_atmosphere` lives in `primitives/panel`).
- **Phase 3:** `ui/interaction` centralizes click capture and `UiClickAction` dispatch (modals, stash sort, gear/skill flows, playback chrome, presentation editor in debug). Legacy per-button `handle_*` click systems removed from `ui/mod.rs`.
- **Phase 4:** `UiPanelStyle` / `UiModalStyle` layout presets in `theme`; `spawn_item_card` / `spawn_item_card_preview` moved to `primitives/card` (gear hub + summary footer call sites updated).
- **Phase 5:** Build / running / summary screen spawn moved to `ui/screens/`; `UiPlugin` registration stays in `mod.rs`.
- **Phase 6:** Presentation editor overlay uses `spawn_button` / `spawn_framed_column` primitives; `+` tune step buttons now dispatch `EditorTuneDelta`.
- **Cleanup:** Playback sync and lifecycle systems moved to `ui/playback_sync.rs` and `ui/systems.rs`; `ui/mod.rs` is plugin registration only (~260 lines). No `mockup_layout` / `placeholder_graphics` references remain under `src/`.

### Presentation Wave 2 — atmosphere-first (title camp)

- **Softer campfire glow:** dual radial layers (wide halo + core), stack `overflow: visible`, no always-on layer borders (editor outlines only when layout mode is on).
- **Breathing / flicker:** default compositional `glow_alpha` / `ground_alpha` tracks; flame vertical breathe; three ember sparks; gentler crossfade.
- **Editor UX:** smoother mouse drag (1 px threshold, committed nudge), hover on sub-layers, **Fire atmosphere** inspector block (glow/ground track bases, breath Hz, crossfade, α floor) when fireplace is selected.

### Presentation editor (phases 0–4)

- **Phase 0 — Types & adapter:** **`PresentationElementTune`**, **`TitleCampSceneLayout`**, and load/save for **`assets/tuning/title_scene.json`** centralized under **`src/presentation/`** with thin aliases in **`src/ui/scene_tune.rs`** (`PresentationScene`-style layering without renaming on-disk JSON in one shot).
- **Phase 1 — Overlay & discoverability:** **`PresentationEditorSession`**, fullscreen Bevy UI overlay (hierarchy · inspector · save/reload), and **Settings → Debug → Presentation editor** toggle ( **`#[cfg(debug_assertions)]`**, same **`active`** flag as backtick layout mode).
- **Phase 2 — Mouse editing:** **`presentation_editor_pick`** / **`presentation_editor_drag`** on **`PresentationElementHost`** (top-`GlobalZIndex` selection, **`Shift`** ×10 drag), plus light hover outline synced with keyboard selection gizmo.
- **Phase 3 — Radial glow:** Campfire **`ImageNode`** soft bloom using **`assets/ui/fire_glow_radial.png`** instead of a flat glow rectangle (**`spawn_title_fire_layers`** / **`TitleFirePresentationTune`**).
- **Phase 4 — Tracks:** **`PresentationTrack`**, **`CurveLayer`**, and **`CurveKind`** with deterministic **`(t_secs, seed)`** evaluation; optional compositional **`glow_alpha`** / **`ground_alpha`** JSON on **`fire_presentation`** (Examples in **`title_scene.example.json`**).
- **Docs:** **`docs/presentation-editor-workflow.md`**, **`docs/presentation-scene-composition.md`** cross-links plus spec links for frequency bands / runtime budget. **Phase 5 gizmos** remain future work (**`presentation/editor/gizmo.rs`** stub only).

## 0.2.31 — 2026-05-09

### Presentation (Wave 7)

- **Floating combat text:** Stronger caption hierarchy and popup de-spam (`FloatingCombatPopup` sequencing, TTL, cap) in [`mod.rs`](src/ui/mod.rs); [`playback_float_text_color`](src/ui/theme.rs) maps anchors (e.g. cleave/ability gold, poison venom on foe).
- **Pack timing UI:** [`CombatEvent::TimingPulse`](src/domain/combat.rs) and [`CombatPlaybackFrame`](src/domain/combat.rs) carry **off-target** foe cast/CD fills when multiple foes live; theater adds a **Flank** row and alt bars in [`mockup_layout`](src/ui/mockup_layout.rs) / [`components`](src/ui/components.rs), synced in [`sync_playback_cast_bars_foe`](src/ui/mod.rs).
- **Skill category chips:** [`SkillCategory::category_abbr`](src/domain/skills.rs) plus [`skill_category_chip_colors`](src/ui/theme.rs); loadout and skill book show category affordance in [`build_panel`](src/ui/build_panel.rs) and [`skill_book`](src/ui/skill_book.rs).

### Docs

- **Wave 7** checked in [`ACTIVE-REMAINING-WORK.md`](docs/superpowers/ACTIVE-REMAINING-WORK.md).

## 0.2.30 — 2026-05-09

### Telemetry (Wave 6)

- **Strike damage totals:** [`RunSummary`](src/domain/run.rs) carries cumulative **white** (weapon/basic) vs **yellow** (ability) damage from party [`HeroAttacked`](src/domain/combat.rs) events (primary + cleave); [`party_strike_damage_white_yellow`](src/domain/combat.rs) aggregates the event stream. [`RunSummary::strike_ability_share_percent`](src/domain/run.rs) exposes **0–100** ability share for display.
- **UI:** Run rewards digest and rewards modal show the strike mix when there was strike damage ([`summary_panel`](src/ui/summary_panel.rs), [`mockup_layout`](src/ui/mockup_layout.rs)).

### Docs

- **Wave 6** checked in [`ACTIVE-REMAINING-WORK.md`](docs/superpowers/ACTIVE-REMAINING-WORK.md); cross-check table links strike telemetry.

## 0.2.29 — 2026-05-09

### Combat / roles / itemization (Wave 5)

- **Party threat routing:** Per-tick decay on both threat slots; on a fixed interval, when the ally has tank stance (`Guard` + `ThickHide`), threat can **transfer** from lead to partner with an extra burst — wired in [`simulate_combat_party`](src/domain/combat.rs) / [`simulate_combat_party_foes`](src/domain/combat.rs) via [`tick_party_threat_routing`](src/domain/party.rs); pulse may emit [`CombatEvent::ThreatSnapshot`](src/domain/combat.rs).
- **Archetype hints:** [`combat_archetype`](src/domain/combat_archetype.rs) derives [`CombatArchetypeHint`](src/domain/combat_archetype.rs) from skills/affixes on the **lead**; [`loot`](src/domain/loot.rs) nudges affix roll weights; [`roll_loot`](src/domain/loot.rs) / profile-guided early drops and treasure in [`run`](src/domain/run.rs) use those hints.
- **Elite twin encounters:** Elite rooms can roll a **twin** pack ([`dungeon`](src/domain/dungeon.rs)) using twin-named bodies.

### Docs

- **Wave 5** checked in [`ACTIVE-REMAINING-WORK.md`](docs/superpowers/ACTIVE-REMAINING-WORK.md); cross-check table links archetype + threat code paths.

## 0.2.28 — 2026-05-09

### Itemization / progression (Wave 4)

- **Nine `GearSlot`s:** main hand, off-hand, head, chest, hands, feet, trinket I & II, relic — distributes power budget across more items; [`loot`](src/domain/loot.rs) uses per-slot stats + compressed depth budget (~55% linear) so drops are smaller but sets stay relevant.
- **Save compatibility:** legacy JSON `"Weapon"` / `"Armor"` / `"Trinket"` keys (and slot fields) deserialize via `serde(alias)` to main hand, chest, trinket I.
- **Rhythm affix:** Heavy/Cleave weave post-swing recovery shortened by 1 tick when Rhythm is equipped on any item.
- **Encounter score:** [`RunSummary.encounter_score`](src/domain/run.rs) from combat clears (depth × role weight) + sim tick slice + treasure/shrine bonuses; shown in run summary rewards line.
- **Two-handed weapons:** `ItemInstance.two_handed` main-hand pieces clear or block off-hand; equipping off-hand unequips a two-hander; stash equip returns all displaced items.
- **Gear hub:** loadout column scrolls when content overflows ([`spawn_scrollable_flex_column`](src/ui/widgets.rs)).

### Docs

- **Spec:** [`docs/superpowers/specs/2026-05-09-wave4-itemization-design.md`](docs/superpowers/specs/2026-05-09-wave4-itemization-design.md); **Wave 4** checked in [`ACTIVE-REMAINING-WORK.md`](docs/superpowers/ACTIVE-REMAINING-WORK.md).

## 0.2.27 — 2026-05-09

### Combat / hardening

- **Fixed-point weapon meters:** Party and foe swing buildup uses [`combat_meter`](src/domain/combat_meter.rs) (`u64` sub-units) instead of float meters in [`simulate_combat_party`](src/domain/combat.rs); unit tests cover consume semantics and long-run add/consume balance.
- **Poison scheduling:** Documented batched end-of-tick [`PoisonTick`](src/domain/combat.rs) (after proactive strikes and [`TimingPulse`](src/domain/combat.rs)); regression test locks event ordering vs [`ActionLane::Dot`](src/domain/combat_timing.rs).
- **Scheduler tests:** Extreme `attack_speed` determinism (`scheduler_extreme_attack_speed_is_deterministic`); solo mutual OHKO outcome follows initiative (`mutual_ohko_outcome_follows_initiative_order_solo`).

### Docs

- **Waves:** [`ACTIVE-REMAINING-WORK.md`](docs/superpowers/ACTIVE-REMAINING-WORK.md) Wave 3 marked complete.

## 0.2.26 — 2026-05-12

### Domain / UX

- **Skill layering:** [`skill_layering`](src/domain/skill_layering.rs) module with [`layering_warnings`](src/domain/skill_layering.rs) (Heavy + Cleave today); build panel appends notices under core stats.
- **Docs:** [`ACTIVE-REMAINING-WORK.md`](docs/superpowers/ACTIVE-REMAINING-WORK.md) waves updated for §D encounter score / equipment slots and §E archetype framework; Waves 1–2 marked done for taxonomy + layering.

## 0.2.25 — 2026-05-09

### Docs / combat readability

- **Backlog:** [`ACTIVE-REMAINING-WORK.md`](docs/superpowers/ACTIVE-REMAINING-WORK.md) adds execution waves + code cross-links atop the A–H design sections.
- **Skills:** [`SkillCategory`](src/domain/skills.rs) (BasicAttack … Proc taxonomy) and [`skill_category`](src/domain/skills.rs) for every [`SkillId`](src/domain/skills.rs); skill book rows show category beside active/passive.
- **Playback float text:** [`playback_float_text_color`](src/ui/theme.rs) takes [`CombatSfxAnchor`](src/domain/combat.rs) so party outbound strikes read as cream/gold instead of foe-red.
- **Timing docs:** Module rustdoc on [`combat_timing`](src/domain/combat_timing.rs) summarizes tick length, GCD vs weapon cadence, and initiative.

## 0.2.24 — 2026-05-09

### Combat / playback

- **Opening frame buffs:** Pre-clock bookkeeping (`HeroHealed` / encounter [`BuffApplied`](src/domain/combat.rs) / [`BuffExpired`](src/domain/combat.rs)) is merged into the first [`CombatPlaybackFrame`](src/domain/combat.rs) and skipped in the stepped replay so encounter-seeded chips (e.g. Inner Strength) appear immediately without double-applying.
- **Ability pacing bars:** Each [`CombatEvent::TimingPulse`](src/domain/combat.rs) and playback frame now carries **shared ability GCD** fill (Empowered Blow queue + Victory Rush lockout), **instant-strike recharge** fill, and per-slot charge counts. The run playback theater adds two thin tracks under lead and ally cast/CD stacks; foe bars use a separate sync system to satisfy Bevy `Query` disjointness.

## 0.2.23 — 2026-05-09

### UI / playback

- **Party status strip:** [`combat_playback_frames_from_result`](src/domain/combat.rs) now fills [`CombatPlaybackFrame::hero_debuff_slots`](src/domain/combat.rs) from **lead** and **ally** [`BuffApplied`](src/domain/combat.rs) / [`BuffExpired`](src/domain/combat.rs) (plus existing enemy poison on the foe row).

## 0.2.22 — 2026-05-09

### Combat

- **Instant strike charges:** [`ability_icd_ticks`](src/domain/skills.rs) is now a **per-charge recharge interval** (one charge gained per pulse while below [`max_charges`](src/domain/skills.rs)), not a full-pool lockout. Each party hero has their own GCD and charge timers; spending a charge starts recharge **only if** none is already running.
- **Party parity:** Slot **1** uses the same Victory Rush / Empowered Blow pacing as the lead (own [`BuffApplied`](src/domain/combat.rs) / [`BuffExpired`](src/domain/combat.rs) targets, [`BuffChargeConsumed`](src/domain/combat.rs) on slot 1, partner [`maybe_devourer_heal_on_kill`](src/domain/combat.rs) on kills).
- **[`CombatSimOptions`](src/domain/combat.rs):** optional `partner_instant_strike_max_charges` override.

## 0.2.21 — 2026-05-09

### Combat / skills

- Laid groundwork for **`SkillDefinition::max_charges`** and **[`CombatSimOptions`](src/domain/combat.rs)**. **Recharge behavior was reworked in 0.2.22** (per-charge pulse instead of empty-pool full refill).

## 0.2.20 — 2026-05-09

### Combat

- **Phase 3 — buff runtime:** same [`BuffId`](src/domain/buff.rs) on the same party slot **merges stacks**, **extends expiry** to the later of old/new end times, and **refreshes charges** when the incoming [`BuffApplication`](src/domain/buff.rs) sets them. Poison [`PoisonTick`](src/domain/combat.rs) appends a cosmetic **[`BuffTick`](src/domain/combat.rs)** with [`BuffId::PoisonVenom`](src/domain/buff.rs); **Victory Rush** strikes emit **[`BuffChargeConsumed`](src/domain/combat.rs)** ([`BuffId::VictoryRush`](src/domain/buff.rs)) for playback. Floating text tints poison tick captions red ([`playback_float_text_color`](src/ui/theme.rs)).

### Skills (Phase 4 policy)

- **Shared ability GCD vs weapon cadence:** [`skill_triggers_shared_ability_gcd`](src/domain/skills.rs) is **`true`** for [`InstantStrike`](src/domain/skills.rs) and **[`NextMeleeBuff`](src/domain/skills.rs)**; **`false`** for **[`SwingWeave`](src/domain/skills.rs)** and passives. [`simulate_combat_party`](src/domain/combat.rs) doc comment cross-links this helper (weapon wind-up stays authoritative for Heavy / Cleave-style actives).

## 0.2.19 — 2026-05-11

### Combat

- **Empowered Blow → buff runtime:** queueing the next-swing buff emits **`BuffApplied`** (`BuffId::EmpoweredBlow`); landing the charged swing emits **`BuffExpired`**.

## 0.2.18 — 2026-05-11

### Combat (Phase 3 — buff runtime)

- New [`domain::buff`](src/domain/buff.rs): [`BuffId`](src/domain/buff.rs), [`BuffApplication`](src/domain/buff.rs), tick-based expiry helper.
- [`CombatEvent`](src/domain/combat.rs): **`BuffApplied`**, **`BuffExpired`**, **`BuffTick`**, **`BuffChargeConsumed`** (latter two ready for HoT/charge hooks).
- Party buff state in [`combat.rs`](src/domain/combat.rs): [`simulate_combat_party_with_initial_buffs`](src/domain/combat.rs) seeds buffs at encounter clock **0**; each tick end removes expired buffs. [`simulate_combat_party`](src/domain/combat.rs) unchanged for callers (`[]`).
- Playback captions and floating anchors for buff lines.

## 0.2.17 — 2026-05-09

### Combat

- **White vs ability damage:** [`HeroAttacked`](src/domain/combat.rs) carries a [`HeroStrikeDamage`](src/domain/combat.rs) split (`white` / `yellow` + optional source skill). Heavy / Cleave bonuses count as **yellow**; total damage matches the previous combined hit for unchanged loadouts.
- **Skill combat styles:** [`SkillCombatStyle`](src/domain/skills.rs) and per-skill [`gcd_ticks`](src/domain/skills.rs) / [`ability_icd_ticks`](src/domain/skills.rs) on [`SkillDefinition`](src/domain/skills.rs). Only **SwingWeave** actives merge into the weapon cadence (`attack_cadence_ticks`); **NextMeleeBuff** and **InstantStrike** do not stretch the heavy wind-up bar.
- **Empowered Blow** (Heroic Strike–style): queues bonus yellow on the **next** white swing; queuing triggers the ability GCD and a short internal cooldown after consume.
- **Victory Rush** (instant strike): yellow-only hits on their own initiative passes with shared ability GCD and a **longer ICD** so it cannot be used every GCD.
- **UI:** combat log captions describe white / ability mix; floating text uses **gold** for lines that mention **ability damage** ([`playback_float_text_color`](src/ui/theme.rs)).

## 0.2.16 — 2026-05-09

### Combat

- **Initiative salt from delves:** `simulate_combat_party` takes `initiative_run_salt` (tests / solo helpers pass **`0`**). [`RunConfig`](src/domain/run.rs) mixes **run seed + room depth** into this salt so encounter initiative can **vary by floor and run** while staying deterministic.

## 0.2.15 — 2026-05-09

### Combat

- **Phase 1 initiative scheduler:** per combat tick, lead / partner / foe take **passes** in **stable initiative order** (from `combat_timing::initiative_ranks`), repeating until idle. Attack-speed meters add **once per tick** per actor; multiple swings in the same 100 ms tick **interleave** with other actors instead of a strict hero-then-foe phase lock. See `combat_round`, `lead_weapon_pass` / `partner_weapon_pass` / `foe_weapon_pass` in `src/domain/combat.rs`.

## 0.2.14 — 2026-05-09

### Balance

- **Loot rarity** is **probability-based** from depth (same seed still yields the same item): **Legendary** starts near **~1%** and rises slowly toward floor **999**; **Legendary is guaranteed only at depth ≥ 1000**. See `roll_rarity_for_depth` in `src/domain/loot.rs`.

### Documentation

- **README** refreshed: correct **Bevy 0.18** stack, skill book + skill guild in the loop, link to changelog, pointer to design philosophy.
- New **`docs/design-philosophy.md`**: how Delvers combines idle/incremental progression with seeded roguelike runs, simulation-first architecture, buildcraft, itemization, saves, and UX direction.

## 0.2.13 — 2026-05-09

### Balance

- **Loot rarity curve** (depth → tier) is stretched: floor **10** rewards are **Uncommon** at most; **Legendary** starts at depth **50** (was **24**). See `rarity_for_depth` in `src/domain/loot.rs`.

### UX

- **UI scroll**: Slower wheel speed (`apply_ui_scroll` uses **12** px per unit instead of **28**).

## 0.2.12 — 2026-05-09

### Fixes

- **UI clicks**: Serialized **capture → modal/button handlers → `clear_ui_click_after_release`** in one chain so clearing the click target cannot race handlers (skill shop buys and other taps were flaky).
- **`Interaction::None` on mouse-up**: Treated like a valid confirmation when resolving release, covering Bevy's brief transient state on fast clicks.
- **Skill guild**: When you lack gold, rows are **non-buttons** so we don't enqueue impossible purchases (tooltip explains how much gold is missing).
- **Scroll**: Mouse wheel deltas are **inverted** in `apply_ui_scroll` so lists move in the intuitive direction.

## 0.2.11 — 2026-05-09

### UX

- **Run rewards**: Loot list shows **read-only item cards** (same look as stash minus actions); loot scroll gets a **minimum height** so it no longer collapses invisible; modal has **extra bottom padding** and the **Accept rewards** button has clearer spacing.
- **Gear hub**: The equipped + stash pair is **horizontally centered** again with symmetric side margins.

## 0.2.10 — 2026-05-09

### UX

- **Gear hub**: Panels sit farther **left**, use **matching top/bottom inset** while stretching vertically, the **stash** column is wider, and stash items arrange in **two columns**.

## 0.2.9 — 2026-05-08

### UX

- **Gear hub**: **Equipped** loadout and **Stash** are shown as **two side-by-side panels** (centered row with gutter space) instead of one tall stack; stash gets a taller scroll viewport.

## 0.2.8 — 2026-05-08

### UX

- **Post-run loot**: The summary screen and rewards modal call out new gear explicitly (counts, item names, and a reminder to open **Gear** after **Accept rewards**).
- **Gear hub**: Dialog height is capped with a shorter scroll viewport so the stash list fits typical window sizes without hiding the footer dock.
- **Playback speed**: Moved out of **Settings** — the header now has **‹** / **›** buttons stepping **1×**, **2×**, **3×**, **5×**, and **10×**; settings only covers reset / close.

## 0.2.7 — 2026-05-09

### Early game / loot

- Guided **skirmish salvage** (first combat win before depth **10**): on your profile, the **first** claim rolls a **weapon**, the **second** an **armor**; later claims use normal loot with **salted RNG** that includes your lifetime claim count — the MVP **fixed delve seed** no longer hands you the **same starter item** on every run while gold still accrues.

## 0.2.6 — 2026-05-08

### UI / input

- **Modal-scoped click capture**: When settings, skill book, skill shop, or gear hub is open, mouse-down **button** resolution ignores **briefing** widgets so **Reset all progress**, **skill picks**, and clears line up with what you actually clicked.
- **Skill book**: Choices apply from the **hovered** pick row on mouse-up (still ties to the captured press target when Bevy marks several rows briefly).
- **Title screen**: Spawns the same **tooltip** layer used elsewhere (`raise_tooltip_above_modals`), so hints are consistent after navigation.

## 0.2.5 — 2026-05-08

### Progression / saves

- **Skill library defaults**: Saves that omit **`unlocked_skill_ids`** deserialize to the **starter four** (same as a fresh profile). An **explicit empty list** on disk is normalized to starters on load.
- **Playback UI**: Tooltip layer is **re-parented each frame** so hover text stays above modals (e.g. skill book).

### Combat playback

- **Damage meters** show **run-wide** totals (carry across encounters in the same delve) plus an approximate **damage per second** (uses `COMBAT_TICK_DISPLAY_SECS` as the display tick length).
- **Depth-10 elite** is stronger again (more HP, damage, and armor) so early runs without a build do not nearly clear the milestone.

## 0.2.4 — 2026-05-08

### Combat & pacing

- **Skill cadence**: **OnAttack** actives use **cast** and **cooldown** simulation ticks (see `skill_timings`); spammy meter-only swings only apply when you have **no** attack actives equipped.
- **Playback**: **Cast** and **cooldown** micro-bars under each hero/enemy HP bar during combat playback.
- **Elite / boss tuning**: Gate Warden stats lowered; longer combat tick budget for simulated fights.

### Progression

- **Starter skills**: New saves only **four** skills in the book until you purchase more from the skill guild.
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
- **Damage meters**: **DAMAGE (RUN TOTAL)** block with bars for **Hero** (lead attacks + poison/thorns to the enemy), **Ally** (partner attacks only; row hidden when solo), and **Foe** (total damage dealt by the enemy to the party). Bar width is relative to the max of the three **run cumulative** totals on the current frame.

### Data / telemetry

- `CombatPlaybackFrame` carries **run-cumulative** damage meter fields (`damage_meter_party_0`, `damage_meter_party_1`, `damage_meter_foe`) and **`run_sim_ticks`** for DPS labeling during playback.

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
