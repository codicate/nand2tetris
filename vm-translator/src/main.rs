use code_writer::CodeWriter;
use std::{
    fs::File,
    io::{self, BufRead, BufReader, Write},
    path::Path,
};

mod code_writer;

fn main() -> io::Result<()> {
    // input file
    let file_name = std::env::args().nth(1).expect("No arguments provided.");
    let input_file = File::open(&file_name)?;
    let reader = BufReader::new(input_file);

    // output file
    let stem = Path::new(&file_name).file_stem().unwrap().to_str().unwrap();
    let output_file_name = format!("{stem}.asm");
    let mut output_file = File::create(output_file_name).unwrap();

    let mut code_writer = CodeWriter::new(stem.to_string());

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

    Ok(())
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
        // // Branching commands (label, goto, if-goto)
        // command @ ("label" | "goto" | "if-goto") => {
        //     let label = parts.next().ok_or("Expected a label")?;
        //     handle_branching(command, label);
        //     Ok(())
        // }

        // // Function-related commands (function, call, return)
        // "function" => {
        //     let name = parts.next().ok_or("Expected a function name")?;
        //     let num_args = parts
        //         .next()
        //         .ok_or("Expected number of arguments for function")?
        //         .parse::<i32>()
        //         .map_err(|_| "Failed to parse number of arguments")?;
        //     handle_function("function", name, Some(num_args));
        //     Ok(())
        // }
        // "call" => {
        //     let name = parts.next().ok_or("Expected a function name")?;
        //     let num_args = parts
        //         .next()
        //         .ok_or("Expected number of arguments for call")?
        //         .parse::<i32>()
        //         .map_err(|_| "Failed to parse number of arguments")?;
        //     handle_function("call", name, Some(num_args));
        //     Ok(())
        // }
        // "return" => {
        //     handle_function("return", "", None);
        //     Ok(())
        // }
        _ => panic!("unknown command: {command}"),
    };

    return Ok(assembly);
}
