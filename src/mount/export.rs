use std::collections::BTreeMap;

use crate::mount::condition::ConditionGroup;
use crate::mount::Mount;
use crate::tools::lua_export::LuaFile;

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

    pub fn export_tradable(&self, mounts: &BTreeMap<i64, Mount>) {
        let mut lua = self.open_file("tradable.db.lua", "Tradable");

        for (mount_id, mount) in mounts.iter() {
            if mount.item_is_tradeable {
                lua.add_line(mount_id, &mount.name);
            }
        }
        lua.close();
    }
    pub fn export_conditions(&self, mounts: &BTreeMap<i64, Mount>) {
        let mut lua = self.open_file("restrictions.db.lua", "Restrictions");

        for (mount_id, mount) in mounts.iter() {
            if !mount.player_conditions.is_empty() {
                let mut values = String::from("{");

                for condition_group in mount.player_conditions.iter() {
                    let group = match condition_group.first().unwrap().group {
                        ConditionGroup::Class => "class",
                        ConditionGroup::Skill => "skill",
                        ConditionGroup::Race => "race",
                        ConditionGroup::Covenant => "covenant",
                    };
                    let vals: Vec<&str> =
                        condition_group.iter().map(|c| c.value.as_str()).collect();
                    values.push_str(lua.format_sublist(group, vals).as_str());
                }
                values.push_str(" }");

                lua.add_line_with_value(mount_id, &mount.name, values);
            }
        }
        lua.close();
    }

    pub fn export_rarities(&self, mounts: &BTreeMap<i64, Mount>) {
        let mut lua = self.open_file("rarities.db.lua", "Rarities");

        for (mount_id, mount) in mounts.iter() {
            match mount.rarity {
                None => (),
                Some(rarity) => {
                    lua.add_line_with_value(mount_id, &mount.name, format!("{:?}", rarity));
                }
            };
        }

        lua.close();
    }
}
