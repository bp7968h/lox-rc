#[derive(Debug, Clone)]
pub struct Chunk {
  code: Vec<u8>,
  lines: Vec<usize>,
  constants: Vec<f64>,
}

impl Chunk {
  pub fn new() -> Self {
    Chunk { 
      code: Vec::new() ,
      lines: Vec::new(),
      constants: Vec::new(),
    }
  }

  pub fn code(&self) -> &[u8] {
    &self.code
  }

  pub fn get_constant(&self, idx: usize) -> Option<f64> {
    if idx < self.constants.len() {
      return Some(self.constants[idx]);
    }
    None
  }

  pub fn write(&mut self, byte: u8, line: usize) {
    self.code.push(byte);
    self.lines.push(line);
  }

  pub fn dissassemble_chunk(&self, name: &str)  {
    println!("== {} ==", name);
    let mut iter = self.code.iter().enumerate();

    while let Some((offset, byte)) = iter.next() {

      print!("{:04}", offset);
      if offset > 0 && self.lines[offset] == self.lines[offset - 1] {
        print!("   | ");
      } else {
        print!("{:4} ", self.lines[offset]);
      }

      match byte {
        0 => {
          let constant = self.code[offset + 1];
          print!("{:<16} {} ", "CONSTANT", byte);
          println!("'{}'", self.constants[constant as usize]);
          //skip next iteration
          let _ = iter.next();
        },
        1 => println!("NEGATE"),
        2 => println!("RETURN"),
        3 => println!("ADD"),
        4 => println!("SUBTRACT"),
        5 => println!("MULTIPLY"),
        6 => println!("DIVIDE"),
        other => println!("Unknown opcode {}", other)
      }
    }
  }

  pub fn add_constant(&mut self, value: f64) -> usize {
    self.constants.push(value);

    self.constants.len() - 1
  }
}
