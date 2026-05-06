# MVP Acceptance Checklist

- [ ] `cargo fmt --check` passes.
- [ ] `cargo test` passes.
- [ ] `cargo check` passes.
- [ ] A seeded run returns the same summary on repeated executions.
- [ ] Hero builds derive stats from base stats, skills, gear, and affixes.
- [ ] Locked skill slots reject equipped skills.
- [ ] Gear only equips into matching gear slots.
- [ ] Dungeon generation places the milestone boss at depth 25.
- [ ] Combat resolves to hero victory, hero death, or timeout.
- [ ] Loot rolls are deterministic under seed and depth.
- [ ] Permanent upgrades spend gold and update levels.
- [ ] Save/load round trips preserve profile data.
- [ ] UI text models expose build and summary information.
