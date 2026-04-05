use crate::{ast::value::Value, bytecode::BytecodeReader, opcodes::OpCodeByte};
use bumpalo::Bump;
use micromap::Map;
use rust_decimal::{Decimal, MathematicalOps};
use rustc_hash::FxHashMap;
use smol_str::SmolStr;

/// Type alias for external (host) functions
pub type ExternalFn = Box<dyn Fn(&[Value]) -> Result<Value, String>>;

/// Type alias for external (host) methods on Value types
pub type ExternalMethod = Box<dyn Fn(&Value, &[Value]) -> Result<Value, String>>;

use super::debug_info::DebugInfo;
use super::error::VMError;

/// Maximum number of registers
pub const MAX_REGISTERS: usize = 8;

#[cfg(debug_assertions)]
macro_rules! log_debug {
  ($vm:expr, $($arg:tt)*) => {
    if $vm.debug {
      println!($($arg)*);
    }
  };
}

#[cfg(not(debug_assertions))]
macro_rules! log_debug {
  ($vm:expr, $($arg:tt)*) => {};
}

/// Virtual Machine for executing dExpr bytecode
pub struct VM<'a> {
  bytecode: &'a [u8],
  pub(super) reader: BytecodeReader<'a>,
  pc: usize,

  pub(super) registers: [Value; MAX_REGISTERS],

  globals: Map<SmolStr, Value, 64>,

  last_result: Value,

  pub(super) external_functions: Option<FxHashMap<SmolStr, ExternalFn>>,

  pub(super) external_methods: Option<FxHashMap<(SmolStr, SmolStr), ExternalMethod>>,

  heap: Bump,

  debug_info: Option<&'a DebugInfo>,

  #[cfg(debug_assertions)]
  pub(super) debug: bool,

  #[cfg(debug_assertions)]
  opcode_counts: [usize; 256],
}

impl<'a> VM<'a> {
  /// Create a new VM instance
  pub fn new(bytecode: &'a [u8]) -> Self {
    Self {
      bytecode,
      reader: BytecodeReader::new(bytecode),
      pc: 0,
      registers: [const { Value::Null }; MAX_REGISTERS],
      globals: Map::new(),
      last_result: Value::Null,
      external_functions: None,
      external_methods: None,
      heap: Bump::new(),
      debug_info: None,
      #[cfg(debug_assertions)]
      debug: false,
      #[cfg(debug_assertions)]
      opcode_counts: [0; 256],
    }
  }

  /// Set debug info for better error messages
  pub fn set_debug_info(&mut self, debug_info: &'a DebugInfo) {
    self.debug_info = Some(debug_info);
  }

  /// Wrap an error with source location if debug info is available
  fn wrap_error(&self, err: VMError) -> VMError {
    if let Some(debug_info) = self.debug_info {
      if let Some(span) = debug_info.get_span(self.pc as u32) {
        return err.with_span(span);
      }
    }
    err
  }

  /// Set a global variable
  #[inline(never)]
  pub fn set_global(&mut self, name: &str, value: Value) {
    self.globals.insert(name.into(), value);
  }

  /// Get a global variable
  pub fn get_global(&self, name: &str) -> Option<&Value> {
    self.globals.get(name)
  }

  /// Register an external (host) function
  pub fn register_function<F>(&mut self, name: &str, f: F)
  where
    F: Fn(&[Value]) -> Result<Value, String> + 'static,
  {
    self
      .external_functions
      .get_or_insert_with(FxHashMap::default)
      .insert(name.into(), Box::new(f));
  }

  /// Register an external (host) method on a specific type
  pub fn register_method<F>(&mut self, type_name: &str, method_name: &str, f: F)
  where
    F: Fn(&Value, &[Value]) -> Result<Value, String> + 'static,
  {
    self
      .external_methods
      .get_or_insert_with(FxHashMap::default)
      .insert((type_name.into(), method_name.into()), Box::new(f));
  }

  /// Enable/disable debug output
  #[cfg(debug_assertions)]
  pub fn set_debug(&mut self, debug: bool) {
    self.debug = debug;
  }

  /// Enable/disable debug output (no-op in release)
  #[cfg(not(debug_assertions))]
  pub fn set_debug(&mut self, _debug: bool) {}

