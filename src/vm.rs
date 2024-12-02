use crate::{
    chunk::Chunk, compiler::Compiler, debug::disassemble_instruction, opcode::OpCode, InterpretResult
};

pub struct VM {
    chunk: Option<Chunk>,
    instr_pos: usize,
    debug: bool,
    stack: Vec<f64>
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
                        OpCode::NEGATE => self.negate_op(),
                        OpCode::ADD => self.binary_op(|a, b| a + b),
                        OpCode::SUBTRACT => self.binary_op(|a,b| a - b),
                        OpCode::MULTIPLY => self.binary_op(|a,b| a * b),
                        OpCode::DIVIDE => self.binary_op(|a,b| a / b),
                    } 
                },
                Err(e) => Err(e)?
            }
        }
    }

    fn read_constant(&mut self) -> f64 {
        let constant_idx = self.read_byte();
        if let Some(chunk) = &self.chunk {
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

    fn binary_op<F>(&mut self, op: F) 
        where F: Fn(f64, f64) -> f64 {
            if let (Some(b), Some(a)) = (self.pop_value(), self.pop_value()) {
                self.push_value(op(a,b));
            } else {
                eprintln!("Error: Not enough values on the stack");
            }
    }

    fn negate_op(&mut self) {
        if let Some(curr_value) = self.stack.last_mut() {
            *curr_value = -*curr_value;
        } else {
            eprintln!("Stack is empty");
        }
    }

    fn push_value(&mut self, value: f64) {
        self.stack.push(value);
    }

    fn pop_value(&mut self) -> Option<f64> {
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