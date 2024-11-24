use jlox_rc::{chunk::{Chunk, OpCode}, vm::VM};

fn main() {
    let mut vm = VM::new();
    let mut chunk = Chunk::new();

    let mut constant: usize = chunk.add_constant(1.2);
    chunk.write(OpCode::CONSTANT as u8, 123);
    chunk.write(constant as u8, 123);

    constant = chunk.add_constant(3.4);
    chunk.write(OpCode::CONSTANT as u8, 123);
    chunk.write(constant as u8, 123);

    chunk.write(OpCode::ADD as u8, 123);

    constant = chunk.add_constant(5.6);
    chunk.write(OpCode::CONSTANT as u8, 123);
    chunk.write(constant as u8, 123);

    chunk.write(OpCode::DIVIDE as u8, 123);
    chunk.write(OpCode::NEGATE as u8, 123);

    chunk.write(OpCode::RETURN as u8, 123);

    chunk.dissassemble_chunk("test chunk");

    vm.interpret(&chunk);
}