  /// Reset the VM state
  pub fn reset(&mut self) {
    self.reader = BytecodeReader::new(self.bytecode);
    self.pc = 0;
    self.registers = [const { Value::Null }; MAX_REGISTERS];
    self.last_result = Value::Null;
    // Preserve globals
    self.heap = Bump::new();
  }

  /// Execute the bytecode program and return the last expression result
  pub fn execute(&mut self) -> Result<Value, VMError> {
    self.reset();

    while self.reader.remaining() > 0 {
      #[cfg(debug_assertions)]
      self.debug_print_state();

      // Store instruction position for error reporting
      self.pc = self.reader.position();

      let opcode_byte = self
        .reader
        .read_byte()
        .map_err(|e| self.wrap_error(VMError::BytecodeError(e)))?;

      #[cfg(debug_assertions)]
      {
        self.opcode_counts[opcode_byte as usize] += 1;
      }

      // Skip 0x00 bytes (treat as NOP)
      if opcode_byte == 0x00 {
        log_debug!(self, "Skipping NOP (0x00)");
        continue;
      }

      let opcode = OpCodeByte::from_byte(opcode_byte)
        .ok_or_else(|| self.wrap_error(VMError::BytecodeError(format!("Invalid opcode: {:02x}", opcode_byte))))?;

      log_debug!(self, "Executing: {:?}", opcode.name());

      let result = match opcode {
        OpCodeByte::LoadConst => self.handle_load_const(),
        OpCodeByte::Move => self.handle_move(),
        OpCodeByte::LoadLocal => self.handle_load_local(),
        OpCodeByte::StoreLocal => self.handle_store_local(),
        OpCodeByte::LoadGlobal => self.handle_load_global(),
        OpCodeByte::StoreGlobal => self.handle_store_global(),
        OpCodeByte::Add => self.handle_add(),
        OpCodeByte::Sub => self.binary_op(|a, b| Ok(a - b), "subtract"),
        OpCodeByte::Mul => self.binary_op(|a, b| Ok(a * b), "multiply"),
        OpCodeByte::Div => self.binary_op(
          |a, b| {
            if b.is_zero() {
              Err(VMError::DivisionByZero)
            } else {
              Ok(a / b)
            }
          },
          "divide",
        ),
        OpCodeByte::Neg => self.handle_neg(),
        OpCodeByte::Mod => self.binary_op(
          |a, b| {
            if b.is_zero() {
              Err(VMError::DivisionByZero)
            } else {
              Ok(a % b)
            }
          },
          "modulo",
        ),
        OpCodeByte::Pow => self.binary_op(|a, b| Ok(a.powd(b)), "power"),
        OpCodeByte::Lt => self.compare_op(|a, b| a < b, "less than"),
        OpCodeByte::Lte => self.compare_op(|a, b| a <= b, "less than or equal"),
        OpCodeByte::Gt => self.compare_op(|a, b| a > b, "greater than"),
        OpCodeByte::Gte => self.compare_op(|a, b| a >= b, "greater than or equal"),
        OpCodeByte::Eq => self.compare_op(|a, b| a == b, "equal"),
        OpCodeByte::Neq => self.compare_op(|a, b| a != b, "not equal"),
        OpCodeByte::Contains => self.handle_contains(),
        OpCodeByte::And => self.handle_and(),
        OpCodeByte::Or => self.handle_or(),
        OpCodeByte::Not => self.handle_not(),
        OpCodeByte::Jump => self.handle_jump(),
        OpCodeByte::JumpIfFalse => self.handle_jump_if_false(),
        OpCodeByte::Concat => self.handle_concat(),
        OpCodeByte::GetProperty => self.handle_get_property(),
        OpCodeByte::SetProperty => self.handle_set_property(),
        OpCodeByte::MethodCall => self.handle_method_call(),
        OpCodeByte::Log => self.handle_log(),
        OpCodeByte::CallExternal => self.handle_call_external(),
        OpCodeByte::CallDefault => self.handle_call_default(),
        OpCodeByte::SetResult => self.handle_set_result(),
        OpCodeByte::ClearResult => {
          self.last_result = Value::Null;
          log_debug!(self, "ClearResult");
          Ok(())
        }
        OpCodeByte::End => {
          log_debug!(self, "End of program");
          #[cfg(debug_assertions)]
          self.print_profile_summary();
          return Ok(std::mem::take(&mut self.last_result));
        }
      };

      // Wrap any error with source location
      result.map_err(|e| self.wrap_error(e))?;
    }

    #[cfg(debug_assertions)]
    self.print_profile_summary();
    Ok(std::mem::take(&mut self.last_result))
  }

