//! Fixed inspect panels — stable regions for skill/gear detail (buildcraft + camp).

use bevy::prelude::*;
use bevy::text::{Justify, TextColor, TextFont, TextLayout};

use crate::domain::party::PartyHeroKind;
use crate::domain::skills::{format_skill_tags, skill_category, skill_definition, SkillId, SkillKind};
use crate::ui::assets::UiPlaceholderImages;
use crate::ui::skill_presentation::icon_for;
use crate::ui::theme::{skill_category_chip_colors, UiTheme};

const INSPECT_FULL_PX: f32 = 200.0;
const INSPECT_COMPACT_PX: f32 = 140.0;

#[derive(Component)]
pub struct BuildcraftInspectPanel;

#[derive(Component)]
pub struct BuildcraftInspectIcon;

#[derive(Component)]
pub struct BuildcraftInspectTitle;

#[derive(Component)]
pub struct BuildcraftInspectMeta;

#[derive(Component)]
pub struct BuildcraftInspectTags;

#[derive(Component)]
pub struct BuildcraftInspectBody;

#[derive(Component)]
pub struct BuildcraftInspectSynergy;

#[derive(Component)]
pub struct BuildcraftInspectSlotHint;

#[derive(Component)]
pub struct CampInspectPanel;

#[derive(Component)]
pub struct CampInspectIcon;

#[derive(Component)]
pub struct CampInspectTitle;

#[derive(Component)]
pub struct CampInspectMeta;

#[derive(Component)]
pub struct CampInspectTags;

#[derive(Component)]
pub struct CampInspectBody;

#[derive(Component)]
pub struct CampInspectHint;

pub fn spawn_inspect_panel(parent: &mut ChildSpawnerCommands<'_>) {
    spawn_inspect_shell(
        parent,
        INSPECT_FULL_PX,
        false,
        "Hover a skill in the library or a loadout slot.",
        BuildcraftInspectPanel,
    );
}

/// Camp build column — ~140px fixed strip below party loadouts. Returns panel root entity.
pub fn spawn_inspect_panel_compact(parent: &mut ChildSpawnerCommands<'_>) -> Entity {
    spawn_inspect_shell(
        parent,
        INSPECT_COMPACT_PX,
        true,
        "Hover a loadout slot to see skill details.",
        CampInspectPanel,
    )
}

