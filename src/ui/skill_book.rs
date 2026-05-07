//! Skill book modal: pick a skill (or clear) for a loadout slot.

use bevy::prelude::*;
use bevy::ui::{FocusPolicy, RelativeCursorPosition};

use crate::domain::skills::{
    format_skill_tags, skill_book_pick_order, skill_definition, SkillId, SkillKind,
};
use crate::ui::components::{
    SkillBookBackdrop, SkillBookCloseButton, SkillBookPickButton, SkillBookRoot, UiButtonPalette,
    UiScrollContent, UiScrollRegion, UiScrollState, UiTooltip,
};
use crate::ui::placeholder_graphics::UiPlaceholderImages;
use crate::ui::theme::{caption_text, headline_text, section_title, UiTheme};

pub fn spawn_skill_book_modal(
    parent: &mut ChildBuilder,
    target_slot: usize,
    ph: &UiPlaceholderImages,
) {
    parent
        .spawn((
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    position_type: PositionType::Absolute,
                    left: Val::Px(0.0),
                    top: Val::Px(0.0),
                    ..default()
                },
                ..default()
            },
            SkillBookRoot,
        ))
        .insert(FocusPolicy::Block)
        .with_children(|layer| {
            let backdrop_pal = UiButtonPalette {
                idle_bg: Color::srgba(0.02, 0.02, 0.04, 0.58),
                hover_bg: Color::srgba(0.04, 0.04, 0.06, 0.65),
                pressed_bg: Color::srgba(0.06, 0.06, 0.08, 0.72),
                idle_border: Color::NONE,
                hover_border: Color::NONE,
                pressed_border: Color::NONE,
            };
            layer.spawn((
                ButtonBundle {
                    style: Style {
                        position_type: PositionType::Absolute,
                        width: Val::Percent(100.0),
                        height: Val::Percent(100.0),
                        left: Val::Px(0.0),
                        top: Val::Px(0.0),
                        ..default()
                    },
                    background_color: backdrop_pal.idle_bg.into(),
                    border_color: BorderColor(backdrop_pal.idle_border),
                    ..default()
                },
                SkillBookBackdrop,
                backdrop_pal,
                UiTooltip::txt("Click outside to close the skill book."),
            ));
            layer
                .spawn(NodeBundle {
                    style: Style {
                        position_type: PositionType::Absolute,
                        left: Val::Percent(50.0),
                        top: Val::Percent(45.0),
                        margin: UiRect {
                            left: Val::Px(-240.0),
                            top: Val::Px(-200.0),
                            right: Val::Auto,
                            bottom: Val::Auto,
                        },
                        width: Val::Px(480.0),
                        max_height: Val::Percent(88.0),
                        padding: UiRect::all(Val::Px(UiTheme::PAD_ROOT)),
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Stretch,
                        row_gap: Val::Px(UiTheme::PANEL_INSET),
                        border: UiRect::all(Val::Px(2.0)),
                        ..default()
                    },
                    background_color: UiTheme::panel_bg_deep().into(),
                    border_color: BorderColor(UiTheme::ornate_gold()),
                    ..default()
                })
                .with_children(|dialog| {
                    dialog.spawn(headline_text(format!(
                        "Skill book — slot {}",
                        target_slot + 1
                    )));
                    dialog.spawn(caption_text(
                        "Choose a skill for this slot. Picking a skill already equipped elsewhere clears it there first.",
                    ));
                    dialog.spawn(section_title("LIBRARY"));
                    dialog
                        .spawn((
                            NodeBundle {
                                style: Style {
                                    width: Val::Percent(100.0),
                                    height: Val::Px(340.0),
                                    flex_shrink: 0.0,
                                    position_type: PositionType::Relative,
                                    flex_direction: FlexDirection::Column,
                                    overflow: Overflow::clip_y(),
                                    border: UiRect::all(Val::Px(1.0)),
                                    ..default()
                                },
                                background_color: UiTheme::panel_bg().into(),
                                border_color: BorderColor(UiTheme::panel_border_inner()),
                                focus_policy: FocusPolicy::Pass,
                                ..default()
                            },
                            RelativeCursorPosition::default(),
                            UiScrollState::default(),
                            UiScrollRegion,
                        ))
                        .with_children(|viewport| {
                            viewport
                                .spawn((
                                    NodeBundle {
                                        style: Style {
                                            position_type: PositionType::Absolute,
                                            left: Val::Px(0.0),
                                            right: Val::Px(0.0),
                                            top: Val::Px(0.0),
                                            padding: UiRect::all(Val::Px(UiTheme::PANEL_INSET_SM)),
                                            flex_direction: FlexDirection::Column,
                                            align_items: AlignItems::Stretch,
                                            row_gap: Val::Px(6.0),
                                            ..default()
                                        },
                                        ..default()
                                    },
                                    UiScrollContent,
                                ))
                                .with_children(|inner| {
                                    spawn_pick_row(
                                        inner,
                                        ph,
                                        target_slot,
                                        None,
                                        "(Clear slot)",
                                        "Remove the skill from this slot.",
                                    );
                                    for id in skill_book_pick_order() {
                                        let d = skill_definition(id);
                                        let kind_str = match d.kind {
                                            SkillKind::Active => "Active",
                                            SkillKind::Passive => "Passive",
                                        };
                                        let label = format!(
                                            "{} · {} · {}",
                                            d.name,
                                            kind_str,
                                            format_skill_tags(d.tags)
                                        );
                                        let tip = format!(
                                            "{}\n{}\n{}\n\nSynergy: {}",
                                            d.name,
                                            kind_str,
                                            d.description,
                                            d.synergy_hint
                                        );
                                        spawn_pick_row(
                                            inner, ph, target_slot, Some(id), &label, &tip,
                                        );
                                    }
                                });
                        });
                    let close_pal = UiButtonPalette::panel_outlined();
                    dialog
                        .spawn((
                            ButtonBundle {
                                style: Style {
                                    width: Val::Percent(100.0),
                                    min_height: Val::Px(40.0),
                                    justify_content: JustifyContent::Center,
                                    align_items: AlignItems::Center,
                                    border: UiRect::all(Val::Px(1.0)),
                                    ..default()
                                },
                                background_color: close_pal.idle_bg.into(),
                                border_color: BorderColor(close_pal.idle_border),
                                ..default()
                            },
                            SkillBookCloseButton,
                            close_pal,
                            UiTooltip::txt("Close without changing the slot."),
                        ))
                        .with_children(|b| {
                            b.spawn(TextBundle::from_section(
                                "Close",
                                TextStyle {
                                    font_size: UiTheme::FONT_BODY,
                                    color: UiTheme::muted_cream(),
                                    ..default()
                                },
                            ));
                        });
                });
        });
}

