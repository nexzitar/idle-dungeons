# Skill catalog vs `simulate_combat` mapping

Authoritative crosswalk between `SkillTrigger` / descriptions in `src/domain/skills.rs` and behavior in `src/domain/combat.rs` (`simulate_combat`). Updated with Phase A combat work (poison DoT, guard scaling, heavy pacing).

| `SkillId` | Catalog trigger | When simulation applies it | Notes |
|-----------|-----------------|----------------------------|--------|
| `LifestealStrike` | `OnAttack` | Hero attack resolution → heal (`HeroHealed`) | Matches text. |
| `Guard` | `OnHitTaken` | Flat reduction on incoming enemy hit damage | Scales with hero `healing_power` (see combat module). |
| `HeavyStrike` | `OnAttack` | Bonus damage on hero swing; reduced effective attack speed | Tradeoff: slower pacing (`hero_as` factor) per “slow attacks hit harder”. |
| `PoisonEdge` | `OnAttack` | Poison stacks on hero hit; each end-of-tick pulse deals `poison_tick × min(stacks, 12)` then burns **1** stack (`PoisonTick` records potency) | Stacks extend duration and ramp tick damage (cap **12** on multiplier). |
| `ThornSkin` | `OnHitTaken` | Reflect after hero loses HP from an enemy hit | Skipped when barrier absorbs all (`hp_loss == 0`). |
| `BarrierPulse` | `OnRoomStart` | Shield at combat start; absorbs before HP | One pulse per combat. |

**Mismatches resolved (roadmap Phase A):** poison periodic ticks, guard/heavy tuning, barrier/thorns ordering — covered by tests in `src/domain/combat.rs`.

**Future nuance:** descriptions could mention stack caps or exact formulas if players need transparency; optional copy-only follow-ups.

## Buildcraft Phase 1 roster (branch `feat/phase1-skill-book`)

| `SkillId` | Kind | Combat / derived behavior |
|-----------|------|---------------------------|
| `Cleave` | Active | Same bonus damage and attack-speed penalty as `HeavyStrike`. |
| `Taunt` | Active | Stub — no effect until threat/party (roadmap Phase 3). |
| `SecondWind` | Passive | Heals a small % of max HP once at combat start (after barrier init). |
| `ToxicMastery` | Passive | Multiplies poison tick damage (~25%). |
| `VampiricAura` | Passive | Boosts lifesteal heal amount when `LifestealStrike` heals. |
| `ThickHide` / `ArcaneOverflow` / `Berserker` / `SwiftStrikes` / `IronWill` / `BattleFocus` / `CautiousAdvance` | Passive | Flat stat modifiers folded in `HeroProfile::derived_stats`. |
| `LuckyStrike` / `Predator` | Passive | Stubs — await crit / encounter metadata pipelines. |

Single-hero combat remains authoritative; duplicates in two slots are prevented on assign (`assign_skill_to_slot`).
