use std::collections::BTreeMap;

use crate::tools::db_reader::DBReader;
use crate::tools::docker_runner::DockerRunner;
use crate::tools::load_config;
use crate::toy::export::Exporter;

mod export;

pub struct Toy {
    item_id: i64,
    name: String,
    item_is_tradable: bool,
}

pub fn handle_toys() {
    let config = load_config("toy.yml");

    let build_version = {
        let mut docker = DockerRunner::new();

        docker.fetch_toy_dbfiles();
        docker.convert_dbfiles_into_csv();
        docker.build_version
        // String::from("10.0.5.48526")
    };

    let mut toys = collect_toys(&build_version);

    for value in config.get("ignored").unwrap().as_sequence().unwrap().iter() {
        toys.remove(&value.as_i64().unwrap())
            .expect("ignored id doesn't exist anymore in game");
    }

    let exporter = Exporter::new(config.get("export_path").unwrap().as_str().unwrap());
    exporter.export_tradable(&toys);
    exporter.export_toys(&toys);
}

fn to_int(field: Option<&str>) -> i64 {
    field
        .unwrap()
        .parse()
        .expect("couldn't convert field into int.")
}

fn collect_toys(build_version: &String) -> BTreeMap<i64, Toy> {
    let mut collection: BTreeMap<i64, Toy> = BTreeMap::new();

    let mut toy_csv = DBReader::new(build_version, "Toy.csv");
    let mut item_csv = DBReader::new(build_version, "ItemSparse.csv");
    for row in toy_csv.reader.records() {
        let record = row.unwrap();
        let item_id = to_int(record.get(2));

        let name = item_csv.fetch_field(&item_id, 6);
        if let Some(..) = name {
            collection.insert(
                item_id,
                Toy {
                    item_id,
                    name: name.unwrap(),
                    item_is_tradable: item_csv.fetch_int_field(&item_id, 80) == 3,
                },
            );
        }
    }

    collection
}
