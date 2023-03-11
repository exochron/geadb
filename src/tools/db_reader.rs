use std::collections::HashMap;
use std::fs::File;

use csv::{Position, Reader, ReaderBuilder, StringRecord};

pub struct DBReader {
    pub reader: Reader<File>,
    id_map: HashMap<i64, Position>,
    id_column: usize,
}

impl DBReader {
    pub(crate) fn new(build_version: &String, file_name: &str) -> Self {
        let file_path = r"extract/".to_owned() + build_version + r"/DBFilesClient/" + file_name;
        let reader = ReaderBuilder::new()
            .delimiter(b',')
            .quote(b'"')
            .from_path(&file_path)
            .expect(&("could not open file: ".to_owned() + &file_path));

        Self {
            reader,
            id_map: HashMap::new(),
            id_column: 0,
        }
    }

    pub(crate) fn id_column(mut self, id_column: usize) -> DBReader {
        self.id_column = id_column;
        self
    }

    pub fn fetch_record(&mut self, id: &i64) -> Option<StringRecord> {
        if self.id_map.is_empty() {
            for result in self.reader.records() {
                let record = result.unwrap();
                let id = record
                    .get(self.id_column)
                    .unwrap()
                    .parse()
                    .expect("couldn't convert id into int");
                self.id_map.insert(id, record.position().unwrap().clone());
            }
        }

        if self.id_map.contains_key(id) {
            let line = self.id_map.get(id).unwrap();
            self.reader
                .seek(line.to_owned())
                .expect("failed to jump to line");
            let mut record = StringRecord::new();
            self.reader
                .read_record(&mut record)
                .expect("failed to read record");

            Some(record)
        } else {
            None
        }
    }

    pub fn fetch_field(&mut self, id: &i64, field: usize) -> Option<String> {
        self.fetch_record(id)
            .map(|record| record.get(field).unwrap().to_string())
    }
    pub fn fetch_int_field(&mut self, id: &i64, field: usize) -> i64 {
        self.fetch_field(id, field)
            .unwrap_or("0".to_string())
            .parse()
            .unwrap()
    }
}
