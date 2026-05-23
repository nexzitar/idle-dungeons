//! Full-screen presentation editor chrome (title camp first).

use crate::presentation::editor::PresentationEditorSession;
use crate::presentation::element::PresentationElementId;
use crate::presentation::layer::{TITLE_CAMP_LAYER_REGISTRY, TITLE_ELEMENT_FIREPLACE};
use crate::ui::components::UiTooltip;
use crate::ui::interaction::UiClickAction;
use crate::ui::primitives::button::{
    spawn_button, spawn_button_with_extra_text, UiButtonConfig, UiButtonVariant,
};
use crate::ui::primitives::panel::spawn_framed_column;
use crate::ui::theme::{section_title, UiPanelStyle, UiTheme};
use bevy::picking::prelude::Pickable;
use bevy::prelude::*;
use bevy::text::{Justify, TextColor, TextFont, TextLayout};
use bevy::ui::{FocusPolicy, GlobalZIndex};

/// Root node for the presentation editor overlay (spawned under title [`UiRoot`](`crate::ui::components::UiRoot`)).
#[derive(Component)]
pub struct PresentationEditorRoot;

/// Hierarchy button: cycles [`PresentationEditorSession::selected_element`].
#[derive(Component)]
pub struct PresentationEditorHierarchyButton(pub PresentationElementId);

/// Footer — writes `assets/tuning/title_scene.json`.
#[derive(Component)]
pub struct PresentationEditorSaveButton;

/// Footer — reloads layout from disk into [`TitleSceneLayout`](`crate::ui::scene_tune::TitleSceneLayout`).
#[derive(Component)]
pub struct PresentationEditorReloadButton;

/// Footer — resets the selected element's offsets/scale/rotation to anchor center.
#[derive(Component)]
pub struct PresentationEditorResetCenterButton;

/// Footer — resets placement for fireplace, lead, and ally slots.
#[derive(Component)]
pub struct PresentationEditorResetAllButton;

/// Settings modal row (debug): toggles [`PresentationEditorSession::active`].
#[derive(Component)]
pub struct PresentationEditorSettingsToggleButton;

/// Text line on the settings toggle row (updated by editor sync).
#[derive(Component)]
pub struct PresentationEditorSettingsToggleText;

/// Main banner line: `Presentation Mode · {id}`.
#[derive(Component)]
pub struct PresentationEditorBannerTitleText;

/// Optional first-visit hint under the banner.
#[derive(Component)]
pub struct PresentationEditorBannerHintText;

/// One inspectable numeric/scalar field in the right-hand column.
#[derive(Component, Clone, Copy, PartialEq, Eq)]
pub enum PresentationEditorTuneField {
    OffsetX,
    OffsetY,
    ScaleX,
    ScaleY,
    SizeBasis,
    RotationDeg,
    Exposure,
    Glow,
    Bloom,
    GlobalZ,
    /// `fire_presentation.glow_alpha.base_value` (compositional track).
    FireGlowAlphaBase,
    /// First sine layer on glow track — breathing rate (Hz).
    FireGlowBreathHz,
    /// `fire_presentation.ground_alpha.base_value`.
    FireGroundAlphaBase,
    /// Flame crossfade period (seconds).
    FireCrossfadeSecs,
    /// Minimum glow alpha floor.
    FireGlowMinAlpha,
}

/// Inspector block for campfire atmosphere scalars / tracks (visible for fireplace selection).
#[derive(Component)]
pub struct PresentationEditorFireAtmosphereBlock;

/// Value text for a tune field row.
#[derive(Component)]
pub struct PresentationEditorTuneValueText(pub PresentationEditorTuneField);

/// Click the value to type a number directly (Enter applies, Esc cancels).
#[derive(Component)]
pub struct PresentationEditorTuneValueButton(pub PresentationEditorTuneField);

/// Read-only pivot / anchor summary at the bottom of the inspector.
#[derive(Component)]
pub struct PresentationEditorPivotSummaryText;

/// Small − / + control for a tune field (`positive` = increment).
#[derive(Component)]
pub struct PresentationEditorTuneDeltaButton {
    pub field: PresentationEditorTuneField,
    pub positive: bool,
}

