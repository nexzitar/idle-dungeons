# Skill catalog vs `simulate_combat` mapping

Authoritative crosswalk between `SkillTrigger` / descriptions in `src/domain/skills.rs` and behavior in `src/domain/combat.rs` (`simulate_combat`). Updated with Phase A combat work (poison DoT, guard scaling, heavy pacing).

| `SkillId` | Catalog trigger | When simulation applies it | Notes |
|-----------|-----------------|----------------------------|--------|
| `LifestealStrike` | `OnAttack` | Hero attack resolution → heal (`HeroHealed`) | Matches text. |
| `Guard` | `OnHitTaken` | Flat reduction on incoming enemy hit damage | Scales with hero `healing_power` (see combat module). |
| `HeavyStrike` | `OnAttack` | Bonus damage on hero swing; reduced effective attack speed | Tradeoff: slower pacing (`hero_as` factor) per “slow attacks hit harder”. |
| `PoisonEdge` | `OnAttack` | Poison stacks on hero hit; stacks tick each clock iteration as `PoisonTick` | DoT across ticks without extra swings; capped stack. |
| `ThornSkin` | `OnHitTaken` | Reflect after hero loses HP from an enemy hit | Skipped when barrier absorbs all (`hp_loss == 0`). |
| `BarrierPulse` | `OnRoomStart` | Shield at combat start; absorbs before HP | One pulse per combat. |

**Mismatches resolved (roadmap Phase A):** poison periodic ticks, guard/heavy tuning, barrier/thorns ordering — covered by tests in `src/domain/combat.rs`.

**Future nuance:** descriptions could mention stack caps or exact formulas if players need transparency; optional copy-only follow-ups.
