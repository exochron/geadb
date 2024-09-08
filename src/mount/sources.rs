use std::collections::BTreeMap;
use crate::mount::Mount;
use crate::mount::wcm::{load_wcm_black_market_mounts, load_wcm_retired_mounts};

fn filter_mounts_by_names(
    mounts: &BTreeMap<u32, Mount>,
    names: Vec<String>,
) -> Vec<u32> {
    let mut result: Vec<u32> = Vec::new();

    for (_, mount) in mounts.iter() {
        let lowered_name = mount.name.to_lowercase();
        names.contains(&lowered_name).then(|| result.push(mount.id));
    }

    result
}

pub fn collect_black_market_mounts(
    mounts: &BTreeMap<u32, Mount>,
) -> Vec<u32> {
    filter_mounts_by_names(mounts, load_wcm_black_market_mounts())
}
pub fn collect_unavailable_mounts(
    mounts: &BTreeMap<u32, Mount>,
) -> Vec<u32> {
    filter_mounts_by_names(mounts, load_wcm_retired_mounts())
}