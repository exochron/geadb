use std::collections::{BTreeMap, HashMap};

use crate::tools::casc_loader::load_dbs;
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

    load_dbs(&config, &build_version);
    load_dbs(&config, &classic_version);

    // // todo: multi thread
    // let build_version = {
    //     let mut docker = DockerRunner::new(game_version);
    //
    //     // docker.fetch_toy_dbfiles();
    //     // docker.convert_dbfiles_into_csv();
    //     // docker.build_version
    //
    //     "10.2.0.52485".to_string()
    // };
    // let classic_build_version = {
    //     let mut docker = DockerRunner::new(ProductVersion::Classic);
    //
    //     docker.fetch_toy_dbfiles();
    //     docker.convert_dbfiles_into_csv();
    //     docker.build_version
    //
    //     // "3.4.3.52237".to_string()
    // };

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

fn to_int(field: Option<&str>) -> i64 {
    field
        .unwrap()
        .parse()
        .expect("couldn't convert field into int.")
}

fn collect_toys(build_version: &String) -> BTreeMap<i64, Toy> {
    let mut toys: BTreeMap<i64, Toy> = BTreeMap::new();
    let mut spell_to_item: HashMap<i64, i64> = HashMap::new();

    let mut toy_db = DBReader::new(build_version, "Toy.csv").unwrap();
    let mut item_sparse_db = DBReader::new(build_version, "ItemSparse.csv").unwrap();
    let mut item_x_effect_db =
        DBReader::new_with_id(build_version, "ItemXItemEffect.csv", "ItemID");
    let mut item_effect_db = match item_x_effect_db {
        None => DBReader::new_with_id(build_version, "ItemEffect.csv", "ParentItemID").unwrap(),
        Some(_) => DBReader::new(build_version, "ItemEffect.csv").unwrap(),
    };

    for toy_id in toy_db.ids() {
        let item_id = toy_db.fetch_int_field(&toy_id, "ItemID");

        let spell_id = match &mut item_x_effect_db {
            None => item_effect_db.fetch_int_field(&item_id, "SpellID"),
            Some(x_effect_csv) => {
                let item_effect_id = x_effect_csv.fetch_int_field(&item_id, "ItemEffectID");
                item_effect_db.fetch_int_field(&item_effect_id, "SpellID")
            }
        };

        let name = item_sparse_db
            .fetch_field(&item_id, "Display_lang")
            .unwrap_or_default();
        if !name.is_empty() {
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
