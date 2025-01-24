use std::{fs::File, path::Path};

fn main() {
    // input file
    let file_name = std::env::args().nth(1).expect("No arguments provided.");
    let contents = std::fs::read_to_string(&file_name).unwrap();
    // output file
    let stem = Path::new(&file_name).file_stem().unwrap().to_str().unwrap();
    let output_file_name = format!("{stem}.hack");
    let mut output_file = File::create(output_file_name).unwrap();
}
