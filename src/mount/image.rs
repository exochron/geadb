use std::collections::{BTreeMap, HashMap};
use std::path::Path;

use kmeans_colors::{get_kmeans, Kmeans};
use palette::{FromColor, Lab, Srgb};

use crate::mount::Mount;
use crate::tools::{BuildInfo, fetch_files};
use crate::tools::blp_reader::BLPReader;
use crate::tools::db_reader::{LookupDB, parse_csv};
use crate::tools::dbs;
use crate::tools::m2_reader::M2Reader;

fn collect_files(
    build_version: &BuildInfo,
    list_file: &HashMap<i64, String>,
) -> HashMap<u32, Vec<String>> {
    let display_db: Vec<dbs::MountXDisplay> =
        parse_csv(&build_version.version, "MountXDisplay.csv").unwrap();
    let creature_display_db: LookupDB<dbs::CreatureDisplayInfo> = LookupDB::new_from_data(
        parse_csv(&build_version.version, "CreatureDisplayInfo.csv").unwrap(),
        |s: &dbs::CreatureDisplayInfo| s.id,
    );
    let creature_model_db: LookupDB<dbs::CreatureModel> = LookupDB::new_from_data(
        parse_csv(&build_version.version, "CreatureModelData.csv").unwrap(),
        |s: &dbs::CreatureModel| s.id,
    );

    let mut files_to_load: HashMap<i64, String> = HashMap::new();
    let mut mount_files: HashMap<u32, Vec<String>> = HashMap::new();
    let mut model_files: HashMap<u32, Vec<String>> = HashMap::new();

    for mount_display in display_db {
        let mount_id = mount_display.mount_id;
        for creature in creature_display_db.lookup(&mount_display.creature_info_id) {
            let model_file_id = creature_model_db
                .lookup(&creature.model_id)
                .first()
                .unwrap()
                .file_id;

            let file_ids = [
                model_file_id,
                creature.texture_variant_file_id_0,
                creature.texture_variant_file_id_1,
                creature.texture_variant_file_id_2,
                creature.texture_variant_file_id_3,
            ];
            for file_id in file_ids {
                if file_id > 0 && list_file.contains_key(&file_id) {
                    let file_path = list_file.get(&file_id).unwrap();
                    if !Path::new(&format!("extract/{}/{}", build_version.version, file_path))
                        .exists()
                    {
                        files_to_load.insert(file_id, file_path.clone());
                    }
                    if file_path.ends_with(".m2") {
                        model_files
                            .entry(mount_id)
                            .or_default()
                            .push(file_path.clone());
                    } else {
                        mount_files
                            .entry(mount_id)
                            .or_default()
                            .push(file_path.clone());
                    }
                }
            }
        }
    }

    fetch_files(build_version, files_to_load);

    let mut files_to_load: HashMap<i64, String> = HashMap::new();

    for (mount_id, model_files) in model_files {
        for file_path in model_files {
            if Path::new(&format!("extract/{}/{}", build_version.version, file_path))
                .exists()
            {
                let m2_reader = M2Reader::new(&build_version.version, &file_path);
                for texture_file_id in m2_reader.read_texture_ids() {
                    let file_id = texture_file_id as i64;
                    if file_id > 0 && list_file.contains_key(&file_id) {
                        let file_path = list_file.get(&file_id).unwrap().clone();
                        if !Path::new(&format!("extract/{}/{}", build_version.version, file_path))
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
    }
    fetch_files(build_version, files_to_load);

    mount_files
}

fn determine_dominant_colors(pixels: Vec<Lab>, seed: &u32) -> Vec<Srgb<u8>> {
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
    build_version: &BuildInfo,
    mounts: &BTreeMap<u32, Mount>,
    list_file: &HashMap<i64, String>,
) -> HashMap<u32, Vec<Srgb<u8>>> {
    let mount_files = collect_files(build_version, list_file);

    let mut result = HashMap::new();

    for mount in mounts.values() {
        match mount_files.get(&mount.id) {
            None => {}
            Some(file_paths) => {
                let mut pixels: Vec<Lab> = Vec::new();

                for file_path in file_paths {
                    if Path::new(&format!("extract/{}/{}", build_version.version, file_path))
                        .exists()
                    {
                        let mut file_pixels =
                            BLPReader::new(&build_version.version, file_path).convert_to_lab();
                        pixels.append(&mut file_pixels);
                    }
                }

                if !pixels.is_empty() {
                    result.insert(mount.id, determine_dominant_colors(pixels, &mount.id));
                }
            }
        };
    }

    result
}
