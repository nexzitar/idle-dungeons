//! Left column: all party hero loadout rows (always visible).

use bevy::prelude::*;

use crate::domain::party::PartyHeroKind;
use crate::ui::assets::UiPlaceholderImages;
use crate::ui::buildcraft::session::BuildcraftEditSession;
use crate::ui::primitives::skill_bar::{spawn_skill_bar, SkillBarConfig};
use crate::ui::theme::{section_title, UiTheme};

#[derive(Component)]
pub struct BuildcraftPartyColumn;

pub fn spawn_party_column(
    parent: &mut ChildSpawnerCommands<'_>,
    session: &BuildcraftEditSession,
    ph: &UiPlaceholderImages,
) {
    parent
        .spawn((
            Node {
                box_sizing: BoxSizing::BorderBox,
                width: Val::Percent(38.0),
                flex_shrink: 0.0,
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(14.0),
                padding: UiRect::all(Val::Px(UiTheme::PANEL_INSET)),
                border: UiRect::all(Val::Px(1.0)),
                ..default()
            },
            BackgroundColor(UiTheme::panel_bg()),
            BorderColor::from(UiTheme::ornate_gold()),
            BuildcraftPartyColumn,
        ))
        .with_children(|col| {
            col.spawn(section_title("PARTY LOADOUTS"));
            spawn_hero_row(col, &session.lead, session.focused, ph);
            if let Some(partner) = &session.partner {
                spawn_hero_row(col, partner, session.focused, ph);
            }
            col.spawn((
                Text::new("Slot order 1→6 sets combat priority when multiple skills are ready."),
                TextFont::from_font_size(UiTheme::FONT_CAPTION),
                TextColor(UiTheme::body_dim()),
            ));
        });
}

fn spawn_hero_row(
    parent: &mut ChildSpawnerCommands<'_>,
    edit: &crate::ui::buildcraft::session::HeroLoadoutEdit,
    focused: (PartyHeroKind, usize),
    ph: &UiPlaceholderImages,
) {
    parent
        .spawn((
            Node {
                box_sizing: BoxSizing::BorderBox,
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(6.0),
                ..default()
            },
        ))
        .with_children(|block| {
            block.spawn((
                Text::new(format!("{} — {}", edit.hero.label(), edit.display_name)),
                TextFont::from_font_size(UiTheme::FONT_SECTION),
                TextColor(UiTheme::muted_cream()),
            ));
            let focused_index = (focused.0 == edit.hero).then_some(focused.1);
            spawn_skill_bar(
                block,
                SkillBarConfig {
                    hero: edit.hero,
                    slots: &edit.pending,
                    unlocked: edit.unlocked,
                    focused_index,
                    interactive: true,
                    cell_px: 52.0,
                },
                ph,
            );
        });
}
