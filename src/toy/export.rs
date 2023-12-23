use std::collections::{BTreeMap, HashMap};

use crate::tools::lua_export::LuaFile;
use crate::toy::Toy;

pub struct Exporter {
    base_path: String,
}

impl Exporter {
    pub fn new(file_path: &str) -> Self {
        Self {
            base_path: file_path.to_string(),
        }
    }

    fn open_file(&self, file_name: &str, variable: &str) -> LuaFile {
        let mut file_path = self.base_path.to_owned();
        file_path.push('/');
        file_path.push_str(file_name);
        LuaFile::new(file_path, variable)
    }

    pub fn export_tradable(&self, toys: &BTreeMap<i64, Toy>) {
        let mut lua = self.open_file("tradable.db.lua", "db.Tradable");

        for toy in toys.values() {
            if toy.item_is_tradable {
                lua.add_line(&toy.item_id, &toy.name);
            }
        }
        lua.close();
    }

    pub fn export_toys(&self, toys: &BTreeMap<i64, Toy>) {
        let mut lua = self.open_file("toys.db.lua", "db.ingameList");

        for toy in toys.values() {
            lua.add_line_with_value(&toy.item_id, &toy.name, "false".to_string())
        }

        lua.close();
    }
    pub fn export_effects(&self, toys: &BTreeMap<i64, Toy>) {
        let mut lua = self.open_file("preview.db.lua", "db.preview");

        for toy in toys.values() {
            if !toy.effects.is_empty() {
                let mut values = String::from("{");

                let mut groups: HashMap<String, Vec<i64>> = HashMap::new();

                for effect in toy.effects.iter() {
                    groups
                        .entry(effect.as_str().to_string())
                        .or_default()
                        .push(effect.value());
                }

                values.push_str(
                    &groups
                        .iter()
                        .map(|(group, list)| {
                            format!(
                                "[\"{}\"] = {{{}}},",
                                group,
                                list.iter()
                                    .fold("".to_string(), |e, id| format!("{}{},", e, id))
                            )
                        })
                        .reduce(|ff, e| e + &*ff)
                        .unwrap(),
                );

                values.push_str(" }");

                lua.add_line_with_value(&toy.item_id, &toy.name, values);
            }
        }

        lua.close();
    }
}
