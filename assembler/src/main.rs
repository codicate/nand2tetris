use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use std::path::Path;

fn main() {
    let comp_map: HashMap<&str, &str> = [
        ("0", "0101010"),
        ("1", "0111111"),
        ("-1", "0111010"),
        ("D", "0001100"),
        ("A", "0110000"),
        ("!D", "0001101"),
        ("!A", "0110001"),
        ("-D", "0001111"),
        ("-A", "0110011"),
        ("D+1", "0011111"),
        ("A+1", "0110111"),
        ("D-1", "0001110"),
        ("A-1", "0110010"),
        ("D+A", "0000010"),
        ("D-A", "0010011"),
        ("A-D", "0000111"),
        ("D&A", "0000000"),
        ("D|A", "0010101"),
        ("M", "1110000"),
        ("!M", "1110001"),
        ("-M", "1110011"),
        ("M+1", "1110111"),
        ("M-1", "1110010"),
        ("D+M", "1000010"),
        ("D-M", "1010011"),
        ("M-D", "1000111"),
        ("D&M", "1000000"),
        ("D|M", "1010101"),
    ]
    .into_iter()
    .collect();

    let dest_map: HashMap<&str, &str> = [
        ("", "000"),
        ("M", "001"),
        ("D", "010"),
        ("MD", "011"),
        ("A", "100"),
        ("AM", "101"),
        ("AD", "110"),
        ("AMD", "111"),
    ]
    .into_iter()
    .collect();

    let jump_map: HashMap<&str, &str> = [
        ("", "000"),
        ("JGT", "001"),
        ("JEQ", "010"),
        ("JGE", "011"),
        ("JLT", "100"),
        ("JNE", "101"),
        ("JLE", "110"),
        ("JMP", "111"),
    ]
    .into_iter()
    .collect();

    let mut symbol_table: HashMap<&str, u16> = [
        ("SP", 0),
        ("LCL", 1),
        ("ARG", 2),
        ("THIS", 3),
        ("THAT", 4),
        ("R0", 0),
        ("R1", 1),
        ("R2", 2),
        ("R3", 3),
        ("R4", 4),
        ("R5", 5),
        ("R6", 6),
        ("R7", 7),
        ("R8", 8),
        ("R9", 9),
        ("R10", 10),
        ("R11", 11),
        ("R12", 12),
        ("R13", 13),
        ("R14", 14),
        ("R15", 15),
        ("SCREEN", 16384),
        ("KBD", 24576),
    ]
    .into_iter()
    .collect();

    // input file
    let file_name = std::env::args().nth(1).expect("No arguments provided.");
    let contents = std::fs::read_to_string(&file_name).unwrap();

    // output file
    let stem = Path::new(&file_name).file_stem().unwrap().to_str().unwrap();
    let output_file_name = format!("{stem}.hack");
    let mut output_file = File::create(output_file_name).unwrap();

    // trim and filter lines
    let lines: Vec<&str> = contents
        .lines()
        .map(|line| line.trim())
        .filter(|line| !line.is_empty()) // rid of empty lines
        .filter(|line| !line.starts_with("//")) // rid of comments
        .collect();

    // populate symbol table with labels
    let mut address = 0;
    let lines: Vec<&str> = lines
        .into_iter()
        .filter(|line| {
            if line.starts_with("(") {
                let label = line.trim_matches(|c| c == '(' || c == ')');
                symbol_table.insert(label, address);
                false
            } else {
                address += 1;
                true
            }
        })
        .collect();

    // translate asm lines to binary instructions
    let mut memory_idx = 15; // 15 since R0-15 are reserved
    let lines: Vec<String> = lines
        .into_iter()
        .map(|line| {
            if line.starts_with("@") {
                // A instructions
                let value = &line[1..];
                let instruction = value
                    .parse::<u16>()
                    .ok()
                    .or(symbol_table.get(value).copied()) // Try to get from the symbol table
                    .unwrap_or_else(|| {
                        // If not present, allocate it to a new memory address
                        memory_idx += 1;
                        symbol_table.insert(value, memory_idx);
                        memory_idx
                    });
                format!("0{:015b}", instruction)
            } else {
                // C instrucitons
                let (dest, rest) = line.split_once('=').unwrap_or(("", line));
                let (comp, jump) = rest.split_once(';').unwrap_or((rest, ""));
                format!("111{}{}{}", comp_map[comp], dest_map[dest], jump_map[jump])
            }
        })
        .collect();

    // write to output file
    lines
        .iter()
        .try_for_each(|line| writeln!(output_file, "{line}"))
        .unwrap();
}
