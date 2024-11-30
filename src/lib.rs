pub mod chunk;
pub mod debug;
pub mod vm;
pub mod compiler;
pub mod scanner;
pub mod token;
pub mod opcode;


pub type InterpretResult = Result<(), InterpretError>;
pub enum InterpretError {
    CompileError,
    RuntimeError
}
