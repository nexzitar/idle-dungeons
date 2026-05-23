use bevy::prelude::*;

use crate::app::{AssignHeroSkill, GameState, StartRun};
use crate::presentation::editor::TITLE_ELEMENT_FIREPLACE;
#[cfg(debug_assertions)]
use crate::presentation::editor::{
    apply_editor_tune_delta, PresentationEditorHierarchyButton, PresentationEditorTuneDeltaButton,
    PresentationEditorTuneValueButton, reset_all_title_placements,
    reset_editor_selection_placement,
};
use crate::ui::interaction::click::{resolve_clicked_action, UiClickPress};
use crate::ui::interaction::effects::apply_cycle_stash_sort;
use crate::ui::interaction::params::{UiClickQueries, UiClickState, UiClickWriters};
use crate::ui::interaction::registry::UiClickAction;
use crate::ui::scene_tune::TitleSceneLayout;

pub(crate) fn dispatch_ui_clicks(
    mouse: Res<ButtonInput<MouseButton>>,
    press: Res<UiClickPress>,
    mut commands: Commands,
    mut writers: UiClickWriters,
    mut q: UiClickQueries,
    mut s: UiClickState,
) {
    let Some(action) = resolve_clicked_action(&mouse, &press, &q.clicked) else {
        return;
    };

    match action {
        UiClickAction::TitleEnterCamp => s.next_state.set(GameState::Build),
        UiClickAction::TitleQuit => {
            writers.exit.write(AppExit::Success);
        }
        UiClickAction::StartRun => {
            writers.start_run.write(StartRun {
                seed: crate::domain::run::DEFAULT_RUN_SEED,
            });
        }
        UiClickAction::SkipPlayback => {
            writers.skip.write(crate::app::SkipRunPlayback);
        }
        UiClickAction::AcceptRewards => {
            writers.accept.write(crate::app::AcceptRunRewards);
        }
        UiClickAction::ResetProgress => {
            writers.reset.write(crate::app::ResetProgress);
        }
        UiClickAction::OpenSettings => {
            if q.settings_modal.is_empty() {
                if let Ok(root) = q.ui_roots.single() {
                    commands.entity(root).with_children(|parent| {
                        crate::ui::shell::spawn_settings_modal(parent);
                    });
                }
            }
        }
        UiClickAction::CloseSettings => {
            for e in &q.settings_modal {
                commands.entity(e).despawn();
            }
        }
        UiClickAction::OpenGearHub => {
            writers.open_gear.write(crate::app::OpenGearHub);
        }
        UiClickAction::CloseGearHub => {
            s.gear_keep.0 = false;
            for e in &q.gear_modal {
                commands.entity(e).despawn();
            }
        }
        UiClickAction::OpenSkillShop => {
            writers.open_shop.write(crate::app::OpenSkillShop);
        }
        UiClickAction::CloseSkillShop => {
            for e in &q.shop_modal {
                commands.entity(e).despawn();
            }
        }
        UiClickAction::CloseSkillBook => {
            for e in &q.book_modal {
                commands.entity(e).despawn();
            }
        }
        UiClickAction::SkillBookPick => {
            if let Some(target) = press.0.and_then(|t| q.book_pick.get(t).ok()) {
                writers.assign_skill.write(AssignHeroSkill {
                    slot: target.slot,
                    skill: target.skill,
                    kind: target.kind,
                });
                for e in &q.book_modal {
                    commands.entity(e).despawn();
                }
            }
        }
        UiClickAction::BuySkillUnlock => {
            if let Some(target) = press.0.and_then(|t| q.shop_buy.get(t).ok()) {
                writers
                    .buy_skill
                    .write(crate::app::BuySkillUnlock { skill: target.skill });
            }
        }
        UiClickAction::CycleStashSort => {
            apply_cycle_stash_sort(
                &mut s.profile,
                &s.save_path,
                &mut commands,
                s.speed.multiplier(),
                *s.game_state.get(),
                s.latest_summary.as_deref(),
                &q.build_roots.iter().collect::<Vec<_>>(),
                &q.summary_roots.iter().collect::<Vec<_>>(),
                &q.running_roots.iter().collect::<Vec<_>>(),
                &s.ph,
                &mut s.name_edit,
                &*s.gear_keep,
            );
        }
        UiClickAction::OpenSkillBook => {
            if *s.game_state.get() == GameState::Build {
                if let Some(target) = press.0.and_then(|t| q.skill_slots.get(t).ok()) {
                    writers.open_book.write(crate::app::OpenSkillBook {
                        slot: target.slot,
                        kind: target.kind,
                    });
                }
            }
        }
        UiClickAction::EquipItem => {
            if let Some(target) = press.0.and_then(|t| q.equip_btns.get(t).ok()) {
                writers.equip.write(crate::app::EquipInventoryItem {
                    item_id: target.item_id,
                });
            }
        }
        UiClickAction::SalvageItem => {
            if let Some(target) = press.0.and_then(|t| q.salvage_btns.get(t).ok()) {
                writers.salvage.write(crate::app::SalvageInventoryItem {
                    item_id: target.item_id,
                });
            }
        }
        UiClickAction::PlaybackSpeedDec => s.speed.dec(),
        UiClickAction::PlaybackSpeedInc => s.speed.inc(),
        UiClickAction::ToggleCombatLog => s.log_vis.0 = !s.log_vis.0,
        UiClickAction::HeroNameEdit => {
            if *s.game_state.get() == GameState::Build {
                if let Some(btn) = press.0.and_then(|t| q.hero_edit.get(t).ok()) {
                    if btn.slot == 1 && s.profile.profile.party_partner.is_none() {
                        return;
                    }
                    s.name_edit.active_slot = Some(btn.slot);
                    s.name_edit.buffer = hero_display_name_for_slot(&s.profile.profile, btn.slot);
                }
            }
        }
        #[cfg(debug_assertions)]
        UiClickAction::EditorToggleLayout => {
            let on = s.session.layout_mode();
            s.session.set_layout_mode(!on);
        }
        #[cfg(debug_assertions)]
        UiClickAction::EditorResetCenter => {
            let sel = s
                .session
                .selected_element
                .as_deref()
                .unwrap_or(TITLE_ELEMENT_FIREPLACE);
            reset_editor_selection_placement(&mut s.layout, sel);
            s.field_edit.clear();
        }
        #[cfg(debug_assertions)]
        UiClickAction::EditorResetAll => {
            reset_all_title_placements(&mut s.layout);
            s.field_edit.clear();
        }
        #[cfg(debug_assertions)]
        UiClickAction::EditorSave => s.layout.try_save_to_disk(),
        #[cfg(debug_assertions)]
        UiClickAction::EditorReload => {
            *s.layout = crate::ui::scene_tune::TitleSceneLayout::try_load_from_disk();
        }
        #[cfg(debug_assertions)]
        UiClickAction::EditorHierarchySelect
        | UiClickAction::EditorTuneValue
        | UiClickAction::EditorTuneDelta => {}
    }
}

