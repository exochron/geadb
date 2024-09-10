use std::collections::{BTreeMap, HashMap};

use palette::Srgb;

use crate::mount::customization::CustomizationSource;
use crate::mount::family::FamilyNode;
use crate::mount::Mount;
use crate::mount::sources::DropData;
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

    pub fn export_tradable(&self, mounts: &BTreeMap<u32, Mount>) {
        let mut lua = self.open_file("tradable.db.lua", "DB.Tradable");

        for (mount_id, mount) in mounts.iter() {
            if mount.item_is_tradeable {
                lua.add_line(mount_id, &mount.name);
            }
        }
        lua.close();
    }
    pub fn export_conditions(&self, mounts: &BTreeMap<u32, Mount>) {
        let mut lua = self.open_file("restrictions.db.lua", "DB.Restrictions");

        for (mount_id, mount) in mounts.iter() {
            if !mount.player_conditions.is_empty() {
                let mut values = String::from("{");

                for condition_group in mount.player_conditions.iter() {
                    let group = condition_group.first().unwrap().group.as_str();
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

    pub fn export_families(
        &self,
        mounts: &BTreeMap<u32, Mount>,
        family_map: BTreeMap<String, FamilyNode>,
    ) {
        let mut lua = self.open_file("families.db.lua", "DB.Family");

        for (name, node) in family_map.iter() {
            lua.start_category(name);
            let mut mount_ids = node.mount_ids.clone();
            mount_ids.sort();
            for mount_id in mount_ids.iter() {
                lua.add_line(mount_id, &mounts.get(mount_id).unwrap().name);
            }

            for (name, sub_node) in node.sub_nodes.iter() {
                lua.start_category(name);

                let mut mount_ids = sub_node.mount_ids.clone();
                mount_ids.sort();
                for mount_id in mount_ids.iter() {
                    lua.add_line(mount_id, &mounts.get(mount_id).unwrap().name);
                }

                lua.close_category();
            }

            lua.close_category();
        }

        lua.close();
    }

    pub fn export_sources(
        &self,
        mounts: &BTreeMap<u32, Mount>,
        black_market_mounts: Vec<u32>,
        retired_mounts: Vec<u32>,
        drop_mounts: BTreeMap<u32, DropData>
    ) {
        let mut lua = self.open_file("sources.db.lua", "DB.Source[\"Black Market\"]");
        let mut ordered_list = BTreeMap::new();

        for mount_id in black_market_mounts.iter() {
            let mount = &mounts.get(mount_id).unwrap();
            ordered_list.insert(&mount.spell_id, &mount.name);
        }
        for (spell_id, name) in ordered_list.iter() {
            lua.add_line(spell_id, name);
        }

        lua.close();

        let mut ordered_list = BTreeMap::new();

        lua.start("DB.Source[\"Unavailable\"]");
        for mount_id in retired_mounts.iter() {
            let mount = &mounts.get(mount_id).unwrap();
            ordered_list.insert(&mount.spell_id, &mount.name);
        }
        for (spell_id, name) in ordered_list.iter() {
            lua.add_line(spell_id, name);
        }
        lua.close();

        let mut ordered_list = BTreeMap::new();

        lua.write_line("");
        lua.write_line("MergeTable(ADDON.DB.Source.Drop, {");
        for (mount_id, drop_data) in drop_mounts.iter() {
            let mount = &mounts.get(mount_id).unwrap();
            ordered_list.insert(&mount.spell_id, (&mount.name, drop_data));
        }
        for (spell_id, (name, drop_data)) in ordered_list.iter() {
            let drop_chance = drop_data.drop_chance.map(|d| format!("{:.2}", d) ).unwrap_or("nil".to_string());
            let position = match &drop_data.map_position {
                Some(position) => format!(
                    "{{{}, {}, {}, {}}}",
                    position.map_id,
                    position.map_x,
                    position.map_y,
                    drop_chance,
                ),
                None => format!("{{nil, nil, nil, {}}}", drop_chance, ),
            };
            lua.add_line_with_value(spell_id, &name, position);
        }
        lua.write_line("})");
    }

    pub fn export_colors(
        &self,
        mounts: &BTreeMap<u32, Mount>,
        dominant_colors: HashMap<u32, Vec<Srgb<u8>>>,
    ) {
        let mut lua = self.open_file("colors.db.lua", "DB.Colors");

        for mount in mounts.values() {
            match dominant_colors.get(&mount.id) {
                None => {}
                Some(colors) => {
                    let colors = colors
                        .iter()
                        .map(|pxl| format!("{{{},{},{}}}", pxl.red, pxl.green, pxl.blue))
                        .collect::<Vec<String>>();
                    lua.add_line_with_value(
                        &mount.id,
                        &mount.name,
                        "{ ".to_string() + colors.join(", ").as_str() + ", }",
                    );
                }
            }
        }

        lua.close();
    }

    pub(crate) fn export_customization(
        &self,
        mounts: &BTreeMap<u32, Mount>,
        collected_customization: HashMap<u32, BTreeMap<CustomizationSource, Vec<u32>>>,
    ) {
        let mut lua = self.open_file("customization.db.lua", "DB.Customization");

        for mount in mounts.values() {
            match collected_customization.get(&mount.id) {
                None => {}
                Some(customs) => {
                    let mut values = String::from("{\n");

                    for (custom_group, ids) in customs.iter() {
                        let vals: Vec<String> = ids.iter().map(|c| c.to_string()).collect();
                        values.push_str(
                            lua.format_sublist(
                                custom_group.to_str(),
                                vals.iter().map(|s| s.as_str()).collect(),
                            )
                            .as_str(),
                        );
                        values.push('\n');
                    }
                    values.push_str(" }");

                    lua.add_line_with_value(&mount.id, &mount.name, values);
                }
            }
        }

        lua.close();
    }
}
