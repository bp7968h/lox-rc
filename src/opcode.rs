use crate::InterpretError;

#[derive(Debug, PartialEq, Clone)]
#[repr(u8)]
pub enum OpCode {
    CONSTANT,
    NEGATE,
    RETURN,
    ADD,
    SUBTRACT,
    MULTIPLY,
    DIVIDE,
    NIL,
    TRUE,
    FALSE,
    NOT,
    EQUAL,
    GREATER,
    LESS,
    PRINT,
    POP,
    DefineGlobal,
    GetGlobal,
    SetGlobal,
    GetLocal,
    SetLocal,
    JumpIfFalse,
    JUMP,
    LOOP,
}

impl TryFrom<u8> for OpCode {
    type Error = InterpretError;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(OpCode::CONSTANT),
            1 => Ok(OpCode::NEGATE),
            2 => Ok(OpCode::RETURN),
            3 => Ok(OpCode::ADD),
            4 => Ok(OpCode::SUBTRACT),
            5 => Ok(OpCode::MULTIPLY),
            6 => Ok(OpCode::DIVIDE),
            7 => Ok(OpCode::NIL),
            8 => Ok(OpCode::TRUE),
            9 => Ok(OpCode::FALSE),
            10 => Ok(OpCode::NOT),
            11 => Ok(OpCode::EQUAL),
            12 => Ok(OpCode::GREATER),
            13 => Ok(OpCode::LESS),
            14 => Ok(OpCode::PRINT),
            15 => Ok(OpCode::POP),
            16 => Ok(OpCode::DefineGlobal),
            17 => Ok(OpCode::GetGlobal),
            18 => Ok(OpCode::SetGlobal),
            19 => Ok(OpCode::GetLocal),
            20 => Ok(OpCode::SetLocal),
            21 => Ok(OpCode::JumpIfFalse),
            22 => Ok(OpCode::JUMP),
            23 => Ok(OpCode::LOOP),
            _ => Err(InterpretError::CompileError),
        }
    }
}

impl From<OpCode> for u8 {
    fn from(value: OpCode) -> Self {
        value as u8
    }
}