  #[cfg(debug_assertions)]
  fn print_profile_summary(&self) {
    if !self.debug {
      return;
    }
    println!("\n--- VM Opcode Profile ---");
    let mut counts: Vec<(u8, usize)> = self
      .opcode_counts
      .iter()
      .enumerate()
      .filter(|(_, &count)| count > 0)
      .map(|(op, &count)| (op as u8, count))
      .collect();

    counts.sort_by(|a, b| b.1.cmp(&a.1));

    for (op_byte, count) in counts {
      if let Some(opcode) = OpCodeByte::from_byte(op_byte) {
        println!("{:?}: {}", opcode.name(), count);
      } else {
        println!("0x{:02x}: {}", op_byte, count);
      }
    }
    println!("-------------------------\n");
  }

  // ============================================================================
  // Helper Methods - Bytecode Reading & Validation
  // ============================================================================

  /// Read and validate a register index
  #[inline(always)]
  fn read_register_checked(&mut self) -> Result<usize, VMError> {
    let reg = self
      .reader
      .read_register()
      .map_err(VMError::BytecodeError)? as usize;
    self.validate_register(reg)?;
    Ok(reg)
  }

  /// Validate that a register index is within bounds
  #[inline(always)]
  fn validate_register(&self, _reg: usize) -> Result<(), VMError> {
    #[cfg(debug_assertions)]
    if _reg >= MAX_REGISTERS {
      return Err(VMError::RuntimeError(format!("Invalid register: {}", _reg)));
    }
    Ok(())
  }

  /// Read and validate a jump address
  #[inline(always)]
  fn read_jump_address(&mut self) -> Result<usize, VMError> {
    let addr = self
      .reader
      .read_u32()
      .map_err(VMError::BytecodeError)? as usize;
    #[cfg(debug_assertions)]
    if addr >= self.bytecode.len() {
      return Err(VMError::RuntimeError(format!(
        "Jump target out of range: {}",
        addr
      )));
    }
    Ok(addr)
  }

  /// Set reader position with validation
  #[inline(always)]
  fn set_position(&mut self, addr: usize) -> Result<(), VMError> {
    self
      .reader
      .set_position(addr)
      .map_err(VMError::BytecodeError)
  }

  // ============================================================================
  // Helper Methods - Debug Output
  // ============================================================================

  /// Print debug state information
  #[cfg(debug_assertions)]
  fn debug_print_state(&self) {
    if self.debug {
      println!(
        "PC: {}",
        self.reader.position(),
      );
      println!("Registers: {:?}", self.registers);
    }
  }

  // ============================================================================
  // Opcode Handlers - Register Operations
  // ============================================================================

  /// Handle LoadConst opcode - load constant value into register
  #[inline]
  fn handle_load_const(&mut self) -> Result<(), VMError> {
    let reg = self.read_register_checked()?;
    let value = self.reader.read_value().map_err(VMError::BytecodeError)?;
    self.registers[reg] = value;

    log_debug!(self, "LoadConst r{} = {:?}", reg, self.registers[reg]);
    Ok(())
  }

  /// Handle Move opcode - copy register to register
  #[inline]
  fn handle_move(&mut self) -> Result<(), VMError> {
    let dest = self.read_register_checked()?;
    let src = self.read_register_checked()?;
    self.registers[dest] = self.registers[src].clone();

    log_debug!(self, "Move r{} = r{} ({})", dest, src, self.registers[dest]);
    Ok(())
  }

  // ============================================================================
  // Opcode Handlers - Memory Operations
  // ============================================================================

  /// Handle LoadLocal opcode - load local variable from stack
  #[inline]
  fn handle_load_local(&mut self) -> Result<(), VMError> {
    let _reg = self.read_register_checked()?;
    let _offset = self
      .reader
      .read_byte()
      .map_err(VMError::BytecodeError)?;
    Err(VMError::RuntimeError("LoadLocal not supported without function scope".to_string()))
  }

