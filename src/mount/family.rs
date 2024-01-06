use std::collections::{BTreeMap, HashMap};

use regex::Regex;
use serde_yaml::Value;

use crate::mount::Mount;
use crate::tools::http_get;

pub struct FamilyNode {
    pub sub_nodes: BTreeMap<String, FamilyNode>,
    pub mount_ids: Vec<u32>,
}

impl FamilyNode {
    fn new() -> Self {
        Self {
            sub_nodes: BTreeMap::new(),
            mount_ids: Vec::new(),
        }
    }
}

fn load_wcm_families() -> HashMap<String, String> {
    let mut result = HashMap::new();

    let html = http_get("https://www.warcraftmounts.com/gallery.php");

    let category_reg = Regex::new("(?si)<h5><a id='(.*?)'>.*?</div>\\s+</span>").unwrap();
    let item_reg = Regex::new("(?si)<img class='thumbimage' src='.*?' alt='(.*?)' />").unwrap();

    for category_cap in category_reg.captures_iter(html.as_str()) {
        let category = category_cap.get(1).unwrap().as_str().to_string();

        for item_cap in item_reg.captures_iter(category_cap.get(0).unwrap().as_str()) {
            let mut mount_name = item_cap.get(1).unwrap().as_str().to_string();
            mount_name = html_escape::decode_html_entities(mount_name.as_str()).to_string();
            mount_name = mount_name
                .to_lowercase()
                .replace(" [horde]", "")
                .replace(" [alliance]", "");
            result.insert(mount_name, category.clone());
        }
    }

    result
}

fn seq_to_string_vec(json: &Value) -> Vec<String> {
    json.as_sequence()
        .unwrap()
        .iter()
        .map(|el| el.as_str().unwrap().to_string())
        .collect()
}

fn seq_to_int_vec(json: &Value) -> Vec<u32> {
    json.as_sequence()
        .unwrap()
        .iter()
        .map(|el| el.as_i64().unwrap() as u32)
        .collect()
}

fn val_to_string(json: &Value) -> String {
    json.as_str().unwrap().to_string()
}

fn match_by_family(
    map_config: &Value,
    wcm_family: &String,
    mount_icon: &String,
) -> Vec<(String, Option<String>)> {
    let mut result = Vec::new();
    for category in map_config.as_sequence().unwrap().iter() {
        let cat_name = category.get("name").unwrap().as_str().unwrap().to_string();

        if match category.get("wcm") {
            None => false,
            Some(families) => {
                seq_to_string_vec(families).contains(wcm_family)
                    && match category.get("icons") {
                        None => true,
                        Some(icons) => seq_to_string_vec(icons)
                            .iter()
                            .any(|s| mount_icon.contains(s.as_str())),
                    }
            }
        } {
            result.push((cat_name.clone(), None));
        }

        match category.get("subfamily") {
            None => {}
            Some(subfamily) => {
                for subcategory in subfamily.as_sequence().unwrap().iter() {
                    if match subcategory.get("wcm") {
                        None => false,
                        Some(families) => {
                            seq_to_string_vec(families).contains(wcm_family)
                                && match subcategory.get("icons") {
                                    None => true,
                                    Some(icons) => seq_to_string_vec(icons)
                                        .iter()
                                        .any(|s| mount_icon.contains(s.as_str())),
                                }
                        }
                    } {
                        result.push((
                            cat_name.clone(),
                            Some(val_to_string(subcategory.get("name").unwrap())),
                        ));
                    }
                }
            }
        }
    }

    result
}

fn match_by_id(mount_id: &u32, map_config: &Value) -> Vec<(String, Option<String>)> {
    let mut result = Vec::new();

    for category in map_config.as_sequence().unwrap().iter() {
        let cat_name = category.get("name").unwrap().as_str().unwrap().to_string();

        if category
            .get("ids")
            .map_or(false, |ids| seq_to_int_vec(ids).contains(mount_id))
        {
            result.push((cat_name.clone(), None));
        }

        match category.get("subfamily") {
            None => {}
            Some(subfamily) => {
                for subcategory in subfamily.as_sequence().unwrap().iter() {
                    if subcategory
                        .get("ids")
                        .map_or(false, |ids| seq_to_int_vec(ids).contains(mount_id))
                    {
                        result.push((
                            cat_name.clone(),
                            Some(val_to_string(subcategory.get("name").unwrap())),
                        ));
                    }
                }
            }
        };
    }

    result
}

pub fn group_by_families(
    mounts: &BTreeMap<u32, Mount>,
    map_config: &Value,
) -> BTreeMap<String, FamilyNode> {
    let mut result = BTreeMap::new();

    let wcm_map = load_wcm_families();

    for (_, mount) in mounts.iter() {
        let lowered_name = mount.name.to_lowercase();

        let matches_by_id = match_by_id(&mount.id, map_config);
        let matches = if matches_by_id.is_empty() {
            match_by_family(
                map_config,
                wcm_map
                    .get(lowered_name.as_str())
                    .unwrap_or(&"".to_string()),
                &mount.icon,
            )
        } else {
            matches_by_id
        };

        if !matches.is_empty() {
            for (main_cat, sub_cat) in matches {
                let main_node = result.entry(main_cat).or_insert(FamilyNode::new());
                let node = match sub_cat {
                    None => main_node,
                    Some(sub_cat) => main_node
                        .sub_nodes
                        .entry(sub_cat)
                        .or_insert(FamilyNode::new()),
                };
                node.mount_ids.push(mount.id);
            }
        } else if wcm_map.contains_key(lowered_name.as_str()) {
            println!(
                "no family found for {} {} {} {}",
                mount.id, mount.spell_id, mount.name, mount.icon
            )
        } else {
            println!(
                "mount not available on warcraftmounts {} {} {} {}",
                mount.id, mount.spell_id, mount.name, mount.icon
            )
        }
    }

    result
}
