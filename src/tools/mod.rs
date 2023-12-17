use std::collections::HashMap;
use std::str::FromStr;

use serde_yaml::Value;

pub(crate) mod blp_reader;
pub mod db_reader;
pub mod docker_runner;
pub mod lua_export;
pub(crate) mod m2_reader;

pub fn load_config(file_name: &str) -> Value {
    let f = std::fs::File::open("config/".to_owned() + file_name).unwrap();
    serde_yaml::from_reader(f).unwrap()
}

pub(crate) fn http_get(url: &str) -> String {
    reqwest::blocking::get(url).unwrap().text().unwrap()
}

pub(crate) fn load_listfile() -> HashMap<i64, String> {
    let plain_txt = http_get(
        "https://github.com/wowdev/wow-listfile/releases/latest/download/community-listfile.csv",
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

#[derive(PartialEq, Eq)]
pub enum GameVersion {
    Retail,
    Ptr,
    XPtr,
    Classic,
}

pub(crate) fn determine_game_version_from_cli() -> GameVersion {
    match std::env::args().nth(1) {
        None => GameVersion::Retail,
        Some(value) => match value.as_str() {
            "--ptr" => GameVersion::Ptr,
            "--xptr" => GameVersion::XPtr,
            _ => GameVersion::Retail,
        },
    }
}
