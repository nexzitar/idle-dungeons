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
    EmpoweredBlow,
    VictoryRush,
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
        SkillId::EmpoweredBlow,
        SkillId::VictoryRush,
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

/// How a skill interacts with the weapon timeline and the shared ability global cooldown.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SkillCombatStyle {
    /// Passives and actives that do not merge into the heavy-style swing cadence.
    Passive,
    /// Heavy / Cleave / … — share one wind-up + post-weapon cooldown channel.
    SwingWeave,
    /// Heroic Strike–style: queue bonus damage on the next white swing; queues trigger [`SkillDefinition::gcd_ticks`].
    NextMeleeBuff,
    /// Victory Rush–style: instant ability damage; uses [`SkillDefinition::gcd_ticks`] and charge rules from
    /// [`SkillDefinition::max_charges`] + [`SkillDefinition::ability_icd_ticks`].
    InstantStrike,
}

/// Player-facing **combat taxonomy** for UI, tooltips, and future layering rules.
/// Orthogonal to [`SkillCombatStyle`] (sim scheduler) and [`SkillKind`] (active/passive).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SkillCategory {
    /// Weapon white swings only (not a [`SkillId`]); kept for legends / future hooks.
    BasicAttack,
    /// Actives that ride the weapon swing cadence or fire as dedicated attack abilities (incl. instant strikes).
    AttackSkill,
    /// Protective windows, next-hit queues, room shields.
    Buff,
    /// Mitigation / reflect tied to inbound hits (actives).
    Reactive,
    /// Always-on or periodic modifiers.
    Passive,
    /// Reserved for channeled skills (none equipped yet).
    Channel,
    /// Crit / proc-style passives (may be stubs until pipelines land).
    Proc,
}

impl SkillCategory {
    pub const fn display_label(self) -> &'static str {
        match self {
            SkillCategory::BasicAttack => "Basic attack",
            SkillCategory::AttackSkill => "Attack skill",
            SkillCategory::Buff => "Buff",
            SkillCategory::Reactive => "Reactive",
            SkillCategory::Passive => "Passive",
            SkillCategory::Channel => "Channel",
            SkillCategory::Proc => "Proc",
        }
    }
}

/// Category for build UI and layering policy; derived from id + catalog fields.
pub fn skill_category(id: SkillId) -> SkillCategory {
    use SkillId::*;
    match id {
        LuckyStrike | Predator => SkillCategory::Proc,
        Guard | ThornSkin => SkillCategory::Reactive,
        BarrierPulse | EmpoweredBlow => SkillCategory::Buff,
        HeavyStrike | Cleave | PoisonEdge | Taunt | LifestealStrike | VictoryRush => {
            SkillCategory::AttackSkill
        }
        SecondWind | ToxicMastery | VampiricAura | ThickHide | ArcaneOverflow | Berserker
        | SwiftStrikes | IronWill | BattleFocus | CautiousAdvance => SkillCategory::Passive,
    }
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
    /// Classification for swing merge, instant strikes, and next-melee buffs.
    pub combat_style: SkillCombatStyle,
    /// Ability GCD length in combat ticks when this skill uses the shared ability clock (`0` = none).
    pub gcd_ticks: u8,
    /// For [`SkillCombatStyle::InstantStrike`]: **recharge interval** in combat ticks between **gaining** each
    /// stored charge (e.g. “5 s cooldown” → 50 ticks at 100 ms/tick). One charge is restored per pulse while below
    /// [`Self::max_charges`]; spending a charge starts this timer **only if** no recharge is already in progress.
    /// Other combat styles may use this field as an extra per-cast gate where wired (often `0`).
    pub ability_icd_ticks: u8,
    /// For [`SkillCombatStyle::InstantStrike`]: maximum stored charges (encounter starts **full**). Each cast
    /// needs a charge and the ability GCD. Ignored for other styles (keep at `1`).
    pub max_charges: u8,
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
        SkillId::EmpoweredBlow => (0, 0),
        SkillId::VictoryRush => (0, 0),
        SkillId::Predator => (0, 0),
    }
}

pub fn skill_combat_meta(id: SkillId) -> (SkillCombatStyle, u8, u8, u8) {
    use SkillCombatStyle::*;
    match id {
        SkillId::HeavyStrike | SkillId::Cleave | SkillId::PoisonEdge | SkillId::Taunt => {
            (SwingWeave, 0, 0, 1)
        }
        SkillId::EmpoweredBlow => (NextMeleeBuff, 3, 8, 1),
        SkillId::VictoryRush => (InstantStrike, 3, 30, 1),
        _ => (Passive, 0, 0, 1),
    }
}

