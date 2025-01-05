pub mod chunk;
pub mod compiler;
pub mod debug;
pub mod object;
pub mod opcode;
pub mod scanner;
pub mod token;
pub mod value;
pub mod vm;

pub type InterpretResult = Result<(), InterpretError>;
pub enum InterpretError {
    CompileError,
    RuntimeError,
}
