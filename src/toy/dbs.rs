use serde::Deserialize;

#[derive(Deserialize, Clone)]
pub struct SpellEffect {
    #[serde(rename(deserialize = "SpellID"))]
    pub spell_id: u32,
    #[serde(rename(deserialize = "EffectAura"))]
    pub effect_aura: u32,
    #[serde(rename(deserialize = "Effect"))]
    pub effect: u32,
    #[serde(rename(deserialize = "EffectTriggerSpell"))]
    pub trigger_spell_id: i32,
    // can be -1
    #[serde(rename(deserialize = "ImplicitTarget[0]"))]
    pub implicit_target: u32,
    #[serde(rename(deserialize = "EffectBasePointsF"))]
    #[serde(alias = "EffectBasePoints")]
    pub effect_points: f32,
}

#[derive(Deserialize, Clone)]
pub struct SpellCategories {
    #[serde(rename(deserialize = "SpellID"))]
    pub spell_id: u32,
    #[serde(rename(deserialize = "Category"))]
    pub category: u32,
}

#[derive(Deserialize, Clone)]
pub struct SpellVisualKitEffect {
    #[serde(rename(deserialize = "ParentSpellVisualKitID"))]
    pub visual_kit_id: u32,
    #[serde(rename(deserialize = "EffectType"))]
    pub effect_type: u8,
    #[serde(rename(deserialize = "Effect"))]
    pub effect_id: u32,
}

#[derive(Deserialize, Clone)]
pub struct SpellProceduralEffect {
    #[serde(rename(deserialize = "ID"))]
    pub id: u32,
    #[serde(rename(deserialize = "Type"))]
    pub effect_type: i8,
    #[serde(rename(deserialize = "Value[0]"))]
    pub value_0: f32,
    #[serde(rename(deserialize = "Value[1]"))]
    pub value_1: f32,
    #[serde(rename(deserialize = "Value[2]"))]
    pub value_2: f32,
}

#[derive(Deserialize, Clone)]
pub struct SpellXSpellVisual {
    #[serde(rename(deserialize = "SpellID"))]
    pub spell_id: u32,
    #[serde(rename(deserialize = "SpellVisualID"))]
    pub visual_id: u32,
}

#[derive(Deserialize, Clone)]
pub struct SpellVisualEvent {
    #[serde(rename(deserialize = "SpellVisualID"))]
    pub visual_id: u32,
    #[serde(rename(deserialize = "SpellVisualKitID"))]
    pub visual_kit_id: u32,
}
