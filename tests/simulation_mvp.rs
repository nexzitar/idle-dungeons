use idle_dungeons::domain::hero::HeroProfile;
use idle_dungeons::domain::run::{simulate_run, RunConfig, RunOutcome};

#[test]
fn seeded_run_reaches_same_result_every_time() {
    let hero = HeroProfile::default();
    let config = RunConfig {
        seed: 7,
        max_depth: 25,
    };

    let first = simulate_run(&hero, config);
    let second = simulate_run(&hero, config);

    assert_eq!(first, second);
}

#[test]
fn run_summary_reports_depth_gold_and_outcome() {
    let hero = HeroProfile::default();
    let result = simulate_run(
        &hero,
        RunConfig {
            seed: 5,
            max_depth: 25,
        },
    );

    assert!(result.deepest_depth >= 1);
    assert!(result.gold_earned > 0);
    assert!(matches!(
        result.outcome,
        RunOutcome::HeroDied | RunOutcome::BossDefeated
    ));
}
