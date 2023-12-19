use std::collections::HashMap;
use std::str::FromStr;

use csv::ReaderBuilder;
use serde_yaml::Value;

pub(crate) mod blp_reader;
pub mod casc_loader;
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
pub enum ProductVersion {
    Retail,
    Ptr,
    XPtr,
    Classic,
}

impl ProductVersion {
    pub fn as_str(&self) -> &'static str {
        match self {
            ProductVersion::Retail => "wow",
            ProductVersion::Ptr => "wowt",
            ProductVersion::XPtr => "wowxptr",
            ProductVersion::Classic => "wow_classic",
        }
    }

    pub fn determine_from_cli() -> Self {
        match std::env::args().nth(1) {
            None => ProductVersion::Retail,
            Some(value) => match value.as_str() {
                "--ptr" => ProductVersion::Ptr,
                "--xptr" => ProductVersion::XPtr,
                _ => ProductVersion::Retail,
            },
        }
    }
}

pub struct BuildInfo {
    pub product: ProductVersion,
    pub version: String,
}

impl BuildInfo {
    pub fn parse_build_info(game_path: &str, product: ProductVersion) -> Result<BuildInfo, String> {
        let mut reader = ReaderBuilder::new()
            .delimiter(b'|')
            .from_path(format!("{}/.build.info", game_path))
            .unwrap();
        let column_count = reader.headers().unwrap().len();

        for record in reader.records() {
            if record.as_ref().unwrap().get(column_count - 1).unwrap() == product.as_str() {
                return Ok(BuildInfo {
                    product,
                    version: record
                        .as_ref()
                        .unwrap()
                        .get(column_count - 3)
                        .unwrap()
                        .to_string(),
                });
            }
        }

        Err("game version not detected".parse().unwrap())
    }
}
