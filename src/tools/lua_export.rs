use std::fs::File;
use std::io::Write;

pub struct LuaFile {
    file: File,
}

impl LuaFile {
    pub fn new(file_path: String, variable: &str) -> Self {
        let mut file = File::create(file_path).unwrap();

        let mut s = Self { file };
        s.write_line("local _, ADDON = ...");
        s.start(variable);
        s
    }

    pub fn write_line(&mut self, line: &str) {
        writeln!(self.file, "{}", line).expect("couldn't write to file");
    }

    pub fn start(&mut self, variable: &str) {
        self.write_line("");
        self.write_line(&*("ADDON.".to_owned() + variable + " = {"));
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
        self.write_line("},")
    }

    pub fn close(&mut self) {
        self.write_line("}")
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
