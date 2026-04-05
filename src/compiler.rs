use crate::ast::expr::Span;
use crate::ast::value::Value;
use crate::ast::{
  expr::{Expr, Op},
  stmt::Stmt,
};
use crate::bytecode::BytecodeWriter;
use crate::opcodes::OpCodeByte;
use crate::parser::offset_to_span;
use crate::vm::DebugInfo;
use smol_str::SmolStr;
use thiserror::Error;

/// Maximum number of registers
pub const MAX_REGISTERS: u8 = 8;

/// Compile-time error
#[derive(Error, Debug)]
pub enum CompileError {
  #[error("Undefined function: {0}")]
  UndefinedFunction(SmolStr),

  #[error("Register limit exceeded")]
  RegisterLimitExceeded,

  #[error("Invalid expression: {0}")]
  InvalidExpression(String),

  #[error("Invalid statement: {0}")]
  InvalidStatement(String),

  #[error("Bytecode error: {0}")]
  BytecodeError(String),
}

/// Compiler for dExpr language
pub struct Compiler {
  writer: BytecodeWriter,
  used_registers: Vec<bool>,
  #[cfg(debug_assertions)]
  debug: bool,

  // Jump address resolution
  pending_jumps: Vec<(usize, usize)>,
  labels: HashMap<usize, usize>,
  next_label: usize,

  // Debug info generation
  debug_info: DebugInfo,
  current_span: Span,
}

use std::collections::HashMap;

impl Compiler {
  /// Create a new compiler
  pub fn new() -> Self {
    Self {
      writer: BytecodeWriter::new(),
      used_registers: vec![false; MAX_REGISTERS as usize],
      #[cfg(debug_assertions)]
      debug: false,
      pending_jumps: Vec::new(),
      labels: HashMap::new(),
      next_label: 0,
      debug_info: DebugInfo::new(),
      current_span: Span::default(),
    }
  }

  /// Set debug mode
  #[cfg(debug_assertions)]
  pub fn set_debug(&mut self, debug: bool) {
    self.debug = debug;
  }

  /// Set debug mode (no-op in release)
  #[cfg(not(debug_assertions))]
  pub fn set_debug(&mut self, _debug: bool) {}

  /// Update current source span and emit debug info
  fn set_span(&mut self, span: Span) {
    if span != self.current_span {
      self.current_span = span;
      let offset = self.writer.position() as u32;
      self.debug_info.add_entry(offset, span);
    }
  }

  /// Get the generated debug info
  pub fn debug_info(&self) -> &DebugInfo {
    &self.debug_info
  }

  /// Take the debug info out of the compiler
  pub fn take_debug_info(&mut self) -> DebugInfo {
    std::mem::take(&mut self.debug_info)
  }

  /// Compile AST to bytecode
  pub fn compile(&mut self, statements: Vec<Stmt>) -> Result<Vec<u8>, CompileError> {
    self.reset_compiler_state();

    for stmt in &statements {
      self.compile_stmt(stmt)?;
    }

    self.emit_byte(OpCodeByte::End.to_byte());
    self.resolve_jumps()?;

    Ok(self.writer.clone().into_bytecode())
  }

  /// Compile source code with debug info for error messages
  /// Returns (bytecode, debug_info)
  pub fn compile_from_source(
    &mut self,
    source: &str,
  ) -> Result<(Vec<u8>, DebugInfo), CompileError> {
    use crate::parser;

    // Parse with position info
    let stmts_with_pos = parser::program_with_spans(source)
      .map_err(|e| CompileError::InvalidStatement(e.to_string()))?;

    self.reset_compiler_state();

    // Compile statements with span info
    for (offset, stmt) in &stmts_with_pos {
      self.set_span(offset_to_span(source, *offset));
      self.compile_stmt(stmt)?;
    }

    self.emit_byte(OpCodeByte::End.to_byte());
    self.resolve_jumps()?;

    let bytecode = self.writer.clone().into_bytecode();
    Ok((bytecode, self.take_debug_info()))
  }

  /// Reset all compiler state for a new compilation
  fn reset_compiler_state(&mut self) {
    self.writer = BytecodeWriter::new();
    self.used_registers = vec![false; MAX_REGISTERS as usize];
    self.pending_jumps.clear();
    self.labels.clear();
    self.next_label = 0;
    self.debug_info = DebugInfo::new();
    self.current_span = Span::default();
  }

  /// Compile a statement
  fn compile_stmt(&mut self, stmt: &Stmt) -> Result<(), CompileError> {
    match stmt {
      Stmt::Assignment(name, expr) => self.compile_assignment(name, expr),
      Stmt::PropertyAssignment(root, path, expr) => self.compile_property_assignment(root, path, expr),
      Stmt::ExprStmt(expr) => self.compile_expr_stmt(expr),
      Stmt::If(condition, then_branch, else_branch) => {
        self.compile_if_statement(condition, then_branch, else_branch)
      }
    }
  }