  /// Handle StoreLocal opcode - store register to local variable
  #[inline]
  fn handle_store_local(&mut self) -> Result<(), VMError> {
    let _offset = self
      .reader
      .read_byte()
      .map_err(VMError::BytecodeError)?;
    let _reg = self.read_register_checked()?;
    Err(VMError::RuntimeError("StoreLocal not supported without function scope".to_string()))
  }

  /// Handle LoadGlobal opcode - load global variable
  #[inline]
  fn handle_load_global(&mut self) -> Result<(), VMError> {
    let reg = self.read_register_checked()?;
    let name = self.reader.read_string().map_err(VMError::BytecodeError)?;

    let value = self
      .globals
      .get(&name)
      .ok_or_else(|| VMError::UndefinedVariable(name.clone()))?;
    self.registers[reg] = value.clone();

    log_debug!(self,
      "LoadGlobal r{} = global.{} ({})",
      reg, name, self.registers[reg]
    );
    Ok(())
  }

  /// Handle StoreGlobal opcode - store register to global variable
  #[inline]
  fn handle_store_global(&mut self) -> Result<(), VMError> {
    let name = self.reader.read_string().map_err(VMError::BytecodeError)?;
    let reg = self.read_register_checked()?;

    self
      .globals
      .insert(name.clone(), self.registers[reg].clone());

    log_debug!(self,
      "StoreGlobal global.{} = r{} ({})",
      name, reg, self.registers[reg]
    );
    Ok(())
  }

  // ============================================================================
  // Opcode Handlers - Arithmetic Operations
  // ============================================================================

  /// Handle Neg opcode - unary negation
  #[inline]
  fn handle_neg(&mut self) -> Result<(), VMError> {
    let dest = self.read_register_checked()?;
    let src = self.read_register_checked()?;

    match &self.registers[src] {
      Value::Number(n) => {
        self.registers[dest] = Value::Number(-n);
      }
      v => {
        return Err(VMError::TypeMismatch {
          expected: "Number".to_string(),
          got: v.type_name().to_string(),
        });
      }
    }

    log_debug!(self, "Neg r{} = -r{}", dest, src);
    Ok(())
  }

  // ============================================================================
  // Opcode Handlers - Add (Number + String coercion)
  // ============================================================================

  /// Handle Add opcode - number addition or string concatenation with auto-coercion
  #[inline]
  fn handle_add(&mut self) -> Result<(), VMError> {
    let dest = self.read_register_checked()?;
    let a = self.read_register_checked()?;
    let b = self.read_register_checked()?;

    match (&self.registers[a], &self.registers[b]) {
      (Value::Number(a_num), Value::Number(b_num)) => {
        self.registers[dest] = Value::Number(*a_num + *b_num);
      }
      (Value::String(a_str), Value::String(b_str)) => {
        let result = format!("{}{}", a_str, b_str);
        self.registers[dest] = Value::String(result.into());
      }
      (Value::String(a_str), other) => {
        let result = format!("{}{}", a_str, value_to_string(other));
        self.registers[dest] = Value::String(result.into());
      }
      (other, Value::String(b_str)) => {
        let result = format!("{}{}", value_to_string(other), b_str);
        self.registers[dest] = Value::String(result.into());
      }
      (a_val, b_val) => {
        return Err(VMError::InvalidOperation {
          operation: "add",
          left_type: a_val.type_name(),
          right_type: b_val.type_name(),
        });
      }
    }

    log_debug!(self, "Add r{} = r{} + r{}", dest, a, b);
    Ok(())
  }

  // ============================================================================
  // Opcode Handlers - Boolean Operations
  // ============================================================================

  /// Handle And opcode - boolean AND
  #[inline]
  fn handle_and(&mut self) -> Result<(), VMError> {
    let dest = self.read_register_checked()?;
    let a = self.read_register_checked()?;
    let b = self.read_register_checked()?;

    match (&self.registers[a], &self.registers[b]) {
      (Value::Boolean(a_bool), Value::Boolean(b_bool)) => {
        self.registers[dest] = Value::Boolean(*a_bool && *b_bool);
      }
      (a_val, b_val) => {
        return Err(VMError::InvalidOperation {
          operation: "and",
          left_type: a_val.type_name(),
          right_type: b_val.type_name(),
        });
      }
    }

    log_debug!(self, "And r{} = r{} && r{}", dest, a, b);
    Ok(())
  }

