use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SkillId {
    LifestealStrike,
    Guard,
    HeavyStrike,
    PoisonEdge,
    ThornSkin,
    BarrierPulse,
    Cleave,
    Taunt,
    SecondWind,
    ToxicMastery,
    VampiricAura,
    ThickHide,
    ArcaneOverflow,
    Berserker,
    SwiftStrikes,
    IronWill,
    BattleFocus,
    CautiousAdvance,
    LuckyStrike,
    Predator,
}

impl SkillId {
    pub const ALL: &'static [SkillId] = &[
        SkillId::LifestealStrike,
        SkillId::Guard,
        SkillId::HeavyStrike,
        SkillId::PoisonEdge,
        SkillId::ThornSkin,
        SkillId::BarrierPulse,
        SkillId::Cleave,
        SkillId::Taunt,
        SkillId::SecondWind,
        SkillId::ToxicMastery,
        SkillId::VampiricAura,
        SkillId::ThickHide,
        SkillId::ArcaneOverflow,
        SkillId::Berserker,
        SkillId::SwiftStrikes,
        SkillId::IronWill,
        SkillId::BattleFocus,
        SkillId::CautiousAdvance,
        SkillId::LuckyStrike,
        SkillId::Predator,
    ];
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SkillKind {
    Active,
    Passive,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SkillTag {
    Attack,
    Defense,
    Healing,
    Poison,
    Barrier,
    Thorns,
    Melee,
    Ranged,
    AoE,
    Threat,
    Sustain,
    Reactive,
    Critical,
    Curse,
    Summon,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SkillTrigger {
    OnAttack,
    OnHitTaken,
    OnRoomStart,
    PeriodicTick,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct SkillDefinition {
    pub id: SkillId,
    pub kind: SkillKind,
    pub name: &'static str,
    pub trigger: SkillTrigger,
    pub tags: &'static [SkillTag],
    pub description: &'static str,
    /// Short hint for buildcraft UI (synergies / pairings).
    pub synergy_hint: &'static str,
    /// Simulated combat "wind-up" ticks before an [`SkillTrigger::OnAttack`] strike lands (`0` = instant).
    pub cast_ticks: u8,
    /// Simulated combat ticks after a strike before the next [`SkillTrigger::OnAttack`] cycle starts.
    pub cooldown_ticks: u8,
}

pub fn format_skill_tags(tags: &[SkillTag]) -> String {
    tags.iter()
        .map(|t| format!("{t:?}"))
        .collect::<Vec<_>>()
        .join(", ")
}

/// Skill book list: actives first, then passives, alphabetical by name within each group.
pub fn skill_book_pick_order() -> impl Iterator<Item = SkillId> {
    let mut v: Vec<SkillId> = SkillId::ALL.iter().copied().collect();
    v.sort_by_key(|id| {
        let d = skill_definition(*id);
        (
            matches!(d.kind, SkillKind::Passive),
            d.name.to_ascii_lowercase(),
        )
    });
    v.into_iter()
}

/// Wind-up and cooldown lengths in combat simulation ticks (see combat `attack_cadence_ticks`).
pub fn skill_timings(id: SkillId) -> (u8, u8) {
    match id {
        SkillId::LifestealStrike => (0, 3),
        SkillId::Guard => (0, 0),
        SkillId::HeavyStrike => (2, 4),
        SkillId::PoisonEdge => (0, 2),
        SkillId::ThornSkin => (0, 0),
        SkillId::BarrierPulse => (0, 0),
        SkillId::Cleave => (2, 4),
        SkillId::Taunt => (0, 3),
        SkillId::SecondWind => (0, 0),
        SkillId::ToxicMastery => (0, 0),
        SkillId::VampiricAura => (0, 0),
        SkillId::ThickHide => (0, 0),
        SkillId::ArcaneOverflow => (0, 0),
        SkillId::Berserker => (0, 0),
        SkillId::SwiftStrikes => (0, 0),
        SkillId::IronWill => (0, 0),
        SkillId::BattleFocus => (0, 0),
        SkillId::CautiousAdvance => (0, 0),
        SkillId::LuckyStrike => (0, 0),
        SkillId::Predator => (0, 0),
    }
}

/// Skills in a new hero book before guild purchases (others unlock in the skill shop).
pub const STARTER_SKILLS: &[SkillId] = &[
    SkillId::LifestealStrike,
    SkillId::Guard,
    SkillId::HeavyStrike,
    SkillId::SecondWind,
];

/// Gold to permanently add a skill to the library (`None` = starter or not sold).
pub fn skill_shop_price_gold(id: SkillId) -> Option<u32> {
    if STARTER_SKILLS.contains(&id) {
        return None;
    }
    Some(match id {
        SkillId::PoisonEdge => 40,
        SkillId::ThornSkin => 45,
        SkillId::BarrierPulse => 35,
        SkillId::Cleave => 50,
        SkillId::Taunt => 30,
        SkillId::ToxicMastery => 55,
        SkillId::VampiricAura => 50,
        SkillId::ThickHide => 45,
        SkillId::ArcaneOverflow => 60,
        SkillId::Berserker => 55,
        SkillId::SwiftStrikes => 65,
        SkillId::IronWill => 50,
        SkillId::BattleFocus => 40,
        SkillId::CautiousAdvance => 45,
        SkillId::LuckyStrike => 35,
        SkillId::Predator => 40,
        SkillId::LifestealStrike | SkillId::Guard | SkillId::HeavyStrike | SkillId::SecondWind => {
            return None;
        }
    })
}

/// Filtered library pick order for the player's unlocked skills.
pub fn skill_book_pick_order_for(unlocked: &[SkillId]) -> Vec<SkillId> {
    use std::collections::HashSet;
    let set: HashSet<SkillId> = unlocked.iter().copied().collect();
    skill_book_pick_order()
        .filter(|id| set.contains(id))
        .collect()
}

pub fn skill_definition(id: SkillId) -> SkillDefinition {
    let (c, d) = skill_timings(id);
    match id {
        SkillId::LifestealStrike => SkillDefinition {
            id,
            kind: SkillKind::Active,
            name: "Lifesteal Strike",
            trigger: SkillTrigger::OnAttack,
            tags: &[SkillTag::Attack, SkillTag::Melee, SkillTag::Healing, SkillTag::Sustain],
            description: "Attacks restore a small amount of health.",
            synergy_hint: "Pairs with attack speed and damage; Vampiric Aura amplifies sustain.",
            cast_ticks: c,
            cooldown_ticks: d,
        },
        SkillId::Guard => SkillDefinition {
            id,
            kind: SkillKind::Active,
            name: "Guard",
            trigger: SkillTrigger::OnHitTaken,
            tags: &[SkillTag::Defense, SkillTag::Reactive, SkillTag::Melee],
            description: "Reduces incoming damage. Strength scales with healing power.",
            synergy_hint: "Bastion gear adds flat block when this skill is equipped; armor and healing power still scale the kit.",
            cast_ticks: c,
            cooldown_ticks: d,
        },
        SkillId::HeavyStrike => SkillDefinition {
            id,
            kind: SkillKind::Active,
            name: "Heavy Strike",
            trigger: SkillTrigger::OnAttack,
            tags: &[SkillTag::Attack, SkillTag::Melee],
            description: "Slow attacks hit harder.",
            synergy_hint: "Heavy weapon affixes further raise burst; avoid pairing with extreme attack speed for now.",
            cast_ticks: c,
            cooldown_ticks: d,
        },
        SkillId::PoisonEdge => SkillDefinition {
            id,
            kind: SkillKind::Active,
            name: "Poison Edge",
            trigger: SkillTrigger::OnAttack,
            tags: &[SkillTag::Attack, SkillTag::Melee, SkillTag::Poison],
            description: "Attacks stack poison; it ticks each moment for more damage at higher stacks (capped).",
            synergy_hint: "Toxic Mastery and Virulent gear deepen stacks faster for stronger poison ticks.",
            cast_ticks: c,
            cooldown_ticks: d,
        },
        SkillId::ThornSkin => SkillDefinition {
            id,
            kind: SkillKind::Active,
            name: "Thorn Skin",
            trigger: SkillTrigger::OnHitTaken,
            tags: &[SkillTag::Defense, SkillTag::Thorns, SkillTag::Reactive],
            description: "Reflects damage when hit.",
            synergy_hint: "Spiked affix and high inbound hit volume increase reflect value.",
            cast_ticks: c,
            cooldown_ticks: d,
        },
        SkillId::BarrierPulse => SkillDefinition {
            id,
            kind: SkillKind::Active,
            name: "Barrier Pulse",
            trigger: SkillTrigger::OnRoomStart,
            tags: &[SkillTag::Defense, SkillTag::Barrier],
            description: "Starts each room with a barrier.",
            synergy_hint: "Scales with healing power; later item effects may convert or reflect barrier.",
            cast_ticks: c,
            cooldown_ticks: d,
        },
        SkillId::Cleave => SkillDefinition {
            id,
            kind: SkillKind::Active,
            name: "Cleave",
            trigger: SkillTrigger::OnAttack,
            tags: &[SkillTag::Attack, SkillTag::Melee, SkillTag::AoE],
            description: "Wide swings hit like a heavy strike—slow pace, high impact (single target for now).",
            synergy_hint: "Placeholder for future multi-foe rooms; build like Heavy Strike today.",
            cast_ticks: c,
            cooldown_ticks: d,
        },
        SkillId::Taunt => SkillDefinition {
            id,
            kind: SkillKind::Active,
            name: "Taunt",
            trigger: SkillTrigger::OnAttack,
            tags: &[SkillTag::Threat, SkillTag::Melee, SkillTag::Reactive],
            description: "Training skill for future threat: currently no combat effect until party AI ships.",
            synergy_hint: "Will pair with tank items and passive threat auras (Phase 3).",
            cast_ticks: c,
            cooldown_ticks: d,
        },
        SkillId::SecondWind => SkillDefinition {
            id,
            kind: SkillKind::Passive,
            name: "Second Wind",
            trigger: SkillTrigger::OnRoomStart,
            tags: &[SkillTag::Sustain, SkillTag::Healing],
            description: "Recover a small fraction of max health at the start of each fight.",
            synergy_hint: "Stronger on high max-health builds; complements Barrier Pulse.",
            cast_ticks: c,
            cooldown_ticks: d,
        },
        SkillId::ToxicMastery => SkillDefinition {
            id,
            kind: SkillKind::Passive,
            name: "Toxic Mastery",
            trigger: SkillTrigger::PeriodicTick,
            tags: &[SkillTag::Poison, SkillTag::Sustain],
            description: "Poison ticks deal increased damage.",
            synergy_hint: "Requires Poison Edge or another poison applicator to shine.",
            cast_ticks: c,
            cooldown_ticks: d,
        },
        SkillId::VampiricAura => SkillDefinition {
            id,
            kind: SkillKind::Passive,
            name: "Vampiric Aura",
            trigger: SkillTrigger::OnAttack,
            tags: &[SkillTag::Healing, SkillTag::Sustain],
            description: "Slightly improves all healing from attacks (including lifesteal).",
            synergy_hint: "Stacks with Lifesteal Strike and healing power.",
            cast_ticks: c,
            cooldown_ticks: d,
        },
        SkillId::ThickHide => SkillDefinition {
            id,
            kind: SkillKind::Passive,
            name: "Thick Hide",
            trigger: SkillTrigger::OnHitTaken,
            tags: &[SkillTag::Defense],
            description: "Passive bonus armor.",
            synergy_hint: "Flat mitigation layer before Guard reductions.",
            cast_ticks: c,
            cooldown_ticks: d,
        },
        SkillId::ArcaneOverflow => SkillDefinition {
            id,
            kind: SkillKind::Passive,
            name: "Arcane Overflow",
            trigger: SkillTrigger::PeriodicTick,
            tags: &[SkillTag::Healing],
            description: "Bonus healing power—barriers, Guard scaling, and heals tick up.",
            synergy_hint: "Synergises with Barrier Pulse and Guard.",
            cast_ticks: c,
            cooldown_ticks: d,
        },
        SkillId::Berserker => SkillDefinition {
            id,
            kind: SkillKind::Passive,
            name: "Berserker",
            trigger: SkillTrigger::OnAttack,
            tags: &[SkillTag::Attack, SkillTag::Melee],
            description: "More damage, less armor—glass cannon leaning.",
            synergy_hint: "Offset with Thick Hide or Barrier Pulse.",
            cast_ticks: c,
            cooldown_ticks: d,
        },
        SkillId::SwiftStrikes => SkillDefinition {
            id,
            kind: SkillKind::Passive,
            name: "Swift Strikes",
            trigger: SkillTrigger::OnAttack,
            tags: &[SkillTag::Attack, SkillTag::Melee, SkillTag::Critical],
            description: "Slightly higher attack speed.",
            synergy_hint: "Multiplies poison applications and lifesteal cadence.",
            cast_ticks: c,
            cooldown_ticks: d,
        },
        SkillId::IronWill => SkillDefinition {
            id,
            kind: SkillKind::Passive,
            name: "Iron Will",
            trigger: SkillTrigger::OnHitTaken,
            tags: &[SkillTag::Defense, SkillTag::Sustain],
            description: "Bonus maximum health.",
            synergy_hint: "Pairs with percentage-based heals like Second Wind.",
            cast_ticks: c,
            cooldown_ticks: d,
        },
        SkillId::BattleFocus => SkillDefinition {
            id,
            kind: SkillKind::Passive,
            name: "Battle Focus",
            trigger: SkillTrigger::OnAttack,
            tags: &[SkillTag::Attack, SkillTag::Melee],
            description: "Small flat weapon damage bonus.",
            synergy_hint: "Builds toward armor-shred encounters; combine with Cleave/Heavy cadence.",
            cast_ticks: c,
            cooldown_ticks: d,
        },
        SkillId::CautiousAdvance => SkillDefinition {
            id,
            kind: SkillKind::Passive,
            name: "Cautious Advance",
            trigger: SkillTrigger::OnHitTaken,
            tags: &[SkillTag::Defense, SkillTag::Melee],
            description: "Much more armor, slightly less damage—safe front-liner stats.",
            synergy_hint: "Future threat kit will reward this profile.",
            cast_ticks: c,
            cooldown_ticks: d,
        },
        SkillId::LuckyStrike => SkillDefinition {
            id,
            kind: SkillKind::Passive,
            name: "Lucky Strike",
            trigger: SkillTrigger::OnAttack,
            tags: &[SkillTag::Attack, SkillTag::Critical],
            description: "Placeholder for critical-hit systems (stub—no combat effect yet).",
            synergy_hint: "Awaiting crit pipelines and telemetry (Phase 5).",
            cast_ticks: c,
            cooldown_ticks: d,
        },
        SkillId::Predator => SkillDefinition {
            id,
            kind: SkillKind::Passive,
            name: "Predator",
            trigger: SkillTrigger::OnAttack,
            tags: &[SkillTag::Attack, SkillTag::Ranged],
            description: "Placeholder for elite-hunter bonuses (stub—no combat effect yet).",
            synergy_hint: "Will hook into encounter metadata when multi-enemy packs arrive.",
            cast_ticks: c,
            cooldown_ticks: d,
        },
    }
}
