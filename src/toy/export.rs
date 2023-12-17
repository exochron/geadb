use std::collections::BTreeMap;

use crate::tools::lua_export::LuaFile;
use crate::toy::effect::Effect;
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
        let mut lua = self.open_file("effects.db.lua", "db.effect");

        for toy in toys.values() {
            if !toy.effects.is_empty() {
                let mut values = String::from("{");

                for effect in toy.effects.iter() {
                    values.push_str(&match effect {
                        Effect::PlayerScale(scale) => format!("[\"scale\"] = {},", scale),
                        Effect::FullVisual(visual_kit, _) => {
                            format!("[\"full_appearance\"] = {},", visual_kit)
                        }
                        Effect::MinorVisual(visual_kit, _) => {
                            format!("[\"minor_appearance\"] = {},", visual_kit)
                        }
                    });
                }
                values.push_str(" }");

                lua.add_line_with_value(&toy.item_id, &toy.name, values);
            }
        }

        lua.close();
    }
}
