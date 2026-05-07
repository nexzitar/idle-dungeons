use crate::domain::dungeon::Enemy;
use crate::domain::hero::HeroProfile;
use crate::domain::items::ItemAffix;
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
    PoisonTick { damage: i32 },
    EnemyAttacked { damage: i32 },
    ThornsReflect { damage: i32 },
    HeroHealed { amount: i32 },
    EnemyDefeated,
    HeroDefeated,
}

fn combat_event_caption(event: &CombatEvent) -> String {
    match event {
        CombatEvent::HeroAttacked { damage } => format!("You strike for {} damage.", damage),
        CombatEvent::PoisonTick { damage } => format!("Poison deals {} damage.", damage),
        CombatEvent::EnemyAttacked { damage } => {
            format!("{} hits you for {} damage.", "The foe", damage)
        }
        CombatEvent::ThornsReflect { damage } => {
            format!("Thorns bite back for {} damage.", damage)
        }
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

/// `max_clock_ticks` — upper bound on combat time steps (each step adds attack speed to both
/// sides' action meters; extra hero or enemy swings in one step when speed is higher).
pub fn simulate_combat(
    hero: &HeroProfile,
    enemy: &Enemy,
    max_clock_ticks: u32,
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

    let skills: Vec<SkillId> = hero.equipped_skill_ids().collect();
    let has = |id: SkillId| skills.iter().any(|&s| s == id);

    let has_lifesteal = has(SkillId::LifestealStrike);
    let has_guard = has(SkillId::Guard);
    let has_heavy = has(SkillId::HeavyStrike);
    let has_poison = has(SkillId::PoisonEdge);
    let has_thorns = has(SkillId::ThornSkin);
    let has_barrier = has(SkillId::BarrierPulse);

    let affix_heavy = hero.has_affix(ItemAffix::Heavy);
    let affix_vamp = hero.has_affix(ItemAffix::Vampiric);
    let affix_spiked = hero.has_affix(ItemAffix::Spiked);
    let affix_cursed = hero.has_affix(ItemAffix::Cursed);

    let mut barrier = if has_barrier {
        let mut b = (10 + stats.healing_power.saturating_mul(2)).clamp(4, max_h / 2);
        if affix_cursed {
            b = (b * 3 / 4).max(2);
        }
        b
    } else {
        0
    };

    let poison_tick = (3 + stats.healing_power.max(0) / 2).clamp(1, 25);

    // Guard: flat reduction on each foe hit; scales with healing_power (baseline 3 when HP stat is 0).
    let guard_flat = (3 + stats.healing_power.max(0) / 2).clamp(3, 25);

    let mut hero_as = stats.attack_speed.max(0.12);
    // Heavy Strike: slower pacing (skill text); penalty after gear is summed into attack_speed.
    if has_heavy {
        hero_as *= 0.75;
        hero_as = hero_as.max(0.12);
    }
    let enemy_as = enemy.attack_speed.max(0.12);
    let mut hero_meter = 0.0_f32;
    let mut enemy_meter = 0.0_f32;

    // Poison Edge: stacks on hit; one stack consumed at end of each clock iteration (DoT).
    let mut poison_stacks: u32 = 0;

    let mut events = Vec::new();

    const MAX_EVENTS: usize = 600;

    macro_rules! hero_win {
        () => {
            return CombatResult {
                outcome: CombatOutcome::HeroWon,
                hero_health,
                enemy_health,
                events,
            };
        };
    }
    macro_rules! enemy_win {
        () => {
            return CombatResult {
                outcome: CombatOutcome::EnemyWon,
                hero_health,
                enemy_health,
                events,
            };
        };
    }

    for _ in 0..max_clock_ticks {
        if hero_health <= 0 || enemy_health <= 0 {
            break;
        }
        if events.len() >= MAX_EVENTS {
            break;
        }

        hero_meter += hero_as;
        enemy_meter += enemy_as;

        while hero_meter >= 1.0 && hero_health > 0 && enemy_health > 0 {
            hero_meter -= 1.0;
            if events.len() >= MAX_EVENTS {
                break;
            }

            let mut hero_damage = (stats.damage - enemy.armor).max(1);
            if has_heavy {
                hero_damage += hero_damage / 2;
            }
            if has_heavy && affix_heavy {
                hero_damage += hero_damage / 5;
            }

            enemy_health -= hero_damage;
            events.push(CombatEvent::HeroAttacked {
                damage: hero_damage,
            });

            if has_lifesteal {
                let mut amount = (hero_damage / 4).max(1);
                if affix_vamp {
                    amount = (hero_damage / 3).max(1);
                }
                hero_health = (hero_health + amount).min(max_h);
                events.push(CombatEvent::HeroHealed { amount });
            }

            if has_poison {
                poison_stacks = (poison_stacks + 2).min(40);
            }

            if enemy_health <= 0 {
                events.push(CombatEvent::EnemyDefeated);
                hero_win!();
            }
        }

        while enemy_meter >= 1.0 && hero_health > 0 && enemy_health > 0 {
            enemy_meter -= 1.0;
            if events.len() >= MAX_EVENTS {
                break;
            }

            let mut enemy_damage = (enemy.damage - stats.armor).max(1);
            if has_guard {
                enemy_damage = (enemy_damage - guard_flat).max(1);
            }

            let absorbed = enemy_damage.min(barrier);
            barrier -= absorbed;
            let hp_loss = enemy_damage - absorbed;
            hero_health -= hp_loss;
            events.push(CombatEvent::EnemyAttacked { damage: hp_loss });

            if has_thorns && hp_loss > 0 {
                let mut reflect = (hp_loss / 3).max(1);
                if affix_spiked {
                    reflect = (hp_loss / 2).max(1);
                }
                enemy_health -= reflect;
                events.push(CombatEvent::ThornsReflect { damage: reflect });
                if enemy_health <= 0 {
                    events.push(CombatEvent::EnemyDefeated);
                    hero_win!();
                }
            }

            if hero_health <= 0 {
                events.push(CombatEvent::HeroDefeated);
                enemy_win!();
            }
        }

        if has_poison && poison_stacks > 0 && hero_health > 0 && enemy_health > 0 {
            if events.len() >= MAX_EVENTS {
                break;
            }
            let d = poison_tick.min(enemy_health);
            enemy_health -= d;
            poison_stacks -= 1;
            events.push(CombatEvent::PoisonTick { damage: d });
            if enemy_health <= 0 {
                events.push(CombatEvent::EnemyDefeated);
                hero_win!();
            }
        }
    }

    let outcome = if hero_health <= 0 {
        CombatOutcome::EnemyWon
    } else if enemy_health <= 0 {
        CombatOutcome::HeroWon
    } else {
        CombatOutcome::TimedOut
    };

    CombatResult {
        outcome,
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
            CombatEvent::PoisonTick { damage } | CombatEvent::ThornsReflect { damage } => {
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

pub fn combat_playback_frames(
    hero: &HeroProfile,
    enemy: &Enemy,
    max_ticks: u32,
) -> Vec<CombatPlaybackFrame> {
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
    use crate::domain::items::{GearSlot, ItemAffix, ItemInstance};
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
    fn guard_reduces_incoming_damage_beyond_armor() {
        let mut plain = HeroProfile::default();
        plain.unlock_skill_slots(1);
        let mut guarded = HeroProfile::default();
        guarded.unlock_skill_slots(1);
        guarded.equip_skill(0, SkillId::Guard).unwrap();

        let enemy = Enemy {
            name: "Spiker".into(),
            max_health: 999,
            damage: 8,
            armor: 0,
            attack_speed: 1.0,
        };

        let a = simulate_combat(&plain, &enemy, 1, 100);
        let b = simulate_combat(&guarded, &enemy, 1, 100);
        assert!(
            b.hero_health > a.hero_health,
            "guard should leave more hero hp after one exchange"
        );
    }

    #[test]
    fn guard_reduction_scales_with_healing_power() {
        let enemy = Enemy {
            name: "Baseline hit".into(),
            max_health: 999,
            damage: 24,
            armor: 0,
            attack_speed: 1.0,
        };

        let mut low = HeroProfile::new(Stats {
            healing_power: 0,
            ..Stats::default()
        });
        low.unlock_skill_slots(1);
        low.equip_skill(0, SkillId::Guard).unwrap();

        let mut high = HeroProfile::new(Stats {
            healing_power: 10,
            ..Stats::default()
        });
        high.unlock_skill_slots(1);
        high.equip_skill(0, SkillId::Guard).unwrap();

        let r_low = simulate_combat(&low, &enemy, 1, 100);
        let r_high = simulate_combat(&high, &enemy, 1, 100);
        assert!(
            r_high.hero_health > r_low.hero_health,
            "more healing_power should strengthen Guard flat reduction"
        );
    }

    #[test]
    fn heavy_strike_increases_weapon_damage() {
        let mut plain = HeroProfile::default();
        plain.unlock_skill_slots(1);
        let mut heavy = HeroProfile::default();
        heavy.unlock_skill_slots(1);
        heavy.equip_skill(0, SkillId::HeavyStrike).unwrap();

        let enemy = Enemy {
            name: "Dummy".into(),
            max_health: 999,
            damage: 0,
            armor: 0,
            attack_speed: 1.0,
        };

        let a = simulate_combat(&plain, &enemy, 2, 100);
        let b = simulate_combat(&heavy, &enemy, 2, 100);
        let plain_fist = a.events.iter().find_map(|e| {
            if let CombatEvent::HeroAttacked { damage } = e {
                Some(*damage)
            } else {
                None
            }
        });
        let heavy_fist = b.events.iter().find_map(|e| {
            if let CombatEvent::HeroAttacked { damage } = e {
                Some(*damage)
            } else {
                None
            }
        });
        assert_eq!(plain_fist, Some(10));
        assert_eq!(heavy_fist, Some(15));
    }

    #[test]
    fn heavy_strike_slows_attack_pacing() {
        let mut nimble = HeroProfile::default();
        nimble.base_stats.attack_speed = 2.0;
        nimble.unlock_skill_slots(1);

        let mut heavy = HeroProfile::default();
        heavy.base_stats.attack_speed = 2.0;
        heavy.unlock_skill_slots(1);
        heavy.equip_skill(0, SkillId::HeavyStrike).unwrap();

        let enemy = Enemy {
            name: "Dummy".into(),
            max_health: 999,
            damage: 0,
            armor: 0,
            attack_speed: 1.0,
        };

        let r_nimble = simulate_combat(&nimble, &enemy, 1, 100);
        let r_heavy = simulate_combat(&heavy, &enemy, 1, 100);

        let swings_nimble = r_nimble
            .events
            .iter()
            .filter(|e| matches!(e, CombatEvent::HeroAttacked { .. }))
            .count();
        let swings_heavy = r_heavy
            .events
            .iter()
            .filter(|e| matches!(e, CombatEvent::HeroAttacked { .. }))
            .count();

        assert_eq!(swings_nimble, 2);
        assert_eq!(swings_heavy, 1);
    }

    #[test]
    fn poison_edge_deals_extra_damage() {
        let mut hero = HeroProfile::default();
        hero.unlock_skill_slots(1);
        hero.equip_skill(0, SkillId::PoisonEdge).unwrap();

        let enemy = Enemy {
            name: "Dummy".into(),
            max_health: 999,
            damage: 0,
            armor: 0,
            attack_speed: 1.0,
        };

        let result = simulate_combat(&hero, &enemy, 1, 100);
        assert!(result.events.iter().any(|e| matches!(
            e,
            CombatEvent::PoisonTick { damage } if *damage > 0
        )));
    }

    #[test]
    fn poison_deals_damage_across_clock_ticks() {
        let mut hero = HeroProfile::default();
        hero.base_stats.attack_speed = 0.12;
        hero.unlock_skill_slots(1);
        hero.equip_skill(0, SkillId::PoisonEdge).unwrap();

        let enemy = Enemy {
            name: "Pacer".into(),
            max_health: 999,
            damage: 1,
            armor: 0,
            attack_speed: 1.0,
        };

        let result = simulate_combat(&hero, &enemy, 12, hero.derived_stats().max_health);
        let hero_swings = result
            .events
            .iter()
            .filter(|e| matches!(e, CombatEvent::HeroAttacked { .. }))
            .count();
        let poison_ticks = result
            .events
            .iter()
            .filter(|e| matches!(e, CombatEvent::PoisonTick { .. }))
            .count();

        assert_eq!(
            hero_swings, 1,
            "slow hero should land one strike in this tick budget"
        );
        assert!(
            poison_ticks >= 2,
            "poison should tick on later clock iterations without a second hero swing, got {poison_ticks} PoisonTick events"
        );
    }

    #[test]
    fn thorns_reflect_hurts_enemy_on_hit() {
        let mut hero = HeroProfile::default();
        hero.unlock_skill_slots(1);
        hero.equip_skill(0, SkillId::ThornSkin).unwrap();

        let enemy = Enemy {
            name: "Bruiser".into(),
            max_health: 999,
            damage: 9,
            armor: 0,
            attack_speed: 1.0,
        };

        let result = simulate_combat(&hero, &enemy, 1, 100);
        assert!(result.events.iter().any(|e| matches!(
            e,
            CombatEvent::ThornsReflect { damage } if *damage > 0
        )));
    }

    #[test]
    fn barrier_pulse_absorbs_part_of_first_hit() {
        let mut hero = HeroProfile::default();
        hero.unlock_skill_slots(1);
        hero.equip_skill(0, SkillId::BarrierPulse).unwrap();

        let enemy = Enemy {
            name: "Bruiser".into(),
            max_health: 999,
            damage: 12,
            armor: 0,
            attack_speed: 1.0,
        };

        let with_barrier = simulate_combat(&hero, &enemy, 1, 100);
        let mut plain = HeroProfile::default();
        plain.unlock_skill_slots(1);
        let no_barrier = simulate_combat(&plain, &enemy, 1, 100);

        assert!(
            with_barrier.hero_health > no_barrier.hero_health,
            "barrier should reduce hp lost to the first strike"
        );
    }

    #[test]
    fn high_attack_speed_hero_strikes_more_per_clock_tick() {
        let mut fast = HeroProfile::default();
        fast.base_stats.attack_speed = 2.0;
        let enemy = Enemy {
            name: "Pacing dummy".into(),
            max_health: 999,
            damage: 1,
            armor: 0,
            attack_speed: 1.0,
        };

        let r = simulate_combat(&fast, &enemy, 1, 100);
        let hero_swings = r
            .events
            .iter()
            .filter(|e| matches!(e, CombatEvent::HeroAttacked { .. }))
            .count();
        let foe_swings = r
            .events
            .iter()
            .filter(|e| matches!(e, CombatEvent::EnemyAttacked { .. }))
            .count();
        assert_eq!(hero_swings, 2);
        assert_eq!(foe_swings, 1);
    }

    #[test]
    fn poison_damage_scales_with_healing_power() {
        let mut tuned = HeroProfile::default();
        tuned.base_stats.healing_power = 6;
        tuned.unlock_skill_slots(1);
        tuned.equip_skill(0, SkillId::PoisonEdge).unwrap();
        let weak = simulate_combat(&tuned, &dummy_enemy(), 1, 100);
        let weak_poison = weak.events.iter().find_map(|e| {
            if let CombatEvent::PoisonTick { damage } = e {
                Some(*damage)
            } else {
                None
            }
        });

        let mut plain = HeroProfile::default();
        plain.unlock_skill_slots(1);
        plain.equip_skill(0, SkillId::PoisonEdge).unwrap();
        let base = simulate_combat(&plain, &dummy_enemy(), 1, 100);
        let base_poison = base.events.iter().find_map(|e| {
            if let CombatEvent::PoisonTick { damage } = e {
                Some(*damage)
            } else {
                None
            }
        });

        assert!(weak_poison.unwrap() > base_poison.unwrap());
    }

    #[test]
    fn heavy_affix_synergy_boosts_heavy_strike() {
        let mut skill_only = HeroProfile::default();
        skill_only.unlock_skill_slots(1);
        skill_only.equip_skill(0, SkillId::HeavyStrike).unwrap();
        let mut club = ItemInstance::basic(1, "Club", GearSlot::Weapon);
        club.stats.damage = 4;
        skill_only.equip_item(club).unwrap();

        let mut with_affix = HeroProfile::default();
        with_affix.unlock_skill_slots(1);
        with_affix.equip_skill(0, SkillId::HeavyStrike).unwrap();
        let mut maul = ItemInstance::basic(2, "Maul", GearSlot::Weapon);
        maul.stats.damage = 0;
        maul.stats.attack_speed = 0.2;
        maul.affixes.push(ItemAffix::Heavy);
        with_affix.equip_item(maul).unwrap();

        let e = dummy_enemy();
        let a = first_hero_damage(&simulate_combat(&skill_only, &e, 2, 100));
        let b = first_hero_damage(&simulate_combat(&with_affix, &e, 2, 100));
        assert!(b > a);
        assert_eq!(a, 21);
        assert_eq!(b, 25);
    }

    #[test]
    fn vampiric_affix_improves_lifesteal_heal() {
        let mut plain = HeroProfile::default();
        plain.unlock_skill_slots(1);
        plain.equip_skill(0, SkillId::LifestealStrike).unwrap();

        let mut geared = HeroProfile::default();
        geared.unlock_skill_slots(1);
        geared.equip_skill(0, SkillId::LifestealStrike).unwrap();
        let mut blade = ItemInstance::basic(2, "Fang", GearSlot::Weapon);
        blade.affixes.push(ItemAffix::Vampiric);
        geared.equip_item(blade).unwrap();

        let e = dummy_enemy();
        let h_plain = first_heal(&simulate_combat(&plain, &e, 1, 100));
        let h_geared = first_heal(&simulate_combat(&geared, &e, 1, 100));
        assert!(h_geared > h_plain);
    }

    #[test]
    fn spiked_affix_synergy_improves_thorns() {
        let mut thorns_only = HeroProfile::default();
        thorns_only.unlock_skill_slots(1);
        thorns_only.equip_skill(0, SkillId::ThornSkin).unwrap();

        let mut both = HeroProfile::default();
        both.unlock_skill_slots(1);
        both.equip_skill(0, SkillId::ThornSkin).unwrap();
        let mut spiky = ItemInstance::basic(3, "Spiky mail", GearSlot::Armor);
        spiky.affixes.push(ItemAffix::Spiked);
        both.equip_item(spiky).unwrap();

        let enemy = Enemy {
            name: "Bruiser".into(),
            max_health: 999,
            damage: 12,
            armor: 0,
            attack_speed: 1.0,
        };

        let r1 = first_thorns(&simulate_combat(&thorns_only, &enemy, 1, 100));
        let r2 = first_thorns(&simulate_combat(&both, &enemy, 1, 100));
        assert!(r2 > r1);
        assert_eq!(r1, 4);
        assert_eq!(r2, 5);
    }

    #[test]
    fn cursed_affix_weakens_barrier_pulse() {
        let mut pure = HeroProfile::default();
        pure.unlock_skill_slots(1);
        pure.equip_skill(0, SkillId::BarrierPulse).unwrap();

        let mut cursed = HeroProfile::default();
        cursed.unlock_skill_slots(1);
        cursed.equip_skill(0, SkillId::BarrierPulse).unwrap();
        let mut cloth = ItemInstance::basic(4, "Shroud", GearSlot::Trinket);
        cloth.affixes.push(ItemAffix::Cursed);
        cursed.equip_item(cloth).unwrap();

        let enemy = Enemy {
            name: "Bruiser".into(),
            max_health: 999,
            damage: 12,
            armor: 0,
            attack_speed: 1.0,
        };

        let hp_pure = simulate_combat(&pure, &enemy, 1, 100).hero_health;
        let hp_cursed = simulate_combat(&cursed, &enemy, 1, 100).hero_health;
        assert!(
            hp_pure > hp_cursed,
            "cursed gear should shrink the barrier bundle"
        );
    }

    fn dummy_enemy() -> Enemy {
        Enemy {
            name: "Dummy".into(),
            max_health: 999,
            damage: 0,
            armor: 0,
            attack_speed: 1.0,
        }
    }

    fn first_hero_damage(r: &CombatResult) -> i32 {
        r.events
            .iter()
            .find_map(|e| {
                if let CombatEvent::HeroAttacked { damage } = e {
                    Some(*damage)
                } else {
                    None
                }
            })
            .unwrap()
    }

    fn first_heal(r: &CombatResult) -> i32 {
        r.events
            .iter()
            .find_map(|e| {
                if let CombatEvent::HeroHealed { amount } = e {
                    Some(*amount)
                } else {
                    None
                }
            })
            .unwrap()
    }

    fn first_thorns(r: &CombatResult) -> i32 {
        r.events
            .iter()
            .find_map(|e| {
                if let CombatEvent::ThornsReflect { damage } = e {
                    Some(*damage)
                } else {
                    None
                }
            })
            .unwrap()
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
