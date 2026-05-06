use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SkillId {
    LifestealStrike,
    Guard,
    HeavyStrike,
    PoisonEdge,
    ThornSkin,
    BarrierPulse,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SkillTag {
    Attack,
    Defense,
    Healing,
    Poison,
    Barrier,
    Thorns,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SkillTrigger {
    OnAttack,
    OnHitTaken,
    OnRoomStart,
    PeriodicTick,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct SkillDefinition {
    pub id: SkillId,
    pub name: &'static str,
    pub trigger: SkillTrigger,
    pub tags: &'static [SkillTag],
    pub description: &'static str,
}

pub fn skill_definition(id: SkillId) -> SkillDefinition {
    match id {
        SkillId::LifestealStrike => SkillDefinition {
            id,
            name: "Lifesteal Strike",
            trigger: SkillTrigger::OnAttack,
            tags: &[SkillTag::Attack, SkillTag::Healing],
            description: "Attacks restore a small amount of health.",
        },
        SkillId::Guard => SkillDefinition {
            id,
            name: "Guard",
            trigger: SkillTrigger::OnHitTaken,
            tags: &[SkillTag::Defense],
            description: "Reduces incoming damage.",
        },
        SkillId::HeavyStrike => SkillDefinition {
            id,
            name: "Heavy Strike",
            trigger: SkillTrigger::OnAttack,
            tags: &[SkillTag::Attack],
            description: "Slow attacks hit harder.",
        },
        SkillId::PoisonEdge => SkillDefinition {
            id,
            name: "Poison Edge",
            trigger: SkillTrigger::OnAttack,
            tags: &[SkillTag::Attack, SkillTag::Poison],
            description: "Attacks apply poison.",
        },
        SkillId::ThornSkin => SkillDefinition {
            id,
            name: "Thorn Skin",
            trigger: SkillTrigger::OnHitTaken,
            tags: &[SkillTag::Defense, SkillTag::Thorns],
            description: "Reflects damage when hit.",
        },
        SkillId::BarrierPulse => SkillDefinition {
            id,
            name: "Barrier Pulse",
            trigger: SkillTrigger::OnRoomStart,
            tags: &[SkillTag::Defense, SkillTag::Barrier],
            description: "Starts each room with a barrier.",
        },
    }
}
