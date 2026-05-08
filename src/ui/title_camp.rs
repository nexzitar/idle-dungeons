//! Title screen: campfire hub preview with lightweight animation and progression silhouettes.

use bevy::prelude::*;
use bevy::text::{TextColor, TextFont};

use crate::app::ProfileState;
use crate::domain::progression::PARTY_SLOT_2_UNLOCK_DEPTH;
use crate::domain::progression::MetaProgression;
use crate::ui::components::{
    CampfireFlame, TitleCampFigureSlot, TitleCampMilestoneExtras, TitleCampSceneRoot,
    TitleEnterCampButton, TitleQuitButton, TitleScreen, UiRoot,
};
use crate::ui::mockup_layout::spawn_mockup_header;
use crate::ui::placeholder_graphics::UiPlaceholderImages;
use crate::ui::theme::{body_text, caption_text, section_title, UiTheme};
use crate::ui::widgets::spawn_atmosphere;

/// Max party figures around the fire (matches current party slot design).
const CAMP_FIGURE_SLOTS: usize = 2;

pub fn spawn_title_screen(
    commands: &mut Commands,
    profile: &ProfileState,
    speed_mult: f32,
    ph: &UiPlaceholderImages,
) {
    let meta = &profile.profile.meta;
    let lead = profile.effective_hero();

    let root_entity = commands.spawn((root_shell(), UiRoot, TitleScreen)).id();
    commands.entity(root_entity).with_children(|root| {
        spawn_atmosphere(root);
        root.spawn(content_column_bundle()).with_children(|col| {
            spawn_mockup_header(
                col,
                ph,
                meta.gold,
                meta.salvage,
                meta.unlocked_skill_slots,
                lead.equipped_skills.len(),
                "—",
                speed_mult,
            );

            col.spawn((
            Node {
                box_sizing: BoxSizing::BorderBox,
                width: Val::Percent(100.0),
                    flex_grow: 1.0,
                    min_height: Val::Px(320.0),
                    flex_direction: FlexDirection::Row,
                    column_gap: Val::Px(18.0),
                    align_items: AlignItems::Stretch,
                    padding: UiRect::vertical(Val::Px(8.0)),
                    ..default()
            }
        ))
            .with_children(|row| {
                spawn_title_nav_column(row);
                spawn_camp_scene(row, meta, ph);
            });

            col.spawn((
            Node {
                box_sizing: BoxSizing::BorderBox,
                flex_direction: FlexDirection::Row,
                    justify_content: JustifyContent::SpaceBetween,
                    width: Val::Percent(100.0),
                    ..default()
            }
        ))
            .with_children(|foot| {
                foot.spawn(caption_text(format!(
                    "Next unlock · party slot 2 at depth {}",
                    PARTY_SLOT_2_UNLOCK_DEPTH
                )));
                foot.spawn(caption_text(format!(
                    "Best depth reached · {}",
                    meta.deepest_floor_reached
                )));
            });

            col.spawn(caption_text(format!(
                "v{}",
                env!("CARGO_PKG_VERSION")
            )));
        });
    });
}

fn root_shell() -> impl Bundle {
    (
        Node {
            box_sizing: BoxSizing::BorderBox,
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            position_type: PositionType::Relative,
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Stretch,
            ..default()
        },
        BackgroundColor(Color::NONE),
    )
}

fn content_column_bundle() -> impl Bundle {
    (
        Node {
            box_sizing: BoxSizing::BorderBox,
            width: Val::Percent(100.0),
            flex_grow: 1.0,
            min_height: Val::Px(0.0),
            flex_direction: FlexDirection::Column,
            padding: UiRect::axes(Val::Px(18.0), Val::Px(14.0)),
            row_gap: Val::Px(10.0),
            align_items: AlignItems::Stretch,
            ..default()
        },
    )
}

