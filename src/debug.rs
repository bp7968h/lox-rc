use crate::{chunk::Chunk, opcode::OpCode};

pub fn disassemble_chunk(chunk: &Chunk, name: &str) {
    println!("== {} ==", name);

    let mut offset = 0;
    loop {
        if offset >= chunk.op_codes_len() {
            break;
        }

        offset = disassemble_instruction(chunk, &offset);
    }
}

pub fn disassemble_instruction(chunk: &Chunk, offset: &usize) -> usize {
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
            OpCode::CONSTANT => constant_instruction("OP_CONSTANT", chunk, offset),
            OpCode::NEGATE => simple_instruction("OP_NEGATE", offset),
            OpCode::RETURN => simple_instruction("OP_RETURN", offset),
            OpCode::ADD => simple_instruction("OP_ADD", offset),
            OpCode::SUBTRACT => simple_instruction("OP_SUBTRACT", offset),
            OpCode::MULTIPLY => simple_instruction("OP_MULTIPLY", offset),
            OpCode::DIVIDE => simple_instruction("OP_DIVIDE", offset),
            OpCode::NIL => simple_instruction("OP_NIL", offset),
            OpCode::TRUE => simple_instruction("OP_TRUE", offset),
            OpCode::FALSE => simple_instruction("OP_FALSE", offset),
            OpCode::NOT => simple_instruction("OP_NOT", offset),
            OpCode::EQUAL => simple_instruction("OP_EQUAL", offset),
            OpCode::GREATER => simple_instruction("OP_GREATER", offset),
            OpCode::LESS => simple_instruction("OP_LESS", offset),
            OpCode::PRINT => simple_instruction("OP_PRINT", offset),
            OpCode::POP => simple_instruction("OP_POP", offset),
            OpCode::DefineGlobal => constant_instruction("OP_DEFINE_GLOBAL", chunk, offset),
            OpCode::GetGlobal => constant_instruction("OP_GET_GLOBAL", chunk, offset),
            OpCode::SetGlobal => constant_instruction("OP_SET_GLOBAL", chunk, offset),
            OpCode::GetLocal => byte_instruction("OP_GET_LOCAL", chunk, offset),
            OpCode::SetLocal => byte_instruction("OP_SET_LOCAL", chunk, offset),
            OpCode::JUMP => jump_instruction("OP_JUMP", 1, chunk, offset),
            OpCode::JumpIfFalse => jump_instruction("OP_JUMP_IF_FALSE", 1, chunk, offset),
            OpCode::LOOP => jump_instruction("OP_LOOP", -1, chunk, offset),
        },
        Err(_) => {
            eprintln!("Unknown OpCode: `invalid instruction received while converting to opcode`");
            *offset + 1
        }
    }
}

fn jump_instruction(instruction_name: &str, sign: isize, chunk: &Chunk, offset: &usize) -> usize {
    let high = chunk.op_codes_at(offset + 1);
    let low = chunk.op_codes_at(offset + 2);

    let jump = ((high as u16) << 8) | (low as u16);
    println!(
        "{:<16} {:4} -> {}",
        instruction_name,
        offset,
        *offset as isize + 3 + sign * jump as isize
    );

    offset + 3
}

fn byte_instruction(instruction_name: &str, chunk: &Chunk, offset: &usize) -> usize {
    let constant_idx = chunk.op_codes_at(*offset + 1);
    print!("{:<16} {:4} ", instruction_name, constant_idx);

    *offset + 2
}

fn simple_instruction(instruction_name: &str, offset: &usize) -> usize {
    println!("{}", instruction_name);
    *offset + 1
}

fn constant_instruction(instruction_name: &str, chunk: &Chunk, offset: &usize) -> usize {
    let constant_idx = chunk.op_codes_at(*offset + 1);
    print!("{:<16} {:4} ", instruction_name, constant_idx);
    println!("'{}'", &chunk.get_constant(constant_idx as usize));

    *offset + 2
}
