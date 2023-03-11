use std::collections::{BTreeMap, HashMap};

use crate::mount::Mount;
use crate::tools::db_reader::DBReader;

const ITEMSPARSE_ITEM_ID: usize = 0;
const ITEMSPARSE_ITEM_NAME_DESCRIPTION_ID: usize = 53;

const ITEMXITEMEFFECT_ITEM_EFFECT_ID: usize = 1;
const ITEMXITEMEFFECT_ITEM_ID: usize = 2;

const ITEMEFFECT_ITEM_ID: usize = 7;

const SPELLEFFECT_EFFECT: usize = 4;
const SPELLEFFECT_EFFECT_PAYLOAD: usize = 25;
const SPELLEFFECT_SPELL_ID: usize = 35;

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

    let mut itemxeffect_csv =
        DBReader::new(build_version, "ItemXItemEffect.csv").id_column(ITEMXITEMEFFECT_ITEM_ID);
    let mut itemeffect_csv = DBReader::new(build_version, "ItemEffect.csv");
    let mut itemsparse_csv = DBReader::new(build_version, "ItemSparse.csv");
    let mut spelleffect_csv =
        DBReader::new(build_version, "SpellEffect.csv").id_column(SPELLEFFECT_SPELL_ID);

    let drakewatcher_quests = {
        // collect Drakewatcher Manuscripts
        let mut drakewatcher_quests: HashMap<String, Vec<i64>> = HashMap::new();
        for itemsparse in itemsparse_csv.reader.records() {
            let itemsparse_record = itemsparse.unwrap();
            let item_id: i64 = itemsparse_record
                .get(ITEMSPARSE_ITEM_ID)
                .unwrap()
                .parse()
                .unwrap();
            let item_name_description_id = itemsparse_record
                .get(ITEMSPARSE_ITEM_NAME_DESCRIPTION_ID)
                .unwrap_or_default();
            if item_name_description_id == "13926" {
                let item_effect_id =
                    itemxeffect_csv.fetch_int_field(&item_id, ITEMXITEMEFFECT_ITEM_EFFECT_ID);
                let spell_id: i64 =
                    itemeffect_csv.fetch_int_field(&item_effect_id, ITEMEFFECT_ITEM_ID);
                match spelleffect_csv.fetch_record(&spell_id) {
                    None => {}
                    Some(spell_effect) => {
                        if spell_effect.get(SPELLEFFECT_EFFECT).unwrap_or_default() == "16" {
                            // is Effect = QUEST_COMPLETE ?
                            let quest_id: i64 = spell_effect
                                .get(SPELLEFFECT_EFFECT_PAYLOAD)
                                .unwrap_or("0")
                                .parse()
                                .unwrap();
                            let item_name = itemsparse_record.get(6).unwrap();
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
