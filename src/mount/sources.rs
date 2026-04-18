use crate::mount::wcm::{load_wcm_black_market_mounts, load_wcm_retired_mounts};
use crate::mount::Mount;
use crate::tools::db_reader::{parse_csv, LookupDB};
use crate::tools::{build_http, dbs, http_get_with_client, ProductVersion};
use regex::Regex;
use serde_json::Value;
use std::collections::{BTreeMap, HashMap};

pub(crate) struct MapPosition {
    pub map_id: u32,
    pub map_x: u16,
    pub map_y: u16,
}

pub(crate) struct DropData {
    pub npc_id: u32,
    pub map_position: Option<MapPosition>,
    pub drop_chance: Option<f32>,
}

fn filter_mounts_by_names(mounts: &BTreeMap<u32, Mount>, names: Vec<String>) -> Vec<u32> {
    let mut result: Vec<u32> = Vec::new();

    for (_, mount) in mounts.iter() {
        let lowered_name = mount.name.to_lowercase();
        names.contains(&lowered_name).then(|| result.push(mount.id));
    }

    result
}

pub fn collect_black_market_mounts(mounts: &BTreeMap<u32, Mount>) -> Vec<u32> {
    filter_mounts_by_names(mounts, load_wcm_black_market_mounts())
}
pub fn collect_unavailable_mounts(mounts: &BTreeMap<u32, Mount>) -> Vec<u32> {
    filter_mounts_by_names(mounts, load_wcm_retired_mounts())
}

