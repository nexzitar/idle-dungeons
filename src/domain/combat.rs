use crate::domain::dungeon::Enemy;
use crate::domain::hero::HeroProfile;
use crate::domain::skills::SkillId;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CombatOutcome {
    HeroWon,
    EnemyWon,
    TimedOut,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CombatEvent {
    HeroAttacked { damage: i32 },
    EnemyAttacked { damage: i32 },
    HeroHealed { amount: i32 },
    EnemyDefeated,
    HeroDefeated,
}

fn combat_event_caption(event: &CombatEvent) -> String {
    match event {
        CombatEvent::HeroAttacked { damage } => format!("You strike for {} damage.", damage),
        CombatEvent::EnemyAttacked { damage } => format!("{} hits you for {} damage.", "The foe", damage),
        CombatEvent::HeroHealed { amount } => format!("You recover {} health.", amount),
        CombatEvent::EnemyDefeated => "Enemy defeated.".to_string(),
        CombatEvent::HeroDefeated => "You collapse...".to_string(),
    }
}

/// One row of combat UI: HP totals after a combat event (plus an opening "engage" row).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CombatPlaybackFrame {
    pub enemy_name: String,
    pub hero_hp: i32,
    pub hero_max_hp: i32,
    pub enemy_hp: i32,
    pub enemy_max_hp: i32,
    pub caption: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CombatResult {
    pub outcome: CombatOutcome,
    pub hero_health: i32,
    pub enemy_health: i32,
    pub events: Vec<CombatEvent>,
}

pub fn simulate_combat(
    hero: &HeroProfile,
    enemy: &Enemy,
    max_ticks: u32,
    hero_health_start: i32,
) -> CombatResult {
    let stats = hero.derived_stats();
    let max_h = stats.max_health;
    let mut hero_health = hero_health_start.clamp(0, max_h);
    if hero_health <= 0 {
        return CombatResult {
            outcome: CombatOutcome::EnemyWon,
            hero_health: 0,
            enemy_health: enemy.max_health,
            events: vec![CombatEvent::HeroDefeated],
        };
    }
    let mut enemy_health = enemy.max_health;
    let has_lifesteal = hero
        .equipped_skill_ids()
        .any(|skill| skill == SkillId::LifestealStrike);
    let mut events = Vec::new();

    for _ in 0..max_ticks {
        let hero_damage = (stats.damage - enemy.armor).max(1);
        enemy_health -= hero_damage;
        events.push(CombatEvent::HeroAttacked {
            damage: hero_damage,
        });

        if has_lifesteal {
            let amount = (hero_damage / 4).max(1);
            hero_health = (hero_health + amount).min(stats.max_health);
            events.push(CombatEvent::HeroHealed { amount });
        }

        if enemy_health <= 0 {
            events.push(CombatEvent::EnemyDefeated);
            return CombatResult {
                outcome: CombatOutcome::HeroWon,
                hero_health,
                enemy_health,
                events,
            };
        }

        let enemy_damage = (enemy.damage - stats.armor).max(1);
        hero_health -= enemy_damage;
        events.push(CombatEvent::EnemyAttacked {
            damage: enemy_damage,
        });

        if hero_health <= 0 {
            events.push(CombatEvent::HeroDefeated);
            return CombatResult {
                outcome: CombatOutcome::EnemyWon,
                hero_health,
                enemy_health,
                events,
            };
        }
    }

    CombatResult {
        outcome: CombatOutcome::TimedOut,
        hero_health,
        enemy_health,
        events,
    }
}

pub fn combat_playback_frames_from_result(
    result: &CombatResult,
    enemy_name: &str,
    hero_max_hp: i32,
    enemy_max_hp: i32,
    hero_hp_at_start: i32,
) -> Vec<CombatPlaybackFrame> {
    let mut hero_hp = hero_hp_at_start.clamp(0, hero_max_hp);
    let mut enemy_hp = enemy_max_hp;

    let mut frames = vec![CombatPlaybackFrame {
        enemy_name: enemy_name.to_string(),
        hero_hp,
        hero_max_hp,
        enemy_hp,
        enemy_max_hp,
        caption: format!("Engaging {enemy_name}."),
    }];

    for event in &result.events {
        match event {
            CombatEvent::HeroAttacked { damage } => {
                enemy_hp -= damage;
            }
            CombatEvent::HeroHealed { amount } => {
                hero_hp = (hero_hp + amount).min(hero_max_hp);
            }
            CombatEvent::EnemyAttacked { damage } => {
                hero_hp -= damage;
            }
            CombatEvent::EnemyDefeated | CombatEvent::HeroDefeated => {}
        }
        frames.push(CombatPlaybackFrame {
            enemy_name: enemy_name.to_string(),
            hero_hp: hero_hp.clamp(0, hero_max_hp),
            hero_max_hp,
            enemy_hp: enemy_hp.clamp(0, enemy_max_hp),
            enemy_max_hp,
            caption: combat_event_caption(event),
        });
    }
    frames
}

pub fn combat_playback_frames(hero: &HeroProfile, enemy: &Enemy, max_ticks: u32) -> Vec<CombatPlaybackFrame> {
    let stats = hero.derived_stats();
    let hero_start = stats.max_health;
    let result = simulate_combat(hero, enemy, max_ticks, hero_start);
    combat_playback_frames_from_result(
        &result,
        &enemy.name,
        stats.max_health,
        enemy.max_health,
        hero_start,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::dungeon::Enemy;
    use crate::domain::hero::HeroProfile;
    use crate::domain::skills::SkillId;
    use crate::domain::stats::Stats;

    #[test]
    fn hero_defeats_weaker_enemy() {
        let mut hero = HeroProfile::new(Stats {
            damage: 12,
            ..Stats::default()
        });
        hero.unlock_skill_slots(1);

        let enemy = Enemy {
            name: "Training Hollow".into(),
            max_health: 20,
            damage: 1,
            armor: 0,
            attack_speed: 1.0,
        };

        let result = simulate_combat(&hero, &enemy, 100, hero.derived_stats().max_health);

        assert_eq!(result.outcome, CombatOutcome::HeroWon);
        assert!(result
            .events
            .iter()
            .any(|event| matches!(event, CombatEvent::EnemyDefeated)));
    }

    #[test]
    fn lifesteal_skill_restores_health_on_attack() {
        let mut hero = HeroProfile::default();
        hero.unlock_skill_slots(1);
        hero.equip_skill(0, SkillId::LifestealStrike).unwrap();

        let enemy = Enemy {
            name: "Durable Hollow".into(),
            max_health: 60,
            damage: 10,
            armor: 0,
            attack_speed: 1.0,
        };

        let result = simulate_combat(&hero, &enemy, 5, hero.derived_stats().max_health);

        assert!(result
            .events
            .iter()
            .any(|event| matches!(event, CombatEvent::HeroHealed { amount } if *amount > 0)));
    }

    #[test]
    fn armor_reduces_incoming_damage_compared_to_unarmored_hero() {
        let unarmored_hero = HeroProfile::default();
        let armored_hero = HeroProfile::new(Stats {
            armor: 3,
            ..Stats::default()
        });
        let enemy = Enemy {
            name: "Rustblade Hollow".into(),
            max_health: 999,
            damage: 5,
            armor: 0,
            attack_speed: 1.0,
        };

        let unarmored_result = simulate_combat(
            &unarmored_hero,
            &enemy,
            1,
            unarmored_hero.derived_stats().max_health,
        );
        let armored_result = simulate_combat(
            &armored_hero,
            &enemy,
            1,
            armored_hero.derived_stats().max_health,
        );
        let unarmored_damage_taken =
            unarmored_hero.derived_stats().max_health - unarmored_result.hero_health;
        let armored_damage_taken =
            armored_hero.derived_stats().max_health - armored_result.hero_health;

        assert!(
            armored_damage_taken < unarmored_damage_taken,
            "expected armor to reduce incoming damage below {unarmored_damage_taken}, got {armored_damage_taken}"
        );
    }

    #[test]
    fn incoming_damage_is_at_least_one_when_armor_exceeds_enemy_damage() {
        let hero = HeroProfile::new(Stats {
            armor: 99,
            ..Stats::default()
        });
        let enemy = Enemy {
            name: "Weak Hollow".into(),
            max_health: 999,
            damage: 5,
            armor: 0,
            attack_speed: 1.0,
        };

        let result = simulate_combat(&hero, &enemy, 1, hero.derived_stats().max_health);

        let damage_taken = hero.derived_stats().max_health - result.hero_health;
        assert!(
            damage_taken >= 1,
            "expected at least 1 damage when enemy damage is floored to 1"
        );
    }

    #[test]
    fn next_encounter_starts_at_remaining_hp() {
        let hero = HeroProfile::default();
        let enemy = Enemy {
            name: "Poker".into(),
            max_health: 999,
            damage: 3,
            armor: 0,
            attack_speed: 1.0,
        };
        let first = simulate_combat(&hero, &enemy, 1, 100);
        assert_eq!(first.hero_health, 97);
        let second = simulate_combat(&hero, &enemy, 1, first.hero_health);
        assert_eq!(second.hero_health, 94);
    }
}
