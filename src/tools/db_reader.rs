use std::collections::HashMap;
use std::fs::File;

use csv::{Position, Reader, ReaderBuilder, StringRecord};

pub struct DBReader {
    reader: Reader<File>,
    colum_map: HashMap<String, usize>,
    id_map: HashMap<i64, Position>,
}

impl DBReader {
    pub fn new(build_version: &String, file_name: &str) -> Option<Self> {
        DBReader::new_with_id(build_version, file_name, "ID")
    }
    pub fn new_with_id(build_version: &String, file_name: &str, id_column: &str) -> Option<Self> {
        let file_path = r"extract/".to_owned() + build_version + r"/DBFilesClient/" + file_name;
        let mut reader;
        match ReaderBuilder::new()
            .delimiter(b',')
            .quote(b'"')
            .from_path(&file_path)
        {
            Ok(r) => reader = r,
            Err(_) => return None,
        }

        let mut colum_map = HashMap::new();
        let headers = reader.headers().unwrap();
        for (index, header) in headers.iter().enumerate() {
            colum_map.insert(header.to_string(), index);
        }

        let mut id_map = HashMap::new();
        for result in reader.records() {
            let record = result.unwrap();
            let id: i64 = record
                .get(*colum_map.get(id_column).unwrap())
                .unwrap()
                .parse()
                .expect("couldn't convert id into int");
            id_map.insert(id, record.position().unwrap().clone());
        }

        Some(Self {
            reader,
            colum_map,
            id_map,
        })
    }

    pub fn ids(&self) -> Vec<i64> {
        self.id_map.keys().cloned().collect()
    }

    fn fetch_record(&mut self, id: &i64) -> Option<StringRecord> {
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

    pub fn fetch_field(&mut self, id: &i64, field: &str) -> Option<String> {
        let index = *self.colum_map.get(field)?;

        self.fetch_record(id)
            .map(|record| record.get(index).unwrap().to_string())
    }
    pub fn fetch_int_field(&mut self, id: &i64, field: &str) -> i64 {
        self.fetch_field(id, field)
            .unwrap_or("0".to_string())
            .parse()
            .unwrap()
    }
}