pub fn collect_drop_mounts(
    mounts: &BTreeMap<u32, Mount>,
    game_version: &ProductVersion,
) -> BTreeMap<u32, DropData> {
    // item -> npc
    let mut MANUAL_NPCS:HashMap<u64, u64> = HashMap::new();
    MANUAL_NPCS.insert(89783u64, 62346u64); // Son of Galleon
    MANUAL_NPCS.insert(94228u64, 69161u64); // Cobalt Primordial Direhorn
    MANUAL_NPCS.insert(94230u64, 69841u64); // Amber Primordial Direhorn
    MANUAL_NPCS.insert(94229u64, 69769u64); // Slate Primordial Direhorn
    MANUAL_NPCS.insert(94231u64, 69842u64); // Jade Primordial Direhorn
    MANUAL_NPCS.insert(104269u64, 73167u64); // Thundering Onyx Cloud Serpent
    MANUAL_NPCS.insert(116771u64, 83746u64); // Solar Spirehawk
    MANUAL_NPCS.insert(163576u64, 128665u64); // Dune Scavenger
    MANUAL_NPCS.insert(163575u64, 127915u64); // Leaping Veinseeker
    MANUAL_NPCS.insert(152903u64, 126040u64); // Biletooth Gnasher
    MANUAL_NPCS.insert(163574u64, 129995u64); // Terrified Pack Mule
    MANUAL_NPCS.insert(163573u64, 132160u64); // Goldenmane
    MANUAL_NPCS.insert(168823u64, 154342u64); // Rusty Mechanocrawler
    MANUAL_NPCS.insert(184062u64, 162853u64); // Battle-Bound Warhound
    MANUAL_NPCS.insert(185973u64, 177330u64); // Hand of Bahmethra
    MANUAL_NPCS.insert(192772u64, 200911u64); // Ancient Salamanther
    MANUAL_NPCS.insert(212645u64, 208029u64); // Clayscale Hornstrider

    let mounts_reg = Regex::new("(?i)var listviewitems = (\\[.*?\\]);").unwrap();
    let drops_listview_reg =
        Regex::new("(?i)new Listview\\((\\{\\s*?template: 'item',\\s*?id: 'drops',.*?\\})\\)")
            .unwrap();
    let mapper_reg = Regex::new("(?i)var g_mapperData = (\\{.*?\\});").unwrap();

    let droppedby_listview_reg = Regex::new("(?s)new Listview\\(\\{\\s*?template: 'npc',\\s*?id: 'dropped-by',.*?data: (\\[.*?\\]),\\s*?\\}\\)").unwrap();

    let http_client = build_http();
    let head_server = match (game_version) {
        ProductVersion::Ptr => "ptr/",
        ProductVersion::XPtr => "ptr-2/",
        _ => "",
    };

    let mount_items_html = http_get_with_client(
        &http_client,
        format!(
            "https://www.wowhead.com/{}items/miscellaneous/mounts",
            head_server
        )
        .as_str(),
    );
    let mount_items_capture = mounts_reg.captures(mount_items_html.as_str()).unwrap();
    let mount_items_json = mount_items_capture.get(1).unwrap().as_str();
    let mount_list_data: Value = serde_json5::from_str(mount_items_json).unwrap();
    let mount_item_list = mount_list_data.as_array().unwrap();

    let mut result = BTreeMap::new();
    for (_, mount) in mounts.iter() {
        let debug = false;
        if debug {
            println!("{}: {:?} {:?}", mount.id, mount.item_id, mount.source_type)
        }

        // Rare drops start with Time Lost Proto Drake
        // later: add treasure drops (source_type=5)
        // later: add egg data ?
        if mount.id > 264
            && mount.item_id.is_some()
            && (mount.source_type == 0 || [1257].contains(&mount.id))
        {
            let item_id = mount.item_id.unwrap() as u64;
            let item_data = mount_item_list
                .iter()
                .find(|v| v.as_object().unwrap().get("id").unwrap().as_u64().unwrap() == item_id);
            if let Some(item_data) = item_data {
                let item_data = item_data.as_object().unwrap();
                if item_data.contains_key("source") {
                    let source_list = item_data.get("source").unwrap().as_array().unwrap();
                    let source_list: Vec<u64> =
                        source_list.iter().map(|v| v.as_u64().unwrap()).collect();
                    if source_list.contains(&2) {
                        let mut npc_id = None;
                        if MANUAL_NPCS.contains_key(&item_id) {
                            npc_id = MANUAL_NPCS.get(&item_id).copied();
                        }

                        if item_data.contains_key("sourcemore") {
                            let sourcemore = item_data
                                .get("sourcemore")
                                .unwrap()
                                .as_array()
                                .unwrap()
                                .first();
                            if let Some(sourcemore) = sourcemore {
                                let sourcemore_data = sourcemore.as_object().unwrap();
                                if sourcemore_data.contains_key("t")
                                    && sourcemore_data.get("t").unwrap().as_u64().unwrap() == 1
                                {
                                    npc_id =
                                        Some(sourcemore_data.get("ti").unwrap().as_u64().unwrap());
                                }
                            }
                        }

                        if npc_id.is_none() && mount.id > 2222 {
                            println!("Fallback fetching item data directly for {} {} {}", item_id, mount.id, mount.name);
                            let item_html = http_get_with_client(
                                &http_client,
                                format!("https://www.wowhead.com/{}item={}",head_server, item_id).as_str(),
                            );

                            let item_dropped_json = droppedby_listview_reg.captures(item_html.as_str());
                            if item_dropped_json.is_some() {
                                let item_dropped_json = item_dropped_json.unwrap().get(1).unwrap().as_str();
                                let item_dropped_data: Value = serde_json5::from_str(item_dropped_json).unwrap();

                                let npc_data = item_dropped_data
                                    .as_array()
                                    .unwrap()
                                    .first()
                                    .unwrap()
                                    .as_object()
                                    .unwrap();

                                let classification = npc_data.get("classification").unwrap().as_u64().unwrap();
                                if classification == 2
                                    || classification == 4
                                    || (classification == 0 && false == npc_data.contains_key("boss"))
                                {
                                    npc_id = npc_data.get("id").unwrap().as_u64();
                                }
                            }
                        }

                        if npc_id.is_some() {
                            let npc_id = npc_id.unwrap() as u32;

                            println!("found npc {} for mount {} {}", npc_id, mount.id, mount.name);

                            let mut map_position = None;
                            let mut drop_chance = None;
                            let npc_html = http_get_with_client(
                                &http_client,
                                format!("https://www.wowhead.com/{}npc={}", head_server, npc_id)
                                    .as_str(),
                            );

                            let listviews = drops_listview_reg.captures(npc_html.as_str());
                            if let Some(listviews) = listviews {
                                let listviews_json = listviews.get(1).unwrap().as_str();
                                let listviews_json =
                                    listviews_json.replace("name: WH.TERMS.drops,", "");
                                let listviews_json =
                                    listviews_json.replace("tabs: tabsRelated,", "");
                                let listviews_json = listviews_json.replace(
                                    "computeDataFunc: Listview.funcBox.initLootTable,",
                                    "",
                                );
                                if debug {
                                    println!("{}", listviews_json);
                                }
                                let item_list: Value =
                                    serde_json5::from_str(listviews_json.as_str()).unwrap();
                                let item_list = item_list
                                    .as_object()
                                    .unwrap()
                                    .get("data")
                                    .unwrap()
                                    .as_array()
                                    .unwrap();
                                let item_data = item_list.iter().find(|v| {
                                    v.as_object().unwrap().get("id").unwrap().as_u64().unwrap()
                                        == item_id
                                });
                                if let Some(drop_data) = item_data {
                                    if debug {
                                        println!("{}", drop_data);
                                    }
                                    let drop_data = drop_data.as_object().unwrap();
                                    if drop_data.contains_key("modes") {
                                        let drop_modes = drop_data
                                            .get("modes")
                                            .unwrap()
                                            .as_object()
                                            .unwrap()
                                            .get("mode")
                                            .unwrap()
                                            .as_array()
                                            .unwrap();
                                        if drop_modes.len() > 1
                                            || drop_modes.first().unwrap().as_u64().unwrap() != 0
                                        {
                                            // abort on instance drop
                                            continue;
                                        }
                                    }

                                    let count = drop_data
                                        .get("count")
                                        .map(|a| a.as_u64().unwrap())
                                        .unwrap_or_default();
                                    let total = drop_data
                                        .get("outof")
                                        .map(|a| a.as_u64().unwrap())
                                        .unwrap_or_default();
                                    if total > 0 {
                                        drop_chance = Some(count as f32 / total as f32 * 100.0)
                                    };
                                }
                            }

                            for position_data in mapper_reg.captures_iter(npc_html.as_str()) {
                                let position_data: Value =
                                    serde_json::from_str(position_data.get(1).unwrap().as_str())
                                        .unwrap();
                                let position_data =
                                    position_data.as_object().unwrap().values().next().unwrap();
                                let position_data =
                                    position_data.as_array().unwrap().first().unwrap();
                                let position_data = position_data.as_object().unwrap();
                                let map_id = position_data.get("uiMapId");
                                if map_id.is_some() {
                                    let map_id = map_id.unwrap().as_u64().unwrap() as u32;
                                    let coords = position_data
                                        .get("coords")
                                        .unwrap()
                                        .as_array()
                                        .unwrap()
                                        .first()
                                        .unwrap()
                                        .as_array()
                                        .unwrap();
                                    let map_x: u16 =
                                        (coords.get(0).unwrap().as_f64().unwrap() * 100.0).round()
                                            as u16;
                                    let map_y: u16 =
                                        (coords.get(1).unwrap().as_f64().unwrap() * 100.0).round()
                                            as u16;
                                    map_position = Some(MapPosition {
                                        map_id,
                                        map_x,
                                        map_y,
                                    });
                                    break;
                                }
                            }

                            if drop_chance.is_some() || map_position.is_some() {
                                result.insert(
                                    mount.id,
                                    DropData {
                                        npc_id,
                                        map_position,
                                        drop_chance,
                                    },
                                );
                            }
                        }
                    }
                }
            }
        }
    }

    result
}

pub fn collect_featsofstrength_mounts(
    mounts: &BTreeMap<u32, Mount>,
    build_version: &String,
) -> BTreeMap<u32, u32> {
    let achievement_category_db: LookupDB<dbs::AchievementCategory> = LookupDB::new_from_data(
        parse_csv(build_version, "AchievementCategory.csv").unwrap(),
        |s: &dbs::AchievementCategory| s.parent,
    );
    let mut fos_ids: Vec<u32> = achievement_category_db
        .lookup(&81)
        .iter()
        .map(|c| c.id)
        .collect();
    fos_ids.push(81);

    let achievement_db: LookupDB<dbs::Achievement> = LookupDB::new_from_data(
        parse_csv(build_version, "Achievement.csv").unwrap(),
        |s: &dbs::Achievement| s.id,
    );

    let mut result = BTreeMap::new();

    for (mount_id, mount) in mounts {
        if mount.item_id.is_some() {
            let item_id = mount.item_id.unwrap();
        }
    }

    result
}
