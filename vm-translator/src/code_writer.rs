// A = *SP
const DEREF_SP: &str = "@SP\nA=M";

// SP++
const INCREMENT_SP: &str = "@SP\nM=M+1";

// SP--
const DECREMENT_SP: &str = "@SP\nM=M-1";

// R13 = D
const STORE_TEMP: &str = "@R13\nM=D";

// D = R13
const GET_TEMP: &str = "@R13\nD=M";

fn get_segment_pointer(segment: &str) -> &str {
    match segment {
        "local" => "LCL",
        "argument" => "ARG",
        "this" => "THIS",
        "that" => "THAT",
        _ => panic!("Unknown segment: {segment}"),
    }
}

fn handle_comparison(command: &str, jump_idx: u32) -> String {
    let jump_type = match command {
        "eq" => "JEQ",
        "lt" => "JGT",
        "gt" => "JLT",
        _ => panic!("unknown command: {command}"),
    };

    let sys_continue = &format!("@SYSCONTINUE{jump_idx}\n0;JMP");
    let asm = [
        &format!("D=D-M\n@SYSJUMP{jump_idx}\nD;{jump_type}\n"), // comparison
        DEREF_SP,
        "M=0", // false
        sys_continue,
        &format!("(SYSJUMP{jump_idx})"), // jump
        DEREF_SP,
        "M=-1", // true
        sys_continue,
        &format!("(SYSCONTINUE{jump_idx})"), // continue
    ];
    asm.join("\n")
}

pub struct CodeWriter {
    file_name: String,
    jump_idx: u32,
}

impl CodeWriter {
    pub fn new(file_name: String) -> Self {
        CodeWriter {
            file_name,
            jump_idx: 0,
        }
    }
    pub fn handle_arithmetic(&mut self, command: &str) -> String {
        let operation = match command {
            "add" => "M=D+M",
            "sub" => "M=D-M\nM=-M",
            "and" => "M=D&M",
            "or" => "M=D|M",
            "neg" => "M=-M",
            "not" => "M=!M",
            "eq" | "lt" | "gt" => {
                self.jump_idx += 1;
                &handle_comparison(command, self.jump_idx)
            }
            _ => panic!("unknown command: {command}"),
        };

        let second_operand = if matches!(command, "neg" | "not") {
            ""
        } else {
            &["D=M", DECREMENT_SP, DEREF_SP].join("\n")
        };

        let asm = [
            DECREMENT_SP,
            DEREF_SP,
            second_operand,
            operation,
            INCREMENT_SP,
        ];
        asm.join("\n")
    }

    pub fn handle_memory_access(&self, command: &str, segment: &str, index: u16) -> String {
        // A = addr
        let get_address = match segment {
            "local" | "argument" | "this" | "that" => {
                let segment_pointer = get_segment_pointer(segment);
                // addr = segmentPointer + index
                format!("@{segment_pointer}\nD=M\n@{index}\nA=D+A")
            }
            "constant" => {
                format!("@{index}")
            }
            "static" => {
                format!("@{}.{}", self.file_name, index)
            }
            "temp" => {
                // temp variables start at address 5
                let temp_addr = index + 5;
                format!("@{temp_addr}")
            }
            "pointer" => {
                let segment_pointer = if index == 0 { "THIS" } else { "THAT" };
                format!("@{segment_pointer}")
            }
            _ => panic!("Unknown segment: {segment}"),
        };

        // D = *addr, except for constant where D = addr and addr is a constant literal
        let store_value_in_d = if segment == "constant" { "D=A" } else { "D=M" };

        let push_asm = vec![
            &get_address,     // A = addr
            store_value_in_d, // D = *addr
            DEREF_SP,         // A = *SP
            "M=D",            // *SP = *addr
            INCREMENT_SP,     // SP++
        ];

        let pop_asm = vec![
            &get_address, // A = addr
            "D=A",        // D = addr
            STORE_TEMP,   // R13 = D
            DECREMENT_SP, // SP--
            DEREF_SP,     // A = *SP
            "D=M",        // D = *SP
            "@R13\nA=M",  // A = addr
            "M=D",        // *addr = *SP
        ];
        let asm = if command == "push" { push_asm } else { pop_asm };
        asm.join("\n")
    }
}
