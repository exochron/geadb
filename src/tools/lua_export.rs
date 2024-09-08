use std::fs::File;
use std::io::Write;

pub struct LuaFile {
    file: File,
}

impl LuaFile {
    pub fn new(file_path: String, variable: &str) -> Self {
        let mut file = File::create(file_path).unwrap();

        writeln!(file, "local _, ADDON = ...").expect("couldn't write to file");

        let mut s = Self { file };
        s.start(variable);
        s
    }

    pub fn start(&mut self, variable: &str) {
        writeln!(self.file).expect("couldn't write to file");
        writeln!(self.file, "ADDON.{} = {{", variable).expect("couldn't write to file");
    }

    pub fn add_line(&mut self, id: &u32, name: &String) {
        writeln!(self.file, "[{}] = true, -- {}", id, name).expect("couldn't write to file")
    }
    pub fn add_line_with_value(&mut self, id: &u32, name: &String, value: String) {
        writeln!(self.file, "[{}] = {}, -- {}", id, value, name).expect("couldn't write to file")
    }

    pub fn start_category(&mut self, name: &String) {
        writeln!(self.file, "[\"{}\"] = {{", name).expect("couldn't write to file")
    }
    pub fn close_category(&mut self) {
        writeln!(self.file, "}},").expect("couldn't write to file")
    }

    pub fn close(&mut self) {
        write!(self.file, "}}").expect("couldn't write to file")
    }

    pub fn format_sublist(&self, key: &str, items: Vec<&str>) -> String {
        let mut row = String::from(" [\"");
        row.push_str(key);
        row.push_str("\"]={");
        row.push_str(items.join(",").as_str());
        row.push_str(",},");

        row
    }
}
