use crate::{
    chunk::Chunk, 
    compiler::Compiler, 
    opcode::OpCode, 
    InterpretResult
};

pub struct VM {
    chunk: Option<Chunk>,
    ip: Option<Vec<u8>>,
    stack: Vec<f64>
}

impl VM{
    pub fn new() -> Self {
        VM { chunk: None, ip: None, stack: Vec::new() }
    }

    pub fn interpret(&mut self, source: &str) -> InterpretResult {
        let mut chunk = Chunk::new();
        let mut compiler = Compiler::new(source, &mut chunk);

        if !compiler.compile() {
            return Err(crate::InterpretError::CompileError);
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
        if let Some(ip) = self.ip.clone() {
            let mut ip_iter = ip.iter();

            while let Some( &instruction) = ip_iter.next() {
                match OpCode::try_from(instruction) {
                    Ok(oc) => {
                        match oc {
                            OpCode::CONSTANT => {
                                if let Some(chunk) = &self.chunk {
                                    if let Some(&const_idx) = ip_iter.next() {
                                        if let Some(constant) = chunk.get_constant(const_idx as usize) {
                                            let cloned = constant.clone();
                                            self.push(cloned);
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
                                return Ok(());
                            },
                            OpCode::ADD => self.binary_op(|a, b| a + b),
                            OpCode::SUBTRACT => self.binary_op(|a,b| a - b),
                            OpCode::MULTIPLY => self.binary_op(|a,b| a * b),
                            OpCode::DIVIDE => self.binary_op(|a,b| a / b),
                        }
                    },
                    Err(e) => Err(e)?,
                }
            }
        }
        Ok(())
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