use crate::{chunk::Chunk, opcode::OpCode};

pub fn disassemble_chunk(chunk: &Chunk, name: &str) {
    println!("== {} ==", name);

    let mut offset = 0;
    loop {
        if offset >= chunk.op_codes_len() {
            break;
        }

        disassemble_instruction(chunk, &mut offset);
    }
}

pub fn disassemble_instruction(chunk: &Chunk, offset: &mut usize) {
    print!("{:04}", offset);
    
    let curr_line = chunk.line_from_offset(*offset);
    if *offset > 0 && curr_line == chunk.line_from_offset(*offset - 1) {
        print!("   | ");
    } else {
        print!("{:4} ", curr_line);
    }

    let instruction = chunk.op_codes_at(*offset);
    match OpCode::try_from(instruction) {
        Ok(o) => match o {
            OpCode::CONSTANT => constant_instruction("OP_CONSTANT",chunk, offset),
            OpCode::NEGATE => simple_instruction("OP_NEGATE", offset),
            OpCode::RETURN => simple_instruction("OP_RETURN", offset),
            OpCode::ADD => simple_instruction("OP_ADD", offset),
            OpCode::SUBTRACT => simple_instruction("OP_SUBTRACT", offset),
            OpCode::MULTIPLY => simple_instruction("OP_MULTIPLY", offset),
            OpCode::DIVIDE => simple_instruction("OP_DIVIDE", offset),
        },
        Err(_) => {
            eprintln!("Unknown OpCode: `invalid instruction received while converting to opcode`");
            *offset += 1;
        }
    }
}

fn simple_instruction(instruction_name: &str, offset: &mut usize) {
    println!("{}", instruction_name);
    *offset += 1;
}

fn constant_instruction(instruction_name: &str, chunk: &Chunk, offset: &mut usize) {
    let constant_idx = chunk.op_codes_at(*offset + 1);
    print!("{:<16} {:4} ", instruction_name, constant_idx);
    println!("'{}'", chunk.get_constant(constant_idx as usize));

    *offset += 2;
}