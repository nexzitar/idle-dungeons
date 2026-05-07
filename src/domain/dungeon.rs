use crate::domain::stats::Stats;
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RoomKind {
    Monster,
    Elite,
    Treasure,
    Shrine,
    Boss,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Enemy {
    pub name: String,
    pub max_health: i32,
    pub damage: i32,
    pub armor: i32,
    pub attack_speed: f32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Encounter {
    pub enemy: Enemy,
}

impl Encounter {
    pub fn monster_for_depth(depth: u32, elite: bool) -> Self {
        let multiplier = if elite { 2 } else { 1 };
        Self {
            enemy: Enemy {
                name: if elite {
                    "Elite Hollow".into()
                } else {
                    "Hollow".into()
                },
                max_health: (24 + depth as i32 * 6) * multiplier,
                damage: (3 + depth as i32 / 2) * multiplier,
                armor: depth as i32 / 5,
                attack_speed: if elite { 0.9 } else { 1.05 },
            },
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DungeonRoom {
    pub depth: u32,
    pub kind: RoomKind,
    pub encounter: Option<Encounter>,
}

/// Builds a linear sequence `depth` = 1 ..= `depth_count`.
///
/// **Layout rules**
/// - Final depth: [`RoomKind::Boss`] (fixed encounter).
/// - Depths divisible by 10: [`RoomKind::Elite`].
/// - **Seeded vein:** depth **11** is always [`RoomKind::Treasure`] when `seed % 97 == 11` (no combat).
/// - All other non-boss depths: one RNG draw (ChaCha8, `seed`) — 10% [`Treasure`], 10% [`Shrine`],
///   80% [`RoomKind::Monster`]. Treasure and shrine have no encounter; monster/elite/boss do.
pub fn generate_dungeon(depth_count: u32, seed: u64) -> Vec<DungeonRoom> {
    let mut rng = ChaCha8Rng::seed_from_u64(seed);
    (1..=depth_count)
        .map(|depth| {
            let kind = if depth == depth_count {
                RoomKind::Boss
            } else if depth % 10 == 0 {
                RoomKind::Elite
            } else if depth == 11 && seed % 97 == 11 {
                // Deterministic “lucky vein” floor: same seed always sees treasure at depth 11.
                RoomKind::Treasure
            } else {
                match rng.gen_range(0..10) {
                    0 => RoomKind::Treasure,
                    1 => RoomKind::Shrine,
                    _ => RoomKind::Monster,
                }
            };

            let encounter = match kind {
                RoomKind::Monster => Some(Encounter::monster_for_depth(depth, false)),
                RoomKind::Elite => Some(Encounter::monster_for_depth(depth, true)),
                RoomKind::Boss => Some(Encounter {
                    enemy: Enemy {
                        name: "Gate Warden".into(),
                        max_health: 250,
                        damage: 18,
                        armor: 4,
                        attack_speed: 0.8,
                    },
                }),
                RoomKind::Treasure | RoomKind::Shrine => None,
            };

            DungeonRoom {
                depth,
                kind,
                encounter,
            }
        })
        .collect()
}

/// Short risk tier for UI (playback type line, tooltips).
pub fn room_risk_hint(kind: RoomKind) -> &'static str {
    match kind {
        RoomKind::Treasure | RoomKind::Shrine => "Low",
        RoomKind::Monster => "Moderate",
        RoomKind::Elite => "High",
        RoomKind::Boss => "Boss",
    }
}

/// Ordinal for comparing danger across a delve (0 = calm, 3 = boss).
pub fn room_risk_rank(kind: RoomKind) -> u8 {
    match kind {
        RoomKind::Treasure | RoomKind::Shrine => 0,
        RoomKind::Monster => 1,
        RoomKind::Elite => 2,
        RoomKind::Boss => 3,
    }
}

/// Summary line after a full or partial run (based on max rank seen).
pub fn peak_risk_note(max_rank: u8) -> String {
    match max_rank {
        0 => "Peak room risk: low (loot or shrines only).".to_string(),
        1 => "Peak room risk: moderate (standard combat).".to_string(),
        2 => "Peak room risk: high (elite encounters).".to_string(),
        3 => "Peak room risk: boss.".to_string(),
        _ => "Peak room risk: —".to_string(),
    }
}

impl From<&Enemy> for Stats {
    fn from(enemy: &Enemy) -> Self {
        Self {
            max_health: enemy.max_health,
            damage: enemy.damage,
            armor: enemy.armor,
            attack_speed: enemy.attack_speed,
            healing_power: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn seeded_depth_11_treasure_when_seed_mod_97_eq_11() {
        let rooms = generate_dungeon(25, 108);
        let r11 = rooms.iter().find(|r| r.depth == 11).unwrap();
        assert_eq!(r11.kind, RoomKind::Treasure);
        assert!(r11.encounter.is_none());
    }

    #[test]
    fn generated_dungeon_is_repeatable_for_seed() {
        let first = generate_dungeon(25, 123);
        let second = generate_dungeon(25, 123);

        assert_eq!(first, second);
    }

    #[test]
    fn milestone_boss_is_at_final_depth() {
        let rooms = generate_dungeon(25, 123);

        assert_eq!(rooms.last().unwrap().depth, 25);
        assert_eq!(rooms.last().unwrap().kind, RoomKind::Boss);
    }

    #[test]
    fn combat_rooms_always_have_encounters_non_combat_never() {
        for seed in [0_u64, 1, 7, 42, 108, 999] {
            let rooms = generate_dungeon(25, seed);
            for room in &rooms {
                match room.kind {
                    RoomKind::Monster | RoomKind::Elite | RoomKind::Boss => assert!(
                        room.encounter.is_some(),
                        "seed {seed} depth {} {:?} must include encounter",
                        room.depth,
                        room.kind
                    ),
                    RoomKind::Treasure | RoomKind::Shrine => {
                        assert!(
                            room.encounter.is_none(),
                            "seed {seed} depth {} {:?} must omit encounter",
                            room.depth,
                            room.kind
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn depth_scaling_increases_enemy_strength() {
        let shallow = Encounter::monster_for_depth(1, false);
        let deep = Encounter::monster_for_depth(20, false);

        assert!(deep.enemy.max_health > shallow.enemy.max_health);
        assert!(deep.enemy.damage > shallow.enemy.damage);
    }
}
