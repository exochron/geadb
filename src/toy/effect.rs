use std::collections::{BTreeMap, HashMap};

use crate::tools::db_reader::DBReader;
use crate::toy::Toy;

pub enum Effect {
    PlayerScale(i64),
    FullVisual(i64, i64),
    MinorVisual(i64, i64),
}

pub fn collect_effects(
    build_version: &String,
    mut toys: BTreeMap<i64, Toy>,
    spell_to_item: HashMap<i64, i64>,
) -> BTreeMap<i64, Toy> {
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
                    toy.effects.push(Effect::PlayerScale(
                        spell_effect_csv.fetch_int_field(&spell_effect_id, "EffectBasePointsF"),
                    ));
                }
            }
        }
    }

    let mut spell_x_visual_csv =
        DBReader::new_with_id(build_version, "SpellXSpellVisual.csv", "SpellID").unwrap();
    let mut spell_visual_effect_csv =
        DBReader::new(build_version, "SpellVisualKitEffect.csv").unwrap();

    let mut visual_to_spells: HashMap<i64, Vec<i64>> = HashMap::new();
    for (_, toy) in toys.iter() {
        if spell_x_visual_csv.has(&toy.spell_id) {
            let visual_kit_id = spell_x_visual_csv.fetch_int_field(&toy.spell_id, "SpellVisualID");
            visual_to_spells
                .entry(visual_kit_id)
                .or_default()
                .push(toy.spell_id)
        }
    }

    for visual_effect_id in spell_visual_effect_csv.ids() {
        let parent_kit_id =
            spell_visual_effect_csv.fetch_int_field(&visual_effect_id, "ParentSpellVisualKitID");
        if visual_to_spells.contains_key(&parent_kit_id) {
            let effect_type =
                spell_visual_effect_csv.fetch_int_field(&visual_effect_id, "EffectType");
            let effect_id = spell_visual_effect_csv.fetch_int_field(&visual_effect_id, "Effect");

            for spell_id in visual_to_spells.get(&parent_kit_id).unwrap() {
                let item_id = spell_to_item.get(spell_id).unwrap();
                let toy = toys.get_mut(item_id).unwrap();

                if effect_type == 2 {
                    toy.effects
                        .push(Effect::MinorVisual(parent_kit_id, effect_id))
                };
            }
        }
    }

    toys
}