fn spawn_title_nav_column(parent: &mut ChildSpawnerCommands<'_>) {
    let pal_idle = UiTheme::panel_bg_deep();
    let pal_border = UiTheme::ornate_gold();
    parent
        .spawn((
            Node {
                box_sizing: BoxSizing::BorderBox,
                width: Val::Px(200.0),
                flex_shrink: 0.0,
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(10.0),
                padding: UiRect::all(Val::Px(12.0)),
                border: UiRect::all(Val::Px(2.0)),
                ..default()
            },
            BackgroundColor(UiTheme::panel_bg().into()),
            BorderColor::from(pal_border)
        ))
        .with_children(|col| {
            col.spawn(section_title("Delvers"));
            col.spawn(caption_text(
                "Prepare. Delve. Survive. The depths remember.",
            ));
            title_menu_button(
                col,
                "Enter camp",
                TitleEnterCampButton,
                pal_idle,
                pal_border,
            );
            title_menu_button(
                col,
                "Party",
                TitleEnterCampButton,
                pal_idle,
                pal_border,
            );
            title_menu_button(
                col,
                "Heroes",
                TitleEnterCampButton,
                pal_idle,
                pal_border,
            );
            title_codex_placeholder(col);
            crate::ui::mockup_layout::title_settings_menu_button(col);
            title_menu_button(
                col,
                "Quit",
                TitleQuitButton,
                pal_idle,
                pal_border,
            );
        });
}

