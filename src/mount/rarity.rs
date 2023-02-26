use std::collections::HashMap;

use regex::Regex;
use serde_json::Value;

use crate::tools::http_get;

pub fn load_rarities() -> HashMap<i64, f64> {
    let mut result = HashMap::new();

    let html = http_get("https://rarityraider.com/en/mounts");

    let json_reg = Regex::new("data-page=\"(\\{.*\\})\"").expect("invalid regexp");
    let json_data = json_reg
        .captures(html.as_str())
        .expect("could't parse json")
        .get(1)
        .unwrap()
        .as_str()
        .replace("&quot;", "\"");

    let json: Value = serde_json::from_str(json_data.as_str()).unwrap();
    let mounts = json
        .get("props")
        .unwrap()
        .get("mounts")
        .unwrap()
        .as_array()
        .unwrap();
    for mount in mounts.iter() {
        match mount.get("sa_rarity").unwrap().as_f64() {
            None => None,
            Some(rarity) => result.insert(mount.get("ext_id").unwrap().as_i64().unwrap(), rarity),
        };
    }

    result
}
