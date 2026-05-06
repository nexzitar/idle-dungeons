# MVP Acceptance Checklist

- [x] `cargo fmt --check` passes.
- [x] `cargo test` passes.
- [x] `cargo check` passes.
- [x] A seeded run returns the same summary on repeated executions.
- [x] Hero builds derive stats from base stats, skills, gear, affixes, and permanent upgrade bonuses.
- [x] Locked skill slots reject equipped skills.
- [x] Gear only equips into matching gear slots.
- [x] Dungeon generation places the milestone boss at depth 25.
- [x] Combat resolves to hero victory, hero death, or timeout.
- [x] Loot rolls are deterministic under seed and depth.
- [x] Permanent upgrades spend gold and update levels.
- [x] Save/load round trips preserve profile data.
- [x] UI exposes build, summary, inventory, salvage, equip, upgrade, and run-again actions.
- [x] Run rewards apply to the persistent profile exactly once.
- [x] Profile progress saves after reward, equipment, salvage, and upgrade changes.