  /// Compile an assignment statement
  fn compile_assignment(
    &mut self,
    name: &SmolStr,
    expr: &Expr,
  ) -> Result<(), CompileError> {
    let expr_reg = self.compile_expr(expr)?;
    self.emit_store_global(name, expr_reg);
    self.free_register(expr_reg);
    Ok(())
  }

  /// Emit store to global variable instruction
  fn emit_store_global(&mut self, name: &SmolStr, reg: u8) {
    self.emit_byte(OpCodeByte::StoreGlobal.to_byte());
    self.emit_string(name);
    self.emit_byte(reg);
  }

  /// Compile a property assignment: `a.b.c = expr`
  /// Strategy: load root, get intermediates, set deepest, then set back up the chain, store root.
  fn compile_property_assignment(
    &mut self,
    root: &SmolStr,
    path: &[SmolStr],
    expr: &Expr,
  ) -> Result<(), CompileError> {
    // Load root object
    let root_reg = self.allocate_register()?;
    self.emit_byte(OpCodeByte::LoadGlobal.to_byte());
    self.emit_byte(root_reg);
    self.emit_string(root);

    // Get intermediate objects along the path (except last)
    let mut chain_regs = vec![root_reg];
    for field in &path[..path.len() - 1] {
      let next_reg = self.allocate_register()?;
      self.emit_byte(OpCodeByte::GetProperty.to_byte());
      self.emit_byte(next_reg);
      self.emit_byte(*chain_regs.last().unwrap());
      self.emit_string(field);
      chain_regs.push(next_reg);
    }

    // Compile the value expression
    let val_reg = self.compile_expr(expr)?;

    // Set the deepest property
    let last_field = &path[path.len() - 1];
    self.emit_byte(OpCodeByte::SetProperty.to_byte());
    self.emit_byte(*chain_regs.last().unwrap());
    self.emit_string(last_field);
    self.emit_byte(val_reg);
    self.free_register(val_reg);

    // Write back up the chain
    for i in (1..chain_regs.len()).rev() {
      let field = &path[i - 1];
      self.emit_byte(OpCodeByte::SetProperty.to_byte());
      self.emit_byte(chain_regs[i - 1]);
      self.emit_string(field);
      self.emit_byte(chain_regs[i]);
      self.free_register(chain_regs[i]);
    }

    // Store root back to global
    self.emit_store_global(root, root_reg);
    self.free_register(root_reg);
    Ok(())
  }

  /// Compile an expression statement (expression without assignment)
  fn compile_expr_stmt(&mut self, expr: &Expr) -> Result<(), CompileError> {
    let expr_reg = self.compile_expr(expr)?;
    self.emit_byte(OpCodeByte::SetResult.to_byte());
    self.emit_byte(expr_reg);
    self.free_register(expr_reg);
    Ok(())
  }

  /// Compile an if statement
  fn compile_if_statement(
    &mut self,
    condition: &Expr,
    then_branch: &[Stmt],
    else_branch: &Option<Vec<Stmt>>,
  ) -> Result<(), CompileError> {
    let cond_reg = self.compile_expr(condition)?;
    let else_label = self.create_label();
    let end_label = self.create_label();

    // Jump to else branch if condition is false
    self.emit_byte(OpCodeByte::JumpIfFalse.to_byte());
    self.emit_byte(cond_reg);
    self.emit_jump_address(else_label);
    self.free_register(cond_reg);

    // Compile then branch
    for stmt in then_branch {
      self.compile_stmt(stmt)?;
    }

    // Jump to end after then branch
    self.emit_jump(end_label);

    // Compile else branch if it exists
    self.set_label(else_label);
    if let Some(else_stmts) = else_branch {
      for stmt in else_stmts {
        self.compile_stmt(stmt)?;
      }
    }

    self.set_label(end_label);
    Ok(())
  }

  /// Compile an expression
  fn compile_expr(&mut self, expr: &Expr) -> Result<u8, CompileError> {
    match expr {
      Expr::Value(value) => self.compile_value(value),
      Expr::Variable(name) => self.compile_variable(name),
      Expr::BinaryOp(left, op, right) => self.compile_binary_op(left, op, right),
      Expr::UnaryOp(op, operand) => self.compile_unary_op(op, operand),
      Expr::FunctionCall(name, args) => self.compile_function_call(name, args),
      Expr::MethodCall(obj, method, args) => self.compile_method_call(obj, method, args),
      Expr::PropertyAccess(obj, prop) => self.compile_property_access(obj, prop),
    }
  }

  /// Compile a constant value
  fn compile_value(&mut self, value: &Value) -> Result<u8, CompileError> {
    let reg = self.allocate_register()?;
    self.emit_load_const(reg, value.clone());
    Ok(reg)
  }