  /// Handle Or opcode - boolean OR
  #[inline]
  fn handle_or(&mut self) -> Result<(), VMError> {
    let dest = self.read_register_checked()?;
    let a = self.read_register_checked()?;
    let b = self.read_register_checked()?;

    match (&self.registers[a], &self.registers[b]) {
      (Value::Boolean(a_bool), Value::Boolean(b_bool)) => {
        self.registers[dest] = Value::Boolean(*a_bool || *b_bool);
      }
      (a_val, b_val) => {
        return Err(VMError::InvalidOperation {
          operation: "or",
          left_type: a_val.type_name(),
          right_type: b_val.type_name(),
        });
      }
    }

    log_debug!(self, "Or r{} = r{} || r{}", dest, a, b);
    Ok(())
  }

  /// Handle Not opcode - boolean NOT
  #[inline]
  fn handle_not(&mut self) -> Result<(), VMError> {
    let dest = self.read_register_checked()?;
    let src = self.read_register_checked()?;

    match &self.registers[src] {
      Value::Boolean(b) => {
        self.registers[dest] = Value::Boolean(!b);
      }
      v => {
        return Err(VMError::TypeMismatch {
          expected: "Boolean".to_string(),
          got: v.type_name().to_string(),
        });
      }
    }

    log_debug!(self, "Not r{} = !r{}", dest, src);
    Ok(())
  }

  /// Handle Contains opcode - membership test (value in collection)
  #[inline]
  fn handle_contains(&mut self) -> Result<(), VMError> {
    let dest = self.read_register_checked()?;
    let needle = self.read_register_checked()?;
    let haystack = self.read_register_checked()?;

    let result = match (&self.registers[needle], &self.registers[haystack]) {
      // String in StringList
      (Value::String(s), Value::StringList(list)) => list.contains(s),
      // Number in NumberList
      (Value::Number(n), Value::NumberList(list)) => list.contains(n),
      // String in String (substring check)
      (Value::String(needle_str), Value::String(haystack_str)) => {
        haystack_str.contains(needle_str.as_str())
      }
      // String in Object (key check)
      (Value::String(key), Value::Object(map)) => map.contains_key(key),
      (needle_val, haystack_val) => {
        return Err(VMError::InvalidOperation {
          operation: "in",
          left_type: needle_val.type_name(),
          right_type: haystack_val.type_name(),
        });
      }
    };

    self.registers[dest] = Value::Boolean(result);
    log_debug!(self, "Contains r{} = r{} in r{}", dest, needle, haystack);
    Ok(())
  }

  // ============================================================================
  // Opcode Handlers - Control Flow
  // ============================================================================

  /// Handle Jump opcode - unconditional jump
  #[inline]
  fn handle_jump(&mut self) -> Result<(), VMError> {
    let addr = self.read_jump_address()?;
    self.set_position(addr)?;

    log_debug!(self, "Jump to {}", addr);
    Ok(())
  }

  /// Handle JumpIfFalse opcode - conditional jump
  #[inline]
  fn handle_jump_if_false(&mut self) -> Result<(), VMError> {
    let cond_reg = self.read_register_checked()?;
    let addr = self.read_jump_address()?;

    match &self.registers[cond_reg] {
      Value::Boolean(condition) => {
        if !condition {
          self.set_position(addr)?;
          log_debug!(self, "JumpIfFalse to {} (condition=false)", addr);
        } else {
          log_debug!(self, "JumpIfFalse not taken (condition=true)");
        }
      }
      v => {
        return Err(VMError::TypeMismatch {
          expected: "Boolean".to_string(),
          got: v.type_name().to_string(),
        });
      }
    }

    Ok(())
  }

  // ============================================================================
  // Opcode Handlers - String Operations
  // ============================================================================

