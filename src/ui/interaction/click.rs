use bevy::ecs::system::SystemParam;
use bevy::input::mouse::MouseButton;
use bevy::prelude::*;

use crate::presentation::editor::PresentationEditorSettingsToggleButton;
use crate::ui::components::{
    EquipItemButton, GearHubBackdrop, GearHubCloseButton, PlaybackSpeedDecButton,
    PlaybackSpeedIncButton, ResetProgressButton, SalvageItemButton, SettingsModalBackdrop,
    SettingsModalCloseButton, SkillBookBackdrop, SkillShopBackdrop, SkillShopBuyButton,
    SkillShopCloseButton, StashSortCycleButton,
};
use crate::ui::interaction::registry::UiClickAction;

/// Button that received [`Interaction::Pressed`] on press; used to confirm click on mouse-up.
#[derive(Resource, Default)]
pub(crate) struct UiClickPress(pub Option<Entity>);

/// Pressed [`Button`] entities on the current mouse-down (consumed by [`capture_ui_click_start`]).
#[derive(Resource, Default)]
pub(crate) struct UiPressedButtonEntitiesOnClick(pub Option<Vec<Entity>>);

/// Modal roots currently present — used to filter pointer press targets away from briefing UI below.
#[derive(SystemParam)]
pub(crate) struct UiBlockingOverlayPresence<'w, 's> {
    settings: Query<'w, 's, Entity, With<crate::ui::components::SettingsModalRoot>>,
    skill_book: Query<'w, 's, Entity, With<crate::ui::components::SkillBookRoot>>,
    skill_shop: Query<'w, 's, Entity, With<crate::ui::components::SkillShopRoot>>,
    gear_hub: Query<'w, 's, Entity, With<crate::ui::components::GearHubRoot>>,
}

#[derive(SystemParam)]
pub(crate) struct UiClickResolveMarkers<'w, 's> {
    reset: Query<'w, 's, (), With<ResetProgressButton>>,
    playback_speed_dec: Query<'w, 's, (), With<PlaybackSpeedDecButton>>,
    playback_speed_inc: Query<'w, 's, (), With<PlaybackSpeedIncButton>>,
    settings_close: Query<'w, 's, (), With<SettingsModalCloseButton>>,
    settings_back: Query<'w, 's, (), With<SettingsModalBackdrop>>,
    #[cfg(debug_assertions)]
    presentation_settings_toggle: Query<'w, 's, (), With<PresentationEditorSettingsToggleButton>>,
    sbook_back: Query<'w, 's, (), With<SkillBookBackdrop>>,
    shop_buy: Query<'w, 's, (), With<SkillShopBuyButton>>,
    shop_close: Query<'w, 's, (), With<SkillShopCloseButton>>,
    shop_back: Query<'w, 's, (), With<SkillShopBackdrop>>,
    gear_back: Query<'w, 's, (), With<GearHubBackdrop>>,
    gear_close: Query<'w, 's, (), With<GearHubCloseButton>>,
    equip: Query<'w, 's, (), With<EquipItemButton>>,
    salvage: Query<'w, 's, (), With<SalvageItemButton>>,
    stash_sort: Query<'w, 's, (), With<StashSortCycleButton>>,
}

/// Returns true when `entity` is `root` or a descendant of `root` in the UI hierarchy.
pub(crate) fn entity_in_ui_subtree(
    entity: Entity,
    root: Entity,
    mut parent_of: impl FnMut(Entity) -> Option<Entity>,
) -> bool {
    let mut current = entity;
    loop {
        if current == root {
            return true;
        }
        let Some(parent) = parent_of(current) else {
            return false;
        };
        current = parent;
    }
}

fn retain_pressed_in_modal(
    pressed: &mut Vec<Entity>,
    modal_root: Entity,
    child_of: &Query<&ChildOf>,
) {
    pressed.retain(|&e| {
        entity_in_ui_subtree(e, modal_root, |ent| child_of.get(ent).ok().map(|c| c.parent()))
    });
}

pub(crate) fn capture_ui_pressed_button_entities(
    mouse: Res<ButtonInput<MouseButton>>,
    mut buf: ResMut<UiPressedButtonEntitiesOnClick>,
    buttons: Query<(Entity, &Interaction), With<Button>>,
) {
    if !mouse.just_pressed(MouseButton::Left) {
        buf.0 = None;
        return;
    }
    let pressed: Vec<Entity> = buttons
        .iter()
        .filter_map(|(e, i)| (*i == Interaction::Pressed).then_some(e))
        .collect();
    buf.0 = (!pressed.is_empty()).then_some(pressed);
}

