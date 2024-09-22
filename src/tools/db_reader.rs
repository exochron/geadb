use std::collections::HashMap;
use std::fs::File;

use csv::{DeserializeRecordsIter, ReaderBuilder};
use serde::de::DeserializeOwned;

use crate::tools::dbs;
use crate::tools::dbs::ItemEffect;

pub struct LookupDB<T: Clone> {
    map: HashMap<u32, Vec<T>>,
}

impl<T: Clone> LookupDB<T> {
    pub fn new_from_data(data: Vec<T>, id_val: fn(&T) -> u32) -> Self {
        let mut map: HashMap<u32, Vec<T>> = HashMap::new();
        for datum in data {
            let id = id_val(&datum);
            map.entry(id).or_default().push(datum);
        }

        Self { map }
    }
    pub fn new(map: HashMap<u32, Vec<T>>) -> Self {
        Self { map }
    }

    pub fn lookup(&self, id: &u32) -> Vec<T> {
        match self.map.get(id) {
            None => Vec::default(),
            Some(v) => v.to_vec(),
        }
    }
}

pub fn parse_csv<T: DeserializeOwned>(build_version: &String, file_name: &str) -> Option<Vec<T>> {
    let file_path = r"extract/".to_owned() + build_version + r"/DBFilesClient/" + file_name;
    let mut reader;
    match ReaderBuilder::new()
        .delimiter(b',')
        .quote(b'"')
        .from_path(file_path)
    {
        Ok(r) => reader = r,
        Err(_) => return None,
    }

    let iter: DeserializeRecordsIter<File, T> = reader.deserialize();
    Some(iter.map(|r| r.unwrap()).collect())
}

pub fn load_item_effects(
    build_version: &String,
    use_spell_id_as_index: bool,
) -> LookupDB<ItemEffect> {
    let item_x_effect_db: Option<Vec<dbs::ItemXItemEffect>> =
        parse_csv(build_version, "ItemXItemEffect.csv");
    match item_x_effect_db {
        None => LookupDB::new_from_data(
            parse_csv(build_version, "ItemEffect.csv").unwrap(),
            if use_spell_id_as_index {
                |s: &ItemEffect| s.spell_id as u32
            } else {
                |s: &ItemEffect| s.item_id
            },
        ),
        Some(item_x_effect_db) => {
            let mut result: HashMap<u32, Vec<dbs::ItemEffect>> = HashMap::new();

            let item_effect_db: LookupDB<dbs::ItemEffect> = LookupDB::new_from_data(
                parse_csv(build_version, "ItemEffect.csv").unwrap(),
                |s: &ItemEffect| s.id,
            );

            for x_effect in item_x_effect_db {
                let effects = item_effect_db.lookup(&x_effect.item_effect_id);
                if !effects.is_empty() {
                    let mut effect = effects.first().unwrap().clone();
                    effect.item_id = x_effect.item_id;
                    result
                        .entry(if use_spell_id_as_index {
                            effect.spell_id as u32
                        } else {
                            effect.item_id
                        })
                        .or_default()
                        .push(effect);
                }
            }

            LookupDB::new(result)
        }
    }
}
