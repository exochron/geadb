use std::collections::{BTreeMap, HashMap};

use crate::mount::Mount;
use crate::tools::db_reader::{load_item_effects, parse_csv, LookupDB};
use crate::tools::dbs;
use crate::tools::dbs::SpellEffect;

#[derive(Eq, PartialEq, Hash)]
pub enum CustomizationSource {
    Quest,
    Achievement,
}

impl CustomizationSource {
    pub fn to_str(&self) -> &str {
        match self {
            CustomizationSource::Quest => "quest",
            CustomizationSource::Achievement => "achievement",
        }
    }
}

pub fn collect_customization(
    mounts: &BTreeMap<u32, Mount>,
    build_version: &String,
) -> HashMap<u32, HashMap<CustomizationSource, Vec<u32>>> {
    let mut result = HashMap::new();

    let item_effects_db = load_item_effects(build_version, false);
    let item_sparse_db: Vec<dbs::ItemSparse> = parse_csv(build_version, "ItemSparse.csv").unwrap();
    let spell_effect_db: LookupDB<SpellEffect> = LookupDB::new_from_data(
        parse_csv(build_version, "SpellEffect.csv").unwrap(),
        |s: &SpellEffect| s.spell_id,
    );

    let drakewatcher_quests = {
        // collect Drakewatcher Manuscripts
        let mut drakewatcher_quests: HashMap<String, Vec<u32>> = HashMap::new();
        for item_sparse in item_sparse_db {
            if item_sparse.description_id == 13926 {
                for item_effect in item_effects_db.lookup(&item_sparse.item_id) {
                    for spell_effect in spell_effect_db.lookup(&(item_effect.spell_id as u32)) {
                        if spell_effect.effect == 16 {
                            // is Effect = QUEST_COMPLETE ?
                            let quest_id = spell_effect.effect_misc_value;
                            let item_name = &item_sparse.display_text;
                            let mount_name = item_name
                                .split(':')
                                .collect::<Vec<&str>>()
                                .first()
                                .unwrap()
                                .to_string();
                            drakewatcher_quests
                                .entry(mount_name)
                                .or_default()
                                .push(quest_id as u32);
                        }
                    }
                }
            }
        }
        drakewatcher_quests
    };

    for mount in mounts.values() {
        // is dragonriding mount?
        if mount.type_id == 402 && drakewatcher_quests.contains_key(&mount.name) {
            let mut quests = drakewatcher_quests.get(&mount.name).unwrap().to_owned();
            quests.sort();
            quests.dedup();
            result
                .entry(mount.id)
                .or_insert(HashMap::new())
                .insert(CustomizationSource::Quest, quests);
        }

        if mount.id == 1239 {
            // X-995 Mechanocat
            let mut data = HashMap::new();
            data.insert(CustomizationSource::Achievement, vec![13513]);
            data.insert(
                CustomizationSource::Quest,
                vec![55451, 55452, 55454, 55455, 55456, 55457, 55517],
            );
            result.insert(mount.id, data);
        }
    }

    result
}