  /// Handle Concat opcode - string concatenation with auto-coercion
  #[inline]
  fn handle_concat(&mut self) -> Result<(), VMError> {
    let dest = self.read_register_checked()?;
    let a = self.read_register_checked()?;
    let b = self.read_register_checked()?;

    let result = format!(
      "{}{}",
      value_to_string(&self.registers[a]),
      value_to_string(&self.registers[b])
    );
    self.registers[dest] = Value::String(result.into());

    log_debug!(self, "Concat r{} = r{} + r{}", dest, a, b);
    Ok(())
  }

  /// Handle GetProperty opcode - read a field from an Object
  fn handle_get_property(&mut self) -> Result<(), VMError> {
    let dest = self.read_register_checked()?;
    let obj = self.read_register_checked()?;
    let prop = self.reader.read_string().map_err(VMError::BytecodeError)?;

    match &self.registers[obj] {
      Value::Object(map) => {
        let value = map.get(&prop).cloned().unwrap_or(Value::Null);
        self.registers[dest] = value;
      }
      other => {
        return Err(VMError::RuntimeError(format!(
          "Cannot access property '{}' on type {}",
          prop,
          other.type_name()
        )));
      }
    }
    log_debug!(self, "GetProperty r{} = r{}.{}", dest, obj, prop);
    Ok(())
  }

  /// Handle SetProperty opcode - set a field on an Object (in-place on register)
  fn handle_set_property(&mut self) -> Result<(), VMError> {
    let obj = self.read_register_checked()?;
    let prop = self.reader.read_string().map_err(VMError::BytecodeError)?;
    let val = self.read_register_checked()?;

    let value = self.registers[val].clone();
    match &mut self.registers[obj] {
      Value::Object(map) => {
        map.insert(prop.clone(), value);
      }
      other => {
        return Err(VMError::RuntimeError(format!(
          "Cannot set property '{}' on type {}",
          prop,
          other.type_name()
        )));
      }
    }
    log_debug!(self, "SetProperty r{}.{} = r{}", obj, prop, val);
    Ok(())
  }

  /// Handle MethodCall opcode - method call on object
  fn handle_method_call(&mut self) -> Result<(), VMError> {
    let dest = self.read_register_checked()?;
    let obj = self.read_register_checked()?;
    let method = self.reader.read_string().map_err(VMError::BytecodeError)?;

    // Read argument registers
    let arg_count = self
      .reader
      .read_byte()
      .map_err(VMError::BytecodeError)? as usize;
    let mut args = Vec::with_capacity(arg_count);

    for _ in 0..arg_count {
      let reg = self.read_register_checked()?;
      args.push(self.registers[reg].clone());
    }

    self.dispatch_method(dest, obj, &method, &args)?;

    log_debug!(self, "MethodCall r{} = r{}.{}(...)", dest, obj, method);
    Ok(())
  }

  // ============================================================================
  // Opcode Handlers - Built-in Functions
  // ============================================================================

  /// Handle CallDefault opcode - call a default (built-in) function by ID
  fn handle_call_default(&mut self) -> Result<(), VMError> {
    let dest = self.read_register_checked()?;
    let fn_id = self.reader.read_byte().map_err(VMError::BytecodeError)?;
    let arg_count = self.reader.read_byte().map_err(VMError::BytecodeError)? as usize;

    let mut arg_regs = Vec::with_capacity(arg_count);
    for _ in 0..arg_count {
      arg_regs.push(self.read_register_checked()?);
    }

    self.dispatch_builtin(dest, fn_id, &arg_regs)?;

    log_debug!(self, "CallDefault r{} = fn#{}(...)", dest, fn_id);
    Ok(())
  }

  /// Handle CallExternal opcode - call host function by name
  fn handle_call_external(&mut self) -> Result<(), VMError> {
    let dest = self.read_register_checked()?;
    let name = self.reader.read_string().map_err(VMError::BytecodeError)?;
    let arg_count = self.reader.read_byte().map_err(VMError::BytecodeError)? as usize;

    let mut args = Vec::with_capacity(arg_count);
    for _ in 0..arg_count {
      let reg = self.read_register_checked()?;
      args.push(self.registers[reg].clone());
    }

    let func = self
      .external_functions
      .as_ref()
      .and_then(|m| m.get(&name))
      .ok_or_else(|| VMError::RuntimeError(format!("Undefined function: {}", name)))?;

    let result = func(&args).map_err(VMError::RuntimeError)?;
    self.registers[dest] = result;

    log_debug!(self, "CallExternal r{} = {}(...)", dest, name);
    Ok(())
  }