  /// Compile a variable reference
  fn compile_variable(&mut self, name: &SmolStr) -> Result<u8, CompileError> {
    let reg = self.allocate_register()?;
    self.emit_byte(OpCodeByte::LoadGlobal.to_byte());
    self.emit_byte(reg);
    self.emit_string(name);
    Ok(reg)
  }

  /// Compile a binary operation
  fn compile_binary_op(
    &mut self,
    left: &Expr,
    op: &Op,
    right: &Expr,
  ) -> Result<u8, CompileError> {
    let left_reg = self.compile_expr(left)?;
    let right_reg = self.compile_expr(right)?;
    let result_reg = self.allocate_register()?;

    let opcode = match op {
      Op::Add => {
        if self.is_string_concatenation(left, right) {
          OpCodeByte::Concat
        } else {
          OpCodeByte::Add
        }
      }
      Op::Sub => OpCodeByte::Sub,
      Op::Mul => OpCodeByte::Mul,
      Op::Div => OpCodeByte::Div,
      Op::Mod => OpCodeByte::Mod,
      Op::Pow => OpCodeByte::Pow,
      Op::Lt => OpCodeByte::Lt,
      Op::Lte => OpCodeByte::Lte,
      Op::Gt => OpCodeByte::Gt,
      Op::Gte => OpCodeByte::Gte,
      Op::Eq => OpCodeByte::Eq,
      Op::Neq => OpCodeByte::Neq,
      Op::And => OpCodeByte::And,
      Op::Or => OpCodeByte::Or,
      Op::In => OpCodeByte::Contains,
      _ => {
        return Err(CompileError::InvalidExpression(format!(
          "Unsupported binary operator: {:?}",
          op
        )));
      }
    };

    self.emit_byte(opcode.to_byte());
    self.emit_byte(result_reg);
    self.emit_byte(left_reg);
    self.emit_byte(right_reg);

    self.free_register(left_reg);
    self.free_register(right_reg);

    Ok(result_reg)
  }

  /// Check if binary operation is string concatenation
  fn is_string_concatenation(&self, left: &Expr, right: &Expr) -> bool {
    matches!(left, Expr::Value(Value::String(_)))
      || matches!(right, Expr::Value(Value::String(_)))
  }

  /// Compile a unary operation
  fn compile_unary_op(&mut self, op: &Op, operand: &Expr) -> Result<u8, CompileError> {
    let operand_reg = self.compile_expr(operand)?;
    let result_reg = self.allocate_register()?;

    let opcode = match op {
      Op::Neg => OpCodeByte::Neg,
      Op::Not => OpCodeByte::Not,
      _ => {
        return Err(CompileError::InvalidExpression(format!(
          "Unsupported unary operator: {:?}",
          op
        )));
      }
    };

    self.emit_byte(opcode.to_byte());
    self.emit_byte(result_reg);
    self.emit_byte(operand_reg);

    self.free_register(operand_reg);
    Ok(result_reg)
  }

  /// Compile a function call (built-in or external)
  fn compile_function_call(
    &mut self,
    name: &SmolStr,
    args: &[Expr],
  ) -> Result<u8, CompileError> {
    let arg_regs = self.compile_arguments(args)?;
    let result_reg = self.allocate_register()?;

    if name == "log" {
      self.compile_builtin_log(&arg_regs, result_reg)?;
    } else if let Some(fn_id) = crate::opcodes::default_fn::id(name) {
      // Default (built-in) function — emit CallDefault with function ID
      self.emit_byte(OpCodeByte::CallDefault.to_byte());
      self.emit_byte(result_reg);
      self.emit_byte(fn_id);
      self.emit_byte(arg_regs.len() as u8);
      for &reg in &arg_regs {
        self.emit_byte(reg);
      }
    } else {
      // Emit CallExternal — resolved by VM at runtime
      self.emit_byte(OpCodeByte::CallExternal.to_byte());
      self.emit_byte(result_reg);
      self.emit_string(name);
      self.emit_byte(arg_regs.len() as u8);
      for &reg in &arg_regs {
        self.emit_byte(reg);
      }
    }

    // Free argument registers
    for reg in arg_regs {
      self.free_register(reg);
    }

    Ok(result_reg)
  }

  /// Compile function arguments and return their register numbers
  fn compile_arguments(&mut self, args: &[Expr]) -> Result<Vec<u8>, CompileError> {
    let mut arg_regs = Vec::with_capacity(args.len());
    for arg in args {
      let reg = self.compile_expr(arg)?;
      arg_regs.push(reg);
    }
    Ok(arg_regs)
  }

