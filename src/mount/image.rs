use std::collections::{BTreeMap, HashMap};
use std::path::Path;

use kmeans_colors::{get_kmeans, Kmeans};
use palette::{FromColor, Lab, Srgb};

use crate::mount::Mount;
use crate::tools::blp_reader::BLPReader;
use crate::tools::db_reader::DBReader;
use crate::tools::docker_runner::DockerRunner;

fn collect_files(
    build_version: &String,
    list_file: &HashMap<i64, String>,
) -> (HashMap<i64, String>, HashMap<i64, Vec<String>>) {
    let mut display_csv = DBReader::new(build_version, "MountXDisplay.csv");
    let mut creature_csv = DBReader::new(build_version, "CreatureDisplayInfo.csv");

    let mut files_to_load: HashMap<i64, String> = HashMap::new();
    let mut mount_files: HashMap<i64, Vec<String>> = HashMap::new();

    for record in display_csv.reader.records() {
        let row = record.unwrap();
        let display_id: i64 = row.get(1).unwrap().parse().unwrap();
        let mount_id: i64 = row.get(3).unwrap().parse().unwrap();

        for i in 0..4 {
            let file_id: i64 = creature_csv
                .fetch_field(&display_id, 24 + i)
                .unwrap_or("0".to_string())
                .parse()
                .unwrap();
            if file_id > 0 && list_file.contains_key(&file_id) {
                let file_path = list_file.get(&file_id).unwrap().clone();
                if !Path::new(&("extract/".to_string() + build_version + "/" + &file_path)).exists()
                {
                    files_to_load.insert(file_id, file_path.clone());
                }
                mount_files
                    .entry(mount_id)
                    .or_insert(Vec::new())
                    .push(file_path.clone());
            }
        }
    }

    (files_to_load, mount_files)
}

fn determine_dominant_colors(pixels: Vec<Lab>, seed: &i64) -> Vec<Srgb<u8>> {
    let mut kmean = Kmeans::new();
    for i in 0..3 {
        let run_result = get_kmeans(5, 20, 5.0, false, &pixels, (seed + i) as u64);
        if run_result.score < kmean.score {
            kmean = run_result;
        }
    }

    // Convert indexed colors back to Srgb<u8> for output
    kmean
        .centroids
        .iter()
        .map(|x| Srgb::from_color(*x).into_format())
        .collect()
}

pub fn collect_dominant_colors(
    build_version: &String,
    mounts: &BTreeMap<i64, Mount>,
    list_file: &HashMap<i64, String>,
) -> HashMap<i64, Vec<Srgb<u8>>> {
    let mount_files = {
        let (files_to_load, mount_files) = collect_files(build_version, list_file);
        DockerRunner::new().fetch_files(files_to_load);
        mount_files
    };

    let mut result = HashMap::new();

    for mount in mounts.values() {
        match mount_files.get(&mount.id) {
            None => {}
            Some(file_paths) => {
                let mut pixels: Vec<Lab> = Vec::new();

                for file_path in file_paths {
                    let mut file_pixels = BLPReader::new(build_version, file_path).convert_to_lab();
                    pixels.append(&mut file_pixels);
                }

                if !pixels.is_empty() {
                    result.insert(mount.id, determine_dominant_colors(pixels, &mount.id));
                }
            }
        };
    }

    result
}
