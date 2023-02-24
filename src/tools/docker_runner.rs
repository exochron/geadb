use regex::Regex;
use std::process::{Command, Stdio};

pub struct DockerRunner {
    pub build_version: String,
}

impl DockerRunner {
    pub(crate) fn new() -> Self {
        Self {
            build_version: String::new(),
        }
    }

    pub(crate) fn fetch_mount_dbfiles(&mut self) {
        let output = Command::new("docker")
            .args(["compose", "run", "--rm", "extract_mount_db"])
            .stdout(Stdio::piped())
            .output()
            .expect("could not start loading mount db files");

        let stdout = String::from_utf8(output.stdout).expect("couldn't convert output into string");

        let matched = Regex::new("(?i)build version: (\\d+\\.\\d+\\.\\d+\\.\\d+)")
            .expect("invalid regexp")
            .captures(stdout.as_str())
            .expect("didn't found build version in output");
        self.build_version = String::from(
            matched
                .get(1)
                .expect("didn't found build version in output")
                .as_str(),
        )
    }

    pub(crate) fn convert_dbfiles_into_csv(&self) {
        Command::new("docker")
            .args([
                "compose",
                "run",
                "--rm",
                "convert_dbs",
                "/game/DBCache.bin",
                &("/out/".to_owned() + self.build_version.as_str() + "/DBFilesClient"),
            ])
            .spawn()
            .expect("could not start converting db files")
            .wait_with_output()
            .expect("could not start converting db files");
    }
}
