use crate::{
    chunk::Chunk, compiler::Compiler, debug::disassemble_instruction, opcode::OpCode, value::ValueType, InterpretError, InterpretResult
};

pub struct VM{
    chunk: Option<Chunk>,
    instr_pos: usize,
    debug: bool,
    stack: Vec<ValueType>
}

impl VM{
    pub fn new() -> Self {
        VM { chunk: None, instr_pos: 0, debug: true, stack: Vec::new() }
    }

    pub fn set_debug(&mut self, state: bool) {
        self.debug = state
    }

    pub fn interpret(&mut self, source: &str) -> InterpretResult {
        let mut chunk = Chunk::new();
        let mut compiler = Compiler::new(source, &mut chunk);
        
        if !compiler.compile() {
            return Err(crate::InterpretError::CompileError);
        }
       
        self.chunk = Some(chunk);
        self.run()
    }

    pub fn run(&mut self) -> InterpretResult {
        loop {
            if self.debug {
                if let Some(chunk) = &self.chunk {
                    self.show_stack();
                    let _ = disassemble_instruction(chunk, &self.instr_pos);
                }
            }
            println!("constants: {:?}", self.chunk.as_ref().unwrap().constants);

            let instruction = self.read_byte();

            match OpCode::try_from(instruction) {
                Ok(opcode) => {
                    match opcode {
                        OpCode::RETURN => {
                            if let Some(stack_value) = self.pop_value() {
                                println!("{}", stack_value);
                            }
                            return Ok(());
                        },
                        OpCode::CONSTANT => {
                            let constant = self.read_constant();
                            self.push_value(constant);
                        },
                        OpCode::NEGATE => self.negate_op()?,
                        OpCode::ADD => self.binary_op(|a, b| a + b)?,
                        OpCode::SUBTRACT => self.binary_op(|a,b| a - b)?,
                        OpCode::MULTIPLY => self.binary_op(|a,b| a * b)?,
                        OpCode::DIVIDE => self.binary_op(|a,b| a / b)?,
                    } 
                },
                Err(e) => Err(e)?
            }
        }
    }

    fn read_constant(&mut self) -> ValueType {
        let constant_idx = self.read_byte();
        if let Some(ref mut chunk) = self.chunk {
            return chunk.get_constant(constant_idx as usize);
        }

        panic!("[Read Constant] no chunk in vm to read!");
    }

    fn read_byte(&mut self) -> u8 {
        if let Some(chunk) = &self.chunk {
            let curr_instr_pos = self.instr_pos;
            self.instr_pos += 1;

            return chunk.op_codes_at(curr_instr_pos);
        }
        panic!("[Read Byte] no chunk in vm to read!");
    }

    fn binary_op<F>(&mut self, op: F) -> InterpretResult 
        where F: Fn(f64, f64) -> f64 {
            match (self.pop_value(), self.pop_value()) {
                (Some(ValueType::Number(b)), Some(ValueType::Number(a))) => {
                    self.push_value(ValueType::Number(op(a, b)));
                    Ok(())
                }
                (Some(_), Some(_)) => {
                    eprintln!("Operands must be numbers.");
                    InterpretResult::Err(InterpretError::RuntimeError)
                }
                _ => {
                    eprintln!("Stack underflow.");
                    InterpretResult::Err(InterpretError::RuntimeError)
                }
            }
    }

    fn negate_op(&mut self) -> InterpretResult {
        if let Some(curr_value) = self.stack.last_mut() {
            match curr_value {
                ValueType::Number(num) => {
                    *num = -*num;
                    return Ok(());
                },
                _ => return Err(InterpretError::RuntimeError),
            }
        }
        Err(InterpretError::RuntimeError)
    }

    fn _peek(&self, distance: usize) -> Option<&ValueType> {
        let stack_len = self.stack.len();
        if distance >= stack_len {
            return None;
        }
        self.stack.get(stack_len - 1 - distance)
    }

    fn push_value(&mut self, value: ValueType) {
        self.stack.push(value);
    }

    fn pop_value(&mut self) -> Option<ValueType> {
        self.stack.pop()
    }

    fn show_stack(&self) {
        if self.stack.is_empty() {
            return;
        }
        print!("          ");
        for stack_value in self.stack.iter() {
            print!("[ {} ]", stack_value);
        }
        println!();
    }
}