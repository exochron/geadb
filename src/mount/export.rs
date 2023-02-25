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

    pub fn export_tradable(&self, mounts: &BTreeMap<i64, Mount>) {
        let mut file_path = self.base_path.to_owned();
        file_path.push_str("/tradable.db.lua");
        let mut lua = LuaFile::new(file_path, "Tradable");

        for (mount_id, mount) in mounts.iter() {
            if mount.item_is_tradeable {
                lua.add_line(mount_id, &mount.name);
            }
        }
        lua.close();
    }
    pub fn export_conditions(&self, mounts: &BTreeMap<i64, Mount>) {
        let mut file_path = self.base_path.to_owned();
        file_path.push_str("/restrictions.db.lua");
        let mut lua = LuaFile::new(file_path, "Restrictions");

        for (mount_id, mount) in mounts.iter() {
            if !mount.player_conditions.is_empty() {
                let mut values = String::from("{");

                for condition_group in mount.player_conditions.iter() {
                    values.push_str(" [\"");
                    values.push_str(match condition_group.first().unwrap().group {
                        ConditionGroup::Class => "class",
                        ConditionGroup::Skill => "skill",
                        ConditionGroup::Race => "race",
                        ConditionGroup::Covenant => "covenant",
                    });
                    values.push_str("\"]={");
                    for condition in condition_group.iter() {
                        values.push_str(condition.value.as_str());
                        values.push(',');
                    }

                    values.push_str("},");
                }
                values.push_str(" }");

                lua.add_line_with_value(mount_id, &mount.name, values);
            }
        }
        lua.close();
    }
}
