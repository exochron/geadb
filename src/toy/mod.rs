use std::collections::{BTreeMap, HashMap};

use crate::tools::db_reader::DBReader;
use crate::tools::{load_config, BuildInfo, ProductVersion};
use crate::toy::effect::{collect_effects, Effect};
use crate::toy::export::Exporter;

mod effect;
mod export;

pub struct Toy {
    item_id: i64,
    spell_id: i64,
    name: String,
    item_is_tradable: bool,
    effects: Vec<Effect>,
}

pub fn handle_toys(game_version: ProductVersion) {
    let config = load_config("toy.yml");

    let build_version = BuildInfo::parse_build_info(
        config.get("game_path").unwrap().as_str().unwrap(),
        game_version,
    )
    .unwrap();
    let classic_version = BuildInfo::parse_build_info(
        config.get("game_path").unwrap().as_str().unwrap(),
        ProductVersion::Classic,
    )
    .unwrap();

    // load_dbs(&config, &build_version);
    // load_dbs(&config, &classic_version);

    let mut toys = collect_toys(&classic_version.version);
    let mut retail_toys = collect_toys(&build_version.version);
    toys.append(&mut retail_toys);

    for value in config.get("ignored").unwrap().as_sequence().unwrap().iter() {
        toys.remove(&value.as_i64().unwrap())
            .expect("ignored id doesn't exist anymore in game");
    }

    let exporter = Exporter::new(config.get("export_path").unwrap().as_str().unwrap());
    exporter.export_tradable(&toys);
    exporter.export_toys(&toys);
    exporter.export_effects(&toys);
}

fn collect_toys(build_version: &String) -> BTreeMap<i64, Toy> {
    let mut toys: BTreeMap<i64, Toy> = BTreeMap::new();
    let mut spell_to_item: HashMap<i64, i64> = HashMap::new();

    let mut toy_db = DBReader::new(build_version, "Toy.csv").unwrap();
    let mut item_sparse_db = DBReader::new(build_version, "ItemSparse.csv").unwrap();
    let item_to_spell = {
        let mut item_to_spell: HashMap<i64, i64> = HashMap::new();
        let item_x_effect_db = DBReader::new(build_version, "ItemXItemEffect.csv");
        match item_x_effect_db {
            None => {
                let mut item_effect_db =
                    DBReader::new_with_id(build_version, "ItemEffect.csv", "ParentItemID").unwrap();
                for item_id in item_effect_db.ids() {
                    let spell_id = item_effect_db.fetch_int_field(&item_id, "SpellID");
                    item_to_spell.insert(item_id, spell_id);
                }
            }
            Some(mut item_x_effect_db) => {
                let mut item_effect_db = DBReader::new(build_version, "ItemEffect.csv").unwrap();
                for row_id in item_x_effect_db.ids() {
                    let effect_id = item_x_effect_db.fetch_int_field(&row_id, "ItemEffectID");
                    if 0 == item_effect_db.fetch_int_field(&effect_id, "TriggerType") {
                        // on Use
                        item_to_spell.insert(
                            item_x_effect_db.fetch_int_field(&row_id, "ItemID"),
                            item_effect_db.fetch_int_field(&effect_id, "SpellID"),
                        );
                    }
                }
            }
        };
        item_to_spell
    };

    for toy_id in toy_db.ids() {
        let item_id = toy_db.fetch_int_field(&toy_id, "ItemID");

        let spell_id = item_to_spell.get(&item_id);

        // items without use spell are just some dev armor pieces
        if let Some(&spell_id) = spell_id {
            let name = item_sparse_db
                .fetch_field(&item_id, "Display_lang")
                .unwrap_or_default();
            spell_to_item.insert(spell_id, item_id);
            toys.insert(
                item_id,
                Toy {
                    item_id,
                    spell_id,
                    name,
                    item_is_tradable: item_sparse_db.fetch_int_field(&item_id, "Bonding") == 3,
                    effects: vec![],
                },
            );
        }
    }

    toys = collect_effects(build_version, toys, spell_to_item);

    toys
}