fn spawn_inspect_shell<M: Component>(
    parent: &mut ChildSpawnerCommands<'_>,
    height_px: f32,
    compact: bool,
    default_body: &'static str,
    panel_marker: M,
) -> Entity {
    let node = if compact {
        Node {
            box_sizing: BoxSizing::BorderBox,
            width: Val::Percent(100.0),
            height: Val::Px(height_px),
            flex_shrink: 0.0,
            margin: UiRect::top(Val::Px(UiTheme::GUTTER_ROW)),
            padding: UiRect::all(Val::Px(UiTheme::PANEL_INSET_SM)),
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(4.0),
            border: UiRect::all(Val::Px(1.0)),
            ..default()
        }
    } else {
        Node {
            box_sizing: BoxSizing::BorderBox,
            width: Val::Percent(100.0),
            height: Val::Px(height_px),
            flex_shrink: 0.0,
            padding: UiRect::all(Val::Px(UiTheme::PANEL_INSET)),
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(6.0),
            border: UiRect::all(Val::Px(1.0)),
            ..default()
        }
    };
    parent
        .spawn((
            node,
            BackgroundColor(UiTheme::panel_bg_deep()),
            BorderColor::from(UiTheme::panel_border_inner()),
            panel_marker,
        ))
        .with_children(|panel| {
            panel
                .spawn(Node {
                    box_sizing: BoxSizing::BorderBox,
                    flex_direction: FlexDirection::Row,
                    column_gap: Val::Px(10.0),
                    align_items: AlignItems::FlexStart,
                    ..default()
                })
                .with_children(|head| {
                    if compact {
                        head.spawn((
                            Node {
                                box_sizing: BoxSizing::BorderBox,
                                width: Val::Px(40.0),
                                height: Val::Px(40.0),
                                flex_shrink: 0.0,
                                ..default()
                            },
                            ImageNode {
                                image: Handle::default(),
                                ..default()
                            },
                            CampInspectIcon,
                        ));
                        spawn_inspect_head_text(head, "Inspect", false);
                    } else {
                        head.spawn((
                            Node {
                                box_sizing: BoxSizing::BorderBox,
                                width: Val::Px(44.0),
                                height: Val::Px(44.0),
                                flex_shrink: 0.0,
                                ..default()
                            },
                            ImageNode {
                                image: Handle::default(),
                                ..default()
                            },
                            BuildcraftInspectIcon,
                        ));
                        spawn_inspect_head_text(head, "Select a skill or slot", true);
                    }
                });
            if compact {
                panel.spawn((
                    Text::new(default_body),
                    TextFont::from_font_size(UiTheme::FONT_COMPACT),
                    TextColor(UiTheme::body()),
                    TextLayout::new_with_justify(Justify::Left),
                    CampInspectBody,
                ));
                panel.spawn((
                    Text::new("Click a slot to open Party Buildcraft."),
                    TextFont::from_font_size(UiTheme::FONT_CAPTION),
                    TextColor(UiTheme::muted_gold()),
                    CampInspectHint,
                ));
            } else {
                panel.spawn((
                    Text::new(default_body),
                    TextFont::from_font_size(UiTheme::FONT_BODY),
                    TextColor(UiTheme::body()),
                    TextLayout::new_with_justify(Justify::Left),
                    BuildcraftInspectBody,
                ));
                panel.spawn((
                    Text::new(""),
                    TextFont::from_font_size(UiTheme::FONT_CAPTION),
                    TextColor(UiTheme::body_dim()),
                    TextLayout::new_with_justify(Justify::Left),
                    BuildcraftInspectSynergy,
                ));
                panel.spawn((
                    Text::new(""),
                    TextFont::from_font_size(UiTheme::FONT_CAPTION),
                    TextColor(UiTheme::muted_gold()),
                    BuildcraftInspectSlotHint,
                ));
            }
        })
        .id()
}

fn spawn_inspect_head_text(
    parent: &mut ChildSpawnerCommands<'_>,
    default_title: &'static str,
    buildcraft: bool,
) {
    parent
        .spawn((
            Node {
                box_sizing: BoxSizing::BorderBox,
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(2.0),
                flex_grow: 1.0,
                min_width: Val::Px(0.0),
                ..default()
            },
        ))
        .with_children(|txt| {
            if buildcraft {
                txt.spawn((
                    Text::new(default_title),
                    TextFont::from_font_size(UiTheme::FONT_SECTION),
                    TextColor(UiTheme::muted_cream()),
                    BuildcraftInspectTitle,
                ));
                txt.spawn((
                    Text::new(""),
                    TextFont::from_font_size(UiTheme::FONT_CAPTION),
                    TextColor(UiTheme::body_dim()),
                    BuildcraftInspectMeta,
                ));
                txt.spawn((
                    Text::new(""),
                    TextFont::from_font_size(UiTheme::FONT_MICRO),
                    TextColor(UiTheme::body_dim()),
                    BuildcraftInspectTags,
                ));
            } else {
                txt.spawn((
                    Text::new(default_title),
                    TextFont::from_font_size(UiTheme::FONT_SECTION),
                    TextColor(UiTheme::muted_cream()),
                    CampInspectTitle,
                ));
                txt.spawn((
                    Text::new(""),
                    TextFont::from_font_size(UiTheme::FONT_CAPTION),
                    TextColor(UiTheme::body_dim()),
                    CampInspectMeta,
                ));
                txt.spawn((
                    Text::new(""),
                    TextFont::from_font_size(UiTheme::FONT_MICRO),
                    TextColor(UiTheme::body_dim()),
                    CampInspectTags,
                ));
            }
        });
}

