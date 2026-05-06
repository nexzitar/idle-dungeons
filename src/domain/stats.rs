use serde::{Deserialize, Serialize};
use std::ops::Add;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Stats {
    pub max_health: i32,
    pub damage: i32,
    pub armor: i32,
    pub attack_speed: f32,
    pub healing_power: i32,
}

impl Stats {
    pub fn default_zero() -> Self {
        Self {
            max_health: 0,
            damage: 0,
            armor: 0,
            attack_speed: 0.0,
            healing_power: 0,
        }
    }
}

impl Default for Stats {
    fn default() -> Self {
        Self {
            max_health: 100,
            damage: 10,
            armor: 0,
            attack_speed: 1.0,
            healing_power: 0,
        }
    }
}

impl Add for Stats {
    type Output = Stats;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            max_health: self.max_health + rhs.max_health,
            damage: self.damage + rhs.damage,
            armor: self.armor + rhs.armor,
            attack_speed: self.attack_speed + rhs.attack_speed,
            healing_power: self.healing_power + rhs.healing_power,
        }
    }
}
