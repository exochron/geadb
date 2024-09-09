use crate::mount::wcm::{load_wcm_black_market_mounts, load_wcm_retired_mounts};
use crate::mount::Mount;
use crate::tools::{build_http, http_get_with_client};
use regex::Regex;
use serde_json::Value;
use std::collections::BTreeMap;

const NPC_WHITELIST: [u32; 13] = [
    50245, // Dormus the Camel-Hoarder
    60491, // Sha of Anger
    62346, // Galleon
    69161, // Oondasta
    69099, // Nalak
    81001, // Nok-Karosh
    83746, // Rukhmar
    138794, // Dunegorger Kraulok
    162875, // Devmorta
    174861, // Gorged Shadehound
    178002, // Mugrem the Soul Devourer
    195353, // Breezebiter
    208029, // Doomshadow
];

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

fn filter_mounts_by_names(
    mounts: &BTreeMap<u32, Mount>,
    names: Vec<String>,
) -> Vec<u32> {
    let mut result: Vec<u32> = Vec::new();

    for (_, mount) in mounts.iter() {
        let lowered_name = mount.name.to_lowercase();
        names.contains(&lowered_name).then(|| result.push(mount.id));
    }

    result
}

pub fn collect_black_market_mounts(
    mounts: &BTreeMap<u32, Mount>,
) -> Vec<u32> {
    filter_mounts_by_names(mounts, load_wcm_black_market_mounts())
}
pub fn collect_unavailable_mounts(
    mounts: &BTreeMap<u32, Mount>,
) -> Vec<u32> {
    filter_mounts_by_names(mounts, load_wcm_retired_mounts())
}

pub fn collect_drop_mounts(
    mounts: &BTreeMap<u32, Mount>,
) -> BTreeMap<u32, DropData> {
    let listview_reg = Regex::new("(?s)new Listview\\(\\{\\s*?template: 'npc',\\s*?id: 'dropped-by',.*?data: (\\[.*?\\]),\\s*?\\}\\)").unwrap();
    let mapper_reg = Regex::new("(?i)var g_mapperData = (\\{.*?\\});").unwrap();

    let http_client = build_http();


    let mut result = BTreeMap::new();
    for (_, mount) in mounts.iter() {
        let debug = false;
        if debug {
            println!("{}: {:?} {:?}", mount.id, mount.item_id, mount.source_type)
        }

        // Rare drops start with Time Lost Proto Drake
        // later: add treasure drops (source_type=5)
        // later: add egg data ?
        if mount.id > 264 && mount.item_id.is_some() && (mount.source_type == 0 || [1257].contains(&mount.id)) {
            let item_html = http_get_with_client(&http_client, format!("https://www.wowhead.com/item={}", mount.item_id.unwrap()).as_str());
            let listviews = listview_reg.captures_iter(item_html.as_str());
            let listviews2 = listview_reg.captures_iter(item_html.as_str());
            if debug {
                println!("list count: {}", listview_reg.captures_iter(item_html.as_str()).count());
            }
            if listviews2.count() == 1 {
                for drop_data in listviews {
                    let drop_data: Value = serde_json::from_str(drop_data.get(1).unwrap().as_str()).unwrap();
                    let npc = drop_data.as_array().unwrap().first().unwrap().as_object().unwrap();
                    let npc_id = npc.get("id").unwrap().as_u64().unwrap() as u32;
                    let classification = npc.get("classification").unwrap().as_u64().unwrap();
                    if debug {
                        println!("classification: {}", classification);
                    }
                    if classification == 2 || classification == 4 || NPC_WHITELIST.contains(&npc_id) || (classification == 0 && false == npc.contains_key("boss")) { // is rare elite || rare || world boss? || zone mob
                        println!("found npc {} for mount {}", npc_id, mount.id);

                        let mut map_position = None;
                        let npc_html = http_get_with_client(&http_client, format!("https://www.wowhead.com/npc={}", npc_id).as_str());
                        for position_data in mapper_reg.captures_iter(npc_html.as_str()) {
                            let position_data: Value = serde_json::from_str(position_data.get(1).unwrap().as_str()).unwrap();
                            let position_data = position_data.as_object().unwrap().values().next().unwrap();
                            let position_data = position_data.as_array().unwrap().first().unwrap();
                            let position_data = position_data.as_object().unwrap();
                            let map_id = position_data.get("uiMapId");
                            if map_id.is_some() {
                                let map_id = map_id.unwrap().as_u64().unwrap() as u32;
                                let coords = position_data.get("coords").unwrap().as_array().unwrap().first().unwrap().as_array().unwrap();
                                let map_x: u16 = (coords.get(0).unwrap().as_f64().unwrap() * 100.0).round() as u16;
                                let map_y: u16 = (coords.get(1).unwrap().as_f64().unwrap() * 100.0).round() as u16;
                                map_position = Some(MapPosition { map_id, map_x, map_y });
                                break;
                            }
                        }

                        let count = npc.get("count").map(|a| a.as_u64().unwrap()).unwrap_or_default();
                        let total = npc.get("outof").map(|a| a.as_u64().unwrap()).unwrap_or_default();
                        let drop_chance = if total > 0 {
                            Some(count as f32 / total as f32 * 100.0)
                        } else {
                            None
                        };

                        result.insert(mount.id, DropData{ npc_id, map_position, drop_chance });
                        break;
                    }
                }
            }
        }
    }

    result
}