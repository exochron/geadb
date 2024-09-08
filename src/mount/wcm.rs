use std::collections::HashMap;
use regex::Regex;
use crate::tools::http_get;

const CATEGORY_REG: &str = "(?si)<h5><a id='(.*?)'>.*?</div>\\s+</span>";
const ITEM_REG: &str = "(?si)<img class='thumbimage' src='.*?' alt='(.*?)' />";

fn parse_name(
    mount_name: &str,
) -> String {
    html_escape::decode_html_entities(mount_name).to_string()
        .to_lowercase()
        .replace(" [horde]", "")
        .replace(" [alliance]", "")
}

pub fn load_wcm_families() -> HashMap<String, String> {
    let mut result = HashMap::new();

    let html = http_get("https://www.warcraftmounts.com/gallery.php");

    let category_reg = Regex::new(CATEGORY_REG).unwrap();
    let item_reg = Regex::new(ITEM_REG).unwrap();

    for category_cap in category_reg.captures_iter(html.as_str()) {
        let category = category_cap.get(1).unwrap().as_str().to_string();

        for item_cap in item_reg.captures_iter(category_cap.get(0).unwrap().as_str()) {
            result.insert(parse_name(item_cap.get(1).unwrap().as_str()), category.clone());
        }
    }

    result
}

fn load_only_names(url: &str) -> Vec<String>
{
    let mut result = Vec::new();

    let html = http_get(url);
    let item_reg = Regex::new(ITEM_REG).unwrap();
    for item_cap in item_reg.captures_iter(html.as_str()) {
        result.push(parse_name(item_cap.get(1).unwrap().as_str()));
    }

    result
}

pub fn load_wcm_black_market_mounts() -> Vec<String> {
    load_only_names("https://www.warcraftmounts.com/bmah.php")
}
pub fn load_wcm_retired_mounts() -> Vec<String> {
    load_only_names("https://www.warcraftmounts.com/retired.php")
}