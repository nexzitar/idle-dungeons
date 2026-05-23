//! Hero identity card — emotional anchor for party UI (portrait, name, role, vitals strip).

use bevy::prelude::*;

use crate::domain::combat_archetype::{hints_for_hero, CombatArchetypeHint};
use crate::domain::hero::HeroProfile;
use crate::domain::party::PartyHeroKind;
use crate::ui::assets::UiPlaceholderImages;
use crate::ui::components::{HeroNameDisplayText, HeroNameEditButton, UiButtonPalette};
use crate::ui::inspect::InspectHint;
use crate::ui::interaction::UiClickAction;
use crate::ui::primitives::panel::{spawn_mounted_panel, MountedPanelConfig};
use crate::ui::theme::{MountedPanelStyle, UiDensity, UiTheme};

const PORTRAIT_PX: f32 = 72.0;

/// Marker on the card root — `slot` matches rename / name sync (`0` = lead, `1` = partner).
#[derive(Component, Clone, Copy, Debug)]
pub struct HeroIdentityCard {
    pub slot: u8,
}

#[derive(Clone, Copy, Debug)]
pub struct HeroIdentityConfig<'a> {
    pub slot: u8,
    pub hero: &'a HeroProfile,
    pub kind: PartyHeroKind,
    pub allow_rename: bool,
    pub show_stat_strip: bool,
    pub density: UiDensity,
}

pub fn hero_identity_subtitle(hero: &HeroProfile, kind: PartyHeroKind) -> String {
    let role = hints_for_hero(hero)
        .first()
        .map(|hint| archetype_hint_label(*hint))
        .unwrap_or("Adventurer");
    format!("{} · {role}", kind.label())
}

