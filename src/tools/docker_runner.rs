use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use std::process::Command;

use regex::Regex;

use crate::tools::ProductVersion;

pub struct DockerRunner {
    game_version: ProductVersion,
    pub build_version: String,
}

impl DockerRunner {
    pub fn new(game_version: ProductVersion) -> Self {
        Self {
            game_version,
            build_version: String::new(),
        }
    }

    fn add_extract_product<'a>(&'a self, mut args: Vec<&'a str>) -> Vec<&str> {
        match self.game_version {
            ProductVersion::Retail => {}
            ProductVersion::Ptr => {
                args.push("--product=wowt");
            }
            ProductVersion::XPtr => {
                args.push("--product=wowxptr");
            }
            ProductVersion::Classic => {
                args.push("--product=wow_classic");
            }
        }

        args
    }

    pub(crate) fn fetch_files(&self, file_list: HashMap<i64, String>) {
        if file_list.is_empty() {
            return;
        }

        let mut txt_file = File::create("extract/download.txt").unwrap();
        for (file_id, file_path) in file_list.iter() {
            writeln!(txt_file, "{}, {}", file_id, file_path).expect("couldn't write to file");
        }

        let mut args = vec!["compose", "run", "--rm", "extract_files"];
        args = self.add_extract_product(args);

        Command::new("docker")
            .args(args)
            .spawn()
            .expect("could not start converting db files")
            .wait_with_output()
            .expect("could not start converting db files");
    }

    fn parse_build_version(&mut self, output: String) {
        let matched = Regex::new("(?i)build version: (\\d+\\.\\d+\\.\\d+\\.\\d+)")
            .expect("invalid regexp")
            .captures(output.as_str())
            .expect("didn't found build version in output! is docker running?");
        self.build_version = String::from(
            matched
                .get(1)
                .expect("didn't found build version in output")
                .as_str(),
        );
    }

    pub(crate) fn fetch_mount_dbfiles(&mut self) {
        let mut args = vec!["compose", "run", "--rm", "extract_mount_db"];
        args = self.add_extract_product(args);

        let output = Command::new("docker")
            .args(args)
            .output()
            .expect("could not start loading mount db files");

        self.parse_build_version(
            String::from_utf8(output.stdout).expect("couldn't convert output into string"),
        );
    }

    pub fn fetch_toy_dbfiles(&mut self) {
        let mut args = vec!["compose", "run", "--rm", "extract_toy_db"];
        args = self.add_extract_product(args);

        let output = Command::new("docker")
            .args(args)
            .output()
            .expect("could not start loading toy db files");

        self.parse_build_version(
            String::from_utf8(output.stdout).expect("couldn't convert output into string"),
        );
    }

    pub fn convert_dbfiles_into_csv(&self) {
        let mut args = vec!["compose", "run", "--rm", "convert_dbs"];
        args.push(match self.game_version {
            ProductVersion::Retail => "/game/DBCache.bin",
            ProductVersion::Ptr => "/game/ptr.bin",
            ProductVersion::XPtr => "/game/xptr.bin",
            ProductVersion::Classic => "/game/classic.bin",
        });
        let extracted_path = "/out/".to_string()
            + self.build_version.as_str()
            + "/DBFilesClient".to_string().as_str();
        args.push(&*extracted_path);

        Command::new("docker")
            .args(args)
            .spawn()
            .expect("could not start converting db files")
            .wait_with_output()
            .expect("could not start converting db files");
    }
}
