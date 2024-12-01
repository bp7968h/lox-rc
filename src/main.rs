use jlox_rc::chunk::Chunk;
use jlox_rc::debug::disassemble_chunk;
use jlox_rc::opcode::OpCode;
use jlox_rc::InterpretError;
use jlox_rc::vm::VM;
use std::env;
use std::process;
use std::fs;

fn main() {
    // if let Some(args) = env::args().nth(1) {
    //     match fs::read_to_string(&args) {
    //         Ok(content) => {
    //             let mut vm = VM::new();
    //             match vm.interpret(&content) {
    //                 Ok(_) => (),
    //                 Err(e) => {
    //                     match e {
    //                         InterpretError::CompileError => process::exit(65),
    //                         InterpretError::RuntimeError => process::exit(70),
    //                     }
    //                 }
    //             }
    //         },
    //         Err(e) => {
    //             eprintln!("Error Reading Filer: [{}]", e);
    //             process::exit(1);
    //         }
    //     }
    // } else {
    //     eprintln!("Usage: jlox-rc <source_file>");
    //     process::exit(1);
    // }

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

    // disassemble_chunk(&chunk,"test chunk");

    let _ = vm.interpret(chunk);
}