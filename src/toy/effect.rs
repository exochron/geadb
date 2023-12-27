use std::collections::BTreeMap;

use crate::tools::db_reader::{parse_csv, LookupDB};
use crate::toy::dbs::{
    Item, SpellCategories, SpellEffect, SpellProceduralEffect, SpellVisualEvent,
    SpellVisualKitEffect, SpellXSpellVisual,
};
use crate::toy::Toy;

const SPELL_CATEGORY_HEARTHSTONE: u32 = 1176;
const SPELL_CATEGORY_GARRISON_HEARTHSTONE: u32 = 1524;
const SPELL_CATEGORY_MAIL: u32 = 2066;

// these ItemIDs are blanks for the correlating slot
const NO_ITEMS: [u32; 12] = [
    81324, 81325, 60620, 60618, 60617, 62814, 81326, 64311, 64311, 60619, 62816, 64310,
];

#[derive(Debug, Clone, PartialEq)]
pub enum Effect {
    Hearthstone,
    Bigger(f32),
    Smaller(f32),
    FullBody(u32, u32),
    Mail,
    ArmorItem(u32),
    ArmorSetItem(u32),
    ArmorWeapon(u32),
    ArmorHead(u32),
    Cloak(u32),
    Color(u32),
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
            Effect::Color(_) => "color",
        }
    }
    pub fn value(&self) -> u32 {
        match self {
            Effect::Bigger(scale) => *scale as u32,
            Effect::Smaller(scale) => *scale as u32,
            Effect::ArmorItem(item_id) => *item_id,
            Effect::ArmorSetItem(item_id) => *item_id,
            Effect::ArmorHead(item_id) => *item_id,
            Effect::Cloak(item_id) => *item_id,
            Effect::ArmorWeapon(item_id) => *item_id,
            Effect::Color(kit_id) => *kit_id,
            _ => 1,
        }
    }
}

fn find_spell_visuals(
    spell_id: &u32,
    spell_categories_db: &LookupDB<SpellCategories>,
    spell_effect_db: &LookupDB<SpellEffect>,
    spell_x_spell_visual_db: &LookupDB<SpellXSpellVisual>,
    spell_visual_event_db: &LookupDB<SpellVisualEvent>,
    spell_visual_kit_effect_db: &LookupDB<SpellVisualKitEffect>,
    spell_procedural_effect_db: &LookupDB<SpellProceduralEffect>,
) -> Vec<Effect> {
    let mut result = Vec::new();

    for spell_categories in spell_categories_db.lookup(spell_id) {
        match spell_categories.category {
            SPELL_CATEGORY_HEARTHSTONE | SPELL_CATEGORY_GARRISON_HEARTHSTONE => {
                result.push(Effect::Hearthstone);
            }
            SPELL_CATEGORY_MAIL => result.push(Effect::Mail),
            _ => {}
        }
    }

    for spell_effect in spell_effect_db.lookup(spell_id) {
        // aura=scale and target=caster
        if spell_effect.effect_aura == 61
            && spell_effect.effect == 6
            && spell_effect.implicit_target == 1
        {
            result.push(if spell_effect.effect_points > 0.0 {
                Effect::Bigger(spell_effect.effect_points)
            } else {
                Effect::Smaller(spell_effect.effect_points * -1.0)
            });
        }
    }

    for xvisual in spell_x_spell_visual_db.lookup(spell_id) {
        for visual_event in spell_visual_event_db.lookup(&xvisual.visual_id) {
            let visual_kit_id = visual_event.visual_kit_id;
            for visual_kit_effect in spell_visual_kit_effect_db.lookup(&visual_kit_id) {
                if visual_kit_effect.effect_type == 1 {
                    let procedural_effects =
                        spell_procedural_effect_db.lookup(&visual_kit_effect.effect_id);
                    let procedural_effect = procedural_effects.first().unwrap();

                    // SpellProceduralEffectID
                    if procedural_effect.effect_type == 1
                        || procedural_effect.effect_type == 21
                        || procedural_effect.effect_type == 22
                    {
                        // some color effect
                        result.push(Effect::Color(visual_kit_id))
                    } else if procedural_effect.effect_type == 17 {
                        // some armor item
                        let val_1 = procedural_effect.value_1 as u32;
                        if val_1 > 0 && !NO_ITEMS.contains(&val_1) {
                            result.push(Effect::ArmorItem(val_1))
                        }
                    }
                } else if visual_kit_effect.effect_type == 7 || visual_kit_effect.effect_type == 16
                {
                    // ShadowyEffectID || GradientEffect
                    result.push(Effect::Color(visual_kit_id))
                }
            }
        }
    }

    result
}

pub fn collect_effects(build_version: &String, mut toys: BTreeMap<u32, Toy>) -> BTreeMap<u32, Toy> {
    let spell_categories_db: LookupDB<SpellCategories> = LookupDB::new_from_data(
        parse_csv(build_version, "SpellCategories.csv").unwrap(),
        |s: &SpellCategories| s.spell_id,
    );
    let spell_effect_db: LookupDB<SpellEffect> = LookupDB::new_from_data(
        parse_csv(build_version, "SpellEffect.csv").unwrap(),
        |s: &SpellEffect| s.spell_id,
    );
    let spell_visual_kit_effect_db: LookupDB<SpellVisualKitEffect> = LookupDB::new_from_data(
        parse_csv(build_version, "SpellVisualKitEffect.csv").unwrap(),
        |s: &SpellVisualKitEffect| s.visual_kit_id,
    );
    let spell_procedural_effect_db: LookupDB<SpellProceduralEffect> = LookupDB::new_from_data(
        parse_csv(build_version, "SpellProceduralEffect.csv").unwrap(),
        |s: &SpellProceduralEffect| s.id,
    );
    let spell_x_spell_visual_db: LookupDB<SpellXSpellVisual> = LookupDB::new_from_data(
        parse_csv(build_version, "SpellXSpellVisual.csv").unwrap(),
        |s: &SpellXSpellVisual| s.spell_id,
    );
    let spell_visual_event_db: LookupDB<SpellVisualEvent> = LookupDB::new_from_data(
        parse_csv(build_version, "SpellVisualEvent.csv").unwrap(),
        |s: &SpellVisualEvent| s.visual_id,
    );

    for toy in toys.values_mut() {
        toy.effects.append(&mut find_spell_visuals(
            &toy.spell_id,
            &spell_categories_db,
            &spell_effect_db,
            &spell_x_spell_visual_db,
            &spell_visual_event_db,
            &spell_visual_kit_effect_db,
            &spell_procedural_effect_db,
        ));

        for effect in &spell_effect_db.lookup(&toy.spell_id) {
            if effect.effect == 32 && effect.trigger_spell_id > 0 {
                // TRIGGER_MISSILE
                toy.effects.append(&mut find_spell_visuals(
                    &(effect.trigger_spell_id as u32),
                    &spell_categories_db,
                    &spell_effect_db,
                    &spell_x_spell_visual_db,
                    &spell_visual_event_db,
                    &spell_visual_kit_effect_db,
                    &spell_procedural_effect_db,
                ));
            }
        }

        //     cleanup_effect_list(&mut toy.effects, build_version);
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
        let item_db: LookupDB<Item> =
            LookupDB::new_from_data(parse_csv(build_version, "Item.csv").unwrap(), |s: &Item| {
                s.item_id
            });
        for effect in effects.iter_mut() {
            if let Effect::ArmorItem(item_id) = effect {
                let inventory_slot = item_db.lookup(item_id).first().unwrap().inventory_type;
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
