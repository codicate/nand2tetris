use std::fs::File;
use std::io::Write;
use std::path::Path;
use strum_macros::{Display, EnumString};

pub struct Writer {
    class_name: String,
    file: File,
    buffer: Vec<String>,
}

#[derive(EnumString, Display, Debug, Clone, Copy)]
#[strum(serialize_all = "lowercase")]
pub enum Segment {
    Local,
    Argument,
    This,
    That,
    Static,
    Constant,
    Pointer,
    Temp,
}

impl Writer {
    pub fn new(path: &Path) -> Self {
        let class_name = path.file_stem().unwrap().to_str().unwrap().to_owned();
        let file = File::create(path.with_extension("vm")).unwrap();
        Writer {
            class_name,
            file,
            buffer: Vec::new(),
        }
    }

    pub fn write_push(&mut self, segment: Segment, index: usize) {
        self.buffer.push(format!("push {} {}", segment, index));
    }

    pub fn write_pop(&mut self, segment: Segment, index: usize) {
        self.buffer.push(format!("pop {} {}", segment, index));
    }

    pub fn write_arithmetic(&mut self, command: &str) {
        self.buffer.push(command.to_string());
    }

    pub fn write_label(&mut self, label: &str) {
        self.buffer.push(format!("label {}", label));
    }

    pub fn write_goto(&mut self, label: &str) {
        self.buffer.push(format!("goto {}", label));
    }

    pub fn write_if(&mut self, label: &str) {
        self.buffer.push(format!("if-goto {}", label));
    }

    pub fn write_call(&mut self, name: &str, n_args: usize) {
        self.buffer.push(format!("call {} {}", name, n_args));
    }

    pub fn write_return(&mut self) {
        self.buffer.push("return".to_string());
    }

    pub fn write_function(&mut self, func_name: &str, n_vars: usize) {
        writeln!(
            self.file,
            "function {}.{} {}",
            self.class_name, func_name, n_vars
        )
        .unwrap();
        for line in &self.buffer {
            writeln!(self.file, "{}", line).unwrap();
        }
    }

    pub fn close(mut self) {
        self.file.flush().unwrap();
    }
}
