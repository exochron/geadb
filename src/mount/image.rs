use std::collections::{BTreeMap, HashMap};
use std::path::Path;

use kmeans_colors::{get_kmeans, Kmeans};
use palette::{FromColor, Lab, Srgb};

use crate::mount::Mount;
use crate::tools::blp_reader::BLPReader;
use crate::tools::db_reader::DBReader;
use crate::tools::docker_runner::DockerRunner;
use crate::tools::m2_reader::M2Reader;

fn collect_files(
    build_version: &String,
    docker: DockerRunner,
    list_file: &HashMap<i64, String>,
) -> HashMap<i64, Vec<String>> {
    let mut display_csv = DBReader::new(build_version, "MountXDisplay.csv").unwrap();
    let mut creature_csv = DBReader::new(build_version, "CreatureDisplayInfo.csv").unwrap();
    let mut creature_model_csv = DBReader::new(build_version, "CreatureModelData.csv").unwrap();

    let mut files_to_load: HashMap<i64, String> = HashMap::new();
    let mut mount_files: HashMap<i64, Vec<String>> = HashMap::new();
    let mut model_files: HashMap<i64, Vec<String>> = HashMap::new();

    for display_id in display_csv.ids() {
        let display_info_id: i64 =
            display_csv.fetch_int_field(&display_id, "CreatureDisplayInfoID");
        let mount_id: i64 = display_csv.fetch_int_field(&display_id, "MountID");

        let model_id = creature_csv.fetch_int_field(&display_info_id, "ModelID");
        let model_file_id = creature_model_csv.fetch_int_field(&model_id, "FileDataID");
        if model_file_id > 0 {
            let file_path = list_file.get(&model_file_id).unwrap();
            if !Path::new(&("extract/".to_string() + build_version + "/" + file_path)).exists() {
                files_to_load.insert(model_file_id, file_path.clone());
            }
            model_files
                .entry(mount_id)
                .or_insert(Vec::new())
                .push(file_path.clone());
        }

        for i in 0..4 {
            let mut file_data_index = String::from("TextureVariationFileDataID[");
            file_data_index.push_str(i.to_string().as_str());
            file_data_index.push_str("]");
            let file_id: i64 =
                creature_csv.fetch_int_field(&display_info_id, file_data_index.as_str());
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

    docker.fetch_files(files_to_load);

    let mut files_to_load: HashMap<i64, String> = HashMap::new();

    for (mount_id, model_files) in model_files {
        for file_path in model_files {
            let m2_reader = M2Reader::new(&build_version, &file_path);
            for texture_file_id in m2_reader.read_texture_ids() {
                let file_id = texture_file_id as i64;
                if file_id > 0 && list_file.contains_key(&file_id) {
                    let file_path = list_file.get(&file_id).unwrap().clone();
                    if !Path::new(&("extract/".to_string() + build_version + "/" + &file_path))
                        .exists()
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
    }
    docker.fetch_files(files_to_load);

    mount_files
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
    docker: DockerRunner,
    mounts: &BTreeMap<i64, Mount>,
    list_file: &HashMap<i64, String>,
) -> HashMap<i64, Vec<Srgb<u8>>> {
    let mount_files = collect_files(build_version, docker, list_file);

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