  /// Handle SetResult opcode - store expression result
  #[inline]
  fn handle_set_result(&mut self) -> Result<(), VMError> {
    let reg = self.read_register_checked()?;
    // Take instead of clone — the register is freed right after this opcode
    self.last_result = std::mem::take(&mut self.registers[reg]);
    log_debug!(self, "SetResult = r{}", reg);
    Ok(())
  }

  /// Handle Log opcode - print value to console
  #[inline]
  fn handle_log(&mut self) -> Result<(), VMError> {
    let reg = self.read_register_checked()?;
    println!("{}", self.registers[reg]);
    log_debug!(self, "Log r{}", reg);
    Ok(())
  }

  /// Helper for binary arithmetic operations
  #[inline]
  fn binary_op<F>(&mut self, op: F, op_name: &'static str) -> Result<(), VMError>
  where
    F: FnOnce(Decimal, Decimal) -> Result<Decimal, VMError>,
  {
    let dest = self
      .reader
      .read_register()
      .map_err(|e| VMError::BytecodeError(e))? as usize;
    let a = self
      .reader
      .read_register()
      .map_err(|e| VMError::BytecodeError(e))? as usize;
    let b = self
      .reader
      .read_register()
      .map_err(|e| VMError::BytecodeError(e))? as usize;

    #[cfg(debug_assertions)]
    if dest >= MAX_REGISTERS || a >= MAX_REGISTERS || b >= MAX_REGISTERS {
      return Err(VMError::RuntimeError(format!(
        "Invalid register: dest={}, a={}, b={}",
        dest, a, b
      )));
    }

    match (&self.registers[a], &self.registers[b]) {
      (Value::Number(a_num), Value::Number(b_num)) => {
        let result = op(*a_num, *b_num)?;
        self.registers[dest] = Value::Number(result);
      }
      (a_val, b_val) => {
        return Err(VMError::InvalidOperation {
          operation: op_name,
          left_type: a_val.type_name(),
          right_type: b_val.type_name(),
        });
      }
    }

    log_debug!(self, "{} r{} = r{} {} r{}", op_name, dest, a, op_name, b);

    Ok(())
  }

  /// Helper for comparison operations
  #[inline]
  fn compare_op<F>(&mut self, op: F, op_name: &'static str) -> Result<(), VMError>
  where
    F: FnOnce(&Decimal, &Decimal) -> bool,
  {
    let dest = self
      .reader
      .read_register()
      .map_err(|e| VMError::BytecodeError(e))? as usize;
    let a = self
      .reader
      .read_register()
      .map_err(|e| VMError::BytecodeError(e))? as usize;
    let b = self
      .reader
      .read_register()
      .map_err(|e| VMError::BytecodeError(e))? as usize;

    #[cfg(debug_assertions)]
    if dest >= MAX_REGISTERS || a >= MAX_REGISTERS || b >= MAX_REGISTERS {
      return Err(VMError::RuntimeError(format!(
        "Invalid register: dest={}, a={}, b={}",
        dest, a, b
      )));
    }

    match (&self.registers[a], &self.registers[b]) {
      (Value::Number(a_num), Value::Number(b_num)) => {
        let result = op(a_num, b_num);
        self.registers[dest] = Value::Boolean(result);
      }
      (a_val, b_val) => {
        return Err(VMError::InvalidOperation {
          operation: op_name,
          left_type: a_val.type_name(),
          right_type: b_val.type_name(),
        });
      }
    }

    log_debug!(self, "{} r{} = r{} {} r{}", op_name, dest, a, op_name, b);

    Ok(())
  }
}

/// Convert a Value to its string representation for concatenation (no quotes around strings)
pub(super) fn value_to_string(val: &Value) -> std::borrow::Cow<'_, str> {
  match val {
    Value::String(s) => std::borrow::Cow::Borrowed(s.as_str()),
    Value::Number(n) => std::borrow::Cow::Owned(n.to_string()),
    Value::Boolean(b) => std::borrow::Cow::Borrowed(if *b { "true" } else { "false" }),
    Value::Null => std::borrow::Cow::Borrowed("null"),
    other => std::borrow::Cow::Owned(format!("{}", other)),
  }
}
