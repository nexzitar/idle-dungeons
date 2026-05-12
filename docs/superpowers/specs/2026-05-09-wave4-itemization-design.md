# Wave 4 — Itemization & progression (2026-05-09)

## Problem

Three gear slots forced **huge per-item stat budgets**, which pushed balance toward **stat sticks** and diluted **affix identity**.

## Direction

1. **Nine equipment slots** (WoW-style spread): main hand, off-hand, head, chest, hands, feet, two trinkets, relic. Same JSON profile uses `serde(alias)` so legacy `"Weapon"` / `"Armor"` / `"Trinket"` keys and slot fields map to main hand, chest, and trinket I.

2. **Stat compression**: loot uses `compressed_stat_budget` (~55% of linear depth + scaled rarity) multiplied by **per-slot stat formulas** so individual drops are smaller; a full set approaches prior total pressure.

3. **Build-defining affix prototype — Rhythm**: with Heavy/Cleave weave, **post-swing recovery** (`attack_cd_total`) is reduced by **1 tick** (min 1). Wired in [`prepare_hero_combat`](../../src/domain/combat.rs).

4. **Encounter score** (experiment): [`RunSummary::encounter_score`](../../src/domain/run.rs) accumulates from combat wins (depth × role multiplier × 15 + `clock_ticks`/8), plus smaller bonuses from treasure/shrine rooms. Shown on the run summary panel; not yet a spendable currency.

## Follow-ups (Wave 5+)

- Off-hand identity (shield vs focus) as distinct item templates.
- Spend encounter score or tie it to meta upgrades.
- Further enemy curve tuning once gearing data arrives from playtests.
