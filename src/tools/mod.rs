use std::collections::HashMap;
use std::str::FromStr;

use serde_yaml::Value;

pub(crate) mod blp_reader;
pub(crate) mod db_reader;
pub(crate) mod docker_runner;
pub(crate) mod lua_export;
pub(crate) mod m2_reader;

pub(crate) fn load_config(file_name: &str) -> Value {
    let f = std::fs::File::open("config/".to_owned() + file_name).unwrap();
    serde_yaml::from_reader(f).unwrap()
}

pub(crate) fn http_get(url: &str) -> String {
    reqwest::blocking::get(url).unwrap().text().unwrap()
}

pub(crate) fn load_listfile() -> HashMap<i64, String> {
    let plain_txt = http_get(
        "https://github.com/wowdev/wow-listfile/blob/master/community-listfile.csv?raw=true",
    );

    let mut result = HashMap::new();
    for line in plain_txt.lines() {
        let chunks: Vec<&str> = line.split(";").collect();

        result.insert(
            i64::from_str(chunks.get(0).unwrap()).unwrap(),
            chunks.get(1).unwrap().to_string(),
        );
    }

    result
}
