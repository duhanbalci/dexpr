use crate::bytecode::BytecodeReader;
use crate::opcodes::OpCodeByte;

/// A utility function to disassemble bytecode for debugging
pub fn disassemble_bytecode(bytecode: &[u8]) -> Vec<String> {
  let mut result = Vec::new();
  let mut reader = BytecodeReader::new(bytecode);

  while reader.remaining() > 0 {
    let start_position = reader.position();
    let opcode_byte = match reader.read_byte() {
      Ok(b) => b,
      Err(_) => break,
    };

    let opcode = match OpCodeByte::from_byte(opcode_byte) {
      Some(op) => op,
      None => {
        result.push(format!(
          "{:04x}: Unknown opcode: 0x{:02x}",
          start_position, opcode_byte
        ));
        continue;
      }
    };

    let instruction = match opcode {
      OpCodeByte::LoadConst => {
        let reg = reader.read_byte();
        let value = reader.read_value();
        match (reg, value) {
          (Ok(r), Ok(v)) => format!("{:04x}: LoadConst r{}, {}", start_position, r, v),
          _ => format!("{:04x}: LoadConst (truncated)", start_position),
        }
      }
      OpCodeByte::Move => {
        let dest = reader.read_byte();
        let src = reader.read_byte();
        match (dest, src) {
          (Ok(d), Ok(s)) => format!("{:04x}: Move r{} = r{}", start_position, d, s),
          _ => format!("{:04x}: Move (truncated)", start_position),
        }
      }
      OpCodeByte::LoadLocal => {
        let reg = reader.read_byte();
        let offset = reader.read_byte();
        match (reg, offset) {
          (Ok(r), Ok(o)) => format!("{:04x}: LoadLocal r{}, offset={}", start_position, r, o),
          _ => format!("{:04x}: LoadLocal (truncated)", start_position),
        }
      }
      OpCodeByte::StoreLocal => {
        let offset = reader.read_byte();
        let reg = reader.read_byte();
        match (offset, reg) {
          (Ok(o), Ok(r)) => format!("{:04x}: StoreLocal offset={}, r{}", start_position, o, r),
          _ => format!("{:04x}: StoreLocal (truncated)", start_position),
        }
      }
      OpCodeByte::LoadGlobal => {
        let reg = reader.read_byte();
        let name = reader.read_string();
        match (reg, name) {
          (Ok(r), Ok(n)) => format!("{:04x}: LoadGlobal r{}, \"{}\"", start_position, r, n),
          _ => format!("{:04x}: LoadGlobal (truncated)", start_position),
        }
      }
      OpCodeByte::StoreGlobal => {
        let name = reader.read_string();
        let reg = reader.read_byte();
        match (name, reg) {
          (Ok(n), Ok(r)) => format!("{:04x}: StoreGlobal \"{}\", r{}", start_position, n, r),
          _ => format!("{:04x}: StoreGlobal (truncated)", start_position),
        }
      }
      OpCodeByte::Add
      | OpCodeByte::Sub
      | OpCodeByte::Mul
      | OpCodeByte::Div
      | OpCodeByte::Mod
      | OpCodeByte::Pow
      | OpCodeByte::Lt
      | OpCodeByte::Lte
      | OpCodeByte::Gt
      | OpCodeByte::Gte
      | OpCodeByte::Eq
      | OpCodeByte::Neq
      | OpCodeByte::And
      | OpCodeByte::Or
      | OpCodeByte::Contains
      | OpCodeByte::Concat => {
        let res = reader.read_byte();
        let left = reader.read_byte();
        let right = reader.read_byte();
        match (res, left, right) {
          (Ok(r), Ok(l), Ok(rg)) => {
            format!("{:04x}: {:?} r{}, r{}, r{}", start_position, opcode, r, l, rg)
          }
          _ => format!("{:04x}: {:?} (truncated)", start_position, opcode),
        }
      }
      OpCodeByte::Neg | OpCodeByte::Not => {
        let res = reader.read_byte();
        let operand = reader.read_byte();
        match (res, operand) {
          (Ok(r), Ok(o)) => format!("{:04x}: {:?} r{}, r{}", start_position, opcode, r, o),
          _ => format!("{:04x}: {:?} (truncated)", start_position, opcode),
        }
      }
      OpCodeByte::Jump => match reader.read_u32() {
        Ok(addr) => format!("{:04x}: Jump -> 0x{:04x}", start_position, addr),
        Err(_) => format!("{:04x}: Jump (truncated)", start_position),
      },
      OpCodeByte::JumpIfFalse => {
        let reg = reader.read_byte();
        let addr = reader.read_u32();
        match (reg, addr) {
          (Ok(r), Ok(a)) => format!("{:04x}: JumpIfFalse r{} -> 0x{:04x}", start_position, r, a),
          _ => format!("{:04x}: JumpIfFalse (truncated)", start_position),
        }
      }
      OpCodeByte::MethodCall => {
        let res = reader.read_byte();
        let obj = reader.read_byte();
        let method = reader.read_string();
        let arg_count = reader.read_byte();
        match (res, obj, method, arg_count) {
          (Ok(r), Ok(o), Ok(m), Ok(count)) => {
            let mut arg_regs = Vec::new();
            let mut truncated = false;
            for _ in 0..count {
              match reader.read_byte() {
                Ok(reg) => arg_regs.push(format!("r{}", reg)),
                Err(_) => {
                  truncated = true;
                  break;
                }
              }
            }
            if truncated {
              format!(
                "{:04x}: MethodCall r{} = r{}.{}(truncated args)",
                start_position, r, o, m
              )
            } else {
              format!(
                "{:04x}: MethodCall r{} = r{}.{}({})",
                start_position,
                r,
                o,
                m,
                arg_regs.join(", ")
              )
            }
          }
          _ => format!("{:04x}: MethodCall (truncated)", start_position),
        }
      }
      OpCodeByte::Log => match reader.read_byte() {
        Ok(reg) => format!("{:04x}: Log r{}", start_position, reg),
        Err(_) => format!("{:04x}: Log (truncated)", start_position),
      },
      OpCodeByte::CallDefault => {
        let res = reader.read_byte();
        let fn_id = reader.read_byte();
        let arg_count = reader.read_byte();
        match (res, fn_id, arg_count) {
          (Ok(r), Ok(id), Ok(count)) => {
            let fn_name = crate::opcodes::default_fn::name(id).unwrap_or("?");
            let mut arg_regs = Vec::new();
            for _ in 0..count {
              if let Ok(reg) = reader.read_byte() {
                arg_regs.push(format!("r{}", reg));
              }
            }
            format!(
              "{:04x}: CallDefault r{} = {}({})",
              start_position, r, fn_name, arg_regs.join(", ")
            )
          }
          _ => format!("{:04x}: CallDefault (truncated)", start_position),
        }
      }
      OpCodeByte::CallExternal => {
        let res = reader.read_byte();
        let name = reader.read_string();
        let arg_count = reader.read_byte();
        match (res, name, arg_count) {
          (Ok(r), Ok(n), Ok(count)) => {
            let mut arg_regs = Vec::new();
            for _ in 0..count {
              if let Ok(reg) = reader.read_byte() {
                arg_regs.push(format!("r{}", reg));
              }
            }
            format!(
              "{:04x}: CallExternal r{} = {}({})",
              start_position, r, n, arg_regs.join(", ")
            )
          }
          _ => format!("{:04x}: CallExternal (truncated)", start_position),
        }
      }
      OpCodeByte::GetProperty => {
        let dest = reader.read_byte();
        let obj = reader.read_byte();
        let prop = reader.read_string();
        match (dest, obj, prop) {
          (Ok(d), Ok(o), Ok(p)) => format!("{:04x}: GetProperty r{} = r{}.{}", start_position, d, o, p),
          _ => format!("{:04x}: GetProperty (truncated)", start_position),
        }
      }
      OpCodeByte::SetProperty => {
        let obj = reader.read_byte();
        let prop = reader.read_string();
        let val = reader.read_byte();
        match (obj, prop, val) {
          (Ok(o), Ok(p), Ok(v)) => format!("{:04x}: SetProperty r{}.{} = r{}", start_position, o, p, v),
          _ => format!("{:04x}: SetProperty (truncated)", start_position),
        }
      }
      OpCodeByte::SetResult => match reader.read_byte() {
        Ok(reg) => format!("{:04x}: SetResult r{}", start_position, reg),
        Err(_) => format!("{:04x}: SetResult (truncated)", start_position),
      },
      OpCodeByte::End => format!("{:04x}: End", start_position),
    };

    result.push(instruction);
  }

  result
}

