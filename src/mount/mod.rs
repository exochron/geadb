use std::collections::{BTreeMap, HashMap};

use regex::Regex;

use crate::mount::condition::{parse_conditions, Condition};
use crate::mount::export::Exporter;
use crate::mount::family::group_by_families;
use crate::mount::rarity::load_rarities;
use crate::tools::db_reader::DBReader;
use crate::tools::docker_runner::DockerRunner;
use crate::tools::{load_config, load_listfile};

mod condition;
mod export;
mod family;
mod rarity;

pub struct Mount {
    id: i64,
    spell_id: i64,
    name: String,
    icon: String,
    item_is_tradeable: bool,
    player_conditions: Vec<Vec<Condition>>,
    colors: Vec<Vec<u8>>,
    rarity: Option<f64>,
}

impl Mount {
    fn new(id: i64, spell_id: i64, name: String, player_conditions: Vec<Vec<Condition>>) -> Self {
        Self {
            id,
            spell_id,
            name,
            icon: String::new(),
            item_is_tradeable: false,
            player_conditions,
            colors: Vec::new(),
            rarity: None,
        }
    }
}

fn to_int(field: Option<&str>) -> i64 {
    field
        .unwrap()
        .parse()
        .expect("couldn't convert field into int.")
}

pub fn handle_mounts() {
    let config = load_config("mount.yml");

    let build_version = {
        let mut docker = DockerRunner::new();

        // docker.fetch_mount_dbfiles();
        // docker.convert_dbfiles_into_csv();
        // docker.build_version
        String::from("10.0.5.48069")
    };

    let mut mounts = collect_mounts(&build_version, load_listfile());

    for value in config.get("ignored").unwrap().as_sequence().unwrap().iter() {
        mounts.remove(&value.as_i64().unwrap());
    }

    for (mount_id, rarity) in load_rarities() {
        mounts.get_mut(&mount_id).unwrap().rarity = Some(rarity);
    }

    let exporter = Exporter::new(config.get("export_path").unwrap().as_str().unwrap());
    exporter.export_tradable(&mounts);
    exporter.export_conditions(&mounts);
    exporter.export_rarities(&mounts);
    exporter.export_families(
        &mounts,
        group_by_families(&mounts, config.get("familymap").unwrap()),
    );
}

fn collect_mounts(build_version: &String, list_file: HashMap<i64, String>) -> BTreeMap<i64, Mount> {
    let mut collection: BTreeMap<i64, Mount> = BTreeMap::new();
    let mut spell_to_mount: HashMap<i64, i64> = HashMap::new();

    {
        let mut mount_csv = DBReader::new(build_version, "Mount.csv");
        let mut playercondition_csv = DBReader::new(build_version, "PlayerCondition.csv");
        for row in mount_csv.reader.records() {
            let record = row.unwrap();
            let id = to_int(record.get(3));
            let spell_id = to_int(record.get(7));

            let playercondition_id = to_int(record.get(8));
            let player_conditions = match playercondition_csv.fetch_record(&playercondition_id) {
                None => Vec::new(),
                Some(record) => parse_conditions(
                    to_int(record.get(1)),
                    record.get(2).unwrap(),
                    to_int(record.get(3)),
                    to_int(record.get(58)),
                ),
            };

            collection.insert(
                id,
                Mount::new(
                    id,
                    spell_id,
                    record.get(0).unwrap().to_string(),
                    player_conditions,
                ),
            );
            spell_to_mount.insert(spell_id, id);
        }
    }

    {
        let mut itemxeffect_csv = DBReader::new(build_version, "ItemXItemEffect.csv");
        let mut itemeffect_csv = DBReader::new(build_version, "ItemEffect.csv");
        let mut itemsparse_csv = DBReader::new(build_version, "ItemSparse.csv");
        for row in itemxeffect_csv.reader.records() {
            let record = row.unwrap();
            let effect_id = to_int(record.get(1));
            let item_id = to_int(record.get(2));

            let spell_id = itemeffect_csv
                .fetch_field(&effect_id, 7)
                .unwrap()
                .parse()
                .unwrap();

            // is mount spell && is TriggerType = OnUse(6) && is Bonding = 0
            if item_id > 0
                && spell_to_mount.contains_key(&spell_id)
                && itemeffect_csv.fetch_field(&effect_id, 2).unwrap() == "6"
                && itemsparse_csv.fetch_field(&item_id, 80).unwrap_or_default() == "0"
            {
                let mount_id = spell_to_mount.get(&spell_id).unwrap();
                collection.get_mut(mount_id).unwrap().item_is_tradeable = true
            }
        }
    }

    {
        let mut spellmisc_csv = DBReader::new(build_version, "SpellMisc.csv");
        let regex = Regex::new("(?i)interface/icons/(.*)\\.blp").expect("invalid regexp");
        for row in spellmisc_csv.reader.records() {
            let record = row.unwrap();
            let spell_id = to_int(record.get(30));
            match spell_to_mount.get(&spell_id) {
                None => {}
                Some(mount_id) => {
                    let spell_icon_file_data_id = to_int(record.get(24));
                    match list_file.get(&spell_icon_file_data_id) {
                        None => {}
                        Some(file_path) => {
                            let mount = collection.get_mut(mount_id).unwrap();
                            mount.icon = regex
                                .captures(file_path)
                                .expect("didn't found icon name in file path")
                                .get(1)
                                .expect("didn't found icon name in file path")
                                .as_str()
                                .to_string();
                        }
                    }
                }
            }
        }
    }

    collection
}
