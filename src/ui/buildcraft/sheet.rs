//! Full-screen party buildcraft sheet layout.

use bevy::prelude::*;

use crate::domain::skill_layering::layering_warnings;
use crate::domain::skills::SkillId;
use crate::ui::assets::UiPlaceholderImages;
use crate::ui::buildcraft::library::spawn_library_column;
use crate::ui::buildcraft::party_column::spawn_party_column;
use crate::ui::buildcraft::session::BuildcraftEditSession;
use crate::ui::components::{SkillBookBackdrop, SkillBookRoot};
use crate::ui::interaction::UiClickAction;
use crate::ui::primitives::button::{spawn_button, UiButtonConfig, UiButtonVariant};
use crate::ui::primitives::inspect_panel::spawn_inspect_panel;
use crate::ui::primitives::modal::{spawn_modal_shell_with_handles, ModalShellConfig};
use crate::ui::theme::{headline_text, UiTheme};

#[derive(Component)]
pub struct BuildcraftApplyButton;

#[derive(Component)]
pub struct BuildcraftCancelButton;

pub fn spawn_buildcraft_sheet(
    parent: &mut ChildSpawnerCommands<'_>,
    session: &BuildcraftEditSession,
    unlocked: &[SkillId],
    ph: &UiPlaceholderImages,
) {
    let shell = spawn_modal_shell_with_handles(
        parent,
        ModalShellConfig {
            backdrop_clicks_close: true,
        },
        |layer| {
            layer
                .spawn((
                    Node {
                        box_sizing: BoxSizing::BorderBox,
                        width: Val::Percent(96.0),
                        height: Val::Percent(92.0),
                        max_width: Val::Px(1180.0),
                        align_self: AlignSelf::Center,
                        margin: UiRect::vertical(Val::Px(24.0)),
                        padding: UiRect::all(Val::Px(UiTheme::PAD_ROOT)),
                        flex_direction: FlexDirection::Column,
                        row_gap: Val::Px(12.0),
                        border: UiRect::all(Val::Px(2.0)),
                        ..default()
                    },
                    BackgroundColor(UiTheme::panel_bg_deep()),
                    BorderColor::from(UiTheme::ornate_gold()),
                ))
                .with_children(|panel| {
                    spawn_header(panel, session);
                    panel
                        .spawn((
                            Node {
                                box_sizing: BoxSizing::BorderBox,
                                width: Val::Percent(100.0),
                                flex_grow: 1.0,
                                min_height: Val::Px(0.0),
                                flex_direction: FlexDirection::Row,
                                column_gap: Val::Px(14.0),
                                align_items: AlignItems::Stretch,
                                ..default()
                            },
                        ))
                        .with_children(|body| {
                            spawn_party_column(body, session, ph);
                            body.spawn((
                                Node {
                                    box_sizing: BoxSizing::BorderBox,
                                    width: Val::Percent(62.0),
                                    flex_grow: 1.0,
                                    min_width: Val::Px(0.0),
                                    flex_direction: FlexDirection::Column,
                                    row_gap: Val::Px(10.0),
                                    ..default()
                                },
                            ))
                            .with_children(|right| {
                                spawn_library_column(right, unlocked, ph);
                                spawn_inspect_panel(right);
                            });
                        });
                    spawn_footer(panel);
                });
        },
    );

    parent.commands_mut().entity(shell.shell_root).insert(SkillBookRoot);
    parent.commands_mut().entity(shell.backdrop).insert((
        SkillBookBackdrop,
        UiClickAction::BuildcraftCancel,
    ));
}

fn spawn_header(parent: &mut ChildSpawnerCommands<'_>, session: &BuildcraftEditSession) {
    parent
        .spawn(Node {
            box_sizing: BoxSizing::BorderBox,
            width: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(4.0),
            ..default()
        })
        .with_children(|head| {
            head.spawn((
                Node {
                    box_sizing: BoxSizing::BorderBox,
                    width: Val::Percent(100.0),
                    flex_direction: FlexDirection::Row,
                    justify_content: JustifyContent::SpaceBetween,
                    align_items: AlignItems::Center,
                    ..default()
                },
            ))
            .with_children(|row| {
                row.spawn(headline_text("Party Buildcraft"));
                row.spawn(Node {
                    box_sizing: BoxSizing::BorderBox,
                    flex_direction: FlexDirection::Row,
                    column_gap: Val::Px(8.0),
                    ..default()
                })
                .with_children(|actions| {
                    let cancel = spawn_button(
                        actions,
                        UiButtonConfig {
                            label: "Cancel",
                            variant: UiButtonVariant::PanelOutlined,
                            width: Val::Px(100.0),
                            height: Val::Px(36.0),
                            font_size: UiTheme::FONT_BODY,
                            text_color: UiTheme::body(),
                            flex_shrink: 0.0,
                        },
                    );
                    actions.commands_mut().entity(cancel).insert((
                        BuildcraftCancelButton,
                        UiClickAction::BuildcraftCancel,
                    ));
                    let apply = spawn_button(
                        actions,
                        UiButtonConfig {
                            label: "Apply",
                            variant: UiButtonVariant::Primary,
                            width: Val::Px(100.0),
                            height: Val::Px(36.0),
                            font_size: UiTheme::FONT_BODY,
                            text_color: Color::WHITE,
                            flex_shrink: 0.0,
                        },
                    );
                    actions.commands_mut().entity(apply).insert((
                        BuildcraftApplyButton,
                        UiClickAction::BuildcraftApply,
                    ));
                });
            });
            for edit in std::iter::once(&session.lead).chain(session.partner.as_ref()) {
                let ids = edit.pending.iter().filter_map(|s| *s);
                for msg in layering_warnings(ids) {
                    head.spawn((
                        Text::new(format!("{}: {msg}", edit.display_name)),
                        TextFont::from_font_size(UiTheme::FONT_CAPTION),
                        TextColor(UiTheme::danger()),
                    ));
                }
            }
        });
}

fn spawn_footer(parent: &mut ChildSpawnerCommands<'_>) {
    parent.spawn((
        Text::new("Click a loadout slot, then pick a library skill. Apply saves all party changes."),
        TextFont::from_font_size(UiTheme::FONT_CAPTION),
        TextColor(UiTheme::body_dim()),
    ));
}
