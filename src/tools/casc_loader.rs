use std::fs::{create_dir_all, File};
use std::io::copy;
use std::path::Path;

use serde_yaml::Value;

use crate::tools::{BuildInfo, ProductVersion};

pub fn load_dbs(config: &Value, build_info: &BuildInfo) {
    let client = reqwest::blocking::ClientBuilder::new().build().unwrap();

    for db_file in config.get("db_files").unwrap().as_sequence().unwrap() {
        let db_file = db_file.as_str().unwrap();

        let wow_tools_url = match build_info.product {
            ProductVersion::Classic => "http://127.0.0.1:5001",
            _ => "http://127.0.0.1:5000",
        }
        .to_string();

        let url = format!(
            "{}/dbc/export/?name={}&build={}&useHotfixes=true",
            wow_tools_url,
            db_file.to_lowercase(),
            build_info.version
        );
        let response = client.get(url).send();
        match response {
            Ok(response) => {
                if response.status().is_success() {
                    let data = response.text().unwrap();

                    let file_name = format!(
                        "extract/{}/DBFilesClient/{}.csv",
                        build_info.version, db_file
                    );
                    let file_path = Path::new(file_name.as_str());
                    create_dir_all(file_path.parent().unwrap()).expect("could not create directories");
                    let mut dest = File::create(file_path).expect("could not write file");

                    copy(&mut data.as_bytes(), &mut dest).expect("could download file");
                }
            }
            Err(e) => println!("http-err: {}", e),
        }
    }
}