fn title_menu_button(
    parent: &mut ChildSpawnerCommands<'_>,
    label: &str,
    marker: impl Component,
    bg: Color,
    border: Color,
) {
    parent
        .spawn((
            Node {
                box_sizing: BoxSizing::BorderBox,
                width: Val::Percent(100.0),
                    min_height: Val::Px(40.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    border: UiRect::all(Val::Px(1.0)),
                    ..default()
            },
            Button,
            BackgroundColor(bg.into()),
            BorderColor::from(border),
            marker,
        ))
        .with_children(|b| {
            b.spawn((
                Text::new(label),
                TextFont::from_font_size(UiTheme::FONT_BODY),
                TextColor(Color::WHITE),
            ));
        });
}

fn title_codex_placeholder(parent: &mut ChildSpawnerCommands<'_>) {
    parent
        .spawn((
            Node {
                box_sizing: BoxSizing::BorderBox,
                width: Val::Percent(100.0),
                min_height: Val::Px(38.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                border: UiRect::all(Val::Px(1.0)),
                ..default()
            },
            BackgroundColor(UiTheme::stone_deep().into()),
            BorderColor::from(UiTheme::body_dim())
        ))
        .with_children(|b| {
            b.spawn((
                Text::new("Codex (soon)"),
                TextFont::from_font_size(UiTheme::FONT_BODY),
                TextColor(UiTheme::body_dim()),
            ));
        });
}

fn spawn_camp_scene(parent: &mut ChildSpawnerCommands<'_>, meta: &MetaProgression, ph: &UiPlaceholderImages) {
    parent
        .spawn((
            Node {
                box_sizing: BoxSizing::BorderBox,
                flex_grow: 1.0,
                    min_width: Val::Px(0.0),
                    min_height: Val::Px(280.0),
                    position_type: PositionType::Relative,
                    padding: UiRect::all(Val::Px(14.0)),
                    border: UiRect::all(Val::Px(2.0)),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::FlexEnd,
                    ..default()
            },
            BackgroundColor(Color::srgba(0.04, 0.04, 0.07, 1.0).into()),
            BorderColor::from(UiTheme::ornate_gold()),
            TitleCampSceneRoot,
        ))
        .with_children(|scene| {
            scene.spawn((
                Node {
                    box_sizing: BoxSizing::BorderBox,
                    position_type: PositionType::Absolute,
                    left: Val::Px(0.0),
                    top: Val::Px(0.0),
                    right: Val::Px(0.0),
                    bottom: Val::Px(0.0),
                    ..default()
                },
                ImageNode {
                    image: ph.dungeon_theater.clone(),
                    color: Color::srgba(0.35, 0.32, 0.45, 0.35),
                    ..default()
                },
            ));

            scene
                .spawn((
            Node {
                box_sizing: BoxSizing::BorderBox,
                position_type: PositionType::Absolute,
                        left: Val::Percent(8.0),
                        bottom: Val::Percent(12.0),
                        flex_direction: FlexDirection::Column,
                        row_gap: Val::Px(6.0),
                        max_width: Val::Percent(55.0),
                        ..default()
            }
        ))
                .with_children(|titles| {
                    titles.spawn((
                Text::new("DELVERS"),
                TextFont::from_font_size(UiTheme::FONT_DISPLAY_HERO),
                TextColor(UiTheme::muted_gold()),
            ));
                    titles.spawn(body_text(
                        "Your expedition gathers at the fire. Each successful run brings new faces, gear, and stories.",
                    ));
                });

            scene
                .spawn((
            Node {
                box_sizing: BoxSizing::BorderBox,
                width: Val::Percent(100.0),
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        row_gap: Val::Px(14.0),
                        padding: UiRect::bottom(Val::Px(18.0)),
                        ..default()
            }
        ))
                .with_children(|camp| {
                    let tent_vis = if meta.deepest_floor_reached >= 15 {
                        Visibility::Visible
                    } else {
                        Visibility::Hidden
                    };
                    camp.spawn((
                        Node {
                box_sizing: BoxSizing::BorderBox,
                align_self: AlignSelf::FlexEnd,
                                margin: UiRect::right(Val::Px(24.0)),
                                padding: UiRect::axes(Val::Px(14.0), Val::Px(6.0)),
                                border: UiRect::all(Val::Px(1.0)),
                                ..default()
            },
            BackgroundColor(Color::srgba(0.12, 0.1, 0.08, 0.92).into()),
            BorderColor::from(UiTheme::panel_border()),
            tent_vis,
                        TitleCampMilestoneExtras,
                    ))
                    .with_children(|t| {
                        t.spawn(caption_text("Camp · supply tent"));
                    });

                    camp.spawn((
            Node {
                box_sizing: BoxSizing::BorderBox,
                flex_direction: FlexDirection::Row,
                            align_items: AlignItems::FlexEnd,
                            justify_content: JustifyContent::Center,
                            column_gap: Val::Px(28.0),
                            ..default()
            }
        ))
                    .with_children(|figures| {
                        for i in 0..CAMP_FIGURE_SLOTS {
                            let vis = if i == 0 {
                                Visibility::Visible
                            } else if meta.party_slots_unlocked() >= 2 {
                                Visibility::Visible
                            } else {
                                Visibility::Hidden
                            };
                            figures
                                .spawn((
                                    Node {
                box_sizing: BoxSizing::BorderBox,
                flex_direction: FlexDirection::Column,
                                            align_items: AlignItems::Center,
                                            row_gap: Val::Px(6.0),
                                            ..default()
            },
            vis,
                                    TitleCampFigureSlot(i as u8),
                                ))
                                .with_children(|fig| {
                                    fig.spawn((
                                        Text::new(if i == 0 { "\u{1F9DD}" } else { "\u{2694}" }),
                                        TextFont::from_font_size(42.0),
                                        TextColor(Color::srgba(0.95, 0.82, 0.6, 0.88)),
                                    ));
                                    fig.spawn(caption_text(if i == 0 {
                                        "Lead"
                                    } else {
                                        "Ally"
                                    }));
                                });
                        }
                    });

                    camp.spawn((
            Node {
                box_sizing: BoxSizing::BorderBox,
                flex_direction: FlexDirection::Column,
                            align_items: AlignItems::Center,
                            row_gap: Val::Px(4.0),
                            ..default()
            }
        ))
                    .with_children(|fire_zone| {
                        fire_zone.spawn((
                            Node {
                box_sizing: BoxSizing::BorderBox,
                width: Val::Px(120.0),
                                    height: Val::Px(28.0),
                                    ..default()
            },
            BackgroundColor(Color::srgba(0.95, 0.45, 0.22, 0.45).into()),
                            CampfireFlame {
                                base: Color::srgba(0.75, 0.28, 0.08, 0.55),
                                peak: Color::srgba(1.0, 0.72, 0.18, 0.78),
                                speed: 5.0,
                                phase_offset: 0.0,
                            },
                        ));
                        fire_zone.spawn((
                            Node {
                box_sizing: BoxSizing::BorderBox,
                width: Val::Px(72.0),
                                    height: Val::Px(48.0),
                                    margin: UiRect::top(Val::Px(-14.0)),
                                    ..default()
            },
            BackgroundColor(Color::srgba(0.98, 0.55, 0.12, 0.85).into()),
                            CampfireFlame {
                                base: Color::srgba(0.92, 0.4, 0.05, 0.82),
                                peak: Color::srgba(1.0, 0.88, 0.35, 0.95),
                                speed: 7.0,
                                phase_offset: 1.7,
                            },
                        ));
                        fire_zone.spawn(caption_text("The fire never lies."));
                    });
                });
        });
}
