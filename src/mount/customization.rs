use std::collections::{BTreeMap, HashMap};

use crate::mount::Mount;
use crate::tools::db_reader::DBReader;

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
    mounts: &BTreeMap<i64, Mount>,
    build_version: &String,
) -> HashMap<i64, HashMap<CustomizationSource, Vec<i64>>> {
    let mut result = HashMap::new();

    let mut itemxeffect_csv = DBReader::new_with_id(build_version, "ItemXItemEffect.csv", "ItemID");
    let mut itemeffect_csv = DBReader::new(build_version, "ItemEffect.csv").unwrap();
    let mut itemsparse_csv = DBReader::new(build_version, "ItemSparse.csv").unwrap();
    let mut spelleffect_csv =
        DBReader::new_with_id(build_version, "SpellEffect.csv", "SpellID").unwrap();

    let drakewatcher_quests = {
        // collect Drakewatcher Manuscripts
        let mut drakewatcher_quests: HashMap<String, Vec<i64>> = HashMap::new();
        for item_id in itemsparse_csv.ids() {
            let item_name_description_id =
                itemsparse_csv.fetch_int_field(&item_id, "ItemNameDescriptionID");
            if item_name_description_id == 13926 {
                let item_effect_id = itemxeffect_csv
                    .as_mut()
                    .unwrap()
                    .fetch_int_field(&item_id, "ItemEffectID");
                let spell_id: i64 = itemeffect_csv.fetch_int_field(&item_effect_id, "SpellID");
                match spelleffect_csv.fetch_field(&spell_id, "Effect") {
                    None => {}
                    Some(spell_effect) => {
                        if spell_effect == "16" {
                            // is Effect = QUEST_COMPLETE ?
                            let quest_id = spelleffect_csv
                                .fetch_field(&spell_id, "EffectMiscValue[0]")
                                .unwrap_or("0".to_string())
                                .parse()
                                .unwrap();
                            let item_name = itemsparse_csv
                                .fetch_field(&item_id, "Display_lang")
                                .unwrap();
                            let mount_name = item_name
                                .split(':')
                                .collect::<Vec<&str>>()
                                .first()
                                .unwrap()
                                .to_string();
                            drakewatcher_quests
                                .entry(mount_name)
                                .or_insert(Vec::new())
                                .push(quest_id);
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
