use std::fs::File;
use std::io::{Result, Write};
use std::path::Path;

use crate::convert_file_extension;

pub struct Writer {
    file: File,
}

impl Writer {
    pub fn new(path: &Path) -> Self {
        let file = File::create(convert_file_extension(path, "vm")).unwrap();
        Writer { file }
    }

    pub fn write_push(&mut self, segment: &str, index: i32) {
        writeln!(self.file, "push {} {}", segment.to_lowercase(), index).unwrap();
    }

    pub fn write_pop(&mut self, segment: &str, index: i32) {
        writeln!(self.file, "pop {} {}", segment.to_lowercase(), index).unwrap();
    }

    pub fn write_arithmetic(&mut self, command: &str) {
        writeln!(self.file, "{}", command.to_lowercase()).unwrap();
    }

    pub fn write_label(&mut self, label: &str) {
        writeln!(self.file, "label {}", label).unwrap();
    }

    pub fn write_goto(&mut self, label: &str) {
        writeln!(self.file, "goto {}", label).unwrap();
    }

    pub fn write_if(&mut self, label: &str) {
        writeln!(self.file, "if-goto {}", label).unwrap();
    }

    pub fn write_call(&mut self, name: &str, n_args: i32) {
        writeln!(self.file, "call {} {}", name, n_args).unwrap();
    }

    pub fn write_function(&mut self, name: &str, n_vars: i32) {
        writeln!(self.file, "function {} {}", name, n_vars).unwrap();
    }

    pub fn write_return(&mut self) {
        writeln!(self.file, "return").unwrap();
    }

    pub fn close(mut self) {
        self.file.flush().unwrap();
    }
}