fn skill_kind_label(kind: SkillKind) -> &'static str {
    match kind {
        SkillKind::Active => "Active",
        SkillKind::Passive => "Passive",
    }
}

pub struct InspectPanelContent {
    pub icon_image: Handle<Image>,
    pub icon_color: Color,
    pub title: String,
    pub meta: String,
    pub tags: String,
    pub body: String,
    pub synergy: String,
    pub hint: String,
}

pub fn inspect_content_none(ph: &UiPlaceholderImages) -> InspectPanelContent {
    InspectPanelContent {
        icon_image: ph.skill_empty.clone(),
        icon_color: Color::srgba(0.5, 0.48, 0.45, 0.6),
        title: "Select a skill or slot".to_string(),
        meta: String::new(),
        tags: String::new(),
        body: "Hover a skill in the library or a loadout slot.".to_string(),
        synergy: String::new(),
        hint: String::new(),
    }
}

pub fn inspect_content_none_camp(ph: &UiPlaceholderImages) -> InspectPanelContent {
    InspectPanelContent {
        icon_image: ph.skill_empty.clone(),
        icon_color: Color::srgba(0.5, 0.48, 0.45, 0.6),
        title: "Inspect".to_string(),
        meta: String::new(),
        tags: String::new(),
        body: "Hover a loadout slot to see skill details.".to_string(),
        synergy: String::new(),
        hint: "Click a slot to open Party Buildcraft.".to_string(),
    }
}

pub fn inspect_content_slot(
    hero: PartyHeroKind,
    index: usize,
    skill: Option<SkillId>,
    ph: &UiPlaceholderImages,
) -> InspectPanelContent {
    let slot_n = index + 1;
    let hint = format!(
        "{} · slot {slot_n} — left-to-right priority when multiple skills are ready.",
        hero.label()
    );
    let Some(id) = skill else {
        return InspectPanelContent {
            icon_image: ph.skill_empty.clone(),
            icon_color: Color::srgba(0.5, 0.48, 0.45, 0.6),
            title: format!("{} — empty slot {slot_n}", hero.label()),
            meta: "Click a library skill to assign.".to_string(),
            tags: String::new(),
            body: "Empty slots are skipped in combat priority order.".to_string(),
            synergy: String::new(),
            hint,
        };
    };
    let mut content = inspect_content_library(id, ph);
    content.hint = hint;
    content
}

pub fn inspect_content_camp_slot(
    hero: PartyHeroKind,
    index: usize,
    skill: Option<SkillId>,
    ph: &UiPlaceholderImages,
) -> InspectPanelContent {
    let slot_n = index + 1;
    let hint = "Click to open Party Buildcraft.".to_string();
    let Some(id) = skill else {
        return InspectPanelContent {
            icon_image: ph.skill_empty.clone(),
            icon_color: Color::srgba(0.5, 0.48, 0.45, 0.6),
            title: format!("{} — empty slot {slot_n}", hero.label()),
            meta: "No skill assigned.".to_string(),
            tags: String::new(),
            body: "Empty slots are skipped in combat priority order.".to_string(),
            synergy: String::new(),
            hint,
        };
    };
    let mut content = inspect_content_library(id, ph);
    content.hint = hint;
    content.meta = format!("{} · slot {slot_n}", hero.label());
    content
}

pub fn inspect_content_library(id: SkillId, ph: &UiPlaceholderImages) -> InspectPanelContent {
    let def = skill_definition(id);
    let cat = skill_category(id);
    let (chip_bg, _chip_fg) = skill_category_chip_colors(cat);
    InspectPanelContent {
        icon_image: icon_for(id, ph),
        icon_color: chip_bg.mix(&Color::WHITE, 0.35),
        title: def.name.to_string(),
        meta: format!("{} · {}", skill_kind_label(def.kind), cat.display_label()),
        tags: format_skill_tags(def.tags),
        body: def.description.to_string(),
        synergy: format!("Synergy: {}", def.synergy_hint),
        hint: "Click while a loadout slot is focused to assign.".to_string(),
    }
}
