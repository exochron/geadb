use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::str::FromStr;
use std::time::Duration;
use csv::ReaderBuilder;
use reqwest::blocking::ClientBuilder;
use serde_yaml::Value;

pub(crate) mod blp_reader;
pub mod casc_loader;
pub mod db_reader;
pub mod dbs;
pub mod lua_export;
pub(crate) mod m2_reader;

pub fn load_config(file_name: &str) -> Value {
    let f = std::fs::File::open("config/".to_owned() + file_name).unwrap();
    serde_yaml::from_reader(f).unwrap()
}

pub(crate) fn http_get(url: &str) -> String {
    let builder = reqwest::blocking::ClientBuilder::new();
    let client = builder.timeout(Duration::from_secs(500)).build().unwrap();
    client.get(url).send().unwrap().text().unwrap()
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

pub(crate) fn fetch_files(build_info: &BuildInfo, file_list: HashMap<i64, String>) {
    if file_list.is_empty() {
        return;
    }

    let base_url = if build_info.product == ProductVersion::Classic {
        "http://localhost:5001/"
    } else {
        "http://localhost:5000/"
    };

    let client = ClientBuilder::new().build().unwrap();

    for (file_id, file_path) in file_list.iter() {
        let path = format!("extract/{}/{}", build_info.version, file_path);
        let path = Path::new(&path);

        let response = client
            .get(format!(
                "{}casc/fdid?fileDataID={}&filename={}",
                base_url,
                file_id,
                path.file_name().unwrap().to_str().unwrap()
            ))
            .send()
            .unwrap();
        if response.status().is_success() {
            let data = response.bytes().unwrap();
            fs::create_dir_all(path.parent().unwrap()).expect("could not create folders");
            fs::write(path, data).unwrap_or_else(|_| panic!("could not write file: {:?}", path));
        }
    }
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
