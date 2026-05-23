use bevy::prelude::*;

use crate::app::{LatestRunSummary, ProfileState, RunSpeedSetting};
use crate::domain::run::RunSummary;
use crate::ui::assets::UiPlaceholderImages;
use crate::ui::components::{SummaryScreen, UiRoot};
use crate::ui::primitives::spawn_atmosphere;
use crate::ui::screens::{content_column_bundle, root_shell};
use crate::ui::shell;

pub(crate) fn spawn_summary_screen(
    mut commands: Commands,
    profile: Res<ProfileState>,
    latest_summary: Option<Res<LatestRunSummary>>,
    speed: Res<RunSpeedSetting>,
    ph: Res<UiPlaceholderImages>,
) {
    let summary = latest_summary
        .as_deref()
        .map(|s| s.summary.clone())
        .unwrap_or_else(crate::ui::summary_panel::empty_run_summary);
    spawn_summary_screen_root(&mut commands, &profile, &summary, speed.multiplier(), &ph);
}

pub(crate) fn spawn_summary_screen_root(
    commands: &mut Commands,
    profile: &ProfileState,
    summary: &RunSummary,
    speed_mult: f32,
    ph: &UiPlaceholderImages,
) -> Entity {
    let meta = &profile.profile.meta;
    let lead = profile.effective_hero();
    let partner = profile.effective_party_partner();
    let party_slots = profile.profile.meta.party_slots_unlocked();

    let root_entity = commands.spawn((root_shell(), UiRoot, SummaryScreen)).id();
    commands.entity(root_entity).with_children(|root| {
        spawn_atmosphere(root);
        root.spawn(content_column_bundle()).with_children(|col| {
            shell::spawn_mockup_header(
                col,
                ph,
                meta.gold,
                meta.salvage,
                meta.unlocked_skill_slots,
                lead.equipped_skills.len(),
                &summary.deepest_depth.to_string(),
                speed_mult,
            );
            shell::spawn_three_column_row(col, |row| {
                shell::spawn_ornate_column(row, 1.0, |panel| {
                    shell::spawn_hero_column(
                        panel,
                        ph,
                        &lead,
                        partner.as_ref(),
                        shell::HeroColumnConfig {
                            party_slots_unlocked: party_slots,
                            skill_slots_interactive: false,
                            allow_rename: false,
                        },
                    );
                });
                shell::spawn_ornate_column(row, 1.25, |panel| {
                    shell::spawn_dungeon_summary_column(panel, summary);
                });
            });
            shell::spawn_mockup_footer(col, shell::FooterMode::Summary);
        });
        shell::spawn_summary_rewards_modal(root, summary, profile.profile.stash_sort, ph);
        crate::ui::tooltip::spawn_tooltip_layer(root);
    });
    root_entity
}