pub(crate) fn capture_ui_click_start(
    mut press: ResMut<UiClickPress>,
    mut buf: ResMut<UiPressedButtonEntitiesOnClick>,
    overlay: UiBlockingOverlayPresence,
    child_of: Query<&ChildOf>,
    m: UiClickResolveMarkers,
) {
    let Some(mut pressed) = buf.0.take() else {
        return;
    };

    if pressed.is_empty() {
        press.0 = None;
        return;
    }

    if let Ok(modal) = overlay.settings.single() {
        retain_pressed_in_modal(&mut pressed, modal, &child_of);
        press.0 = pressed.iter().copied().min_by_key(|&e| {
            let tier = if m.presentation_settings_toggle.get(e).is_ok() {
                0u8
            } else if m.reset.get(e).is_ok() {
                1
            } else if m.settings_close.get(e).is_ok() {
                2
            } else if m.settings_back.get(e).is_ok() {
                3
            } else {
                255
            };
            (tier, e.to_bits())
        });
        return;
    }

    if let Ok(modal) = overlay.skill_book.single() {
        retain_pressed_in_modal(&mut pressed, modal, &child_of);
        press.0 = pressed.iter().copied().min_by_key(|&e| {
            let tier = if m.sbook_back.get(e).is_ok() { 2u8 } else { 0 };
            (tier, e.to_bits())
        });
        return;
    }

    if let Ok(modal) = overlay.skill_shop.single() {
        retain_pressed_in_modal(&mut pressed, modal, &child_of);
        press.0 = pressed.iter().copied().min_by_key(|&e| {
            let tier = if m.shop_buy.get(e).is_ok() {
                0u8
            } else if m.shop_close.get(e).is_ok() {
                1
            } else if m.shop_back.get(e).is_ok() {
                2
            } else {
                255
            };
            (tier, e.to_bits())
        });
        return;
    }

    if let Ok(modal) = overlay.gear_hub.single() {
        retain_pressed_in_modal(&mut pressed, modal, &child_of);
        press.0 = pressed.iter().copied().min_by_key(|&e| {
            let tier = if m.equip.get(e).is_ok() {
                0u8
            } else if m.salvage.get(e).is_ok() {
                1
            } else if m.stash_sort.get(e).is_ok() {
                2
            } else if m.gear_close.get(e).is_ok() {
                3
            } else if m.gear_back.get(e).is_ok() {
                4
            } else {
                255
            };
            (tier, e.to_bits())
        });
        return;
    }

    press.0 = pressed
        .iter()
        .copied()
        .filter(|&e| m.playback_speed_dec.get(e).is_ok() || m.playback_speed_inc.get(e).is_ok())
        .min_by_key(|e| e.to_bits())
        .or_else(|| pressed.iter().min_by_key(|e| e.to_bits()).copied());
}

/// Bevy UI's `ui_focus_system` may leave [`Interaction::Pressed`] on the frame where the
/// mouse button is released if press and release occur in the same update (common with quick taps).
/// After a longer hold, release instead becomes [`Interaction::Hovered`].
/// Bevy can briefly flip a control to [`Interaction::None`] on mouse-up before hover restabilizes,
/// especially with nested UI hit targets—still treat release as confirming if [`UiClickPress`]
/// captured this entity on mouse-down (callers gate on matching `target`).
#[inline]
pub(crate) fn ui_click_release_confirms(interaction: Interaction) -> bool {
    matches!(
        interaction,
        Interaction::Hovered | Interaction::Pressed | Interaction::None
    )
}

pub(crate) fn resolve_clicked_action(
    mouse: &ButtonInput<MouseButton>,
    press: &UiClickPress,
    clicked: &Query<(Entity, &Interaction, &crate::ui::interaction::UiClickAction)>,
) -> Option<crate::ui::interaction::UiClickAction> {
    if !mouse.just_released(MouseButton::Left) {
        return None;
    }
    let target = press.0?;
    clicked.iter().find_map(|(entity, interaction, action)| {
        (entity == target && ui_click_release_confirms(*interaction)).then_some(*action)
    })
}

pub(crate) fn clear_ui_click_after_release(
    mouse: Res<ButtonInput<MouseButton>>,
    mut press: ResMut<UiClickPress>,
) {
    if mouse.just_released(MouseButton::Left) {
        press.0 = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn entity_in_ui_subtree_walks_parent_chain() {
        let root = Entity::from_bits(1);
        let mid = Entity::from_bits(2);
        let leaf = Entity::from_bits(3);
        let parents = |e: Entity| match e {
            e if e == leaf => Some(mid),
            e if e == mid => Some(root),
            _ => None,
        };
        assert!(entity_in_ui_subtree(leaf, root, parents));
        assert!(entity_in_ui_subtree(mid, root, parents));
        assert!(entity_in_ui_subtree(root, root, parents));
        assert!(!entity_in_ui_subtree(Entity::from_bits(99), root, parents));
    }
}
