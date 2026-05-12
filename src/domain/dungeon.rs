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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct Enemy {
    #[serde(default)]
    pub name: String,
    pub max_health: i32,
    pub damage: i32,
    pub armor: i32,
    pub attack_speed: f32,
    /// Simulated wind-up ticks before a strike (telegraph / cast bar).
    #[serde(default)]
    pub cast_ticks: u32,
    /// Simulated recovery ticks after a strike.
    #[serde(default)]
    pub cooldown_ticks: u32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Encounter {
    pub enemy: Enemy,
    /// Second foe in the same room (dual-/multi-pack). Omitted in saves / JSON → single-foe.
    #[serde(default)]
    pub enemy_b: Option<Enemy>,
}

impl Encounter {
    /// `1` or `2` enemy slots in combat.
    pub fn foe_count(&self) -> u8 {
        1u8.saturating_add(self.enemy_b.is_some() as u8)
    }

    pub fn monster_for_depth(depth: u32, elite: bool) -> Self {
        Self::monster_body(depth, elite, false)
    }

    fn monster_body(depth: u32, elite: bool, twin_pack: bool) -> Self {
        let mult = if elite { 2 } else { 1 };
        let base_hp = 24 + depth as i32 * 6;
        let mut max_health = base_hp * mult;
        let mut dmg = (3 + depth as i32 / 2) * mult;
        let mut armor = depth as i32 / 5;
        // Milestone elites need to threaten early runs that already earned a loot piece.
        if elite && depth == 10 {
            max_health = (max_health * 135 / 100).max(base_hp * 3 / 2);
            dmg += 4;
            armor += 2;
        }
        if twin_pack {
            max_health = max_health.saturating_mul(4).saturating_div(5);
            dmg = dmg.saturating_mul(9).saturating_div(10).max(1);
        }
        let name = match (elite, twin_pack) {
            (true, true) => "Twin elite hollows",
            (true, false) => "Elite Hollow",
            (false, true) => "Twin hollows",
            (false, false) => "Hollow",
        };
        Self {
            enemy: Enemy {
                name: name.into(),
                max_health,
                damage: dmg,
                armor,
                attack_speed: if elite { 0.9 } else { 1.05 },
                cast_ticks: if elite { 2 } else { 1 },
                cooldown_ticks: if elite { 4 } else { 3 },
            },
            enemy_b: None,
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
                RoomKind::Monster => {
                    let twin = rng.gen_bool(0.22);
                    let mut e = Encounter::monster_body(depth, false, false);
                    if twin {
                        e.enemy.name = "Twin hollows".into();
                        let mut flank = Encounter::monster_body(depth, false, true).enemy;
                        flank.name = "Hollow (flank)".into();
                        e.enemy_b = Some(flank);
                    }
                    Some(e)
                }
                RoomKind::Elite => {
                    let twin = rng.gen_bool(0.30);
                    Some(if twin {
                        let mut e = Encounter::monster_body(depth, true, true);
                        e.enemy.name = "Twin elite hollows".into();
                        let mut flank = Encounter::monster_body(depth, true, true).enemy;
                        flank.name = "Elite hollow (flank)".into();
                        e.enemy_b = Some(flank);
                        e
                    } else {
                        Encounter::monster_for_depth(depth, true)
                    })
                }
                RoomKind::Boss => Some(Encounter {
                    enemy: Enemy {
                        name: "Gate Warden".into(),
                        max_health: 168,
                        damage: 12,
                        armor: 3,
                        attack_speed: 0.75,
                        cast_ticks: 2,
                        cooldown_ticks: 5,
                    },
                    enemy_b: None,
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
    fn elite_room_can_spawn_twin_pack() {
        let found = (0_u64..4000).any(|seed| {
            let rooms = generate_dungeon(35, seed);
            rooms.iter().any(|r| {
                r.kind == RoomKind::Elite
                    && r.encounter
                        .as_ref()
                        .is_some_and(|e| e.enemy_b.is_some())
            })
        });
        assert!(
            found,
            "expected some seed to roll elite twin within 4000 tries"
        );
    }

    #[test]
    fn depth_scaling_increases_enemy_strength() {
        let shallow = Encounter::monster_for_depth(1, false);
        let deep = Encounter::monster_for_depth(20, false);

        assert!(deep.enemy.max_health > shallow.enemy.max_health);
        assert!(deep.enemy.damage > shallow.enemy.damage);
    }
}
