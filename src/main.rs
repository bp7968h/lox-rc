use jlox_rc::chunk::{Chunk, OpCode};

fn main() {
    let mut chunk = Chunk::new();

    let constant: usize = chunk.add_constant(1.2);
    chunk.write(OpCode::CONSTANT as u8, 123);
    chunk.write(constant as u8, 123);

    chunk.write(OpCode::RETURN as u8, 123);
    chunk.dissassemble_chunk("test chunk");
}