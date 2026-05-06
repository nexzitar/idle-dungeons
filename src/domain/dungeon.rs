use crate::domain::stats::Stats;
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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
                attack_speed: 1.0,
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

pub fn generate_dungeon(depth_count: u32, seed: u64) -> Vec<DungeonRoom> {
    let mut rng = ChaCha8Rng::seed_from_u64(seed);
    (1..=depth_count)
        .map(|depth| {
            let kind = if depth == depth_count {
                RoomKind::Boss
            } else if depth % 10 == 0 {
                RoomKind::Elite
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
    fn depth_scaling_increases_enemy_strength() {
        let shallow = Encounter::monster_for_depth(1, false);
        let deep = Encounter::monster_for_depth(20, false);

        assert!(deep.enemy.max_health > shallow.enemy.max_health);
        assert!(deep.enemy.damage > shallow.enemy.damage);
    }
}
