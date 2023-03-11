use std::fs;
use std::io::Cursor;

use byteorder::{LittleEndian, ReadBytesExt};

pub(crate) struct M2Reader {
    data: Vec<u8>,
}

impl M2Reader {
    pub(crate) fn new(build_version: &String, file_name: &str) -> Self {
        let file_path = r"extract/".to_owned() + build_version + r"/" + file_name;
        let data = fs::read(file_path).unwrap();

        Self { data }
    }

    pub fn read_texture_ids(&self) -> Vec<i32> {
        let mut result = Vec::new();

        match self.data.windows(4).position(|w| w == b"TXID") {
            None => {}
            Some(position) => {
                let mut cursor = Cursor::new(&self.data);
                cursor.set_position((position + 4) as u64);
                let mut len = cursor.read_i32::<LittleEndian>().unwrap();
                while len > 0 {
                    result.push(cursor.read_i32::<LittleEndian>().unwrap());
                    len -= 4;
                }
            }
        };

        result
    }
}
