use crate::chunk::{Chunk, OpCode};

pub struct VM<'a> {
    chunk: Option<&'a Chunk>,
    ip: Option<&'a [u8]>,
}

pub enum InterpretResult {
    INTERPRET_OK,
    INTERPRET_COMPILE_ERROR,
    INTERPRET_RUNTIME_ERROR,
}

impl<'a> VM<'a>{
    pub fn new() -> Self {
        VM { chunk: None, ip: None }
    }

    pub fn interpret(&mut self, chunk: &'a Chunk) -> InterpretResult {
        self.chunk = Some(chunk);
        self.ip = Some(chunk.code());

        self.run()
    }

    pub fn run(&mut self) -> InterpretResult {
        if let Some(ip) = self.ip {
            println!("Ip Len: {:?}", ip.len());
            let mut ip_iter = ip.iter().enumerate();

            while let Some((offset ,&instruction)) = ip_iter.next() {
                println!("Offset: {}", offset);
                match OpCode::from_byte(instruction) {
                    Some(oc) => {
                        match oc {
                            OpCode::CONSTANT => {
                                if let Some(chunk) = self.chunk {
                                    if let Some((_, &const_idx)) = ip_iter.next() {
                                        let constant = chunk.get_constant(const_idx as usize);
                                        println!("{:?}", constant);
                                    }
                                }
                                // break;
                            }
                            OpCode::RETURN => return InterpretResult::INTERPRET_OK,
                        }
                    },
                    None => return InterpretResult::INTERPRET_COMPILE_ERROR,
                }
            }
        }

        InterpretResult::INTERPRET_OK
    }

}