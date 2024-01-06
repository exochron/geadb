use std::collections::BTreeMap;

use crate::tools::{BuildInfo, load_config, ProductVersion};
use crate::tools::casc_loader::load_dbs;
use crate::tools::db_reader::{load_item_effects, LookupDB, parse_csv};
use crate::tools::dbs;
use crate::toy::effect::{collect_effects, Effect};
use crate::toy::export::Exporter;

mod effect;
mod export;

pub struct Toy {
    item_id: u32,
    spell_id: u32,
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

    let mut toys = collect_toys(&classic_version.version);
    let mut retail_toys = collect_toys(&build_version.version);
    toys.append(&mut retail_toys);

    for value in config.get("ignored").unwrap().as_sequence().unwrap().iter() {
        toys.remove(&(value.as_i64().unwrap() as u32))
            .expect("ignored id doesn't exist anymore in game");
    }

    let exporter = Exporter::new(config.get("export_path").unwrap().as_str().unwrap());
    exporter.export_tradable(&toys);
    exporter.export_toys(&toys);
    exporter.export_effects(&toys);
}

fn collect_toys(build_version: &String) -> BTreeMap<u32, Toy> {
    let mut toys: BTreeMap<u32, Toy> = BTreeMap::new();

    let toy_db: Vec<dbs::Toy> = parse_csv(build_version, "Toy.csv").unwrap();
    let item_sparse_db: LookupDB<dbs::ItemSparse> = LookupDB::new_from_data(
        parse_csv(build_version, "ItemSparse.csv").unwrap(),
        |s: &dbs::ItemSparse| s.item_id,
    );

    let item_effects_db = load_item_effects(build_version, false);

    for toy_row in toy_db {
        let item_id = toy_row.item_id;

        let spell_id = {
            let mut result = None;
            for item_effect in item_effects_db.lookup(&item_id) {
                if item_effect.trigger_type == 0 {
                    result = Some(item_effect.spell_id as u32);
                    break;
                }
            }
            result
        };

        // items without use spell are just some dev armor pieces
        if let Some(spell_id) = spell_id {
            let item_sparses = item_sparse_db.lookup(&item_id);
            let item_sparse = item_sparses.first();
            toys.insert(
                item_id,
                Toy {
                    item_id,
                    spell_id,
                    name: item_sparse
                        .map(|sparse| sparse.display_text.clone())
                        .unwrap_or_default(),
                    item_is_tradable: item_sparse.map(|sparse| sparse.bonding).unwrap_or_default()
                        == 3,
                    effects: vec![],
                },
            );
        }
    }

    toys = collect_effects(build_version, toys);

    toys
}