/// Whether this style advances the **shared ability GCD** (instant strikes and next-melee buff queues).
/// [`SkillCombatStyle::SwingWeave`] uses the weapon wind-up / post-swing cooldown only.
pub fn skill_triggers_shared_ability_gcd(style: SkillCombatStyle) -> bool {
    matches!(
        style,
        SkillCombatStyle::InstantStrike | SkillCombatStyle::NextMeleeBuff
    )
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
        SkillId::EmpoweredBlow => 45,
        SkillId::VictoryRush => 55,
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
    let (combat_style, gcd_ticks, ability_icd_ticks, max_charges) = skill_combat_meta(id);
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
            combat_style,
            gcd_ticks,
            ability_icd_ticks,
            max_charges,
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
            combat_style,
            gcd_ticks,
            ability_icd_ticks,
            max_charges,
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
            combat_style,
            gcd_ticks,
            ability_icd_ticks,
            max_charges,
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
            combat_style,
            gcd_ticks,
            ability_icd_ticks,
            max_charges,
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
            combat_style,
            gcd_ticks,
            ability_icd_ticks,
            max_charges,
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
            combat_style,
            gcd_ticks,
            ability_icd_ticks,
            max_charges,
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
            combat_style,
            gcd_ticks,
            ability_icd_ticks,
            max_charges,
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
            combat_style,
            gcd_ticks,
            ability_icd_ticks,
            max_charges,
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
            combat_style,
            gcd_ticks,
            ability_icd_ticks,
            max_charges,
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
            combat_style,
            gcd_ticks,
            ability_icd_ticks,
            max_charges,
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
            combat_style,
            gcd_ticks,
            ability_icd_ticks,
            max_charges,
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
            combat_style,
            gcd_ticks,
            ability_icd_ticks,
            max_charges,
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
            combat_style,
            gcd_ticks,
            ability_icd_ticks,
            max_charges,
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
            combat_style,
            gcd_ticks,
            ability_icd_ticks,
            max_charges,
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
            combat_style,
            gcd_ticks,
            ability_icd_ticks,
            max_charges,
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
            combat_style,
            gcd_ticks,
            ability_icd_ticks,
            max_charges,
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
            combat_style,
            gcd_ticks,
            ability_icd_ticks,
            max_charges,
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
            combat_style,
            gcd_ticks,
            ability_icd_ticks,
            max_charges,
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
            combat_style,
            gcd_ticks,
            ability_icd_ticks,
            max_charges,
        },
        SkillId::EmpoweredBlow => SkillDefinition {
            id,
            kind: SkillKind::Active,
            name: "Empowered Blow",
            trigger: SkillTrigger::OnAttack,
            tags: &[SkillTag::Attack, SkillTag::Melee],
            description: "Queue bonus ability damage on your next white swing—costs a global cooldown to line up.",
            synergy_hint: "Weaves between instant strikes; pairs with fast autos and heavy hitters.",
            cast_ticks: c,
            cooldown_ticks: d,
            combat_style,
            gcd_ticks,
            ability_icd_ticks,
            max_charges,
        },
        SkillId::VictoryRush => SkillDefinition {
            id,
            kind: SkillKind::Active,
            name: "Victory Rush",
            trigger: SkillTrigger::OnAttack,
            tags: &[SkillTag::Attack, SkillTag::Melee, SkillTag::Sustain],
            description: "Instant ability strike—hits for yellow damage on its own beat, respects GCD and a longer cooldown.",
            synergy_hint: "Strong opener filler; track ICD so it does not collide with Empowered Blow timing.",
            cast_ticks: c,
            cooldown_ticks: d,
            combat_style,
            gcd_ticks,
            ability_icd_ticks,
            max_charges,
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
            combat_style,
            gcd_ticks,
            ability_icd_ticks,
            max_charges,
        },
    }
}

#[cfg(test)]
mod combat_style_tests {
    use super::{
        skill_category, skill_combat_meta, skill_definition, skill_triggers_shared_ability_gcd,
        SkillCategory, SkillId,
    };

    #[test]
    fn skill_category_maps_every_id_consistently() {
        for id in SkillId::ALL {
            let c = skill_category(*id);
            let d = skill_definition(*id);
            match c {
                SkillCategory::Passive | SkillCategory::Proc => {
                    assert_eq!(d.kind, super::SkillKind::Passive);
                }
                SkillCategory::AttackSkill | SkillCategory::Buff | SkillCategory::Reactive => {
                    assert_eq!(d.kind, super::SkillKind::Active);
                }
                SkillCategory::BasicAttack | SkillCategory::Channel => {
                    panic!("unused category on SkillId");
                }
            }
        }
    }

    #[test]
    fn shared_ability_gcd_only_on_instant_and_next_melee_buff() {
        let (_, _, _, _) = skill_combat_meta(SkillId::HeavyStrike);
        let d = skill_definition(SkillId::HeavyStrike);
        assert!(!skill_triggers_shared_ability_gcd(d.combat_style));

        let d = skill_definition(SkillId::VictoryRush);
        assert!(skill_triggers_shared_ability_gcd(d.combat_style));

        let d = skill_definition(SkillId::EmpoweredBlow);
        assert!(skill_triggers_shared_ability_gcd(d.combat_style));
    }

    #[test]
    fn instant_strike_has_max_charges_on_definition() {
        assert_eq!(skill_definition(SkillId::VictoryRush).max_charges, 1);
        assert_eq!(skill_definition(SkillId::HeavyStrike).max_charges, 1);
    }
}
