use crate::{chunk::{Chunk, OpCode}, compiler::Compiler};

pub struct VM<'a> {
    chunk: Option<Chunk>,
    ip: Option<Vec<u8>>,
    stack: Vec<f64>
}

pub enum InterpretResult {
    INTERPRET_OK,
    INTERPRET_COMPILE_ERROR,
    INTERPRET_RUNTIME_ERROR,
}

impl<'a> VM<'a>{
    pub fn new() -> Self {
        VM { chunk: None, ip: None, stack: Vec::new() }
    }

    pub fn interpret(&mut self, source: &str) -> InterpretResult {
        let mut chunk = Chunk::new();

        if !Compiler::compile(source, &mut chunk) {
            return InterpretResult::INTERPRET_COMPILE_ERROR;
        }

        let code = chunk.code().to_owned();
        self.chunk = Some(chunk);
        self.ip = Some(code);

        self.run()
    }

    fn push(&mut self, value: f64) {
        self.stack.push(value);
    }

    fn pop(&mut self) -> Option<f64> {
        self.stack.pop()
    }

    pub fn run(&mut self) -> InterpretResult {
        if let Some(ip) = &self.ip {
            let mut ip_iter = ip.iter().enumerate();

            while let Some((_offset ,&instruction)) = ip_iter.next() {
                match OpCode::from_byte(instruction) {
                    Some(oc) => {
                        match oc {
                            OpCode::CONSTANT => {
                                if let Some(chunk) = self.chunk {
                                    if let Some((_, &const_idx)) = ip_iter.next() {
                                        if let Some(constant) = chunk.get_constant(const_idx as usize) {
                                            self.push(constant);
                                        } else {
                                            println!("Value not found");
                                        }
                                    }
                                }
                            },
                            OpCode::NEGATE => {
                                if let Some(value) = self.pop() {
                                    self.push(-value);
                                }
                            },
                            OpCode::RETURN => {
                                if let Some(value) = self.pop() {
                                    println!("{}", value);
                                }
                                return InterpretResult::INTERPRET_OK
                            },
                            OpCode::ADD => self.binary_op(|a, b| a + b),
                            OpCode::SUBTRACT => self.binary_op(|a,b| a - b),
                            OpCode::MULTIPLY => self.binary_op(|a,b| a * b),
                            OpCode::DIVIDE => self.binary_op(|a,b| a / b),
                        }
                    },
                    None => return InterpretResult::INTERPRET_COMPILE_ERROR,
                }
            }
        }

        InterpretResult::INTERPRET_OK
    }

    fn binary_op<F>(&mut self, op: F) 
        where F: Fn(f64, f64) -> f64 {
            if let (Some(b), Some(a)) = (self.pop(), self.pop()) {
                self.push(op(a,b));
            } else {
                eprintln!("Error: Not enough values on the stack");
            }
        }

}