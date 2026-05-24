//! Left column: all party hero loadout rows (always visible).

use bevy::prelude::*;

use crate::domain::party::PartyHeroKind;
use crate::ui::assets::UiPlaceholderImages;
use crate::ui::buildcraft::session::BuildcraftEditSession;
use crate::ui::primitives::loadout::{spawn_loadout_row, LoadoutRowConfig};
use crate::ui::primitives::panel::{spawn_mounted_panel, MountedPanelConfig};
use crate::ui::primitives::section::spawn_framed_section_header;
use crate::ui::primitives::skill_bar::SkillBarInteraction;
use crate::ui::theme::{UiDensity, UiTheme};

#[derive(Component)]
pub struct BuildcraftPartyColumn;

pub fn spawn_party_column(
    parent: &mut ChildSpawnerCommands<'_>,
    session: &BuildcraftEditSession,
    ph: &UiPlaceholderImages,
) {
    let density = UiDensity::Buildcraft;
    let panel = spawn_mounted_panel(
        parent,
        MountedPanelConfig::ornate_column(Val::Percent(38.0)),
        |col| {
            spawn_framed_section_header(col, "PARTY LOADOUTS");
            spawn_buildcraft_row(col, &session.lead, session.focused, ph, density);
            if let Some(partner) = &session.partner {
                spawn_buildcraft_row(col, partner, session.focused, ph, density);
            }
            col.spawn((
                Text::new("Slot order 1→6 sets combat priority when multiple skills are ready."),
                TextFont::from_font_size(UiTheme::FONT_CAPTION),
                TextColor(UiTheme::body_dim()),
            ));
        },
    );
    parent.commands_mut().entity(panel).insert(BuildcraftPartyColumn);
}

fn spawn_buildcraft_row(
    parent: &mut ChildSpawnerCommands<'_>,
    edit: &crate::ui::buildcraft::session::HeroLoadoutEdit,
    focused: (PartyHeroKind, usize),
    ph: &UiPlaceholderImages,
    density: UiDensity,
) {
    let label = format!("{} — {}", edit.hero.label(), edit.display_name);
    let focused_index = (focused.0 == edit.hero).then_some(focused.1);
    spawn_loadout_row(
        parent,
        LoadoutRowConfig {
            hero_kind: edit.hero,
            label: Some(&label),
            slots: &edit.pending,
            unlocked: edit.unlocked,
            focused_index,
            interaction: SkillBarInteraction::BuildcraftFocus,
            density,
        },
        ph,
    );
}