fn hero_display_name_for_slot(profile: &crate::save::SaveProfile, slot: u8) -> String {
    match slot {
        0 => profile.hero.name.clone(),
        1 => profile
            .party_partner
            .as_ref()
            .map(|p| p.name.clone())
            .unwrap_or_default(),
        _ => String::new(),
    }
}

#[cfg(debug_assertions)]
pub(crate) fn dispatch_editor_ui_clicks(
    mouse: Res<ButtonInput<MouseButton>>,
    press: Res<UiClickPress>,
    clicked: Query<(Entity, &Interaction, &UiClickAction)>,
    mut session: ResMut<crate::presentation::PresentationEditorSession>,
    mut layout: ResMut<TitleSceneLayout>,
    mut field_edit: ResMut<crate::presentation::editor::PresentationEditorFieldEditState>,
    kb: Res<ButtonInput<KeyCode>>,
    hierarchy: Query<&PresentationEditorHierarchyButton>,
    tune_value: Query<&PresentationEditorTuneValueButton>,
    tune_delta: Query<&PresentationEditorTuneDeltaButton>,
) {
    let Some(action) = resolve_clicked_action(&mouse, &press, &clicked) else {
        return;
    };
    let Some(target) = press.0 else {
        return;
    };
    let sel = session
        .selected_element
        .as_deref()
        .unwrap_or(TITLE_ELEMENT_FIREPLACE);

    match action {
        UiClickAction::EditorHierarchySelect => {
            if let Ok(hb) = hierarchy.get(target) {
                session.selected_element = Some(hb.0.clone());
                field_edit.clear();
            }
        }
        UiClickAction::EditorTuneValue => {
            if let Ok(vb) = tune_value.get(target) {
                field_edit.begin(vb.0, &layout, sel);
            }
        }
        UiClickAction::EditorTuneDelta => {
            if let Ok(delta) = tune_delta.get(target) {
                let coarse = kb.any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight]);
                apply_editor_tune_delta(&mut layout, sel, delta.field, delta.positive, coarse);
                field_edit.clear();
            }
        }
        _ => {}
    }
}
