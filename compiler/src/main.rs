mod parser;
mod tokenizer;
mod writer;

use parser::Parser;
use std::path::{Path, PathBuf};
use std::{env, fs};

// Convert the file extension of a given path to the specified extension
pub fn convert_file_extension(path: &Path, extension: &str) -> PathBuf {
    let stem = path.file_stem().unwrap().to_string_lossy();
    let parent = path.parent().unwrap_or_else(|| Path::new(""));

    // Construct a new path: parent + stem + ".vm"
    parent.join(format!("{}.{}", stem, extension))
}

// fn generate_tokenizer_output(path: &Path, content: String) {
//     let mut tokenizer = Tokenizer::new(path.to_string_lossy().to_string(), content);
//     let output = tokenizer.output();
//     let output_path = convert_file_extension(path, "tokens.xml");
//     let mut output_file = File::create(output_path).unwrap();
//     output_file.write_all(output.as_bytes()).unwrap();
// }

fn parse_file(path: &Path) {
    let content = match fs::read_to_string(path) {
        Ok(text) => text,
        Err(e) => {
            eprintln!("Failed to read file {}: {}", path.display(), e);
            return;
        }
    };

    let mut parser = Parser::new(path, content);
    parser.parse();
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <file_or_directory>", args[0]);
        std::process::exit(1);
    }

    let input_path = Path::new(&args[1]);
    if !input_path.exists() {
        eprintln!("Error: Path does not exist: {}", input_path.display());
        std::process::exit(1);
    }

    if input_path.is_file() {
        if input_path.extension().and_then(|s| s.to_str()) == Some("jack") {
            parse_file(input_path);
        } else {
            eprintln!(
                "Error: File does not end with .jack: {}",
                input_path.display()
            );
            std::process::exit(1);
        }
    } else if input_path.is_dir() {
        match fs::read_dir(input_path) {
            Ok(entries) => {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.extension().and_then(|s| s.to_str()) == Some("jack") {
                        parse_file(&path);
                    }
                }
            }
            Err(e) => {
                eprintln!("Error reading directory: {}", e);
                std::process::exit(1);
            }
        }
    } else {
        eprintln!("Error: Not a file or directory: {}", input_path.display());
        std::process::exit(1);
    }
}
