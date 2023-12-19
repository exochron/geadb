use std::collections::{BTreeMap, HashMap};

use crate::tools::db_reader::DBReader;
use crate::toy::Toy;

const SPELL_CATEGORY_HEARTHSTONE: i64 = 1176;
const SPELL_CATEGORY_GARRISON_HEARTHSTONE: i64 = 1524;
const SPELL_CATEGORY_MAIL: i64 = 2066;

#[derive(Debug, Copy, Clone)]
pub enum Effect {
    Bigger(i64),
    Smaller(i64),
    FullBody(i64, i64),
    MinorBody(i64, i64),
    Hearthstone,
    Mail,
}

impl Effect {
    pub fn as_str(&self) -> &'static str {
        match self {
            Effect::Bigger(_) => "bigger",
            Effect::Smaller(_) => "smaller",
            Effect::FullBody(_, _) => "full_body",
            Effect::MinorBody(_, _) => "minor_body",
            Effect::Hearthstone => "hearthstone",
            Effect::Mail => "mail",
        }
    }
    pub fn value(&self) -> String {
        format!(
            "{}",
            match self {
                Effect::Bigger(scale) => scale,
                Effect::Smaller(scale) => scale,
                Effect::FullBody(kit_id, _) => kit_id,
                Effect::MinorBody(kit_id, _) => kit_id,
                _ => &1,
            }
        )
    }
}

pub fn collect_effects(
    build_version: &String,
    mut toys: BTreeMap<i64, Toy>,
    spell_to_item: HashMap<i64, i64>,
) -> BTreeMap<i64, Toy> {
    let mut spell_categories_csv =
        DBReader::new_with_id(build_version, "SpellCategories.csv", "SpellID").unwrap();
    for (_, toy) in toys.iter_mut() {
        let spell_category = spell_categories_csv.fetch_int_field(&toy.spell_id, "Category");
        match spell_category {
            SPELL_CATEGORY_HEARTHSTONE | SPELL_CATEGORY_GARRISON_HEARTHSTONE => {
                toy.effects.push(Effect::Hearthstone);
            }
            SPELL_CATEGORY_MAIL => toy.effects.push(Effect::Mail),
            _ => {}
        }
    }

    let mut spell_effect_csv = DBReader::new(build_version, "SpellEffect.csv").unwrap();
    for spell_effect_id in spell_effect_csv.ids() {
        let spell_id = spell_effect_csv.fetch_int_field(&spell_effect_id, "SpellID");
        match spell_to_item.get(&spell_id) {
            None => {}
            Some(item_id) => {
                let toy = toys.get_mut(item_id).unwrap();

                let effect_aura = spell_effect_csv.fetch_int_field(&spell_effect_id, "EffectAura");
                let effect = spell_effect_csv.fetch_int_field(&spell_effect_id, "Effect");
                let effect_target =
                    spell_effect_csv.fetch_int_field(&spell_effect_id, "ImplicitTarget[0]");

                // aura=scale and target=caster
                if effect_aura == 61 && effect == 6 && effect_target == 1 {
                    let scale =
                        spell_effect_csv.fetch_int_field(&spell_effect_id, "EffectBasePointsF");
                    toy.effects.push(if scale > 0 {
                        Effect::Bigger(scale)
                    } else {
                        Effect::Smaller(scale)
                    });
                }
            }
        }
    }

    let visual_kit_to_toy = {
        let mut spell_x_visual_csv =
            DBReader::new_with_id(build_version, "SpellXSpellVisual.csv", "SpellID").unwrap();

        let mut visual_to_toy: HashMap<i64, i64> = HashMap::new();
        for (_, toy) in toys.iter() {
            if spell_x_visual_csv.has(&toy.spell_id) {
                let visual_id = spell_x_visual_csv.fetch_int_field(&toy.spell_id, "SpellVisualID");
                visual_to_toy.insert(visual_id, toy.item_id);
            }
        }

        let mut spell_visual_event_csv =
            DBReader::new(build_version, "SpellVisualEvent.csv").unwrap();
        let mut visual_kit_to_toy: HashMap<i64, i64> = HashMap::new();
        for visual_event_id in spell_visual_event_csv.ids() {
            let visual_id =
                spell_visual_event_csv.fetch_int_field(&visual_event_id, "SpellVisualID");
            if visual_to_toy.contains_key(&visual_id) {
                let visual_kit_id =
                    spell_visual_event_csv.fetch_int_field(&visual_event_id, "SpellVisualKitID");
                visual_kit_to_toy.insert(visual_kit_id, *visual_to_toy.get(&visual_id).unwrap());
            }
        }

        visual_kit_to_toy
    };

    let mut spell_visual_effect_csv =
        DBReader::new(build_version, "SpellVisualKitEffect.csv").unwrap();
    let mut spell_visual_model_attach_csv =
        DBReader::new(build_version, "SpellVisualKitModelAttach.csv").unwrap();
    for visual_effect_id in spell_visual_effect_csv.ids() {
        let parent_kit_id =
            spell_visual_effect_csv.fetch_int_field(&visual_effect_id, "ParentSpellVisualKitID");
        if visual_kit_to_toy.contains_key(&parent_kit_id) {
            let effect_type =
                spell_visual_effect_csv.fetch_int_field(&visual_effect_id, "EffectType");
            let effect_id = spell_visual_effect_csv.fetch_int_field(&visual_effect_id, "Effect");

            let item_id = visual_kit_to_toy.get(&parent_kit_id).unwrap();
            let toy = toys.get_mut(item_id).unwrap();

            if effect_type == 2 {
                let attachment_id =
                    spell_visual_model_attach_csv.fetch_int_field(&effect_id, "AttachmentID");
                let effect_name_id = spell_visual_model_attach_csv
                    .fetch_int_field(&effect_id, "SpellVisualEffectNameID");
                if attachment_id == 22 && effect_name_id != 269 {
                    toy.effects
                        .push(Effect::MinorBody(parent_kit_id, effect_id))
                }
            };
        }
    }

    toys
}
