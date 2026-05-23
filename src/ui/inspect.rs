//! Inspect region policy — fixed strips replace cursor tooltips on camp surfaces and modals.
//!
//! ## Sync pattern
//!
//! 1. **`sync_ui_inspect_hover`** collects `Interaction::Hovered` from marked sources and writes
//!    [`UiInspectState`]. Skill slots, gear, and catalogue rows beat generic control hints.
//! 2. **`sync_ui_inspect_panel`** maps the target to [`InspectPanelContent`] and updates the
//!    active scoped strip (`InspectRegion` + [`InspectRegionScope`]).
//! 3. Party Buildcraft keeps its own full inspect panel while `SkillBookRoot` is open.

use bevy::prelude::*;

use crate::app::ProfileState;
use crate::domain::items::GearSlot;
use crate::domain::party::PartyHeroKind;
use crate::domain::skills::SkillId;
use crate::ui::assets::UiPlaceholderImages;
use crate::ui::components::{
    BuildScreen, GearHubRoot, RunPlaybackScreen, SkillBookRoot, SkillShopRoot, SummaryScreen,
};
use crate::ui::primitives::inspect_panel::{
    inspect_content_camp_slot, inspect_content_gear_item, inspect_content_gear_slot,
    inspect_content_hint, inspect_content_none_for_scope, inspect_content_skill_shop,
    CampInspectBody, CampInspectHint, CampInspectIcon, CampInspectMeta, CampInspectTags,
    CampInspectTitle, InspectStripScope,
};
use crate::ui::primitives::skill_bar::SkillBarSlot;

/// Marks stable inspect real estate (see `docs/ui-design-system.md` §4).
#[derive(Component, Clone, Copy, Debug)]
pub struct InspectRegion;

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub enum InspectRegionScope {
    Camp,
    GearHub,
    SkillShop,
}

