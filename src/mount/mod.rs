use std::collections::{BTreeMap, HashMap};

use regex::Regex;

use crate::mount::condition::{parse_conditions, Condition};
use crate::mount::customization::collect_customization;
use crate::mount::export::Exporter;
use crate::mount::family::group_by_families;
use crate::mount::image::collect_dominant_colors;
use crate::mount::sources::{collect_black_market_mounts, collect_drop_mounts, collect_unavailable_mounts};
use crate::tools::casc_loader::load_dbs;
use crate::tools::db_reader::{load_item_effects, parse_csv, LookupDB};
use crate::tools::dbs;
use crate::tools::{load_config, load_listfile, BuildInfo, ProductVersion};

mod condition;
mod customization;
mod export;
mod family;
mod image;
mod wcm;
mod sources;

pub struct Mount {
    id: u32,
    spell_id: u32,
    type_id: u32,
    name: String,
    icon: String,
    item_id: Option<u32>,
    item_is_tradeable: bool,
    player_conditions: Vec<Vec<Condition>>,
    source_type: i8,
}

pub fn handle_mounts(game_version: ProductVersion) {
    let config = load_config("mount.yml");

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
    let list_file = load_listfile();

    let mut mounts = collect_mounts(&classic_version.version, &list_file);
    mounts.append(&mut collect_mounts(&build_version.version, &list_file));

    for value in config.get("ignored").unwrap().as_sequence().unwrap().iter() {
        mounts
            .remove(&(value.as_i64().unwrap() as u32))
            .expect("ignored id doesn't exist anymore in game");
    }

    let exporter = Exporter::new(config.get("export_path").unwrap().as_str().unwrap());
    exporter.export_tradable(&mounts);
    exporter.export_conditions(&mounts);
    exporter.export_families(
        &mounts,
        group_by_families(&mounts, config.get("familymap").unwrap()),
    );
    exporter.export_sources(
        &mounts,
        collect_black_market_mounts(&mounts),
        collect_unavailable_mounts(&mounts),
        collect_drop_mounts(&mounts),
    );
    exporter.export_customization(
        &mounts,
        collect_customization(&mounts, &build_version.version),
    );
    exporter.export_colors(
        &mounts,
        collect_dominant_colors(&build_version, &mounts, &list_file),
    );
}

fn collect_mounts(
    build_version: &String,
    list_file: &HashMap<i64, String>,
) -> BTreeMap<u32, Mount> {
    let mut collection: BTreeMap<u32, Mount> = BTreeMap::new();

    let mount_db: Vec<dbs::Mount> = parse_csv(build_version, "Mount.csv").unwrap();
    let player_condition_db: LookupDB<dbs::PlayerCondition> = LookupDB::new_from_data(
        parse_csv(build_version, "PlayerCondition.csv").unwrap(),
        |s: &dbs::PlayerCondition| s.id,
    );

    let icon_file_regex = Regex::new("(?i)interface/icons/(.*)\\.blp").expect("invalid regexp");
    let spell_misc_db: LookupDB<dbs::SpellMisc> = LookupDB::new_from_data(
        parse_csv(build_version, "SpellMisc.csv").unwrap(),
        |s: &dbs::SpellMisc| s.spell_id,
    );
    let item_sparse_db: LookupDB<dbs::ItemSparse> = LookupDB::new_from_data(
        parse_csv(build_version, "ItemSparse.csv").unwrap(),
        |s: &dbs::ItemSparse| s.item_id,
    );
    let item_effects_db = load_item_effects(build_version, true);

    for mount in mount_db {
        let conditions = player_condition_db.lookup(&mount.player_condition_id);
        let player_conditions = match conditions.first() {
            None => Vec::new(),
            Some(player_condition) => parse_conditions(
                player_condition.race_mask,
                player_condition.description.as_str(),
                player_condition.class_mask,
                player_condition.skill_id,
                player_condition.quest_id,
            ),
        };

        let item_id = {
            let mut item_id = None;
            for effect in item_effects_db.lookup(&mount.spell_id) {
                if effect.trigger_type == 6 {
                    let items = item_sparse_db.lookup(&effect.item_id);
                    item_id = items.first().map(|i| i.item_id);
                }
            }
            item_id
        };

        let item_is_tradeable = if item_id.is_some() {
            item_sparse_db.lookup(&item_id.unwrap()).first().map(|i| i.bonding == 0).unwrap_or(false)
        } else {
            false
        };

        let icon = {
            match spell_misc_db.lookup(&mount.spell_id).first() {
                None => "".to_string(),
                Some(spell) => match list_file.get(&spell.spell_icon_file_id) {
                    None => "".to_string(),
                    Some(file_path) => icon_file_regex
                        .captures(file_path)
                        .expect("didn't found icon name in file path")
                        .get(1)
                        .expect("didn't found icon name in file path")
                        .as_str()
                        .to_string(),
                },
            }
        };

        collection.insert(
            mount.id,
            Mount {
                id: mount.id,
                spell_id: mount.spell_id,
                type_id: mount.type_id,
                name: mount.name,
                item_id,
                icon,
                item_is_tradeable,
                player_conditions,
                source_type: mount.source_type,
            },
        );
    }

    collection
}
