use crate::value::ValueType;

#[derive(Debug)]
pub struct Chunk {
  op_codes: Vec<u8>,
  lines: Vec<usize>,
  pub constants: Vec<ValueType>,
}

impl Chunk {
  pub fn new() -> Self {
    Chunk { 
      op_codes: Vec::new() ,
      lines: Vec::new(),
      constants: Vec::new(),
    }
  }

  pub fn op_codes(&self) -> &[u8] {
    &self.op_codes
  }

  pub fn op_codes_len(&self) -> usize {
    self.op_codes.len()
  }

  pub fn op_codes_at(&self, offset: usize) -> u8 {
    self.op_codes[offset]
  }

  pub fn line_from_offset(&self, offset: usize) -> usize {
    if offset < self.lines.len() {
      return self.lines[offset];
    }

    panic!("[Chunk-Lines] Offset is higher than line count");
  }

  pub fn get_constant(&self, idx: usize) -> ValueType {
    if idx < self.constants.len() {
      return self.constants[idx];
    }
    panic!("[Chunk-Constant] Offset is higher than line count");
  }

  pub fn write(&mut self, byte: u8, line: usize) {
    self.op_codes.push(byte);
    self.lines.push(line);
  }

  pub fn add_constant(&mut self, value: ValueType) -> usize {
    self.constants.push(value);

    self.constants.len() - 1
  }
}
