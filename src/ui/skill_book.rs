//! Skill book modal: pick a skill (or clear) for a loadout slot.

use bevy::prelude::*;
use bevy::ui::{FocusPolicy, RelativeCursorPosition};

use crate::domain::party::PartyHeroKind;
use crate::domain::skills::{
    format_skill_tags, skill_book_pick_order_for, skill_category, skill_definition, SkillId,
    SkillKind,
};
use crate::ui::components::{
    SkillBookBackdrop, SkillBookCloseButton, SkillBookPickButton, SkillBookRoot, UiButtonPalette,
    UiScrollContent, UiScrollRegion, UiScrollState, UiTooltip,
};
use crate::ui::placeholder_graphics::UiPlaceholderImages;
use crate::ui::theme::{caption_text, headline_text, section_title, skill_category_chip_colors, UiTheme};

pub fn spawn_skill_book_modal(
    parent: &mut ChildSpawnerCommands<'_>,
    target_slot: usize,
    sheet: PartyHeroKind,
    unlocked: &[crate::domain::skills::SkillId],
    ph: &UiPlaceholderImages,
) {
    parent
        .spawn((
            Node {
                box_sizing: BoxSizing::BorderBox,
                width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    position_type: PositionType::Absolute,
                    left: Val::Px(0.0),
                    top: Val::Px(0.0),
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
                Node {
                box_sizing: BoxSizing::BorderBox,
                position_type: PositionType::Absolute,
                        width: Val::Percent(100.0),
                        height: Val::Percent(100.0),
                        left: Val::Px(0.0),
                        top: Val::Px(0.0),
                        ..default()
            },
            Button,
            BackgroundColor(backdrop_pal.idle_bg.into()),
            BorderColor::from(backdrop_pal.idle_border),
                SkillBookBackdrop,
                backdrop_pal,
                UiTooltip::txt("Click outside to close the skill book."),
            ));
            layer
                .spawn((
            Node {
                box_sizing: BoxSizing::BorderBox,
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
            BackgroundColor(UiTheme::panel_bg_deep().into()),
            BorderColor::from(UiTheme::ornate_gold())
        ))
                .with_children(|dialog| {
                    let who = match sheet {
                        PartyHeroKind::Lead => "Lead",
                        PartyHeroKind::Partner => "Ally",
                    };
                    dialog.spawn(headline_text(format!(
                        "Skill book — {who} — slot {}",
                        target_slot + 1
                    )));
                    dialog.spawn(caption_text(
                        "Choose a skill for this slot. Picking a skill already equipped elsewhere clears it there first.",
                    ));
                    dialog.spawn(section_title("LIBRARY"));
                    dialog
                        .spawn((
                            Node {
                box_sizing: BoxSizing::BorderBox,
                width: Val::Percent(100.0),
                                    height: Val::Px(340.0),
                                    flex_shrink: 0.0,
                                    position_type: PositionType::Relative,
                                    flex_direction: FlexDirection::Column,
                                    overflow: Overflow::clip_y(),
                                    border: UiRect::all(Val::Px(1.0)),
                                    ..default()
            },
            BackgroundColor(UiTheme::panel_bg().into()),
            BorderColor::from(UiTheme::panel_border_inner()),
            FocusPolicy::Pass,
                            RelativeCursorPosition::default(),
                            UiScrollState::default(),
                            UiScrollRegion,
                        ))
                        .with_children(|viewport| {
                            viewport
                                .spawn((
                                    Node {
                box_sizing: BoxSizing::BorderBox,
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
                                    UiScrollContent,
                                ))
                                .with_children(|inner| {
                                    spawn_pick_row(
                                        inner,
                                        ph,
                                        target_slot,
                                        sheet,
                                        None,
                                        "(Clear slot)",
                                        "Remove the skill from this slot.",
                                    );
                                    for id in skill_book_pick_order_for(unlocked) {
                                        let d = skill_definition(id);
                                        let kind_str = match d.kind {
                                            SkillKind::Active => "Active",
                                            SkillKind::Passive => "Passive",
                                        };
                                        let cat = skill_category(id);
                                        let label = format!(
                                            "{} · {} · {} · {}",
                                            d.name,
                                            kind_str,
                                            cat.display_label(),
                                            format_skill_tags(d.tags)
                                        );
                                        let tip = format!(
                                            "{}\n{} · {}\n{}\n\nSynergy: {}",
                                            d.name,
                                            kind_str,
                                            cat.display_label(),
                                            d.description,
                                            d.synergy_hint
                                        );
                                        spawn_pick_row(
                                            inner, ph, target_slot, sheet, Some(id), &label, &tip,
                                        );
                                    }
                                });
                        });
                    let close_pal = UiButtonPalette::panel_outlined();
                    dialog
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
            BackgroundColor(close_pal.idle_bg.into()),
            BorderColor::from(close_pal.idle_border),
                            SkillBookCloseButton,
                            close_pal,
                            UiTooltip::txt("Close without changing the slot."),
                        ))
                        .with_children(|b| {
                            b.spawn((
                Text::new("Close"),
                TextFont::from_font_size(UiTheme::FONT_BODY),
                TextColor(UiTheme::muted_cream()),
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
    inner: &mut ChildSpawnerCommands<'_>,
    ph: &UiPlaceholderImages,
    target_slot: usize,
    sheet: PartyHeroKind,
    skill: Option<SkillId>,
    label: &str,
    tip: &str,
) {
    let p = UiButtonPalette::panel_outlined();
    inner
        .spawn((
            Node {
                box_sizing: BoxSizing::BorderBox,
                width: Val::Percent(100.0),
                    min_height: Val::Px(44.0),
                    justify_content: JustifyContent::FlexStart,
                    align_items: AlignItems::Center,
                    padding: UiRect::axes(Val::Px(8.0), Val::Px(4.0)),
                    border: UiRect::all(Val::Px(1.0)),
                    ..default()
            },
            Button,
            BackgroundColor(p.idle_bg.into()),
            BorderColor::from(p.idle_border),
            SkillBookPickButton {
                slot: target_slot,
                skill,
                kind: sheet,
            },
            p,
            UiTooltip::txt(tip.to_string()),
        ))
        .with_children(|b| {
            b.spawn(Node {
                box_sizing: BoxSizing::BorderBox,
                width: Val::Percent(100.0),
                    flex_direction: FlexDirection::Row,
                    column_gap: Val::Px(10.0),
                    align_items: AlignItems::Center,
                    ..default()
            })
            .with_children(|row| {
                if let Some(id) = skill {
                    let cat = skill_category(id);
                    let (chip_bg, chip_fg) = skill_category_chip_colors(cat);
                    row.spawn((
                        Node {
                            box_sizing: BoxSizing::BorderBox,
                            min_width: Val::Px(30.0),
                            padding: UiRect::axes(Val::Px(4.0), Val::Px(1.0)),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            flex_shrink: 0.0,
                            border: UiRect::all(Val::Px(1.0)),
                            ..default()
                        },
                        BackgroundColor(chip_bg.into()),
                        BorderColor::from(UiTheme::panel_border_inner()),
                    ))
                    .with_children(|c| {
                        c.spawn((
                            Text::new(cat.category_abbr()),
                            TextFont::from_font_size(UiTheme::FONT_MICRO),
                            TextColor(chip_fg),
                        ));
                    });
                } else {
                    row.spawn((Node {
                        box_sizing: BoxSizing::BorderBox,
                            width: Val::Px(30.0),
                            flex_shrink: 0.0,
                            ..default()
                    },));
                }
                row.spawn((
                    Node {
                        box_sizing: BoxSizing::BorderBox,
                        width: Val::Px(22.0),
                        height: Val::Px(22.0),
                        flex_shrink: 0.0,
                        ..default()
                    },
                    ImageNode {
                        image: pick_row_icon(ph, skill),
                        color: Color::WHITE,
                        ..default()
                    },
                ));
                row.spawn((
                Text::new(label),
                TextFont::from_font_size(UiTheme::FONT_COMPACT),
                TextColor(UiTheme::body()),
            ));
            });
        });
}
