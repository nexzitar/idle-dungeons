//! Title screen: campfire hub using **`assets/ui/campfire_scene.png`**.
//!
//! - **Fire:** `Fireplace.png` (single still) on the stone ring.
//! - **Players:** six physical seats (`player1` … `player6`); unlocked roster members are assigned to
//!   distinct seats at random each time the title screen spawns (`assign_players_to_camp_seats`).

use bevy::prelude::*;
use bevy::text::{TextColor, TextFont};
use bevy::ui::{GlobalZIndex, RelativeCursorPosition, UiTransform, Val2};

use crate::app::ProfileState;
use crate::domain::progression::MetaProgression;
use crate::domain::progression::PARTY_SLOT_2_UNLOCK_DEPTH;
use crate::domain::title_camp::{assign_players_to_camp_seats, CAMP_FIRE_SEATS};
use crate::presentation::editor::TITLE_ELEMENT_FIREPLACE;
use crate::presentation::layer::title_player_element_id;
use crate::presentation::markers::PresentationLayerHost;
use crate::presentation::{compose_layer_id, spawn_title_fire_layers};
use crate::ui::components::{
    PresentationElementHost, TitleCampFigureEmoji, TitleCampFigureSlot, TitleCampFigureTuneMarker,
    TitleCampMilestoneExtras, TitleCampSceneRoot, TitleCampStageRoot, TitleCampfireTuneMarker,
    TitleEnterCampButton, TitleQuitButton, TitleScreen, UiRoot,
};
use crate::ui::shell::spawn_mockup_header;
use crate::ui::assets::UiPlaceholderImages;
use crate::ui::scene_tune::{
    title_fireplace_base_px, tune_to_figure_emoji_color, TitleSceneLayout,
};
use crate::ui::theme::{body_text, caption_text, section_title, UiTheme};
use crate::ui::tooltip;
use crate::ui::interaction::UiClickAction;
use crate::ui::primitives::spawn_atmosphere;

fn camp_seat_assignment_seed(meta: &MetaProgression) -> u64 {
    (meta.gold as u64)
        .wrapping_mul(0x9E37_79B9_7F4A_7C15)
        .wrapping_add(meta.salvage as u64)
        .wrapping_add(meta.deepest_floor_reached as u64)
        ^ 0xCAFE_5EA7_u64
}

fn player_camp_emoji(player_idx: u8) -> &'static str {
    match player_idx {
        0 => "\u{1F9DD}",
        1 => "\u{2694}",
        _ => "\u{1F464}",
    }
}

pub fn spawn_title_screen(
    commands: &mut Commands,
    profile: &ProfileState,
    speed_mult: f32,
    ph: &UiPlaceholderImages,
    layout: &TitleSceneLayout,
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

            col.spawn(Node {
                box_sizing: BoxSizing::BorderBox,
                width: Val::Percent(100.0),
                flex_grow: 1.0,
                min_height: Val::Px(320.0),
                flex_direction: FlexDirection::Row,
                column_gap: Val::Px(18.0),
                align_items: AlignItems::Stretch,
                padding: UiRect::vertical(Val::Px(8.0)),
                ..default()
            })
            .with_children(|row| {
                spawn_title_nav_column(row);
                spawn_camp_scene(row, meta, ph, layout);
            });

            col.spawn(Node {
                box_sizing: BoxSizing::BorderBox,
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::SpaceBetween,
                width: Val::Percent(100.0),
                ..default()
            })
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

            col.spawn(caption_text(format!("v{}", env!("CARGO_PKG_VERSION"))));
        });
        tooltip::spawn_tooltip_layer(root);
        #[cfg(debug_assertions)]
        {
            use crate::presentation::editor::spawn_presentation_editor_overlay;
            spawn_presentation_editor_overlay(root);
        }
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
    (Node {
        box_sizing: BoxSizing::BorderBox,
        width: Val::Percent(100.0),
        flex_grow: 1.0,
        min_height: Val::Px(0.0),
        flex_direction: FlexDirection::Column,
        padding: UiRect::axes(Val::Px(18.0), Val::Px(14.0)),
        row_gap: Val::Px(10.0),
        align_items: AlignItems::Stretch,
        ..default()
    },)
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
            BorderColor::from(pal_border),
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
                UiClickAction::TitleEnterCamp,
                pal_idle,
                pal_border,
            );
            title_menu_button(
                col,
                "Party",
                TitleEnterCampButton,
                UiClickAction::TitleEnterCamp,
                pal_idle,
                pal_border,
            );
            title_menu_button(
                col,
                "Heroes",
                TitleEnterCampButton,
                UiClickAction::TitleEnterCamp,
                pal_idle,
                pal_border,
            );
            title_codex_placeholder(col);
            crate::ui::shell::title_settings_menu_button(col);
            title_menu_button(
                col,
                "Quit",
                TitleQuitButton,
                UiClickAction::TitleQuit,
                pal_idle,
                pal_border,
            );
        });
}