/// Generic one-line control hint — footer buttons, header chips, modal chrome.
#[derive(Component, Clone, Copy, Debug)]
pub struct InspectHint(pub &'static str);

#[derive(Component, Clone, Copy, Debug)]
pub struct GearSlotInspect(pub GearSlot);

#[derive(Component, Clone, Copy, Debug)]
pub struct StashItemInspect {
    pub item_id: u64,
}

#[derive(Component, Clone, Copy, Debug)]
pub struct SkillShopInspect(pub SkillId);

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum UiInspectTarget {
    #[default]
    None,
    SkillSlot {
        hero: PartyHeroKind,
        index: usize,
    },
    GearSlot(GearSlot),
    StashItem(u64),
    SkillShop(SkillId),
    Hint(String),
}

#[derive(Resource, Default)]
pub struct UiInspectState {
    pub target: UiInspectTarget,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum HoverPriority {
    Hint = 1,
    SkillShop = 2,
    GearSlot = 3,
    StashItem = 4,
    SkillSlot = 5,
}

struct HoverCandidate {
    priority: HoverPriority,
    target: UiInspectTarget,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ActiveInspectScope {
    None,
    Camp,
    GearHub,
    SkillShop,
}

pub fn sync_ui_inspect_hover(
    build: Query<(), With<BuildScreen>>,
    summary: Query<(), With<SummaryScreen>>,
    running: Query<(), With<RunPlaybackScreen>>,
    gear: Query<(), With<GearHubRoot>>,
    shop: Query<(), With<SkillShopRoot>>,
    book: Query<(), With<SkillBookRoot>>,
    mut state: ResMut<UiInspectState>,
    slots: Query<(&SkillBarSlot, &Interaction)>,
    gear_slots: Query<(&GearSlotInspect, &Interaction)>,
    stash_items: Query<(&StashItemInspect, &Interaction)>,
    shop_skills: Query<(&SkillShopInspect, &Interaction)>,
    hints: Query<(&InspectHint, &Interaction)>,
) {
    if !book.is_empty() {
        return;
    }
    let scope = active_scope(build, summary, running, gear, shop);
    if scope == ActiveInspectScope::None {
        if state.target != UiInspectTarget::None {
            state.target = UiInspectTarget::None;
        }
        return;
    }

    let mut best: Option<HoverCandidate> = None;
    let mut consider = |priority: HoverPriority, target: UiInspectTarget| {
        if best.as_ref().is_none_or(|b| priority > b.priority) {
            best = Some(HoverCandidate { priority, target });
        }
    };

    if matches!(scope, ActiveInspectScope::Camp) {
        for (slot, interaction) in &slots {
            if *interaction == Interaction::Hovered {
                consider(
                    HoverPriority::SkillSlot,
                    UiInspectTarget::SkillSlot {
                        hero: slot.hero,
                        index: slot.index,
                    },
                );
            }
        }
    }

    if matches!(scope, ActiveInspectScope::GearHub) {
        for (slot, interaction) in &gear_slots {
            if *interaction == Interaction::Hovered {
                consider(HoverPriority::GearSlot, UiInspectTarget::GearSlot(slot.0));
            }
        }
        for (item, interaction) in &stash_items {
            if *interaction == Interaction::Hovered {
                consider(HoverPriority::StashItem, UiInspectTarget::StashItem(item.item_id));
            }
        }
    }

    if matches!(scope, ActiveInspectScope::SkillShop) {
        for (skill, interaction) in &shop_skills {
            if *interaction == Interaction::Hovered {
                consider(HoverPriority::SkillShop, UiInspectTarget::SkillShop(skill.0));
            }
        }
    }

    for (hint, interaction) in &hints {
        if *interaction == Interaction::Hovered {
            consider(
                HoverPriority::Hint,
                UiInspectTarget::Hint(hint.0.to_string()),
            );
        }
    }

    let target = best.map(|b| b.target).unwrap_or(UiInspectTarget::None);
    if state.target != target {
        state.target = target;
    }
}

pub fn sync_ui_inspect_panel(
    build: Query<(), With<BuildScreen>>,
    summary: Query<(), With<SummaryScreen>>,
    running: Query<(), With<RunPlaybackScreen>>,
    gear: Query<(), With<GearHubRoot>>,
    shop: Query<(), With<SkillShopRoot>>,
    book: Query<(), With<SkillBookRoot>>,
    state: Res<UiInspectState>,
    profile: Res<ProfileState>,
    ph: Res<UiPlaceholderImages>,
    regions: Query<&InspectRegionScope, With<InspectRegion>>,
    mut icon: Query<(&InspectStripScope, &mut ImageNode), With<CampInspectIcon>>,
    mut texts: ParamSet<(
        Query<(&InspectStripScope, &mut Text), With<CampInspectTitle>>,
        Query<(&InspectStripScope, &mut Text), With<CampInspectMeta>>,
        Query<(&InspectStripScope, &mut Text), With<CampInspectTags>>,
        Query<(&InspectStripScope, &mut Text), With<CampInspectBody>>,
        Query<(&InspectStripScope, &mut Text), With<CampInspectHint>>,
    )>,
) {
    if !book.is_empty() {
        return;
    }
    let scope = active_scope(build, summary, running, gear, shop);
    let region_scope = match scope {
        ActiveInspectScope::Camp => InspectRegionScope::Camp,
        ActiveInspectScope::GearHub => InspectRegionScope::GearHub,
        ActiveInspectScope::SkillShop => InspectRegionScope::SkillShop,
        ActiveInspectScope::None => return,
    };
    if regions.iter().all(|s| *s != region_scope) {
        return;
    }
    if !state.is_changed() && !profile.is_changed() {
        return;
    }

    let interactive_skills = !build.is_empty();
    let gold = profile.profile.meta.gold;
    let content = resolve_inspect_content(
        &state.target,
        &profile,
        interactive_skills,
        gold,
        region_scope,
        &ph,
    );

    for (strip, mut icon) in &mut icon {
        if strip.0 != region_scope {
            continue;
        }
        icon.image = content.icon_image.clone();
        icon.color = content.icon_color;
        break;
    }
    for (strip, mut text) in texts.p0().iter_mut() {
        if strip.0 == region_scope {
            text.0 = content.title.clone();
        }
    }
    for (strip, mut text) in texts.p1().iter_mut() {
        if strip.0 == region_scope {
            text.0 = content.meta.clone();
        }
    }
    for (strip, mut text) in texts.p2().iter_mut() {
        if strip.0 == region_scope {
            text.0 = content.tags.clone();
        }
    }
    for (strip, mut text) in texts.p3().iter_mut() {
        if strip.0 == region_scope {
            text.0 = content.body.clone();
        }
    }
    for (strip, mut text) in texts.p4().iter_mut() {
        if strip.0 == region_scope {
            text.0 = content.hint.clone();
        }
    }
}

fn active_scope(
    build: Query<(), With<BuildScreen>>,
    summary: Query<(), With<SummaryScreen>>,
    running: Query<(), With<RunPlaybackScreen>>,
    gear: Query<(), With<GearHubRoot>>,
    shop: Query<(), With<SkillShopRoot>>,
) -> ActiveInspectScope {
    if !gear.is_empty() {
        return ActiveInspectScope::GearHub;
    }
    if !shop.is_empty() {
        return ActiveInspectScope::SkillShop;
    }
    if !build.is_empty() || !summary.is_empty() || !running.is_empty() {
        return ActiveInspectScope::Camp;
    }
    ActiveInspectScope::None
}

fn resolve_inspect_content(
    target: &UiInspectTarget,
    profile: &ProfileState,
    interactive_skills: bool,
    gold: u32,
    scope: InspectRegionScope,
    ph: &UiPlaceholderImages,
) -> crate::ui::primitives::inspect_panel::InspectPanelContent {
    match target {
        UiInspectTarget::None => inspect_content_none_for_scope(scope, ph),
        UiInspectTarget::SkillSlot { hero, index } => {
            let skill = skill_at_slot(profile, *hero, *index);
            let mut content = inspect_content_camp_slot(*hero, *index, skill, ph);
            if !interactive_skills {
                content.hint = "Priority runs left to right when multiple skills are ready."
                    .to_string();
            }
            content
        }
        UiInspectTarget::GearSlot(slot) => {
            let main_two_handed = profile
                .profile
                .hero
                .equipped_item(GearSlot::MainHand)
                .is_some_and(|i| i.two_handed);
            let blocked_off_hand = *slot == GearSlot::OffHand && main_two_handed;
            let item = profile.profile.hero.equipped_item(*slot);
            inspect_content_gear_slot(*slot, item, blocked_off_hand, ph)
        }
        UiInspectTarget::StashItem(item_id) => profile
            .profile
            .inventory
            .iter()
            .find(|i| i.id == *item_id)
            .map(|item| inspect_content_gear_item(item, ph, "Equip or salvage from action buttons."))
            .unwrap_or_else(|| inspect_content_none_for_scope(scope, ph)),
        UiInspectTarget::SkillShop(id) => inspect_content_skill_shop(*id, gold, ph),
        UiInspectTarget::Hint(text) => inspect_content_hint("Control", text, ""),
    }
}

fn skill_at_slot(
    profile: &ProfileState,
    hero: PartyHeroKind,
    index: usize,
) -> Option<SkillId> {
    match hero {
        PartyHeroKind::Player1 => profile
            .effective_hero()
            .equipped_skills
            .get(index)
            .and_then(|s| *s),
        PartyHeroKind::Player2 => profile
            .effective_party_partner()
            .and_then(|h| h.equipped_skills.get(index).and_then(|s| *s)),
    }
}
