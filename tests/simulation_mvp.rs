use idle_dungeons::domain::hero::HeroProfile;
use idle_dungeons::domain::run::{simulate_run, RunConfig, RunOutcome};

#[test]
fn seeded_run_reaches_same_result_every_time() {
    let hero = HeroProfile::default();
    let config = RunConfig::new(7, 25);

    let first = simulate_run(&hero, None, config);
    let second = simulate_run(&hero, None, config);

    assert_eq!(first, second);
}

#[test]
fn run_summary_reports_depth_gold_and_outcome() {
    let hero = HeroProfile::default();
    let result = simulate_run(&hero, None, RunConfig::new(5, 25));

    assert!(result.deepest_depth >= 1);
    assert!(result.gold_earned > 0);
    assert!(matches!(
        result.outcome,
        RunOutcome::HeroDied | RunOutcome::BossDefeated
    ));
    assert!(
        !result.peak_risk_note.is_empty(),
        "peak risk hint should summarize room pressure"
    );
    assert!(
        result.encounter_score > 0,
        "encounter score should accumulate from cleared rooms"
    );
}

#[test]
fn run_summary_encounter_score_is_deterministic_for_seed() {
    let hero = HeroProfile::default();
    let config = RunConfig::new(5, 25);
    let a = simulate_run(&hero, None, config);
    let b = simulate_run(&hero, None, config);
    assert_eq!(a.encounter_score, b.encounter_score);
}

#[test]
fn run_summary_reports_playback_log() {
    let hero = HeroProfile::default();
    let result = simulate_run(&hero, None, RunConfig::new(5, 25));

    assert!(!result.log.is_empty());
    assert!(result.log.first().unwrap().contains("Depth"));
}

#[test]
fn gold_gain_multiplier_scales_run_gold() {
    let hero = HeroProfile::default();
    let seed = 11u64;
    let base = simulate_run(&hero, None, RunConfig::new(seed, 25));
    let boosted = simulate_run(
        &hero,
        None,
        RunConfig {
            seed,
            max_depth: 25,
            gold_gain_multiplier: 1.2,
            guided_early_combat_claims_already: 0,
        },
    );
    let expected = (base.gold_earned as f32 * 1.2).round() as u32;
    assert_eq!(boosted.gold_earned, expected);
}
