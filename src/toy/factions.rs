use crate::tools::db_reader::parse_csv;
use crate::tools::dbs;

#[derive(PartialEq)]
pub enum Faction {
    Alliance = 0,
    Horde = 1,
}

pub struct FactionParser {
    alliance_mask: i64,
    horde_mask: i64,
}

impl FactionParser {
    pub fn new(build_version: &String) -> Self {
        let mut alliance_mask: i64 = 0;
        let mut horde_mask: i64 = 0;

        let race_db: Vec<dbs::ChrRace> = parse_csv(build_version, "ChrRaces.csv").unwrap();
        for race in race_db {
            if race.race_bit >= 0 {
                let bit_mask = i64::pow(2, race.race_bit as u32);
                match race.alliance {
                    0 => alliance_mask |= bit_mask,
                    1 => horde_mask |= bit_mask,
                    _ => {}
                }
            }
        }

        Self {
            alliance_mask,
            horde_mask,
        }
    }

    pub fn determine_faction(&self, race_mask: i64, flags: [i64; 5]) -> Option<Faction> {
        if flags[1] & 1 == 1 {
            return Some(Faction::Horde);
        }
        if flags[1] & 2 == 2 {
            return Some(Faction::Alliance);
        }
        if race_mask != -1 && race_mask != 0 {
            if race_mask & self.alliance_mask != 0 {
                return Some(Faction::Alliance);
            }
            if race_mask & self.horde_mask != 0 {
                return Some(Faction::Horde);
            }
        }

        None
    }
}