fn title_menu_button(
    parent: &mut ChildSpawnerCommands<'_>,
    label: &str,
    marker: impl Component,
    action: UiClickAction,
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
            action,
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
            BorderColor::from(UiTheme::body_dim()),
        ))
        .with_children(|b| {
            b.spawn((
                Text::new("Codex (soon)"),
                TextFont::from_font_size(UiTheme::FONT_BODY),
                TextColor(UiTheme::body_dim()),
            ));
        });
}

fn spawn_camp_scene(
    parent: &mut ChildSpawnerCommands<'_>,
    meta: &MetaProgression,
    ph: &UiPlaceholderImages,
    layout: &TitleSceneLayout,
) {
    let (base_w, base_h) = title_fireplace_base_px(&layout.fireplace);

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
                    image: ph.campfire_scene.clone(),
                    color: Color::WHITE,
                    ..default()
                },
            ));

            scene
                .spawn(Node {
                    box_sizing: BoxSizing::BorderBox,
                    position_type: PositionType::Absolute,
                    left: Val::Percent(8.0),
                    bottom: Val::Percent(12.0),
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(6.0),
                    max_width: Val::Percent(55.0),
                    ..default()
                })
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
                    },
                    TitleCampStageRoot,
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

                    camp.spawn(Node {
                        box_sizing: BoxSizing::BorderBox,
                        flex_direction: FlexDirection::Row,
                        align_items: AlignItems::FlexEnd,
                        justify_content: JustifyContent::Center,
                        column_gap: Val::Px(28.0),
                        ..default()
                    })
                    .with_children(|figures| {
                        let seat_map = assign_players_to_camp_seats(
                            meta.party_slots_unlocked(),
                            camp_seat_assignment_seed(meta),
                        );
                        for seat in 0..CAMP_FIRE_SEATS {
                            let Some(player_idx) = seat_map[seat] else {
                                continue;
                            };
                            let tune = layout.player_tune(seat);
                            let element_id = title_player_element_id(seat + 1);
                            figures
                                .spawn((
                                    Node {
                                        box_sizing: BoxSizing::BorderBox,
                                        flex_direction: FlexDirection::Column,
                                        align_items: AlignItems::Center,
                                        row_gap: Val::Px(6.0),
                                        ..default()
                                    },
                                    TitleCampFigureSlot(seat as u8),
                                    TitleCampFigureTuneMarker(seat as u8),
                                    PresentationElementHost(element_id.to_string()),
                                    Interaction::default(),
                                    RelativeCursorPosition::default(),
                                    UiTransform {
                                        translation: Val2::ZERO,
                                        ..default()
                                    },
                                    GlobalZIndex(tune.global_z),
                                ))
                                .with_children(|fig| {
                                    fig.spawn((
                                        Node {
                                            box_sizing: BoxSizing::BorderBox,
                                            flex_direction: FlexDirection::Column,
                                            align_items: AlignItems::Center,
                                            ..default()
                                        },
                                        PresentationLayerHost(compose_layer_id(element_id, "emoji")),
                                        Interaction::default(),
                                        RelativeCursorPosition::default(),
                                        UiTransform::default(),
                                        GlobalZIndex(tune.global_z + 1),
                                    ))
                                    .with_children(|emoji| {
                                        emoji.spawn((
                                            Text::new(player_camp_emoji(player_idx)),
                                            TextFont::from_font_size(
                                                tune.size_basis.clamp(8.0, 160.0),
                                            ),
                                            TextColor(tune_to_figure_emoji_color(tune)),
                                            TitleCampFigureEmoji(seat as u8),
                                        ));
                                    });
                                    fig.spawn(caption_text(format!(
                                        "Player {}",
                                        player_idx as usize + 1
                                    )));
                                });
                        }
                    });

                    camp.spawn(Node {
                        box_sizing: BoxSizing::BorderBox,
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        row_gap: Val::Px(4.0),
                        ..default()
                    })
                    .with_children(|fire_zone| {
                        fire_zone
                            .spawn((
                                Node {
                                    box_sizing: BoxSizing::BorderBox,
                                    position_type: PositionType::Relative,
                                    align_items: AlignItems::Center,
                                    justify_content: JustifyContent::FlexEnd,
                                    ..default()
                                },
                                TitleCampfireTuneMarker,
                                PresentationElementHost(TITLE_ELEMENT_FIREPLACE.to_string()),
                                Interaction::default(),
                                RelativeCursorPosition::default(),
                                UiTransform {
                                    translation: Val2::ZERO,
                                    ..default()
                                },
                                GlobalZIndex(layout.fireplace.global_z),
                            ))
                            .with_children(|host| {
                                spawn_title_fire_layers(
                                    host,
                                    ph.fireplace.clone(),
                                    ph.fire_glow_radial.clone(),
                                    base_w,
                                    base_h,
                                    &layout.fire_presentation,
                                );
                            });
                        fire_zone.spawn(caption_text("The fire never lies."));
                    });
                });
        });
}
