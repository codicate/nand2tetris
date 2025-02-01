use code_writer::CodeWriter;
use std::{
    fs::{self, File},
    io::{self, BufRead, BufReader, Write},
    path::{Path, PathBuf},
};

mod code_writer;

fn main() -> io::Result<()> {
    // input file
    let path = std::env::args().nth(1).expect("No arguments provided.");
    let path = Path::new(&path);
    let input_files = get_input_files(path)?;

    if input_files.len() <= 0 {
        panic!("No .vm file provided");
    }

    // output file
    let stem = path.file_stem().unwrap().to_str().unwrap();
    let output_file_name = format!("{stem}.asm");
    let mut output_file = File::create(output_file_name).unwrap();

    let mut code_writer = CodeWriter::new(stem.to_string());
    for file in input_files {
        let file = File::open(file)?;
        let reader = BufReader::new(file);

        for (_, line) in reader.lines().enumerate() {
            let line = line?;
            let line = line.trim();
            if line.is_empty() || line.starts_with("//") {
                continue;
            }

            let assembly = parse_line(&mut code_writer, line)?;
            writeln!(output_file, "//{line}")?;
            writeln!(output_file, "{assembly}\n")?;
        }
    }

    Ok(())
}

fn is_vm_file(path: &Path) -> bool {
    path.extension().map_or(false, |ext| ext == "vm")
}

fn get_input_files(path: &Path) -> io::Result<Vec<PathBuf>> {
    let mut files: Vec<PathBuf> = vec![];

    if path.is_file() {
        if is_vm_file(path) {
            files.push(path.to_path_buf());
        }
    } else if path.is_dir() {
        let entries = fs::read_dir(path)?;
        for entry in entries.flatten() {
            let file_path = entry.path();
            if file_path.is_file() && is_vm_file(&file_path) {
                files.push(file_path);
            }
        }
    }

    Ok(files)
}

fn parse_line(code_writer: &mut CodeWriter, line: &str) -> io::Result<String> {
    let mut parts = line.split_whitespace();
    let command = parts.next().expect("emtpy line");

    let assembly = match command {
        // arithmetic/logical commands
        "add" | "sub" | "neg" | "eq" | "gt" | "lt" | "and" | "or" | "not" => {
            code_writer.handle_arithmetic(command)
        }
        // memory access commands
        "push" | "pop" => {
            let segment = parts.next().expect("Expected a segment after push/pop");
            let index = parts
                .next()
                .expect("Expected an index after segment")
                .parse::<u16>()
                .expect("Failed to parse index");
            code_writer.handle_memory_access(command, segment, index)
        }
        // Branching commands
        "label" | "goto" | "if-goto" => {
            let label = parts.next().expect("Expected a label");
            code_writer.handle_branching(command, label)
        }
        // Function commands
        "return" => code_writer.handle_return(),
        "function" | "call" => {
            let name = parts.next().expect("Expected a function name");
            let num_args = parts
                .next()
                .expect("Expected number of arguments for function")
                .parse::<u16>()
                .expect("Failed to parse number of arguments");
            code_writer.handle_function(command, name, num_args)
        }
        _ => panic!("unknown command: {command}"),
    };

    return Ok(assembly);
}
