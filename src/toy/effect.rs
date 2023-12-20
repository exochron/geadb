use std::collections::{BTreeMap, HashMap};

use crate::tools::db_reader::DBReader;
use crate::toy::Toy;

const SPELL_CATEGORY_HEARTHSTONE: i64 = 1176;
const SPELL_CATEGORY_GARRISON_HEARTHSTONE: i64 = 1524;
const SPELL_CATEGORY_MAIL: i64 = 2066;

// these ItemIDs are blanks for the correlating slot
const NO_ITEMS: [i64; 12] = [
    81324, 81325, 60620, 60618, 60617, 62814, 81326, 64311, 64311, 60619, 62816, 64310,
];

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Effect {
    Bigger(i64),
    Smaller(i64),
    FullBody(i64, i64),
    Hearthstone,
    Mail,
    ArmorItem(i64),
    ArmorSetItem(i64),
    ArmorWeapon(i64),
    ArmorHead(i64),
    Cloak(i64),
}

impl Effect {
    pub fn as_str(&self) -> &'static str {
        match self {
            Effect::Bigger(_) => "bigger",
            Effect::Smaller(_) => "smaller",
            Effect::FullBody(_, _) => "full_body",
            Effect::Hearthstone => "hearthstone",
            Effect::Mail => "mail",
            Effect::ArmorItem(_) => "armor",
            Effect::ArmorSetItem(_) => "armor_set",
            Effect::ArmorHead(_) => "head",
            Effect::Cloak(_) => "cloak",
            Effect::ArmorWeapon(_) => "weapon",
        }
    }
    pub fn value(&self) -> i64 {
        *match self {
            Effect::Bigger(scale) => scale,
            Effect::Smaller(scale) => scale,
            Effect::ArmorItem(item_id) => item_id,
            Effect::ArmorSetItem(item_id) => item_id,
            Effect::ArmorHead(item_id) => item_id,
            Effect::Cloak(item_id) => item_id,
            Effect::ArmorWeapon(item_id) => item_id,
            _ => &1,
        }
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
    let mut spell_procedural_effect =
        DBReader::new(build_version, "SpellProceduralEffect.csv").unwrap();
    for visual_effect_id in spell_visual_effect_csv.ids() {
        let parent_kit_id =
            spell_visual_effect_csv.fetch_int_field(&visual_effect_id, "ParentSpellVisualKitID");
        if visual_kit_to_toy.contains_key(&parent_kit_id) {
            let effect_type =
                spell_visual_effect_csv.fetch_int_field(&visual_effect_id, "EffectType");
            let effect_id = spell_visual_effect_csv.fetch_int_field(&visual_effect_id, "Effect");

            let item_id = visual_kit_to_toy.get(&parent_kit_id).unwrap();
            let toy = toys.get_mut(item_id).unwrap();

            if effect_type == 1 {
                // SpellProceduralEffectID
                let procedure_type = spell_procedural_effect.fetch_int_field(&effect_id, "Type");
                if procedure_type == 17 {
                    let val_1 = spell_procedural_effect.fetch_int_field(&effect_id, "Value[1]");
                    if val_1 > 0 && !NO_ITEMS.contains(&val_1) {
                        toy.effects.push(Effect::ArmorItem(val_1))
                    }
                }
            };
        }
    }

    for (_, toy) in toys.iter_mut() {
        cleanup_effect_list(&mut toy.effects, build_version);
    }

    toys
}

fn cleanup_effect_list(effects: &mut Vec<Effect>, build_version: &String) {
    effects.dedup();

    // Hearthstones are only that
    if effects.contains(&Effect::Hearthstone) {
        effects.clear();
        effects.push(Effect::Hearthstone);
        return;
    }

    let armor_count = {
        let mut count = 0;
        for effect in effects.iter() {
            if let Effect::ArmorItem(_) = effect {
                count += 1
            }
        }

        count
    };

    // 3 or more armor pieces are a set
    if armor_count > 2 {
        let mut collector: Vec<Effect> = Vec::new();
        while let Some(effect) = effects.pop() {
            if let Effect::ArmorItem(item_id) = effect {
                collector.push(Effect::ArmorSetItem(item_id));
            }
        }
        effects.append(&mut collector);
        return;
    }

    if armor_count > 0 {
        let mut item_db = DBReader::new(build_version, "Item.csv").unwrap();
        for effect in effects.iter_mut() {
            if let Effect::ArmorItem(item_id) = effect {
                let inventory_slot = item_db.fetch_int_field(item_id, "InventoryType");
                *effect = match inventory_slot {
                    1 => Effect::ArmorHead(*item_id),
                    13 => Effect::ArmorWeapon(*item_id),
                    14 => Effect::ArmorWeapon(*item_id),
                    15 => Effect::ArmorWeapon(*item_id),
                    17 => Effect::ArmorWeapon(*item_id),
                    21 => Effect::ArmorWeapon(*item_id),
                    22 => Effect::ArmorWeapon(*item_id),
                    23 => Effect::ArmorWeapon(*item_id),
                    26 => Effect::ArmorWeapon(*item_id),
                    16 => Effect::Cloak(*item_id),
                    _ => Effect::ArmorItem(*item_id),
                };
            }
        }
    }
}
