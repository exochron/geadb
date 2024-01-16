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
    #[serde(rename(deserialize = "EffectMiscValue[0]"))]
    pub effect_misc_value: i64,
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

#[derive(Deserialize, Clone)]
pub struct SpellMisc {
    #[serde(rename(deserialize = "SpellID"))]
    pub spell_id: u32,
    #[serde(rename(deserialize = "SpellIconFileDataID"))]
    pub spell_icon_file_id: i64,
}

#[derive(Deserialize, Clone)]
pub struct Item {
    #[serde(rename(deserialize = "ID"))]
    pub item_id: u32,
    #[serde(rename(deserialize = "InventoryType"))]
    pub inventory_type: u32,
}

#[derive(Deserialize, Clone)]
pub struct ItemSparse {
    #[serde(rename(deserialize = "ID"))]
    pub item_id: u32,
    #[serde(rename(deserialize = "Display_lang"))]
    pub display_text: String,
    #[serde(rename(deserialize = "Bonding"))]
    pub bonding: i32,
    #[serde(rename(deserialize = "ItemNameDescriptionID"))]
    pub description_id: u32,
}

#[derive(Deserialize, Clone)]
pub struct ItemXItemEffect {
    #[serde(rename(deserialize = "ItemID"))]
    pub item_id: u32,
    #[serde(rename(deserialize = "ItemEffectID"))]
    pub item_effect_id: u32,
}

#[derive(Deserialize, Clone)]
pub struct ItemEffect {
    #[serde(rename(deserialize = "ID"))]
    pub id: u32,
    #[serde(rename(deserialize = "TriggerType"))]
    pub trigger_type: i32,
    #[serde(rename(deserialize = "SpellID"))]
    pub spell_id: i32,
    // can be -1
    #[serde(rename(deserialize = "ParentItemID"), default)]
    pub item_id: u32,
}

#[derive(Deserialize, Clone)]
pub struct Toy {
    #[serde(rename(deserialize = "ID"))]
    pub toy_id: u32,
    #[serde(rename(deserialize = "ItemID"))]
    pub item_id: u32,
}

#[derive(Deserialize, Clone)]
pub struct Mount {
    #[serde(rename(deserialize = "ID"))]
    pub id: u32,
    #[serde(rename(deserialize = "Name_lang"))]
    pub name: String,
    #[serde(rename(deserialize = "MountTypeID"))]
    pub type_id: u32,
    #[serde(rename(deserialize = "SourceSpellID"))]
    pub spell_id: u32,
    #[serde(rename(deserialize = "PlayerConditionID"))]
    pub player_condition_id: u32,
}

#[derive(Deserialize, Clone)]
pub struct PlayerCondition {
    #[serde(rename(deserialize = "ID"))]
    pub id: u32,
    #[serde(rename(deserialize = "Failure_description_lang"))]
    pub description: String,
    #[serde(rename(deserialize = "RaceMask"))]
    pub race_mask: i64,
    #[serde(rename(deserialize = "ClassMask"))]
    pub class_mask: i64,
    #[serde(rename(deserialize = "SkillID[0]"))]
    pub skill_id: u32,
    #[serde(rename(deserialize = "PrevQuestID[0]"))]
    pub quest_id: u32,
}

#[derive(Deserialize, Clone)]
pub struct MountXDisplay {
    #[serde(rename(deserialize = "CreatureDisplayInfoID"))]
    pub creature_info_id: u32,
    #[serde(rename(deserialize = "MountID"))]
    pub mount_id: u32,
}

#[derive(Deserialize, Clone)]
pub struct CreatureDisplayInfo {
    #[serde(rename(deserialize = "ID"))]
    pub id: u32,
    #[serde(rename(deserialize = "ModelID"))]
    pub model_id: u32,
    #[serde(rename(deserialize = "TextureVariationFileDataID[0]"))]
    pub texture_variant_file_id_0: i64,
    #[serde(rename(deserialize = "TextureVariationFileDataID[1]"))]
    pub texture_variant_file_id_1: i64,
    #[serde(rename(deserialize = "TextureVariationFileDataID[2]"))]
    pub texture_variant_file_id_2: i64,
    #[serde(rename(deserialize = "TextureVariationFileDataID[3]"))]
    pub texture_variant_file_id_3: i64,
}

#[derive(Deserialize, Clone)]
pub struct CreatureModel {
    #[serde(rename(deserialize = "ID"))]
    pub id: u32,
    #[serde(rename(deserialize = "FileDataID"))]
    pub file_id: i64,
}
