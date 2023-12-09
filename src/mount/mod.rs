use std::collections::{BTreeMap, HashMap};

use regex::Regex;

use crate::mount::condition::{parse_conditions, Condition};
use crate::mount::customization::collect_customization;
use crate::mount::export::Exporter;
use crate::mount::family::group_by_families;
use crate::mount::image::collect_dominant_colors;
use crate::tools::db_reader::DBReader;
use crate::tools::docker_runner::DockerRunner;
use crate::tools::{load_config, load_listfile, GameVersion};

mod condition;
mod customization;
mod export;
mod family;
mod image;

pub struct Mount {
    id: i64,
    spell_id: i64,
    type_id: i64,
    name: String,
    icon: String,
    item_is_tradeable: bool,
    player_conditions: Vec<Vec<Condition>>,
}

impl Mount {
    fn new(
        id: i64,
        spell_id: i64,
        type_id: i64,
        name: String,
        player_conditions: Vec<Vec<Condition>>,
    ) -> Self {
        Self {
            id,
            spell_id,
            type_id,
            name,
            icon: String::new(),
            item_is_tradeable: false,
            player_conditions,
        }
    }
}

pub fn handle_mounts(game_version: GameVersion) {
    let config = load_config("mount.yml");

    let mut docker = DockerRunner::new(game_version);
    let build_version = {
        docker.fetch_mount_dbfiles();
        docker.convert_dbfiles_into_csv();
        docker.build_version.clone()
        // "10.2.0.52148".to_string()
    };
    let classic_build_version = {
        let mut docker = DockerRunner::new(GameVersion::Classic);
        docker.fetch_mount_dbfiles();
        docker.convert_dbfiles_into_csv();
        docker.build_version.clone()
    };

    let list_file = load_listfile();

    let mut mounts: BTreeMap<i64, Mount> = BTreeMap::new();
    mounts.append(&mut collect_mounts(&classic_build_version, &list_file));
    mounts.append(&mut collect_mounts(&build_version, &list_file));

    for value in config.get("ignored").unwrap().as_sequence().unwrap().iter() {
        mounts.remove(&value.as_i64().unwrap()).unwrap_or_else(|| {
            panic!(
                "ignored id doesn't exist anymore in game {}",
                value.as_i64().unwrap()
            )
        });
    }

    let exporter = Exporter::new(config.get("export_path").unwrap().as_str().unwrap());
    exporter.export_tradable(&mounts);
    exporter.export_conditions(&mounts);
    exporter.export_families(
        &mounts,
        group_by_families(&mounts, config.get("familymap").unwrap()),
    );
    exporter.export_customization(&mounts, collect_customization(&mounts, &build_version));
    exporter.export_colors(
        &mounts,
        collect_dominant_colors(&build_version, docker, &mounts, &list_file),
    );
}

fn collect_mounts(
    build_version: &String,
    list_file: &HashMap<i64, String>,
) -> BTreeMap<i64, Mount> {
    let mut collection: BTreeMap<i64, Mount> = BTreeMap::new();
    let mut spell_to_mount: HashMap<i64, i64> = HashMap::new();

    {
        let mut mount_csv = DBReader::new(build_version, "Mount.csv").unwrap();
        let mut playercondition_csv = DBReader::new(build_version, "PlayerCondition.csv").unwrap();
        for id in mount_csv.ids() {
            let type_id = mount_csv.fetch_int_field(&id, "MountTypeID");
            let spell_id = mount_csv.fetch_int_field(&id, "SourceSpellID");

            let playercondition_id = mount_csv.fetch_int_field(&id, "PlayerConditionID");
            let player_conditions = match playercondition_csv
                .fetch_field(&playercondition_id, "Failure_description_lang")
            {
                None => Vec::new(),
                Some(failure_condition) => parse_conditions(
                    playercondition_csv.fetch_int_field(&playercondition_id, "RaceMask"),
                    failure_condition.as_str(),
                    playercondition_csv.fetch_int_field(&playercondition_id, "ClassMask"),
                    playercondition_csv.fetch_int_field(&playercondition_id, "SkillID[0]"),
                    playercondition_csv.fetch_int_field(&playercondition_id, "PrevQuestID[0]"),
                ),
            };

            collection.insert(
                id.clone(),
                Mount::new(
                    id.clone(),
                    spell_id,
                    type_id,
                    mount_csv.fetch_field(&id, "Name_lang").unwrap(),
                    player_conditions,
                ),
            );
            spell_to_mount.insert(spell_id, id.clone());
        }
    }

    {
        let mut itemxeffect_csv =
            DBReader::new_with_id(build_version, "ItemXItemEffect.csv", "ItemEffectID");
        let mut itemeffect_csv = DBReader::new(build_version, "ItemEffect.csv").unwrap();
        let mut itemsparse_csv = DBReader::new(build_version, "ItemSparse.csv").unwrap();
        for effect_id in itemeffect_csv.ids() {
            let item_id: i64 = match itemeffect_csv.fetch_field(&effect_id, "ParentItemID") {
                Some(item_id) => item_id.parse().unwrap(), // classic format
                None => itemxeffect_csv
                    .as_mut()
                    .unwrap()
                    .fetch_int_field(&effect_id, "ItemID"), // retail format
            };

            let spell_id = itemeffect_csv.fetch_int_field(&effect_id, "SpellID");

            // is mount spell && is TriggerType = OnUse(6) && is Bonding = 0
            if item_id > 0
                && spell_to_mount.contains_key(&spell_id)
                && itemeffect_csv
                    .fetch_field(&effect_id, "TriggerType")
                    .unwrap()
                    == "6"
                && itemsparse_csv
                    .fetch_field(&item_id, "Bonding")
                    .unwrap_or_default()
                    == "0"
            {
                let mount_id = spell_to_mount.get(&spell_id).unwrap();
                collection.get_mut(mount_id).unwrap().item_is_tradeable = true
            }
        }
    }

    {
        let mut spellmisc_csv = DBReader::new(build_version, "SpellMisc.csv").unwrap();
        let regex = Regex::new("(?i)interface/icons/(.*)\\.blp").expect("invalid regexp");
        for row_id in spellmisc_csv.ids() {
            let spell_id = spellmisc_csv.fetch_int_field(&row_id, "SpellID");
            match spell_to_mount.get(&spell_id) {
                None => {}
                Some(mount_id) => {
                    let spell_icon_file_data_id =
                        spellmisc_csv.fetch_int_field(&row_id, "SpellIconFileDataID");
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
