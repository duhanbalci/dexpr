use smol_str::SmolStr;

use crate::ast::value::Value;

/// Bytecode writer for generating VM bytecode
#[derive(Debug, Clone)]
pub struct BytecodeWriter {
  buffer: Vec<u8>,
}

impl BytecodeWriter {
  /// Create a new bytecode writer
  pub fn new() -> Self {
    Self { buffer: Vec::new() }
  }

  /// Write a single byte
  pub fn write_byte(&mut self, byte: u8) {
    self.buffer.push(byte);
  }

  /// Write a 16-bit unsigned integer
  pub fn write_u16(&mut self, value: u16) {
    self.buffer.push((value >> 8) as u8);
    self.buffer.push(value as u8);
  }

  /// Write a 32-bit unsigned integer
  pub fn write_u32(&mut self, value: u32) {
    self.buffer.push((value >> 24) as u8);
    self.buffer.push((value >> 16) as u8);
    self.buffer.push((value >> 8) as u8);
    self.buffer.push(value as u8);
  }

  /// Write a register
  pub fn write_register(&mut self, reg: u8) {
    self.buffer.push(reg);
  }

  /// Write a string
  pub fn write_string(&mut self, s: &SmolStr) {
    // String length (2 bytes)
    self.write_u16(s.len() as u16);
    // String data
    self.buffer.extend_from_slice(s.as_bytes());
  }

  /// Write a value
  pub fn write_value(&mut self, value: &Value) {
    self.buffer.extend_from_slice(&value.serialize());
  }

  /// Get the current position
  pub fn position(&self) -> usize {
    self.buffer.len()
  }

  /// Get the bytecode buffer
  pub fn bytecode(&self) -> &[u8] {
    &self.buffer
  }

  /// Consume the writer and return the bytecode
  pub fn into_bytecode(self) -> Vec<u8> {
    self.buffer
  }
}

/// Bytecode reader for parsing VM bytecode
pub struct BytecodeReader<'a> {
  bytecode: &'a [u8],
  position: usize,
}

impl<'a> BytecodeReader<'a> {
  /// Create a new bytecode reader
  pub fn new(bytecode: &'a [u8]) -> Self {
    Self {
      bytecode,
      position: 0,
    }
  }

  /// Read a single byte
  #[inline(always)]
  pub fn read_byte(&mut self) -> Result<u8, String> {
    if self.position >= self.bytecode.len() {
      return Err("Unexpected end of bytecode".to_string());
    }

    let byte = self.bytecode[self.position];
    self.position += 1;
    Ok(byte)
  }

  /// Read a 16-bit unsigned integer
  pub fn read_u16(&mut self) -> Result<u16, String> {
    if self.position + 1 >= self.bytecode.len() {
      return Err("Unexpected end of bytecode".to_string());
    }

    let value =
      ((self.bytecode[self.position] as u16) << 8) | (self.bytecode[self.position + 1] as u16);
    self.position += 2;
    Ok(value)
  }

  /// Read a 32-bit unsigned integer
  pub fn read_u32(&mut self) -> Result<u32, String> {
    if self.position + 3 >= self.bytecode.len() {
      return Err("Unexpected end of bytecode".to_string());
    }

    let value = ((self.bytecode[self.position] as u32) << 24)
      | ((self.bytecode[self.position + 1] as u32) << 16)
      | ((self.bytecode[self.position + 2] as u32) << 8)
      | (self.bytecode[self.position + 3] as u32);
    self.position += 4;
    Ok(value)
  }

  /// Read a register
  #[inline(always)]
  pub fn read_register(&mut self) -> Result<u8, String> {
    self.read_byte()
  }

  /// Read a string
  #[inline(always)]
  pub fn read_string(&mut self) -> Result<SmolStr, String> {
    let length = self.read_u16()? as usize;

    if self.position + length > self.bytecode.len() {
      return Err("Unexpected end of bytecode".to_string());
    }

    let s = std::str::from_utf8(&self.bytecode[self.position..self.position + length])
      .map_err(|_| "Invalid UTF-8 string".to_string())?;
    self.position += length;

    Ok(s.into())
  }

  /// Read a value
  pub fn read_value(&mut self) -> Result<Value, String> {
    if self.position >= self.bytecode.len() {
      return Err("Unexpected end of bytecode".to_string());
    }

    let (value, bytes_read) = Value::deserialize(&self.bytecode[self.position..])?;
    self.position += bytes_read;

    Ok(value)
  }

  /// Get the current position
  pub fn position(&self) -> usize {
    self.position
  }

  /// Set the position
  pub fn set_position(&mut self, position: usize) -> Result<(), String> {
    if position > self.bytecode.len() {
      return Err("Position out of range".to_string());
    }

    self.position = position;
    Ok(())
  }

  /// Get the remaining bytes
  #[inline(always)]
  pub fn remaining(&self) -> usize {
    self.bytecode.len() - self.position
  }
}