pub fn spawn_presentation_editor_overlay(parent: &mut ChildSpawnerCommands<'_>) {
    parent
        .spawn((
            Node {
                box_sizing: BoxSizing::BorderBox,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                position_type: PositionType::Absolute,
                left: Val::Px(0.0),
                top: Val::Px(0.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Stretch,
                ..default()
            },
            GlobalZIndex(10_000),
            BackgroundColor(Color::srgba(0.02, 0.02, 0.04, 0.55)),
            PresentationEditorRoot,
            Visibility::Hidden,
            FocusPolicy::Pass,
            Pickable::IGNORE,
        ))
        .with_children(|root| {
            root.spawn((
                Node {
                    box_sizing: BoxSizing::BorderBox,
                    width: Val::Percent(100.0),
                    flex_shrink: 0.0,
                    min_height: Val::Px(44.0),
                    padding: UiRect::axes(Val::Px(14.0), Val::Px(8.0)),
                    border: UiRect::bottom(Val::Px(1.0)),
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(4.0),
                    justify_content: JustifyContent::Center,
                    ..default()
                },
                BackgroundColor(UiTheme::panel_bg_deep().into()),
                BorderColor::from(UiTheme::panel_border_inner()),
            ))
            .with_children(|banner| {
                banner.spawn((
                    Text::new("Presentation Mode · …"),
                    TextFont::from_font_size(UiTheme::FONT_SECTION),
                    TextColor(UiTheme::muted_cream()),
                    TextLayout::new_with_justify(Justify::Left),
                    PresentationEditorBannerTitleText,
                ));
                banner.spawn((
                    Text::new(""),
                    TextFont::from_font_size(UiTheme::FONT_CAPTION),
                    TextColor(UiTheme::body_dim()),
                    TextLayout::new_with_justify(Justify::Left),
                    PresentationEditorBannerHintText,
                ));
            });

            root.spawn((
                Node {
                    box_sizing: BoxSizing::BorderBox,
                    width: Val::Percent(100.0),
                    flex_grow: 1.0,
                    flex_shrink: 1.0,
                    min_height: Val::Px(0.0),
                    flex_direction: FlexDirection::Row,
                    justify_content: JustifyContent::SpaceBetween,
                    align_items: AlignItems::Stretch,
                    padding: UiRect::axes(Val::Px(12.0), Val::Px(10.0)),
                    column_gap: Val::Px(12.0),
                    ..default()
                },
                FocusPolicy::Pass,
            ))
            .with_children(|mid| {
                spawn_hierarchy_column(mid);
                spawn_inspector_column(mid);
            });

            root.spawn((
                Node {
                    box_sizing: BoxSizing::BorderBox,
                    width: Val::Percent(100.0),
                    flex_shrink: 0.0,
                    min_height: Val::Px(52.0),
                    padding: UiRect::axes(Val::Px(14.0), Val::Px(10.0)),
                    border: UiRect::top(Val::Px(1.0)),
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::FlexEnd,
                    column_gap: Val::Px(10.0),
                    ..default()
                },
                BackgroundColor(UiTheme::panel_bg_deep().into()),
                BorderColor::from(UiTheme::panel_border_inner()),
            ))
            .with_children(|foot| {
                spawn_footer_button(
                    foot,
                    "Reset to center",
                    PresentationEditorResetCenterButton,
                    UiClickAction::EditorResetCenter,
                    "Selected element: zero offsets on its anchor, scale 1, rotation 0.",
                );
                spawn_footer_button(
                    foot,
                    "Reset all",
                    PresentationEditorResetAllButton,
                    UiClickAction::EditorResetAll,
                    "All camp elements: snap fireplace, lead, and ally back to anchor center.",
                );
                spawn_footer_button(
                    foot,
                    "Save to disk",
                    PresentationEditorSaveButton,
                    UiClickAction::EditorSave,
                    "Write title scene JSON (same as debug Ctrl+S).",
                );
                spawn_footer_button(
                    foot,
                    "Reload from disk",
                    PresentationEditorReloadButton,
                    UiClickAction::EditorReload,
                    "Reload title scene JSON (same as debug F5).",
                );
            });
        });
}

fn spawn_hierarchy_column(parent: &mut ChildSpawnerCommands<'_>) {
    spawn_framed_column(
        parent,
        UiPanelStyle::editor_sidebar(),
        Val::Px(210.0),
        0.0,
        8.0,
        Overflow::default(),
        None,
        |col| {
            col.spawn(section_title("Elements"));
            for entry in TITLE_CAMP_LAYER_REGISTRY {
                hierarchy_row(col, entry.host_label, entry.element_id);
                for layer in entry.layers {
                    hierarchy_row_indented(col, layer.label, layer.id);
                }
            }
        },
    );
}

fn hierarchy_row_indented(parent: &mut ChildSpawnerCommands<'_>, label: &str, id: &'static str) {
    parent
        .spawn(Node {
            box_sizing: BoxSizing::BorderBox,
            padding: UiRect::left(Val::Px(10.0)),
            width: Val::Percent(100.0),
            ..default()
        })
        .with_children(|wrap| {
            hierarchy_row(wrap, label, id);
        });
}

fn hierarchy_row(parent: &mut ChildSpawnerCommands<'_>, label: &str, id: &'static str) {
    spawn_editor_button(
        parent,
        label,
        UiButtonVariant::PanelSecondary,
        UiTheme::body(),
        Val::Percent(100.0),
        Val::Px(36.0),
        UiTheme::FONT_BODY,
        PresentationEditorHierarchyButton(id.into()),
        UiClickAction::EditorHierarchySelect,
        "Select this presentation element for editing.",
    );
}

fn spawn_inspector_column(parent: &mut ChildSpawnerCommands<'_>) {
    spawn_framed_column(
        parent,
        UiPanelStyle::editor_sidebar(),
        Val::Px(320.0),
        0.0,
        6.0,
        Overflow::scroll_y(),
        Some(Val::Percent(100.0)),
        |col| {
            col.spawn(section_title("Inspector"));
            tune_row(
                col,
                "Offset X (px)",
                PresentationEditorTuneField::OffsetX,
                "±1 px",
            );
            tune_row(
                col,
                "Offset Y (px)",
                PresentationEditorTuneField::OffsetY,
                "±1 px",
            );
            tune_row(col, "Scale X", PresentationEditorTuneField::ScaleX, "±0.01");
            tune_row(col, "Scale Y", PresentationEditorTuneField::ScaleY, "±0.01");
            tune_row(
                col,
                "Size basis",
                PresentationEditorTuneField::SizeBasis,
                "±2 px",
            );
            tune_row(
                col,
                "Rotation °",
                PresentationEditorTuneField::RotationDeg,
                "±1°",
            );
            tune_row(
                col,
                "Exposure",
                PresentationEditorTuneField::Exposure,
                "±0.02",
            );
            tune_row(col, "Glow", PresentationEditorTuneField::Glow, "±0.02");
            tune_row(col, "Bloom", PresentationEditorTuneField::Bloom, "±0.02");
            tune_row(
                col,
                "Global Z",
                PresentationEditorTuneField::GlobalZ,
                "±1 layer",
            );
            col.spawn((
                Node {
                    box_sizing: BoxSizing::BorderBox,
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(4.0),
                    margin: UiRect::top(Val::Px(8.0)),
                    ..default()
                },
                Visibility::Hidden,
                PresentationEditorFireAtmosphereBlock,
            ))
            .with_children(|atm| {
                atm.spawn(section_title("Fire atmosphere"));
                tune_row(
                    atm,
                    "Glow α base",
                    PresentationEditorTuneField::FireGlowAlphaBase,
                    "±0.01",
                );
                tune_row(
                    atm,
                    "Glow breath Hz",
                    PresentationEditorTuneField::FireGlowBreathHz,
                    "±0.01",
                );
                tune_row(
                    atm,
                    "Ground α base",
                    PresentationEditorTuneField::FireGroundAlphaBase,
                    "±0.01",
                );
                tune_row(
                    atm,
                    "Flame period s",
                    PresentationEditorTuneField::FireCrossfadeSecs,
                    "±0.1",
                );
                tune_row(
                    atm,
                    "Glow α floor",
                    PresentationEditorTuneField::FireGlowMinAlpha,
                    "±0.01",
                );
            });
            col.spawn((
                Text::new("Pivot: …"),
                TextFont::from_font_size(UiTheme::FONT_CAPTION),
                TextColor(UiTheme::body_dim()),
                TextLayout::new_with_justify(Justify::Left),
                PresentationEditorPivotSummaryText,
            ));
        },
    );
}

fn spawn_editor_button<M: Component>(
    parent: &mut ChildSpawnerCommands<'_>,
    label: &str,
    variant: UiButtonVariant,
    text_color: Color,
    width: Val,
    height: Val,
    font_size: f32,
    marker: M,
    action: UiClickAction,
    tip: &'static str,
) {
    let entity = spawn_button(
        parent,
        UiButtonConfig {
            label,
            variant,
            width,
            height,
            font_size,
            text_color,
            flex_shrink: 0.0,
        },
    );
    parent
        .commands_mut()
        .entity(entity)
        .insert((marker, action, UiTooltip::txt(tip)));
}

fn spawn_footer_button<M: Component>(
    parent: &mut ChildSpawnerCommands<'_>,
    label: &str,
    marker: M,
    action: UiClickAction,
    tip: &'static str,
) {
    spawn_editor_button(
        parent,
        label,
        UiButtonVariant::PanelOutlined,
        UiTheme::body(),
        Val::Px(150.0),
        Val::Px(36.0),
        UiTheme::FONT_BODY,
        marker,
        action,
        tip,
    );
}

fn tune_row(
    parent: &mut ChildSpawnerCommands<'_>,
    label: &str,
    field: PresentationEditorTuneField,
    _step_hint: &'static str,
) {
    parent
        .spawn((
            Node {
                box_sizing: BoxSizing::BorderBox,
                width: Val::Percent(100.0),
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::SpaceBetween,
                column_gap: Val::Px(6.0),
                padding: UiRect::vertical(Val::Px(2.0)),
                ..default()
            },
            FocusPolicy::Pass,
        ))
        .with_children(|row| {
            row.spawn((
                Text::new(format!("{label}:")),
                TextFont::from_font_size(UiTheme::FONT_CAPTION),
                TextColor(UiTheme::body_dim()),
                TextLayout::new_with_justify(Justify::Left),
            ));
            row.spawn((
                Node {
                    box_sizing: BoxSizing::BorderBox,
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    column_gap: Val::Px(4.0),
                    ..default()
                },
                FocusPolicy::Pass,
            ))
            .with_children(|r| {
                spawn_editor_button(
                    r,
                    "−",
                    UiButtonVariant::PanelOutlined,
                    UiTheme::body(),
                    Val::Px(28.0),
                    Val::Px(26.0),
                    UiTheme::FONT_LABEL,
                    PresentationEditorTuneDeltaButton {
                        field,
                        positive: false,
                    },
                    UiClickAction::EditorTuneDelta,
                    "Decrease (Shift = 10× step).",
                );
                let value_ent = spawn_button_with_extra_text(
                    r,
                    UiButtonConfig {
                        label: "0.00",
                        variant: UiButtonVariant::PanelSecondary,
                        width: Val::Px(72.0),
                        height: Val::Px(26.0),
                        font_size: UiTheme::FONT_CAPTION,
                        text_color: UiTheme::muted_cream(),
                        flex_shrink: 0.0,
                    },
                    (
                        TextLayout::new_with_justify(Justify::Center),
                        PresentationEditorTuneValueText(field),
                    ),
                );
                r.commands_mut().entity(value_ent).insert((
                    PresentationEditorTuneValueButton(field),
                    UiClickAction::EditorTuneValue,
                    UiTooltip::txt(
                        "Click to type a value. Enter applies, Esc cancels. Hold Shift with −/+ for 10× steps.",
                    ),
                ));
                spawn_editor_button(
                    r,
                    "+",
                    UiButtonVariant::PanelOutlined,
                    UiTheme::body(),
                    Val::Px(28.0),
                    Val::Px(26.0),
                    UiTheme::FONT_LABEL,
                    PresentationEditorTuneDeltaButton {
                        field,
                        positive: true,
                    },
                    UiClickAction::EditorTuneDelta,
                    "Increase (Shift = 10× step).",
                );
            });
        });
}

// --- Helpers shared with `mod.rs` systems ---

pub(super) fn banner_selected_id(session: &PresentationEditorSession) -> &str {
    session
        .selected_element
        .as_deref()
        .unwrap_or(TITLE_ELEMENT_FIREPLACE)
}

pub(super) fn format_pivot_line(
    tune: &crate::presentation::element::PresentationElementTune,
) -> String {
    let anchor = tune.anchor_ref.as_deref().unwrap_or("(none)");
    format!("Pivot: {:?} · Anchor: {anchor}", tune.pivot)
}