  /// Compile built-in log function
  fn compile_builtin_log(
    &mut self,
    arg_regs: &[u8],
    result_reg: u8,
  ) -> Result<(), CompileError> {
    if let Some(&arg_reg) = arg_regs.first() {
      self.emit_byte(OpCodeByte::Log.to_byte());
      self.emit_byte(arg_reg);
      self.emit_load_const(result_reg, Value::Null);
      Ok(())
    } else {
      Err(CompileError::InvalidExpression(
        "log requires an argument".to_string(),
      ))
    }
  }

  /// Compile a property access: `obj.field`
  fn compile_property_access(
    &mut self,
    obj: &Expr,
    prop: &SmolStr,
  ) -> Result<u8, CompileError> {
    let obj_reg = self.compile_expr(obj)?;
    let result_reg = self.allocate_register()?;
    self.emit_byte(OpCodeByte::GetProperty.to_byte());
    self.emit_byte(result_reg);
    self.emit_byte(obj_reg);
    self.emit_string(prop);
    self.free_register(obj_reg);
    Ok(result_reg)
  }

  /// Compile a method call
  fn compile_method_call(
    &mut self,
    obj: &Expr,
    method: &SmolStr,
    args: &[Expr],
  ) -> Result<u8, CompileError> {
    let obj_reg = self.compile_expr(obj)?;
    let arg_regs = self.compile_arguments(args)?;
    let result_reg = self.allocate_register()?;

    // Emit method call instruction
    self.emit_byte(OpCodeByte::MethodCall.to_byte());
    self.emit_byte(result_reg);
    self.emit_byte(obj_reg);
    self.emit_string(method);
    self.emit_byte(arg_regs.len() as u8);

    // Emit argument registers
    for &reg in &arg_regs {
      self.emit_byte(reg);
    }

    // Free object and argument registers
    self.free_register(obj_reg);
    for reg in arg_regs {
      self.free_register(reg);
    }

    Ok(result_reg)
  }

  /// Allocate a register
  fn allocate_register(&mut self) -> Result<u8, CompileError> {
    for i in 0..MAX_REGISTERS {
      if !self.used_registers[i as usize] {
        self.used_registers[i as usize] = true;
        return Ok(i);
      }
    }

    Err(CompileError::RegisterLimitExceeded)
  }

  /// Free a register
  fn free_register(&mut self, reg: u8) {
    if reg < MAX_REGISTERS {
      self.used_registers[reg as usize] = false;
    }
  }

  /// Create a new label
  fn create_label(&mut self) -> usize {
    let label = self.next_label;
    self.next_label += 1;
    label
  }

  /// Set a label position
  fn set_label(&mut self, label: usize) {
    let pos = self.writer.position();
    self.labels.insert(label, pos);
  }

  /// Emit a jump address placeholder to be resolved later
  fn emit_jump_address(&mut self, label: usize) -> usize {
    let pos = self.writer.position();
    self.emit_u32(0); // Placeholder
    self.pending_jumps.push((pos, label));
    pos
  }

  /// Emit a jump instruction (opcode + address)
  fn emit_jump(&mut self, label: usize) {
    self.emit_byte(OpCodeByte::Jump.to_byte());
    self.emit_jump_address(label);
  }

  /// Resolve pending jumps by filling in jump addresses
  fn resolve_jumps(&mut self) -> Result<(), CompileError> {
    let bytecode = self.writer.bytecode();
    let mut result = bytecode.to_vec();

    for (jump_pos, label) in &self.pending_jumps {
      let target_pos = self
        .labels
        .get(label)
        .ok_or_else(|| CompileError::BytecodeError(format!("Undefined label: {}", label)))?;

      self.write_u32_at_position(&mut result, *jump_pos, *target_pos as u32);
    }

    // Replace bytecode with resolved jumps
    self.writer = BytecodeWriter::new();
    for byte in result {
      self.emit_byte(byte);
    }

    Ok(())
  }

  /// Write a u32 value at a specific position in bytecode (big-endian)
  fn write_u32_at_position(&self, bytecode: &mut [u8], pos: usize, value: u32) {
    bytecode[pos] = (value >> 24) as u8;
    bytecode[pos + 1] = (value >> 16) as u8;
    bytecode[pos + 2] = (value >> 8) as u8;
    bytecode[pos + 3] = value as u8;
  }

  /// Emit a byte
  fn emit_byte(&mut self, byte: u8) {
    self.writer.write_byte(byte);
  }

  /// Emit a 32-bit integer
  fn emit_u32(&mut self, value: u32) {
    self.writer.write_u32(value);
  }

  /// Emit a string
  fn emit_string(&mut self, s: &SmolStr) {
    self.writer.write_string(s);
  }

  /// Emit a load constant instruction
  fn emit_load_const(&mut self, reg: u8, value: Value) {
    self.emit_byte(OpCodeByte::LoadConst.to_byte());
    self.emit_byte(reg);
    self.writer.write_value(&value);
  }
}
