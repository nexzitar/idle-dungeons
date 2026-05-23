//! Shared party hero column for build, summary, and running camp screens.

use bevy::prelude::*;

use crate::domain::hero::HeroProfile;
use crate::domain::party::PartyHeroKind;
use crate::domain::progression::PARTY_SLOT_2_UNLOCK_DEPTH;
use crate::ui::assets::UiPlaceholderImages;
use crate::ui::build_panel::hero_layering_warnings;
use crate::ui::primitives::hero_card::{spawn_hero_identity_card, HeroIdentityConfig};
use crate::ui::primitives::loadout::{spawn_loadout_row, slots_from_hero, LoadoutRowConfig};
use crate::ui::primitives::panel::{spawn_mounted_panel, MountedPanelConfig};
use crate::ui::primitives::section::spawn_framed_section_header;
use crate::ui::primitives::skill_bar::SkillBarInteraction;
use crate::ui::theme::{caption_text, UiDensity, UiTheme};

use super::layout::spawn_column_flex_scroll;

#[derive(Clone, Copy, Debug)]
pub struct HeroColumnConfig {
    pub party_slots_unlocked: usize,
    pub skill_slots_interactive: bool,
    pub allow_rename: bool,
}

pub fn spawn_hero_column(
    parent: &mut ChildSpawnerCommands<'_>,
    ph: &UiPlaceholderImages,
    lead: &HeroProfile,
    partner: Option<&HeroProfile>,
    config: HeroColumnConfig,
) {
    let density = UiDensity::Camp;
    spawn_mounted_panel(
        parent,
        MountedPanelConfig {
            style: crate::ui::theme::MountedPanelStyle::Deep,
            width: Val::Percent(100.0),
            flex_grow: 1.0,
            flex_shrink: 1.0,
            min_height: Val::Px(0.0),
        },
        |panel| {
            spawn_framed_section_header(panel, "PARTY");
            spawn_column_flex_scroll(panel, None, |body| {
                spawn_hero_block(
                    body,
                    ph,
                    lead,
                    PartyHeroKind::Player1,
                    0,
                    config,
                    density,
                );

                if config.party_slots_unlocked >= 2 {
                    if let Some(phero) = partner {
                        spawn_hero_block(
                            body,
                            ph,
                            phero,
                            PartyHeroKind::Player2,
                            1,
                            config,
                            density,
                        );
                    } else {
                        body.spawn(caption_text(
                            "Companion will appear after rewards sync (new unlock).",
                        ));
                    }
                } else {
                    body.spawn(caption_text(format!(
                        "Reach depth {} on a run to unlock a second party hero.",
                        PARTY_SLOT_2_UNLOCK_DEPTH
                    )));
                }
            });
        },
    );
}

fn spawn_hero_block(
    parent: &mut ChildSpawnerCommands<'_>,
    ph: &UiPlaceholderImages,
    hero: &HeroProfile,
    kind: PartyHeroKind,
    slot: u8,
    config: HeroColumnConfig,
    density: UiDensity,
) {
    spawn_hero_identity_card(
        parent,
        HeroIdentityConfig {
            slot,
            hero,
            kind,
            allow_rename: config.allow_rename,
            show_stat_strip: true,
            density,
        },
        ph,
    );
    for msg in hero_layering_warnings(hero) {
        parent.spawn((
            Text::new(format!("{}: {msg}", hero.name)),
            TextFont::from_font_size(UiTheme::FONT_CAPTION),
            TextColor(UiTheme::muted_gold()),
        ));
    }
    spawn_framed_section_header(parent, "SKILLS");
    if config.skill_slots_interactive {
        let hint = match kind {
            PartyHeroKind::Player1 => "Click a slot to open Party Buildcraft.",
            PartyHeroKind::Player2 => {
                "Player 2 has their own skills — click a slot to change them."
            }
        };
        parent.spawn(caption_text(hint));
    }
    let slots = slots_from_hero(hero);
    spawn_loadout_row(
        parent,
        LoadoutRowConfig {
            hero_kind: kind,
            label: None,
            slots: &slots,
            unlocked: hero.unlocked_skill_slots,
            focused_index: None,
            interaction: if config.skill_slots_interactive {
                SkillBarInteraction::OpenSkillBook(kind)
            } else {
                SkillBarInteraction::None
            },
            density,
        },
        ph,
    );
}
