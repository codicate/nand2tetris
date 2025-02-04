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

    let sys_continue = &format!("@SYS.CONTINUE{jump_idx}\n0;JMP");
    let asm = [
        &format!("D=D-M\n@SYS.JUMP{jump_idx}\nD;{jump_type}\n"), // comparison
        DEREF_SP,
        "M=0", // false
        sys_continue,
        &format!("(SYS.JUMP{jump_idx})"), // jump
        DEREF_SP,
        "M=-1", // true
        sys_continue,
        &format!("(SYS.CONTINUE{jump_idx})"), // continue
    ];
    asm.join("\n")
}

pub struct CodeWriter {
    file_name: String,
    jump_idx: u32,
    call_idx: u32,
}

impl CodeWriter {
    pub fn new(file_name: String) -> Self {
        CodeWriter {
            file_name,
            jump_idx: 0,
            call_idx: 0,
        }
    }

    pub fn booting_code(&mut self) -> String {
        // set SP = 256 then call Sys.init
        let call_sys_init = self.handle_function_call("Sys.init", 0);
        format!("@256\nD=A\n@SP\nM=D\n{call_sys_init}")
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

    pub fn handle_branching(&self, command: &str, label: &str) -> String {
        match command {
            "label" => format!("({label})"),
            "goto" => format!("@{label}\n0;JMP"),
            "if-goto" => [DECREMENT_SP, DEREF_SP, "D=M", &format!("@{label}\nD;JNE")].join("\n"),
            _ => panic!("Unknown command: {command}"),
        }
    }

    pub fn handle_return(&self) -> String {
        let mut asm = vec![
            // D = *LCL
            "@LCL\nA=M\nD=A",
            // D = *(LCL - 5), which is the return address
            "@5\nA=D-A\nD=M",
            // store return address to temp variable
            STORE_TEMP,
            // copy return value to *ARG, which will be at the top of the stack when function ends
            DECREMENT_SP,
            DEREF_SP,
            "D=M",
            // *ARG = D
            "@ARG\nA=M\nM=D",
        ];

        // SP = @ARG + 1
        asm.push("@ARG\nD=M");
        asm.push("@SP\nM=D");
        asm.push(INCREMENT_SP);

        // restore caller's original memory segment pointers
        let labels = ["@THAT", "@THIS", "@ARG", "@LCL"];
        for label in labels.iter() {
            // LCL--
            asm.push("@LCL\nM=M-1");
            // D = *LCL
            asm.push("A=M\nD=M");
            // A = @label
            asm.push(label);
            // label = D
            asm.push("M=D")
        }

        // jump to return address
        asm.push(GET_TEMP);
        asm.push("A=D\n0;JMP");

        return asm.join("\n");
    }

    pub fn handle_function_init(&self, name: &str, num_args: u16) -> String {
        // function label
        let label = format!("({name})");
        let mut asm: Vec<&str> = vec![&label];

        // set LCL to SP to initialize local variable segment
        asm.push("@SP\nD=M\n@LCL\nM=D");

        // push and initialize all local vars to 0
        for _ in 0..num_args {
            asm.push(DEREF_SP);
            asm.push("M=0");
            asm.push(INCREMENT_SP);
        }

        return asm.join("\n");
    }

    pub fn handle_function_call(&mut self, func_name: &str, num_args: u16) -> String {
        let return_label = &format!("{}.{}.RETURN{}", self.file_name, func_name, self.call_idx);
        let mut asm: Vec<&str> = vec![];
        let labels = [
            &format!("@{return_label}"),
            "@LCL",
            "@ARG",
            "@THIS",
            "@THAT",
        ];

        // store return address and old segment values
        for label in labels.iter() {
            asm.push(label);
            if *label == labels[0] {
                asm.push("D=A")
            } else {
                asm.push("D=M")
            };

            asm.push(DEREF_SP);
            asm.push("M=D");
            asm.push(INCREMENT_SP);
        }

        // calculate new ARG offset and set it
        asm.push("@SP\nD=M");
        let num_args_offset = &format!("@{num_args}");
        asm.push(num_args_offset);
        asm.push("D=D-A\n@5\nD=D-A");
        asm.push("@ARG\nM=D");

        // jump to function
        let function_label = &format!("@{func_name}");
        asm.push(function_label);
        asm.push("0;JMP");

        // return label
        let return_label = &format!("({return_label})");
        asm.push(return_label);

        self.call_idx += 1;
        return asm.join("\n");
    }
}
