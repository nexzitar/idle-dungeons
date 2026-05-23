use bevy::prelude::*;

use crate::app::{ProfileState, RunSpeedSetting};
use crate::ui::assets::UiPlaceholderImages;
use crate::ui::components::{BuildScreen, UiRoot};
use crate::ui::primitives::spawn_atmosphere;
use crate::ui::screens::{content_column_bundle, root_shell};
use crate::ui::shell;

pub(crate) fn spawn_build_screen(
    mut commands: Commands,
    profile: Res<ProfileState>,
    speed: Res<RunSpeedSetting>,
    ph: Res<UiPlaceholderImages>,
) {
    spawn_build_screen_root(&mut commands, &profile, speed.multiplier(), &ph);
}

pub(crate) fn spawn_build_screen_root(
    commands: &mut Commands,
    profile: &ProfileState,
    speed_mult: f32,
    ph: &UiPlaceholderImages,
) -> Entity {
    let lead = profile.effective_hero();
    let partner = profile.effective_party_partner();
    let party_slots = profile.profile.meta.party_slots_unlocked();
    let meta = &profile.profile.meta;

    let stash = profile.profile.inventory.len();

    let root_entity = commands.spawn((root_shell(), UiRoot, BuildScreen)).id();
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
                "—",
                speed_mult,
            );
            shell::spawn_three_column_row(col, |row| {
                shell::spawn_ornate_column(row, 1.0, |panel| {
                    shell::spawn_hero_column_mockup(
                        panel,
                        ph,
                        &lead,
                        partner.as_ref(),
                        party_slots,
                        true,
                        true,
                    );
                });
                shell::spawn_ornate_column(row, 1.25, |panel| {
                    shell::spawn_dungeon_briefing_column(panel, stash, meta);
                });
            });
            shell::spawn_mockup_footer(col, shell::FooterMode::Briefing);
        });
        crate::ui::tooltip::spawn_tooltip_layer(root);
    });
    root_entity
}
