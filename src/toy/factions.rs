use crate::tools::dbs;

#[derive(PartialEq)]
pub enum Faction {
    Alliance = 0,
    Horde = 1,
}

pub fn determine_faction(race_mask: i64, flags: [i64; 5], races: &Vec<dbs::ChrRace>) -> Option<Faction> {

    if flags[1] & 1 == 1 {
        return Some(Faction::Horde);
    }
    if flags[1] & 2 == 2 {
        return Some(Faction::Alliance);
    }
    if race_mask != -1 && race_mask != 0 {
        for race in races {
            if race.race_bit >=0 {
                let bit_mask = i64::pow(2, race.race_bit as u32);
                if race_mask & bit_mask > 0 {
                    match race.alliance {
                        0 => return Some(Faction::Alliance),
                        1 => return Some(Faction::Horde),
                        _ => {}
                    }
                }
            }
        }
    }

    None
}