pub fn spawn_hero_identity_card(
    parent: &mut ChildSpawnerCommands<'_>,
    config: HeroIdentityConfig<'_>,
    ph: &UiPlaceholderImages,
) {
    let subtitle = hero_identity_subtitle(config.hero, config.kind);
    let stats = config.hero.derived_stats();
    let warm_accent = UiTheme::torch_glow().mix(&UiTheme::muted_gold(), 0.35);

    let card = spawn_mounted_panel(
        parent,
        MountedPanelConfig {
            style: MountedPanelStyle::Deep,
            width: Val::Percent(100.0),
            flex_grow: 0.0,
            flex_shrink: 0.0,
            min_height: Val::Px(0.0),
        },
        |panel| {
            panel
                .spawn((
                    Node {
                        box_sizing: BoxSizing::BorderBox,
                        width: Val::Percent(100.0),
                        flex_direction: FlexDirection::Row,
                        column_gap: Val::Px(12.0),
                        align_items: AlignItems::FlexStart,
                        ..default()
                    },
                ))
                .with_children(|head| {
                    head.spawn((
                        Node {
                            box_sizing: BoxSizing::BorderBox,
                            width: Val::Px(4.0),
                            align_self: AlignSelf::Stretch,
                            min_height: Val::Px(PORTRAIT_PX),
                            ..default()
                        },
                        BackgroundColor(warm_accent),
                    ));
                    head.spawn((
                        Node {
                            box_sizing: BoxSizing::BorderBox,
                            width: Val::Px(PORTRAIT_PX),
                            height: Val::Px(PORTRAIT_PX),
                            flex_shrink: 0.0,
                            padding: UiRect::all(Val::Px(3.0)),
                            border: UiRect::all(Val::Px(1.0)),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        BackgroundColor(UiTheme::stone_deep()),
                        BorderColor::from(UiTheme::panel_border_inner()),
                    ))
                    .with_children(|frame| {
                        frame.spawn((
                            Node {
                                box_sizing: BoxSizing::BorderBox,
                                width: Val::Percent(100.0),
                                height: Val::Percent(100.0),
                                ..default()
                            },
                            ImageNode {
                                image: ph.hero_portrait.clone(),
                                color: Color::srgba(0.92, 0.86, 0.78, 0.95),
                                ..default()
                            },
                        ));
                    });
                    head.spawn((
                        Node {
                            box_sizing: BoxSizing::BorderBox,
                            flex_direction: FlexDirection::Column,
                            row_gap: Val::Px(4.0),
                            flex_grow: 1.0,
                            min_width: Val::Px(0.0),
                            ..default()
                        },
                    ))
                    .with_children(|txt| {
                        txt.spawn((
                            Text::new(""),
                            TextFont::from_font_size(UiTheme::FONT_SECTION),
                            TextColor(UiTheme::muted_cream()),
                            HeroNameDisplayText {
                                slot: config.slot,
                            },
                        ));
                        txt.spawn((
                            Text::new(subtitle),
                            TextFont::from_font_size(UiTheme::FONT_CAPTION),
                            TextColor(UiTheme::body_dim()),
                        ));
                    });
                    if config.allow_rename {
                        let p = UiButtonPalette::panel_outlined();
                        head.spawn((
                            Node {
                                box_sizing: BoxSizing::BorderBox,
                                min_width: Val::Px(72.0),
                                min_height: Val::Px(30.0),
                                flex_shrink: 0.0,
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                padding: UiRect::horizontal(Val::Px(6.0)),
                                border: UiRect::all(Val::Px(1.0)),
                                ..default()
                            },
                            Button,
                            BackgroundColor(p.idle_bg),
                            BorderColor::from(p.idle_border),
                            HeroNameEditButton {
                                slot: config.slot,
                            },
                            UiClickAction::HeroNameEdit,
                            p,
                            InspectHint("Rename this hero."),
                        ))
                        .with_children(|b| {
                            b.spawn((
                                Text::new("Rename"),
                                TextFont::from_font_size(UiTheme::FONT_LABEL),
                                TextColor(UiTheme::body()),
                            ));
                        });
                    }
                });
            if config.show_stat_strip {
                panel
                    .spawn((
                        Node {
                            box_sizing: BoxSizing::BorderBox,
                            width: Val::Percent(100.0),
                            flex_direction: FlexDirection::Row,
                            column_gap: Val::Px(8.0),
                            padding: UiRect::top(Val::Px(4.0)),
                            ..default()
                        },
                    ))
                    .with_children(|strip| {
                        spawn_stat_chip(strip, "HP", stats.max_health);
                        spawn_stat_chip(strip, "DMG", stats.damage);
                        spawn_stat_chip(strip, "ARM", stats.armor);
                    });
            }
        },
    );
    parent.commands_mut().entity(card).insert(HeroIdentityCard {
        slot: config.slot,
    });
}

fn spawn_stat_chip(parent: &mut ChildSpawnerCommands<'_>, label: &'static str, value: i32) {
    parent
        .spawn((
            Node {
                box_sizing: BoxSizing::BorderBox,
                flex_grow: 1.0,
                flex_basis: Val::Px(0.0),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(2.0),
                padding: UiRect::all(Val::Px(6.0)),
                border: UiRect::all(Val::Px(1.0)),
                ..default()
            },
            BackgroundColor(UiTheme::panel_bg()),
            BorderColor::from(UiTheme::panel_border_inner()),
        ))
        .with_children(|chip| {
            chip.spawn((
                Text::new(label),
                TextFont::from_font_size(UiTheme::FONT_MICRO),
                TextColor(UiTheme::body_dim()),
            ));
            chip.spawn((
                Text::new(value.to_string()),
                TextFont::from_font_size(UiTheme::FONT_COMPACT),
                TextColor(UiTheme::muted_cream()),
            ));
        });
}

fn archetype_hint_label(hint: CombatArchetypeHint) -> &'static str {
    match hint {
        CombatArchetypeHint::ThreatAnchor => "Threat anchor",
        CombatArchetypeHint::PoisonRamping => "Poison ramp",
        CombatArchetypeHint::HeavyWeave => "Heavy weave",
        CombatArchetypeHint::CleaveAoE => "Cleave",
        CombatArchetypeHint::InstantStrike => "Instant strike",
        CombatArchetypeHint::ReactiveThorns => "Reactive",
        CombatArchetypeHint::SustainStrike => "Sustain",
        CombatArchetypeHint::BurstStrike => "Burst",
        CombatArchetypeHint::WardFocused => "Ward focus",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::party::default_party_partner_hero;

    #[test]
    fn partner_subtitle_includes_player_label() {
        let p = default_party_partner_hero();
        let sub = hero_identity_subtitle(&p, PartyHeroKind::Player2);
        assert!(sub.starts_with("Player 2"));
        assert!(sub.contains("Threat anchor"));
    }
}
