mod tokenizer;

use std::env;
use std::fs;
use std::path::Path;
use tokenizer::Tokenizer;

fn process_file(path: &Path) {
    let content = match fs::read_to_string(path) {
        Ok(text) => text,
        Err(e) => {
            eprintln!("Failed to read file {}: {}", path.display(), e);
            return;
        }
    };

    let tokenizer = Tokenizer::new(content);
    let output = tokenizer.output();
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
            process_file(input_path);
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
                        process_file(&path);
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
