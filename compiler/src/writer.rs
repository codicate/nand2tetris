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

    pub fn write_push(&mut self, segment: &str, index: i32) -> Result<()> {
        writeln!(self.file, "push {} {}", segment.to_lowercase(), index)
    }

    pub fn write_pop(&mut self, segment: &str, index: i32) -> Result<()> {
        writeln!(self.file, "pop {} {}", segment.to_lowercase(), index)
    }

    pub fn write_arithmetic(&mut self, command: &str) -> Result<()> {
        writeln!(self.file, "{}", command.to_lowercase())
    }

    pub fn write_label(&mut self, label: &str) -> Result<()> {
        writeln!(self.file, "label {}", label)
    }

    pub fn write_goto(&mut self, label: &str) -> Result<()> {
        writeln!(self.file, "goto {}", label)
    }

    pub fn write_if(&mut self, label: &str) -> Result<()> {
        writeln!(self.file, "if-goto {}", label)
    }

    pub fn write_call(&mut self, name: &str, n_args: i32) -> Result<()> {
        writeln!(self.file, "call {} {}", name, n_args)
    }

    pub fn write_function(&mut self, name: &str, n_vars: i32) -> Result<()> {
        writeln!(self.file, "function {} {}", name, n_vars)
    }

    pub fn write_return(&mut self) -> Result<()> {
        writeln!(self.file, "return")
    }

    pub fn close(mut self) -> Result<()> {
        self.file.flush() // Ensure everything is written
    }
}