fn pick_row_icon(ph: &UiPlaceholderImages, skill: Option<SkillId>) -> Handle<Image> {
    match skill {
        None => ph.skill_empty.clone(),
        Some(id) => match skill_definition(id).kind {
            SkillKind::Active => ph.skill_active.clone(),
            SkillKind::Passive => ph.skill_passive.clone(),
        },
    }
}

fn spawn_pick_row(
    inner: &mut ChildBuilder,
    ph: &UiPlaceholderImages,
    target_slot: usize,
    skill: Option<SkillId>,
    label: &str,
    tip: &str,
) {
    let p = UiButtonPalette::panel_outlined();
    inner
        .spawn((
            ButtonBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    min_height: Val::Px(44.0),
                    justify_content: JustifyContent::FlexStart,
                    align_items: AlignItems::Center,
                    padding: UiRect::axes(Val::Px(8.0), Val::Px(4.0)),
                    border: UiRect::all(Val::Px(1.0)),
                    ..default()
                },
                background_color: p.idle_bg.into(),
                border_color: BorderColor(p.idle_border),
                ..default()
            },
            SkillBookPickButton {
                slot: target_slot,
                skill,
            },
            p,
            UiTooltip::txt(tip.to_string()),
        ))
        .with_children(|b| {
            b.spawn(NodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    flex_direction: FlexDirection::Row,
                    column_gap: Val::Px(10.0),
                    align_items: AlignItems::Center,
                    ..default()
                },
                ..default()
            })
            .with_children(|row| {
                row.spawn(ImageBundle {
                    style: Style {
                        width: Val::Px(22.0),
                        height: Val::Px(22.0),
                        flex_shrink: 0.0,
                        ..default()
                    },
                    image: UiImage::new(pick_row_icon(ph, skill)),
                    background_color: Color::NONE.into(),
                    ..default()
                });
                row.spawn(TextBundle::from_section(
                    label,
                    TextStyle {
                        font_size: UiTheme::FONT_COMPACT,
                        color: UiTheme::body(),
                        ..default()
                    },
                ));
            });
        });
}
