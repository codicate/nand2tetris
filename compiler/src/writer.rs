use std::fs::File;
use std::io::Write;
use std::path::Path;

pub struct Writer {
    class_name: String,
    file: File,
    buffer: Vec<String>,
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

    pub fn write_push(&mut self, segment: &str, index: u32) {
        self.buffer
            .push(format!("push {} {}", segment.to_lowercase(), index));
    }

    pub fn write_pop(&mut self, segment: &str, index: u32) {
        self.buffer
            .push(format!("pop {} {}", segment.to_lowercase(), index));
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

    pub fn write_call(&mut self, name: &str, n_args: u32) {
        self.buffer.push(format!("call {} {}", name, n_args));
    }

    pub fn write_return(&mut self) {
        self.buffer.push("return".to_string());
    }

    pub fn write_function(&mut self, func_name: &str, n_vars: u32) {
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
