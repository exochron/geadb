use std::collections::BTreeMap;

#[derive(Debug, Copy, Clone)]
pub enum ConditionGroup {
    Class,
    Skill,
    Race,
    Covenant,
}

impl ConditionGroup {
    pub fn as_str(&self) -> &'static str {
        match self {
            ConditionGroup::Class => "class",
            ConditionGroup::Skill => "skill",
            ConditionGroup::Race => "race",
            ConditionGroup::Covenant => "covenant",
        }
    }
}

pub struct Condition {
    pub group: ConditionGroup,
    pub value: String,
}

fn check_bitmask(group: ConditionGroup, bitmap: BTreeMap<i64, &str>, mask: i64) -> Vec<Condition> {
    let mut result = Vec::new();

    for (bitmask, class) in bitmap.iter() {
        if mask & bitmask > 0 {
            result.push(Condition {
                group,
                value: ("\"".to_owned() + class + "\""),
            })
        }
    }

    result
}

fn check_class(mask: i64) -> Vec<Condition> {
    let map = BTreeMap::from([
        (0x1, "WARRIOR"),
        (0x2, "PALADIN"),
        (0x4, "HUNTER"),
        (0x8, "ROGUE"),
        (0x10, "PRIEST"),
        (0x20, "DEATHKNIGHT"),
        (0x40, "SHAMAN"),
        (0x80, "MAGE"),
        (0x100, "WARLOCK"),
        (0x200, "MONK"),
        (0x400, "DRUID"),
        (0x800, "DEMONHUNTER"),
        (0x1000, "EVOKER"),
    ]);
    check_bitmask(ConditionGroup::Class, map, mask)
}

fn check_race(mask: i64) -> Vec<Condition> {
    if mask == 6130900294268439629 || mask == -6184943489809468494 {
        // skip full faction masks
        return Vec::new();
    }

    let map = BTreeMap::from([
        (0x4, "Dwarf"),
        (0x20, "Tauren"),
        (0x200, "BloodElf"),
        (0x400, "Draenei"),
        (0x800, "DarkIronDwarf"),
        (0x20000000, "LightforgedDraenei"),
        (0x40000000, "ZandalariTroll"),
    ]);
    check_bitmask(ConditionGroup::Race, map, mask)
}

pub fn parse_conditions(
    race_mask: i64,
    failure_description: &str,
    class_mask: i64,
    skill_id: i64,
) -> Vec<Vec<Condition>> {
    let mut result = Vec::new();

    let class_conditions = check_class(class_mask);
    if !class_conditions.is_empty() {
        result.push(class_conditions);
    }

    let race_conditions = check_race(race_mask);
    if !race_conditions.is_empty() {
        result.push(race_conditions);
    }

    if skill_id != 0 {
        result.push(Vec::from([Condition {
            group: ConditionGroup::Skill,
            value: skill_id.to_string(),
        }]))
    }

    if failure_description.contains("Kyrian") {
        result.push(Vec::from([Condition {
            group: ConditionGroup::Covenant,
            value: String::from("1"),
        }]))
    } else if failure_description.contains("Venthyr ") {
        result.push(Vec::from([Condition {
            group: ConditionGroup::Covenant,
            value: String::from("2"),
        }]))
    } else if failure_description.contains("Night Fae") {
        result.push(Vec::from([Condition {
            group: ConditionGroup::Covenant,
            value: String::from("3"),
        }]))
    } else if failure_description.contains("Necrolord") {
        result.push(Vec::from([Condition {
            group: ConditionGroup::Covenant,
            value: String::from("4"),
        }]))
    }

    result
}
